//! Exec-time guardrails for host-mounted toolchain binaries in host-visible worlds.
//!
//! Reference: `docs/reference/config/world.md`

use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub const OVERRIDE_WORLD_EXEC_GUARD_ENV: &str = "SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD";
pub const OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS_ENV: &str =
    "SUBSTRATE_OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS";

const DEFAULT_DENY_CONTAINS: [&str; 6] = [
    "/.nvm/",
    "/.config/nvm/",
    "/.pyenv/",
    "/.cargo/bin/",
    "/.local/bin/",
    "/.bun/bin/",
];

const DEFAULT_WORLD_DEPS_BIN: &str = "/var/lib/substrate/world-deps/bin";
const BASELINE_PATH: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorldExecGuardDeny {
    pub resolved_executable: String,
    pub matched_substring: String,
}

#[derive(Debug, Clone)]
struct WorldExecGuardConfig {
    enabled: bool,
    deny_contains: Vec<String>,
}

pub fn deny_message(deny: &WorldExecGuardDeny) -> String {
    format!(
        "substrate: error: host toolchain execution denied in host-visible world (exit=5)\n\
         substrate: reason: resolved executable path matched denylist (matched={matched:?})\n\
         substrate: resolved_executable: {exe}\n\
         substrate: hint: enable a world-deps package instead of executing host-mounted toolchains\n\
         substrate: hint: to override, set {guard_env}=0 (disable) OR set {deny_env}=\"<substr1>,<substr2>,...\" (replace denylist) if you accept the risk\n",
        matched = deny.matched_substring,
        exe = deny.resolved_executable,
        guard_env = OVERRIDE_WORLD_EXEC_GUARD_ENV,
        deny_env = OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS_ENV
    )
}

pub fn check_command(
    cmd: &str,
    cwd: &Path,
    env: &HashMap<String, String>,
    host_visible: bool,
) -> Option<WorldExecGuardDeny> {
    let cfg = resolve_config(env, host_visible);
    if !cfg.enabled || cfg.deny_contains.is_empty() {
        return None;
    }

    let computed_baseline_path = baseline_path(env);
    let path_var = env
        .get("PATH")
        .map(String::as_str)
        .unwrap_or(computed_baseline_path.as_str());
    check_shell_command_inner(cmd, cwd, path_var, &cfg, 0)
}

fn baseline_path(env: &HashMap<String, String>) -> String {
    let world_deps_bin = env
        .get("SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR")
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| DEFAULT_WORLD_DEPS_BIN.to_string());
    let world_deps_bin_norm = world_deps_bin.trim_end_matches('/');
    format!("{world_deps_bin_norm}:{BASELINE_PATH}")
}

fn resolve_config(env: &HashMap<String, String>, host_visible: bool) -> WorldExecGuardConfig {
    let enabled_default = host_visible;
    let enabled = env
        .get(OVERRIDE_WORLD_EXEC_GUARD_ENV)
        .and_then(|raw| parse_bool(raw))
        .unwrap_or(enabled_default);

    let deny_contains = if env.contains_key(OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS_ENV) {
        let raw = env
            .get(OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS_ENV)
            .map(String::as_str)
            .unwrap_or("");
        raw.split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect()
    } else {
        DEFAULT_DENY_CONTAINS
            .iter()
            .map(|s| s.to_string())
            .collect()
    };

    WorldExecGuardConfig {
        enabled,
        deny_contains,
    }
}

