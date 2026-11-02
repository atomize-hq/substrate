use std::ffi::OsString;

use crate::{ApplyDiffArtifacts, CodexClient, CodexError, UpdateCommandRequest};

impl CodexClient {
    /// Runs `codex update` and returns captured output.
    pub async fn update(
        &self,
        request: UpdateCommandRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(vec![OsString::from("update")], request.overrides)
            .await
    }
}
