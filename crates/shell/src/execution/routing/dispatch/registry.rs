//! Command registry and PTY classification helpers.

use std::io::{self, IsTerminal};
use std::path::Path;

/// Check if it's sudo that needs PTY for password prompt
pub(crate) fn sudo_wants_pty(cmd_lower: &str, tokens: &[String]) -> bool {
    if cmd_lower != "sudo" {
        return false;
    }

    // No PTY if -n/-S/-A or their long forms
    !tokens.iter().any(|t| {
        matches!(
            t.as_str(),
            "-n" | "--non-interactive" | "-S" | "--stdin" | "-A" | "--askpass"
        )
    })
}

/// Check if it's an interactive shell
pub(crate) fn is_interactive_shell(cmd_lower: &str, tokens: &[String]) -> bool {
    let is_shell = matches!(cmd_lower, "bash" | "zsh" | "sh" | "fish" | "dash" | "ksh");
    if !is_shell {
        return false;
    }

    // No PTY if executing command with -c
    let has_command = tokens.iter().any(|t| t == "-c");

    // Explicit interactive flag
    let has_interactive = tokens.iter().any(|t| t == "-i" || t == "--interactive");

    // It's interactive if: no -c flag OR explicit -i flag
    !has_command || has_interactive
}

/// Check if interpreter command looks like interactive REPL
pub(crate) fn looks_like_repl(cmd_lower: &str, tokens: &[String]) -> bool {
    let is_interp = matches!(
        cmd_lower,
        "python" | "python3" | "ipython" | "bpython" | "node" | "irb" | "pry"
    );
    if !is_interp {
        return false;
    }

    // Force interactive if -i/--interactive present, regardless of script/inline code
    let has_i = tokens.iter().any(|t| t == "-i" || t == "--interactive");
    if has_i {
        return true;
    }

    // Check for script file (any non-option argument after the command)
    let has_script = tokens.iter().skip(1).any(|t| !t.starts_with('-'));

    // Check for inline code execution flags
    let has_inline = tokens.iter().any(|t| {
        matches!(
            t.as_str(),
            "-c" |                                    // python
            "-e" | "--eval" | "-p" | "--print" // node
        )
    });

    // REPL when no script AND not inline
    !has_script && !has_inline
}

/// Check if it's a container/k8s command that needs PTY
pub(crate) fn container_wants_pty(cmd_lower: &str, tokens: &[String]) -> bool {
    // Check for "docker compose" (space-separated form)
    let is_docker_compose = cmd_lower == "docker"
        && tokens
            .get(1)
            .map(|s| s.as_str() == "compose")
            .unwrap_or(false);

    // Docker/Podman/docker-compose run|exec: only scan flags up to image/container name
    if matches!(cmd_lower, "docker" | "podman" | "docker-compose") || is_docker_compose {
        if let Some(subcmd_idx) = tokens.iter().position(|t| t == "run" || t == "exec") {
            let mut has_i = false;
            let mut has_t = false;

            for token in tokens.iter().skip(subcmd_idx + 1) {
                if token == "--" {
                    break;
                }
                if let Some(stripped) = token.strip_prefix('-') {
                    if token == "-it" || token == "-ti" {
                        return true;
                    }
                    if token == "-i" || token == "--interactive" || token == "--stdin" {
                        has_i = true;
                    }
                    if token == "-t" || token == "--tty" {
                        has_t = true;
                    }
                    if !token.starts_with("--") && !stripped.is_empty() {
                        let chars: Vec<char> = stripped.chars().collect();
                        if chars.contains(&'i') {
                            has_i = true;
                        }
                        if chars.contains(&'t') {
                            has_t = true;
                        }
                    }
                } else {
                    // First non-option = image (run) or container (exec)
                    break; // stop scanning; rest belongs to the in-container command
                }
            }
            // Need both -i and -t for interactive container session
            return has_i && has_t;
        }
    }

    // kubectl exec with proper flag detection (scoped to after exec, stop at --)
    if cmd_lower == "kubectl" {
        if let Some(exec_idx) = tokens.iter().position(|t| t == "exec") {
            let mut has_i = false;
            let mut has_t = false;

            // Only check flags after exec and before --
            for token in tokens.iter().skip(exec_idx + 1) {
                // Stop scanning at -- (rest are remote command args)
                if token == "--" {
                    break;
                }

                if token == "-it" || token == "-ti" {
                    return true;
                }
                if token == "-i" || token == "--stdin" {
                    has_i = true;
                }
                if token == "-t" || token == "--tty" {
                    has_t = true;
                }
                // Check for flags in clusters
                if token.starts_with("-") && !token.starts_with("--") && token.len() > 1 {
                    let chars: Vec<char> = token[1..].chars().collect();
                    if chars.contains(&'i') {
                        has_i = true;
                    }
                    if chars.contains(&'t') {
                        has_t = true;
                    }
                }
            }
            // kubectl: need both -i and -t for interactive exec
            return has_i && has_t;
        }
    }

    false
}

