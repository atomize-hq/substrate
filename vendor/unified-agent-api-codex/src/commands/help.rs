use std::ffi::OsString;

use crate::{ApplyDiffArtifacts, CodexClient, CodexError, HelpCommandRequest};

impl CodexClient {
    /// Runs `codex <scope> help [COMMAND]...` and returns captured output.
    pub async fn help(
        &self,
        request: HelpCommandRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let mut args: Vec<OsString> = request
            .scope
            .argv_prefix()
            .iter()
            .map(|value| OsString::from(*value))
            .collect();
        args.extend(request.command.into_iter().map(OsString::from));
        self.run_simple_command_with_overrides(args, request.overrides)
            .await
    }
}
