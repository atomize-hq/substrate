//! Socket activation detection helpers for Linux world-service integration.

use std::env;
use std::io::Read;
use std::path::Path;
use std::process::Command;
use std::process::Output;
use std::process::Stdio;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

const SOCKET_PATH: &str = "/run/substrate.sock";
const SOCKET_UNIT: &str = "substrate-world-service.socket";
const SERVICE_UNIT: &str = "substrate-world-service.service";
const DEFAULT_SYSTEMCTL_SHOW_TIMEOUT_MS: u64 = 2_000;

static REPORT_CACHE: OnceLock<Mutex<Option<SocketActivationReport>>> = OnceLock::new();

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum SocketActivationMode {
    Manual,
    SocketActivation,
    Unknown,
}

impl SocketActivationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            SocketActivationMode::Manual => "manual",
            SocketActivationMode::SocketActivation => "socket_activation",
            SocketActivationMode::Unknown => "unknown",
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SystemdUnitStatus {
    pub name: &'static str,
    pub active_state: String,
    pub unit_file_state: String,
    pub listens: Vec<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct SocketActivationReport {
    pub socket_path: String,
    pub socket_exists: bool,
    pub mode: SocketActivationMode,
    pub socket_unit: Option<SystemdUnitStatus>,
    pub service_unit: Option<SystemdUnitStatus>,
    pub systemd_error: Option<String>,
}

impl SocketActivationReport {
    pub fn is_socket_activated(&self) -> bool {
        matches!(self.mode, SocketActivationMode::SocketActivation)
    }
}

impl Default for SocketActivationReport {
    fn default() -> Self {
        let socket_path = resolved_socket_path();
        let socket_exists = Path::new(&socket_path).exists();
        Self {
            socket_path,
            socket_exists,
            mode: SocketActivationMode::Manual,
            socket_unit: None,
            service_unit: None,
            systemd_error: None,
        }
    }
}

pub(crate) fn socket_activation_report() -> SocketActivationReport {
    let cache = REPORT_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().expect("socket activation cache poisoned");
    if let Some(report) = &*guard {
        return report.clone();
    }
    let report = gather_report();
    *guard = Some(report.clone());
    report
}

pub(crate) fn refresh_socket_activation_report() -> SocketActivationReport {
    let cache = REPORT_CACHE.get_or_init(|| Mutex::new(None));
    let mut guard = cache.lock().expect("socket activation cache poisoned");
    let report = gather_report();
    *guard = Some(report.clone());
    report
}

fn gather_report() -> SocketActivationReport {
    let socket_path = resolved_socket_path();
    let socket_exists = Path::new(&socket_path).exists();
    if let Ok(force_mode) = env::var("SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE") {
        let mode = match force_mode.as_str() {
            "socket_activation" => SocketActivationMode::SocketActivation,
            "manual" => SocketActivationMode::Manual,
            _ => SocketActivationMode::Unknown,
        };
        return SocketActivationReport {
            socket_path,
            socket_exists,
            mode,
            socket_unit: None,
            service_unit: None,
            systemd_error: Some("overridden via SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE".into()),
        };
    }

    let mut systemd_error: Option<String> = None;

    let socket_unit = match read_unit(SOCKET_UNIT) {
        Ok(state) => state,
        Err(err) => {
            systemd_error = Some(err);
            None
        }
    };
    let service_unit = match read_unit(SERVICE_UNIT) {
        Ok(state) => state,
        Err(err) => {
            if systemd_error.is_none() {
                systemd_error = Some(err);
            }
            None
        }
    };

    let mut mode = SocketActivationMode::Manual;
    if let Some(unit) = &socket_unit {
        if is_socket_active(unit) {
            mode = SocketActivationMode::SocketActivation;
        } else {
            mode = SocketActivationMode::Unknown;
        }
    } else if systemd_error.is_some() && socket_exists {
        mode = SocketActivationMode::Unknown;
    }

    SocketActivationReport {
        socket_path,
        socket_exists,
        mode,
        socket_unit,
        service_unit,
        systemd_error,
    }
}

fn resolved_socket_path() -> String {
    env::var("SUBSTRATE_WORLD_SOCKET").unwrap_or_else(|_| SOCKET_PATH.to_string())
}

fn is_socket_active(unit: &SystemdUnitStatus) -> bool {
    matches!(
        unit.active_state.as_str(),
        "active" | "listening" | "running" | "activating"
    )
}

fn systemctl_show_timeout() -> Duration {
    env::var("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS")
        .ok()
        .and_then(|raw| raw.parse::<u64>().ok())
        .map(Duration::from_millis)
        .unwrap_or_else(|| Duration::from_millis(DEFAULT_SYSTEMCTL_SHOW_TIMEOUT_MS))
}

fn systemctl_show(unit: &'static str) -> Result<Output, String> {
    let timeout = systemctl_show_timeout();
    let mut child = Command::new("systemctl")
        .arg("--no-pager")
        .arg("show")
        .arg(unit)
        .arg("--property=ActiveState")
        .arg("--property=UnitFileState")
        .arg("--property=Listen")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|err| format!("failed to invoke systemctl: {err}"))?;

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut out) = child.stdout.take() {
                    let _ = out.read_to_end(&mut stdout);
                }
                if let Some(mut err) = child.stderr.take() {
                    let _ = err.read_to_end(&mut stderr);
                }
                return Ok(Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if started.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!(
                        "systemctl show {unit} timed out after {}ms",
                        timeout.as_millis()
                    ));
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(err) => return Err(format!("failed to poll systemctl show {unit}: {err}")),
        }
    }
}

