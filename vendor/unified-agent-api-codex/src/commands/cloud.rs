use std::ffi::OsString;

use crate::{
    ApplyDiffArtifacts, CloudApplyRequest, CloudDiffRequest, CloudExecRequest, CloudListOutput,
    CloudListRequest, CloudOverviewRequest, CloudStatusRequest, CodexClient, CodexError,
};

impl CodexClient {
    /// Runs `codex cloud --help` and returns captured output.
    pub async fn cloud_overview(
        &self,
        request: CloudOverviewRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(
            vec![OsString::from("cloud"), OsString::from("--help")],
            request.overrides,
        )
        .await
    }

    /// Lists Codex Cloud tasks via `codex cloud list`.
    pub async fn cloud_list(
        &self,
        request: CloudListRequest,
    ) -> Result<CloudListOutput, CodexError> {
        let CloudListRequest {
            json,
            env_id,
            limit,
            cursor,
            overrides,
        } = request;

        let mut args = vec![OsString::from("cloud"), OsString::from("list")];
        if let Some(env_id) = env_id {
            args.push(OsString::from("--env"));
            args.push(OsString::from(env_id));
        }
        if let Some(limit) = limit {
            args.push(OsString::from("--limit"));
            args.push(OsString::from(limit.to_string()));
        }
        if let Some(cursor) = cursor {
            args.push(OsString::from("--cursor"));
            args.push(OsString::from(cursor));
        }
        if json {
            args.push(OsString::from("--json"));
        }

        let artifacts = self
            .run_simple_command_with_overrides(args, overrides)
            .await?;
        let parsed = if json {
            Some(serde_json::from_str(&artifacts.stdout).map_err(|source| {
                CodexError::JsonParse {
                    context: "cloud list",
                    stdout: artifacts.stdout.clone(),
                    source,
                }
            })?)
        } else {
            None
        };

        Ok(CloudListOutput {
            status: artifacts.status,
            stdout: artifacts.stdout,
            stderr: artifacts.stderr,
            json: parsed,
        })
    }

    /// Shows the status of a Codex Cloud task via `codex cloud status <TASK_ID>`.
    pub async fn cloud_status(
        &self,
        request: CloudStatusRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = request.task_id.trim();
        if task_id.is_empty() {
            return Err(CodexError::EmptyTaskId);
        }

        self.run_simple_command_with_overrides(
            vec![
                OsString::from("cloud"),
                OsString::from("status"),
                OsString::from(task_id),
            ],
            request.overrides,
        )
        .await
    }

    /// Shows the unified diff for a Codex Cloud task via `codex cloud diff [--attempt N] <TASK_ID>`.
    pub async fn cloud_diff(
        &self,
        request: CloudDiffRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = request.task_id.trim();
        if task_id.is_empty() {
            return Err(CodexError::EmptyTaskId);
        }

        let mut args = vec![OsString::from("cloud"), OsString::from("diff")];
        if let Some(attempt) = request.attempt {
            args.push(OsString::from("--attempt"));
            args.push(OsString::from(attempt.to_string()));
        }
        args.push(OsString::from(task_id));
        self.run_simple_command_with_overrides(args, request.overrides)
            .await
    }

    /// Applies the diff for a Codex Cloud task locally via `codex cloud apply [--attempt N] <TASK_ID>`.
    pub async fn cloud_apply(
        &self,
        request: CloudApplyRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let task_id = request.task_id.trim();
        if task_id.is_empty() {
            return Err(CodexError::EmptyTaskId);
        }

        let mut args = vec![OsString::from("cloud"), OsString::from("apply")];
        if let Some(attempt) = request.attempt {
            args.push(OsString::from("--attempt"));
            args.push(OsString::from(attempt.to_string()));
        }
        args.push(OsString::from(task_id));
        self.run_simple_command_with_overrides(args, request.overrides)
            .await
    }

    /// Submits a new Codex Cloud task via `codex cloud exec`.
    pub async fn cloud_exec(
        &self,
        request: CloudExecRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let env_id = request.env_id.trim();
        if env_id.is_empty() {
            return Err(CodexError::EmptyEnvId);
        }

        let mut args = vec![OsString::from("cloud"), OsString::from("exec")];
        args.push(OsString::from("--env"));
        args.push(OsString::from(env_id));
        if let Some(attempts) = request.attempts {
            args.push(OsString::from("--attempts"));
            args.push(OsString::from(attempts.to_string()));
        }
        if let Some(branch) = request.branch {
            args.push(OsString::from("--branch"));
            args.push(OsString::from(branch));
        }
        if let Some(query) = request.query {
            args.push(OsString::from(query));
        }

        self.run_simple_command_with_overrides(args, request.overrides)
            .await
    }
}
