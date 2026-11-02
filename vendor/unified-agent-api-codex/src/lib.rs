#![forbid(unsafe_code)]
//! Async helper around the OpenAI Codex CLI for programmatic prompting, streaming, apply/diff helpers, and server flows.
//!
//! Shells out to `codex exec`, applies sane defaults (non-interactive color handling, timeouts, model hints), and surfaces single-response, streaming, apply/diff, and MCP/app-server helpers.
//!
//! ## Setup: binary + `CODEX_HOME`
//! - Defaults pull `CODEX_BINARY` or `codex` on `PATH`; call [`CodexClientBuilder::binary`] (optionally fed by [`resolve_bundled_binary`]) to pin an app-bundled binary without touching user installs.
//! - Isolate state with [`CodexClientBuilder::codex_home`] (config/auth/history/logs live under that directory) and optionally create the layout with [`CodexClientBuilder::create_home_dirs`]. [`CodexHomeLayout`] inspects `config.toml`, `auth.json`, `.credentials.json`, `history.jsonl`, `conversations/`, and `logs/`.
//! - [`CodexHomeLayout::seed_auth_from`] copies `auth.json`/`.credentials.json` from a trusted seed home into an isolated `CODEX_HOME` without touching history/logs; use [`AuthSeedOptions`] to require files or skip missing ones.
//! - [`AuthSessionHelper`] checks `codex login status` and can launch ChatGPT or API key login flows with an app-scoped `CODEX_HOME` without mutating the parent process env.
//! - Wrapper defaults: temp working dir per call unless `working_dir` is set, `--skip-git-repo-check`, 120s timeout (use `Duration::ZERO` to disable), ANSI colors off, `RUST_LOG=error` if unset.
//! - Model defaults: `gpt-5*`/`gpt-5.1*` (including codex variants) get `model_reasoning_effort="medium"`/`model_reasoning_summary="auto"`/`model_verbosity="low"` to avoid unsupported “minimal” combos.
//!
//! ## Bundled binary (Workstream J)
//! - Apps can ship Codex inside an app-owned bundle rooted at e.g. `~/.myapp/codex-bin/<platform>/<version>/codex`; [`resolve_bundled_binary`] resolves that path without ever falling back to `PATH` or `CODEX_BINARY`. Hosts own downloads and version pins; missing bundles are hard errors.
//! - Pair bundled binaries with per-project `CODEX_HOME` roots such as `~/.myapp/codex-homes/<project>/`, optionally seeding `auth.json` + `.credentials.json` from an app-owned seed home. History/logs remain per project; the wrapper still injects `CODEX_BINARY`/`CODEX_HOME` per spawn so the parent env stays untouched.
//! - Default behavior remains unchanged until the helper is used; env/CLI defaults stay as documented above.
//!
//! ```rust,no_run
//! use codex::CodexClient;
//! # use std::time::Duration;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! std::env::set_var("CODEX_HOME", "/tmp/my-app-codex");
//! let client = CodexClient::builder()
//!     .binary("/opt/myapp/bin/codex")
//!     .model("gpt-5-codex")
//!     .timeout(Duration::from_secs(45))
//!     .build();
//! let reply = client.send_prompt("Health check").await?;
//! println!("{reply}");
//! # Ok(()) }
//! ```
//!
//! Surfaces:
//! - [`CodexClient::send_prompt`] for a single prompt/response with optional `--json` output.
//! - [`CodexClient::stream_exec`] for typed, real-time JSONL events from `codex exec --json`, returning an [`ExecStream`] with an event stream plus a completion future.
//! - [`CodexClient::apply`] / [`CodexClient::diff`] to run `codex apply <TASK_ID>` and `codex cloud diff <TASK_ID>`, echo stdout/stderr according to the builder (`mirror_stdout` / `quiet`), and return captured output + exit status.
//! - [`CodexClient::generate_app_server_bindings`] to refresh app-server protocol bindings via `codex app-server generate-ts` (optional `--prettier`) or `generate-json-schema`, returning captured stdout/stderr plus the exit status.
//! - [`CodexClient::start_app_server_proxy`] and [`CodexClient::start_exec_server`] to spawn the newer `codex app-server proxy` and `codex exec-server` helpers with piped stdio for host-managed lifecycles.
//! - [`CodexClient::run_sandbox`] to wrap `codex sandbox <platform>` (macOS/Linux/Windows), pass `--full-auto`/`--log-denials`/`--config`/`--enable`/`--disable`, and return the inner command status + output. macOS is the only platform that emits denial logs; Linux depends on the bundled `codex-linux-sandbox`; Windows sandboxing is experimental and relies on the upstream helper (no capability gating—non-zero exits bubble through).
//! - [`CodexClient::check_execpolicy`] to evaluate shell commands against Starlark execpolicy files with repeatable `--policy` flags, optional pretty JSON, and parsed decision output (allow/prompt/forbidden or noMatch).
//! - [`CodexClient::list_features`] to wrap `codex features list` with optional `--json` parsing, shared config/profile overrides, and parsed feature entries (name/stage/enabled).
//! - [`CodexClient::debug_models`], [`CodexClient::debug_prompt_input`], and the `plugin*` helpers expose the additive 0.125.0 debug/plugin command families without dropping to raw process management.
//! - [`CodexClient::start_responses_api_proxy`] to launch the `codex responses-api-proxy` helper with an API key piped via stdin plus optional port/server-info/upstream/shutdown flags.
//! - [`CodexClient::stdio_to_uds`] to spawn `codex stdio-to-uds <SOCKET_PATH>` with piped stdio so callers can bridge Unix domain sockets manually.
//!
//! ## Streaming, events, and artifacts
//! - `.json(true)` requests JSONL streaming. Expect `thread.started`/`thread.resumed`, `turn.started`/`turn.completed`/`turn.failed`, and `item.created`/`item.updated` with `item.type` such as `agent_message`, `reasoning`, `command_execution`, `file_change`, `mcp_tool_call`, `web_search`, or `todo_list` plus optional `status`/`content`/`input`. Errors surface as `{"type":"error","message":...}`.
//! - Sample payloads ship with the streaming examples (`crates/codex/examples/fixtures/*`); most examples support `--sample` for offline inspection.
//! - Disable `mirror_stdout` when parsing JSON so stdout stays under caller control; `quiet` controls stderr mirroring. `json_event_log` tees raw JSONL lines to disk before parsing; `idle_timeout`, `output_last_message`, and `output_schema` cover artifact handling.
//! - `crates/codex/examples/stream_events.rs`, `stream_last_message.rs`, `stream_with_log.rs`, and `json_stream.rs` cover typed consumption, artifact handling, log teeing, and minimal streaming.
//!
//! ## Resume + apply/diff
//! - `codex exec --json resume --last [-]` streams the same `thread/turn/item` events as `codex exec --json` but starts from an existing session (`thread.resumed`).
//! - Apply/diff require task IDs: `codex apply <TASK_ID>` applies a diff, and `codex cloud diff <TASK_ID>` prints a cloud task diff when supported by the binary.
//! - Convenience: [`CodexClient::apply`] / [`CodexClient::diff`] will append `<TASK_ID>` from `CODEX_TASK_ID` when set; otherwise they still spawn the command and return the non-zero exit status/output from the CLI.
//! - `crates/codex/examples/resume_apply.rs` shows a CLI-native resume/apply flow and ships `--sample` fixtures for offline inspection.
//!
//! ## Servers and capability detection
//! - Integrate the stdio servers via `codex mcp-server` / `codex app-server` (`crates/codex/examples/mcp_codex_flow.rs`, `mcp_codex_tool.rs`, `mcp_codex_reply.rs`, `app_server_turns.rs`, `app_server_thread_turn.rs`) to drive JSON-RPC flows, approvals, and shutdown.
//! - `probe_capabilities` and the `feature_detection` example focus on `--output-schema`, `--add-dir`, `codex login --mcp`, and `codex features list` availability; other subcommand drift (like cloud-only commands) is surfaced by the parity snapshot/reports in `cli_manifests/codex/`.
//!
//! More end-to-end flows and CLI mappings live in `crates/codex/README.md` and `crates/codex/EXAMPLES.md`.
//!
//! ## Capability/versioning surfaces (Workstream F)
//! - `probe_capabilities` captures `--version`, `features list`, and `--help` hints into a `CodexCapabilities` snapshot with `collected_at` timestamps and `BinaryFingerprint` metadata keyed by canonical binary path.
//! - Guard helpers (`guard_output_schema`, `guard_add_dir`, `guard_mcp_login`, `guard_features_list`) keep optional flags disabled when support is unknown and return operator-facing notes for unsupported features.
//! - Cache controls: `CapabilityCachePolicy::{PreferCache, Refresh, Bypass}` plus builder helpers steer cache reuse. Use `Refresh` for TTL/backoff windows or hot-swaps that reuse the same binary path; use `Bypass` when metadata is missing (FUSE/overlay filesystems) or when you need an isolated probe.
//! - TTL/backoff helper: `capability_cache_ttl_decision` inspects `collected_at` to suggest when to reuse, refresh, or bypass cached snapshots and stretches the recommended policy when metadata is missing.
//! - Overrides + persistence: `capability_snapshot`, `capability_overrides`, `write_capabilities_snapshot`, `read_capabilities_snapshot`, and `capability_snapshot_matches_binary` let hosts reuse snapshots across processes and fall back to probes when fingerprints diverge.

