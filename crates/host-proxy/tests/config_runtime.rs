#[cfg(unix)]
use std::sync::Mutex;

#[cfg(unix)]
use host_proxy::load_config_from_env;
#[cfg(unix)]
use host_proxy::AgentTransportConfig;
use host_proxy::{cleanup_socket, ensure_socket_dir};
#[cfg(unix)]
use std::path::PathBuf;
use tempfile::tempdir;

#[cfg(unix)]
static ENV_LOCK: Mutex<()> = Mutex::new(());

#[cfg(unix)]
struct EnvGuard {
    previous: Vec<(String, Option<std::ffi::OsString>)>,
}

#[cfg(unix)]
impl EnvGuard {
    fn set(vars: &[(&str, Option<&str>)]) -> Self {
        let previous = vars
            .iter()
            .map(|(key, _)| (key.to_string(), std::env::var_os(key)))
            .collect::<Vec<_>>();

        for (key, value) in vars {
            match value {
                Some(v) => std::env::set_var(key, v),
                None => std::env::remove_var(key),
            }
        }

        Self { previous }
    }
}

#[cfg(unix)]
impl Drop for EnvGuard {
    fn drop(&mut self) {
        for (key, value) in self.previous.drain(..) {
            match value {
                Some(v) => std::env::set_var(&key, v),
                None => std::env::remove_var(&key),
            }
        }
    }
}

#[cfg(unix)]
#[test]
fn load_config_from_env_applies_overrides() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::set(&[
        ("HOST_PROXY_SOCKET", Some("/tmp/custom.sock")),
        ("SUBSTRATE_AGENT_TRANSPORT", Some("tcp://10.1.2.3:4555")),
        ("MAX_BODY_SIZE", Some("4096")),
        ("REQUEST_TIMEOUT", Some("42")),
        ("RATE_LIMIT_RPM", Some("24")),
        ("RATE_LIMIT_CONCURRENT", Some("3")),
        ("AUTH_ENABLED", Some("true")),
        ("AUTH_TOKEN_FILE", Some("/tmp/token.txt")),
        // Clear legacy vars that could interfere with parsing.
        ("AGENT_TRANSPORT", None),
        ("AGENT_TCP_HOST", None),
        ("AGENT_TCP_PORT", None),
        ("AGENT_SOCKET", None),
    ]);

    let config = load_config_from_env().expect("load config from env");
    assert_eq!(config.host_socket, PathBuf::from("/tmp/custom.sock"));
    match config.agent {
        AgentTransportConfig::Tcp { ref host, port } => {
            assert_eq!(host, "10.1.2.3");
            assert_eq!(port, 4555);
        }
        other => panic!("expected tcp transport, got {other:?}"),
    }
    assert_eq!(config.max_body_size, 4096);
    assert_eq!(config.request_timeout, 42);
    assert_eq!(config.rate_limits.requests_per_minute, 24);
    assert_eq!(config.rate_limits.max_concurrent, 3);
    assert!(config.auth.enabled);
    assert_eq!(
        config.auth.token_file.as_deref(),
        Some(PathBuf::from("/tmp/token.txt").as_path())
    );
}

#[cfg(unix)]
#[test]
fn load_config_from_env_supports_legacy_tcp_vars() {
    let _lock = ENV_LOCK.lock().unwrap();
    let _guard = EnvGuard::set(&[
        ("HOST_PROXY_SOCKET", Some("/tmp/legacy.sock")),
        ("AGENT_TRANSPORT", Some("tcp://agent.internal:17780")),
        // Ensure new transport var does not take precedence here.
        ("SUBSTRATE_AGENT_TRANSPORT", None),
        ("AGENT_SOCKET", None),
        ("MAX_BODY_SIZE", None),
        ("REQUEST_TIMEOUT", None),
        ("RATE_LIMIT_RPM", None),
        ("RATE_LIMIT_CONCURRENT", None),
    ]);

    let config = load_config_from_env().expect("load config from env");
    assert_eq!(config.host_socket, PathBuf::from("/tmp/legacy.sock"));
    match config.agent {
        AgentTransportConfig::Tcp { ref host, port } => {
            assert_eq!(host, "agent.internal");
            assert_eq!(port, 17780);
        }
        other => panic!("expected tcp transport, got {other:?}"),
    }
}

#[tokio::test]
async fn ensure_and_cleanup_socket_directory() {
    let temp = tempdir().unwrap();
    let socket_path = temp.path().join("nested").join("agent.sock");

    ensure_socket_dir(&socket_path)
        .await
        .expect("create socket parent");
    assert!(socket_path.parent().unwrap().exists());

    // Create a placeholder socket file to exercise cleanup logic.
    tokio::fs::write(&socket_path, b"placeholder")
        .await
        .expect("write socket placeholder");
    assert!(socket_path.exists());

    cleanup_socket(&socket_path)
        .await
        .expect("cleanup removes socket file");
    assert!(
        !socket_path.exists(),
        "cleanup should remove existing socket file"
    );
}