fn parse_bool(raw: &str) -> Option<bool> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[derive(Debug, Clone)]
enum Token {
    Word(String),
    Op(Op),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Op {
    Seq,
    AndIf,
    OrIf,
    Pipe,
    Background,
    Newline,
    Redir,
}

fn check_shell_command_inner(
    cmd: &str,
    cwd: &Path,
    path_var: &str,
    cfg: &WorldExecGuardConfig,
    depth: usize,
) -> Option<WorldExecGuardDeny> {
    const MAX_DEPTH: usize = 4;
    if depth >= MAX_DEPTH {
        return None;
    }

    let tokens = tokenize_shellish(cmd)?;
    let mut current: Vec<Token> = Vec::new();

    for token in tokens {
        match token {
            Token::Op(op) if is_boundary(op) => {
                if let Some(deny) = check_simple_command(&current, cwd, path_var, cfg, depth) {
                    return Some(deny);
                }
                current.clear();
            }
            other => current.push(other),
        }
    }

    check_simple_command(&current, cwd, path_var, cfg, depth)
}

fn check_simple_command(
    tokens: &[Token],
    cwd: &Path,
    path_var: &str,
    cfg: &WorldExecGuardConfig,
    depth: usize,
) -> Option<WorldExecGuardDeny> {
    let words = words_excluding_redirections(tokens);
    if words.is_empty() {
        return None;
    }

    let mut idx = 0usize;
    while idx < words.len() && is_assignment_word(&words[idx]) {
        idx += 1;
    }
    let program = words.get(idx)?;
    let args = &words[idx.saturating_add(1)..];

    if let Some(deny) = check_program_word(program, cwd, path_var, cfg) {
        return Some(deny);
    }

    // Wrapper handling to reduce trivial bypasses.
    if let Some(deny) = check_wrapper_invocation(program, args, cwd, path_var, cfg, depth) {
        return Some(deny);
    }

    None
}

fn check_wrapper_invocation(
    program: &str,
    args: &[String],
    cwd: &Path,
    path_var: &str,
    cfg: &WorldExecGuardConfig,
    depth: usize,
) -> Option<WorldExecGuardDeny> {
    let basename = Path::new(program)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(program);

    match basename {
        // `env VAR=... cmd ...` runs `cmd` directly.
        "env" => {
            let mut i = 0usize;
            while i < args.len() {
                let a = &args[i];
                if a == "--" {
                    i += 1;
                    break;
                }
                if a.starts_with('-') {
                    i += 1;
                    continue;
                }
                if is_assignment_word(a) {
                    i += 1;
                    continue;
                }
                break;
            }
            let nested = args.get(i)?;
            if let Some(deny) = check_program_word(nested, cwd, path_var, cfg) {
                return Some(deny);
            }
            // If the nested command is itself a shell `-c`, recurse on its script.
            let nested_args = &args.get(i + 1..).unwrap_or(&[]);
            check_wrapper_invocation(nested, nested_args, cwd, path_var, cfg, depth)
        }
        // `command [opts] cmd ...` runs cmd directly.
        "command" => {
            let mut i = 0usize;
            while i < args.len() && args[i].starts_with('-') {
                i += 1;
            }
            let nested = args.get(i)?;
            if let Some(deny) = check_program_word(nested, cwd, path_var, cfg) {
                return Some(deny);
            }
            let nested_args = &args.get(i + 1..).unwrap_or(&[]);
            check_wrapper_invocation(nested, nested_args, cwd, path_var, cfg, depth)
        }
        // `sudo [opts] cmd ...` runs cmd directly.
        "sudo" => {
            let mut i = 0usize;
            while i < args.len() {
                let a = &args[i];
                if a == "--" {
                    i += 1;
                    break;
                }
                if a.starts_with('-') {
                    i += 1;
                    continue;
                }
                break;
            }
            let nested = args.get(i)?;
            if let Some(deny) = check_program_word(nested, cwd, path_var, cfg) {
                return Some(deny);
            }
            let nested_args = &args.get(i + 1..).unwrap_or(&[]);
            check_wrapper_invocation(nested, nested_args, cwd, path_var, cfg, depth)
        }
        // Nested shells: `sh -c '...'`, `bash -lc '...'`, etc.
        "sh" | "bash" | "dash" | "ksh" | "zsh" => {
            let mut i = 0usize;
            while i < args.len() {
                let a = &args[i];
                if a == "-c" || a == "-lc" || a == "-ic" || a == "-lic" {
                    let script = args.get(i + 1)?;
                    return check_shell_command_inner(script, cwd, path_var, cfg, depth + 1);
                }
                i += 1;
            }
            None
        }
        _ => None,
    }
}

fn check_program_word(
    word: &str,
    cwd: &Path,
    path_var: &str,
    cfg: &WorldExecGuardConfig,
) -> Option<WorldExecGuardDeny> {
    let resolved = resolve_executable(word, cwd, path_var)?;
    let resolved_str = resolved.to_string_lossy().to_string();
    for needle in &cfg.deny_contains {
        if needle.is_empty() {
            continue;
        }
        if resolved_str.contains(needle) {
            return Some(WorldExecGuardDeny {
                resolved_executable: resolved_str,
                matched_substring: needle.clone(),
            });
        }
    }
    None
}

fn resolve_executable(word: &str, cwd: &Path, path_var: &str) -> Option<PathBuf> {
    if word.contains('/') {
        let path = PathBuf::from(word);
        let path = if path.is_absolute() {
            path
        } else {
            cwd.join(path)
        };
        return Some(canonicalize_or(path));
    }

    let path_var = path_var.trim();
    if path_var.is_empty() {
        return None;
    }

    for seg in path_var.split(':') {
        let seg = if seg.is_empty() { "." } else { seg };
        let base = if seg == "." { cwd } else { Path::new(seg) };
        let candidate = base.join(word);
        if is_executable_file(&candidate) {
            return Some(canonicalize_or(candidate));
        }
    }

    None
}

fn canonicalize_or(path: PathBuf) -> PathBuf {
    std::fs::canonicalize(&path).unwrap_or(path)
}

fn is_executable_file(path: &Path) -> bool {
    let meta = match std::fs::metadata(path) {
        Ok(m) => m,
        Err(_) => return false,
    };
    if !meta.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        (meta.permissions().mode() & 0o111) != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

fn is_boundary(op: Op) -> bool {
    matches!(
        op,
        Op::Seq | Op::AndIf | Op::OrIf | Op::Pipe | Op::Background | Op::Newline
    )
}

fn words_excluding_redirections(tokens: &[Token]) -> Vec<String> {
    let mut out = Vec::new();
    let mut skip_next_word = false;
    for token in tokens {
        match token {
            Token::Op(Op::Redir) => {
                skip_next_word = true;
            }
            Token::Word(w) => {
                if skip_next_word {
                    skip_next_word = false;
                    continue;
                }
                out.push(w.clone());
            }
            Token::Op(_) => {}
        }
    }
    out
}

fn is_assignment_word(word: &str) -> bool {
    let mut parts = word.splitn(2, '=');
    let Some(name) = parts.next() else {
        return false;
    };
    let Some(_value) = parts.next() else {
        return false;
    };
    is_valid_sh_identifier(name)
}

fn is_valid_sh_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }
    chars.all(|c| c == '_' || c.is_ascii_alphanumeric())
}

