use std::path::PathBuf;

use crate::{CliOverridesPatch, ConfigOverride, FlagState};

/// Request for `codex exec-server`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecServerRequest {
    /// Optional address passed via `--listen`.
    pub listen: Option<String>,
    /// Optional executor identifier passed via `--executor-id`.
    pub executor_id: Option<String>,
    /// Optional display name passed via `--name`.
    pub name: Option<String>,
    /// Optional working directory override for the spawned process.
    pub working_dir: Option<PathBuf>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl ExecServerRequest {
    pub fn new() -> Self {
        Self {
            listen: None,
            executor_id: None,
            name: None,
            working_dir: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional address passed via `--listen`.
    pub fn listen(mut self, listen: impl Into<String>) -> Self {
        let listen = listen.into();
        self.listen = (!listen.trim().is_empty()).then_some(listen);
        self
    }

    /// Sets the optional executor identifier passed via `--executor-id`.
    pub fn executor_id(mut self, executor_id: impl Into<String>) -> Self {
        let executor_id = executor_id.into();
        self.executor_id = (!executor_id.trim().is_empty()).then_some(executor_id);
        self
    }

    /// Sets the optional display name passed via `--name`.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        self.name = (!name.trim().is_empty()).then_some(name);
        self
    }

    /// Sets the working directory used to resolve relative paths.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }

    /// Adds a `--config key=value` override for this request.
    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    /// Adds a raw `--config key=value` override without validation.
    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    /// Sets the config profile (`--profile`) for this request.
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    /// Requests the CLI `--oss` flag for this call.
    pub fn oss(mut self, enable: bool) -> Self {
        self.overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    /// Adds a `--enable <feature>` toggle for this call.
    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.enable.push(name.into());
        self
    }

    /// Adds a `--disable <feature>` toggle for this call.
    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.disable.push(name.into());
        self
    }

    /// Controls whether `--search` is passed through to Codex.
    pub fn search(mut self, enable: bool) -> Self {
        self.overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }
}

impl Default for ExecServerRequest {
    fn default() -> Self {
        Self::new()
    }
}
