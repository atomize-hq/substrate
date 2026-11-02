use std::path::PathBuf;

const REASONING_CONFIG_KEYS: &[&str] = &[
    "model_reasoning_effort",
    "model_reasoning_summary",
    "model_verbosity",
    "model_reasoning_summary_format",
    "model_supports_reasoning_summaries",
];

/// ANSI color behavior for `codex exec` output.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ColorMode {
    /// Match upstream defaults: use color codes when stdout/stderr look like terminals.
    Auto,
    /// Force colorful output even when piping.
    Always,
    /// Fully disable ANSI sequences for deterministic parsing/logging (default).
    Never,
}

impl ColorMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            ColorMode::Auto => "auto",
            ColorMode::Always => "always",
            ColorMode::Never => "never",
        }
    }
}

/// Approval policy used by `--ask-for-approval`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ApprovalPolicy {
    Untrusted,
    OnFailure,
    OnRequest,
    Never,
}

impl ApprovalPolicy {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            ApprovalPolicy::Untrusted => "untrusted",
            ApprovalPolicy::OnFailure => "on-failure",
            ApprovalPolicy::OnRequest => "on-request",
            ApprovalPolicy::Never => "never",
        }
    }
}

/// Sandbox isolation level.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SandboxMode {
    ReadOnly,
    WorkspaceWrite,
    DangerFullAccess,
}

impl SandboxMode {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            SandboxMode::ReadOnly => "read-only",
            SandboxMode::WorkspaceWrite => "workspace-write",
            SandboxMode::DangerFullAccess => "danger-full-access",
        }
    }
}

/// Safety overrides that collapse approval/sandbox behavior.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SafetyOverride {
    #[default]
    Inherit,
    FullAuto,
    DangerouslyBypass,
}

/// Local provider selection for OSS backends.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalProvider {
    LmStudio,
    Ollama,
    Custom,
}

impl LocalProvider {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            LocalProvider::LmStudio => "lmstudio",
            LocalProvider::Ollama => "ollama",
            LocalProvider::Custom => "custom",
        }
    }
}

/// Three-state flag used when requests can override builder defaults.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FlagState {
    #[default]
    Inherit,
    Enable,
    Disable,
}

/// Feature toggles forwarded to `--enable/--disable`.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct FeatureToggles {
    pub enable: Vec<String>,
    pub disable: Vec<String>,
}

/// Config values for `model_reasoning_effort`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasoningEffort {
    Minimal,
    Low,
    Medium,
    High,
}

impl ReasoningEffort {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            ReasoningEffort::Minimal => "minimal",
            ReasoningEffort::Low => "low",
            ReasoningEffort::Medium => "medium",
            ReasoningEffort::High => "high",
        }
    }
}

/// Config values for `model_reasoning_summary`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasoningSummary {
    Auto,
    Concise,
    Detailed,
    None,
}

impl ReasoningSummary {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            ReasoningSummary::Auto => "auto",
            ReasoningSummary::Concise => "concise",
            ReasoningSummary::Detailed => "detailed",
            ReasoningSummary::None => "none",
        }
    }
}

/// Config values for `model_verbosity`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelVerbosity {
    Low,
    Medium,
    High,
}

impl ModelVerbosity {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            ModelVerbosity::Low => "low",
            ModelVerbosity::Medium => "medium",
            ModelVerbosity::High => "high",
        }
    }
}

/// Config values for `model_reasoning_summary_format`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReasoningSummaryFormat {
    None,
    Experimental,
}

impl ReasoningSummaryFormat {
    pub(super) const fn as_str(self) -> &'static str {
        match self {
            ReasoningSummaryFormat::None => "none",
            ReasoningSummaryFormat::Experimental => "experimental",
        }
    }
}

/// Represents a single `--config key=value` override.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigOverride {
    pub key: String,
    pub value: String,
}

