//! Replay engine for re-executing traced commands

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use substrate_common::FsDiff;
use tokio::process::Command;
use tokio::time::{timeout, Duration};
#[cfg(target_os = "linux")]
use std::process as stdprocess;

/// State required to execute a command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionState {
    /// The original raw command string as captured in the span
    pub raw_cmd: String,
    pub command: String,
    pub args: Vec<String>,
    pub cwd: PathBuf,
    pub env: HashMap<String, String>,
    pub stdin: Option<Vec<u8>>,
    pub session_id: String,
    pub span_id: String,
}

/// Result of executing a command
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub fs_diff: Option<FsDiff>,
    pub scopes_used: Vec<String>,
    pub duration_ms: u64,
}

/// Execute a command in an isolated world
pub async fn execute_in_world(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    // Use world-api backend on Linux when enabled
    if world_isolation_available() {
        #[cfg(target_os = "linux")]
        {
            let verbose = std::env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1";
            let world_id = &state.span_id;
            let bash_cmd = format!("bash -lc '{}'", state.raw_cmd.replace("'", "'\\''"));
            let start = std::time::Instant::now();

            // Best-effort per-world cgroup + netns + nftables setup
            let mut cgroup_mgr = world::cgroups::CgroupManager::new(world_id);
            let mut cgroup_active = false;
            match cgroup_mgr.setup() {
                Ok(true) => {
                    // Attach current process immediately; child attach attempted later when possible
                    let _ = cgroup_mgr.attach_current();
                    cgroup_active = true;
                }
                Ok(false) | Err(_) => {
                    if verbose {
                        eprintln!("[replay] warn: cgroup v2 unavailable or insufficient privileges; skipping cgroup attach");
                    }
                }
            }

            // Best-effort network namespace
            let mut netns_name: Option<String> = None;
            let mut netns_handle: Option<world::netns::NetNs> = None; // ensure drop after netfilter
            if world::netns::NetNs::ip_available() {
                let ns = format!("substrate-{}", world_id);
                let mut netns = world::netns::NetNs::new(&ns);
                match netns.add() {
                    Ok(true) => {
                        if let Err(_e) = netns.lo_up() {
                            if verbose { eprintln!("[replay] warn: netns unavailable or insufficient privileges; applying host-wide rules or skipping network scoping"); }
                        } else {
                            // Ensure netns lives until after netfilter drop; declared before netfilter
                            netns_name = Some(ns);
                            netns_handle = Some(netns);
                        }
                    }
                    _ => {
                        if verbose {
                            eprintln!("[replay] warn: netns unavailable or insufficient privileges; applying host-wide rules or skipping network scoping");
                        }
                    }
                }
            } else if verbose {
                eprintln!("[replay] warn: netns unavailable or insufficient privileges; applying host-wide rules or skipping network scoping");
            }

            // Best-effort nftables setup with conservative LOG+drop default (within netns when available)
            let mut netfilter_opt: Option<world::netfilter::NetFilter> = None;
            let nft_ok = stdprocess::Command::new("nft").arg("--version").status().map(|s| s.success()).unwrap_or(false);
            if nft_ok {
                match world::netfilter::NetFilter::new(world_id, Vec::new()) {
                    Ok(mut nf) => {
                        if let Some(ref ns) = netns_name { nf.set_namespace(Some(ns.clone())); }
                        if let Err(e) = nf.install_rules() {
                            if verbose {
                                eprintln!("[replay] warn: nft setup failed; netfilter scoping/logging disabled: {}", e);
                            }
                        } else {
                            // Warn when LOG visibility may be restricted
                            if let Ok(val) = std::fs::read_to_string("/proc/sys/kernel/dmesg_restrict") {
                                if val.trim() == "1" && verbose {
                                    eprintln!("[replay] warn: kernel.dmesg_restrict=1; LOG lines may not be visible");
                                }
                            }
                            netfilter_opt = Some(nf);
                        }
                    }
                    Err(e) => {
                        if verbose {
                            eprintln!("[replay] warn: nft initialization failed; netfilter scoping/logging disabled: {}", e);
                        }
                    }
                }
            } else if verbose {
                eprintln!("[replay] warn: nft not available; netfilter scoping/logging disabled");
            }

            // Strategy probe and selection
            // 1) Try kernel overlay: mount + tiny write probe
            let mut chosen_strategy = String::new();

            // Helper to run inside overlay (kernel or fuse decided by how we mounted)
            fn run_in_overlay(
                mut ovl: world::overlayfs::OverlayFs,
                cmd: &str,
                project_dir: &std::path::Path,
                cwd: &std::path::Path,
                env: &std::collections::HashMap<String, String>,
                cgroup_mgr: Option<&world::cgroups::CgroupManager>,
                netns_name: Option<&str>,
            ) -> Result<(std::process::Output, FsDiff, bool, usize)> {
                let merged_dir = ovl.merged_dir_path().to_path_buf();
                // Execute command in merged by cd into the equivalent path under merged
                let mut rel = if cwd.starts_with(project_dir) {
                    cwd.strip_prefix(project_dir).unwrap_or_else(|_| std::path::Path::new(".")).to_path_buf()
                } else {
                    std::path::PathBuf::from(".")
                };
                if rel.as_os_str().is_empty() { rel = std::path::PathBuf::from("."); }
                let target_dir = merged_dir.join(&rel);
                let mut command = std::process::Command::new(if netns_name.is_some() { "ip" } else { "sh" });
                if let Some(ns) = netns_name {
                    command.args(["netns","exec", ns, "sh", "-lc", cmd]);
                } else {
                    command.args(["-lc", cmd]);
                }
                let mut child = command
                    .current_dir(&target_dir)
                    .envs(env)
                    .spawn()
                    .context("Failed to spawn command in overlay")?;

                // Best-effort attach child PID to cgroup when available
                if let Some(mgr) = cgroup_mgr { let _ = mgr.attach_pid(child.id()); }

                let output = child
                    .wait_with_output()
                    .context("Failed to wait for command in overlay")?;

                // Count upper entries for diagnostics before cleaning
                let upper = ovl.upper_dir_path().to_path_buf();
                fn count_entries(p: &std::path::Path) -> usize {
                    let mut cnt = 0usize;
                    if let Ok(rd) = std::fs::read_dir(p) {
                        for ent in rd.flatten() {
                            cnt += 1;
                            let path = ent.path();
                            if path.is_dir() {
                                cnt += count_entries(&path);
                            }
                        }
                    }
                    cnt
                }
                let upper_entries = count_entries(&upper);

                // Compute diff before cleanup
                let diff = ovl.compute_diff()?;
                ovl.cleanup()?;
                Ok((output, diff, ovl.is_using_fuse(), upper_entries))
            }

            // Probe kernel overlay
            let mut tried_overlay_kernel = false;
            let mut overlay_kernel_ok = false;
            if std::fs::read_to_string("/proc/filesystems").map(|s| s.contains("overlay")).unwrap_or(false) {
                tried_overlay_kernel = true;
                let mut probe = world::overlayfs::OverlayFs::new(&format!("{}-probe", world_id))?;
                if let Ok(_m) = probe.mount(&state.cwd) {
                    if !probe.is_using_fuse() {
                        // tiny write probe
                        let merged = probe.merged_dir_path().to_path_buf();
                        let _ = std::fs::create_dir_all(merged.join(".substrate-probe"));
                        let _ = std::fs::write(merged.join(".substrate-probe/probe.txt"), b"x");
                        let diff = probe.compute_diff().unwrap_or_default();
                        overlay_kernel_ok = !diff.is_empty();
                    }
                    let _ = probe.cleanup();
                }
            }

            if overlay_kernel_ok {
                // Use kernel overlay
                let mut ovl = world::overlayfs::OverlayFs::new(world_id)?;
                ovl.mount(&state.cwd)?;
                let (output, fs_diff, using_fuse, upper_entries) = run_in_overlay(
                    ovl,
                    &bash_cmd,
                    &state.cwd,
                    &state.cwd,
                    &state.env,
                    if cgroup_active { Some(&cgroup_mgr) } else { None },
                    netns_name.as_deref(),
                )?;
                chosen_strategy = if using_fuse { "fuse" } else { "overlay" }.to_string();
                if verbose {
                    eprintln!("[replay] world strategy: {}", chosen_strategy);
                    if fs_diff.is_empty() {
                        eprintln!("[replay] upper entries: {}", upper_entries);
                    }
                }
                let duration_ms = start.elapsed().as_millis() as u64;
                // netfilter rules teardown via Drop at end of scope
                // cgroup teardown best-effort via Drop
                return Ok(ExecutionResult {
                    exit_code: output.status.code().unwrap_or(-1),
                    stdout: output.stdout,
                    stderr: output.stderr,
                    fs_diff: Some(fs_diff),
                    scopes_used: Vec::new(),
                    duration_ms,
                });
            }

            // 2) Try fuse-overlayfs when /dev/fuse exists and binary is present
            let fuse_dev = std::path::Path::new("/dev/fuse").exists();
            let fuse_bin_ok = std::process::Command::new("sh")
                .arg("-lc")
                .arg("command -v fuse-overlayfs >/dev/null 2>&1")
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if fuse_dev && fuse_bin_ok {
                let mut ovl = world::overlayfs::OverlayFs::new(world_id)?;
                if let Ok(_m) = ovl.mount_fuse_only(&state.cwd) {
                    // quick readiness probe via /proc/self/mounts already done in mount_fuse_only
                    let (output, fs_diff, using_fuse, upper_entries) = run_in_overlay(
                        ovl,
                        &bash_cmd,
                        &state.cwd,
                        &state.cwd,
                        &state.env,
                        if cgroup_active { Some(&cgroup_mgr) } else { None },
                        netns_name.as_deref(),
                    )?;
                    chosen_strategy = if using_fuse { "fuse" } else { "overlay" }.to_string();
                    if verbose {
                        eprintln!("[replay] world strategy: {}", chosen_strategy);
                        if fs_diff.is_empty() {
                            eprintln!("[replay] upper entries: {}", upper_entries);
                        }
                    }
                    let duration_ms = start.elapsed().as_millis() as u64;
                    return Ok(ExecutionResult {
                        exit_code: output.status.code().unwrap_or(-1),
                        stdout: output.stdout,
                        stderr: output.stderr,
                        fs_diff: Some(fs_diff),
                        scopes_used: Vec::new(),
                        duration_ms,
                    });
                }
            }

            // 3) Userspace copy-diff fallback (no privileges)
            if verbose {
                eprintln!("[replay] warn: overlay and fuse-overlayfs unavailable; using copy-diff (userspace snapshot)");
                eprintln!("[replay] world strategy: copy-diff");
            }
            let (output, fs_diff, child_pid_opt) = world::copydiff::execute_with_copydiff(
                world_id,
                &bash_cmd,
                &state.cwd,
                &state.cwd,
                &state.env,
                netns_name.as_deref(),
            )?;
            if cgroup_active {
                if let Some(pid) = child_pid_opt { let _ = cgroup_mgr.attach_pid(pid); }
            }
            // Note: cannot directly attach child PID here because copydiff currently uses output().
            // Ensure the cgroup remains non-empty via current PID attachment performed earlier.
            let duration_ms = start.elapsed().as_millis() as u64;
            return Ok(ExecutionResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: output.stdout,
                stderr: output.stderr,
                fs_diff: Some(fs_diff),
                scopes_used: Vec::new(),
                duration_ms,
            });
        }
        #[cfg(not(target_os = "linux"))]
        {
            // Fallback to direct on non-Linux
            return execute_direct(state, timeout_secs).await;
        }
    }
    // Fallback direct execution when isolation disabled
    if std::env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1" {
        eprintln!("[replay] world strategy: direct");
    }
    execute_direct(state, timeout_secs).await
}

