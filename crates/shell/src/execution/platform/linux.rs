use crate::execution::socket_activation;
use agent_api_client::AgentClient;
use serde_json::json;
use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use substrate_broker::{detect_profile, world_fs_policy};
use which::which;

pub(crate) fn host_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
    // Helpers
    fn pass(msg: &str) {
        println!("PASS  | {}", msg);
    }
    fn warn(msg: &str) {
        println!("WARN  | {}", msg);
    }
    fn fail(msg: &str) {
        println!("FAIL  | {}", msg);
    }
    fn info(msg: &str) {
        println!("INFO  | {}", msg);
    }

    fn overlay_present() -> bool {
        std::fs::read_to_string("/proc/filesystems")
            .ok()
            .map(|s| s.contains("overlay"))
            .unwrap_or(false)
    }

    fn fuse_dev_present() -> bool {
        Path::new("/dev/fuse").exists()
    }
    fn fuse_bin_present() -> bool {
        which("fuse-overlayfs").is_ok()
    }
    fn cgroup_v2_present() -> bool {
        Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
    }
    fn nft_present() -> bool {
        Command::new("nft")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    fn dmesg_restrict() -> Option<String> {
        Command::new("sh")
            .arg("-lc")
            .arg("sysctl -n kernel.dmesg_restrict 2>/dev/null || echo n/a")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }
    fn overlay_root() -> PathBuf {
        let uid = unsafe { libc::geteuid() } as u32;
        if uid == 0 {
            return PathBuf::from("/var/lib/substrate/overlay");
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("substrate/overlay");
            }
        }
        let run = PathBuf::from(format!("/run/user/{}/substrate/overlay", uid));
        if run.parent().unwrap_or(Path::new("/run")).exists() {
            return run;
        }
        PathBuf::from(format!("/tmp/substrate-{}-overlay", uid))
    }
    fn copydiff_root() -> PathBuf {
        let uid = unsafe { libc::geteuid() } as u32;
        if uid == 0 {
            return PathBuf::from("/var/lib/substrate/copydiff");
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("substrate/copydiff");
            }
        }
        let run = PathBuf::from(format!("/run/user/{}/substrate/copydiff", uid));
        if run.parent().unwrap_or(Path::new("/run")).exists() {
            return run;
        }
        PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid))
    }

    let activation_report = socket_activation::socket_activation_report();

    // Align doctor output with the effective workspace policy when invoked from a workspace.
    //
    // This mirrors the execution path, which refreshes profile/policy per-cwd before reading world_fs.
    if let Ok(cwd) = std::env::current_dir() {
        let _ = detect_profile(&cwd);
    }
    let fs_policy = world_fs_policy();

    let overlay_ok = overlay_present();
    let fuse_dev = fuse_dev_present();
    let fuse_bin = fuse_bin_present();
    let cgv2 = cgroup_v2_present();
    let nft = nft_present();
    let dmsg = dmesg_restrict().unwrap_or_else(|| "n/a".to_string());
    let o_root = overlay_root();
    let c_root = copydiff_root();

    let (socket_probe_ok, socket_probe_error) = if !world_enabled {
        (
            false,
            Some("world disabled by effective config".to_string()),
        )
    } else if activation_report.socket_exists {
        match probe_world_socket(&activation_report.socket_path) {
            Ok(()) => (true, None),
            Err(err) => (false, Some(err.to_string())),
        }
    } else {
        (false, None)
    };

    let host_ok = world_enabled
        && activation_report.socket_exists
        && socket_probe_ok
        && (overlay_ok || (fuse_dev && fuse_bin))
        && cgv2
        && nft;

    if json_mode {
        let socket_json = json!({
            "mode": if activation_report.is_socket_activated() { "socket_activation" } else { "manual" },
            "socket_path": activation_report.socket_path,
            "socket_exists": activation_report.socket_exists,
            "probe_ok": socket_probe_ok,
            "probe_error": socket_probe_error,
            "systemd_error": activation_report.systemd_error,
            "systemd_socket": activation_report.socket_unit.as_ref().map(|unit| json!({
                "name": unit.name,
                "active_state": unit.active_state,
                "unit_file_state": unit.unit_file_state,
                "listens": unit.listens,
            })),
            "systemd_service": activation_report.service_unit.as_ref().map(|unit| json!({
                "name": unit.name,
                "active_state": unit.active_state,
                "unit_file_state": unit.unit_file_state,
                "listens": unit.listens,
            })),
        });

        let out = json!({
            "schema_version": 1,
            "platform": "linux",
            "world_enabled": world_enabled,
            "ok": host_ok,
            "host": {
                "platform": "linux",
                "ok": host_ok,
                "overlay_present": overlay_ok,
                "fuse": {"dev": fuse_dev, "bin": fuse_bin},
                "cgroup_v2": cgv2,
                "nft_present": nft,
                "dmesg_restrict": dmsg,
                "overlay_root": o_root,
                "copydiff_root": c_root,
                "world_fs_mode": fs_policy.mode.as_str(),
                "world_fs_isolation": fs_policy.isolation.as_str(),
                "world_fs_require_world": fs_policy.require_world,
                "world_socket": socket_json,
            },
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate host doctor ==");
        if !world_enabled {
            fail("world isolation disabled by effective config (--no-world)");
        }

        if overlay_ok {
            pass("overlayfs: present");
        } else if fuse_dev && fuse_bin {
            warn("overlayfs: missing; fuse-overlayfs available as fallback");
        } else {
            fail("overlayfs: missing (and fuse-overlayfs unavailable)");
        }

        if fuse_dev && fuse_bin {
            pass("fuse-overlayfs: available");
        } else if fuse_dev || fuse_bin {
            warn(&format!(
                "fuse-overlayfs: partial ({}, {})",
                if fuse_dev {
                    "/dev/fuse"
                } else {
                    "missing /dev/fuse"
                },
                if fuse_bin {
                    "binary found"
                } else {
                    "missing binary"
                }
            ));
        } else {
            warn("fuse-overlayfs: not available");
        }

        if cgv2 {
            pass("cgroup v2: present");
        } else {
            fail("cgroup v2: missing");
        }
        if nft {
            pass("nft: present");
        } else {
            fail("nft: missing");
        }

        info(&format!("dmesg_restrict={}", dmsg));
        info(&format!("overlay_root: {}", o_root.display()));
        info(&format!("copydiff_root: {}", c_root.display()));
        info(&format!(
            "world_fs: mode={} isolation={} require_world={}",
            fs_policy.mode.as_str(),
            fs_policy.isolation.as_str(),
            fs_policy.require_world
        ));

        if activation_report.socket_exists {
            match (activation_report.is_socket_activated(), socket_probe_ok) {
                (true, true) => pass("world-agent socket: systemd-managed and reachable"),
                (true, false) => fail("world-agent socket: systemd-managed but unreachable"),
                (false, true) => pass("world-agent socket: reachable"),
                (false, false) => fail("world-agent socket: present but unreachable"),
            }
        } else {
            fail(&format!(
                "world-agent socket: missing at {}",
                activation_report.socket_path
            ));
        }

        if let Some(err) = &socket_probe_error {
            info(&format!("world_socket.probe_error: {err}"));
        }
    }

    if world_enabled && host_ok {
        0
    } else {
        4
    }
}

pub(crate) fn world_doctor_main(json_mode: bool, world_enabled: bool) -> i32 {
    // Helpers
    fn pass(msg: &str) {
        println!("PASS  | {}", msg);
    }
    fn warn(msg: &str) {
        println!("WARN  | {}", msg);
    }
    fn fail(msg: &str) {
        println!("FAIL  | {}", msg);
    }
    fn info(msg: &str) {
        println!("INFO  | {}", msg);
    }

    fn overlay_present() -> bool {
        std::fs::read_to_string("/proc/filesystems")
            .ok()
            .map(|s| s.contains("overlay"))
            .unwrap_or(false)
    }

    fn fuse_dev_present() -> bool {
        Path::new("/dev/fuse").exists()
    }
    fn fuse_bin_present() -> bool {
        which("fuse-overlayfs").is_ok()
    }
    fn cgroup_v2_present() -> bool {
        Path::new("/sys/fs/cgroup/cgroup.controllers").exists()
    }
    fn nft_present() -> bool {
        Command::new("nft")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .ok()
            .map(|s| s.success())
            .unwrap_or(false)
    }
    fn dmesg_restrict() -> Option<String> {
        Command::new("sh")
            .arg("-lc")
            .arg("sysctl -n kernel.dmesg_restrict 2>/dev/null || echo n/a")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
    }
    fn overlay_root() -> PathBuf {
        let uid = unsafe { libc::geteuid() } as u32;
        if uid == 0 {
            return PathBuf::from("/var/lib/substrate/overlay");
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("substrate/overlay");
            }
        }
        let run = PathBuf::from(format!("/run/user/{}/substrate/overlay", uid));
        if run.parent().unwrap_or(Path::new("/run")).exists() {
            return run;
        }
        PathBuf::from(format!("/tmp/substrate-{}-overlay", uid))
    }
    fn copydiff_root() -> PathBuf {
        let uid = unsafe { libc::geteuid() } as u32;
        if uid == 0 {
            return PathBuf::from("/var/lib/substrate/copydiff");
        }
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            if !xdg.is_empty() {
                return PathBuf::from(xdg).join("substrate/copydiff");
            }
        }
        let run = PathBuf::from(format!("/run/user/{}/substrate/copydiff", uid));
        if run.parent().unwrap_or(Path::new("/run")).exists() {
            return run;
        }
        PathBuf::from(format!("/tmp/substrate-{}-copydiff", uid))
    }

    let activation_report = socket_activation::socket_activation_report();
    if let Ok(cwd) = std::env::current_dir() {
        let _ = detect_profile(&cwd);
    }
    let fs_policy = world_fs_policy();

    let overlay_ok = overlay_present();
    let fuse_dev = fuse_dev_present();
    let fuse_bin = fuse_bin_present();
    let cgv2 = cgroup_v2_present();
    let nft = nft_present();
    let dmsg = dmesg_restrict().unwrap_or_else(|| "n/a".to_string());
    let o_root = overlay_root();
    let c_root = copydiff_root();

    // World doctor short-circuit: no socket probing, no agent calls.
    let (socket_probe_ok, socket_probe_error) = if !world_enabled {
        (
            false,
            Some("world disabled by effective config".to_string()),
        )
    } else if activation_report.socket_exists {
        match probe_world_socket(&activation_report.socket_path) {
            Ok(()) => (true, None),
            Err(err) => (false, Some(err.to_string())),
        }
    } else {
        (false, None)
    };

    let host_ok = world_enabled
        && activation_report.socket_exists
        && socket_probe_ok
        && (overlay_ok || (fuse_dev && fuse_bin))
        && cgv2
        && nft;

    let host_value = {
        let socket_json = json!({
            "mode": if activation_report.is_socket_activated() { "socket_activation" } else { "manual" },
            "socket_path": activation_report.socket_path,
            "socket_exists": activation_report.socket_exists,
            "probe_ok": socket_probe_ok,
            "probe_error": socket_probe_error,
            "systemd_error": activation_report.systemd_error,
            "systemd_socket": activation_report.socket_unit.as_ref().map(|unit| json!({
                "name": unit.name,
                "active_state": unit.active_state,
                "unit_file_state": unit.unit_file_state,
                "listens": unit.listens,
            })),
            "systemd_service": activation_report.service_unit.as_ref().map(|unit| json!({
                "name": unit.name,
                "active_state": unit.active_state,
                "unit_file_state": unit.unit_file_state,
                "listens": unit.listens,
            })),
        });

        json!({
            "platform": "linux",
            "ok": host_ok,
            "overlay_present": overlay_ok,
            "fuse": {"dev": fuse_dev, "bin": fuse_bin},
            "cgroup_v2": cgv2,
            "nft_present": nft,
            "dmesg_restrict": dmsg,
            "overlay_root": o_root,
            "copydiff_root": c_root,
            "world_fs_mode": fs_policy.mode.as_str(),
            "world_fs_isolation": fs_policy.isolation.as_str(),
            "world_fs_require_world": fs_policy.require_world,
            "world_socket": socket_json,
        })
    };

    let mut exit_code = 4;
    let world_value = if !world_enabled {
        json!({"status": "disabled", "ok": false})
    } else if !activation_report.socket_exists {
        json!({"status": "not_provisioned", "ok": false})
    } else if !socket_probe_ok {
        if json_mode {
            let detail = socket_probe_error
                .as_deref()
                .unwrap_or("world-agent socket probe failed");
            if activation_report.is_socket_activated() {
                eprintln!("world-agent readiness (socket activation) probe failed: {detail}");
            } else {
                eprintln!("world-agent readiness probe failed: {detail}");
            }
        }
        exit_code = 3;
        json!({"status": "unreachable", "ok": false})
    } else {
        let report = match tokio::runtime::Runtime::new() {
            Ok(rt) => Some(rt.block_on(async {
                let client = AgentClient::unix_socket(activation_report.socket_path.as_str())?;
                client.doctor_world().await
            })),
            Err(err) => {
                if json_mode {
                    eprintln!("substrate world doctor: internal error: failed to create tokio runtime: {err}");
                }
                exit_code = 1;
                None
            }
        };

        match report {
            None => json!({"status": "unreachable", "ok": false}),
            Some(report) => match report {
                Ok(report) => {
                    let status = if report.ok { "ok" } else { "missing_prereqs" };
                    let mut value = serde_json::to_value(report).unwrap_or_else(|_| json!({}));
                    if let Some(obj) = value.as_object_mut() {
                        obj.insert("status".to_string(), json!(status));
                    }
                    if host_ok && value.get("ok").and_then(serde_json::Value::as_bool) == Some(true)
                    {
                        exit_code = 0;
                    } else {
                        exit_code = 4;
                    }
                    value
                }
                Err(_) => {
                    if json_mode {
                        if activation_report.is_socket_activated() {
                            eprintln!("world-agent readiness (socket activation) request failed");
                        } else {
                            eprintln!("world-agent readiness request failed");
                        }
                    }
                    exit_code = 3;
                    json!({"status": "unreachable", "ok": false})
                }
            },
        }
    };

    let ok = host_ok && world_value.get("ok").and_then(serde_json::Value::as_bool) == Some(true);

    if json_mode {
        let out = json!({
            "schema_version": 1,
            "platform": "linux",
            "world_enabled": world_enabled,
            "ok": ok,
            "host": host_value,
            "world": world_value,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate world doctor ==");
        println!("== Host ==");

        if !world_enabled {
            fail("world isolation disabled by effective config (--no-world)");
        }

        if overlay_ok {
            pass("overlayfs: present");
        } else if fuse_dev && fuse_bin {
            warn("overlayfs: missing; fuse-overlayfs available as fallback");
        } else {
            fail("overlayfs: missing (and fuse-overlayfs unavailable)");
        }

        if fuse_dev && fuse_bin {
            pass("fuse-overlayfs: available");
        } else if fuse_dev || fuse_bin {
            warn(&format!(
                "fuse-overlayfs: partial ({}, {})",
                if fuse_dev {
                    "/dev/fuse"
                } else {
                    "missing /dev/fuse"
                },
                if fuse_bin {
                    "binary found"
                } else {
                    "missing binary"
                }
            ));
        } else {
            warn("fuse-overlayfs: not available");
        }

        if cgv2 {
            pass("cgroup v2: present");
        } else {
            fail("cgroup v2: missing");
        }
        if nft {
            pass("nft: present");
        } else {
            fail("nft: missing");
        }

        info(&format!("dmesg_restrict={}", dmsg));
        info(&format!("overlay_root: {}", o_root.display()));
        info(&format!("copydiff_root: {}", c_root.display()));
        info(&format!(
            "world_fs: mode={} isolation={} require_world={}",
            fs_policy.mode.as_str(),
            fs_policy.isolation.as_str(),
            fs_policy.require_world
        ));

        if activation_report.socket_exists {
            match (activation_report.is_socket_activated(), socket_probe_ok) {
                (true, true) => pass("world-agent socket: systemd-managed and reachable"),
                (true, false) => fail("world-agent socket: systemd-managed but unreachable"),
                (false, true) => pass("world-agent socket: reachable"),
                (false, false) => fail("world-agent socket: present but unreachable"),
            }
        } else {
            fail(&format!(
                "world-agent socket: missing at {}",
                activation_report.socket_path
            ));
        }

        if let Some(err) = &socket_probe_error {
            info(&format!("world_socket.probe_error: {err}"));
        }

        println!("== World ==");
        match world_value
            .get("status")
            .and_then(serde_json::Value::as_str)
        {
            Some("disabled") => fail("world doctor disabled (world isolation is off)"),
            Some("not_provisioned") => {
                fail("world backend not provisioned (missing socket/service)")
            }
            Some("unreachable") => fail("world backend unreachable (agent did not respond)"),
            Some("missing_prereqs") | Some("ok") => {
                let ok = world_value
                    .get("ok")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                let landlock_supported = world_value
                    .get("landlock")
                    .and_then(|l| l.get("supported"))
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false);
                let landlock_abi = world_value
                    .get("landlock")
                    .and_then(|l| l.get("abi"))
                    .and_then(serde_json::Value::as_u64);
                if landlock_supported {
                    pass(&format!(
                        "landlock: supported{}",
                        landlock_abi
                            .map(|abi| format!(" (abi {abi})"))
                            .unwrap_or_default()
                    ));
                } else {
                    fail("landlock: unsupported");
                }
                let probe_result = world_value
                    .get("world_fs_strategy")
                    .and_then(|w| w.get("probe"))
                    .and_then(|p| p.get("result"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("fail");
                if probe_result == "pass" {
                    pass("world fs strategy probe: pass");
                } else {
                    fail("world fs strategy probe: fail");
                }
                if ok {
                    pass("world doctor: ok");
                } else {
                    fail("world doctor: ok=false");
                }
            }
            _ => fail("world doctor: unknown status"),
        }
    }

    exit_code
}

fn probe_world_socket(path: &str) -> io::Result<()> {
    let mut stream = UnixStream::connect(path)?;
    stream.set_read_timeout(Some(Duration::from_secs(2)))?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let request = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    stream.write_all(request)?;
    let mut buf = [0u8; 512];
    let read = stream.read(&mut buf)?;
    if read == 0 {
        return Err(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            "socket returned no data",
        ));
    }
    let response = std::str::from_utf8(&buf[..read]).unwrap_or("");
    if !response.contains(" 200 ") {
        return Err(io::Error::other(format!(
            "unexpected response: {}",
            response.lines().next().unwrap_or("")
        )));
    }
    }
    Ok(())
}
