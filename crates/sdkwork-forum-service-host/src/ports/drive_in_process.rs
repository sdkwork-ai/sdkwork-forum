//! In-process sdkwork-drive port for Forum attachments (MODULE_SPEC.md §4.2).
//!
//! Forum attachments reference Drive nodes (`drive_space_id`, `drive_node_id`,
//! optional `media_resource_id`). In every packaged deployment Drive runs in this
//! same process against this same PostgreSQL, so this adapter calls Drive's own
//! store and permission service on the host pool instead of Drive's app-api.
//!
//! # Drive API consumed (all pre-existing public surface)
//!
//! | Need | Drive API |
//! | --- | --- |
//! | tenant-scoped active node read | `DriveUploaderStore::find_active_node` (`SqlUploaderStore`) |
//! | active-space precondition | `DriveUploaderStore::find_active_space` (`SqlUploaderStore`) |
//! | effective ACL for the caller | `SqlDrivePermissionService::resolve_effective_node_access` |
//! | grant TTL bounds 30..=300 s | the Drive download-grant route's window |
//! | download-token signature + URL | `build_download_token` + Drive's canonical token path |
//!
//! No new Drive surface was added for this integration, and the adapter holds no
//! SQL of its own — every read goes through a Drive port.
//!
//! # What the Drive routes enforced, and where it is reproduced here
//!
//! The Drive node read and download-grant routes resolved the node inside the
//! caller's tenant (`dr_drive_node.tenant_id`), required the active lifecycle, and
//! required the verified subject to hold the `reader` role
//! (`acl::ensure_ctx_node_role(.., "reader")`). This adapter performs the same
//! three checks with the caller's own request context:
//!
//! - `tenant_id` comes from [`ForumRequestContext`], never from an environment
//!   default;
//! - the subject is the authenticated forum user (`principal_user_id`);
//! - the verdict is Drive's own effective-access resolver, so ownership, direct
//!   and inherited grants are decided by Drive and not re-implemented here.
//!
//! ## Share-link download cap
//!
//! Drive's grant route additionally applies
//! `collaboration_repository::enforce_share_link_download_limit_for_subject`, which
//! caps downloads for a subject holding an **active Drive share-link** permission.
//! A forum attachment grant is not that case: `ForumDrivePort` carries only the
//! authenticated forum request context and a Drive node id — there is no share-link
//! token, no anonymous subject and no share-link claim, so a forum grant cannot
//! consume a share-link download slot. Copying the cap here would create a second
//! source of truth for a Drive quota; it stays enforced by Drive for Drive
//! share-link traffic.
//!
//! ## Why the grant returns Drive's token URL
//!
//! Drive's `POST /nodes/{nodeId}/download_grants` returns a `download_url` of
//! `{download base}/download_tokens/{token}`. This adapter mints exactly that token
//! URL on this process's public ingress, so the client redeems it through Drive's
//! own route, which re-resolves the storage object and presigns it with Drive's
//! signer. Minting an object-store URL here instead would require duplicating that
//! signer (it is `pub(crate)` in Drive's route crate, not a reusable API) and would
//! hand out a URL that bypasses Drive.
//!
//! # Error mapping
//!
//! The port keeps returning `Result<_, String>`; the service maps a validation
//! failure to `ForumServiceError::Validation` exactly as before, so the HTTP status
//! for a rejected attachment is unchanged.

use sqlx::PgPool;

use sdkwork_communication_forum_service::integration::drive::ForumDrivePort;
use sdkwork_communication_forum_service::value_objects::ForumRequestContext;
use sdkwork_drive_workspace_service::application::download_service::build_download_token;
use sdkwork_drive_workspace_service::application::permission_service::{
    ResolveEffectiveNodeAccessCommand, SqlDrivePermissionService,
};
use sdkwork_drive_workspace_service::infrastructure::sql::uploader_store::SqlUploaderStore;
use sdkwork_drive_workspace_service::ports::permission_store::DriveEffectiveNodeAccess;
use sdkwork_drive_workspace_service::ports::uploader_store::{
    DriveUploaderNodeRecord, DriveUploaderStore,
};
use sdkwork_drive_workspace_service::DriveServiceError;

use super::block_on_db;

/// Forum's own published origin, used to address Drive's download-token route on
/// the shared public ingress. Empty means same-origin (a relative URL), which is
/// correct for a browser already talking to the ingress.
///
/// This is Forum's *own* published URL, not a sibling module endpoint.
const FORUM_PUBLIC_HTTP_URL_ENV: &str = "SDKWORK_FORUM_APPLICATION_PUBLIC_HTTP_URL";

/// Drive's canonical app-api path for a download token (Drive's published route).
const DRIVE_DOWNLOAD_TOKENS_PATH: &str = "/app/v3/api/drive/download_tokens";

