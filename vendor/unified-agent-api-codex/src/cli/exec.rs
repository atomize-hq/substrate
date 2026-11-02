use crate::{CliOverridesPatch, ConfigOverride, FlagState};

/// Options configuring a single exec request.
#[derive(Clone, Debug)]
pub struct ExecRequest {
    pub prompt: String,
    pub ephemeral: bool,
    pub ignore_rules: bool,
    pub ignore_user_config: bool,
    pub overrides: CliOverridesPatch,
}

impl ExecRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: prompt.into(),
            ephemeral: false,
            ignore_rules: false,
            ignore_user_config: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }

    pub fn ephemeral(mut self, enable: bool) -> Self {
        self.ephemeral = enable;
        self
    }

    pub fn ignore_rules(mut self, enable: bool) -> Self {
        self.ignore_rules = enable;
        self
    }

    pub fn ignore_user_config(mut self, enable: bool) -> Self {
        self.ignore_user_config = enable;
        self
    }

    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    pub fn oss(mut self, enable: bool) -> Self {
        self.overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.enable.push(name.into());
        self
    }

    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.overrides.feature_toggles.disable.push(name.into());
        self
    }

    pub fn search(mut self, enable: bool) -> Self {
        self.overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }
}
