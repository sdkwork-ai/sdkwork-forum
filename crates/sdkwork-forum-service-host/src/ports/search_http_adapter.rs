//! Opt-in sdkwork-search HTTP adapter for a **deliberate split deployment**.
//!
//! MODULE_SPEC.md §4.2 makes the in-process provider the default. This module is
//! the only place that reads sdkwork-search's endpoint, and it is reached only when
//! `SDKWORK_FORUM_SEARCH_MODE=http` is configured — never by default.
//!
//! A split deployment is a deliberate operational choice, so its configuration is
//! validated **at startup**: a missing endpoint or service credential aborts the
//! boot instead of failing the first topic write after forum state has already
//! changed (MODULE_SPEC.md §4.2.3, §4.2.4). The exception is recorded in
//! `specs/backend-module-call-boundary.allow.json` with its reason.
//!
//! Identity is carried by the service credential: sdkwork-search resolves the
//! verified tenant/organization from it, so this adapter projects no identity
//! through request headers (AGENTS.md HTTP request header contract).

use sdkwork_communication_forum_service::integration::search::HttpForumSearchPort;

use super::mode::SEARCH_MODE_ENV;
use super::search_in_process::{forum_search_index_key, FORUM_SEARCH_INDEX_KEY_ENV};

/// Endpoint of the split-deployment sdkwork-search surface.
pub const SEARCH_URL_ENV: &str = "SDKWORK_FORUM_SEARCH_URL";

/// Service credential the sdkwork-search backend-api requires from a
/// split-deployment caller.
pub const ACCESS_TOKEN_ENV: &str = "SDKWORK_ACCESS_TOKEN";

/// Builds and validates the split-deployment adapter at startup.
///
/// # Errors
///
/// Returns an error when the opt-in endpoint or the service credential is missing,
/// naming the variable the operator has to configure — or the variable to unset in
/// order to fall back to the in-process provider.
pub fn http_search_port() -> Result<HttpForumSearchPort, String> {
    let base_url = std::env::var(SEARCH_URL_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "{SEARCH_URL_ENV} must be set to select the HTTP search adapter; unset \
                 {SEARCH_MODE_ENV} to use the default in-process provider"
            )
        })?;
    let access_token = std::env::var(ACCESS_TOKEN_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            format!(
                "{ACCESS_TOKEN_ENV} is required by the split-deployment HTTP search adapter; \
                 unset {SEARCH_MODE_ENV} to use the default in-process provider instead"
            )
        })?;
    let index_id = std::env::var(FORUM_SEARCH_INDEX_KEY_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(forum_search_index_key);
    Ok(HttpForumSearchPort::configured(
        base_url,
        index_id,
        None,
        Some(access_token),
    ))
}

#[cfg(test)]
mod tests {
    use super::{SEARCH_URL_ENV, ACCESS_TOKEN_ENV};

    #[test]
    fn the_opt_in_variables_are_named_explicitly() {
        let source = include_str!("search_http_adapter.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("the module has a production section");
        assert!(production.contains(SEARCH_URL_ENV));
        assert!(production.contains(ACCESS_TOKEN_ENV));
        assert!(production.contains("SEARCH_MODE_ENV"));
    }
}