mod apply_diff;
mod auth;
mod builder;
mod bundled_binary;
mod cli;
mod client_core;
mod process_custody;
pub use process_custody::{ExecProcessCustody, ExecSettlementError};
mod commands;
mod defaults;
mod error;
mod events;
mod exec;
mod execpolicy;
mod home;
pub mod jsonl;
pub mod mcp;
mod process;
pub mod rollout_jsonl;
pub mod wrapper_coverage_manifest;

pub use crate::error::CodexError;
pub use apply_diff::{ApplyDiffArtifacts, CloudApplyRequest, CloudDiffRequest};
pub use auth::{AuthSessionHelper, CodexAuthMethod, CodexAuthStatus, CodexLogoutStatus};
pub use builder::{
    ApprovalPolicy, CliOverrides, CliOverridesPatch, CodexClientBuilder, ColorMode, ConfigOverride,
    FeatureToggles, FlagState, LocalProvider, ModelVerbosity, ReasoningEffort, ReasoningOverrides,
    ReasoningSummary, ReasoningSummaryFormat, SafetyOverride, SandboxMode,
};
pub use bundled_binary::{
    default_bundled_platform_label, resolve_bundled_binary, BundledBinary, BundledBinaryError,
    BundledBinarySpec,
};
pub use cli::{
    AppServerCodegenOutput, AppServerCodegenRequest, AppServerCodegenTarget, AppServerProxyRequest,
    AppServerRequest, CloudExecRequest, CloudListOutput, CloudListRequest, CloudOverviewRequest,
    CloudStatusRequest, CodexFeature, CodexFeatureStage, DebugAppServerHelpRequest,
    DebugAppServerRequest, DebugAppServerSendMessageV2Request, DebugCommandRequest,
    DebugHelpRequest, DebugModelsRequest, DebugPromptInputRequest, ExecRequest,
    ExecReviewCommandRequest, ExecServerRequest, FeaturesCommandRequest, FeaturesDisableRequest,
    FeaturesEnableRequest, FeaturesListFormat, FeaturesListOutput, FeaturesListRequest,
    ForkSessionRequest, HelpCommandRequest, HelpScope, McpAddRequest, McpAddTransport,
    McpGetRequest, McpListOutput, McpListRequest, McpLogoutRequest, McpOauthLoginRequest,
    McpOverviewRequest, McpRemoveRequest, PluginCommandRequest, PluginHelpRequest,
    PluginMarketplaceAddRequest, PluginMarketplaceCommandRequest, PluginMarketplaceHelpRequest,
    PluginMarketplaceRemoveRequest, PluginMarketplaceUpgradeRequest, ResponsesApiProxyHandle,
    ResponsesApiProxyInfo, ResponsesApiProxyRequest, ResumeSessionRequest, ReviewCommandRequest,
    SandboxCommandRequest, SandboxPlatform, SandboxRun, StdioToUdsRequest, UpdateCommandRequest,
};
pub use events::{
    CommandExecutionDelta, CommandExecutionState, EventError, FileChangeDelta, FileChangeKind,
    FileChangeState, ItemDelta, ItemDeltaPayload, ItemEnvelope, ItemFailure, ItemPayload,
    ItemSnapshot, ItemStatus, McpToolCallDelta, McpToolCallState, TextContent, TextDelta,
    ThreadEvent, ThreadStarted, TodoItem, TodoListDelta, TodoListState, ToolCallStatus,
    TurnCompleted, TurnFailed, TurnStarted, WebSearchDelta, WebSearchState, WebSearchStatus,
};
pub use exec::{
    DynExecCompletion, DynThreadEventStream, ExecCompletion, ExecStream, ExecStreamControl,
    ExecStreamError, ExecStreamRequest, ExecTerminationHandle, ResumeRequest, ResumeSelector,
};
pub use execpolicy::{
    ExecPolicyCheckRequest, ExecPolicyCheckResult, ExecPolicyDecision, ExecPolicyEvaluation,
    ExecPolicyMatch, ExecPolicyNoMatch, ExecPolicyRuleMatch,
};
pub use home::{AuthSeedError, AuthSeedOptions, AuthSeedOutcome, CodexHomeLayout};
pub use jsonl::{
    thread_event_jsonl_file, thread_event_jsonl_reader, JsonlThreadEventParser,
    ThreadEventJsonlFileReader, ThreadEventJsonlReader, ThreadEventJsonlRecord,
};
pub use rollout_jsonl::{
    find_rollout_file_by_id, find_rollout_files, rollout_jsonl_file, rollout_jsonl_reader,
    RolloutBaseInstructions, RolloutContentPart, RolloutEvent, RolloutEventMsg,
    RolloutEventMsgPayload, RolloutJsonlError, RolloutJsonlFileReader, RolloutJsonlParser,
    RolloutJsonlReader, RolloutJsonlRecord, RolloutResponseItem, RolloutResponseItemPayload,
    RolloutSessionMeta, RolloutSessionMetaPayload, RolloutUnknown,
};

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use home::CommandEnvironment;
use process::command_output_text;
use tracing::warn;