/// Drive's download-grant default TTL, inside the 30..=300 s window the grant
/// route accepts.
pub const GRANT_TTL_SECONDS: u32 = 120;

/// Minimum TTL the Drive grant route accepts.
pub const GRANT_MIN_TTL_SECONDS: u32 = 30;

/// Maximum TTL the Drive grant route accepts.
pub const GRANT_MAX_TTL_SECONDS: u32 = 300;

/// The role Drive's node-read and download-grant routes require.
const REQUIRED_ROLE: &str = "reader";

/// The Drive subject kind for an authenticated user.
const SUBJECT_TYPE: &str = "user";

/// Drive attachment integration through Drive's public service ports on the
/// shared PostgreSQL.
pub struct InProcessForumDrivePort {
    pool: PgPool,
    download_base_url: String,
}

impl InProcessForumDrivePort {
    /// Build the port over the host pool.
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            download_base_url: forum_public_base_url(),
        }
    }

    fn uploader_store(&self) -> SqlUploaderStore {
        SqlUploaderStore::new(self.pool.clone())
    }

    /// Tenant-scoped, active-only node read.
    fn load_active_node(
        &self,
        tenant_id: &str,
        node_id: &str,
    ) -> Result<DriveUploaderNodeRecord, String> {
        let fetched = block_on_db(self.uploader_store().find_active_node(tenant_id, node_id))?;
        fetched
            .map_err(map_drive_error)?
            .ok_or_else(|| format!("drive node {node_id} was not found"))
    }

    /// Mirrors `find_active_node`'s active-space precondition.
    fn ensure_active_space(
        &self,
        tenant_id: &str,
        node: &DriveUploaderNodeRecord,
    ) -> Result<(), String> {
        let fetched = block_on_db(
            self.uploader_store()
                .find_active_space(tenant_id, &node.space_id),
        )?;
        fetched
            .map_err(map_drive_error)?
            .map(|_| ())
            .ok_or_else(|| format!("drive space {} was not found", node.space_id))
    }

    /// Reproduces `acl::ensure_ctx_node_role(.., "reader")` with Drive's own
    /// effective-access resolver.
    fn ensure_reader(
        &self,
        tenant_id: &str,
        node: &DriveUploaderNodeRecord,
        subject_id: &str,
    ) -> Result<(), String> {
        let permissions = SqlDrivePermissionService::new(self.pool.clone());
        let access = block_on_db(permissions.resolve_effective_node_access(
            ResolveEffectiveNodeAccessCommand {
                tenant_id: tenant_id.to_string(),
                space_id: node.space_id.clone(),
                node_id: node.id.clone(),
                subject_type: SUBJECT_TYPE.to_string(),
                subject_id: subject_id.to_string(),
            },
        ))?
        .map_err(map_drive_error)?;
        require_reader_access(&access, &node.id)
    }

    /// Full read check: active node, active space, caller `reader` access.
    fn require_readable_node(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<DriveUploaderNodeRecord, String> {
        let node_id = media_resource_id.trim();
        if node_id.is_empty() {
            return Err("mediaResourceId is required".to_string());
        }
        let tenant_id = require_tenant_id(ctx)?;
        let node = self.load_active_node(&tenant_id, node_id)?;
        self.ensure_active_space(&tenant_id, &node)?;
        self.ensure_reader(&tenant_id, &node, &ctx.user_id_value().to_string())?;
        Ok(node)
    }

    fn download_url(&self, token: &str) -> String {
        build_download_url(&self.download_base_url, token)
    }
}

/// Addresses Drive's canonical download-token route on the shared public ingress.
fn build_download_url(base_url: &str, token: &str) -> String {
    format!("{base_url}{DRIVE_DOWNLOAD_TOKENS_PATH}/{token}")
}