/// Check if command is launching an interactive debugger
pub(crate) fn wants_debugger_pty(cmd_lower: &str, tokens: &[String]) -> bool {
    // Python debuggers: python -m pdb/ipdb
    if cmd_lower == "python" || cmd_lower == "python3" {
        if let Some(i) = tokens.iter().position(|t| t == "-m") {
            if let Some(modname) = tokens.get(i + 1) {
                if modname == "pdb" || modname == "ipdb" {
                    return true;
                }
            }
        }
    }

    // Node debuggers: node inspect or node --inspect-brk
    if cmd_lower == "node"
        && tokens
            .iter()
            .any(|t| t == "inspect" || t == "--inspect" || t == "--inspect-brk")
    {
        return true;
    }

    false
}

/// Check if git command needs interactive PTY
pub(crate) fn git_wants_pty(tokens: &[String]) -> bool {
    // Skip "git"
    let mut i = 1;

    // Git global options that may appear before the subcommand.
    // Options that consume a value: -C <path>, -c <name=val>, --git-dir <path>, --work-tree <path>, --namespace <ns>
    while i < tokens.len() {
        let t = tokens[i].as_str();
        match t {
            "-C" | "-c" | "--git-dir" | "--work-tree" | "--namespace" => {
                i += 2; // skip option + value
            }
            _ if t.starts_with("--git-dir=")
                || t.starts_with("--work-tree=")
                || t.starts_with("--namespace=") =>
            {
                i += 1;
            }
            // First non-option token is the subcommand
            _ if !t.starts_with('-') => break,
            // Unknown global flag without value (safe to skip)
            _ => i += 1,
        }
    }

    if i >= tokens.len() {
        return false;
    }
    let sub = tokens[i].as_str();

    match sub {
        "add" => tokens.iter().any(|t| t == "-p" || t == "-i"),
        "rebase" => tokens.iter().any(|t| t == "-i"),
        "commit" => {
            // Scan all flags - -e/--edit can override -m/-F to open editor
            let mut no_editor = false;
            let mut force_editor = false;
            for t in tokens.iter().skip(i + 1) {
                if t == "-e" || t == "--edit" {
                    force_editor = true;
                }
                if t == "-m"
                    || t == "--message"
                    || t.starts_with("-m")
                    || t.starts_with("--message=")
                {
                    no_editor = true;
                }
                if t == "-F" || t == "--file" || t.starts_with("--file=") {
                    no_editor = true;
                }
                if t == "--no-edit" {
                    no_editor = true;
                    force_editor = false; // --no-edit overrides -e
                }
            }
            // Editor opens if forced OR if no message provided
            force_editor || !no_editor
        }
        _ => false,
    }
}

/// Check for shell metacharacters at top-level (not inside quotes, subshells, or backticks)
pub(crate) fn has_top_level_shell_meta(s: &str) -> bool {
    let mut in_single = false;
    let mut in_double = false;
    let mut in_backticks = false;
    let mut escape = false;
    let mut subshell_depth = 0;
    let mut chars = s.chars().peekable();

    while let Some(ch) = chars.next() {
        if escape {
            escape = false;
            continue;
        }

        // Check for $( subshell start
        if ch == '$' && !in_single && !in_backticks && chars.peek() == Some(&'(') {
            chars.next(); // consume '('
            subshell_depth += 1;
            continue;
        }

        match ch {
            '\\' if !in_single => {
                escape = true;
            }
            '`' if !in_single && !in_double && subshell_depth == 0 => {
                in_backticks = !in_backticks;
            }
            '\'' if !in_double && !in_backticks && subshell_depth == 0 => {
                in_single = !in_single;
            }
            '"' if !in_single && !in_backticks && subshell_depth == 0 => {
                in_double = !in_double;
            }
            '(' if !in_single && !in_double && !in_backticks && subshell_depth > 0 => {
                subshell_depth += 1;
            }
            ')' if !in_single && !in_double && !in_backticks && subshell_depth > 0 => {
                subshell_depth -= 1;
            }
            '|' | '>' | '<' | '&' | ';'
                if !in_single && !in_double && !in_backticks && subshell_depth == 0 =>
            {
                return true
            }
            _ => {}
        }
    }
    false
}

