use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeOutputFormat {
    Text,
    Json,
    StreamJson,
}

impl ClaudeOutputFormat {
    pub(crate) fn as_arg_value(&self) -> &'static str {
        match self {
            ClaudeOutputFormat::Text => "text",
            ClaudeOutputFormat::Json => "json",
            ClaudeOutputFormat::StreamJson => "stream-json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeInputFormat {
    Text,
    StreamJson,
}

impl ClaudeInputFormat {
    pub(crate) fn as_arg_value(&self) -> &'static str {
        match self {
            ClaudeInputFormat::Text => "text",
            ClaudeInputFormat::StreamJson => "stream-json",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaudeChromeMode {
    Chrome,
    NoChrome,
}

#[derive(Debug, Clone)]
pub struct ClaudePrintRequest {
    pub(crate) prompt: Option<String>,
    pub(crate) stdin: Option<Vec<u8>>,
    pub(crate) output_format: ClaudeOutputFormat,
    pub(crate) input_format: Option<ClaudeInputFormat>,
    pub(crate) json_schema: Option<String>,
    pub(crate) model: Option<String>,
    pub(crate) allowed_tools: Vec<String>,
    pub(crate) disallowed_tools: Vec<String>,
    pub(crate) permission_mode: Option<String>,
    pub(crate) dangerously_skip_permissions: bool,
    pub(crate) add_dirs: Vec<String>,
    pub(crate) mcp_config: Option<String>,
    pub(crate) strict_mcp_config: bool,
    pub(crate) agent: Option<String>,
    pub(crate) agents: Option<String>,
    pub(crate) allow_dangerously_skip_permissions: bool,
    pub(crate) append_system_prompt: Option<String>,
    pub(crate) betas: Vec<String>,
    pub(crate) chrome_mode: Option<ClaudeChromeMode>,
    pub(crate) continue_session: bool,
    pub(crate) debug: bool,
    pub(crate) debug_file: Option<String>,
    pub(crate) disable_slash_commands: bool,
    pub(crate) fallback_model: Option<String>,
    pub(crate) files: Vec<String>,
    pub(crate) fork_session: bool,
    pub(crate) from_pr: bool,
    pub(crate) from_pr_value: Option<String>,
    pub(crate) ide: bool,
    pub(crate) include_partial_messages: bool,
    pub(crate) max_budget_usd: Option<f64>,
    pub(crate) mcp_debug: bool,
    pub(crate) no_session_persistence: bool,
    pub(crate) plugin_dirs: Vec<String>,
    pub(crate) replay_user_messages: bool,
    pub(crate) resume: bool,
    pub(crate) resume_value: Option<String>,
    pub(crate) session_id: Option<String>,
    pub(crate) setting_sources: Option<String>,
    pub(crate) settings: Option<String>,
    pub(crate) system_prompt: Option<String>,
    pub(crate) tools: Vec<String>,
    pub(crate) verbose: bool,
    pub(crate) timeout: Option<Duration>,
    pub(crate) extra_args: Vec<String>,
}

impl ClaudePrintRequest {
    pub fn new(prompt: impl Into<String>) -> Self {
        Self {
            prompt: Some(prompt.into()),
            stdin: None,
            output_format: ClaudeOutputFormat::Text,
            input_format: None,
            json_schema: None,
            model: None,
            allowed_tools: Vec::new(),
            disallowed_tools: Vec::new(),
            permission_mode: None,
            dangerously_skip_permissions: false,
            add_dirs: Vec::new(),
            mcp_config: None,
            strict_mcp_config: false,
            agent: None,
            agents: None,
            allow_dangerously_skip_permissions: false,
            append_system_prompt: None,
            betas: Vec::new(),
            chrome_mode: None,
            continue_session: false,
            debug: false,
            debug_file: None,
            disable_slash_commands: false,
            fallback_model: None,
            files: Vec::new(),
            fork_session: false,
            from_pr: false,
            from_pr_value: None,
            ide: false,
            include_partial_messages: false,
            max_budget_usd: None,
            mcp_debug: false,
            no_session_persistence: false,
            plugin_dirs: Vec::new(),
            replay_user_messages: false,
            resume: false,
            resume_value: None,
            session_id: None,
            setting_sources: None,
            settings: None,
            system_prompt: None,
            tools: Vec::new(),
            verbose: false,
            timeout: None,
            extra_args: Vec::new(),
        }
    }

    pub fn no_prompt(mut self) -> Self {
        self.prompt = None;
        self
    }

    pub fn output_format(mut self, format: ClaudeOutputFormat) -> Self {
        self.output_format = format;
        self
    }

    pub fn input_format(mut self, format: ClaudeInputFormat) -> Self {
        self.input_format = Some(format);
        self
    }

    pub fn json_schema(mut self, schema: impl Into<String>) -> Self {
        self.json_schema = Some(schema.into());
        self
    }

    pub fn model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    pub fn allowed_tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.allowed_tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn disallowed_tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.disallowed_tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn permission_mode(mut self, mode: impl Into<String>) -> Self {
        self.permission_mode = Some(mode.into());
        self
    }

    pub fn dangerously_skip_permissions(mut self, enabled: bool) -> Self {
        self.dangerously_skip_permissions = enabled;
        self
    }

    pub fn add_dirs(mut self, dirs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.add_dirs = dirs.into_iter().map(Into::into).collect();
        self
    }

    pub fn mcp_config(mut self, config: impl Into<String>) -> Self {
        self.mcp_config = Some(config.into());
        self
    }

    pub fn strict_mcp_config(mut self, enabled: bool) -> Self {
        self.strict_mcp_config = enabled;
        self
    }

    pub fn agent(mut self, agent: impl Into<String>) -> Self {
        self.agent = Some(agent.into());
        self
    }

    pub fn agents(mut self, json: impl Into<String>) -> Self {
        self.agents = Some(json.into());
        self
    }

    pub fn allow_dangerously_skip_permissions(mut self, enabled: bool) -> Self {
        self.allow_dangerously_skip_permissions = enabled;
        self
    }

    pub fn append_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.append_system_prompt = Some(prompt.into());
        self
    }

    pub fn betas(mut self, betas: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.betas = betas.into_iter().map(Into::into).collect();
        self
    }

    pub fn chrome(mut self) -> Self {
        self.chrome_mode = Some(ClaudeChromeMode::Chrome);
        self
    }

    pub fn no_chrome(mut self) -> Self {
        self.chrome_mode = Some(ClaudeChromeMode::NoChrome);
        self
    }

    pub fn continue_session(mut self, enabled: bool) -> Self {
        self.continue_session = enabled;
        self
    }

    pub fn debug(mut self, enabled: bool) -> Self {
        self.debug = enabled;
        self
    }

    pub fn debug_file(mut self, path: impl Into<String>) -> Self {
        self.debug_file = Some(path.into());
        self
    }

    pub fn disable_slash_commands(mut self, enabled: bool) -> Self {
        self.disable_slash_commands = enabled;
        self
    }

    pub fn fallback_model(mut self, model: impl Into<String>) -> Self {
        self.fallback_model = Some(model.into());
        self
    }

    pub fn files(mut self, specs: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.files = specs.into_iter().map(Into::into).collect();
        self
    }

    pub fn fork_session(mut self, enabled: bool) -> Self {
        self.fork_session = enabled;
        self
    }

    pub fn from_pr(mut self, enabled: bool) -> Self {
        self.from_pr = enabled;
        self
    }

    pub fn from_pr_value(mut self, value: impl Into<String>) -> Self {
        self.from_pr_value = Some(value.into());
        self
    }

    pub fn ide(mut self, enabled: bool) -> Self {
        self.ide = enabled;
        self
    }

    pub fn include_partial_messages(mut self, enabled: bool) -> Self {
        self.include_partial_messages = enabled;
        self
    }

    pub fn max_budget_usd(mut self, amount: f64) -> Self {
        self.max_budget_usd = Some(amount);
        self
    }

    pub fn mcp_debug(mut self, enabled: bool) -> Self {
        self.mcp_debug = enabled;
        self
    }

    pub fn no_session_persistence(mut self, enabled: bool) -> Self {
        self.no_session_persistence = enabled;
        self
    }

    pub fn plugin_dirs(mut self, paths: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.plugin_dirs = paths.into_iter().map(Into::into).collect();
        self
    }

    pub fn replay_user_messages(mut self, enabled: bool) -> Self {
        self.replay_user_messages = enabled;
        self
    }

    pub fn resume(mut self, enabled: bool) -> Self {
        self.resume = enabled;
        self
    }

    pub fn resume_value(mut self, value: impl Into<String>) -> Self {
        self.resume_value = Some(value.into());
        self
    }

    pub fn session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    pub fn setting_sources(mut self, sources: impl Into<String>) -> Self {
        self.setting_sources = Some(sources.into());
        self
    }

    pub fn settings(mut self, file_or_json: impl Into<String>) -> Self {
        self.settings = Some(file_or_json.into());
        self
    }

    pub fn system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }

