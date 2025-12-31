use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub const BASH_PREEXEC_SCRIPT: &str = r#"# Substrate PTY command logging
substrate_home="${SUBSTRATE_HOME:-$HOME/.substrate}"
substrate_manager_env="${substrate_home%/}/manager_env.sh"
if [[ -n "$substrate_manager_env" && -f "$substrate_manager_env" ]]; then
    # shellcheck disable=SC1090
    source "$substrate_manager_env"
fi

# Source user's bashrc ONLY in interactive shells
[[ $- == *i* ]] && [[ -f ~/.bashrc ]] && source ~/.bashrc

if [[ "${SUBSTRATE_ENABLE_PREEXEC:-0}" == "1" ]]; then
__substrate_preexec() {
    [[ -z "$SHIM_TRACE_LOG" ]] && return 0
    [[ "$BASH_COMMAND" == __substrate_preexec* ]] && return 0
    [[ -n "$COMP_LINE" ]] && return 0
    printf '{"ts":"%s","event_type":"builtin_command","command":%q,"session_id":%q,"component":"shell","pty":true}\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)" \
        "$BASH_COMMAND" \
        "${SHIM_SESSION_ID:-unknown}" >> "$SHIM_TRACE_LOG" 2>/dev/null || true
}
trap '__substrate_preexec' DEBUG
fi
"#;

pub fn write_bash_preexec_script(path: &Path) -> Result<()> {
    fs::write(path, BASH_PREEXEC_SCRIPT)
        .with_context(|| format!("failed to write bash preexec script at {}", path.display()))?;
    Ok(())
}
