use std::{
    collections::BTreeMap,
    env,
    ffi::OsString,
    future::Future,
    path::{Path, PathBuf},
    pin::Pin,
    process::ExitStatus,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use futures_core::Stream;
use thiserror::Error;
use tokio::{fs, io::AsyncWriteExt, process::Command, sync::Notify, time};
use tracing::debug;

use crate::{
    builder::{apply_cli_overrides, resolve_cli_overrides},
    capabilities::{guard_is_supported, log_guard_skip},
    process::{spawn_with_retry, tee_stream, ConsoleTarget},
    ApplyDiffArtifacts, CliOverridesPatch, CodexClient, CodexError, ConfigOverride, ExecRequest,
    FlagState, ResumeSessionRequest, ThreadEvent,
};

mod streaming;

#[derive(Clone)]
pub struct ExecTerminationHandle {
    inner: Arc<ExecTerminationInner>,
    custody: Option<crate::ExecProcessCustody>,
}

#[derive(Debug)]
struct ExecTerminationInner {
    requested: AtomicBool,
    notify: Notify,
}

impl ExecTerminationHandle {
    fn new() -> Self {
        Self {
            custody: None,
            inner: Arc::new(ExecTerminationInner {
                requested: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub(crate) fn with_custody(custody: crate::ExecProcessCustody) -> Self {
        Self {
            custody: Some(custody),
            ..Self::new()
        }
    }

    pub fn request_termination(&self) {
        if let Some(custody) = &self.custody {
            custody.request_termination();
        }
        if !self.inner.requested.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }

    fn is_requested(&self) -> bool {
        self.inner.requested.load(Ordering::SeqCst)
    }
}

impl std::fmt::Debug for ExecTerminationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecTerminationHandle")
            .field("requested", &self.is_requested())
            .finish()
    }
}

/// Control-capable variant of [`ExecStream`], providing a best-effort termination hook.
pub struct ExecStreamControl {
    pub events: DynThreadEventStream,
    pub completion: DynExecCompletion,
    pub termination: ExecTerminationHandle,
}

impl CodexClient {
    /// Sends `prompt` to `codex exec` and returns its stdout (the final agent message) on success.
    ///
    /// When `.json(true)` is enabled the CLI emits JSONL events (`thread.started` or
    /// `thread.resumed`, `turn.started`/`turn.completed`/`turn.failed`,
    /// `item.created`/`item.updated`, or `error`). The stream is mirrored to stdout unless
    /// `.mirror_stdout(false)`; the returned string contains the buffered lines for offline
    /// parsing. For per-event handling, see `crates/codex/examples/stream_events.rs`.
    ///
    /// ```rust,no_run
    /// use codex::CodexClient;
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = CodexClient::builder().json(true).mirror_stdout(false).build();
    /// let jsonl = client.send_prompt("Stream repo status").await?;
    /// println!("{jsonl}");
    /// # Ok(()) }
    /// ```
    pub async fn send_prompt(&self, prompt: impl AsRef<str>) -> Result<String, CodexError> {
        self.send_prompt_with(ExecRequest::new(prompt.as_ref()))
            .await
    }

    /// Sends an exec request with per-call CLI overrides.
    pub async fn send_prompt_with(&self, request: ExecRequest) -> Result<String, CodexError> {
        if request.prompt.trim().is_empty() {
            return Err(CodexError::EmptyPrompt);
        }

        self.invoke_codex_exec(request).await
    }

    /// Streams structured JSONL events from `codex exec --json`.
    ///
    /// Respects `mirror_stdout` (raw JSON echoing) and tees raw lines to `json_event_log` when
    /// configured on the builder or request. Returns an [`ExecStream`] with both the parsed event
    /// stream and a completion future that reports `--output-last-message`/schema paths.
    pub async fn stream_exec(
        &self,
        request: ExecStreamRequest,
    ) -> Result<ExecStream, ExecStreamError> {
        self.stream_exec_with_overrides(request, CliOverridesPatch::default())
            .await
    }

    /// Streams JSONL events from `codex exec --json` with per-invocation environment overrides.
    ///
    /// Env overrides are applied to the spawned `Command` for this invocation only and do not
    /// mutate the parent process environment. Overrides are applied after the wrapper's internal
    /// environment injection (`CODEX_HOME`, `CODEX_BINARY`, default `RUST_LOG`) so callers can
    /// override those keys when needed.
    pub async fn stream_exec_with_env_overrides(
        &self,
        request: ExecStreamRequest,
        env_overrides: &BTreeMap<String, String>,
    ) -> Result<ExecStream, ExecStreamError> {
        let env_overrides: Vec<(String, String)> = env_overrides
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        streaming::stream_exec_with_overrides_and_env_overrides(
            self,
            request,
            CliOverridesPatch::default(),
            &env_overrides,
        )
        .await
    }

    /// Streams JSONL events from `codex exec --json` and returns a termination handle alongside the
    /// stream and completion future.
    ///
    /// The termination handle is best-effort and idempotent; callers may request termination at any
    /// point after this returns.
    pub async fn stream_exec_with_env_overrides_control(
        &self,
        request: ExecStreamRequest,
        env_overrides: &BTreeMap<String, String>,
    ) -> Result<ExecStreamControl, ExecStreamError> {
        let env_overrides: Vec<(String, String)> = env_overrides
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        streaming::stream_exec_with_overrides_and_env_overrides_control(
            self,
            request,
            CliOverridesPatch::default(),
            &env_overrides,
        )
        .await
    }

    /// Starts exec using custody retained before this future is polled.
    /// Await `custody.settle()` after cancellation, startup error or completion before publishing
    /// an enclosing terminal result. A termination request alone does not acknowledge cleanup.
    pub async fn stream_exec_with_env_overrides_custody(
        &self,
        request: ExecStreamRequest,
        env: &BTreeMap<String, String>,
        custody: &crate::ExecProcessCustody,
    ) -> Result<ExecStreamControl, ExecStreamError> {
        let env: Vec<_> = env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        streaming::stream_exec_with_custody(
            self,
            request,
            CliOverridesPatch::default(),
            &env,
            custody,
        )
        .await
    }

    /// Starts resume under the same checked custody contract as exec.
    pub async fn stream_resume_with_env_overrides_custody(
        &self,
        request: ResumeRequest,
        env: &BTreeMap<String, String>,
        custody: &crate::ExecProcessCustody,
    ) -> Result<ExecStreamControl, ExecStreamError> {
        let env: Vec<_> = env.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        streaming::stream_resume_with_custody(self, request, &env, custody).await
    }

    /// Streams JSONL events with per-request CLI overrides.
    pub async fn stream_exec_with_overrides(
        &self,
        request: ExecStreamRequest,
        overrides: CliOverridesPatch,
    ) -> Result<ExecStream, ExecStreamError> {
        streaming::stream_exec_with_overrides(self, request, overrides).await
    }

    /// Streams structured events from `codex exec --json resume ...`.
    pub async fn stream_resume(
        &self,
        request: ResumeRequest,
    ) -> Result<ExecStream, ExecStreamError> {
        streaming::stream_resume(self, request).await
    }

    /// Streams JSONL events from `codex exec --json resume ...` and returns a termination handle
    /// alongside the stream and completion future.
    ///
    /// Env overrides are applied to the spawned `Command` for this invocation only and do not
    /// mutate the parent process environment.
    pub async fn stream_resume_with_env_overrides_control(
        &self,
        request: ResumeRequest,
        env_overrides: &BTreeMap<String, String>,
    ) -> Result<ExecStreamControl, ExecStreamError> {
        let env_overrides: Vec<(String, String)> = env_overrides
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();

        streaming::stream_resume_with_env_overrides_control(self, request, &env_overrides).await
    }

    /// Runs `codex resume [OPTIONS] [SESSION_ID] [PROMPT]` and returns captured output.
    pub async fn resume_session(
        &self,
        request: ResumeSessionRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        if matches!(request.prompt.as_deref(), Some(prompt) if prompt.trim().is_empty()) {
            return Err(CodexError::EmptyPrompt);
        }

        let mut args = vec![OsString::from("resume")];
        if request.all {
            args.push(OsString::from("--all"));
        }
        if request.include_non_interactive {
            args.push(OsString::from("--include-non-interactive"));
        }
        if request.last {
            args.push(OsString::from("--last"));
        }
        if let Some(session_id) = request.session_id {
            if !session_id.trim().is_empty() {
                args.push(OsString::from(session_id));
            }
        }
        if let Some(prompt) = request.prompt {
            if !prompt.trim().is_empty() {
                args.push(OsString::from(prompt));
            }
        }

        self.run_simple_command_with_overrides(args, request.overrides)
            .await
    }

    async fn invoke_codex_exec(&self, request: ExecRequest) -> Result<String, CodexError> {
        let ExecRequest {
            prompt,
            ephemeral,
            ignore_rules,
            ignore_user_config,
            overrides,
        } = request;
        let dir_ctx = self.directory_context()?;
        let dir_path = dir_ctx.path().to_path_buf();
        let needs_capabilities = self.output_schema || !self.add_dirs.is_empty();
        let capabilities = if needs_capabilities {
            Some(self.probe_capabilities_for_current_dir(&dir_path).await)
        } else {
            None
        };

        let resolved_overrides =
            resolve_cli_overrides(&self.cli_overrides, &overrides, self.model.as_deref());
        let mut command = Command::new(self.command_env.binary_path());
        command
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(&dir_path);

        apply_cli_overrides(&mut command, &resolved_overrides, true);
        command
            .arg("exec")
            .arg("--color")
            .arg(self.color_mode.as_str())
            .arg("--skip-git-repo-check");

        if ephemeral {
            command.arg("--ephemeral");
        }
        if ignore_rules {
            command.arg("--ignore-rules");
        }
        if ignore_user_config {
            command.arg("--ignore-user-config");
        }

        let send_prompt_via_stdin = self.json_output;
        if !send_prompt_via_stdin {
            command.arg(&prompt);
        }
        let stdin_mode = if send_prompt_via_stdin {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::null()
        };
        command.stdin(stdin_mode);

        if let Some(model) = &self.model {
            command.arg("--model").arg(model);
        }

        if let Some(capabilities) = &capabilities {
            if self.output_schema {
                let guard = capabilities.guard_output_schema();
                if guard_is_supported(&guard) {
                    command.arg("--output-schema");
                } else {
                    log_guard_skip(&guard);
                }
            }

            if !self.add_dirs.is_empty() {
                let guard = capabilities.guard_add_dir();
                if guard_is_supported(&guard) {
                    for dir in &self.add_dirs {
                        command.arg("--add-dir").arg(dir);
                    }
                } else {
                    log_guard_skip(&guard);
                }
            }
        }

        for image in &self.images {
            command.arg("--image").arg(image);
        }

        if self.json_output {
            command.arg("--json");
        }

        self.command_env.apply(&mut command)?;

        let mut child = spawn_with_retry(&mut command, self.command_env.binary_path())?;

        if send_prompt_via_stdin {
            let mut stdin = child.stdin.take().ok_or(CodexError::StdinUnavailable)?;
            if let Err(source) = stdin.write_all(prompt.as_bytes()).await {
                if source.kind() != std::io::ErrorKind::BrokenPipe {
                    return Err(CodexError::StdinWrite(source));
                }
            }
            if let Err(source) = stdin.write_all(b"\n").await {
                if source.kind() != std::io::ErrorKind::BrokenPipe {
                    return Err(CodexError::StdinWrite(source));
                }
            }
            if let Err(source) = stdin.shutdown().await {
                if source.kind() != std::io::ErrorKind::BrokenPipe {
                    return Err(CodexError::StdinWrite(source));
                }
            }
        } else {
            let _ = child.stdin.take();
        }

        let stdout = child.stdout.take().ok_or(CodexError::StdoutUnavailable)?;
        let stderr = child.stderr.take().ok_or(CodexError::StderrUnavailable)?;

        let stdout_task = tokio::spawn(tee_stream(
            stdout,
            ConsoleTarget::Stdout,
            self.mirror_stdout,
        ));
        let stderr_task = tokio::spawn(tee_stream(stderr, ConsoleTarget::Stderr, !self.quiet));

        let wait_task = async move {
            let status = child
                .wait()
                .await
                .map_err(|source| CodexError::Wait { source })?;
            let stdout_bytes = stdout_task
                .await
                .map_err(CodexError::Join)?
                .map_err(CodexError::CaptureIo)?;
            let stderr_bytes = stderr_task
                .await
                .map_err(CodexError::Join)?
                .map_err(CodexError::CaptureIo)?;
            Ok::<_, CodexError>((status, stdout_bytes, stderr_bytes))
        };

        let (status, stdout_bytes, stderr_bytes) = if self.timeout.is_zero() {
            wait_task.await?
        } else {
            match time::timeout(self.timeout, wait_task).await {
                Ok(result) => result?,
                Err(_) => {
                    return Err(CodexError::Timeout {
                        timeout: self.timeout,
                    });
                }
            }
        };

        let stderr_string = String::from_utf8(stderr_bytes).unwrap_or_default();
        if !status.success() {
            return Err(CodexError::NonZeroExit {
                status,
                stderr: stderr_string,
            });
        }

        let primary_output = if self.json_output && stdout_bytes.is_empty() {
            stderr_string
        } else {
            String::from_utf8(stdout_bytes)?
        };
        let trimmed = if self.json_output {
            primary_output
        } else {
            primary_output.trim().to_string()
        };
        debug!(
            binary = ?self.command_env.binary_path(),
            bytes = trimmed.len(),
            "received Codex output"
        );
        Ok(trimmed)
    }
}

/// Options configuring a streaming exec invocation.
#[derive(Clone, Debug)]
pub struct ExecStreamRequest {
    /// User prompt that will be forwarded to `codex exec`.
    pub prompt: String,
    /// Passes `--ephemeral` to avoid writing conversation history.
    pub ephemeral: bool,
    /// Passes `--ignore-rules` to bypass repo-specific rule loading.
    pub ignore_rules: bool,
    /// Passes `--ignore-user-config` to bypass user config loading.
    pub ignore_user_config: bool,
    /// Per-event idle timeout. If no JSON lines arrive before the duration elapses,
    /// [`ExecStreamError::IdleTimeout`] is returned.
    pub idle_timeout: Option<Duration>,
    /// Optional file path passed through to `--output-last-message`. When unset, the wrapper
    /// will request a temporary path and return it in [`ExecCompletion::last_message_path`].
    pub output_last_message: Option<PathBuf>,
    /// Optional file path passed through to `--output-schema` so clients can persist the schema
    /// describing the item envelope structure seen during the run.
    pub output_schema: Option<PathBuf>,
    /// Optional file path that receives a tee of every raw JSONL event line as it streams in.
    /// Appends to existing files, flushes each line, and creates parent directories. Overrides
    /// [`CodexClientBuilder::json_event_log`] for this request when provided.
    pub json_event_log: Option<PathBuf>,
}

/// Selector for `codex resume` targets.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResumeSelector {
    Id(String),
    Last,
    All,
}

/// Options configuring a streaming resume invocation.
#[derive(Clone, Debug)]
pub struct ResumeRequest {
    pub selector: ResumeSelector,
    pub prompt: Option<String>,
    pub ephemeral: bool,
    pub ignore_rules: bool,
    pub ignore_user_config: bool,
    pub idle_timeout: Option<Duration>,
    pub output_last_message: Option<PathBuf>,
    pub output_schema: Option<PathBuf>,
    pub json_event_log: Option<PathBuf>,
    pub overrides: CliOverridesPatch,
}

impl ResumeRequest {
    pub fn new(selector: ResumeSelector) -> Self {
        Self {
            selector,
            prompt: None,
            ephemeral: false,
            ignore_rules: false,
            ignore_user_config: false,
            idle_timeout: None,
            output_last_message: None,
            output_schema: None,
            json_event_log: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn with_id(id: impl Into<String>) -> Self {
        Self::new(ResumeSelector::Id(id.into()))
    }

    pub fn last() -> Self {
        Self::new(ResumeSelector::Last)
    }

    pub fn all() -> Self {
        Self::new(ResumeSelector::All)
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    pub fn idle_timeout(mut self, idle_timeout: Duration) -> Self {
        self.idle_timeout = Some(idle_timeout);
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

/// Ergonomic container for the streaming surface; produced by `stream_exec` (implemented in D2).
///
/// `events` yields parsed [`ThreadEvent`] values as soon as each JSONL line arrives from the CLI.
/// `completion` resolves once the Codex process exits and is the place to surface `--output-last-message`
/// and `--output-schema` paths after streaming finishes.
pub struct ExecStream {
    pub events: DynThreadEventStream,
    pub completion: DynExecCompletion,
}

/// Type-erased stream of events from the Codex CLI.
pub type DynThreadEventStream =
    Pin<Box<dyn Stream<Item = Result<ThreadEvent, ExecStreamError>> + Send>>;

/// Type-erased completion future that resolves when streaming stops.
pub type DynExecCompletion =
    Pin<Box<dyn Future<Output = Result<ExecCompletion, ExecStreamError>> + Send>>;

/// Summary returned when the codex child process exits.
#[derive(Clone, Debug)]
pub struct ExecCompletion {
    pub status: ExitStatus,
    /// Path that codex wrote when `--output-last-message` was enabled. The wrapper may eagerly
    /// read the file and populate `last_message` when feasible.
    pub last_message_path: Option<PathBuf>,
    pub last_message: Option<String>,
    /// Path to the JSON schema requested via `--output-schema`, if provided by the caller.
    pub schema_path: Option<PathBuf>,
}

/// Errors that may occur while consuming the JSONL stream.
#[derive(Debug, Error)]
pub enum ExecStreamError {
    #[error(transparent)]
    Codex(#[from] CodexError),
    #[error("failed to parse codex JSONL event: {source}: `{line}`")]
    Parse {
        line: String,
        #[source]
        source: serde_json::Error,
    },
    #[error("codex JSONL event missing required context: {message}: `{line}`")]
    Normalize { line: String, message: String },
    #[error("codex JSON stream idle for {idle_for:?}")]
    IdleTimeout { idle_for: Duration },
    #[error("codex JSON stream closed unexpectedly")]
    ChannelClosed,
}

async fn read_last_message(path: &Path) -> Option<String> {
    (fs::read_to_string(path).await).ok()
}

fn unique_temp_path(prefix: &str, extension: &str) -> PathBuf {
    let mut path = env::temp_dir();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_nanos();
    path.push(format!(
        "{prefix}{timestamp}_{}.{}",
        std::process::id(),
        extension
    ));
    path
}