    pub fn tools(mut self, tools: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.tools = tools.into_iter().map(Into::into).collect();
        self
    }

    pub fn verbose(mut self, enabled: bool) -> Self {
        self.verbose = enabled;
        self
    }

    pub fn stdin_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.stdin = Some(bytes);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn extra_args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extra_args = args.into_iter().map(Into::into).collect();
        self
    }

    pub fn argv(&self) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();

        out.push("--print".to_string());
        out.push("--output-format".to_string());
        out.push(self.output_format.as_arg_value().to_string());

        if let Some(input_format) = self.input_format {
            out.push("--input-format".to_string());
            out.push(input_format.as_arg_value().to_string());
        }

        if let Some(schema) = self.json_schema.as_ref() {
            out.push("--json-schema".to_string());
            out.push(schema.clone());
        }

        if let Some(model) = self.model.as_ref() {
            out.push("--model".to_string());
            out.push(model.clone());
        }

        if !self.allowed_tools.is_empty() {
            out.push("--allowedTools".to_string());
            out.extend(self.allowed_tools.iter().cloned());
        }

        if !self.disallowed_tools.is_empty() {
            out.push("--disallowedTools".to_string());
            out.extend(self.disallowed_tools.iter().cloned());
        }

        if let Some(mode) = self.permission_mode.as_ref() {
            out.push("--permission-mode".to_string());
            out.push(mode.clone());
        }