/// Check if world isolation backend is available
fn world_isolation_available() -> bool {
    // Check if world isolation is enabled and we're on Linux
    #[cfg(target_os = "linux")]
    {
        std::env::var("SUBSTRATE_REPLAY_USE_WORLD").unwrap_or_default() == "1"
    }
    
    #[cfg(not(target_os = "linux"))]
    {
        false // World backend only available on Linux
    }
}

/// Execute with full world isolation
// legacy world-specific execution removed in favor of world-api path

/// Execute a command directly (without world isolation)
pub async fn execute_direct(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    // Prefer running via a shell to preserve quoting, pipes, redirects, etc.
    let mut cmd = Command::new("/bin/bash");
    cmd.arg("-lc").arg(&state.raw_cmd);
    cmd.current_dir(&state.cwd);
    // Minimal environment reinjection
    cmd.envs(&state.env);
    // Ensure a reasonable default shell environment
    if std::env::var("SHELL").is_err() { cmd.env("SHELL", "/bin/bash"); }
    if std::env::var("LANG").is_err() { cmd.env("LANG", "C.UTF-8"); }
    if std::env::var("LC_ALL").is_err() { cmd.env("LC_ALL", "C.UTF-8"); }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    
    // Add substrate environment variables for correlation
    cmd.env("SHIM_SESSION_ID", &state.session_id);
    cmd.env("SHIM_PARENT_SPAN", &state.span_id);
    cmd.env("SUBSTRATE_REPLAY", "1");
    
    if state.stdin.is_some() {
        cmd.stdin(Stdio::piped());
    }
    
    // Execute with timeout
    let start = std::time::Instant::now();
    let result = match timeout(Duration::from_secs(timeout_secs), async {
        let mut child = cmd.spawn().context("Failed to spawn command")?;
        
        // Provide stdin if needed
        if let Some(stdin_data) = &state.stdin {
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin.write_all(stdin_data).await
                    .context("Failed to write stdin")?;
            }
        }
        
        Ok::<_, anyhow::Error>(child.wait_with_output().await?)
    })
    .await {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(e),
        Err(_) => anyhow::bail!("Command execution timed out"),
    };
    
    let duration_ms = start.elapsed().as_millis() as u64;
    
    Ok(ExecutionResult {
        exit_code: result.status.code().unwrap_or(-1),
        stdout: result.stdout,
        stderr: result.stderr,
        fs_diff: None, // No isolation means no diff tracking
        scopes_used: Vec::new(),
        duration_ms,
    })
}