#[cfg(test)]
use tokio::time;

#[cfg(test)]
use tokio::sync::mpsc;

#[cfg(test)]
use builder::{
    cli_override_args, reasoning_config_for, DEFAULT_REASONING_CONFIG_GPT5,
    DEFAULT_REASONING_CONFIG_GPT5_1, DEFAULT_REASONING_CONFIG_GPT5_CODEX,
};

fn normalize_non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then_some(trimmed.to_string())
}

type Command = tokio::process::Command;
type ConsoleTarget = crate::process::ConsoleTarget;

#[cfg(test)]
type OsString = std::ffi::OsString;

async fn tee_stream<R>(
    reader: R,
    target: ConsoleTarget,
    mirror_console: bool,
) -> Result<Vec<u8>, std::io::Error>
where
    R: tokio::io::AsyncRead + Unpin,
{
    crate::process::tee_stream(reader, target, mirror_console).await
}

fn spawn_with_retry(
    command: &mut Command,
    binary: &std::path::Path,
) -> Result<tokio::process::Child, CodexError> {
    crate::process::spawn_with_retry(command, binary)
}

fn resolve_cli_overrides(
    builder: &CliOverrides,
    patch: &CliOverridesPatch,
    model: Option<&str>,
) -> builder::ResolvedCliOverrides {
    builder::resolve_cli_overrides(builder, patch, model)
}

