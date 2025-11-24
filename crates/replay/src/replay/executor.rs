use std::process::Stdio;
use std::time::Instant;

use anyhow::{Context, Result};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::{ExecutionResult, ExecutionState};
use crate::replay::helpers::replay_verbose;

#[cfg(target_os = "linux")]
use std::collections::HashMap;
#[cfg(target_os = "linux")]
use std::path::Path;
#[cfg(target_os = "linux")]
use std::path::PathBuf;
#[cfg(target_os = "linux")]
use std::process as stdprocess;
#[cfg(target_os = "linux")]
use substrate_common::FsDiff;
#[cfg(target_os = "linux")]
use world::{copydiff, overlayfs};

/// Execute a command directly (without world isolation)
pub async fn execute_direct(state: &ExecutionState, timeout_secs: u64) -> Result<ExecutionResult> {
    let mut cmd = Command::new("/bin/bash");
    cmd.arg("-lc").arg(&state.raw_cmd);
    cmd.current_dir(&state.cwd);
    cmd.envs(&state.env);
    if std::env::var("SHELL").is_err() {
        cmd.env("SHELL", "/bin/bash");
    }
    if std::env::var("LANG").is_err() {
        cmd.env("LANG", "C.UTF-8");
    }
    if std::env::var("LC_ALL").is_err() {
        cmd.env("LC_ALL", "C.UTF-8");
    }
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    cmd.env("SHIM_SESSION_ID", &state.session_id);
    cmd.env("SHIM_PARENT_SPAN", &state.span_id);
    cmd.env("SUBSTRATE_REPLAY", "1");

    if state.stdin.is_some() {
        cmd.stdin(Stdio::piped());
    }

    let start = Instant::now();
    let result = match timeout(Duration::from_secs(timeout_secs), async {
        let mut child = cmd.spawn().context("Failed to spawn command")?;

        if let Some(stdin_data) = &state.stdin {
            if let Some(mut stdin) = child.stdin.take() {
                use tokio::io::AsyncWriteExt;
                stdin
                    .write_all(stdin_data)
                    .await
                    .context("Failed to write stdin")?;
            }
        }

        Ok::<_, anyhow::Error>(child.wait_with_output().await?)
    })
    .await
    {
        Ok(Ok(output)) => output,
        Ok(Err(e)) => return Err(e),
        Err(_) => anyhow::bail!("Command execution timed out"),
    };

    let duration_ms = start.elapsed().as_millis() as u64;

    let out = ExecutionResult {
        exit_code: result.status.code().unwrap_or(-1),
        stdout: result.stdout,
        stderr: result.stderr,
        fs_diff: None,
        scopes_used: Vec::new(),
        duration_ms,
    };
    if replay_verbose() && !out.scopes_used.is_empty() {
        eprintln!("[replay] scopes: {}", out.scopes_used.join(","));
    }
    Ok(out)
}

pub async fn execute_with_world_backends(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    let verbose = replay_verbose();
    if let Some(result) = try_world_backend(state, verbose).await? {
        return Ok(result);
    }

    #[cfg(target_os = "linux")]
    {
        return execute_on_linux(state, verbose);
    }

    #[cfg(not(target_os = "linux"))]
    {
        return execute_direct(state, timeout_secs).await;
    }
}