impl ForumDrivePort for InProcessForumDrivePort {
    fn validate_media_reference(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<(), String> {
        self.require_readable_node(ctx, media_resource_id)
            .map(|_| ())
    }

    fn create_download_grant(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<String, String> {
        let node = self.require_readable_node(ctx, media_resource_id)?;
        let tenant_id = require_tenant_id(ctx)?;
        let ttl_seconds = clamp_grant_ttl(GRANT_TTL_SECONDS);
        let expires_at_epoch_ms = now_epoch_ms() + i64::from(ttl_seconds) * 1_000;
        let token = build_download_token(&tenant_id, &node.id, expires_at_epoch_ms)
            .map_err(map_drive_error)?;
        Ok(self.download_url(&token))
    }
}

fn require_tenant_id(ctx: &ForumRequestContext) -> Result<String, String> {
    let tenant_id = ctx.tenant_id_value();
    if tenant_id <= 0 {
        return Err("tenantId is required for drive media access".to_string());
    }
    Ok(tenant_id.to_string())
}

/// Drive's verdict must satisfy `reader`; this is the only authorization gate
/// between forum and Drive now that the HTTP hop is gone.
fn require_reader_access(access: &DriveEffectiveNodeAccess, node_id: &str) -> Result<(), String> {
    if access.allows_role(REQUIRED_ROLE) {
        return Ok(());
    }
    Err(format!("permission denied for drive node {node_id}"))
}

/// Keeps the grant TTL inside the bounds Drive's grant route enforced.
fn clamp_grant_ttl(ttl_seconds: u32) -> u32 {
    ttl_seconds.clamp(GRANT_MIN_TTL_SECONDS, GRANT_MAX_TTL_SECONDS)
}

fn now_epoch_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

/// Forum's own published origin for this process.
fn forum_public_base_url() -> String {
    std::env::var(FORUM_PUBLIC_HTTP_URL_ENV)
        .map(|value| value.trim().trim_end_matches('/').to_string())
        .unwrap_or_default()
}

/// Maps Drive's typed service errors to the flat port error string.
///
/// The forum attachment path maps every port failure to one validation kind, so the
/// distinction only shapes the message. A not-found node keeps saying "not found";
/// permission and validation failures say "permission denied"/the Drive message;
/// everything else is an internal Drive failure.
fn map_drive_error(error: DriveServiceError) -> String {
    match error {
        DriveServiceError::NotFound(message) => {
            format!("drive media reference not found: {message}")
        }
        DriveServiceError::PermissionDenied(message) => format!("permission denied: {message}"),
        DriveServiceError::Validation(message) => message,
        DriveServiceError::Conflict(message) => format!("drive conflict: {message}"),
        DriveServiceError::Internal(message) => format!("drive unavailable: {message}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> ForumRequestContext {
        ForumRequestContext::new(100_001, 0, 42)
    }

    fn access(role: &str) -> DriveEffectiveNodeAccess {
        DriveEffectiveNodeAccess {
            role: role.to_string(),
            source: "permission".to_string(),
            permission_id: None,
            inherited: false,
            inherited_from_node_id: None,
        }
    }

    #[test]
    fn the_tenant_comes_from_the_request_context() {
        let error = require_tenant_id(&context()).expect("tenant");
        assert_eq!(error, "100001");
    }

    #[test]
    fn a_blank_tenant_is_rejected() {
        let error = require_tenant_id(&ForumRequestContext::new(0, 0, 1))
            .expect_err("a blank tenant must not reach Drive");
        assert!(error.contains("tenantId"), "{error}");
    }

    #[test]
    fn every_reader_satisfying_role_is_accepted() {
        for allowed in ["reader", "commenter", "writer", "owner"] {
            assert!(
                require_reader_access(&access(allowed), "node-1").is_ok(),
                "role {allowed:?} satisfies reader"
            );
        }
    }

    #[test]
    fn a_non_reader_subject_is_denied() {
        for denied in ["none", "unknown", ""] {
            let error = require_reader_access(&access(denied), "node-1")
                .expect_err("a non-reader must be denied");
            assert!(error.contains("node-1"), "{error}");
        }
    }

    #[test]
    fn grant_ttl_stays_inside_the_drive_route_bounds() {
        assert_eq!(clamp_grant_ttl(GRANT_TTL_SECONDS), 120);
        assert_eq!(clamp_grant_ttl(1), GRANT_MIN_TTL_SECONDS);
        assert_eq!(clamp_grant_ttl(3_600), GRANT_MAX_TTL_SECONDS);
    }

    #[test]
    fn download_url_uses_the_canonical_drive_app_api_path() {
        assert_eq!(
            build_download_url("https://forum.example.com", "dlv2_token"),
            "https://forum.example.com/app/v3/api/drive/download_tokens/dlv2_token"
        );
        // No published origin means same-origin, which is what a browser on the
        // ingress needs.
        assert_eq!(
            build_download_url("", "dlv2_token"),
            "/app/v3/api/drive/download_tokens/dlv2_token"
        );
    }

    #[test]
    fn drive_errors_reach_the_port_as_the_same_kind_of_message() {
        assert!(map_drive_error(DriveServiceError::NotFound("node".to_string()))
            .contains("not found"));
        assert!(
            map_drive_error(DriveServiceError::PermissionDenied("node".to_string()))
                .contains("permission denied")
        );
        // A validation message is passed through unchanged; the service maps the
        // port error to one validation kind either way.
        assert_eq!(
            map_drive_error(DriveServiceError::Validation("bad".to_string())),
            "bad"
        );
    }

    #[test]
    fn the_adapter_never_addresses_drive_over_http() {
        // Regression guard for MODULE_SPEC.md §4.2.
        let source = include_str!("drive_in_process.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("the module has a production section");
        assert!(!production.contains("reqwest"));
        assert!(!production.contains("ureq"));
        assert!(!production.contains("DRIVE_URL\""));
    }
}
