//! Shared utilities for substrate components

use std::collections::HashSet;

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

    #[test]
    fn test_dedupe_path() {
        let path = "/usr/bin:/bin:/usr/bin:/usr/local/bin:/bin";
        let result = dedupe_path(path);
        assert_eq!(result, "/usr/bin:/bin:/usr/local/bin");
    }

    #[test]
    fn test_redact_sensitive() {
        assert_eq!(redact_sensitive("normal_arg"), "normal_arg");
        assert_eq!(redact_sensitive("token=secret123"), "token=***");
        assert_eq!(redact_sensitive("--password"), "***");
    }
}