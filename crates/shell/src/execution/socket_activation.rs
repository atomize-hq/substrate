//! Socket activation detection helpers for Linux world-agent integration.

use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

const SOCKET_PATH: &str = "/run/substrate.sock";
const SOCKET_UNIT: &str = "substrate-world-agent.socket";
const SERVICE_UNIT: &str = "substrate-world-agent.service";

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
    pub socket_path: &'static str,
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
        Self {
            socket_path: SOCKET_PATH,
            socket_exists: Path::new(SOCKET_PATH).exists(),
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
    let socket_exists = Path::new(SOCKET_PATH).exists();
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
        socket_path: SOCKET_PATH,
        socket_exists,
        mode,
        socket_unit,
        service_unit,
        systemd_error,
    }
}

fn is_socket_active(unit: &SystemdUnitStatus) -> bool {
    matches!(
        unit.active_state.as_str(),
        "active" | "listening" | "running" | "activating"
    )
}

fn read_unit(unit: &'static str) -> Result<Option<SystemdUnitStatus>, String> {
    let output = Command::new("systemctl")
        .arg("show")
        .arg(unit)
        .arg("--property=ActiveState")
        .arg("--property=UnitFileState")
        .arg("--property=Listen")
        .output()
        .map_err(|err| format!("failed to invoke systemctl: {err}"))?;

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
