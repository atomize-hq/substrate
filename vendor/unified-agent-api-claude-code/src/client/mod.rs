use std::{
    collections::BTreeMap,
    future::Future,
    path::PathBuf,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
    time::Duration,
};

use futures_core::Stream;
use tokio::{
    process::Command,
    sync::{mpsc, oneshot},
};

use crate::{
    builder::ClaudeClientBuilder,
    commands::command::ClaudeCommandRequest,
    commands::doctor::ClaudeDoctorRequest,
    commands::mcp::{
        McpAddFromClaudeDesktopRequest, McpAddJsonRequest, McpAddRequest, McpGetRequest,
        McpRemoveRequest, McpServeRequest,
    },
    commands::plugin::{
        PluginDisableRequest, PluginEnableRequest, PluginInstallRequest, PluginListRequest,
        PluginManifestMarketplaceRequest, PluginManifestRequest, PluginMarketplaceAddRequest,
        PluginMarketplaceListRequest, PluginMarketplaceRemoveRequest, PluginMarketplaceRepoRequest,
        PluginMarketplaceRequest, PluginMarketplaceUpdateRequest, PluginRequest,
        PluginUninstallRequest, PluginUpdateRequest, PluginValidateRequest,
    },
    commands::print::{ClaudeOutputFormat, ClaudePrintRequest},
    commands::update::ClaudeUpdateRequest,
    home::{ClaudeHomeLayout, ClaudeHomeSeedRequest},
    parse_stream_json_lines, process, ClaudeCodeError, ClaudePrintStreamJsonControlHandle,
    ClaudePrintStreamJsonHandle, ClaudeStreamJsonEvent, ClaudeStreamJsonParseError,
    ClaudeTerminationHandle, CommandOutput, DynClaudeStreamJsonCompletion,
    DynClaudeStreamJsonEventStream, StreamJsonLineOutcome,
};

mod setup_token;
mod streaming_custody;

pub use setup_token::ClaudeSetupTokenSession;

#[derive(Debug, Clone)]
pub struct ClaudeClient {
    pub(crate) binary: Option<PathBuf>,
    pub(crate) working_dir: Option<PathBuf>,
    pub(crate) env: BTreeMap<String, String>,
    pub(crate) claude_home: Option<ClaudeHomeLayout>,
    pub(crate) create_home_dirs: bool,
    pub(crate) home_seed: Option<ClaudeHomeSeedRequest>,
    pub(crate) home_materialize_status: Arc<std::sync::OnceLock<Result<(), String>>>,
    pub(crate) home_seed_status: Arc<std::sync::OnceLock<Result<(), String>>>,
    pub(crate) timeout: Option<Duration>,
    pub(crate) mirror_stdout: bool,
    pub(crate) mirror_stderr: bool,
}

impl ClaudeClient {
    pub fn builder() -> ClaudeClientBuilder {
        ClaudeClientBuilder::default()
    }

