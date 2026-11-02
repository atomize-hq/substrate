use crate::CliOverridesPatch;

/// Selector for `codex help`-style command families.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HelpScope {
    Root,
    Exec,
    Features,
    Login,
    AppServer,
    Sandbox,
    Cloud,
    Mcp,
}

impl HelpScope {
    pub(crate) fn argv_prefix(&self) -> &'static [&'static str] {
        match self {
            HelpScope::Root => &["help"],
            HelpScope::Exec => &["exec", "help"],
            HelpScope::Features => &["features", "help"],
            HelpScope::Login => &["login", "help"],
            HelpScope::AppServer => &["app-server", "help"],
            HelpScope::Sandbox => &["sandbox", "help"],
            HelpScope::Cloud => &["cloud", "help"],
            HelpScope::Mcp => &["mcp", "help"],
        }
    }
}

/// Request for `codex <scope> help [COMMAND]...`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HelpCommandRequest {
    pub scope: HelpScope,
    /// Optional command path components appended after `help` (variadic upstream).
    pub command: Vec<String>,
    /// Per-call CLI overrides layered on top of the builder.
    pub overrides: CliOverridesPatch,
}

impl HelpCommandRequest {
    pub fn new(scope: HelpScope) -> Self {
        Self {
            scope,
            command: Vec::new(),
            overrides: CliOverridesPatch::default(),
        }
    }

    /// Appends one or more command tokens to the help invocation.
    pub fn command<I, S>(mut self, tokens: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.command.extend(tokens.into_iter().map(Into::into));
        self
    }

    /// Replaces the default CLI overrides for this request.
    pub fn with_overrides(mut self, overrides: CliOverridesPatch) -> Self {
        self.overrides = overrides;
        self
    }
}
