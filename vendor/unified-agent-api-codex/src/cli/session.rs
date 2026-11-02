use crate::CliOverridesPatch;

/// Request for `codex resume [OPTIONS] [SESSION_ID] [PROMPT]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResumeSessionRequest {
    pub session_id: Option<String>,
    pub prompt: Option<String>,
    pub all: bool,
    pub last: bool,
    pub include_non_interactive: bool,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl ResumeSessionRequest {
    pub fn new() -> Self {
        Self {
            session_id: None,
            prompt: None,
            all: false,
            last: false,
            include_non_interactive: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        self.session_id = (!session_id.trim().is_empty()).then_some(session_id);
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        let prompt = prompt.into();
        self.prompt = (!prompt.trim().is_empty()).then_some(prompt);
        self
    }

    pub fn all(mut self, enable: bool) -> Self {
        self.all = enable;
        self
    }

    pub fn last(mut self, enable: bool) -> Self {
        self.last = enable;
        self
    }

    pub fn include_non_interactive(mut self, enable: bool) -> Self {
        self.include_non_interactive = enable;
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for ResumeSessionRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex fork [OPTIONS] [SESSION_ID] [PROMPT]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForkSessionRequest {
    pub session_id: Option<String>,
    pub prompt: Option<String>,
    pub all: bool,
    pub last: bool,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl ForkSessionRequest {
    pub fn new() -> Self {
        Self {
            session_id: None,
            prompt: None,
            all: false,
            last: false,
            overrides: CliOverridesPatch::default(),
        }
    }

    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        let session_id = session_id.into();
        self.session_id = (!session_id.trim().is_empty()).then_some(session_id);
        self
    }

    pub fn prompt(mut self, prompt: impl Into<String>) -> Self {
        let prompt = prompt.into();
        self.prompt = (!prompt.trim().is_empty()).then_some(prompt);
        self
    }

    pub fn all(mut self, enable: bool) -> Self {
        self.all = enable;
        self
    }

    pub fn last(mut self, enable: bool) -> Self {
        self.last = enable;
        self
    }

    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for ForkSessionRequest {
    fn default() -> Self {
        Self::new()
    }
}
