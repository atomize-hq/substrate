//! Shared utilities for substrate components

use std::collections::HashSet;

pub mod agent_events;
pub mod fs_diff;
pub mod manager_manifest;
pub mod paths;
pub mod settings;
pub mod world_deps_manifest;

pub use agent_events::{AgentEvent, AgentEventKind};
pub use fs_diff::FsDiff;
pub use manager_manifest::{
    DetectSpec, GuestSpec, InitSpec, InstallSpec, ManagerManifest, ManagerSpec, Platform,
    RegexPattern,
};
pub use settings::WorldRootMode;
pub use world_deps_manifest::{
    WorldDepDetectSpec, WorldDepInstallRecipe, WorldDepTool, WorldDepsManifest,
};

/// Convenience re-exports for consumers that need the common substrate types.
///
/// ```
/// use substrate_common::prelude::*;
///
/// let mut diff = FsDiff::default();
/// assert!(diff.is_empty());
///
/// let redacted = redact_sensitive("token=secret");
/// assert_eq!(redacted, "token=***");
///
/// let sample = if cfg!(windows) { r"C:\\bin;C:\\bin" } else { "/bin:/bin" };
/// let deduped = dedupe_path(sample);
/// assert_eq!(deduped, dedupe_path(&deduped));
/// ```
pub mod prelude {
    pub use crate::agent_events::{AgentEvent, AgentEventKind};
    pub use crate::fs_diff::FsDiff;
    pub use crate::log_schema;
    pub use crate::manager_manifest::{
        DetectSpec, GuestSpec, InitSpec, InstallSpec, ManagerManifest, ManagerSpec, Platform,
        RegexPattern,
    };
    pub use crate::paths;
    pub use crate::settings::WorldRootMode;
    pub use crate::world_deps_manifest::{
        WorldDepDetectSpec, WorldDepInstallRecipe, WorldDepTool, WorldDepsManifest,
    };
    pub use crate::{dedupe_path, redact_sensitive};
}

/// Deduplicate PATH-like strings while preserving order
pub fn dedupe_path(path: &str) -> String {
    let separator = if cfg!(windows) { ';' } else { ':' };
    let mut seen = HashSet::new();
    let mut deduped = Vec::new();

    for component in path.split(separator) {
        if !component.is_empty() {
            let canonical = component.trim_end_matches('/').trim_end_matches('\\');
            if seen.insert(canonical.to_string()) {
                deduped.push(component);
            }
        }
    }

    deduped.join(&separator.to_string())
}

/// Standard log schema constants
pub mod log_schema {
    pub const EVENT_TYPE: &str = "event_type";
    pub const SESSION_ID: &str = "session_id";
    pub const COMMAND_ID: &str = "cmd_id";
    pub const TIMESTAMP: &str = "ts";
    pub const COMPONENT: &str = "component";
    pub const EXIT_CODE: &str = "exit_code";
    pub const DURATION_MS: &str = "duration_ms";
}

/// Redact sensitive information from command arguments
pub fn redact_sensitive(arg: &str) -> String {
    if std::env::var("SHIM_LOG_OPTS").as_deref() == Ok("raw") {
        return arg.to_string();
    }

    // Token/password patterns
    if arg.contains("token=") || arg.contains("password=") || arg.contains("SECRET=") {
        let parts: Vec<&str> = arg.splitn(2, '=').collect();
        if parts.len() == 2 {
            return format!("{}=***", parts[0]);
        }
    }

    // Flag-based redaction
    match arg {
        "--token" | "--password" | "-p" | "-H" | "--header" => "***".to_string(),
        _ => arg.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::string::string_regex;

    #[test]
    fn test_dedupe_path() {
        if cfg!(windows) {
            let path = r"C:\bin;C:\Windows;C:\bin;C:\Tools;C:\Windows";
            let result = dedupe_path(path);
            assert_eq!(result, r"C:\bin;C:\Windows;C:\Tools");
        } else {
            let path = "/usr/bin:/bin:/usr/bin:/usr/local/bin:/bin";
            let result = dedupe_path(path);
            assert_eq!(result, "/usr/bin:/bin:/usr/local/bin");
        }
    }

    #[test]
    fn test_redact_sensitive() {
        assert_eq!(redact_sensitive("normal_arg"), "normal_arg");
        assert_eq!(redact_sensitive("token=secret123"), "token=***");
        assert_eq!(redact_sensitive("--password"), "***");
    }

    proptest! {
        #[test]
        fn dedupe_path_is_idempotent(segments in proptest::collection::vec(
            string_regex(r"[A-Za-z0-9_./\\:-]{1,12}").unwrap(),
            1..6
        )) {
            let separator = if cfg!(windows) { ";" } else { ":" };
            let path = segments.join(separator);
            let once = dedupe_path(&path);
            let twice = dedupe_path(&once);

            prop_assert_eq!(once.clone(), twice);
            if !once.is_empty() {
                prop_assert!(once.split(separator).all(|part| !part.is_empty()));
            }
        }
    }
}
