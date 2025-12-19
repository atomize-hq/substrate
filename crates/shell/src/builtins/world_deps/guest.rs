use crate::execution::{build_agent_client_and_request, stream_non_pty_via_agent};
use anyhow::{bail, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::env;
use std::error::Error as StdError;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use substrate_common::paths as substrate_paths;
use tokio::runtime::Runtime;
use which::which;

static WORLD_EXEC_FALLBACK: AtomicBool = AtomicBool::new(false);

pub(crate) struct HostDetectionResult {
    pub detected: bool,
    pub reason: Option<String>,
}

enum HostDetectionContext {
    Bash(HostBashContext),
    Legacy { reason: Option<String> },
}

struct HostBashContext {
    bash_path: PathBuf,
    manager_env_path: PathBuf,
    original_bash_env: Option<String>,
}

pub(crate) fn detect_host(commands: &[String]) -> HostDetectionResult {
    let context = resolve_host_detection_context();
    match &context {
        HostDetectionContext::Legacy { reason } => {
            let detected = commands.iter().any(|cmd| run_host_command(cmd));
            HostDetectionResult {
                detected,
                reason: if detected { None } else { reason.clone() },
            }
        }
        HostDetectionContext::Bash(bash_ctx) => HostDetectionResult {
            detected: commands
                .iter()
                .any(|cmd| run_host_detection_command(cmd, bash_ctx)),
            reason: None,
        },
    }
}

pub(crate) fn detect_guest(commands: &[String]) -> Result<bool> {
    for cmd in commands {
        if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
            if run_host_command(cmd) {
                return Ok(true);
            }
            continue;
        }

        let wrapped = wrap_for_bash(cmd, false);
        match run_world_command(&wrapped) {
            Ok(response) => {
                if response.exit == 0 {
                    return Ok(true);
                }
            }
            Err(err) if should_fallback_to_host(&err) => {
                mark_world_exec_unavailable(&err);
                if run_host_command(cmd) {
                    return Ok(true);
                }
            }
            Err(err) => return Err(err),
        }
    }
    Ok(false)
}

fn resolve_host_detection_context() -> HostDetectionContext {
    if cfg!(windows) {
        return HostDetectionContext::Legacy { reason: None };
    }

    let bash_path = match which("bash") {
        Ok(path) => path,
        Err(_) => {
            return HostDetectionContext::Legacy {
                reason: Some(
                    "bash not found; host detection requires bash to load manager init".to_string(),
                ),
            };
        }
    };

    let manager_env_path = match resolve_manager_env_path() {
        Ok(path) => path,
        Err(err) => return HostDetectionContext::Legacy { reason: Some(err) },
    };

    if !manager_env_path.exists() {
        return HostDetectionContext::Legacy {
            reason: Some(format!(
                "manager env script missing at {}",
                manager_env_path.display()
            )),
        };
    }

    let original_bash_env = env::var("BASH_ENV").ok().and_then(|value| {
        let manager_env = manager_env_path.display().to_string();
        if value == manager_env {
            None
        } else {
            Some(value)
        }
    });

    HostDetectionContext::Bash(HostBashContext {
        bash_path,
        manager_env_path,
        original_bash_env,
    })
}

fn resolve_manager_env_path() -> std::result::Result<PathBuf, String> {
    if let Ok(path) = env::var("SUBSTRATE_MANAGER_ENV") {
        if path.trim().is_empty() {
            return Err("SUBSTRATE_MANAGER_ENV is set but empty".to_string());
        }
        return Ok(PathBuf::from(path));
    }

    substrate_paths::substrate_home()
        .map(|home| home.join("manager_env.sh"))
        .map_err(|err| {
            format!(
                "failed to resolve Substrate home for manager env: {:#}",
                err
            )
        })
}

pub(crate) fn install_in_guest(script: &str, verbose: bool) -> Result<()> {
    if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
        return run_host_install(script, verbose);
    }

    let command = wrap_for_bash(&wrap_guest_install_script(script), true);
    if verbose {
        match stream_non_pty_via_agent(&command) {
            Ok(outcome) => {
                if outcome.exit_code == 0 {
                    Ok(())
                } else {
                    bail!("installer exited with status {}", outcome.exit_code);
                }
            }
            Err(err) if should_fallback_to_host(&err) => {
                mark_world_exec_unavailable(&err);
                run_host_install(script, verbose)
            }
            Err(err) => Err(err),
        }
    } else {
        match run_world_command(&command) {
            Ok(response) => {
                if response.exit == 0 {
                    Ok(())
                } else {
                    let stdout = BASE64
                        .decode(response.stdout_b64.as_bytes())
                        .unwrap_or_default();
                    let stderr = BASE64
                        .decode(response.stderr_b64.as_bytes())
                        .unwrap_or_default();
                    eprintln!("{}", String::from_utf8_lossy(&stdout));
                    eprintln!("{}", String::from_utf8_lossy(&stderr));
                    bail!("installer exited with status {}", response.exit);
                }
            }
            Err(err) if should_fallback_to_host(&err) => {
                mark_world_exec_unavailable(&err);
                run_host_install(script, verbose)
            }
            Err(err) => Err(err),
        }
    }
}

