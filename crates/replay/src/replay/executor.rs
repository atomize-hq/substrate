use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::{ExecutionResult, ExecutionState};
use crate::replay::helpers::replay_verbose;
use substrate_common::{FsDiff, WorldRootMode};

#[cfg(target_os = "linux")]
use agent_api_client::AgentClient;
#[cfg(target_os = "linux")]
use agent_api_types::{ExecuteRequest, ExecuteResponse};
#[cfg(target_os = "linux")]
use base64::engine::general_purpose::STANDARD as BASE64;
#[cfg(target_os = "linux")]
use base64::Engine;
#[cfg(target_os = "linux")]
use std::process as stdprocess;
#[cfg(target_os = "linux")]
use std::sync::OnceLock;
#[cfg(target_os = "linux")]
use world::{copydiff, overlayfs};

const ANCHOR_MODE_ENV: &str = "SUBSTRATE_ANCHOR_MODE";
const ANCHOR_PATH_ENV: &str = "SUBSTRATE_ANCHOR_PATH";
const LEGACY_ROOT_MODE_ENV: &str = "SUBSTRATE_WORLD_ROOT_MODE";
const LEGACY_ROOT_PATH_ENV: &str = "SUBSTRATE_WORLD_ROOT_PATH";

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
    emit_scopes_line(replay_verbose(), &out.scopes_used);
    Ok(out)
}

pub async fn execute_with_world_backends(
    state: &ExecutionState,
    timeout_secs: u64,
) -> Result<ExecutionResult> {
    let verbose = replay_verbose();
    let project_dir = project_dir_from_env(&state.env, &state.cwd)?;

    #[cfg(target_os = "linux")]
    {
        if let Some(result) = try_agent_backend(state, &project_dir, timeout_secs, verbose).await? {
            return Ok(result);
        }
    }

    if let Some(result) = try_world_backend(state, &project_dir, verbose).await? {
        return Ok(result);
    }

    #[cfg(target_os = "linux")]
    {
        execute_on_linux(state, &project_dir, verbose)
    }

    #[cfg(not(target_os = "linux"))]
    {
        execute_direct(state, timeout_secs).await
    }
}

