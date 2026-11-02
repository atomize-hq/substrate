use crate::{ConfigOverride, FeatureToggles};
use std::{ffi::OsString, path::PathBuf, process::ExitStatus};

/// Sandbox platform variant; maps to platform subcommands of `codex sandbox`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxPlatform {
    Macos,
    Linux,
    Windows,
}

impl SandboxPlatform {
    pub(crate) fn subcommand(self) -> &'static str {
        match self {
            SandboxPlatform::Macos => "macos",
            SandboxPlatform::Linux => "linux",
            SandboxPlatform::Windows => "windows",
        }
    }
}

/// Request to run an arbitrary command inside a Codex-provided sandbox.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SandboxCommandRequest {
    /// Target platform subcommand; maps to `macos` (alias `seatbelt`), `linux` (alias `landlock`), or `windows`.
    pub platform: SandboxPlatform,
    /// Trailing command arguments to execute. Must be non-empty to avoid the upstream CLI panic.
    pub command: Vec<OsString>,
    /// Request the workspace-write sandbox preset (`--full-auto`).
    pub full_auto: bool,
    /// Stream macOS sandbox denials after the child process exits (no-op on other platforms).
    pub log_denials: bool,
    /// Allow Unix sockets on macOS (`--allow-unix-socket`).
    pub allow_unix_socket: bool,
    /// Include Codex-managed config in the sandbox environment.
    pub include_managed_config: bool,
    /// Optional named permissions profile passed via `--permissions-profile`.
    pub permissions_profile: Option<String>,
    /// Additional `--config key=value` overrides to pass through.
    pub config_overrides: Vec<ConfigOverride>,
    /// Feature toggles forwarded to `--enable`/`--disable`.
    pub feature_toggles: FeatureToggles,
    /// Working directory for the spawned command; falls back to the builder value, then the current process directory.
    pub working_dir: Option<PathBuf>,
}

impl SandboxCommandRequest {
    pub fn new<I, S>(platform: SandboxPlatform, command: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        Self {
            platform,
            command: command.into_iter().map(Into::into).collect(),
            full_auto: false,
            log_denials: false,
            allow_unix_socket: false,
            include_managed_config: false,
            permissions_profile: None,
            config_overrides: Vec::new(),
            feature_toggles: FeatureToggles::default(),
            working_dir: None,
        }
    }

    pub fn full_auto(mut self, enable: bool) -> Self {
        self.full_auto = enable;
        self
    }

    pub fn log_denials(mut self, enable: bool) -> Self {
        self.log_denials = enable;
        self
    }

    pub fn allow_unix_socket(mut self, enable: bool) -> Self {
        self.allow_unix_socket = enable;
        self
    }

    pub fn include_managed_config(mut self, enable: bool) -> Self {
        self.include_managed_config = enable;
        self
    }

    pub fn permissions_profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.permissions_profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config_overrides.push(ConfigOverride::new(key, value));
        self
    }

    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.config_overrides.push(ConfigOverride::from_raw(raw));
        self
    }

    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.feature_toggles.enable.push(name.into());
        self
    }

    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.feature_toggles.disable.push(name.into());
        self
    }

    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}

/// Captured output from `codex sandbox <platform>`.
#[derive(Clone, Debug)]
pub struct SandboxRun {
    /// Exit status returned by the inner command (mirrors the sandbox helper).
    pub status: ExitStatus,
    /// Captured stdout (mirrored to the console when `mirror_stdout` is true).
    pub stdout: String,
    /// Captured stderr (mirrored unless `quiet` is set).
    pub stderr: String,
}