fn apply_cli_overrides(
    command: &mut Command,
    resolved: &builder::ResolvedCliOverrides,
    include_search: bool,
) {
    builder::apply_cli_overrides(command, resolved, include_search);
}

#[cfg(test)]
fn bundled_binary_filename(platform: &str) -> &'static str {
    bundled_binary::bundled_binary_filename(platform)
}

mod capabilities;
mod version;
pub use capabilities::*;
pub use version::update_advisory_from_capabilities;

/// High-level client for interacting with `codex exec`.
///
/// Spawns the CLI with safe defaults (`--skip-git-repo-check`, temp working dirs unless
/// `working_dir` is set, 120s timeout unless zero, ANSI colors off, `RUST_LOG=error` if unset),
/// mirrors stdout by default, and returns whatever the CLI printed. See the crate docs for
/// streaming/log tee/server patterns and example links.
#[derive(Clone, Debug)]
pub struct CodexClient {
    command_env: CommandEnvironment,
    model: Option<String>,
    timeout: Duration,
    color_mode: ColorMode,
    working_dir: Option<PathBuf>,
    add_dirs: Vec<PathBuf>,
    images: Vec<PathBuf>,
    json_output: bool,
    output_schema: bool,
    quiet: bool,
    mirror_stdout: bool,
    json_event_log: Option<PathBuf>,
    cli_overrides: CliOverrides,
    capability_overrides: CapabilityOverrides,
    capability_cache_policy: CapabilityCachePolicy,
}