impl ConfigOverride {
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    pub fn from_raw(raw: impl Into<String>) -> Self {
        let raw = raw.into();
        let (key, value) = raw
            .split_once('=')
            .map(|(key, value)| (key.to_string(), value.to_string()))
            .unwrap_or_else(|| (raw.clone(), String::new()));
        ConfigOverride { key, value }
    }

    pub(super) fn is_reasoning_key(&self) -> bool {
        REASONING_CONFIG_KEYS.contains(&self.key.as_str())
    }
}

/// Structured reasoning overrides converted into config entries.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReasoningOverrides {
    pub effort: Option<ReasoningEffort>,
    pub summary: Option<ReasoningSummary>,
    pub verbosity: Option<ModelVerbosity>,
    pub summary_format: Option<ReasoningSummaryFormat>,
    pub supports_summaries: Option<bool>,
}

impl ReasoningOverrides {
    pub(crate) fn has_overrides(&self) -> bool {
        self.effort.is_some()
            || self.summary.is_some()
            || self.verbosity.is_some()
            || self.summary_format.is_some()
            || self.supports_summaries.is_some()
    }

    pub(super) fn append_overrides(&self, configs: &mut Vec<ConfigOverride>) {
        if let Some(value) = self.effort {
            configs.push(ConfigOverride::new(
                "model_reasoning_effort",
                value.as_str(),
            ));
        }
        if let Some(value) = self.summary {
            configs.push(ConfigOverride::new(
                "model_reasoning_summary",
                value.as_str(),
            ));
        }
        if let Some(value) = self.verbosity {
            configs.push(ConfigOverride::new("model_verbosity", value.as_str()));
        }
        if let Some(value) = self.summary_format {
            configs.push(ConfigOverride::new(
                "model_reasoning_summary_format",
                value.as_str(),
            ));
        }
        if let Some(value) = self.supports_summaries {
            configs.push(ConfigOverride::new(
                "model_supports_reasoning_summaries",
                value.to_string(),
            ));
        }
    }
}

/// Builder-scoped CLI overrides.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CliOverrides {
    pub config_overrides: Vec<ConfigOverride>,
    pub feature_toggles: FeatureToggles,
    pub reasoning: ReasoningOverrides,
    pub approval_policy: Option<ApprovalPolicy>,
    pub sandbox_mode: Option<SandboxMode>,
    pub safety_override: SafetyOverride,
    pub profile: Option<String>,
    pub cd: Option<PathBuf>,
    pub remote: Option<String>,
    pub remote_auth_token_env: Option<String>,
    pub local_provider: Option<LocalProvider>,
    pub oss: FlagState,
    pub search: FlagState,
    pub auto_reasoning_defaults: bool,
}

impl Default for CliOverrides {
    fn default() -> Self {
        Self {
            config_overrides: Vec::new(),
            feature_toggles: FeatureToggles::default(),
            reasoning: ReasoningOverrides::default(),
            approval_policy: None,
            sandbox_mode: None,
            safety_override: SafetyOverride::Inherit,
            profile: None,
            cd: None,
            remote: None,
            remote_auth_token_env: None,
            local_provider: None,
            oss: FlagState::Inherit,
            search: FlagState::Inherit,
            auto_reasoning_defaults: true,
        }
    }
}

/// Request-level overlay of builder overrides.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct CliOverridesPatch {
    pub config_overrides: Vec<ConfigOverride>,
    pub feature_toggles: FeatureToggles,
    pub reasoning: ReasoningOverrides,
    pub approval_policy: Option<ApprovalPolicy>,
    pub sandbox_mode: Option<SandboxMode>,
    pub safety_override: Option<SafetyOverride>,
    pub profile: Option<String>,
    pub cd: Option<PathBuf>,
    pub remote: Option<String>,
    pub remote_auth_token_env: Option<String>,
    pub local_provider: Option<LocalProvider>,
    pub oss: FlagState,
    pub search: FlagState,
    pub auto_reasoning_defaults: Option<bool>,
}
