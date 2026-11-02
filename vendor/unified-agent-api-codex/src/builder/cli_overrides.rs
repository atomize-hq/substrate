use std::{ffi::OsString, path::PathBuf};

use tokio::process::Command;

use super::{
    ApprovalPolicy, CliOverrides, CliOverridesPatch, ConfigOverride, FeatureToggles, FlagState,
    LocalProvider, SafetyOverride, SandboxMode,
};

pub(super) const DEFAULT_REASONING_CONFIG_GPT5: &[(&str, &str)] = &[
    ("model_reasoning_effort", "medium"),
    ("model_reasoning_summary", "auto"),
    ("model_verbosity", "low"),
];

pub(super) const DEFAULT_REASONING_CONFIG_GPT5_CODEX: &[(&str, &str)] = &[
    ("model_reasoning_effort", "medium"),
    ("model_reasoning_summary", "auto"),
    ("model_verbosity", "low"),
];

pub(super) const DEFAULT_REASONING_CONFIG_GPT5_1: &[(&str, &str)] = &[
    ("model_reasoning_effort", "medium"),
    ("model_reasoning_summary", "auto"),
    ("model_verbosity", "low"),
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ResolvedCliOverrides {
    pub(crate) config_overrides: Vec<ConfigOverride>,
    pub(crate) feature_toggles: FeatureToggles,
    pub(crate) approval_policy: Option<ApprovalPolicy>,
    pub(crate) sandbox_mode: Option<SandboxMode>,
    pub(crate) safety_override: SafetyOverride,
    pub(crate) profile: Option<String>,
    pub(crate) cd: Option<PathBuf>,
    pub(crate) remote: Option<String>,
    pub(crate) remote_auth_token_env: Option<String>,
    pub(crate) local_provider: Option<LocalProvider>,
    pub(crate) oss: bool,
    pub(crate) search: FlagState,
}

impl ResolvedCliOverrides {
    fn search_enabled(&self) -> bool {
        matches!(self.search, FlagState::Enable)
    }
}

pub(super) fn reasoning_config_for(
    model: Option<&str>,
) -> Option<&'static [(&'static str, &'static str)]> {
    let name = model.map(|value| value.to_ascii_lowercase())?;
    match name.as_str() {
        name if name.starts_with("gpt-5.1-codex") => Some(DEFAULT_REASONING_CONFIG_GPT5_1),
        name if name.starts_with("gpt-5.1") => Some(DEFAULT_REASONING_CONFIG_GPT5_1),
        "gpt-5-codex" => Some(DEFAULT_REASONING_CONFIG_GPT5_CODEX),
        name if name.starts_with("gpt-5") => Some(DEFAULT_REASONING_CONFIG_GPT5),
        _ => None,
    }
}

fn has_reasoning_config_override(overrides: &[ConfigOverride]) -> bool {
    overrides.iter().any(ConfigOverride::is_reasoning_key)
}

