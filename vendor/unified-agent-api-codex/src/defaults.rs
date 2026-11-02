use std::{env, path::PathBuf, time::Duration};

pub(crate) const DEFAULT_TIMEOUT: Duration = Duration::from_secs(120);
pub(crate) const CODEX_BINARY_ENV: &str = "CODEX_BINARY";
pub(crate) const CODEX_HOME_ENV: &str = "CODEX_HOME";
pub(crate) const RUST_LOG_ENV: &str = "RUST_LOG";
pub(crate) const DEFAULT_RUST_LOG: &str = "error";

pub(crate) fn default_rust_log_value() -> Option<&'static str> {
    env::var_os(RUST_LOG_ENV)
        .is_none()
        .then_some(DEFAULT_RUST_LOG)
}

pub(crate) fn default_binary_path() -> PathBuf {
    env::var_os(CODEX_BINARY_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("codex"))
}
