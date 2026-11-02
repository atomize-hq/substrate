use std::path::PathBuf;

use crate::CliOverridesPatch;

/// Request for `codex debug`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugCommandRequest {
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugCommandRequest {
    pub fn new() -> Self {
        Self {
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for DebugCommandRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex debug help [COMMAND]...`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugHelpRequest {
    /// Optional command tokens passed after `help` (variadic).
    pub command: Vec<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugHelpRequest {
    pub fn new() -> Self {
        Self {
            command: Vec::new(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional `COMMAND` tokens.
    pub fn command(mut self, command: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.command = command.into_iter().map(Into::into).collect();
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for DebugHelpRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex debug app-server`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugAppServerRequest {
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugAppServerRequest {
    pub fn new() -> Self {
        Self {
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for DebugAppServerRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex debug app-server help [COMMAND]...`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugAppServerHelpRequest {
    /// Optional command tokens passed after `help` (variadic).
    pub command: Vec<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugAppServerHelpRequest {
    pub fn new() -> Self {
        Self {
            command: Vec::new(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional `COMMAND` tokens.
    pub fn command(mut self, command: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.command = command.into_iter().map(Into::into).collect();
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for DebugAppServerHelpRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex debug app-server send-message-v2 <USER_MESSAGE>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugAppServerSendMessageV2Request {
    /// Message payload sent to the app-server debug shim.
    pub user_message: String,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugAppServerSendMessageV2Request {
    pub fn new(user_message: impl Into<String>) -> Self {
        Self {
            user_message: user_message.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex debug models`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugModelsRequest {
    /// Requests the `--bundled` flag when enabled.
    pub bundled: bool,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugModelsRequest {
    pub fn new() -> Self {
        Self {
            bundled: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Controls whether `--bundled` is passed to the command.
    pub fn bundled(mut self, enable: bool) -> Self {
        self.bundled = enable;
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for DebugModelsRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex debug prompt-input [PROMPT]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DebugPromptInputRequest {
    /// Optional prompt payload appended after the debug context.
    pub prompt: Option<String>,
    /// Image paths forwarded via repeated `--image <FILE>` flags.
    pub images: Vec<PathBuf>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl DebugPromptInputRequest {
    pub fn new() -> Self {
        Self {
            prompt: None,
            images: Vec::new(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional prompt payload.
    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Appends a single `--image <FILE>` argument.
    pub fn image(mut self, image: impl Into<PathBuf>) -> Self {
        self.images.push(image.into());
        self
    }

    /// Extends the repeated `--image <FILE>` arguments.
    pub fn images<I, P>(mut self, images: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.images.extend(images.into_iter().map(Into::into));
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for DebugPromptInputRequest {
    fn default() -> Self {
        Self::new()
    }
}