/// Strip known wrapper commands to find the actual command being run
pub(crate) fn peel_wrappers(tokens: &[String]) -> Vec<String> {
    if tokens.is_empty() {
        return tokens.to_vec();
    }

    let i = 0;
    if i < tokens.len() {
        let cmd = tokens[i].as_str();

        // Get base command name (strip path)
        let base_cmd = std::path::Path::new(cmd)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or(cmd);

        match base_cmd {
            // Wrappers that take 1 argument
            "sshpass" => {
                // sshpass -p pass cmd... or sshpass -f file cmd...
                if i + 1 < tokens.len()
                    && (tokens[i + 1] == "-p" || tokens[i + 1] == "-f")
                    && i + 3 < tokens.len()
                {
                    return tokens[i + 3..].to_vec(); // Skip sshpass -p pass
                }
                return tokens[i + 1..].to_vec(); // Skip just sshpass
            }
            "timeout" => {
                // timeout [opts] duration command...
                let mut j = i + 1;
                // Skip options
                while j < tokens.len() && tokens[j].starts_with('-') {
                    j += if tokens[j] == "-s" || tokens[j] == "--signal" {
                        2
                    } else {
                        1
                    };
                }
                // Skip duration
                if j < tokens.len() && !tokens[j].starts_with('-') {
                    j += 1;
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            "env" => {
                // env [-i] [-u NAME]... [VAR=val]... command...
                let mut j = i + 1;
                while j < tokens.len() {
                    let t = tokens[j].as_str();
                    match t {
                        "-i" => j += 1,                    // clear environment
                        "-u" => j += 2,                    // unset NAME
                        _ if t.starts_with('-') => j += 1, // other env flags
                        _ if t.contains('=') => j += 1,    // VAR=val
                        _ => break,                        // first real command
                    }
                }
                return tokens.get(j..).map(|s| s.to_vec()).unwrap_or_else(Vec::new);
            }
            "stdbuf" => {
                // stdbuf -oL|-eL|-iL command...
                let mut j = i + 1;
                while j < tokens.len() && tokens[j].starts_with('-') {
                    j += 1;
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            "nice" | "ionice" => {
                // nice [-n priority] command...
                let mut j = i + 1;
                if j < tokens.len() && tokens[j] == "-n" {
                    j += 2; // Skip -n and value
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            "doas" => {
                // doas [-u user] command... (sudo alternative)
                let mut j = i + 1;
                if j < tokens.len() && tokens[j] == "-u" {
                    j += 2; // Skip -u and user
                }
                if j < tokens.len() {
                    return tokens[j..].to_vec();
                }
                return vec![];
            }
            _ => return tokens.to_vec(), // Not a wrapper
        }
    }

    tokens.to_vec()
}

/// Determines if a command needs PTY allocation for proper terminal control
pub(crate) fn needs_pty(cmd: &str) -> bool {
    // For unit tests, skip actual TTY detection
    let is_test_mode = std::env::var("TEST_MODE").is_ok();

    // If parent stdio isn't a TTY, never use PTY (skip in test mode)
    if !is_test_mode {
        let stdin_is_tty = io::stdin().is_terminal();
        let stdout_is_tty = io::stdout().is_terminal();
        if !(stdin_is_tty && stdout_is_tty) {
            return false;
        }
    }

    // Optional: Enable pipeline-last TUI detection
    let enable_pipeline_last = std::env::var("SUBSTRATE_PTY_PIPELINE_LAST").is_ok();

    // Check for shell metacharacters at top-level (not inside quotes)
    if has_top_level_shell_meta(cmd) {
        // If pipeline-last is enabled, check if last segment needs PTY
        if enable_pipeline_last && cmd.contains('|') {
            // Simple heuristic: split by top-level pipes and check last segment
            // This is simplified - a full implementation would parse properly
            if let Some(last_segment) = cmd.rsplit('|').next() {
                // Check if output is redirected (>, <, >>, 1>, 2>, 2>&1, etc.)
                let has_redirect = last_segment.chars().any(|c| c == '>' || c == '<')
                    || last_segment.contains("&>");
                if !has_redirect {
                    // Recursively check if last segment needs PTY
                    return needs_pty(last_segment.trim());
                }
            }
        }
        return false;
    }

    // Conservative allowlist for known TUIs that definitely need PTY
    const KNOWN_TUIS: &[&str] = &[
        "vim", "vi", "nvim", "neovim", "nano", "emacs", // editors
        "less", "more", "most", // pagers
        "top", "htop", "btop", "glances", // monitors
        "telnet", "ftp", "sftp", // network tools
        "claude", "codex", "gemini", "atomize", // AI tools
        "tmux", "screen", "zellij", // multiplexers
        "fzf", "lazygit", "gitui", "tig", // git/file tools
        "ranger", "yazi", "k9s", "nmtui", // additional TUIs
        "ipython", "bpython", // interactive pythons
        "sqlite3", "psql",
        "mysql", // database CLIs
                 // Note: python, node, git, ssh handled by special logic
                 // 🔥 PRODUCTION FIX: Removed ssh from list since dedicated logic is comprehensive
    ];

    // Parse command properly using shell_words for quoted argument handling
    let tokens = match shell_words::split(cmd) {
        Ok(tokens) => tokens,
        Err(_) => {
            // Fallback: on Windows, accept bare paths like C:\Foo\bar.exe
            #[cfg(windows)]
            {
                vec![cmd.to_string()]
            }
            #[cfg(not(windows))]
            {
                return false; // Malformed command, don't use PTY
            }
        }
    };

    // Peel off wrapper commands to find the actual command
    let peeled_tokens = peel_wrappers(&tokens);

    // Use peeled tokens if available, otherwise original
    let working_tokens = if !peeled_tokens.is_empty() {
        &peeled_tokens
    } else {
        &tokens
    };

    // Windows-safe program extraction: prefer the program component from the original string
    #[cfg(windows)]
    let first_raw = {
        // Try to extract <...>.exe from the original string regardless of spaces
        let lower = cmd.to_ascii_lowercase();
        if let Some(pos) = lower.find(".exe") {
            &cmd[..pos + 4]
        } else {
            working_tokens.first().map(|s| s.as_str()).unwrap_or("")
        }
    };
    #[cfg(not(windows))]
    let first_raw = working_tokens.first().map(|s| s.as_str()).unwrap_or("");
    let first_token = Path::new(first_raw)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");

    // 🔥 EXPERT FIX: Convert to lowercase FIRST, then strip Windows extensions
    let lower = first_token.to_ascii_lowercase();
    let cmd_lower = if cfg!(windows) {
        lower
            .trim_end_matches(".exe")
            .trim_end_matches(".cmd")
            .trim_end_matches(".bat")
            .to_string()
    } else {
        lower
    };

    // Check for sudo (needs PTY for password prompt)
    if sudo_wants_pty(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's an interactive shell
    if is_interactive_shell(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's an interactive REPL
    if looks_like_repl(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's launching a debugger
    if wants_debugger_pty(&cmd_lower, working_tokens) {
        return true;
    }

    // Check for container/k8s commands
    if container_wants_pty(&cmd_lower, working_tokens) {
        return true;
    }

    // Check if it's an interactive git command
    if cmd_lower == "git" && git_wants_pty(working_tokens) {
        return true;
    }

    // Special SSH handling for -t/-T flags and remote commands
    if cmd_lower == "ssh" {
        // If no args at all, assume interactive client
        if working_tokens.len() == 1 {
            return true;
        }
        // Create lowercase versions for case-insensitive option checking
        let tokens_lc: Vec<String> = working_tokens
            .iter()
            .map(|t| t.to_ascii_lowercase())
            .collect();

        // Check for explicit -t or -tt flag (force PTY)
        let has_t = tokens_lc.iter().any(|arg| arg == "-t" || arg == "-tt");

        // Check for explicit -T flag (no PTY) - uppercase T
        if working_tokens.iter().any(|arg| arg == "-T") {
            return false;
        }

        // Check for -N flag (no remote command, typically for port forwarding)
        // Only deny PTY if -t/-tt not present
        if working_tokens.iter().any(|arg| arg == "-N") && !has_t {
            return false;
        }

        // Check for -O control operations (check|exit|stop|forward|cancel)
        if working_tokens.iter().any(|arg| arg == "-O") && !has_t {
            return false;
        }

        // Check for -W (stdio forwarding) - never PTY unless -t is explicit
        if tokens_lc.iter().any(|arg| arg == "-w") && !has_t {
            return false;
        }

        // If -t or -tt was present, force PTY
        if has_t {
            return true;
        }

        // Check for BatchMode=yes (case-insensitive, no PTY)
        // First check inline form: -oBatchMode=yes
        for arg in &tokens_lc {
            if let Some(val) = arg.strip_prefix("-obatchmode=") {
                if val == "yes" {
                    return false;
                }
            }
        }
        // Check spaced form: -o BatchMode=yes or -o BatchMode = yes
        for (i, arg) in tokens_lc.iter().enumerate() {
            if arg == "-o" && i + 1 < tokens_lc.len() {
                // Handle: -o BatchMode=yes
                if tokens_lc[i + 1] == "batchmode=yes" {
                    return false;
                }
                // Handle: -o BatchMode = yes (with spaces)
                if tokens_lc[i + 1] == "batchmode"
                    && i + 3 < tokens_lc.len()
                    && tokens_lc[i + 2] == "="
                    && tokens_lc[i + 3] == "yes"
                {
                    return false;
                }
            }
        }

        // Check for RequestTTY option (case-insensitive, ssh_config style)
        // First check spaced form: -o RequestTTY=value or -o RequestTTY = value
        for (i, arg) in tokens_lc.iter().enumerate() {
            if arg == "-o" && i + 1 < tokens_lc.len() {
                // Handle: -o RequestTTY=value
                if let Some(val) = tokens_lc[i + 1].strip_prefix("requesttty=") {
                    match val {
                        "yes" | "force" => return true,
                        "no" => return false,
                        _ => {}
                    }
                }
                // Handle: -o RequestTTY = value (with spaces)
                if tokens_lc[i + 1] == "requesttty"
                    && i + 3 < tokens_lc.len()
                    && tokens_lc[i + 2] == "="
                {
                    match tokens_lc[i + 3].as_str() {
                        "yes" | "force" => return true,
                        "no" => return false,
                        _ => {}
                    }
                }
            }
        }

        // Check inline form: -oRequestTTY=value
        for arg in &tokens_lc {
            if let Some(val) = arg.strip_prefix("-orequesttty=") {
                match val {
                    "yes" | "force" => return true,
                    "no" => return false,
                    _ => {}
                }
            }
        }

        // Handle all 2-arg SSH options (not just -l)
        // 🔥 EXPERT FIX: Skip ALL 2-arg options to correctly identify host
        let mut skip_next = false;
        let mut host_idx = None;
        // Start from index 1 to skip "ssh" itself
        for i in 1..working_tokens.len() {
            let arg = &working_tokens[i];
            if skip_next {
                skip_next = false;
                continue;
            }
            // Skip all 2-arg SSH options: -p -l -i -F -J -b -c -D -L -R -S -E -B -o
            if matches!(
                arg.as_str(),
                "-p" | "-l"
                    | "-i"
                    | "-F"
                    | "-J"
                    | "-b"
                    | "-c"
                    | "-D"
                    | "-L"
                    | "-R"
                    | "-S"
                    | "-E"
                    | "-B"
            ) {
                skip_next = true;
                continue;
            }
            // Handle -o option (can be -o key=val or -okey=val)
            if arg == "-o" {
                skip_next = true;
                continue;
            }
            // Stop at -- delimiter
            if arg == "--" {
                if i + 1 < working_tokens.len() {
                    host_idx = Some(i + 1);
                }
                break;
            }
            // First non-option is the host
            if !arg.starts_with('-') && !arg.contains('=') {
                host_idx = Some(i);
                break;
            }
        }

        // Check if there's a remote command after the host
        if let Some(idx) = host_idx {
            if idx + 1 < working_tokens.len() {
                // There's a remote command, no explicit -t, so no PTY
                return false;
            }
        }

        // 🔥 CRITICAL FIX: No -T/-W/BatchMode, no remote command => interactive login
        return true;
    }

    // Check if it's a known TUI
    KNOWN_TUIS.iter().any(|&tui| cmd_lower == tui)
}

/// Force PTY for specific command (user override)
pub(crate) fn is_force_pty_command(cmd: &str) -> bool {
    cmd.starts_with(":pty ") || std::env::var("SUBSTRATE_FORCE_PTY").is_ok()
}

/// Check if PTY is disabled globally
pub(crate) fn is_pty_disabled() -> bool {
    std::env::var("SUBSTRATE_DISABLE_PTY").is_ok()
}

pub(crate) fn parse_demo_burst_command(input: &str) -> Option<(usize, usize, u64)> {
    let rest = input.strip_prefix(":demo-burst")?.trim();
    if rest.is_empty() {
        return Some((4, 400, 0));
    }

    let mut parts = rest.split_whitespace();
    let agents = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(4);
    let events = parts
        .next()
        .and_then(|s| s.parse::<usize>().ok())
        .unwrap_or(400);
    let delay_ms = parts
        .next()
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(0);

    Some((agents, events, delay_ms))
}
