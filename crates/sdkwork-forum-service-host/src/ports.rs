//! Outbound port composition for the Forum service host (MODULE_SPEC.md §4.2).
//!
//! Every SDKWork packaged deployment runs **one backend process against one
//! PostgreSQL**: the gateway embeds every module and their data lives in the same
//! database. The Forum host therefore reaches its sibling modules in-process, by
//! calling their service/repository crates on the host `PgPool`:
//!
//! | Port | Default (in-process) | Opt-in |
//! | --- | --- | --- |
//! | [`ForumDrivePort`] | `sdkwork-drive-workspace-service` store + permission service | none: Drive has no HTTP transport left |
//! | [`ForumSearchPort`] | `sdkwork-search-provider-postgresql` via the provider SPI | `SDKWORK_FORUM_SEARCH_MODE=http` + `SDKWORK_FORUM_SEARCH_URL` |
//!
//! The HTTP search path is a deliberate **split deployment** exception. It is
//! never the default, it is validated at startup (`search_http_adapter`), and a
//! deployment that configures nothing boots in-process and needs no endpoint or
//! service credential.
//!
//! The retired `SDKWORK_FORUM_DRIVE_URL` variable used to select a logging
//! no-op — not an HTTP call — so it is deleted rather than kept "just in case"
//! (MODULE_SPEC.md §4.2.7 dead configuration). Drive reads are now always
//! in-process, which is also the only mode that can enforce Drive's own ACL
//! before an attachment row is written.

mod drive_in_process;
mod mode;
mod search_http_adapter;
mod search_in_process;

use sqlx::PgPool;

use sdkwork_communication_forum_service::integration::drive::ForumDrivePort;
use sdkwork_communication_forum_service::integration::notifications::{
    ForumNotificationPort, HttpForumNotificationPort, LoggingForumNotificationPort,
    NoopForumNotificationPort,
};
use sdkwork_communication_forum_service::integration::search::ForumSearchPort;

use drive_in_process::InProcessForumDrivePort;
use search_in_process::InProcessForumSearchPort;

/// Build the Drive port.
///
/// Always in-process: Drive's node store and permission service are called on the
/// host pool, so the tenant scope and the `reader` ACL Drive's own node routes
/// enforced are reproduced without a socket hop.
pub fn build_drive_port(pg_pool: &PgPool) -> Box<dyn ForumDrivePort> {
    Box::new(InProcessForumDrivePort::new(pg_pool.clone()))
}

/// Build the Search port.
///
/// Defaults to the in-process Search provider. `SDKWORK_FORUM_SEARCH_MODE=http`
/// selects the split-deployment adapter, and that opt-in is validated here — at
/// startup — so a misconfigured split deployment cannot boot and then fail the
/// first topic mutation after its state already changed.
pub fn build_search_port(pg_pool: &PgPool) -> Result<Box<dyn ForumSearchPort>, String> {
    let selected = mode::search_mode()?;
    tracing::info!(transport = selected.label(), "forum search port selected");
    match selected {
        mode::SearchMode::InProcess => Ok(Box::new(InProcessForumSearchPort::new(pg_pool.clone()))),
        mode::SearchMode::Http => Ok(Box::new(search_http_adapter::http_search_port()?)),
    }
}

/// Build the notification port.
///
/// The messaging transport is outside this migration (MODULE_SPEC.md §4.2.6.2
/// lists it under a different module's debt); its configuration is unchanged.
pub fn build_notification_port() -> Box<dyn ForumNotificationPort> {
    match std::env::var("SDKWORK_FORUM_NOTIFICATION_URL") {
        Ok(url) if !url.trim().is_empty() => Box::new(HttpForumNotificationPort::new(url)),
        _ if logging_ports_enabled() => Box::new(LoggingForumNotificationPort),
        _ => Box::new(NoopForumNotificationPort),
    }
}

fn logging_ports_enabled() -> bool {
    matches!(
        std::env::var("SDKWORK_FORUM_INTEGRATION_LOG").as_deref(),
        Ok("1") | Ok("true") | Ok("TRUE")
    )
}

/// Runs a database future from a synchronous port method.
///
/// The Forum repository already bridges its synchronous trait to sqlx with
/// `block_in_place` on the gateway's multi-threaded runtime; the ports follow the
/// same rule so the in-process call reuses the host connection pool instead of
/// opening a second one (MODULE_SPEC.md §4.2.7).
///
/// A non-async caller, or a current-thread runtime that cannot park its worker,
/// gets a typed error instead of a panic.
pub(crate) fn block_on_db<F>(future: F) -> Result<F::Output, String>
where
    F: std::future::Future,
{
    let handle = tokio::runtime::Handle::try_current().map_err(|_| {
        "forum in-process port requires a Tokio runtime; the forum service runs inside the \
         gateway runtime"
            .to_string()
    })?;
    if matches!(
        handle.runtime_flavor(),
        tokio::runtime::RuntimeFlavor::CurrentThread
    ) {
        return Err(
            "forum in-process port requires the multi-threaded Tokio runtime the gateway uses; \
             it cannot block on a current-thread runtime"
                .to_string(),
        );
    }
    Ok(tokio::task::block_in_place(|| handle.block_on(future)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blocking_without_a_runtime_is_an_error_not_a_panic() {
        let error = block_on_db(async { 1_u8 }).expect_err("no runtime must not panic");
        assert!(error.contains("Tokio runtime"), "{error}");
    }

    #[test]
    fn the_composition_never_reads_a_drive_endpoint_variable() {
        // Regression guard for MODULE_SPEC.md §4.2: the retired
        // `SDKWORK_FORUM_DRIVE_URL` must not come back as a call path.
        let source = include_str!("ports.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("the module has a production section");
        assert!(!production.contains("DRIVE_URL\""));
        assert!(!production.contains("SEARCH_URL\""));
        assert!(production.contains("build_drive_port"));
    }
}
