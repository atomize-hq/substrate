//! Helpers for simulating/inspecting socket activation (LISTEN_FDS) hand-offs.

use serde::Serialize;

#[cfg(unix)]
mod platform {
    use super::SocketActivationTelemetry;
    use anyhow::{Context, Result};
    use std::ffi::OsString;
    use std::os::unix::io::{FromRawFd, OwnedFd, RawFd};
    use std::process;

    pub const DEFAULT_FD_START: RawFd = 3;

    const LISTEN_FDS_ENV: &str = "LISTEN_FDS";
    const LISTEN_PID_ENV: &str = "LISTEN_PID";
    const LISTEN_FDNAMES_ENV: &str = "LISTEN_FDNAMES";
    const LISTEN_FD_START_ENV: &str = "LISTEN_FD_START";

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SocketActivationMode {
        DirectBind,
        SocketActivated(SocketActivationDetails),
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SocketActivationDetails {
        pub fd_start: RawFd,
        pub inherited_fds: Vec<RawFd>,
        pub fd_names: Vec<String>,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct SocketActivationState {
        mode: SocketActivationMode,
    }

    impl SocketActivationState {
        /// Detect socket activation configuration from the current process environment.
        pub fn detect() -> Self {
            Self::detect_with_reader(StdEnv)
        }

        /// Returns the detected mode.
        pub fn mode(&self) -> &SocketActivationMode {
            &self.mode
        }

        /// Returns telemetry describing the detected mode.
        pub fn telemetry(&self) -> SocketActivationTelemetry {
            match &self.mode {
                SocketActivationMode::DirectBind => SocketActivationTelemetry {
                    mode: "direct_bind",
                    fd_count: None,
                    fd_start: None,
                    fd_names: Vec::new(),
                },
                SocketActivationMode::SocketActivated(details) => SocketActivationTelemetry {
                    mode: "socket_activation",
                    fd_count: Some(details.inherited_fds.len()),
                    fd_start: Some(details.fd_start),
                    fd_names: details.fd_names.clone(),
                },
            }
        }

        /// Duplicates inherited descriptors so tests can consume them without affecting process stdio.
        pub fn duplicate_inherited_fds(&self) -> Result<Vec<OwnedFd>> {
            match &self.mode {
                SocketActivationMode::DirectBind => Ok(Vec::new()),
                SocketActivationMode::SocketActivated(details) => details
                    .inherited_fds
                    .iter()
                    .map(|fd| duplicate_fd(*fd))
                    .collect(),
            }
        }

        #[cfg(test)]
        pub(crate) fn detect_from_env_map(
            entries: impl IntoIterator<Item = (&'static str, String)>,
        ) -> Self {
            let reader = MapEnv {
                vars: entries
                    .into_iter()
                    .map(|(key, value)| (key, OsString::from(value)))
                    .collect(),
            };
            Self::detect_with_reader(reader)
        }

        fn detect_with_reader(reader: impl EnvReader) -> Self {
            let listen_fds = parse_u32(reader.get(LISTEN_FDS_ENV)).unwrap_or(0);
            if listen_fds == 0 {
                return Self {
                    mode: SocketActivationMode::DirectBind,
                };
            }

            let listen_pid = parse_u32(reader.get(LISTEN_PID_ENV)).unwrap_or(0);
            if listen_pid == 0 || listen_pid != process::id() {
                return Self {
                    mode: SocketActivationMode::DirectBind,
                };
            }

            let fd_start =
                parse_fd_start(reader.get(LISTEN_FD_START_ENV)).unwrap_or(DEFAULT_FD_START);

            let inherited_fds = build_fd_sequence(fd_start, listen_fds as usize);
            if inherited_fds.is_empty() {
                return Self {
                    mode: SocketActivationMode::DirectBind,
                };
            }

            let fd_names = parse_fd_names(reader.get(LISTEN_FDNAMES_ENV), inherited_fds.len());

            Self {
                mode: SocketActivationMode::SocketActivated(SocketActivationDetails {
                    fd_start,
                    inherited_fds,
                    fd_names,
                }),
            }
        }
    }

    fn build_fd_sequence(fd_start: RawFd, count: usize) -> Vec<RawFd> {
        if fd_start < 0 {
            return Vec::new();
        }

        let mut fds = Vec::with_capacity(count);
        for idx in 0..count {
            match fd_start.checked_add(idx as RawFd) {
                Some(value) if value >= 0 => fds.push(value),
                _ => return Vec::new(),
            }
        }
        fds
    }

    fn parse_u32(value: Option<OsString>) -> Option<u32> {
        value?.to_str()?.trim().parse().ok()
    }

    fn parse_fd_start(value: Option<OsString>) -> Option<RawFd> {
        value?.to_str()?.trim().parse().ok()
    }

    fn parse_fd_names(value: Option<OsString>, expected: usize) -> Vec<String> {
        let raw = match value {
            Some(val) => val,
            None => return Vec::new(),
        };

        let string = match raw.into_string() {
            Ok(text) => text,
            Err(_) => return Vec::new(),
        };

        let mut names: Vec<String> = string
            .split(':')
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();

        names.truncate(expected);
        names
    }

    fn duplicate_fd(fd: RawFd) -> Result<OwnedFd> {
        // Safety: `fcntl` duplicates the descriptor. Errors handled below.
        let duplicated = unsafe { libc::fcntl(fd, libc::F_DUPFD_CLOEXEC, 0) };
        if duplicated < 0 {
            return Err(std::io::Error::last_os_error())
                .context("Failed to duplicate inherited descriptor");
        }

        // Safety: `duplicated` refers to a valid descriptor returned by fcntl.
        Ok(unsafe { OwnedFd::from_raw_fd(duplicated) })
    }

    trait EnvReader {
        fn get(&self, key: &str) -> Option<OsString>;
    }

    struct StdEnv;

    impl EnvReader for StdEnv {
        fn get(&self, key: &str) -> Option<OsString> {
            std::env::var_os(key)
        }
    }

    #[cfg(test)]
    struct MapEnv {
        vars: std::collections::HashMap<&'static str, OsString>,
    }

    #[cfg(test)]
    impl EnvReader for MapEnv {
        fn get(&self, key: &str) -> Option<OsString> {
            self.vars.get(key).cloned()
        }
    }

    pub use SocketActivationDetails as Details;
    pub use SocketActivationMode as Mode;
    pub use SocketActivationState as State;
}

#[cfg(not(unix))]
mod platform {
    use super::SocketActivationTelemetry;

    pub const DEFAULT_FD_START: i32 = 3;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Mode {
        DirectBind,
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Details;

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct State {
        mode: Mode,
    }

    impl State {
        pub fn detect() -> Self {
            Self {
                mode: Mode::DirectBind,
            }
        }

        pub fn mode(&self) -> &Mode {
            &self.mode
        }

        pub fn telemetry(&self) -> SocketActivationTelemetry {
            SocketActivationTelemetry {
                mode: "direct_bind",
                fd_count: None,
                fd_start: None,
                fd_names: Vec::new(),
            }
        }
    }
}

pub use platform::Details as SocketActivationDetails;
pub use platform::Mode as SocketActivationMode;
pub use platform::State as SocketActivationState;
pub use platform::DEFAULT_FD_START;

/// Structured telemetry describing socket activation status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SocketActivationTelemetry {
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fd_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fd_start: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fd_names: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_manual_mode_when_env_missing() {
        let state = SocketActivationState::detect_from_env_map(std::iter::empty::<(
            &'static str,
            String,
        )>());
        assert!(matches!(state.mode(), SocketActivationMode::DirectBind));
        let telemetry = state.telemetry();
        assert_eq!(telemetry.mode, "direct_bind");
        assert_eq!(telemetry.fd_count, None);
    }

    #[cfg(unix)]
    #[test]
    fn detects_socket_activation_with_valid_env() {
        let pid = std::process::id().to_string();
        let state = SocketActivationState::detect_from_env_map([
            ("LISTEN_FDS", "2".to_string()),
            ("LISTEN_PID", pid.clone()),
            ("LISTEN_FDNAMES", "uds:tcp".to_string()),
        ]);

        match state.mode() {
            SocketActivationMode::SocketActivated(details) => {
                assert_eq!(details.fd_start, DEFAULT_FD_START);
                assert_eq!(details.inherited_fds.len(), 2);
                assert_eq!(details.fd_names, vec!["uds".to_string(), "tcp".to_string()]);
            }
            _ => panic!("expected socket activation mode"),
        }

        let telemetry = state.telemetry();
        assert_eq!(telemetry.mode, "socket_activation");
        assert_eq!(telemetry.fd_count, Some(2));
        assert_eq!(
            telemetry.fd_names,
            vec!["uds".to_string(), "tcp".to_string()]
        );
    }

    #[cfg(unix)]
    #[test]
    fn rejects_socket_activation_when_pid_mismatch() {
        let state = SocketActivationState::detect_from_env_map([
            ("LISTEN_FDS", "1".to_string()),
            ("LISTEN_PID", "999999".to_string()),
        ]);

        assert!(matches!(state.mode(), SocketActivationMode::DirectBind));
    }

    #[cfg(unix)]
    #[test]
    fn honors_custom_fd_start() {
        let pid = std::process::id().to_string();
        let state = SocketActivationState::detect_from_env_map([
            ("LISTEN_FDS", "3".to_string()),
            ("LISTEN_PID", pid),
            ("LISTEN_FD_START", "5".to_string()),
        ]);

        match state.mode() {
            SocketActivationMode::SocketActivated(details) => {
                assert_eq!(details.fd_start, 5);
                assert_eq!(details.inherited_fds, vec![5, 6, 7]);
            }
            _ => panic!("expected socket activation mode"),
        }
    }
}