        if self.dangerously_skip_permissions {
            out.push("--dangerously-skip-permissions".to_string());
        }

        if self.allow_dangerously_skip_permissions {
            out.push("--allow-dangerously-skip-permissions".to_string());
        }

        if !self.add_dirs.is_empty() {
            out.push("--add-dir".to_string());
            out.extend(self.add_dirs.iter().cloned());
        }

        if let Some(config) = self.mcp_config.as_ref() {
            out.push("--mcp-config".to_string());
            out.push(config.clone());
        }

        if self.strict_mcp_config {
            out.push("--strict-mcp-config".to_string());
        }

        if let Some(agent) = self.agent.as_ref() {
            out.push("--agent".to_string());
            out.push(agent.clone());
        }

        if let Some(agents) = self.agents.as_ref() {
            out.push("--agents".to_string());
            out.push(agents.clone());
        }

        if let Some(prompt) = self.append_system_prompt.as_ref() {
            out.push("--append-system-prompt".to_string());
            out.push(prompt.clone());
        }

        if !self.betas.is_empty() {
            out.push("--betas".to_string());
            out.extend(self.betas.iter().cloned());
        }

        if let Some(mode) = self.chrome_mode {
            match mode {
                ClaudeChromeMode::Chrome => out.push("--chrome".to_string()),
                ClaudeChromeMode::NoChrome => out.push("--no-chrome".to_string()),
            }
        }

        if self.continue_session {
            out.push("--continue".to_string());
        }

        if self.debug {
            out.push("--debug".to_string());
        }

        if let Some(path) = self.debug_file.as_ref() {
            out.push("--debug-file".to_string());
            out.push(path.clone());
        }

        if self.disable_slash_commands {
            out.push("--disable-slash-commands".to_string());
        }

        if let Some(model) = self.fallback_model.as_ref() {
            out.push("--fallback-model".to_string());
            out.push(model.clone());
        }

        if !self.files.is_empty() {
            out.push("--file".to_string());
            out.extend(self.files.iter().cloned());
        }

        if self.fork_session {
            out.push("--fork-session".to_string());
        }

        if let Some(value) = self.from_pr_value.as_ref() {
            out.push("--from-pr".to_string());
            out.push(value.clone());
        } else if self.from_pr {
            out.push("--from-pr".to_string());
        }

        if self.ide {
            out.push("--ide".to_string());
        }

        if self.include_partial_messages {
            out.push("--include-partial-messages".to_string());
        }

        if let Some(amount) = self.max_budget_usd {
            out.push("--max-budget-usd".to_string());
            out.push(amount.to_string());
        }

        if self.mcp_debug {
            out.push("--mcp-debug".to_string());
        }

        if self.no_session_persistence {
            out.push("--no-session-persistence".to_string());
        }

        if !self.plugin_dirs.is_empty() {
            out.push("--plugin-dir".to_string());
            out.extend(self.plugin_dirs.iter().cloned());
        }

        if self.replay_user_messages {
            out.push("--replay-user-messages".to_string());
        }

        if let Some(value) = self.resume_value.as_ref() {
            out.push("--resume".to_string());
            out.push(value.clone());
        } else if self.resume {
            out.push("--resume".to_string());
        }

        if let Some(id) = self.session_id.as_ref() {
            out.push("--session-id".to_string());
            out.push(id.clone());
        }

        if let Some(sources) = self.setting_sources.as_ref() {
            out.push("--setting-sources".to_string());
            out.push(sources.clone());
        }

        if let Some(settings) = self.settings.as_ref() {
            out.push("--settings".to_string());
            out.push(settings.clone());
        }

        if let Some(prompt) = self.system_prompt.as_ref() {
            out.push("--system-prompt".to_string());
            out.push(prompt.clone());
        }

        if !self.tools.is_empty() {
            out.push("--tools".to_string());
            out.extend(self.tools.iter().cloned());
        }

        // Upstream requires `--verbose` to be set when emitting stream-json output.
        //
        // This is surprising (the help text doesn't mention it), but without it the CLI exits
        // non-zero and emits an error to stderr.
        let stream_json_requires_verbose =
            matches!(self.output_format, ClaudeOutputFormat::StreamJson);
        if self.verbose || stream_json_requires_verbose {
            out.push("--verbose".to_string());
        }

        out.extend(self.extra_args.iter().cloned());

        if let Some(prompt) = self.prompt.as_ref() {
            out.push(prompt.clone());
        }

        out
    }
}
