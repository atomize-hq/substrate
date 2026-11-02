use std::{path::PathBuf, time::Duration};

#[cfg(test)]
use std::ffi::OsString;

use crate::home::CommandEnvironment;
use tokio::process::Command;

mod cli_overrides;
mod types;

pub use types::{
    ApprovalPolicy, CliOverrides, CliOverridesPatch, ColorMode, ConfigOverride, FeatureToggles,
    FlagState, LocalProvider, ModelVerbosity, ReasoningEffort, ReasoningOverrides,
    ReasoningSummary, ReasoningSummaryFormat, SafetyOverride, SandboxMode,
};

pub(super) type ResolvedCliOverrides = cli_overrides::ResolvedCliOverrides;

#[cfg(test)]
pub(super) const DEFAULT_REASONING_CONFIG_GPT5: &[(&str, &str)] =
    cli_overrides::DEFAULT_REASONING_CONFIG_GPT5;
#[cfg(test)]
pub(super) const DEFAULT_REASONING_CONFIG_GPT5_CODEX: &[(&str, &str)] =
    cli_overrides::DEFAULT_REASONING_CONFIG_GPT5_CODEX;
#[cfg(test)]
pub(super) const DEFAULT_REASONING_CONFIG_GPT5_1: &[(&str, &str)] =
    cli_overrides::DEFAULT_REASONING_CONFIG_GPT5_1;

#[cfg(test)]
pub(super) fn reasoning_config_for(
    model: Option<&str>,
) -> Option<&'static [(&'static str, &'static str)]> {
    cli_overrides::reasoning_config_for(model)
}

pub(super) fn resolve_cli_overrides(
    builder: &CliOverrides,
    patch: &CliOverridesPatch,
    model: Option<&str>,
) -> ResolvedCliOverrides {
    cli_overrides::resolve_cli_overrides(builder, patch, model)
}

#[cfg(test)]
pub(super) fn cli_override_args(
    resolved: &ResolvedCliOverrides,
    include_search: bool,
) -> Vec<OsString> {
    cli_overrides::cli_override_args(resolved, include_search)
}

pub(super) fn apply_cli_overrides(
    command: &mut Command,
    resolved: &ResolvedCliOverrides,
    include_search: bool,
) {
    cli_overrides::apply_cli_overrides(command, resolved, include_search);
}

/// Builder for [`crate::CodexClient`].
///
/// CLI parity planning and implementation history lives under `.archived/project_management/next/`
/// (see `.archived/project_management/next/codex-cli-parity/`) and the parity ADRs in `docs/adr/`.
#[derive(Clone, Debug)]
pub struct CodexClientBuilder {
    pub(super) binary: PathBuf,
    pub(super) codex_home: Option<PathBuf>,
    pub(super) create_home_dirs: bool,
    pub(super) model: Option<String>,
    pub(super) timeout: Duration,
    pub(super) color_mode: ColorMode,
    pub(super) working_dir: Option<PathBuf>,
    pub(super) add_dirs: Vec<PathBuf>,
    pub(super) images: Vec<PathBuf>,
    pub(super) json_output: bool,
    pub(super) output_schema: bool,
    pub(super) quiet: bool,
    pub(super) mirror_stdout: bool,
    pub(super) json_event_log: Option<PathBuf>,
    pub(super) cli_overrides: CliOverrides,
    pub(super) capability_overrides: crate::CapabilityOverrides,
    pub(super) capability_cache_policy: crate::CapabilityCachePolicy,
}

impl CodexClientBuilder {
    /// Starts a new builder with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the path to the Codex binary.
    ///
    /// Defaults to `CODEX_BINARY` when present or `codex` on `PATH`. Use this to pin a packaged
    /// binary, e.g. the path returned from [`crate::resolve_bundled_binary`] when your app ships Codex
    /// inside an isolated bundle.
    pub fn binary(mut self, binary: impl Into<PathBuf>) -> Self {
        self.binary = binary.into();
        self
    }

    /// Sets a custom `CODEX_HOME` path that will be applied per command.
    /// Directories are created by default; disable via [`Self::create_home_dirs`].
    pub fn codex_home(mut self, home: impl Into<PathBuf>) -> Self {
        self.codex_home = Some(home.into());
        self
    }

