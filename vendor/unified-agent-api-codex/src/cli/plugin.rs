use crate::CliOverridesPatch;

/// Request for `codex plugin`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginCommandRequest {
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginCommandRequest {
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

impl Default for PluginCommandRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex plugin help [COMMAND]...`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginHelpRequest {
    /// Optional command tokens passed after `help` (variadic).
    pub command: Vec<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginHelpRequest {
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

impl Default for PluginHelpRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex plugin marketplace`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginMarketplaceCommandRequest {
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginMarketplaceCommandRequest {
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

impl Default for PluginMarketplaceCommandRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex plugin marketplace help [COMMAND]...`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginMarketplaceHelpRequest {
    /// Optional command tokens passed after `help` (variadic).
    pub command: Vec<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginMarketplaceHelpRequest {
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

impl Default for PluginMarketplaceHelpRequest {
    fn default() -> Self {
        Self::new()
    }
}

/// Request for `codex plugin marketplace add <SOURCE>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginMarketplaceAddRequest {
    /// Marketplace source to install.
    pub source: String,
    /// Optional ref passed via `--ref`.
    pub source_ref: Option<String>,
    /// Optional sparse path passed via `--sparse`.
    pub sparse_path: Option<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginMarketplaceAddRequest {
    pub fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            source_ref: None,
            sparse_path: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional `--ref` value.
    pub fn source_ref(mut self, source_ref: impl Into<String>) -> Self {
        let source_ref = source_ref.into();
        self.source_ref = (!source_ref.trim().is_empty()).then_some(source_ref);
        self
    }

    /// Sets the optional `--sparse` value.
    pub fn sparse_path(mut self, sparse_path: impl Into<String>) -> Self {
        let sparse_path = sparse_path.into();
        self.sparse_path = (!sparse_path.trim().is_empty()).then_some(sparse_path);
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex plugin marketplace remove <MARKETPLACE_NAME>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginMarketplaceRemoveRequest {
    /// Configured marketplace name to remove.
    pub marketplace_name: String,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginMarketplaceRemoveRequest {
    pub fn new(marketplace_name: impl Into<String>) -> Self {
        Self {
            marketplace_name: marketplace_name.into(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

/// Request for `codex plugin marketplace upgrade [MARKETPLACE_NAME]`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PluginMarketplaceUpgradeRequest {
    /// Optional configured marketplace name to upgrade.
    pub marketplace_name: Option<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl PluginMarketplaceUpgradeRequest {
    pub fn new() -> Self {
        Self {
            marketplace_name: None,
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Sets the optional marketplace name.
    pub fn marketplace_name(mut self, marketplace_name: impl Into<String>) -> Self {
        let marketplace_name = marketplace_name.into();
        self.marketplace_name = (!marketplace_name.trim().is_empty()).then_some(marketplace_name);
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}

impl Default for PluginMarketplaceUpgradeRequest {
    fn default() -> Self {
        Self::new()
    }
}