pub(super) fn resolve_cli_overrides(
    builder: &CliOverrides,
    patch: &CliOverridesPatch,
    model: Option<&str>,
) -> ResolvedCliOverrides {
    let auto_reasoning_defaults = patch
        .auto_reasoning_defaults
        .unwrap_or(builder.auto_reasoning_defaults);

    let has_reasoning_overrides = builder.reasoning.has_overrides()
        || patch.reasoning.has_overrides()
        || has_reasoning_config_override(&builder.config_overrides)
        || has_reasoning_config_override(&patch.config_overrides);

    let mut config_overrides = Vec::new();
    if auto_reasoning_defaults && !has_reasoning_overrides {
        if let Some(defaults) = reasoning_config_for(model) {
            for (key, value) in defaults {
                config_overrides.push(ConfigOverride::new(*key, *value));
            }
        }
    }

    config_overrides.extend(builder.config_overrides.clone());
    builder.reasoning.append_overrides(&mut config_overrides);
    config_overrides.extend(patch.config_overrides.clone());
    patch.reasoning.append_overrides(&mut config_overrides);

    let approval_policy = patch.approval_policy.or(builder.approval_policy);
    let sandbox_mode = patch.sandbox_mode.or(builder.sandbox_mode);
    let safety_override = patch.safety_override.unwrap_or(builder.safety_override);
    let profile = patch.profile.clone().or_else(|| builder.profile.clone());
    let cd = patch.cd.clone().or_else(|| builder.cd.clone());
    let remote = patch.remote.clone().or_else(|| builder.remote.clone());
    let remote_auth_token_env = patch
        .remote_auth_token_env
        .clone()
        .or_else(|| builder.remote_auth_token_env.clone());
    let local_provider = patch.local_provider.or(builder.local_provider);
    let search = match patch.search {
        FlagState::Inherit => builder.search,
        other => other,
    };
    let oss = match patch.oss {
        FlagState::Inherit => builder.oss,
        other => other,
    };
    let mut feature_toggles = builder.feature_toggles.clone();
    feature_toggles
        .enable
        .extend(patch.feature_toggles.enable.iter().cloned());
    feature_toggles
        .disable
        .extend(patch.feature_toggles.disable.iter().cloned());

    ResolvedCliOverrides {
        config_overrides,
        feature_toggles,
        approval_policy,
        sandbox_mode,
        safety_override,
        profile,
        cd,
        remote,
        remote_auth_token_env,
        local_provider,
        oss: matches!(oss, FlagState::Enable),
        search,
    }
}

pub(super) fn cli_override_args(
    resolved: &ResolvedCliOverrides,
    include_search: bool,
) -> Vec<OsString> {
    let mut args = Vec::new();
    for config in &resolved.config_overrides {
        args.push(OsString::from("--config"));
        args.push(OsString::from(format!("{}={}", config.key, config.value)));
    }

    for feature in &resolved.feature_toggles.enable {
        args.push(OsString::from("--enable"));
        args.push(OsString::from(feature));
    }

    for feature in &resolved.feature_toggles.disable {
        args.push(OsString::from("--disable"));
        args.push(OsString::from(feature));
    }

    if let Some(profile) = &resolved.profile {
        args.push(OsString::from("--profile"));
        args.push(OsString::from(profile));
    }

    match resolved.safety_override {
        SafetyOverride::DangerouslyBypass => {
            args.push(OsString::from("--dangerously-bypass-approvals-and-sandbox"));
        }
        other => {
            if let Some(policy) = resolved.approval_policy {
                args.push(OsString::from("--ask-for-approval"));
                args.push(OsString::from(policy.as_str()));
            }

            if let Some(mode) = resolved.sandbox_mode {
                args.push(OsString::from("--sandbox"));
                args.push(OsString::from(mode.as_str()));
            } else if resolved.approval_policy.is_none()
                && matches!(other, SafetyOverride::FullAuto)
            {
                args.push(OsString::from("--full-auto"));
            }
        }
    }

    if let Some(cd) = &resolved.cd {
        args.push(OsString::from("--cd"));
        args.push(cd.as_os_str().to_os_string());
    }

    if let Some(remote) = &resolved.remote {
        args.push(OsString::from("--remote"));
        args.push(OsString::from(remote));
    }

    if let Some(remote_auth_token_env) = &resolved.remote_auth_token_env {
        args.push(OsString::from("--remote-auth-token-env"));
        args.push(OsString::from(remote_auth_token_env));
    }

    if let Some(provider) = resolved.local_provider {
        args.push(OsString::from("--local-provider"));
        args.push(OsString::from(provider.as_str()));
    }

    if resolved.oss {
        args.push(OsString::from("--oss"));
    }

    if include_search && resolved.search_enabled() {
        args.push(OsString::from("--search"));
    }

    args
}

pub(super) fn apply_cli_overrides(
    command: &mut Command,
    resolved: &ResolvedCliOverrides,
    include_search: bool,
) {
    for arg in cli_override_args(resolved, include_search) {
        command.arg(arg);
    }
}
