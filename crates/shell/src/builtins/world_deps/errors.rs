#[derive(Debug)]
pub(crate) struct WorldDepsBackendRequiredError {
    pub(crate) message: String,
}

impl WorldDepsBackendRequiredError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WorldDepsBackendRequiredError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorldDepsBackendRequiredError {}

#[derive(Debug)]
pub(crate) struct WorldDepsBackendUnavailableError {
    reason: String,
}

impl WorldDepsBackendUnavailableError {
    pub(crate) fn new(reason: impl Into<String>) -> Self {
        Self {
            reason: reason.into(),
        }
    }

    pub(crate) fn reason(&self) -> &str {
        &self.reason
    }
}

impl std::fmt::Display for WorldDepsBackendUnavailableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "world backend unavailable: {}", self.reason)
    }
}

impl std::error::Error for WorldDepsBackendUnavailableError {}

#[derive(Debug)]
pub(crate) struct WorldDepsUnmetPrerequisiteError {
    pub(crate) message: String,
}

impl WorldDepsUnmetPrerequisiteError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WorldDepsUnmetPrerequisiteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorldDepsUnmetPrerequisiteError {}
