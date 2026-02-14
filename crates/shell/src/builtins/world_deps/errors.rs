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

#[derive(Debug)]
pub(crate) struct WorldDepsProvisionUnsupportedError {
    pub(crate) message: String,
}

impl WorldDepsProvisionUnsupportedError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl std::fmt::Display for WorldDepsProvisionUnsupportedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for WorldDepsProvisionUnsupportedError {}