impl CodexClient {
    /// Returns a [`CodexClientBuilder`] preloaded with safe defaults.
    pub fn builder() -> CodexClientBuilder {
        CodexClientBuilder::default()
    }

    /// Returns the configured `CODEX_HOME` layout, if one was provided.
    /// This does not create any directories on disk; pair with
    /// [`CodexClientBuilder::create_home_dirs`] to control materialization.
    pub fn codex_home_layout(&self) -> Option<CodexHomeLayout> {
        self.command_env.codex_home_layout()
    }

    /// Probes the configured binary for version/build metadata and supported feature flags.
    ///
    /// Results are cached per canonical binary path and invalidated when file metadata changes.
    /// Caller-supplied overrides (see [`CodexClientBuilder::capability_overrides`]) can
    /// short-circuit probes or layer hints; snapshots are still cached against the current
    /// binary fingerprint so changes on disk trigger revalidation. Missing fingerprints skip
    /// cache reuse to force a re-probe. Cache interaction follows the policy configured on
    /// the builder (see [`CodexClientBuilder::capability_cache_policy`]).
    /// Failures are logged and return conservative defaults so callers can gate optional flags.
    pub async fn probe_capabilities(&self) -> CodexCapabilities {
        self.probe_capabilities_internal(self.capability_cache_policy, &[], None, None)
            .await
    }

    /// Probes capabilities using per-invocation environment overrides.
    ///
    /// Env overrides are applied after the wrapper's internal environment injection so the probe
    /// observes the same effective environment as `stream_*_with_env_overrides_control`.
    /// Non-empty overrides bypass the process-wide capability cache to avoid polluting cached
    /// snapshots keyed only by binary path.
    pub async fn probe_capabilities_with_env_overrides(
        &self,
        env_overrides: &BTreeMap<String, String>,
    ) -> CodexCapabilities {
        if env_overrides.is_empty() {
            return self.probe_capabilities().await;
        }

        let env_overrides: Vec<(String, String)> = env_overrides
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        self.probe_capabilities_internal(CapabilityCachePolicy::Bypass, &env_overrides, None, None)
            .await
    }

    /// Probes capabilities with an explicit cache policy.
    pub async fn probe_capabilities_with_policy(
        &self,
        cache_policy: CapabilityCachePolicy,
    ) -> CodexCapabilities {
        self.probe_capabilities_internal(cache_policy, &[], None, None)
            .await
    }

    pub(crate) async fn probe_capabilities_for_current_dir(
        &self,
        current_dir: &Path,
    ) -> CodexCapabilities {
        self.probe_capabilities_internal(self.capability_cache_policy, &[], Some(current_dir), None)
            .await
    }