async fn try_world_backend(
    state: &ExecutionState,
    project_dir: &Path,
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
            project_dir: project_dir.to_path_buf(),
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
                        emit_scopes_line(verbose, &out.scopes_used);
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
pub fn execute_on_linux(
    state: &ExecutionState,
    project_dir: &Path,
    verbose: bool,
) -> Result<ExecutionResult> {
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
            Ok(true) => match netns.lo_up() {
                Ok(_) => {
                    netns_name = Some(ns);
                    _netns_handle = Some(netns);
                }
                Err(err) => {
                    if verbose {
                        eprintln!(
                            "[replay] warn: failed to bring loopback up in {}: {}",
                            ns, err
                        );
                        eprintln!(
                            "          run 'substrate world cleanup --purge' or rerun with CAP_NET_ADMIN to reset namespaces"
                        );
                    }
                }
            },
            Ok(false) => {
                if verbose {
                    eprintln!(
                        "[replay] warn: netns {} unavailable (ip netns add returned false)",
                        ns
                    );
                    eprintln!(
                        "          run 'substrate world cleanup --purge' or rerun with elevated privileges to remove stale namespaces"
                    );
                }
            }
            Err(err) => {
                if verbose {
                    eprintln!("[replay] warn: netns {} unavailable: {}", ns, err);
                    eprintln!(
                        "          run 'substrate world cleanup --purge' or rerun with elevated privileges to remove stale namespaces"
                    );
                }
            }
        }
    } else if verbose {
        eprintln!("[replay] warn: netns unavailable or insufficient privileges; applying host-wide rules or skipping network scoping");
    }

    enum NetScope {
        Namespace(String),
        Cgroup,
    }
    let scope_choice = if let Some(ref ns) = netns_name {
        Some(NetScope::Namespace(ns.clone()))
    } else if cgroup_active {
        Some(NetScope::Cgroup)
    } else {
        None
    };

    let mut _netfilter_opt: Option<world::netfilter::NetFilter> = None;
    let nft_ok = stdprocess::Command::new("nft")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if nft_ok {
        if let Some(scope) = scope_choice {
            match world::netfilter::NetFilter::new(world_id, Vec::new()) {
                Ok(mut nf) => {
                    match scope {
                        NetScope::Namespace(ns) => {
                            nf.set_namespace(Some(ns));
                        }
                        NetScope::Cgroup => {
                            nf.set_cgroup_path(cgroup_mgr.path());
                            if verbose {
                                eprintln!(
                                    "[replay] warn: using socket cgroup fallback for nft rules ({})",
                                    cgroup_mgr.path().display()
                                );
                                eprintln!(
                                    "          run 'substrate world cleanup --purge' if namespaces cannot be created"
                                );
                            }
                        }
                    }
                    if let Err(e) = nf.install_rules() {
                        if verbose {
                            eprintln!(
                                "[replay] warn: nft setup failed; netfilter scoping/logging disabled: {}",
                                e
                            );
                        }
                    } else {
                        if let Ok(val) = std::fs::read_to_string("/proc/sys/kernel/dmesg_restrict")
                        {
                            if val.trim() == "1" && verbose {
                                eprintln!(
                                    "[replay] warn: kernel.dmesg_restrict=1; LOG lines may not be visible"
                                );
                            }
                        }
                        _netfilter_opt = Some(nf);
                    }
                }
                Err(e) => {
                    if verbose {
                        eprintln!(
                            "[replay] warn: nft initialization failed; netfilter scoping/logging disabled: {}",
                            e
                        );
                    }
                }
            }
        } else if verbose {
            eprintln!("[replay] warn: nft fallback unavailable (no netns/cgroup); network scoping disabled");
            eprintln!("          ensure cgroup v2 is writable or run 'substrate world cleanup --purge' to reset namespaces");
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
        if let Ok(_m) = probe.mount(project_dir) {
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
        ovl.mount(project_dir)?;
        let (output, fs_diff, using_fuse, upper_entries) = run_in_overlay(
            ovl,
            &bash_cmd,
            project_dir,
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
        emit_scopes_line(verbose, &out.scopes_used);
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
        if let Ok(_m) = ovl.mount_fuse_only(project_dir) {
            let (output, fs_diff, using_fuse, upper_entries) = run_in_overlay(
                ovl,
                &bash_cmd,
                project_dir,
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
            emit_scopes_line(verbose, &out.scopes_used);
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
        project_dir,
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
    emit_scopes_line(verbose, &out.scopes_used);
    Ok(out)
}
fn emit_scopes_line(verbose: bool, scopes: &[String]) {
    if !verbose {
        return;
    }
    if scopes.is_empty() {
        eprintln!("[replay] scopes: []");
    } else {
        eprintln!("[replay] scopes: [{}]", scopes.join(", "));
    }
}

fn project_dir_from_env(env: &HashMap<String, String>, cwd: &Path) -> Result<PathBuf> {
    let mode = env
        .get(ANCHOR_MODE_ENV)
        .or_else(|| env.get(LEGACY_ROOT_MODE_ENV))
        .and_then(|value| WorldRootMode::parse(value))
        .unwrap_or(WorldRootMode::Project);

    let root_path = env
        .get(ANCHOR_PATH_ENV)
        .or_else(|| env.get(LEGACY_ROOT_PATH_ENV))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from);

    let base_dir = match mode {
        WorldRootMode::Project => root_path.unwrap_or_else(|| cwd.to_path_buf()),
        WorldRootMode::FollowCwd => cwd.to_path_buf(),
        WorldRootMode::Custom => root_path.ok_or_else(|| {
            anyhow!("world root mode 'custom' requires SUBSTRATE_WORLD_ROOT_PATH")
        })?,
    };

    Ok(base_dir)
}

#[cfg(target_os = "linux")]
async fn try_agent_backend(
    state: &ExecutionState,
    project_dir: &Path,
    timeout_secs: u64,
    verbose: bool,
) -> Result<Option<ExecutionResult>> {
    if !std::path::Path::new("/run/substrate.sock").exists() {
        warn_agent_fallback("agent socket missing".to_string());
        return Ok(None);
    }

    let client = match AgentClient::unix_socket("/run/substrate.sock") {
        Ok(client) => client,
        Err(err) => {
            warn_agent_fallback(format!("connect failed: {}", err));
            return Ok(None);
        }
    };

    match timeout(Duration::from_millis(500), client.capabilities()).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            warn_agent_fallback(format!("capabilities error: {}", err));
            return Ok(None);
        }
        Err(_) => {
            warn_agent_fallback("capabilities probe timed out".to_string());
            return Ok(None);
        }
    }

    let request = ExecuteRequest {
        profile: None,
        cmd: format!("bash -lc '{}'", state.raw_cmd.replace("'", "'\\''")),
        cwd: Some(state.cwd.display().to_string()),
        env: Some(state.env.clone()),
        pty: false,
        agent_id: std::env::var("SUBSTRATE_AGENT_ID").unwrap_or_else(|_| "replay".to_string()),
        budget: None,
    };

    let start = Instant::now();
    match timeout(Duration::from_secs(timeout_secs), client.execute(request)).await {
        Ok(Ok(resp)) => {
            if verbose {
                eprintln!(
                    "[replay] world strategy: agent (project_dir={})",
                    project_dir.display()
                );
            }
            let duration_ms = start.elapsed().as_millis() as u64;
            let out = convert_agent_response(resp, duration_ms);
            emit_scopes_line(verbose, &out.scopes_used);
            Ok(Some(out))
        }
        Ok(Err(err)) => {
            warn_agent_fallback(format!("agent execute failed: {}", err));
            Ok(None)
        }
        Err(_) => {
            warn_agent_fallback("agent execute timed out".to_string());
            Ok(None)
        }
    }
}

#[cfg(target_os = "linux")]
fn convert_agent_response(resp: ExecuteResponse, duration_ms: u64) -> ExecutionResult {
    ExecutionResult {
        exit_code: resp.exit,
        stdout: BASE64
            .decode(resp.stdout_b64.as_bytes())
            .unwrap_or_else(|_| resp.stdout_b64.into_bytes()),
        stderr: BASE64
            .decode(resp.stderr_b64.as_bytes())
            .unwrap_or_else(|_| resp.stderr_b64.into_bytes()),
        fs_diff: resp.fs_diff,
        scopes_used: resp.scopes_used,
        duration_ms,
    }
}

#[cfg(target_os = "linux")]
fn warn_agent_fallback(reason: String) {
    static WARN_ONCE: OnceLock<String> = OnceLock::new();
    if WARN_ONCE.set(reason.clone()).is_ok() {
        eprintln!(
            "[replay] warn: agent replay unavailable, using local backend ({})",
            reason
        );
    }
}
