use tokio::process::Command;

use crate::{
    builder::{apply_cli_overrides, resolve_cli_overrides},
    process::spawn_with_retry,
    CodexClient, CodexError, ExecServerRequest,
};

impl CodexClient {
    /// Spawns `codex exec-server` with piped stdio for direct server integration.
    pub fn start_exec_server(
        &self,
        request: ExecServerRequest,
    ) -> Result<tokio::process::Child, CodexError> {
        let ExecServerRequest {
            listen,
            executor_id,
            name,
            working_dir,
            overrides,
        } = request;

        let resolved_overrides =
            resolve_cli_overrides(&self.cli_overrides, &overrides, self.model.as_deref());

        let mut command = Command::new(self.command_env.binary_path());
        command
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .current_dir(self.sandbox_working_dir(working_dir)?);

        apply_cli_overrides(&mut command, &resolved_overrides, true);
        command.arg("exec-server");

        if let Some(listen) = listen {
            command.arg("--listen").arg(listen);
        }

        if let Some(executor_id) = executor_id {
            command.arg("--executor-id").arg(executor_id);
        }

        if let Some(name) = name {
            command.arg("--name").arg(name);
        }

        self.command_env.apply(&mut command)?;

        spawn_with_retry(&mut command, self.command_env.binary_path())
    }
}