    /// Probes under custody retained before startup. Probe fallback does not hide settlement
    /// failures: callers must check `custody.settle()` at the enclosing terminal boundary.
    pub async fn probe_capabilities_with_env_overrides_custody(
        &self,
        env: &BTreeMap<String, String>,
        custody: &ExecProcessCustody,
    ) -> CodexCapabilities {
        let env: Vec<_> = env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let policy = if env.is_empty() {
            self.capability_cache_policy
        } else {
            CapabilityCachePolicy::Bypass
        };
        self.probe_capabilities_internal(policy, &env, None, Some(custody))
            .await
    }

    pub(crate) async fn probe_capabilities_for_current_dir_with_custody(
        &self,
        current_dir: &Path,
        custody: Option<&ExecProcessCustody>,
    ) -> CodexCapabilities {
        self.probe_capabilities_internal(
            self.capability_cache_policy,
            &[],
            Some(current_dir),
            custody,
        )
        .await
    }

    pub(crate) async fn probe_capabilities_with_env_overrides_for_current_dir_with_custody(
        &self,
        env: &BTreeMap<String, String>,
        current_dir: &Path,
        custody: Option<&ExecProcessCustody>,
    ) -> CodexCapabilities {
        let env: Vec<_> = env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let policy = if env.is_empty() {
            self.capability_cache_policy
        } else {
            CapabilityCachePolicy::Bypass
        };
        self.probe_capabilities_internal(policy, &env, Some(current_dir), custody)
            .await
    }

    async fn probe_capabilities_internal(
        &self,
        cache_policy: CapabilityCachePolicy,
        env_overrides: &[(String, String)],
        current_dir: Option<&Path>,
        custody: Option<&ExecProcessCustody>,
    ) -> CodexCapabilities {
        let cache_key = capability_cache_key_for_current_dir_with_env(
            self.command_env.binary_path(),
            current_dir,
            env_overrides,
        );
        let fingerprint = current_fingerprint(&cache_key);
        let overrides = &self.capability_overrides;

        let cache_reads_enabled = matches!(cache_policy, CapabilityCachePolicy::PreferCache)
            && has_fingerprint_metadata(&fingerprint);
        let cache_writes_enabled = !matches!(cache_policy, CapabilityCachePolicy::Bypass)
            && has_fingerprint_metadata(&fingerprint);

        if let Some(snapshot) = overrides.snapshot.clone() {
            let capabilities = finalize_capabilities_with_overrides(
                snapshot,
                overrides,
                cache_key.clone(),
                fingerprint.clone(),
                true,
            );
            if cache_writes_enabled {
                update_capability_cache(capabilities.clone());
            }
            return capabilities;
        }

        if cache_reads_enabled {
            if let Some(cached) = cached_capabilities(&cache_key, &fingerprint) {
                if overrides.is_empty() {
                    return cached;
                }
                let merged = finalize_capabilities_with_overrides(
                    cached,
                    overrides,
                    cache_key.clone(),
                    fingerprint.clone(),
                    false,
                );
                if cache_writes_enabled {
                    update_capability_cache(merged.clone());
                }
                return merged;
            }
        }

        let probed = self
            .probe_capabilities_uncached(
                &cache_key,
                fingerprint.clone(),
                env_overrides,
                current_dir,
                custody,
            )
            .await;

        let capabilities =
            finalize_capabilities_with_overrides(probed, overrides, cache_key, fingerprint, false);

        if cache_writes_enabled {
            update_capability_cache(capabilities.clone());
        }

        capabilities
    }

