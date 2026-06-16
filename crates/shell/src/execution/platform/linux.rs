use crate::execution::socket_activation;
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use serde_json::json;
use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::os::unix::fs::{FileTypeExt, MetadataExt};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use substrate_broker::{detect_profile, world_fs_policy};
use transport_api_client::AgentClient;
use transport_api_types::{
    ExecuteRequest, WorldDoctorLandlockV1, WorldDoctorNetfilterStatusV1, WorldDoctorReportV1,
    WorldDoctorWorldFsStrategyKindV1, WorldDoctorWorldFsStrategyProbeResultV1,
    WorldDoctorWorldFsStrategyProbeV1, WorldDoctorWorldFsStrategyV1, WorldFsMode,
};
use which::which;

pub(crate) fn host_doctor_main(
    json_mode: bool,
    world_enabled: bool,
    world_disable_attribution: Option<&crate::execution::config_model::DoctorDisableAttribution>,
) -> i32 {
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

    let (socket_probe_ok, socket_probe_error, socket_probe_error_kind) = if !world_enabled {
        (
            false,
            Some("world disabled by effective config".to_string()),
            None,
        )
    } else if activation_report.socket_exists {
        match probe_world_socket(&activation_report.socket_path) {
            Ok(()) => (true, None, None),
            Err(err) => (false, Some(err.to_string()), Some(err.kind())),
        }
    } else {
        (false, None, None)
    };

    let host_ok = world_enabled
        && activation_report.socket_exists
        && socket_probe_ok
        && (overlay_ok || (fuse_dev && fuse_bin))
        && cgv2
        && nft;

    if json_mode {
        let (socket_acl, socket_acl_error) = if activation_report.socket_exists {
            match read_socket_acl_details(&activation_report.socket_path) {
                Ok(details) => (Some(details), None),
                Err(err) => (None, Some(err.to_string())),
            }
        } else {
            (None, None)
        };

        let socket_access = diagnose_socket_access(
            activation_report.socket_exists,
            activation_report.socket_path.as_str(),
            socket_acl.as_ref(),
            socket_probe_ok,
            socket_probe_error_kind,
        );

        let permission_denied_help =
            permission_denied_help(socket_probe_error_kind, socket_access.as_ref());

        let socket_json = json!({
            "mode": if activation_report.is_socket_activated() { "socket_activation" } else { "manual" },
            "socket_path": activation_report.socket_path,
            "socket_exists": activation_report.socket_exists,
            "authorization_boundary": {
                "kind": "unix_socket_acl",
                "description": "Socket ACL is the caller authorization boundary; any local user that can open the socket can issue world-service requests."
            },
            "socket_acl": socket_acl.as_ref().map(|acl| json!({
                "is_socket": acl.is_socket,
                "owner_uid": acl.owner_uid,
                "owner_user": acl.owner_user.as_deref(),
                "group_gid": acl.group_gid,
                "group_name": acl.group_name.as_deref(),
                "mode_octal": acl.mode_octal.as_str(),
            })),
            "socket_acl_error": socket_acl_error,
            "access": socket_access.as_ref().map(socket_access_json),
            "probe_ok": socket_probe_ok,
            "probe_error": socket_probe_error,
            "probe_error_kind": socket_probe_error_kind.map(|kind| format!("{kind:?}")),
            "permission_denied_help": permission_denied_help,
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

        let mut out = json!({
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
        if let Some(attribution) = world_disable_attribution {
            out["world_disable_reason"] = json!(attribution.reason);
            out["world_disable_source"] = json!(attribution.source);
        }
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate host doctor ==");
        if !world_enabled {
            if let Some(attribution) = world_disable_attribution {
                fail(attribution.reason);
            }
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

        let socket_acl = if activation_report.socket_exists {
            read_socket_acl_details(&activation_report.socket_path).ok()
        } else {
            None
        };
        let socket_access = diagnose_socket_access(
            activation_report.socket_exists,
            activation_report.socket_path.as_str(),
            socket_acl.as_ref(),
            socket_probe_ok,
            socket_probe_error_kind,
        );

        info("authorization boundary: socket ACL (local users with RW access to the socket can issue world-service requests)");
        if activation_report.socket_exists {
            if let Some(acl) = &socket_acl {
                let owner = acl
                    .owner_user
                    .clone()
                    .unwrap_or_else(|| acl.owner_uid.to_string());
                let group = acl
                    .group_name
                    .clone()
                    .unwrap_or_else(|| acl.group_gid.to_string());
                info(&format!(
                    "world-service socket ACL: owner={owner} group={group} mode={}",
                    acl.mode_octal,
                ));
            }
            match (activation_report.is_socket_activated(), socket_probe_ok) {
                (true, true) => pass("world-service socket: systemd-managed and reachable"),
                (true, false) => fail("world-service socket: systemd-managed but unreachable"),
                (false, true) => pass("world-service socket: reachable"),
                (false, false) => fail("world-service socket: present but unreachable"),
            }
        } else {
            fail(&format!(
                "world-service socket: missing at {}",
                activation_report.socket_path
            ));
        }

        if let Some(err) = &socket_probe_error {
            info(&format!("world_socket.probe_error: {err}"));
            if let Some(socket_access) = socket_access.as_ref() {
                info(&format!(
                    "world_socket.access: status={} authorization_source={}",
                    socket_access.status, socket_access.authorization_source
                ));
                if let Some(active) = socket_access.active_process_has_socket_group {
                    info(&format!(
                        "world_socket.access.active_process_has_socket_group={active}"
                    ));
                }
                if let Some(account) = socket_access.account_is_in_socket_group {
                    info(&format!(
                        "world_socket.access.account_is_in_socket_group={account}"
                    ));
                }
                if let Some(named_user_acl) = socket_access.named_user_acl_grants_rw {
                    info(&format!(
                        "world_socket.access.named_user_acl_grants_rw={named_user_acl}"
                    ));
                }
                if let Some(contract_ok) = socket_access.contract_ok {
                    info(&format!("world_socket.access.contract_ok={contract_ok}"));
                }
                if let Some(parse_error) = &socket_access.acl_parse_error {
                    warn(&format!(
                        "world_socket.access.acl_parse_error={parse_error}"
                    ));
                }
                if !socket_access.getfacl_available || !socket_access.setfacl_available {
                    warn(&format!(
                        "world_socket.access.acl_tooling: getfacl_available={} setfacl_available={}",
                        socket_access.getfacl_available, socket_access.setfacl_available
                    ));
                }
            }
            if let Some(help) =
                permission_denied_help(socket_probe_error_kind, socket_access.as_ref())
            {
                info(&help);
            }
        }
    }

    if world_enabled && host_ok {
        0
    } else {
        4
    }
}

pub(crate) fn world_doctor_main(
    json_mode: bool,
    world_enabled: bool,
    world_disable_attribution: Option<&crate::execution::config_model::DoctorDisableAttribution>,
) -> i32 {
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
    let (socket_probe_ok, socket_probe_error, socket_probe_error_kind) = if !world_enabled {
        (
            false,
            Some("world disabled by effective config".to_string()),
            None,
        )
    } else if activation_report.socket_exists {
        match probe_world_socket(&activation_report.socket_path) {
            Ok(()) => (true, None, None),
            Err(err) => (false, Some(err.to_string()), Some(err.kind())),
        }
    } else {
        (false, None, None)
    };

    let host_ok = world_enabled && activation_report.socket_exists && socket_probe_ok;

    let host_value = {
        let (socket_acl, socket_acl_error) = if activation_report.socket_exists {
            match read_socket_acl_details(&activation_report.socket_path) {
                Ok(details) => (Some(details), None),
                Err(err) => (None, Some(err.to_string())),
            }
        } else {
            (None, None)
        };

        let socket_access = diagnose_socket_access(
            activation_report.socket_exists,
            activation_report.socket_path.as_str(),
            socket_acl.as_ref(),
            socket_probe_ok,
            socket_probe_error_kind,
        );

        let permission_denied_help =
            permission_denied_help(socket_probe_error_kind, socket_access.as_ref());

        let socket_json = json!({
            "mode": if activation_report.is_socket_activated() { "socket_activation" } else { "manual" },
            "socket_path": activation_report.socket_path,
            "socket_exists": activation_report.socket_exists,
            "authorization_boundary": {
                "kind": "unix_socket_acl",
                "description": "Socket ACL is the caller authorization boundary; any local user that can open the socket can issue world-service requests."
            },
            "socket_acl": socket_acl.as_ref().map(|acl| json!({
                "is_socket": acl.is_socket,
                "owner_uid": acl.owner_uid,
                "owner_user": acl.owner_user.as_deref(),
                "group_gid": acl.group_gid,
                "group_name": acl.group_name.as_deref(),
                "mode_octal": acl.mode_octal.as_str(),
            })),
            "socket_acl_error": socket_acl_error,
            "access": socket_access.as_ref().map(socket_access_json),
            "probe_ok": socket_probe_ok,
            "probe_error": socket_probe_error,
            "probe_error_kind": socket_probe_error_kind.map(|kind| format!("{kind:?}")),
            "permission_denied_help": permission_denied_help,
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
                .unwrap_or("world-service socket probe failed");
            if activation_report.is_socket_activated() {
                eprintln!("world-service readiness (socket activation) probe failed: {detail}");
            } else {
                eprintln!("world-service readiness probe failed: {detail}");
            }
        }
        exit_code = 3;
        json!({"status": "unreachable", "ok": false})
    } else {
        let report = match tokio::runtime::Runtime::new() {
            Ok(rt) => Some(rt.block_on(async {
                let client = AgentClient::unix_socket(activation_report.socket_path.as_str())?;
                match client.doctor_world().await {
                    Ok(report) => Ok(report),
                    Err(err) => {
                        let message = err.to_string();
                        if message.contains("HTTP 404") {
                            legacy_world_doctor_report_v1_via_execute(&client).await
                        } else {
                            Err(err)
                        }
                    }
                }
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
                            eprintln!("world-service readiness (socket activation) request failed");
                        } else {
                            eprintln!("world-service readiness request failed");
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
        let mut out = json!({
            "schema_version": 1,
            "platform": "linux",
            "world_enabled": world_enabled,
            "ok": ok,
            "host": host_value,
            "world": world_value,
        });
        if let Some(attribution) = world_disable_attribution {
            out["world_disable_reason"] = json!(attribution.reason);
            out["world_disable_source"] = json!(attribution.source);
        }
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        println!("== substrate world doctor ==");
        println!("== Host ==");

        if !world_enabled {
            if let Some(attribution) = world_disable_attribution {
                fail(attribution.reason);
            }
        }

        if overlay_ok {
            pass("overlayfs: present");
        } else if fuse_dev && fuse_bin {
            warn("overlayfs: missing; fuse-overlayfs available as fallback");
        } else {
            warn("overlayfs: missing (and fuse-overlayfs unavailable)");
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
            warn("cgroup v2: missing");
        }
        if nft {
            pass("nft: present");
        } else {
            warn("nft: missing");
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

        let socket_acl = if activation_report.socket_exists {
            read_socket_acl_details(&activation_report.socket_path).ok()
        } else {
            None
        };
        let socket_access = diagnose_socket_access(
            activation_report.socket_exists,
            activation_report.socket_path.as_str(),
            socket_acl.as_ref(),
            socket_probe_ok,
            socket_probe_error_kind,
        );

        info("authorization boundary: socket ACL (local users with RW access to the socket can issue world-service requests)");
        if activation_report.socket_exists {
            if let Some(acl) = &socket_acl {
                let owner = acl
                    .owner_user
                    .clone()
                    .unwrap_or_else(|| acl.owner_uid.to_string());
                let group = acl
                    .group_name
                    .clone()
                    .unwrap_or_else(|| acl.group_gid.to_string());
                info(&format!(
                    "world-service socket ACL: owner={owner} group={group} mode={}",
                    acl.mode_octal
                ));
            }
            match (activation_report.is_socket_activated(), socket_probe_ok) {
                (true, true) => pass("world-service socket: systemd-managed and reachable"),
                (true, false) => fail("world-service socket: systemd-managed but unreachable"),
                (false, true) => pass("world-service socket: reachable"),
                (false, false) => fail("world-service socket: present but unreachable"),
            }
        } else {
            fail(&format!(
                "world-service socket: missing at {}",
                activation_report.socket_path
            ));
        }

        if let Some(err) = &socket_probe_error {
            info(&format!("world_socket.probe_error: {err}"));
            if let Some(socket_access) = socket_access.as_ref() {
                info(&format!(
                    "world_socket.access: status={} authorization_source={}",
                    socket_access.status, socket_access.authorization_source
                ));
                if let Some(active) = socket_access.active_process_has_socket_group {
                    info(&format!(
                        "world_socket.access.active_process_has_socket_group={active}"
                    ));
                }
                if let Some(account) = socket_access.account_is_in_socket_group {
                    info(&format!(
                        "world_socket.access.account_is_in_socket_group={account}"
                    ));
                }
                if let Some(named_user_acl) = socket_access.named_user_acl_grants_rw {
                    info(&format!(
                        "world_socket.access.named_user_acl_grants_rw={named_user_acl}"
                    ));
                }
                if let Some(contract_ok) = socket_access.contract_ok {
                    info(&format!("world_socket.access.contract_ok={contract_ok}"));
                }
                if let Some(parse_error) = &socket_access.acl_parse_error {
                    warn(&format!(
                        "world_socket.access.acl_parse_error={parse_error}"
                    ));
                }
                if !socket_access.getfacl_available || !socket_access.setfacl_available {
                    warn(&format!(
                        "world_socket.access.acl_tooling: getfacl_available={} setfacl_available={}",
                        socket_access.getfacl_available, socket_access.setfacl_available
                    ));
                }
            }
            if let Some(help) =
                permission_denied_help(socket_probe_error_kind, socket_access.as_ref())
            {
                info(&help);
            }
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

                match world_value
                    .get("policy_snapshot_v1_supported")
                    .and_then(serde_json::Value::as_bool)
                {
                    Some(true) => pass("policy snapshot v1: supported"),
                    Some(false) => warn("policy snapshot v1: unsupported"),
                    None => warn("policy snapshot v1: unknown"),
                }

                match world_value
                    .get("policy_resolution_mode")
                    .and_then(serde_json::Value::as_str)
                {
                    Some("snapshot_v3") => pass("policy resolution mode: snapshot_v3"),
                    Some("legacy_local") => warn("policy resolution mode: legacy_local"),
                    Some(other) => warn(&format!("policy resolution mode: {other}")),
                    None => info("policy resolution mode: unknown (no agent executions yet)"),
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

async fn legacy_world_doctor_report_v1_via_execute(
    client: &AgentClient,
) -> anyhow::Result<WorldDoctorReportV1> {
    let landlock = world::landlock::detect_support();
    let cwd_path = std::path::PathBuf::from("/tmp");
    let network_policy =
        crate::execution::policy_snapshot::resolve_world_network_policy_for_cwd(&cwd_path)?;
    let world_network =
        crate::execution::policy_snapshot::request_world_network_routing(&network_policy);
    let requested = world_network.isolate_network;
    let world_netfilter_enable_present = world::netfilter::world_netfilter_enable_present();
    let policy_snapshot = network_policy.snapshot;

    let req = ExecuteRequest {
        profile: None,
        cmd: "sh -lc 'set -e; d=\".substrate_doctor_probe.$$\"; rm -rf \"$d\"; mkdir \"$d\"; cd \"$d\"; rm -f .substrate_enum_probe; touch .substrate_enum_probe; ls -a1; rm -f .substrate_enum_probe; cd ..; rmdir \"$d\"'".to_string(),
        cwd: Some(cwd_path.display().to_string()),
        env: Some(HashMap::new()),
        pty: false,
        agent_id: "doctor-world-probe".to_string(),
        budget: None,
        policy_snapshot,
        shared_world: None,
        world_network: Some(world_network),
        world_fs_mode: Some(WorldFsMode::Writable),
        member_dispatch: None,
    };

    let resp = client.execute(req).await?;
    let stdout = String::from_utf8_lossy(
        &BASE64
            .decode(resp.stdout_b64.as_bytes())
            .unwrap_or_default(),
    )
    .into_owned();
    let stderr = String::from_utf8_lossy(
        &BASE64
            .decode(resp.stderr_b64.as_bytes())
            .unwrap_or_default(),
    )
    .into_owned();

    let (result, failure_reason) =
        if resp.exit == 0 && stdout.lines().any(|l| l == ".substrate_enum_probe") {
            (WorldDoctorWorldFsStrategyProbeResultV1::Pass, None)
        } else if resp.exit == 0 {
            (
                WorldDoctorWorldFsStrategyProbeResultV1::Fail,
                Some("probe file missing from directory enumeration".to_string()),
            )
        } else {
            (
                WorldDoctorWorldFsStrategyProbeResultV1::Fail,
                Some(format!(
                    "probe execute failed (exit={}): {}",
                    resp.exit,
                    stderr.trim()
                )),
            )
        };

    let landlock = WorldDoctorLandlockV1 {
        supported: landlock.supported,
        abi: landlock.abi,
        reason: landlock.reason,
    };
    let probe = WorldDoctorWorldFsStrategyProbeV1 {
        id: "enumeration_v1".to_string(),
        probe_file: ".substrate_enum_probe".to_string(),
        result,
        failure_reason,
    };
    let ok =
        landlock.supported && matches!(probe.result, WorldDoctorWorldFsStrategyProbeResultV1::Pass);

    Ok(WorldDoctorReportV1 {
        schema_version: 2,
        ok,
        collected_at_utc: chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
        policy_snapshot_v1_supported: false,
        policy_resolution_mode: None,
        netfilter_status: Some(WorldDoctorNetfilterStatusV1 {
            requested,
            enabled: requested && world_netfilter_enable_present,
            world_netfilter_enable_present,
            last_failure_reason: None,
        }),
        landlock,
        world_fs_strategy: WorldDoctorWorldFsStrategyV1 {
            primary: WorldDoctorWorldFsStrategyKindV1::Overlay,
            fallback: WorldDoctorWorldFsStrategyKindV1::Fuse,
            probe,
        },
    })
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
    Ok(())
}

#[derive(Debug, Clone)]
struct SocketAclDetails {
    owner_uid: u32,
    owner_user: Option<String>,
    group_gid: u32,
    group_name: Option<String>,
    mode_octal: String,
    is_socket: bool,
}

#[derive(Debug, Clone)]
struct SocketAccessDiagnosis {
    current_uid: u32,
    current_user: Option<String>,
    active_process_has_socket_group: Option<bool>,
    account_is_in_socket_group: Option<bool>,
    named_user_acl_grants_rw: Option<bool>,
    authorization_source: String,
    status: String,
    remediation: Option<String>,
    contract_ok: Option<bool>,
    getfacl_available: bool,
    setfacl_available: bool,
    acl_parse_error: Option<String>,
}

#[derive(Debug, Clone)]
struct ParsedSocketAcl {
    named_users: HashMap<String, u8>,
    mask: u8,
}

fn read_socket_acl_details(path: &str) -> io::Result<SocketAclDetails> {
    let metadata = std::fs::symlink_metadata(path)?;
    let owner_uid = metadata.uid();
    let group_gid = metadata.gid();
    let mode = metadata.mode() & 0o7777;
    let mode_octal = format!("{mode:04o}");
    let is_socket = metadata.file_type().is_socket();

    Ok(SocketAclDetails {
        owner_uid,
        owner_user: lookup_user_by_uid(owner_uid),
        group_gid,
        group_name: lookup_group_by_gid(group_gid),
        mode_octal,
        is_socket,
    })
}

fn socket_access_json(diagnosis: &SocketAccessDiagnosis) -> serde_json::Value {
    json!({
        "current_uid": diagnosis.current_uid,
        "current_user": diagnosis.current_user,
        "active_process_has_socket_group": diagnosis.active_process_has_socket_group,
        "account_is_in_socket_group": diagnosis.account_is_in_socket_group,
        "named_user_acl_grants_rw": diagnosis.named_user_acl_grants_rw,
        "authorization_source": diagnosis.authorization_source,
        "status": diagnosis.status,
        "remediation": diagnosis.remediation,
        "contract_ok": diagnosis.contract_ok,
        "acl_tooling": {
            "getfacl_available": diagnosis.getfacl_available,
            "setfacl_available": diagnosis.setfacl_available,
        },
        "acl_parse_error": diagnosis.acl_parse_error,
    })
}

fn diagnose_socket_access(
    socket_exists: bool,
    socket_path: &str,
    socket_acl: Option<&SocketAclDetails>,
    socket_probe_ok: bool,
    socket_probe_error_kind: Option<io::ErrorKind>,
) -> Option<SocketAccessDiagnosis> {
    let current_uid = unsafe { libc::geteuid() } as u32;
    let current_user = lookup_user_by_uid(current_uid);
    let getfacl_available = which("getfacl").is_ok();
    let setfacl_available = which("setfacl").is_ok();

    if !socket_exists {
        return Some(SocketAccessDiagnosis {
            current_uid,
            current_user,
            active_process_has_socket_group: None,
            account_is_in_socket_group: None,
            named_user_acl_grants_rw: None,
            authorization_source: "denied".to_string(),
            status: "failed.socket_missing".to_string(),
            remediation: Some(
                "provision the Linux world socket so /run/substrate.sock exists".to_string(),
            ),
            contract_ok: Some(false),
            getfacl_available,
            setfacl_available,
            acl_parse_error: None,
        });
    }

    let socket_acl = socket_acl?;
    let contract_ok = socket_contract_matches_expected(socket_acl);
    let active_process_has_socket_group = current_process_has_group(socket_acl.group_gid).ok();
    let account_is_in_socket_group = current_account_has_group(socket_acl.group_gid)
        .ok()
        .flatten();
    let (named_user_acl_grants_rw, acl_parse_error) = named_user_acl_grants_rw(
        socket_path,
        current_uid,
        current_user.as_deref(),
        getfacl_available,
    );
    let world_access_grants_rw = world_access_grants_rw(socket_acl);
    let owner_matches = socket_acl.owner_uid == current_uid;

    let authorization_source = if socket_probe_ok {
        if owner_matches {
            "owner"
        } else if active_process_has_socket_group == Some(true) {
            "active-group"
        } else if named_user_acl_grants_rw == Some(true) {
            "named-user-acl"
        } else if world_access_grants_rw {
            "other/world"
        } else {
            "unknown"
        }
    } else {
        "denied"
    }
    .to_string();

    let status = if !contract_ok && !socket_probe_ok {
        "failed.socket_wrong_owner_group_mode".to_string()
    } else if socket_probe_ok {
        if !contract_ok {
            "degraded.contract_mismatch".to_string()
        } else {
            match authorization_source.as_str() {
                "active-group" => "ok.active_group".to_string(),
                "named-user-acl" => "ok.named_user_acl".to_string(),
                "owner" => "ok.owner".to_string(),
                "other/world" => "ok.other_world".to_string(),
                _ => "ok".to_string(),
            }
        }
    } else if socket_probe_error_kind == Some(io::ErrorKind::PermissionDenied)
        && account_is_in_socket_group == Some(true)
        && active_process_has_socket_group != Some(true)
        && (!getfacl_available || !setfacl_available)
    {
        "degraded.acl_tools_missing".to_string()
    } else if socket_probe_error_kind == Some(io::ErrorKind::PermissionDenied)
        && account_is_in_socket_group == Some(true)
        && active_process_has_socket_group != Some(true)
        && named_user_acl_grants_rw == Some(false)
    {
        "degraded.account_group_member_but_acl_missing".to_string()
    } else if account_is_in_socket_group == Some(false) {
        "failed.not_authorized".to_string()
    } else if socket_probe_error_kind == Some(io::ErrorKind::PermissionDenied) {
        "failed.permission_denied".to_string()
    } else {
        "failed.unreachable".to_string()
    };

    let remediation = match status.as_str() {
        "failed.socket_wrong_owner_group_mode" | "degraded.contract_mismatch" => Some(
            "rerun scripts/linux/world-provision.sh to restore root:substrate 0660 and the socket ACL bridge"
                .to_string(),
        ),
        "degraded.acl_tools_missing" => Some(
            "install ACL tools on the host or refresh the shell with `exec newgrp substrate`"
                .to_string(),
        ),
        "degraded.account_group_member_but_acl_missing" => Some(
            "rerun Linux world provisioning to reapply named-user socket ACLs, or refresh the shell with `exec newgrp substrate`"
                .to_string(),
        ),
        "failed.not_authorized" => Some(
            "add your user to group 'substrate' and reprovision the Linux world socket"
                .to_string(),
        ),
        "failed.permission_denied" => Some(
            "check the socket ACL and ensure the current shell is authorized via the active substrate group or a named-user ACL"
                .to_string(),
        ),
        "failed.unreachable" => Some(
            "inspect the world socket and systemd units with `substrate host doctor --json`"
                .to_string(),
        ),
        "failed.socket_missing" => Some(
            "provision the Linux world socket so /run/substrate.sock exists".to_string(),
        ),
        _ => None,
    };

    Some(SocketAccessDiagnosis {
        current_uid,
        current_user,
        active_process_has_socket_group,
        account_is_in_socket_group,
        named_user_acl_grants_rw,
        authorization_source,
        status,
        remediation,
        contract_ok: Some(contract_ok),
        getfacl_available,
        setfacl_available,
        acl_parse_error,
    })
}

fn socket_contract_matches_expected(socket_acl: &SocketAclDetails) -> bool {
    if socket_acl.owner_uid != 0 || socket_acl.mode_octal != "0660" {
        return false;
    }
    if let Some(expected_gid) = lookup_gid_by_group_name("substrate") {
        return socket_acl.group_gid == expected_gid;
    }
    socket_acl.group_name.as_deref() == Some("substrate")
}

fn world_access_grants_rw(socket_acl: &SocketAclDetails) -> bool {
    socket_acl
        .mode_octal
        .chars()
        .last()
        .and_then(|digit| digit.to_digit(8))
        .map(|bits| bits & 0o6 == 0o6)
        .unwrap_or(false)
}

fn current_process_has_group(target_gid: u32) -> io::Result<bool> {
    if unsafe { libc::getegid() } as u32 == target_gid {
        return Ok(true);
    }

    let count = unsafe { libc::getgroups(0, std::ptr::null_mut()) };
    if count < 0 {
        return Err(io::Error::last_os_error());
    }
    if count == 0 {
        return Ok(false);
    }

    let mut groups = vec![0 as libc::gid_t; count as usize];
    let rc = unsafe { libc::getgroups(count, groups.as_mut_ptr()) };
    if rc < 0 {
        return Err(io::Error::last_os_error());
    }
    Ok(groups.into_iter().any(|gid| gid == target_gid))
}

fn current_account_has_group(target_gid: u32) -> io::Result<Option<bool>> {
    use std::ffi::{CStr, CString};
    use std::ptr;

    let current_uid = unsafe { libc::geteuid() };
    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = ptr::null_mut();
    let buf_len = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let buf_len = if buf_len <= 0 {
        16 * 1024
    } else {
        buf_len as usize
    };
    let mut buf = vec![0u8; buf_len];

    let rc = unsafe {
        libc::getpwuid_r(
            current_uid,
            &mut pwd,
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            &mut result,
        )
    };
    if rc != 0 {
        return Err(io::Error::from_raw_os_error(rc));
    }
    if result.is_null() || pwd.pw_name.is_null() {
        return Ok(None);
    }

    let user_name = unsafe { CStr::from_ptr(pwd.pw_name) }
        .to_string_lossy()
        .into_owned();
    let primary_gid = pwd.pw_gid;
    let c_user_name = CString::new(user_name)
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "username contained NUL"))?;

    let mut ngroups: libc::c_int = 0;
    unsafe {
        libc::getgrouplist(
            c_user_name.as_ptr(),
            primary_gid,
            std::ptr::null_mut(),
            &mut ngroups,
        );
    }
    if ngroups <= 0 {
        return Ok(Some(primary_gid == target_gid));
    }

    let mut groups = vec![0 as libc::gid_t; ngroups as usize];
    let rc = unsafe {
        libc::getgrouplist(
            c_user_name.as_ptr(),
            primary_gid,
            groups.as_mut_ptr(),
            &mut ngroups,
        )
    };
    if rc == -1 {
        return Ok(None);
    }

    groups.truncate(ngroups as usize);
    Ok(Some(groups.into_iter().any(|gid| gid == target_gid)))
}

fn named_user_acl_grants_rw(
    socket_path: &str,
    current_uid: u32,
    current_user: Option<&str>,
    getfacl_available: bool,
) -> (Option<bool>, Option<String>) {
    if !getfacl_available {
        return (None, None);
    }

    match read_parsed_socket_acl(socket_path) {
        Ok(parsed_acl) => {
            let mut candidates = Vec::new();
            if let Some(current_user) = current_user {
                candidates.push(current_user.to_string());
            }
            candidates.push(current_uid.to_string());

            for candidate in candidates {
                if let Some(perms) = parsed_acl.named_users.get(&candidate) {
                    return (Some(has_read_write_bits(*perms & parsed_acl.mask)), None);
                }
            }

            (Some(false), None)
        }
        Err(err) => (None, Some(err.to_string())),
    }
}

fn read_parsed_socket_acl(path: &str) -> io::Result<ParsedSocketAcl> {
    let output = Command::new("getfacl").arg("-cp").arg(path).output()?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        let message = if stderr.is_empty() {
            "getfacl failed".to_string()
        } else {
            stderr
        };
        return Err(io::Error::other(message));
    }
    parse_socket_acl_output(&String::from_utf8_lossy(&output.stdout))
}

fn parse_socket_acl_output(output: &str) -> io::Result<ParsedSocketAcl> {
    let mut named_users = HashMap::new();
    let mut mask = 0o7u8;

    for raw_line in output.lines() {
        let line = raw_line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(':').collect();
        match parts.as_slice() {
            ["user", name, perms] if !name.is_empty() => {
                named_users.insert((*name).to_string(), parse_permission_bits(perms)?);
            }
            ["mask", "", perms] => {
                mask = parse_permission_bits(perms)?;
            }
            ["group", "", perms] => {
                let _ = parse_permission_bits(perms)?;
            }
            ["user", "", perms] | ["other", "", perms] => {
                let _ = parse_permission_bits(perms)?;
            }
            _ => {}
        }
    }

    Ok(ParsedSocketAcl { named_users, mask })
}

fn parse_permission_bits(perms: &str) -> io::Result<u8> {
    let mut bits = 0u8;
    let chars: Vec<char> = perms.trim().chars().collect();
    if chars.len() != 3 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unexpected ACL permissions: {perms}"),
        ));
    }
    if chars[0] == 'r' {
        bits |= 0o4;
    }
    if chars[1] == 'w' {
        bits |= 0o2;
    }
    if chars[2] == 'x' {
        bits |= 0o1;
    }
    Ok(bits)
}

fn has_read_write_bits(bits: u8) -> bool {
    bits & 0o6 == 0o6
}

fn lookup_user_by_uid(uid: u32) -> Option<String> {
    use std::ffi::CStr;
    use std::ptr;

    let mut pwd: libc::passwd = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::passwd = ptr::null_mut();

    let buf_len = unsafe { libc::sysconf(libc::_SC_GETPW_R_SIZE_MAX) };
    let buf_len = if buf_len <= 0 {
        16 * 1024
    } else {
        buf_len as usize
    };
    let mut buf = vec![0u8; buf_len];

    let rc = unsafe {
        libc::getpwuid_r(
            uid as libc::uid_t,
            &mut pwd,
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() || pwd.pw_name.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(pwd.pw_name) }
        .to_string_lossy()
        .into_owned();
    if name.trim().is_empty() {
        None
    } else {
        Some(name)
    }
}

fn lookup_group_by_gid(gid: u32) -> Option<String> {
    use std::ffi::CStr;
    use std::ptr;

    let mut grp: libc::group = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::group = ptr::null_mut();

    let buf_len = unsafe { libc::sysconf(libc::_SC_GETGR_R_SIZE_MAX) };
    let buf_len = if buf_len <= 0 {
        16 * 1024
    } else {
        buf_len as usize
    };
    let mut buf = vec![0u8; buf_len];

    let rc = unsafe {
        libc::getgrgid_r(
            gid as libc::gid_t,
            &mut grp,
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() || grp.gr_name.is_null() {
        return None;
    }
    let name = unsafe { CStr::from_ptr(grp.gr_name) }
        .to_string_lossy()
        .into_owned();
    if name.trim().is_empty() {
        None
    } else {
        Some(name)
    }
}

fn lookup_gid_by_group_name(group_name: &str) -> Option<u32> {
    use std::ffi::CString;
    use std::ptr;

    let c_group_name = CString::new(group_name).ok()?;
    let mut grp: libc::group = unsafe { std::mem::zeroed() };
    let mut result: *mut libc::group = ptr::null_mut();

    let buf_len = unsafe { libc::sysconf(libc::_SC_GETGR_R_SIZE_MAX) };
    let buf_len = if buf_len <= 0 {
        16 * 1024
    } else {
        buf_len as usize
    };
    let mut buf = vec![0u8; buf_len];

    let rc = unsafe {
        libc::getgrnam_r(
            c_group_name.as_ptr(),
            &mut grp,
            buf.as_mut_ptr() as *mut libc::c_char,
            buf.len(),
            &mut result,
        )
    };
    if rc != 0 || result.is_null() {
        return None;
    }
    Some(grp.gr_gid)
}

fn permission_denied_help(
    probe_error_kind: Option<io::ErrorKind>,
    socket_access: Option<&SocketAccessDiagnosis>,
) -> Option<String> {
    if probe_error_kind != Some(io::ErrorKind::PermissionDenied) {
        return None;
    }

    let socket_access = socket_access?;
    let remediation = socket_access.remediation.clone().unwrap_or_else(|| {
        "check the socket ACL and ensure the current shell is authorized".to_string()
    });
    Some(format!(
        "access denied: the socket ACL is the authorization boundary; status={} authorization_source={}; {remediation}",
        socket_access.status, socket_access.authorization_source
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_socket_acl_output_honors_named_users_and_mask() {
        let parsed = parse_socket_acl_output(
            "user::rw-\nuser:spenser:rw-\ngroup::rw-\nmask::r--\nother::---\n",
        )
        .unwrap();

        assert_eq!(parsed.mask, 0o4);
        assert_eq!(parsed.named_users.get("spenser"), Some(&0o6));
        assert!(!has_read_write_bits(
            parsed.named_users["spenser"] & parsed.mask
        ));
    }

    #[test]
    fn world_access_detects_world_writable_socket_bits() {
        let socket_acl = SocketAclDetails {
            owner_uid: 0,
            owner_user: Some("root".to_string()),
            group_gid: 999,
            group_name: Some("substrate".to_string()),
            mode_octal: "0666".to_string(),
            is_socket: true,
        };

        assert!(world_access_grants_rw(&socket_acl));
    }
}
