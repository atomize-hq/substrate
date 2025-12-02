use crate::execution::socket_activation;
use serde_json::json;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use which::which;

pub(crate) fn world_doctor_main(json_mode: bool) -> i32 {
    // Helpers
    fn pass(msg: &str) {
        println!("PASS  | {}", msg);
    }
    fn warn(msg: &str) {
        println!("WARN  | {}", msg);
    }
    // fn fail(msg: &str) { println!("FAIL  | {}", msg); }

    fn overlay_present() -> bool {
        std::fs::read_to_string("/proc/filesystems")
            .ok()
            .map(|s| s.contains("overlay"))
            .unwrap_or(false)
    }

    fn try_modprobe_overlay_if_root() {
        let is_root = unsafe { libc::geteuid() } == 0;
        if !is_root {
            return;
        }
        let _ = Command::new("modprobe").arg("overlay").status();
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

    // overlay
    let mut overlay_ok = overlay_present();
    if !json_mode {
        println!("== substrate world doctor ==");
        if overlay_ok {
            pass("overlayfs: present");
        } else {
            warn("overlayfs: not present; attempting modprobe overlay (root only)");
            try_modprobe_overlay_if_root();
            overlay_ok = overlay_present();
            if overlay_ok {
                pass("overlayfs: present after modprobe");
            } else {
                warn("overlayfs: unavailable");
            }
        }
    } else {
        // still try modprobe if root to improve signal
        if !overlay_ok {
            try_modprobe_overlay_if_root();
            overlay_ok = overlay_present();
        }
    }

    // fuse
    let fuse_dev = fuse_dev_present();
    let fuse_bin = fuse_bin_present();
    if !json_mode {
        if fuse_dev && fuse_bin {
            pass("fuse-overlayfs: /dev/fuse present and binary found");
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
    }

    let cgv2 = cgroup_v2_present();
    let nft = nft_present();
    let dmsg = dmesg_restrict().unwrap_or_else(|| "n/a".to_string());
    let o_root = overlay_root();
    let c_root = copydiff_root();

    if !json_mode {
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
        println!("INFO  | dmesg_restrict={}", dmsg);
        println!("INFO  | overlay_root: {}", o_root.display());
        println!("INFO  | copydiff_root: {}", c_root.display());
        if activation_report.is_socket_activated() {
            pass(&format!(
                "agent socket: systemd-managed ({} {})",
                activation_report
                    .socket_unit
                    .as_ref()
                    .map(|u| u.name)
                    .unwrap_or("substrate-world-agent.socket"),
                activation_report
                    .socket_unit
                    .as_ref()
                    .map(|u| u.active_state.as_str())
                    .unwrap_or("unknown")
            ));
        } else if activation_report.socket_unit.is_some() {
            warn(&format!(
                "agent socket: {} detected but inactive (state: {})",
                activation_report
                    .socket_unit
                    .as_ref()
                    .map(|u| u.name)
                    .unwrap_or("substrate-world-agent.socket"),
                activation_report
                    .socket_unit
                    .as_ref()
                    .map(|u| u.active_state.as_str())
                    .unwrap_or("unknown")
            ));
        } else if activation_report.socket_exists {
            pass(&format!(
                "agent socket: manual listener present at {}",
                activation_report.socket_path
            ));
        } else {
            warn(&format!(
                "agent socket: listener missing at {}; run `substrate world enable`",
                activation_report.socket_path
            ));
        }
    } else {
        let ok = overlay_ok || (fuse_dev && fuse_bin);
        let socket_json = json!({
            "mode": activation_report.mode.as_str(),
            "socket_path": activation_report.socket_path,
            "socket_exists": activation_report.socket_exists,
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
            "platform": std::env::consts::OS,
            "overlay_present": overlay_ok,
            "fuse": {"dev": fuse_dev, "bin": fuse_bin},
            "cgroup_v2": cgv2,
            "nft_present": nft,
            "dmesg_restrict": dmsg,
            "overlay_root": o_root,
            "copydiff_root": c_root,
            "agent_socket": socket_json,
            "ok": ok,
        });
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    }

    // Exit code policy
    if overlay_ok || (fuse_dev && fuse_bin) {
        0
    } else {
        2
    }
}
