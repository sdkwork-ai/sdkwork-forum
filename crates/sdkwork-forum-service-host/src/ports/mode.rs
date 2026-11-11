//! How the Forum host reaches sdkwork-search (MODULE_SPEC.md §4.2).
//!
//! The default is **in-process**: the search provider SPI runs against the host
//! PostgreSQL, in this process, with no endpoint or credential. `http` is a
//! deliberate split-deployment opt-in for the case where sdkwork-search really
//! runs in another process or container.
//!
//! Absent or blank means in-process, so a deployment that configures nothing still
//! boots and serves. An unknown value is an error rather than a silent fallback: a
//! misspelled opt-in must not boot the in-process adapters and look like a working
//! split deployment.

/// Selects the transport used to reach sdkwork-search.
pub const SEARCH_MODE_ENV: &str = "SDKWORK_FORUM_SEARCH_MODE";

/// How this process reaches sdkwork-search.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Call Search's provider SPI in this process on the shared database pool.
    InProcess,
    /// Call Search's HTTP surface. Only valid for an explicit split deployment.
    Http,
}

impl SearchMode {
    /// Stable spelling used in logs and as the opt-in value.
    pub fn label(self) -> &'static str {
        match self {
            Self::InProcess => "in-process",
            Self::Http => "http",
        }
    }
}

/// Resolves the search transport from the process environment.
///
/// # Errors
///
/// Returns an error for an unrecognised non-blank value; see the module docs.
pub fn search_mode() -> Result<SearchMode, String> {
    let configured = std::env::var(SEARCH_MODE_ENV).ok();
    resolve_search_mode(configured.as_deref())
}

/// Pure mode resolution, so the environment-free rules stay unit-testable.
///
/// # Errors
///
/// See [`search_mode`].
pub fn resolve_search_mode(configured: Option<&str>) -> Result<SearchMode, String> {
    match configured.map(str::trim).filter(|value| !value.is_empty()) {
        None => Ok(SearchMode::InProcess),
        Some(raw) => match raw.to_ascii_lowercase().as_str() {
            // `in-process` is the documented spelling; the underscore and bare
            // forms are accepted so a trivial spelling difference cannot stop a
            // deployment from booting.
            "in-process" | "in_process" | "inprocess" => Ok(SearchMode::InProcess),
            "http" => Ok(SearchMode::Http),
            other => Err(format!(
                "unsupported {SEARCH_MODE_ENV} value: {other} (expected \"in-process\" or \"http\")"
            )),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::{resolve_search_mode, SearchMode, SEARCH_MODE_ENV};

    #[test]
    fn an_unset_or_blank_mode_boots_in_process() {
        for value in [None, Some(""), Some("   "), Some("\t")] {
            assert_eq!(
                resolve_search_mode(value),
                Ok(SearchMode::InProcess),
                "value {value:?} must resolve to the in-process default"
            );
        }
    }

    #[test]
    fn http_is_an_explicit_opt_in() {
        for value in ["http", "HTTP", " Http "] {
            assert_eq!(resolve_search_mode(Some(value)), Ok(SearchMode::Http));
        }
    }

    #[test]
    fn in_process_spellings_are_accepted() {
        for value in [
            "in-process",
            "IN-PROCESS",
            "in_process",
            "inprocess",
            " In-Process ",
        ] {
            assert_eq!(resolve_search_mode(Some(value)), Ok(SearchMode::InProcess));
        }
    }

    #[test]
    fn an_unknown_mode_names_the_variable_and_the_accepted_values() {
        let error = resolve_search_mode(Some("grpc"))
            .expect_err("an unknown mode must not silently fall back");
        assert!(error.contains(SEARCH_MODE_ENV), "{error}");
        assert!(error.contains("\"in-process\""), "{error}");
        assert!(error.contains("\"http\""), "{error}");
    }

    #[test]
    fn the_http_mode_is_never_the_default() {
        assert_eq!(SearchMode::InProcess.label(), "in-process");
        assert_eq!(SearchMode::Http.label(), "http");
        // Regression guard: the unset branch must stay the in-process adapters.
        let source = include_str!("mode.rs");
        assert!(
            source.contains("None => Ok(SearchMode::InProcess)"),
            "the unset mode must resolve to the in-process provider"
        );
    }
}
