use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Instant;

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use serde_json::json;
use tokio::process::Command;
use tokio::time::{timeout, Duration};

use super::{ExecutionResult, ExecutionState};
use crate::replay::helpers::replay_verbose;
use substrate_common::{log_schema, FsDiff, WorldRootMode};
use substrate_trace::append_to_trace;

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

#[cfg(target_os = "linux")]
#[derive(Clone, Debug)]
struct AgentFallback {
    reason: String,
    socket_path: PathBuf,
}

#[cfg(target_os = "linux")]
struct AgentBackendOutcome {
    result: Option<ExecutionResult>,
    fallback: Option<AgentFallback>,
    socket_path: PathBuf,
}

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
    let mut agent_fallback_reason: Option<String> = None;
    let agent_socket: Option<PathBuf>;

    #[cfg(target_os = "linux")]
    {
        let outcome = try_agent_backend(state, &project_dir, timeout_secs, verbose).await?;
        agent_socket = match outcome {
            AgentBackendOutcome {
                result: Some(result),
                fallback: _,
                socket_path,
            } => {
                record_replay_strategy(
                    state,
                    "agent",
                    Some(&socket_path),
                    None,
                    json!({ "project_dir": project_dir.display().to_string() }),
                );
                return Ok(result);
            }
            AgentBackendOutcome {
                result: None,
                fallback,
                socket_path,
            } => {
                if let Some(fallback) = fallback {
                    agent_fallback_reason = Some(fallback.reason.clone());
                    Some(fallback.socket_path)
                } else {
                    Some(socket_path)
                }
            }
        };
    }
    #[cfg(not(target_os = "linux"))]
    {
        agent_socket = None;
    }

    if let Some(result) = try_world_backend(state, &project_dir, verbose).await? {
        record_replay_strategy(
            state,
            "world-backend",
            agent_socket.as_deref(),
            agent_fallback_reason.as_deref(),
            json!({
                "project_dir": project_dir.display().to_string(),
                "backend": "world-api"
            }),
        );
        return Ok(result);
    }

    #[cfg(target_os = "linux")]
    {
        execute_on_linux(
            state,
            &project_dir,
            verbose,
            agent_fallback_reason,
            agent_socket,
        )
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
            fs_mode: substrate_broker::world_fs_mode(),
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
    agent_fallback_reason: Option<String>,
    agent_socket: Option<PathBuf>,
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
        record_replay_strategy(
            state,
            if using_fuse { "fuse" } else { "overlay" },
            agent_socket.as_deref(),
            agent_fallback_reason.as_deref(),
            json!({
                "project_dir": project_dir.display().to_string(),
                "cgroup_attached": cgroup_active,
                "netns": netns_name.clone(),
            }),
        );
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
            record_replay_strategy(
                state,
                if using_fuse { "fuse" } else { "overlay" },
                agent_socket.as_deref(),
                agent_fallback_reason.as_deref(),
                json!({
                    "project_dir": project_dir.display().to_string(),
                    "cgroup_attached": cgroup_active,
                    "netns": netns_name.clone(),
                }),
            );
            return Ok(out);
        }
    }

    if verbose {
        eprintln!("[replay] warn: overlay and fuse-overlayfs unavailable; using copy-diff (userspace snapshot)");
        eprintln!("[replay] world strategy: copy-diff");
    }
    let copydiff_outcome = copydiff::execute_with_copydiff(
        world_id,
        &bash_cmd,
        project_dir,
        &state.cwd,
        &state.env,
        netns_name.as_deref(),
    )?;
    let copydiff::CopyDiffOutcome {
        output,
        fs_diff,
        child_pid,
        root,
        root_source,
    } = copydiff_outcome;
    if cgroup_active {
        if let Some(pid) = child_pid {
            let _ = cgroup_mgr.attach_pid(pid);
        }
    }
    if verbose {
        eprintln!(
            "[replay] copy-diff root: {} ({})",
            root.display(),
            root_source.as_str()
        );
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
    record_replay_strategy(
        state,
        "copy-diff",
        agent_socket.as_deref(),
        agent_fallback_reason.as_deref(),
        json!({
            "project_dir": project_dir.display().to_string(),
            "copydiff_root": root.display().to_string(),
            "copydiff_root_source": root_source.as_str(),
            "cgroup_attached": cgroup_active,
            "netns": netns_name,
        }),
    );
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

pub fn record_replay_strategy(
    state: &ExecutionState,
    strategy: &str,
    agent_socket: Option<&Path>,
    fallback_reason: Option<&str>,
    extra: serde_json::Value,
) {
    let mut entry = json!({
        log_schema::TIMESTAMP: Utc::now().to_rfc3339(),
        log_schema::EVENT_TYPE: "replay_strategy",
        log_schema::SESSION_ID: state.session_id,
        log_schema::COMMAND_ID: state.span_id,
        log_schema::COMPONENT: "replay",
        "strategy": strategy,
    });

    entry["recorded_origin"] = json!(state.recorded_origin.as_str());
    entry["target_origin"] = json!(state.target_origin.as_str());

    if let Some(source) = &state.recorded_origin_source {
        entry["recorded_origin_source"] = json!(source);
    }

    if let Some(reason) = &state.origin_reason {
        entry["origin_reason"] = json!(reason);
    }
    if let Some(code) = &state.origin_reason_code {
        entry["origin_reason_code"] = json!(code);
    }

    if let Some(transport) = state.recorded_transport.as_ref() {
        let mut transport_obj = json!({
            "mode": transport.mode,
        });
        if let Some(endpoint) = &transport.endpoint {
            transport_obj["endpoint"] = json!(endpoint);
        }
        if let Some(activated) = transport.socket_activation {
            transport_obj["socket_activation"] = json!(activated);
        }
        entry["recorded_transport"] = transport_obj;
    }

    if let Some(reason) = fallback_reason {
        entry["fallback_reason"] = json!(reason);
    }
    if let Some(socket) = agent_socket {
        entry["agent_socket"] = json!(socket.display().to_string());
    }
    if let Some(obj) = entry.as_object_mut() {
        if let Some(extra_obj) = extra.as_object() {
            for (k, v) in extra_obj {
                obj.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
    }

    let _ = append_to_trace(&entry);
}

#[cfg(target_os = "linux")]
async fn try_agent_backend(
    state: &ExecutionState,
    project_dir: &Path,
    timeout_secs: u64,
    verbose: bool,
) -> Result<AgentBackendOutcome> {
    let socket_path = agent_socket_path(state);
    if !socket_path.exists() {
        let fallback = warn_agent_fallback(
            format!("agent socket missing ({})", socket_path.display()),
            &socket_path,
        );
        return Ok(AgentBackendOutcome {
            result: None,
            fallback: Some(fallback),
            socket_path,
        });
    }

    let client = match AgentClient::unix_socket(&socket_path) {
        Ok(client) => client,
        Err(err) => {
            let fallback = warn_agent_fallback(format!("connect failed: {}", err), &socket_path);
            return Ok(AgentBackendOutcome {
                result: None,
                fallback: Some(fallback),
                socket_path,
            });
        }
    };

    match timeout(Duration::from_millis(500), client.capabilities()).await {
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            let fallback =
                warn_agent_fallback(format!("capabilities error: {}", err), &socket_path);
            return Ok(AgentBackendOutcome {
                result: None,
                fallback: Some(fallback),
                socket_path,
            });
        }
        Err(_) => {
            let fallback =
                warn_agent_fallback("capabilities probe timed out".to_string(), &socket_path);
            return Ok(AgentBackendOutcome {
                result: None,
                fallback: Some(fallback),
                socket_path,
            });
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
        world_fs_mode: Some(substrate_broker::world_fs_mode()),
    };

    let start = Instant::now();
    match timeout(Duration::from_secs(timeout_secs), client.execute(request)).await {
        Ok(Ok(resp)) => {
            if verbose {
                eprintln!(
                    "[replay] world strategy: agent (socket={}, project_dir={})",
                    socket_path.display(),
                    project_dir.display()
                );
            }
            let duration_ms = start.elapsed().as_millis() as u64;
            let out = convert_agent_response(resp, duration_ms);
            emit_scopes_line(verbose, &out.scopes_used);
            Ok(AgentBackendOutcome {
                result: Some(out),
                fallback: None,
                socket_path,
            })
        }
        Ok(Err(err)) => {
            let fallback =
                warn_agent_fallback(format!("agent execute failed: {}", err), &socket_path);
            Ok(AgentBackendOutcome {
                result: None,
                fallback: Some(fallback),
                socket_path,
            })
        }
        Err(_) => {
            let fallback = warn_agent_fallback("agent execute timed out".to_string(), &socket_path);
            Ok(AgentBackendOutcome {
                result: None,
                fallback: Some(fallback),
                socket_path,
            })
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
fn warn_agent_fallback(reason: String, socket_path: &Path) -> AgentFallback {
    static WARN_ONCE: OnceLock<String> = OnceLock::new();
    let reason_with_socket = format!("{} (socket: {})", reason, socket_path.display());
    let stored_reason = WARN_ONCE.get_or_init(|| reason_with_socket.clone()).clone();
    if stored_reason == reason_with_socket {
        eprintln!(
            "[replay] warn: agent replay unavailable ({}); falling back to local backend. Run 'substrate world doctor --json' or set SUBSTRATE_WORLD_SOCKET to point at a healthy agent socket.",
            stored_reason
        );
    }
    AgentFallback {
        reason: stored_reason,
        socket_path: socket_path.to_path_buf(),
    }
}

#[cfg(target_os = "linux")]
fn agent_socket_path(state: &ExecutionState) -> std::path::PathBuf {
    if let Some(from_env) = std::env::var_os("SUBSTRATE_WORLD_SOCKET") {
        return std::path::PathBuf::from(from_env);
    }

    if let Some(transport) = state.recorded_transport.as_ref() {
        if transport.mode == "unix" {
            if let Some(endpoint) = &transport.endpoint {
                return std::path::PathBuf::from(endpoint);
            }
        }
    }

    std::path::PathBuf::from("/run/substrate.sock")
}