    /// Controls whether the CODEX_HOME directory tree should be created if missing.
    /// Defaults to `true` when [`Self::codex_home`] is set.
    pub fn create_home_dirs(mut self, enable: bool) -> Self {
        self.create_home_dirs = enable;
        self
    }

    /// Sets the model that should be used for every `codex exec` call.
    pub fn model(mut self, model: impl Into<String>) -> Self {
        let model = model.into();
        self.model = (!model.trim().is_empty()).then_some(model);
        self
    }

    /// Overrides the maximum amount of time to wait for Codex to respond.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Controls whether Codex may emit ANSI colors (`--color`). Defaults to [`ColorMode::Never`].
    pub fn color_mode(mut self, color_mode: ColorMode) -> Self {
        self.color_mode = color_mode;
        self
    }

    /// Forces Codex to run with the provided working directory instead of a fresh temp dir.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }

    /// Requests that `codex exec` include one or more `--add-dir` flags when the
    /// probed binary supports them. Unsupported or unknown capability results
    /// skip the flag to avoid CLI errors.
    pub fn add_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.add_dirs.push(path.into());
        self
    }

    /// Replaces the current add-dir list with the provided collection.
    pub fn add_dirs<I, P>(mut self, dirs: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.add_dirs = dirs.into_iter().map(Into::into).collect();
        self
    }

    /// Adds an image to the prompt by passing `--image <path>` to `codex exec`.
    pub fn image(mut self, path: impl Into<PathBuf>) -> Self {
        self.images.push(path.into());
        self
    }

    /// Replaces the current image list with the provided collection.
    pub fn images<I, P>(mut self, images: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.images = images.into_iter().map(Into::into).collect();
        self
    }

    /// Enables Codex's JSONL output mode (`--json`).
    ///
    /// Prompts are piped via stdin when enabled. Events include `thread.started`
    /// (or `thread.resumed` when continuing), `turn.started`/`turn.completed`/`turn.failed`,
    /// and `item.created`/`item.updated` with `item.type` such as `agent_message` or `reasoning`.
    /// Pair with `.mirror_stdout(false)` if you plan to parse the stream instead of just mirroring it.
    pub fn json(mut self, enable: bool) -> Self {
        self.json_output = enable;
        self
    }

    /// Requests the `--output-schema` flag when the probed binary reports
    /// support. When capability detection is inconclusive, the flag is skipped
    /// to maintain compatibility with older releases.
    pub fn output_schema(mut self, enable: bool) -> Self {
        self.output_schema = enable;
        self
    }

    /// Suppresses mirroring Codex stderr to the console.
    pub fn quiet(mut self, enable: bool) -> Self {
        self.quiet = enable;
        self
    }

    /// Controls whether Codex stdout should be mirrored to the console while
    /// also being captured. Disable this when you plan to parse JSONL output or
    /// tee the stream to a log file (see `crates/codex/examples/stream_with_log.rs`).
    pub fn mirror_stdout(mut self, enable: bool) -> Self {
        self.mirror_stdout = enable;
        self
    }

    /// Tees each JSONL event line from [`crate::CodexClient::stream_exec`] into a log file.
    /// Logs append to existing files, flush after each line, and create parent directories as
    /// needed. [`crate::ExecStreamRequest::json_event_log`] overrides this default per request.
    pub fn json_event_log(mut self, path: impl Into<PathBuf>) -> Self {
        self.json_event_log = Some(path.into());
        self
    }

    /// Adds a `--config key=value` override that will be applied to every Codex invocation.
    pub fn config_override(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.cli_overrides
            .config_overrides
            .push(ConfigOverride::new(key, value));
        self
    }

    /// Adds a preformatted `--config key=value` override without parsing the input.
    pub fn config_override_raw(mut self, raw: impl Into<String>) -> Self {
        self.cli_overrides
            .config_overrides
            .push(ConfigOverride::from_raw(raw));
        self
    }

    /// Replaces the config overrides with the provided collection.
    pub fn config_overrides<I, K, V>(mut self, overrides: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.cli_overrides.config_overrides = overrides
            .into_iter()
            .map(|(key, value)| ConfigOverride::new(key, value))
            .collect();
        self
    }

    /// Selects a Codex config profile (`--profile`).
    pub fn profile(mut self, profile: impl Into<String>) -> Self {
        let profile = profile.into();
        self.cli_overrides.profile = (!profile.trim().is_empty()).then_some(profile);
        self
    }

    /// Sets `model_reasoning_effort` via `--config`.
    pub fn reasoning_effort(mut self, effort: ReasoningEffort) -> Self {
        self.cli_overrides.reasoning.effort = Some(effort);
        self
    }

    /// Sets `model_reasoning_summary` via `--config`.
    pub fn reasoning_summary(mut self, summary: ReasoningSummary) -> Self {
        self.cli_overrides.reasoning.summary = Some(summary);
        self
    }

    /// Sets `model_verbosity` via `--config`.
    pub fn reasoning_verbosity(mut self, verbosity: ModelVerbosity) -> Self {
        self.cli_overrides.reasoning.verbosity = Some(verbosity);
        self
    }

    /// Sets `model_reasoning_summary_format` via `--config`.
    pub fn reasoning_summary_format(mut self, format: ReasoningSummaryFormat) -> Self {
        self.cli_overrides.reasoning.summary_format = Some(format);
        self
    }

    /// Sets `model_supports_reasoning_summaries` via `--config`.
    pub fn supports_reasoning_summaries(mut self, enable: bool) -> Self {
        self.cli_overrides.reasoning.supports_summaries = Some(enable);
        self
    }

    /// Controls whether GPT-5* reasoning defaults should be injected automatically.
    pub fn auto_reasoning_defaults(mut self, enable: bool) -> Self {
        self.cli_overrides.auto_reasoning_defaults = enable;
        self
    }

    /// Sets the approval policy for Codex subprocesses.
    pub fn approval_policy(mut self, policy: ApprovalPolicy) -> Self {
        self.cli_overrides.approval_policy = Some(policy);
        self
    }

    /// Sets the sandbox mode for Codex subprocesses.
    pub fn sandbox_mode(mut self, mode: SandboxMode) -> Self {
        self.cli_overrides.sandbox_mode = Some(mode);
        self
    }

    /// Applies the `--full-auto` safety override unless explicit sandbox/approval options are set.
    pub fn full_auto(mut self, enable: bool) -> Self {
        self.cli_overrides.safety_override = if enable {
            SafetyOverride::FullAuto
        } else {
            SafetyOverride::Inherit
        };
        self
    }

    /// Applies the `--dangerously-bypass-approvals-and-sandbox` override.
    pub fn dangerously_bypass_approvals_and_sandbox(mut self, enable: bool) -> Self {
        self.cli_overrides.safety_override = if enable {
            SafetyOverride::DangerouslyBypass
        } else {
            SafetyOverride::Inherit
        };
        self
    }

    /// Applies `--cd <dir>` to Codex invocations while keeping the process cwd set to `working_dir`.
    pub fn cd(mut self, dir: impl Into<PathBuf>) -> Self {
        self.cli_overrides.cd = Some(dir.into());
        self
    }

    /// Selects a remote Codex target (`--remote`).
    pub fn remote(mut self, remote: impl Into<String>) -> Self {
        let remote = remote.into();
        self.cli_overrides.remote = (!remote.trim().is_empty()).then_some(remote);
        self
    }

    /// Selects the env var that provides the remote auth token (`--remote-auth-token-env`).
    pub fn remote_auth_token_env(mut self, env_var: impl Into<String>) -> Self {
        let env_var = env_var.into();
        self.cli_overrides.remote_auth_token_env = (!env_var.trim().is_empty()).then_some(env_var);
        self
    }

    /// Selects a local provider backend (`--local-provider`).
    pub fn local_provider(mut self, provider: LocalProvider) -> Self {
        self.cli_overrides.local_provider = Some(provider);
        self
    }

    /// Requests the CLI `--oss` flag to favor OSS/local backends when available.
    pub fn oss(mut self, enable: bool) -> Self {
        self.cli_overrides.oss = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    /// Adds a `--enable <feature>` toggle to Codex invocations.
    pub fn enable_feature(mut self, name: impl Into<String>) -> Self {
        self.cli_overrides.feature_toggles.enable.push(name.into());
        self
    }

    /// Adds a `--disable <feature>` toggle to Codex invocations.
    pub fn disable_feature(mut self, name: impl Into<String>) -> Self {
        self.cli_overrides.feature_toggles.disable.push(name.into());
        self
    }

    /// Controls whether `--search` is passed through to Codex.
    pub fn search(mut self, enable: bool) -> Self {
        self.cli_overrides.search = if enable {
            FlagState::Enable
        } else {
            FlagState::Disable
        };
        self
    }

    /// Supplies manual capability data to skip probes or adjust feature flags.
    pub fn capability_overrides(mut self, overrides: crate::CapabilityOverrides) -> Self {
        self.capability_overrides = overrides;
        self
    }

    /// Convenience to apply feature overrides or vendor hints without touching versions.
    pub fn capability_feature_overrides(
        mut self,
        overrides: crate::CapabilityFeatureOverrides,
    ) -> Self {
        self.capability_overrides.features = overrides;
        self
    }

    /// Convenience to opt into specific feature flags while leaving other probes intact.
    pub fn capability_feature_hints(mut self, features: crate::CodexFeatureFlags) -> Self {
        self.capability_overrides.features = crate::CapabilityFeatureOverrides::enabling(features);
        self
    }

    /// Supplies a precomputed capability snapshot for pinned or bundled Codex builds.
    /// Combine with `write_capabilities_snapshot` / `read_capabilities_snapshot`
    /// to persist probe results between processes.
    pub fn capability_snapshot(mut self, snapshot: crate::CodexCapabilities) -> Self {
        self.capability_overrides.snapshot = Some(snapshot);
        self
    }

    /// Overrides the probed version data with caller-provided metadata.
    pub fn capability_version_override(mut self, version: crate::CodexVersionInfo) -> Self {
        self.capability_overrides.version = Some(version);
        self
    }

    /// Controls how capability probes interact with the in-process cache.
    /// Use [`crate::CapabilityCachePolicy::Refresh`] to enforce a TTL/backoff when
    /// binaries are hot-swapped without changing fingerprints.
    pub fn capability_cache_policy(mut self, policy: crate::CapabilityCachePolicy) -> Self {
        self.capability_cache_policy = policy;
        self
    }

    /// Convenience to bypass the capability cache when a fresh snapshot is required.
    /// Bypass skips cache reads and writes for the probe.
    pub fn bypass_capability_cache(mut self, bypass: bool) -> Self {
        self.capability_cache_policy = if bypass {
            crate::CapabilityCachePolicy::Bypass
        } else {
            crate::CapabilityCachePolicy::PreferCache
        };
        self
    }

    /// Builds the [`crate::CodexClient`].
    pub fn build(self) -> crate::CodexClient {
        let command_env =
            CommandEnvironment::new(self.binary, self.codex_home, self.create_home_dirs);
        crate::CodexClient {
            command_env,
            model: self.model,
            timeout: self.timeout,
            color_mode: self.color_mode,
            working_dir: self.working_dir,
            add_dirs: self.add_dirs,
            images: self.images,
            json_output: self.json_output,
            output_schema: self.output_schema,
            quiet: self.quiet,
            mirror_stdout: self.mirror_stdout,
            json_event_log: self.json_event_log,
            cli_overrides: self.cli_overrides,
            capability_overrides: self.capability_overrides,
            capability_cache_policy: self.capability_cache_policy,
        }
    }
}

impl Default for CodexClientBuilder {
    fn default() -> Self {
        Self {
            binary: crate::defaults::default_binary_path(),
            codex_home: None,
            create_home_dirs: true,
            model: None,
            timeout: crate::defaults::DEFAULT_TIMEOUT,
            color_mode: ColorMode::Never,
            working_dir: None,
            add_dirs: Vec::new(),
            images: Vec::new(),
            json_output: false,
            output_schema: false,
            quiet: false,
            mirror_stdout: true,
            json_event_log: None,
            cli_overrides: CliOverrides::default(),
            capability_overrides: crate::CapabilityOverrides::default(),
            capability_cache_policy: crate::CapabilityCachePolicy::default(),
        }
    }
}