fn tokenize_shellish(cmd: &str) -> Option<Vec<Token>> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut buf = String::new();
    let mut chars = cmd.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    let flush_buf = |tokens: &mut Vec<Token>, buf: &mut String| {
        if !buf.is_empty() {
            tokens.push(Token::Word(std::mem::take(buf)));
        }
    };

    while let Some(ch) = chars.next() {
        if in_single {
            if ch == '\'' {
                in_single = false;
            } else {
                buf.push(ch);
            }
            continue;
        }

        if in_double {
            match ch {
                '"' => {
                    in_double = false;
                }
                '\\' => {
                    if let Some(next) = chars.next() {
                        buf.push(next);
                    }
                }
                _ => buf.push(ch),
            }
            continue;
        }

        match ch {
            '\'' => in_single = true,
            '"' => in_double = true,
            '\\' => {
                if let Some(next) = chars.next() {
                    buf.push(next);
                }
            }
            '\n' => {
                flush_buf(&mut tokens, &mut buf);
                tokens.push(Token::Op(Op::Newline));
            }
            // Treat `#` as a comment leader only at a word boundary.
            '#' if buf.is_empty() => {
                for next in chars.by_ref() {
                    if next == '\n' {
                        tokens.push(Token::Op(Op::Newline));
                        break;
                    }
                }
            }
            // Whitespace.
            c if c.is_whitespace() => {
                flush_buf(&mut tokens, &mut buf);
            }
            // Control operators.
            ';' => {
                flush_buf(&mut tokens, &mut buf);
                tokens.push(Token::Op(Op::Seq));
            }
            '&' => {
                flush_buf(&mut tokens, &mut buf);
                if chars.peek() == Some(&'&') {
                    let _ = chars.next();
                    tokens.push(Token::Op(Op::AndIf));
                } else {
                    tokens.push(Token::Op(Op::Background));
                }
            }
            '|' => {
                flush_buf(&mut tokens, &mut buf);
                if chars.peek() == Some(&'|') {
                    let _ = chars.next();
                    tokens.push(Token::Op(Op::OrIf));
                } else {
                    tokens.push(Token::Op(Op::Pipe));
                }
            }
            // Redirections (including fd-prefixed redirections like `2>` handled below).
            '<' | '>' => {
                flush_buf(&mut tokens, &mut buf);
                // Consume a few common multi-char redirection forms (>>, <<, <&, >&).
                let first = ch;
                if chars.peek() == Some(&first) {
                    let _ = chars.next();
                }
                if chars.peek() == Some(&'&') {
                    let _ = chars.next();
                }
                tokens.push(Token::Op(Op::Redir));
            }
            // fd-prefixed redirections like `2>`, `10>>`, etc.
            d if d.is_ascii_digit() && buf.is_empty() => {
                let mut digits = String::new();
                digits.push(d);
                while matches!(chars.peek(), Some(p) if p.is_ascii_digit()) {
                    digits.push(chars.next().expect("peeked digit"));
                }
                match chars.peek() {
                    Some('<') | Some('>') => {
                        // Consume the redirection operator and emit as a redir token.
                        let op = chars.next().expect("peeked redir");
                        if chars.peek() == Some(&op) {
                            let _ = chars.next();
                        }
                        if chars.peek() == Some(&'&') {
                            let _ = chars.next();
                        }
                        tokens.push(Token::Op(Op::Redir));
                    }
                    _ => {
                        // Not a redirection; treat as a normal word prefix.
                        buf.push_str(&digits);
                    }
                }
            }
            _ => buf.push(ch),
        }
    }

    if in_single || in_double {
        // Unbalanced quotes; fail open (do not enforce based on partial parse).
        return None;
    }

    flush_buf(&mut tokens, &mut buf);
    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_executable(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create executable parent dir");
        }
        fs::write(path, "#!/bin/sh\necho wdh2-should-not-run\n").expect("write executable");

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(path).expect("stat executable").permissions();
            perms.set_mode(0o755);
            fs::set_permissions(path, perms).expect("chmod executable");
        }
    }

    #[test]
    fn denies_explicit_toolchain_path_when_enabled() {
        let tmp = tempdir().unwrap();
        let cwd = tmp.path();
        let blocked = cwd.join("home/.cargo/bin/wdh2-blocked");
        write_executable(&blocked);

        let mut env: HashMap<String, String> = HashMap::new();
        env.insert(OVERRIDE_WORLD_EXEC_GUARD_ENV.to_string(), "1".to_string());

        let deny = check_command(&blocked.to_string_lossy(), cwd, &env, true)
            .expect("expected deny for explicit toolchain path");
        assert_eq!(deny.matched_substring, "/.cargo/bin/");
        assert!(
            deny.resolved_executable.contains("/.cargo/bin/"),
            "expected resolved executable to contain toolchain segment"
        );

        let message = deny_message(&deny);
        assert!(
            message.contains("enable a world-deps package instead"),
            "expected remediation message"
        );
        assert!(message.contains(OVERRIDE_WORLD_EXEC_GUARD_ENV));
        assert!(message.contains(OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS_ENV));
    }

    #[test]
    fn does_not_deny_when_disabled_by_override() {
        let tmp = tempdir().unwrap();
        let cwd = tmp.path();
        let blocked = cwd.join("home/.cargo/bin/wdh2-blocked");
        write_executable(&blocked);

        let mut env: HashMap<String, String> = HashMap::new();
        env.insert(OVERRIDE_WORLD_EXEC_GUARD_ENV.to_string(), "0".to_string());

        assert!(
            check_command(&blocked.to_string_lossy(), cwd, &env, true).is_none(),
            "expected guard to be disabled by override"
        );
    }

    #[test]
    fn denylist_override_replaces_default() {
        let tmp = tempdir().unwrap();
        let cwd = tmp.path();
        let blocked = cwd.join("home/.cargo/bin/wdh2-blocked");
        write_executable(&blocked);

        let mut env: HashMap<String, String> = HashMap::new();
        env.insert(OVERRIDE_WORLD_EXEC_GUARD_ENV.to_string(), "1".to_string());
        env.insert(
            OVERRIDE_WORLD_EXEC_GUARD_DENY_CONTAINS_ENV.to_string(),
            "/.pyenv/".to_string(),
        );

        assert!(
            check_command(&blocked.to_string_lossy(), cwd, &env, true).is_none(),
            "expected denylist replacement to prevent matching default needles"
        );
    }
}