/// Parse command string into command and args
pub fn parse_command(cmd_str: &str) -> (String, Vec<String>) {
    // Simple parsing - in production would use shell_words or similar
    let parts: Vec<String> = cmd_str.split_whitespace().map(String::from).collect();
    
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    
    let command = parts[0].clone();
    let args = parts[1..].to_vec();
    
    (command, args)
}

/// Replay a command sequence (multiple related commands)
pub async fn replay_sequence(
    states: Vec<ExecutionState>,
    timeout_secs: u64,
    use_world: bool,
) -> Result<Vec<ExecutionResult>> {
    let mut results = Vec::new();
    
    for state in states {
        let result = if use_world {
            execute_in_world(&state, timeout_secs).await?
        } else {
            execute_direct(&state, timeout_secs).await?
        };
        
        // Check if we should continue after failure
        if result.exit_code != 0 {
            tracing::warn!(
                "Command failed with exit code {}: {}",
                result.exit_code, state.command
            );
        }
        
        results.push(result);
    }
    
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    #[test]
    fn test_parse_command() {
        let (cmd, args) = parse_command("echo hello world");
        assert_eq!(cmd, "echo");
        assert_eq!(args, vec!["hello", "world"]);
        
        let (cmd, args) = parse_command("ls");
        assert_eq!(cmd, "ls");
        assert!(args.is_empty());
    }
    
    #[tokio::test]
    async fn test_execute_direct_simple() {
        let state = ExecutionState {
            raw_cmd: "echo test".to_string(),
            command: "echo".to_string(),
            args: vec!["test".to_string()],
            cwd: std::env::current_dir().unwrap(),
            env: HashMap::new(),
            stdin: None,
            session_id: "test-session".to_string(),
            span_id: "test-span".to_string(),
        };
        
        let result = execute_direct(&state, 10).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), "test");
    }

    #[tokio::test]
    async fn test_execute_direct_with_redirection() {
        let dir = tempdir().unwrap();
        let cwd = dir.path().to_path_buf();
        let state = ExecutionState {
            raw_cmd: "echo hello > out.txt".to_string(),
            command: "echo".to_string(),
            args: vec!["hello".to_string(), ">".to_string(), "out.txt".to_string()],
            cwd: cwd.clone(),
            env: HashMap::new(),
            stdin: None,
            session_id: "s".to_string(),
            span_id: "sp".to_string(),
        };
        let res = execute_direct(&state, 10).await.unwrap();
        assert_eq!(res.exit_code, 0);
        let content = std::fs::read_to_string(cwd.join("out.txt")).unwrap();
        assert_eq!(content.trim(), "hello");
    }
}
