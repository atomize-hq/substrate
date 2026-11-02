use std::ffi::OsString;

use crate::{
    ApplyDiffArtifacts, CodexClient, CodexError, DebugAppServerHelpRequest, DebugAppServerRequest,
    DebugAppServerSendMessageV2Request, DebugCommandRequest, DebugHelpRequest, DebugModelsRequest,
    DebugPromptInputRequest,
};

impl CodexClient {
    /// Runs `codex debug` and returns captured output.
    pub async fn debug(
        &self,
        request: DebugCommandRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(vec![OsString::from("debug")], request.overrides)
            .await
    }

    /// Runs `codex debug help [COMMAND]...` and returns captured output.
    pub async fn debug_help(
        &self,
        request: DebugHelpRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let DebugHelpRequest { command, overrides } = request;
        let mut args = vec![OsString::from("debug"), OsString::from("help")];
        args.extend(command.into_iter().map(OsString::from));
        self.run_simple_command_with_overrides(args, overrides)
            .await
    }

    /// Runs `codex debug app-server` and returns captured output.
    pub async fn debug_app_server(
        &self,
        request: DebugAppServerRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        self.run_simple_command_with_overrides(
            vec![OsString::from("debug"), OsString::from("app-server")],
            request.overrides,
        )
        .await
    }

    /// Runs `codex debug app-server help [COMMAND]...` and returns captured output.
    pub async fn debug_app_server_help(
        &self,
        request: DebugAppServerHelpRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let DebugAppServerHelpRequest { command, overrides } = request;
        let mut args = vec![
            OsString::from("debug"),
            OsString::from("app-server"),
            OsString::from("help"),
        ];
        args.extend(command.into_iter().map(OsString::from));
        self.run_simple_command_with_overrides(args, overrides)
            .await
    }

    /// Runs `codex debug app-server send-message-v2 <USER_MESSAGE>` and returns captured output.
    pub async fn debug_app_server_send_message_v2(
        &self,
        request: DebugAppServerSendMessageV2Request,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let DebugAppServerSendMessageV2Request {
            user_message,
            overrides,
        } = request;
        self.run_simple_command_with_overrides(
            vec![
                OsString::from("debug"),
                OsString::from("app-server"),
                OsString::from("send-message-v2"),
                OsString::from(user_message),
            ],
            overrides,
        )
        .await
    }

    /// Runs `codex debug models [--bundled]` and returns captured output.
    pub async fn debug_models(
        &self,
        request: DebugModelsRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let DebugModelsRequest { bundled, overrides } = request;
        let mut args = vec![OsString::from("debug"), OsString::from("models")];
        if bundled {
            args.push(OsString::from("--bundled"));
        }
        self.run_simple_command_with_overrides(args, overrides)
            .await
    }

    /// Runs `codex debug prompt-input [--image <FILE>...] [PROMPT]` and returns captured output.
    pub async fn debug_prompt_input(
        &self,
        request: DebugPromptInputRequest,
    ) -> Result<ApplyDiffArtifacts, CodexError> {
        let DebugPromptInputRequest {
            prompt,
            images,
            overrides,
        } = request;
        let mut args = vec![OsString::from("debug"), OsString::from("prompt-input")];
        for image in images {
            args.push(OsString::from("--image"));
            args.push(image.into_os_string());
        }
        if let Some(prompt) = prompt {
            args.push(OsString::from(prompt));
        }
        self.run_simple_command_with_overrides(args, overrides)
            .await
    }
}
