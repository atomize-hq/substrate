#![cfg(unix)]

use once_cell::sync::Lazy;
use std::io::{Read, Write};
use std::os::fd::IntoRawFd;
use std::os::unix::io::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::process;
use std::sync::Mutex;
use world_agent::socket_activation::{SocketActivationMode, SocketActivationState};

const ACTIVATION_ENV_VARS: &[&str] = &[
    "LISTEN_FDS",
    "LISTEN_PID",
    "LISTEN_FDNAMES",
    "LISTEN_FD_START",
];

static ENV_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[test]
fn direct_bind_mode_when_listen_fds_unset() {
    let state = with_activation_env(&[], || SocketActivationState::detect());
    assert!(matches!(state.mode(), SocketActivationMode::DirectBind));
    let telemetry = state.telemetry();
    assert_eq!(telemetry.mode, "direct_bind");
    assert_eq!(telemetry.fd_count, None);
}

#[test]
fn socket_activation_mode_exposes_inherited_fds() {
    let pid = process::id().to_string();
    let inherited_start = 200;
    with_activation_env(
        &[
            ("LISTEN_FDS", "2"),
            ("LISTEN_PID", &pid),
            ("LISTEN_FDNAMES", "uds:tcp"),
            ("LISTEN_FD_START", &inherited_start.to_string()),
        ],
        || {
            let mut inherited_fds = setup_inherited_fds(inherited_start as RawFd, 2);
            let state = SocketActivationState::detect();

            let duplicates = state
                .duplicate_inherited_fds()
                .expect("duplicating inherited descriptors");
            assert_eq!(duplicates.len(), 2);

            // Convert OwnedFd into UnixStream so we can exchange bytes with the peers.
            let mut paired_streams: Vec<UnixStream> = duplicates
                .into_iter()
                .map(|fd| unsafe { UnixStream::from_raw_fd(fd.into_raw_fd()) })
                .collect();

            for ((socket, peer), name) in paired_streams
                .iter_mut()
                .zip(inherited_fds.peers.iter_mut())
                .zip(["uds", "tcp"])
            {
                socket
                    .write_all(name.as_bytes())
                    .expect("write through inherited fd");
                let mut buf = vec![0_u8; name.len()];
                peer.read_exact(&mut buf).expect("read from peer");
                assert_eq!(buf, name.as_bytes());
            }

            let telemetry = state.telemetry();
            assert_eq!(telemetry.mode, "socket_activation");
            assert_eq!(telemetry.fd_count, Some(2));
            assert_eq!(telemetry.fd_start, Some(inherited_start as RawFd));
            assert_eq!(
                telemetry.fd_names,
                vec!["uds".to_string(), "tcp".to_string()]
            );

            drop(inherited_fds);
        },
    );
}

#[test]
fn honors_custom_fd_start_offsets() {
    let pid = process::id().to_string();
    let custom_start = 9;
    with_activation_env(
        &[
            ("LISTEN_FDS", "3"),
            ("LISTEN_PID", &pid),
            ("LISTEN_FD_START", &custom_start.to_string()),
        ],
        || {
            let inherited_fds = setup_inherited_fds(custom_start, 3);
            let state = SocketActivationState::detect();

            match state.mode() {
                SocketActivationMode::SocketActivated(details) => {
                    assert_eq!(details.fd_start, custom_start);
                    assert_eq!(
                        details.inherited_fds,
                        vec![custom_start, custom_start + 1, custom_start + 2]
                    );
                }
                _ => panic!("expected socket activation mode"),
            }

            drop(inherited_fds);
        },
    );
}

struct InheritedFdSet {
    targets: Vec<RawFd>,
    peers: Vec<UnixStream>,
}

impl Drop for InheritedFdSet {
    fn drop(&mut self) {
        for fd in &self.targets {
            unsafe {
                libc::close(*fd);
            }
        }
    }
}

fn setup_inherited_fds(fd_start: RawFd, count: usize) -> InheritedFdSet {
    let mut targets = Vec::with_capacity(count);
    let mut peers = Vec::with_capacity(count);

    for offset in 0..count {
        let desired = fd_start + offset as RawFd;
        let (left, right) = UnixStream::pair().expect("create unix stream pair");
        unsafe {
            assert!(
                libc::dup2(left.as_raw_fd(), desired) >= 0,
                "dup2 into LISTEN_FD slot"
            );
        }

        targets.push(desired);
        peers.push(right);
    }

    InheritedFdSet { targets, peers }
}

fn with_activation_env<T>(vars: &[(&str, &str)], f: impl FnOnce() -> T) -> T {
    let _lock = ENV_GUARD.lock().unwrap();
    let snapshot: Vec<(String, Option<String>)> = ACTIVATION_ENV_VARS
        .iter()
        .map(|&key| (key.to_string(), std::env::var(key).ok()))
        .collect();

    for key in ACTIVATION_ENV_VARS {
        std::env::remove_var(key);
    }

    for (key, value) in vars {
        std::env::set_var(key, value);
    }

    let output = f();

    for (key, value) in snapshot {
        if let Some(val) = value {
            std::env::set_var(&key, val);
        } else {
            std::env::remove_var(&key);
        }
    }

    output
}