fn read_unit(unit: &'static str) -> Result<Option<SystemdUnitStatus>, String> {
    let output = systemctl_show(unit)?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        if stderr.contains("could not be found") {
            return Ok(None);
        }
        return Err(if stderr.is_empty() {
            format!("systemctl show {unit} exited with {}", output.status)
        } else {
            format!(
                "systemctl show {unit} exited with {} ({stderr})",
                output.status
            )
        });
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut active_state = "unknown".to_string();
    let mut unit_file_state = "unknown".to_string();
    let mut listens = Vec::new();

    for line in stdout.lines() {
        if let Some(value) = line.strip_prefix("ActiveState=") {
            active_state = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("UnitFileState=") {
            unit_file_state = value.trim().to_string();
        } else if let Some(value) = line.strip_prefix("Listen=") {
            let trimmed = value.trim();
            if !trimmed.is_empty() {
                listens.push(trimmed.to_string());
            }
        }
    }

    Ok(Some(SystemdUnitStatus {
        name: unit,
        active_state,
        unit_file_state,
        listens,
    }))
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;

    use serial_test::serial;

    #[test]
    #[serial]
    fn systemctl_timeout_is_fail_fast() {
        let dir = tempfile::tempdir().expect("tempdir");
        let systemctl_path = dir.path().join("systemctl");

        std::fs::write(
            &systemctl_path,
            "#!/usr/bin/env sh\nsleep 10\nprintf 'ActiveState=active\\n'\n",
        )
        .expect("write fake systemctl");

        let mut perms = std::fs::metadata(&systemctl_path)
            .expect("stat fake systemctl")
            .permissions();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            perms.set_mode(0o755);
            std::fs::set_permissions(&systemctl_path, perms).expect("chmod fake systemctl");
        }

        let prev_path = env::var("PATH").ok();
        let prev_timeout = env::var("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS").ok();
        env::set_var(
            "PATH",
            format!(
                "{}:{}",
                dir.path().display(),
                prev_path.clone().unwrap_or_default()
            ),
        );
        env::set_var("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS", "50");

        let report = refresh_socket_activation_report();
        assert!(
            report
                .systemd_error
                .as_deref()
                .unwrap_or("")
                .contains("timed out"),
            "expected a timeout error, got: {:?}",
            report.systemd_error
        );

        match prev_timeout {
            Some(v) => env::set_var("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS", v),
            None => env::remove_var("SUBSTRATE_SYSTEMCTL_TIMEOUT_MS"),
        }
        match prev_path {
            Some(v) => env::set_var("PATH", v),
            None => env::remove_var("PATH"),
        }
    }
}
