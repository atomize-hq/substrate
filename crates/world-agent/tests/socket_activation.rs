#![cfg(unix)]

use once_cell::sync::Lazy;
use std::os::fd::RawFd;
use std::os::unix::io::AsRawFd;
use std::os::unix::net::UnixStream;
use std::sync::Mutex;
use tokio::runtime::Runtime;
use world_agent::socket_activation_test_support;

const ACTIVATION_ENV_VARS: &[&str] = &[
    "LISTEN_FDS",
    "LISTEN_PID",
    "LISTEN_FDNAMES",
    "LISTEN_FD_START",
];

static ENV_GUARD: Lazy<Mutex<()>> = Lazy::new(|| Mutex::new(()));

#[test]
fn direct_bind_mode_when_listen_fds_unset() {
    let summary = with_activation_env(&[], || {
        socket_activation_test_support::collect_summary().unwrap()
    });
    assert!(summary.is_none());
}

#[test]
fn socket_activation_mode_exposes_inherited_unix_sockets() {
    let pid = std::process::id().to_string();
    let base_fd = 200;
    with_activation_env(
        &[
            ("LISTEN_FDS", "2"),
            ("LISTEN_PID", &pid),
            ("LISTEN_FDNAMES", "uds:metrics"),
            ("LISTEN_FD_START", &base_fd.to_string()),
        ],
        || {
            let mut inherited = setup_inherited_fds(base_fd as RawFd, 2);
            let summary = socket_activation_test_support::collect_summary()
                .expect("collect summary")
                .expect("expected socket activation");
            inherited.mark_consumed();

            assert_eq!(summary.total_fds, 2);
            assert_eq!(summary.unix_listeners.len(), 2);
            assert!(summary.tcp_listeners.is_empty());
            assert_eq!(
                summary
                    .unix_listeners
                    .iter()
                    .map(|meta| meta.fd)
                    .collect::<Vec<_>>(),
                vec![base_fd, base_fd + 1]
            );
            assert_eq!(summary.unix_listeners[0].name.as_deref(), Some("uds"));
            assert_eq!(summary.unix_listeners[1].name.as_deref(), Some("metrics"));
        },
    );
}

#[test]
fn honors_custom_fd_start_offsets() {
    let pid = std::process::id().to_string();
    let base_fd = 280;
    with_activation_env(
        &[
            ("LISTEN_FDS", "3"),
            ("LISTEN_PID", &pid),
            ("LISTEN_FD_START", &base_fd.to_string()),
        ],
        || {
            let mut inherited = setup_inherited_fds(base_fd as RawFd, 3);
            let summary = socket_activation_test_support::collect_summary()
                .expect("collect summary")
                .expect("expected socket activation");
            inherited.mark_consumed();

            assert_eq!(
                summary
                    .unix_listeners
                    .iter()
                    .map(|meta| meta.fd)
                    .collect::<Vec<_>>(),
                vec![base_fd, base_fd + 1, base_fd + 2]
            );
        },
    );
}

struct InheritedFdSet {
    targets: Vec<RawFd>,
    should_close: bool,
}

impl InheritedFdSet {
    fn mark_consumed(&mut self) {
        self.should_close = false;
    }
}

impl Drop for InheritedFdSet {
    fn drop(&mut self) {
        if self.should_close {
            for fd in &self.targets {
                unsafe {
                    libc::close(*fd);
                }
            }
        }
    }
}

fn setup_inherited_fds(fd_start: RawFd, count: usize) -> InheritedFdSet {
    let mut targets = Vec::with_capacity(count);
    for offset in 0..count {
        let desired = fd_start + offset as RawFd;
        let (left, _) = UnixStream::pair().expect("create unix stream pair");
        unsafe {
            assert!(
                libc::dup2(left.as_raw_fd(), desired) >= 0,
                "dup2 into LISTEN_FD slot"
            );
        }
        targets.push(desired);
    }

    InheritedFdSet {
        targets,
        should_close: true,
    }
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

    let output = with_runtime(f);

    for (key, value) in snapshot {
        if let Some(val) = value {
            std::env::set_var(&key, val);
        } else {
            std::env::remove_var(&key);
        }
    }

    output
}

fn with_runtime<T>(f: impl FnOnce() -> T) -> T {
    let runtime = Runtime::new().expect("create Tokio runtime");
    let guard = runtime.enter();
    let result = f();
    drop(guard);
    runtime.shutdown_background();
    result
}
