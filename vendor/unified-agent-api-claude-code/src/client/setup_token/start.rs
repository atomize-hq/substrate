use std::sync::Arc;

use tokio::{
    process::Command,
    sync::{oneshot, Mutex},
};

use super::super::ClaudeClient;
use super::{
    capture::spawn_capture_task, process::SetupTokenProcess, session::ClaudeSetupTokenSession,
    url::UrlCapture,
};
use crate::{
    commands::setup_token::ClaudeSetupTokenRequest,
    process::{self as cli_process, ConsoleTarget},
    ClaudeCodeError,
};

#[cfg(unix)]
use super::pty::spawn_setup_token_pty;

impl ClaudeClient {
    pub async fn setup_token_start(&self) -> Result<ClaudeSetupTokenSession, ClaudeCodeError> {
        self.setup_token_start_with(ClaudeSetupTokenRequest::new())
            .await
    }

    pub async fn setup_token_start_with(
        &self,
        request: ClaudeSetupTokenRequest,
    ) -> Result<ClaudeSetupTokenSession, ClaudeCodeError> {
        self.ensure_home_prepared()?;
        let requested_timeout = request.timeout;
        let binary = self.resolve_binary();
        let argv = request.into_command().argv();

        let stdout_buf = Arc::new(Mutex::new(Vec::<u8>::new()));
        let stderr_buf = Arc::new(Mutex::new(Vec::<u8>::new()));

        let (url_tx, url_rx) = oneshot::channel::<String>();
        let url_tx = Arc::new(Mutex::new(Some(url_tx)));
        let url_state = Arc::new(Mutex::new(UrlCapture::default()));

        #[cfg(unix)]
        if let Ok((process, stdout_task)) = spawn_setup_token_pty(
            &binary,
            &argv,
            self.working_dir.as_deref(),
            &self.env,
            self.mirror_stdout,
            self.mirror_stderr,
            stdout_buf.clone(),
            url_state.clone(),
            url_tx.clone(),
        ) {
            return Ok(ClaudeSetupTokenSession::new(
                process,
                stdout_buf,
                stderr_buf,
                stdout_task,
                None,
                url_rx,
                requested_timeout,
            ));
        }

        let mut cmd = Command::new(&binary);
        cmd.args(&argv);

        if let Some(dir) = self.working_dir.as_ref() {
            cmd.current_dir(dir);
        }

        cli_process::apply_env(&mut cmd, &self.env);

        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());
        cmd.kill_on_drop(true);

        let mut child = cli_process::spawn_with_retry(&mut cmd, &binary)?;
        let stdin = child.stdin.take();
        let stdout = child.stdout.take().ok_or(ClaudeCodeError::MissingStdout)?;
        let stderr = child.stderr.take().ok_or(ClaudeCodeError::MissingStderr)?;

        let stdout_task = spawn_capture_task(
            stdout,
            ConsoleTarget::Stdout,
            self.mirror_stdout,
            stdout_buf.clone(),
            url_state.clone(),
            url_tx.clone(),
        );
        let stderr_task = spawn_capture_task(
            stderr,
            ConsoleTarget::Stderr,
            self.mirror_stderr,
            stderr_buf.clone(),
            url_state.clone(),
            url_tx.clone(),
        );

        Ok(ClaudeSetupTokenSession::new(
            SetupTokenProcess::Pipes { child, stdin },
            stdout_buf,
            stderr_buf,
            stdout_task,
            Some(stderr_task),
            url_rx,
            // `setup-token` is inherently interactive and can require human/browser steps.
            // Do not apply the client's default timeout unless the caller explicitly requests one.
            requested_timeout,
        ))
    }
}
