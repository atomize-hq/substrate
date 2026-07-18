use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformPrincipalV1};

pub const BASH_PREEXEC_SCRIPT: &str = r#"# Substrate PTY command logging
substrate_home="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
if [[ -v SUBSTRATE_HOME && "$SUBSTRATE_HOME" != "$substrate_home" ]]; then
    return 1
fi
if [[ -v SUBSTRATE_ROOT && "$SUBSTRATE_ROOT" != "$substrate_home" ]]; then
    return 1
fi
if [[ -v SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1 && "$SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1" != "$substrate_install_context" ]]; then
    return 1
fi
export SUBSTRATE_HOME="$substrate_home"
export SUBSTRATE_ROOT="$substrate_home"
export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="$substrate_install_commitment"
export SUBSTRATE_INSTALL_PRIMARY_USER="$substrate_install_account"
export SUBSTRATE_INSTALL_PRIMARY_UID="$substrate_install_uid"
export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="$substrate_install_context"
substrate_manager_env="${substrate_home%/}/manager_env.sh"
if [[ -n "$substrate_manager_env" && -f "$substrate_manager_env" ]]; then
    # shellcheck disable=SC1090
    source "$substrate_manager_env"
fi

# Source user's bashrc ONLY in interactive shells
[[ $- == *i* ]] && [[ -f "$substrate_account_home/.bashrc" ]] && source "$substrate_account_home/.bashrc"

if [[ "${SUBSTRATE_ENABLE_PREEXEC:-0}" == "1" ]]; then
__substrate_json_escape() {
    local s="$1"
    s="${s//\\/\\\\}"
    s="${s//\"/\\\"}"
    s="${s//$'\n'/\\n}"
    s="${s//$'\r'/\\r}"
    s="${s//$'\t'/\\t}"
    printf '%s' "$s"
}
__substrate_preexec() {
    [[ -z "$SHIM_TRACE_LOG" ]] && return 0
    [[ "$BASH_COMMAND" == __substrate_preexec* ]] && return 0
    [[ -n "$COMP_LINE" ]] && return 0
    # Canonical trace MUST omit the raw command body; it can contain secrets.
    printf '{"ts":"%s","event_type":"builtin_command","session_id":"%s","component":"shell","pty":true,"preexec":true,"command_omitted":true,"parent_cmd_id":"%s"}\n' \
        "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)" \
        "$(__substrate_json_escape "${SHIM_SESSION_ID:-unknown}")" \
        "$(__substrate_json_escape "${SHIM_PARENT_CMD_ID:-}")" >> "$SHIM_TRACE_LOG" 2>/dev/null || true

    # Optional debug-only raw log (explicit opt-in). This may contain secrets.
    if [[ -n "${SUBSTRATE_PREEXEC_RAW_LOG:-}" ]]; then
        printf '{"ts":"%s","event_type":"builtin_command_raw","command":"%s","session_id":"%s","component":"shell","pty":true,"preexec":true,"may_contain_secrets":true,"parent_cmd_id":"%s"}\n' \
            "$(date -u +%Y-%m-%dT%H:%M:%S.%3NZ)" \
            "$(__substrate_json_escape "$BASH_COMMAND")" \
            "$(__substrate_json_escape "${SHIM_SESSION_ID:-unknown}")" \
            "$(__substrate_json_escape "${SHIM_PARENT_CMD_ID:-}")" >> "$SUBSTRATE_PREEXEC_RAW_LOG" 2>/dev/null || true
    fi
}
trap '__substrate_preexec' DEBUG
fi
"#;

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn render_bash_preexec_script(
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<String> {
    crate::execution::install_bootstrap::bind_unix_install_bootstrap_context(install_context)?;
    let encoded = install_context
        .encode()
        .context("failed to encode Bash preexec install bootstrap context")?;
    let account_home = crate::execution::install_bootstrap::unix_account_home_for_principal(
        &install_context.context.intended_host_principal,
    )?;
    let PlatformPrincipalV1::Unix { account, uid } =
        &install_context.context.intended_host_principal
    else {
        anyhow::bail!("Bash preexec requires a Unix principal");
    };
    Ok(format!(
        "substrate_install_commitment={}\nsubstrate_install_account={}\nsubstrate_install_uid={}\nsubstrate_install_context={}\nsubstrate_account_home={}\n{}",
        shell_single_quote(&install_context.host_context_commitment),
        shell_single_quote(account),
        shell_single_quote(&uid.to_string()),
        shell_single_quote(&encoded),
        shell_single_quote(&account_home.display().to_string()),
        BASH_PREEXEC_SCRIPT
    ))
}

pub fn write_bash_preexec_script(
    path: &Path,
    install_context: &InstallBootstrapContextCarrierV1,
) -> Result<()> {
    crate::execution::install_bootstrap::bind_unix_install_bootstrap_context(install_context)?;
    let expected =
        PathBuf::from(&install_context.context.selected_host_prefix).join(".substrate_preexec");
    if path != expected {
        anyhow::bail!("Bash preexec target does not match install bootstrap context");
    }
    let parent = path.parent().context("Bash preexec target has no parent")?;
    let parent_metadata = fs::symlink_metadata(parent).with_context(|| {
        format!(
            "failed to inspect Bash preexec parent at {}",
            parent.display()
        )
    })?;
    if !parent_metadata.file_type().is_dir() || parent_metadata.file_type().is_symlink() {
        anyhow::bail!("Bash preexec parent is not a no-follow directory");
    }
    fs::write(path, render_bash_preexec_script(install_context)?)
        .with_context(|| format!("failed to write bash preexec script at {}", path.display()))?;
    Ok(())
}
