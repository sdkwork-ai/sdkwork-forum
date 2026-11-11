//! Drive integration port for Forum attachments.
//!
//! MODULE_SPEC.md §4.2: a packaged SDKWork deployment runs one backend process
//! against one PostgreSQL, so the Forum host reaches sdkwork-drive through Drive's
//! own service/repository crates instead of Drive's HTTP surface. The port carries
//! the [`ForumRequestContext`] so the in-process adapter can scope the Drive node
//! read to the caller's tenant and enforce Drive's own `reader` ACL for the
//! caller — the decisions the Drive node routes made from the request context.

use crate::value_objects::ForumRequestContext;

pub trait ForumDrivePort: Send + Sync {
    /// Verifies that `media_resource_id` addresses an active Drive node inside the
    /// caller's tenant that the caller may read.
    fn validate_media_reference(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<(), String>;

    /// Mints a scoped download grant (Drive's download-token URL) for the caller.
    fn create_download_grant(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<String, String>;
}

pub struct NoopForumDrivePort;

impl ForumDrivePort for NoopForumDrivePort {
    fn validate_media_reference(
        &self,
        _ctx: &ForumRequestContext,
        _media_resource_id: &str,
    ) -> Result<(), String> {
        Ok(())
    }

    fn create_download_grant(
        &self,
        _ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<String, String> {
        Ok(format!("noop-grant:{media_resource_id}"))
    }
}

/// Diagnostic port that records the request without contacting Drive.
///
/// It is not wired by the host: validation on the attachment write path must not
/// be replaced by a logging no-op (MODULE_SPEC.md §4.2.4).
pub struct LoggingForumDrivePort;

impl ForumDrivePort for LoggingForumDrivePort {
    fn validate_media_reference(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<(), String> {
        tracing::info!(
            tenant_id = ctx.tenant_id_value(),
            media_resource_id,
            "forum drive media reference validation requested"
        );
        Ok(())
    }

    fn create_download_grant(
        &self,
        ctx: &ForumRequestContext,
        media_resource_id: &str,
    ) -> Result<String, String> {
        tracing::info!(
            tenant_id = ctx.tenant_id_value(),
            media_resource_id,
            "forum drive download grant requested"
        );
        let grant_id = format!("grant:{media_resource_id}");
        Ok(grant_id)
    }
}
