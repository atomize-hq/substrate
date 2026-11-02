use std::ffi::OsString;

use crate::{ApplyDiffArtifacts, CodexClient, CodexError, ForkSessionRequest};

impl CodexClient {
    /// Runs `codex fork [OPTIONS] [SESSION_ID] [PROMPT]` and returns captured output.
    pub async fn fork_session(
        &self,
        request: ForkSessionRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        if matches!(request.prompt.as_deref(), Some(prompt) if prompt.trim().is_empty()) {
            return Err(CodexError::EmptyPrompt);
        }

        let mut args = vec![OsString::from("fork")];
        if request.all {
            args.push(OsString::from("--all"));
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
}