    async fn probe_capabilities_uncached(
        &self,
        cache_key: &CapabilityCacheKey,
        fingerprint: Option<BinaryFingerprint>,
        env_overrides: &[(String, String)],
        current_dir: Option<&Path>,
        custody: Option<&ExecProcessCustody>,
    ) -> CodexCapabilities {
        let mut plan = CapabilityProbePlan::default();
        let mut features = CodexFeatureFlags::default();
        let mut version = None;

        plan.steps.push(CapabilityProbeStep::VersionFlag);
        match self
            .run_basic_command_with_custody(["--version"], env_overrides, current_dir, custody)
            .await
        {
            Ok(output) => {
                if !output.status.success() {
                    warn!(
                        status = ?output.status,
                        binary = ?cache_key.binary_path,
                        "codex --version exited non-zero"
                    );
                }
                let text = command_output_text(&output);
                if !text.trim().is_empty() {
                    version = Some(version::parse_version_output(&text));
                }
            }
            Err(error) => warn!(
                ?error,
                binary = ?cache_key.binary_path,
                "codex --version probe failed"
            ),
        }

        let mut parsed_features = false;

        plan.steps.push(CapabilityProbeStep::FeaturesListJson);
        match self
            .run_basic_command_with_custody(
                ["features", "list", "--json"],
                env_overrides,
                current_dir,
                custody,
            )
            .await
        {
            Ok(output) => {
                if !output.status.success() {
                    warn!(
                        status = ?output.status,
                        binary = ?cache_key.binary_path,
                        "codex features list --json exited non-zero"
                    );
                }
                if output.status.success() {
                    features.supports_features_list = true;
                }
                let text = command_output_text(&output);
                if let Some(parsed) = version::parse_features_from_json(&text) {
                    version::merge_feature_flags(&mut features, parsed);
                    parsed_features = version::detected_feature_flags(&features);
                } else if !text.is_empty() {
                    let parsed = version::parse_features_from_text(&text);
                    version::merge_feature_flags(&mut features, parsed);
                    parsed_features = version::detected_feature_flags(&features);
                }
            }
            Err(error) => warn!(
                ?error,
                binary = ?cache_key.binary_path,
                "codex features list --json probe failed"
            ),
        }

        if !parsed_features {
            plan.steps.push(CapabilityProbeStep::FeaturesListText);
            match self
                .run_basic_command_with_custody(
                    ["features", "list"],
                    env_overrides,
                    current_dir,
                    custody,
                )
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        warn!(
                            status = ?output.status,
                            binary = ?cache_key.binary_path,
                            "codex features list exited non-zero"
                        );
                    }
                    if output.status.success() {
                        features.supports_features_list = true;
                    }
                    let text = command_output_text(&output);
                    let parsed = version::parse_features_from_text(&text);
                    version::merge_feature_flags(&mut features, parsed);
                }
                Err(error) => warn!(
                    ?error,
                    binary = ?cache_key.binary_path,
                    "codex features list probe failed"
                ),
            }
        }

        if version::should_run_help_fallback(&features) {
            plan.steps.push(CapabilityProbeStep::HelpFallback);
            match self
                .run_basic_command_with_custody(["--help"], env_overrides, current_dir, custody)
                .await
            {
                Ok(output) => {
                    if !output.status.success() {
                        warn!(
                            status = ?output.status,
                            binary = ?cache_key.binary_path,
                            "codex --help exited non-zero"
                        );
                    }
                    let text = command_output_text(&output);
                    let parsed = version::parse_help_output(&text);
                    version::merge_feature_flags(&mut features, parsed);
                }
                Err(error) => warn!(
                    ?error,
                    binary = ?cache_key.binary_path,
                    "codex --help probe failed"
                ),
            }
        }

        CodexCapabilities {
            cache_key: cache_key.clone(),
            fingerprint,
            version,
            features,
            probe_plan: plan,
            collected_at: SystemTime::now(),
        }
    }

    /// Computes an update advisory by comparing the probed Codex version against
    /// caller-supplied latest releases.
    ///
    /// The crate does not fetch release metadata itself; hosts should populate
    /// [`CodexLatestReleases`] using their preferred update channel (npm,
    /// Homebrew, GitHub releases) and then call this helper. Results leverage
    /// the capability probe cache; callers with an existing
    /// [`CodexCapabilities`] snapshot can skip the probe by invoking
    /// [`update_advisory_from_capabilities`].
    pub async fn update_advisory(
        &self,
        latest_releases: &CodexLatestReleases,
    ) -> CodexUpdateAdvisory {
        let capabilities = self.probe_capabilities().await;
        update_advisory_from_capabilities(&capabilities, latest_releases)
    }
}

impl Default for CodexClient {
    fn default() -> Self {
        CodexClient::builder().build()
    }
}

#[cfg(all(test, unix))]
mod tests;