    pub async fn help(&self) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(ClaudeCommandRequest::root().arg("--help"))
            .await
    }

    pub async fn version(&self) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(ClaudeCommandRequest::root().arg("--version"))
            .await
    }

    pub async fn run_command(
        &self,
        request: ClaudeCommandRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.ensure_home_prepared()?;
        let binary = self.resolve_binary();
        let mut cmd = Command::new(&binary);
        cmd.args(request.argv());

        if let Some(dir) = self.working_dir.as_ref() {
            cmd.current_dir(dir);
        }

        process::apply_env(&mut cmd, &self.env);

        let timeout = request.timeout.or(self.timeout);
        process::run_command(
            cmd,
            &binary,
            request.stdin.as_deref(),
            timeout,
            self.mirror_stdout,
            self.mirror_stderr,
        )
        .await
    }

    pub async fn print(
        &self,
        request: ClaudePrintRequest,
    ) -> Result<ClaudePrintResult, ClaudeCodeError> {
        let allow_missing_prompt = request.stdin.is_some()
            || request.continue_session
            || request.resume
            || request.resume_value.is_some()
            || request.from_pr
            || request.from_pr_value.is_some();
        if request.prompt.is_none() && !allow_missing_prompt {
            return Err(ClaudeCodeError::InvalidRequest(
                "either prompt, stdin_bytes, or a continuation flag must be provided".to_string(),
            ));
        }

        self.ensure_home_prepared()?;
        let binary = self.resolve_binary();
        let mut cmd = Command::new(&binary);
        cmd.args(request.argv());

        if let Some(dir) = self.working_dir.as_ref() {
            cmd.current_dir(dir);
        }

        process::apply_env(&mut cmd, &self.env);

        let timeout = request.timeout.or(self.timeout);
        let output = process::run_command(
            cmd,
            &binary,
            request.stdin.as_deref(),
            timeout,
            self.mirror_stdout,
            self.mirror_stderr,
        )
        .await?;

        let parsed = match request.output_format {
            ClaudeOutputFormat::Json => {
                let v = serde_json::from_slice(&output.stdout)?;
                Some(ClaudeParsedOutput::Json(v))
            }
            ClaudeOutputFormat::StreamJson => {
                let s = String::from_utf8_lossy(&output.stdout);
                Some(ClaudeParsedOutput::StreamJson(parse_stream_json_lines(&s)))
            }
            ClaudeOutputFormat::Text => None,
        };

        Ok(ClaudePrintResult { output, parsed })
    }

    pub fn print_stream_json(
        &self,
        request: ClaudePrintRequest,
    ) -> Pin<
        Box<dyn Future<Output = Result<ClaudePrintStreamJsonHandle, ClaudeCodeError>> + Send + '_>,
    > {
        Box::pin(async move {
            let (events, completion, _termination) = self.spawn_print_stream_json(request).await?;
            Ok(ClaudePrintStreamJsonHandle { events, completion })
        })
    }

    pub fn print_stream_json_control(
        &self,
        request: ClaudePrintRequest,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<ClaudePrintStreamJsonControlHandle, ClaudeCodeError>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            let (events, completion, termination) = self.spawn_print_stream_json(request).await?;
            Ok(ClaudePrintStreamJsonControlHandle {
                events,
                completion,
                termination,
            })
        })
    }

    async fn spawn_print_stream_json(
        &self,
        request: ClaudePrintRequest,
    ) -> Result<
        (
            DynClaudeStreamJsonEventStream,
            DynClaudeStreamJsonCompletion,
            ClaudeTerminationHandle,
        ),
        ClaudeCodeError,
    > {
        let custody = crate::ClaudeProcessCustody::new();
        self.spawn_print_stream_json_with_custody(request, &custody)
            .await
    }

    /// Streaming startup with an independently retained child/stdio acknowledgment owner.
    /// Construct custody before polling this future and await `settle` after completion/errors.
    pub async fn print_stream_json_control_with_custody(
        &self,
        request: ClaudePrintRequest,
        custody: &crate::ClaudeProcessCustody,
    ) -> Result<ClaudePrintStreamJsonControlHandle, ClaudeCodeError> {
        let (events, completion, termination) = self
            .spawn_print_stream_json_with_custody(request, custody)
            .await?;
        Ok(ClaudePrintStreamJsonControlHandle {
            events,
            completion,
            termination,
        })
    }

    async fn spawn_print_stream_json_with_custody(
        &self,
        request: ClaudePrintRequest,
        custody: &crate::ClaudeProcessCustody,
    ) -> Result<
        (
            DynClaudeStreamJsonEventStream,
            DynClaudeStreamJsonCompletion,
            ClaudeTerminationHandle,
        ),
        ClaudeCodeError,
    > {
        custody.ensure_open()?;
        let allow_missing_prompt = request.stdin.is_some()
            || request.continue_session
            || request.resume
            || request.resume_value.is_some()
            || request.from_pr
            || request.from_pr_value.is_some();
        if request.prompt.is_none() && !allow_missing_prompt {
            return Err(ClaudeCodeError::InvalidRequest(
                "either prompt, stdin_bytes, or a continuation flag must be provided".to_string(),
            ));
        }

        self.ensure_home_prepared()?;
        let binary = self.resolve_binary();

        let mut request = request;
        request.output_format = ClaudeOutputFormat::StreamJson;
        let stdin_bytes = request.stdin.take();
        let mirror_stdout = self.mirror_stdout;
        let mirror_stderr = self.mirror_stderr;
        let timeout = request.timeout.or(self.timeout);

        let mut cmd = Command::new(&binary);
        cmd.args(request.argv());

        if let Some(dir) = self.working_dir.as_ref() {
            cmd.current_dir(dir);
        }

        process::apply_env(&mut cmd, &self.env);

        cmd.kill_on_drop(true);
        cmd.stdin(if stdin_bytes.is_some() {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::null()
        });
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(if mirror_stderr {
            std::process::Stdio::piped()
        } else {
            std::process::Stdio::null()
        });

        let (child, owner) = custody.spawn_admitted(&mut cmd, &binary).await?;
        let (events_tx, events_rx) = mpsc::channel(32);
        let (ready_tx, ready_rx) = oneshot::channel();
        let (completion_tx, completion_rx) = oneshot::channel();
        let event_custody = custody.clone();
        tokio::spawn(async move {
            let result = crate::process_custody::drive_child(
                child,
                owner,
                stdin_bytes,
                false,
                timeout,
                if mirror_stderr {
                    crate::process_custody::StderrMode::Mirror
                } else {
                    crate::process_custody::StderrMode::Absent
                },
                ready_tx,
                move |stdout| {
                    streaming_custody::forward_events(
                        stdout,
                        events_tx,
                        mirror_stdout,
                        event_custody,
                    )
                },
            )
            .await;
            let completion = streaming_custody::completion(result, timeout);
            let _ = completion_tx.send(completion);
        });
        let mut startup_drop = custody.cancel_on_drop();
        ready_rx
            .await
            .map_err(|_| ClaudeCodeError::Join("stream startup owner failed".to_string()))?
            .map_err(ClaudeCodeError::StdinWrite)?;
        startup_drop.disarm();
        let completion = crate::process_custody::OwnedCompletion {
            receiver: completion_rx,
            cancel: custody.cancel_on_drop(),
        };
        Ok((
            Box::pin(ClaudeStreamJsonEventChannelStream::new(events_rx)),
            Box::pin(async move {
                completion.await.map_err(|_| {
                    ClaudeCodeError::Join("stream completion owner failed".to_string())
                })?
            }),
            ClaudeTerminationHandle::with_custody(custody.clone()),
        ))
    }

    /// Runs the actual help preflight under custody retained before polling startup.
    pub async fn help_with_custody(
        &self,
        custody: &crate::ClaudeProcessCustody,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        custody.ensure_open()?;
        self.ensure_home_prepared()?;
        let binary = self.resolve_binary();
        let mut command = Command::new(&binary);
        command.args(ClaudeCommandRequest::root().arg("--help").argv());
        if let Some(dir) = &self.working_dir {
            command.current_dir(dir);
        }
        process::apply_env(&mut command, &self.env);
        process::run_command_with_custody(
            command,
            &binary,
            None,
            self.timeout,
            self.mirror_stdout,
            self.mirror_stderr,
            custody,
        )
        .await
    }

    pub async fn mcp_list(&self) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(ClaudeCommandRequest::new(["mcp", "list"]))
            .await
    }

    pub async fn mcp_reset_project_choices(&self) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(ClaudeCommandRequest::new(["mcp", "reset-project-choices"]))
            .await
    }

    pub async fn mcp_get(&self, req: McpGetRequest) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn mcp_add(&self, req: McpAddRequest) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn mcp_remove(
        &self,
        req: McpRemoveRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn mcp_add_json(
        &self,
        req: McpAddJsonRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn mcp_serve(&self, req: McpServeRequest) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn mcp_add_from_claude_desktop(
        &self,
        req: McpAddFromClaudeDesktopRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn doctor(&self) -> Result<CommandOutput, ClaudeCodeError> {
        self.doctor_with(ClaudeDoctorRequest::new()).await
    }

    pub async fn doctor_with(
        &self,
        req: ClaudeDoctorRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_list(
        &self,
        req: PluginListRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin(&self, req: PluginRequest) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_enable(
        &self,
        req: PluginEnableRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_disable(
        &self,
        req: PluginDisableRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_install(
        &self,
        req: PluginInstallRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_uninstall(
        &self,
        req: PluginUninstallRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_update(
        &self,
        req: PluginUpdateRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_validate(
        &self,
        req: PluginValidateRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_manifest(
        &self,
        req: PluginManifestRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_manifest_marketplace(
        &self,
        req: PluginManifestMarketplaceRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_marketplace_repo(
        &self,
        req: PluginMarketplaceRepoRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_marketplace(
        &self,
        req: PluginMarketplaceRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_marketplace_add(
        &self,
        req: PluginMarketplaceAddRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_marketplace_list(
        &self,
        req: PluginMarketplaceListRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_marketplace_remove(
        &self,
        req: PluginMarketplaceRemoveRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn plugin_marketplace_update(
        &self,
        req: PluginMarketplaceUpdateRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub async fn update(&self) -> Result<CommandOutput, ClaudeCodeError> {
        self.update_with(ClaudeUpdateRequest::new()).await
    }

    pub async fn update_with(
        &self,
        req: ClaudeUpdateRequest,
    ) -> Result<CommandOutput, ClaudeCodeError> {
        self.run_command(req.into_command()).await
    }

    pub fn claude_home_layout(&self) -> Option<ClaudeHomeLayout> {
        self.claude_home.clone()
    }

    fn resolve_binary(&self) -> PathBuf {
        if let Some(b) = self.binary.as_ref() {
            return b.clone();
        }
        if let Ok(v) = std::env::var("CLAUDE_BINARY") {
            if !v.trim().is_empty() {
                return PathBuf::from(v);
            }
        }
        PathBuf::from("claude")
    }

    fn ensure_home_prepared(&self) -> Result<(), ClaudeCodeError> {
        if self.claude_home.is_none() {
            return Ok(());
        }

        let materialize = self.home_materialize_status.get_or_init(|| {
            let Some(layout) = self.claude_home.as_ref() else {
                return Ok(());
            };
            layout
                .materialize(self.create_home_dirs)
                .map_err(|e| e.to_string())
        });
        if let Err(msg) = materialize {
            return Err(ClaudeCodeError::ClaudeHomePrepareFailed(msg.clone()));
        }

        let seeded = self.home_seed_status.get_or_init(|| {
            let Some(layout) = self.claude_home.as_ref() else {
                return Ok(());
            };
            let Some(seed_req) = self.home_seed.as_ref() else {
                return Ok(());
            };
            // Seeding implies directories must exist even when the caller disabled auto-creation.
            let _ = layout.materialize(true);
            layout
                .seed_from_user_home(&seed_req.seed_user_home, seed_req.level)
                .map(|_| ())
                .map_err(|e| e.to_string())
        });
        if let Err(msg) = seeded {
            return Err(ClaudeCodeError::ClaudeHomeSeedFailed(msg.clone()));
        }

        Ok(())
    }
}

struct ClaudeStreamJsonEventChannelStream {
    rx: mpsc::Receiver<Result<ClaudeStreamJsonEvent, ClaudeStreamJsonParseError>>,
}

impl ClaudeStreamJsonEventChannelStream {
    fn new(rx: mpsc::Receiver<Result<ClaudeStreamJsonEvent, ClaudeStreamJsonParseError>>) -> Self {
        Self { rx }
    }
}

impl Stream for ClaudeStreamJsonEventChannelStream {
    type Item = Result<ClaudeStreamJsonEvent, ClaudeStreamJsonParseError>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();
        this.rx.poll_recv(cx)
    }
}

#[derive(Debug, Clone)]
pub struct ClaudePrintResult {
    pub output: CommandOutput,
    pub parsed: Option<ClaudeParsedOutput>,
}

#[derive(Debug, Clone)]
pub enum ClaudeParsedOutput {
    Json(serde_json::Value),
    StreamJson(Vec<StreamJsonLineOutcome>),
}

#[cfg(test)]
mod tests;