async fn try_world_backend(
    state: &ExecutionState,
    verbose: bool,
) -> Result<Option<ExecutionResult>> {
    if let Ok(backend) = world_backend_factory::factory() {
        use world_api::{ExecRequest, ResourceLimits, WorldSpec};
        let start = Instant::now();
        let spec = WorldSpec {
            reuse_session: true,
            isolate_network: true,
            limits: ResourceLimits::default(),
            enable_preload: false,
            allowed_domains: substrate_broker::allowed_domains(),
            project_dir: state.cwd.clone(),
            always_isolate: true,
        };
        match backend.ensure_session(&spec) {
            Ok(handle) => {
                let req = ExecRequest {
                    cmd: format!("bash -lc '{}'", state.raw_cmd.replace("'", "'\\''")),
                    cwd: state.cwd.clone(),
                    env: state.env.clone(),
                    pty: false,
                    span_id: Some(state.span_id.clone()),
                };
                match backend.exec(&handle, req) {
                    Ok(res) => {
                        if verbose {
                            eprintln!("[replay] world strategy: overlay");
                        }
                        let duration_ms = start.elapsed().as_millis() as u64;
                        let out = ExecutionResult {
                            exit_code: res.exit,
                            stdout: res.stdout,
                            stderr: res.stderr,
                            fs_diff: res.fs_diff,
                            scopes_used: res.scopes_used,
                            duration_ms,
                        };
                        if verbose && !out.scopes_used.is_empty() {
                            eprintln!("[replay] scopes: {}", out.scopes_used.join(","));
                        }
                        return Ok(Some(out));
                    }
                    Err(e) => {
                        if verbose {
                            eprintln!("[replay] warn: world exec failed: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                if verbose {
                    eprintln!("[replay] warn: world session creation failed: {}", e);
                }
            }
        }
    } else if verbose {
        eprintln!("[replay] warn: no world backend available on this platform");
    }

    Ok(None)
}

#[cfg(target_os = "linux")]
pub fn execute_on_linux(state: &ExecutionState, verbose: bool) -> Result<ExecutionResult> {
    let world_id = &state.span_id;
    let bash_cmd = format!("bash -lc '{}'", state.raw_cmd.replace("'", "'\\''"));
    let start = std::time::Instant::now();

    let mut cgroup_mgr = world::cgroups::CgroupManager::new(world_id);
    let mut cgroup_active = false;
    match cgroup_mgr.setup() {
        Ok(true) => {
            let _ = cgroup_mgr.attach_current();
            cgroup_active = true;
        }
        Ok(false) | Err(_) => {
            if verbose {
                eprintln!("[replay] warn: cgroup v2 unavailable or insufficient privileges; skipping cgroup attach");
            }
        }
    }

    let mut netns_name: Option<String> = None;
    let mut _netns_handle: Option<world::netns::NetNs> = None;
    if world::netns::NetNs::ip_available() {
        let ns = format!("substrate-{}", world_id);
        let mut netns = world::netns::NetNs::new(&ns);
        match netns.add() {
            Ok(true) => {
                if let Err(_e) = netns.lo_up() {
                    if verbose {
                        eprintln!("[replay] warn: netns unavailable or insufficient privileges; applying host-wide rules or skipping network scoping");
                    }
                } else {
                    netns_name = Some(ns);
                    _netns_handle = Some(netns);
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

    let mut _netfilter_opt: Option<world::netfilter::NetFilter> = None;
    let nft_ok = stdprocess::Command::new("nft")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if nft_ok {
        match world::netfilter::NetFilter::new(world_id, Vec::new()) {
            Ok(mut nf) => {
                if let Some(ref ns) = netns_name {
                    nf.set_namespace(Some(ns.clone()));
                }
                if let Err(e) = nf.install_rules() {
                    if verbose {
                        eprintln!("[replay] warn: nft setup failed; netfilter scoping/logging disabled: {}", e);
                    }
                } else {
                    if let Ok(val) = std::fs::read_to_string("/proc/sys/kernel/dmesg_restrict") {
                        if val.trim() == "1" && verbose {
                            eprintln!("[replay] warn: kernel.dmesg_restrict=1; LOG lines may not be visible");
                        }
                    }
                    _netfilter_opt = Some(nf);
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

    fn run_in_overlay(
        mut ovl: overlayfs::OverlayFs,
        cmd: &str,
        project_dir: &Path,
        cwd: &Path,
        env: &HashMap<String, String>,
        cgroup_mgr: Option<&world::cgroups::CgroupManager>,
        netns_name: Option<&str>,
    ) -> Result<(std::process::Output, FsDiff, bool, usize)> {
        let merged_dir = ovl.merged_dir_path().to_path_buf();
        let mut rel = if cwd.starts_with(project_dir) {
            cwd.strip_prefix(project_dir)
                .unwrap_or_else(|_| Path::new("."))
                .to_path_buf()
        } else {
            PathBuf::from(".")
        };
        if rel.as_os_str().is_empty() {
            rel = PathBuf::from(".");
        }
        let target_dir = merged_dir.join(&rel);
        let mut command = stdprocess::Command::new(if netns_name.is_some() { "ip" } else { "sh" });
        if let Some(ns) = netns_name {
            command.args(["netns", "exec", ns, "sh", "-lc", cmd]);
        } else {
            command.args(["-lc", cmd]);
        }
        let child = command
            .current_dir(&target_dir)
            .envs(env)
            .spawn()
            .context("Failed to spawn command in overlay")?;

        if let Some(mgr) = cgroup_mgr {
            let _ = mgr.attach_pid(child.id());
        }

        let output = child
            .wait_with_output()
            .context("Failed to wait for command in overlay")?;

        let upper = ovl.upper_dir_path().to_path_buf();
        fn count_entries(p: &Path) -> usize {
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

        let diff = ovl.compute_diff()?;
        ovl.cleanup()?;
        Ok((output, diff, ovl.is_using_fuse(), upper_entries))
    }

    let _tried_overlay_kernel;
    let mut overlay_kernel_ok = false;
    if std::fs::read_to_string("/proc/filesystems")
        .map(|s| s.contains("overlay"))
        .unwrap_or(false)
    {
        _tried_overlay_kernel = true;
        let mut probe = overlayfs::OverlayFs::new(&format!("{}-probe", world_id))?;
        if let Ok(_m) = probe.mount(&state.cwd) {
            if !probe.is_using_fuse() {
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
        let mut ovl = overlayfs::OverlayFs::new(world_id)?;
        ovl.mount(&state.cwd)?;
        let (output, fs_diff, using_fuse, upper_entries) = run_in_overlay(
            ovl,
            &bash_cmd,
            &state.cwd,
            &state.cwd,
            &state.env,
            if cgroup_active {
                Some(&cgroup_mgr)
            } else {
                None
            },
            netns_name.as_deref(),
        )?;
        if verbose {
            eprintln!(
                "[replay] world strategy: {}",
                if using_fuse { "fuse" } else { "overlay" }
            );
            if fs_diff.is_empty() {
                eprintln!("[replay] upper entries: {}", upper_entries);
            }
        }
        let duration_ms = start.elapsed().as_millis() as u64;
        let out = ExecutionResult {
            exit_code: output.status.code().unwrap_or(-1),
            stdout: output.stdout,
            stderr: output.stderr,
            fs_diff: Some(fs_diff),
            scopes_used: Vec::new(),
            duration_ms,
        };
        if verbose && !out.scopes_used.is_empty() {
            eprintln!("[replay] scopes: {}", out.scopes_used.join(","));
        }
        return Ok(out);
    }

    let fuse_dev = std::path::Path::new("/dev/fuse").exists();
    let fuse_bin_ok = stdprocess::Command::new("sh")
        .arg("-lc")
        .arg("command -v fuse-overlayfs >/dev/null 2>&1")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if fuse_dev && fuse_bin_ok {
        let mut ovl = overlayfs::OverlayFs::new(world_id)?;
        if let Ok(_m) = ovl.mount_fuse_only(&state.cwd) {
            let (output, fs_diff, using_fuse, upper_entries) = run_in_overlay(
                ovl,
                &bash_cmd,
                &state.cwd,
                &state.cwd,
                &state.env,
                if cgroup_active {
                    Some(&cgroup_mgr)
                } else {
                    None
                },
                netns_name.as_deref(),
            )?;
            if verbose {
                eprintln!(
                    "[replay] world strategy: {}",
                    if using_fuse { "fuse" } else { "overlay" }
                );
                if fs_diff.is_empty() {
                    eprintln!("[replay] upper entries: {}", upper_entries);
                }
            }
            let duration_ms = start.elapsed().as_millis() as u64;
            let out = ExecutionResult {
                exit_code: output.status.code().unwrap_or(-1),
                stdout: output.stdout,
                stderr: output.stderr,
                fs_diff: Some(fs_diff),
                scopes_used: Vec::new(),
                duration_ms,
            };
            if verbose && !out.scopes_used.is_empty() {
                eprintln!("[replay] scopes: {}", out.scopes_used.join(","));
            }
            return Ok(out);
        }
    }

    if verbose {
        eprintln!("[replay] warn: overlay and fuse-overlayfs unavailable; using copy-diff (userspace snapshot)");
        eprintln!("[replay] world strategy: copy-diff");
    }
    let (output, fs_diff, child_pid_opt) = copydiff::execute_with_copydiff(
        world_id,
        &bash_cmd,
        &state.cwd,
        &state.cwd,
        &state.env,
        netns_name.as_deref(),
    )?;
    if cgroup_active {
        if let Some(pid) = child_pid_opt {
            let _ = cgroup_mgr.attach_pid(pid);
        }
    }
    let duration_ms = start.elapsed().as_millis() as u64;
    let out = ExecutionResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: output.stdout,
        stderr: output.stderr,
        fs_diff: Some(fs_diff),
        scopes_used: Vec::new(),
        duration_ms,
    };
    if verbose && !out.scopes_used.is_empty() {
        eprintln!("[replay] scopes: {}", out.scopes_used.join(","));
    }
    Ok(out)
}
