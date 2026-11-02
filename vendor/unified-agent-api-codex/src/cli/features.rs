use crate::{CliOverridesPatch, ConfigOverride, FlagState};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeMap, process::ExitStatus};

/// Stage labels reported by `codex features list`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(from = "String", into = "String")]
pub enum CodexFeatureStage {
    Experimental,
    Beta,
    Stable,
    Deprecated,
    Removed,
    Unknown(String),
}

impl CodexFeatureStage {
    pub(crate) fn parse(raw: &str) -> Self {
        let normalized = raw.trim();
        match normalized.to_ascii_lowercase().as_str() {
            "experimental" => CodexFeatureStage::Experimental,
            "beta" => CodexFeatureStage::Beta,
            "stable" => CodexFeatureStage::Stable,
            "deprecated" => CodexFeatureStage::Deprecated,
            "removed" => CodexFeatureStage::Removed,
            _ => CodexFeatureStage::Unknown(normalized.to_string()),
        }
    }

    /// Returns the normalized label for this stage.
    pub fn as_str(&self) -> &str {
        match self {
            CodexFeatureStage::Experimental => "experimental",
            CodexFeatureStage::Beta => "beta",
            CodexFeatureStage::Stable => "stable",
            CodexFeatureStage::Deprecated => "deprecated",
            CodexFeatureStage::Removed => "removed",
            CodexFeatureStage::Unknown(label) => label.as_str(),
        }
    }
}

impl From<String> for CodexFeatureStage {
    fn from(value: String) -> Self {
        CodexFeatureStage::parse(&value)
    }
}

impl From<CodexFeatureStage> for String {
    fn from(stage: CodexFeatureStage) -> Self {
        String::from(&stage)
    }
}

impl From<&CodexFeatureStage> for String {
    fn from(stage: &CodexFeatureStage) -> Self {
        stage.as_str().to_string()
    }
}

/// Single feature entry reported by `codex features list`.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct CodexFeature {
    /// Feature name as reported by the CLI.
    pub name: String,
    /// Feature stage (experimental/beta/stable/deprecated/removed) when provided.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stage: Option<CodexFeatureStage>,
    /// Whether the feature is enabled for the current config/profile.
    pub enabled: bool,
    /// Unrecognized fields from JSON output are preserved here.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl CodexFeature {
    /// Convenience helper mirroring the `enabled` flag.
    pub const fn is_enabled(&self) -> bool {
        self.enabled
    }
}

/// Format used to parse `codex features list` output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeaturesListFormat {
    Json,
    Text,
}

/// Parsed output from `codex features list`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturesListOutput {
    /// Exit status returned by the subcommand.
    pub status: ExitStatus,
    /// Captured stdout (mirrored to the console when `mirror_stdout` is true).
    pub stdout: String,
    /// Captured stderr (mirrored unless `quiet` is set).
    pub stderr: String,
    /// Parsed feature entries.
    pub features: Vec<CodexFeature>,
    /// Indicates whether JSON or text parsing was used.
    pub format: FeaturesListFormat,
}

/// Request for `codex features list`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturesListRequest {
    /// Request JSON output via `--json` (falls back to text parsing when JSON is absent).
    pub json: bool,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl FeaturesListRequest {
    /// Creates a request with JSON disabled by default for compatibility with older binaries.
    pub fn new() -> Self {
        Self {
            json: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Controls whether `--json` is passed to `codex features list`.
    pub fn json(mut self, enable: bool) -> Self {
        self.json = enable;
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

impl Default for FeaturesListRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex features`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturesCommandRequest {
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl FeaturesCommandRequest {
    pub fn new() -> Self {
        Self {
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for FeaturesCommandRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex features enable <FEATURE>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturesEnableRequest {
    /// Feature key to enable.
    pub feature: String,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl FeaturesEnableRequest {
    pub fn new(feature: impl Into<String>) -> Self {
        Self {
            feature: feature.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex features disable <FEATURE>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeaturesDisableRequest {
    /// Feature key to disable.
    pub feature: String,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl FeaturesDisableRequest {
    pub fn new(feature: impl Into<String>) -> Self {
        Self {
            feature: feature.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}
