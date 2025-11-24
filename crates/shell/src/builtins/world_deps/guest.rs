use crate::execution::{build_agent_client_and_request, stream_non_pty_via_agent};
use anyhow::{bail, Result};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use std::error::Error as StdError;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use tokio::runtime::Runtime;

static WORLD_EXEC_FALLBACK: AtomicBool = AtomicBool::new(false);

pub(crate) fn detect_host(commands: &[String]) -> bool {
    for cmd in commands {
        if run_host_command(cmd) {
            return true;
        }
    }
    false
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

pub(crate) fn install_in_guest(script: &str, verbose: bool) -> Result<()> {
    if WORLD_EXEC_FALLBACK.load(Ordering::SeqCst) {
        return run_host_install(script, verbose);
    }

    let command = wrap_for_bash(script, true);
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
        println!(
            "substrate: warn: world backend unavailable for world deps (falling back to host execution): {:#}",
            err
        );
    }
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
    println!("substrate: warn: world backend unavailable; running installer on the host.");
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
        {
            return true;
        }
        current = err.source();
    }
    false
}
