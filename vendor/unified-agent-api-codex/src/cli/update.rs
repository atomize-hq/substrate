use crate::CliOverridesPatch;

/// Request for `codex update`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdateCommandRequest {
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl UpdateCommandRequest {
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

impl Default for UpdateCommandRequest {
    fn default() -> Self {
        Self::new()
    }
}