pub(crate) fn world_exec_fallback_active() -> bool {
    WORLD_EXEC_FALLBACK.load(Ordering::SeqCst)
}

pub(crate) fn mark_world_exec_unavailable(err: &anyhow::Error) {
    let previously = WORLD_EXEC_FALLBACK.swap(true, Ordering::SeqCst);
    if !previously {
        eprintln!(
            "substrate: warn: world backend unavailable for world deps (falling back to host execution): {:#}",
            err
        );
    }
}

fn run_host_detection_command(command: &str, bash_ctx: &HostBashContext) -> bool {
    let mut cmd = Command::new(&bash_ctx.bash_path);
    cmd.arg("-c").arg(command);
    // Use BASH_ENV to source the manager env without touching user rc files.
    cmd.env("BASH_ENV", &bash_ctx.manager_env_path);
    cmd.env("SUBSTRATE_MANAGER_ENV", &bash_ctx.manager_env_path);
    if let Some(original) = &bash_ctx.original_bash_env {
        cmd.env("SUBSTRATE_ORIGINAL_BASH_ENV", original);
    } else {
        cmd.env_remove("SUBSTRATE_ORIGINAL_BASH_ENV");
    }
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.status().map(|status| status.success()).unwrap_or(false)
}

pub(crate) fn run_host_command(command: &str) -> bool {
    let mut cmd = if cfg!(windows) {
        let mut c = Command::new("cmd");
        c.arg("/C").arg(command);
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c").arg(command);
        c
    };
    cmd.stdout(Stdio::null());
    cmd.stderr(Stdio::null());
    cmd.status().map(|status| status.success()).unwrap_or(false)
}

pub(crate) fn run_guest_install(script: &str, verbose: bool) -> Result<()> {
    install_in_guest(script, verbose)
}

fn run_world_command(command: &str) -> Result<agent_api_types::ExecuteResponse> {
    let (client, request, _) = build_agent_client_and_request(command)?;
    let rt = Runtime::new()?;
    let response = rt.block_on(async move { client.execute(request).await })?;
    Ok(response)
}

fn run_host_install(script: &str, verbose: bool) -> Result<()> {
    eprintln!("substrate: warn: world backend unavailable; running installer on the host.");
    let body = build_bash_body(script, true);
    let mut cmd = Command::new("bash");
    cmd.arg("-lc").arg(&body);
    if verbose {
        let status = cmd.status()?;
        if status.success() {
            Ok(())
        } else {
            bail!(
                "installer exited with status {}",
                status.code().unwrap_or(-1)
            );
        }
    } else {
        let output = cmd.output()?;
        if output.status.success() {
            Ok(())
        } else {
            eprintln!("{}", String::from_utf8_lossy(&output.stdout));
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            bail!(
                "installer exited with status {}",
                output.status.code().unwrap_or(-1)
            );
        }
    }
}

fn wrap_for_bash(script: &str, strict: bool) -> String {
    let body = build_bash_body(script, strict);
    let escaped = body.replace('\'', "'\"'\"'");
    format!("bash -lc '{}'", escaped)
}

fn wrap_guest_install_script(script: &str) -> String {
    // `sudo` can fail inside some world environments (e.g. userns where only uid 0 is mapped).
    // For guest installs we prefer to run commands directly when already root, and otherwise
    // fall back to invoking the real sudo.
    //
    // Must handle common sudo flags used by recipes (e.g. `sudo -E bash -`, `sudo -n`).
    let prelude = r#"
substrate_sudo() {
  if [ "$(id -u)" -eq 0 ]; then
    # Strip common sudo flags when we're already root (options are for sudo, not the target cmd).
    while [ "$#" -gt 0 ]; do
      case "$1" in
        -E|-n|-S|-k|-H) shift ;;
        --) shift; break ;;
        -*) shift ;;
        *) break ;;
      esac
    done
    "$@"
  else
    command sudo "$@"
  fi
}
sudo() { substrate_sudo "$@"; }
"#;

    format!("{prelude}\n{script}")
}

fn build_bash_body(script: &str, strict: bool) -> String {
    let mut body = String::new();
    if strict {
        body.push_str("set -euo pipefail; ");
    }
    body.push_str(script);
    body
}

fn should_fallback_to_host(err: &anyhow::Error) -> bool {
    if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
        return true;
    }
    let mut current: Option<&(dyn StdError + 'static)> = Some(err.as_ref());
    while let Some(err) = current {
        let message = err.to_string();
        if message.contains("world-agent")
            || message.contains("platform world context")
            || message.contains("world backend")
            // Connectivity/transport failures should degrade to host execution.
            || message.contains("connect UDS")
            || message.contains("unix socket")
            || message.contains("Connection refused")
            || message.contains("connection refused")
            || message.contains("timed out")
            || message.contains("No such file or directory")
        {
            return true;
        }
        current = err.source();
    }
    false
}
