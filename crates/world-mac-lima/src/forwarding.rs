//! Forwarding management for host-VM communication.

use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::{debug, info, warn};

/// Forwarding transport kind.
#[derive(Debug, Clone)]
pub enum ForwardingKind {
    /// VSock proxy forwarding
    Vsock { port: u16 },
    /// SSH Unix Domain Socket forwarding
    SshUds { path: PathBuf },
    /// SSH TCP forwarding
    SshTcp { port: u16 },
}

/// Handle for an active forwarding process.
pub struct ForwardingHandle {
    kind: ForwardingKind,
    child: Option<Child>,
}

impl ForwardingHandle {
    /// Get the forwarding kind.
    pub fn kind(&self) -> &ForwardingKind {
        &self.kind
    }
}

impl Drop for ForwardingHandle {
    fn drop(&mut self) {
        debug!("Dropping ForwardingHandle for {:?}", self.kind);

        // Terminate child process if running
        if let Some(mut child) = self.child.take() {
            match child.kill() {
                Ok(_) => {
                    debug!("Killed forwarding process");
                    let _ = child.wait();
                }
                Err(e) => {
                    warn!("Failed to kill forwarding process: {}", e);
                }
            }
        }

        // Clean up sockets if needed
        if let ForwardingKind::SshUds { ref path } = self.kind {
            if path.exists() {
                if let Err(e) = std::fs::remove_file(path) {
                    warn!("Failed to remove socket file: {}", e);
                }
            }
        }
    }
}

/// Auto-select and establish forwarding.
pub fn auto_select(vm_name: &str) -> Result<ForwardingHandle> {
    eprintln!(
        "DEBUG: Auto-selecting forwarding transport for VM '{}'",
        vm_name
    );
    info!("Auto-selecting forwarding transport for VM '{}'", vm_name);

    // Try VSock first
    if vsock_supported() {
        eprintln!("DEBUG: VSock proxy is available, attempting VSock forwarding");
        info!("VSock proxy is available, attempting VSock forwarding");
        match create_vsock_forwarding(vm_name) {
            Ok(handle) => {
                eprintln!("DEBUG: Successfully established VSock forwarding");
                info!("Successfully established VSock forwarding");
                return Ok(handle);
            }
            Err(e) => {
                eprintln!("DEBUG: VSock forwarding failed: {}", e);
                warn!("VSock forwarding failed, falling back: {}", e);
            }
        }
    } else {
        eprintln!("DEBUG: VSock proxy not available");
        info!("VSock proxy not available");
    }

    // Try SSH UDS
    if ssh_available() {
        eprintln!("DEBUG: SSH is available, attempting SSH Unix socket forwarding");
        info!("SSH is available, attempting SSH Unix socket forwarding");
        match create_ssh_uds_forwarding(vm_name) {
            Ok(handle) => {
                eprintln!("DEBUG: Successfully established SSH Unix socket forwarding");
                info!("Successfully established SSH Unix socket forwarding");
                return Ok(handle);
            }
            Err(e) => {
                eprintln!("DEBUG: SSH UDS forwarding failed: {}", e);
                warn!("SSH UDS forwarding failed, falling back: {}", e);
            }
        }

        // SSH TCP fallback requires a TCP <-> UDS bridge inside the guest (e.g., socat).
        // We do not provision that in the base image, and the agent listens on UDS only.
        // To avoid confusing half-open forwards, we intentionally skip TCP fallback here.
        warn!("Skipping SSH TCP fallback: agent uses UDS; enable vsock-proxy or fix SSH UDS");
    } else {
        eprintln!("DEBUG: SSH is not available");
        warn!("SSH is not available");
    }

    anyhow::bail!("No forwarding transport available. Run scripts/mac/lima-doctor.sh")
}

fn vsock_supported() -> bool {
    // Check if vsock-proxy is available
    which::which("vsock-proxy").is_ok()
}

fn ssh_available() -> bool {
    which::which("ssh").is_ok()
}

fn probe_caps_uds(path: &Path) -> bool {
    use std::io::{Read as _, Write as _};
    use std::os::unix::net::UnixStream;

    let Ok(mut stream) = UnixStream::connect(path) else {
        return false;
    };
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
    let request = b"GET /v1/capabilities HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
    if stream.write_all(request).is_err() {
        return false;
    }

    let mut buf = [0u8; 512];
    match stream.read(&mut buf) {
        Ok(n) if n > 0 => std::str::from_utf8(&buf[..n])
            .unwrap_or("")
            .contains(" 200 "),
        _ => false,
    }
}

fn create_vsock_forwarding(vm_name: &str) -> Result<ForwardingHandle> {
    // Find available port
    let port = 17788u16;

    // Start vsock-proxy
    let child = Command::new("vsock-proxy")
        .args([
            "--vm",
            vm_name,
            &port.to_string(),
            "unix:///run/substrate.sock",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start vsock-proxy")?;

    // Wait for proxy to be ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Health check
    let url = format!("http://127.0.0.1:{}/v1/capabilities", port);
    let response = ureq::get(&url)
        .timeout(std::time::Duration::from_secs(2))
        .call();

    if response.is_err() {
        anyhow::bail!("VSock proxy health check failed");
    }

    Ok(ForwardingHandle {
        kind: ForwardingKind::Vsock { port },
        child: Some(child),
    })
}

fn create_ssh_uds_forwarding(vm_name: &str) -> Result<ForwardingHandle> {
    // Create socket directory
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("No home directory"))?;
    let socket_dir = home_dir.join(".substrate/sock");

    debug!("Creating socket directory: {}", socket_dir.display());
    std::fs::create_dir_all(&socket_dir).context("Failed to create socket directory")?;

    // Set permissions to 0700
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = std::fs::metadata(&socket_dir)?.permissions();
        perms.set_mode(0o700);
        std::fs::set_permissions(&socket_dir, perms)?;
    }

    let socket_path = socket_dir.join("agent.sock");

    // Remove old socket if exists
    if socket_path.exists() {
        debug!("Removing old socket file");
        std::fs::remove_file(&socket_path)?;
    }

    let ssh_config = lima_ssh_config_path(vm_name)?;
    debug!("Using SSH config: {}", ssh_config.display());

    let ssh_config_str = ssh_config.to_string_lossy();
    let socket_forward = format!("{}:/run/substrate.sock", socket_path.display());
    let vm_host = format!("lima-{}", vm_name);
    let known_hosts_path = home_dir.join(".substrate/lima_known_hosts");

    // Keep SSH non-interactive and fail-fast so we don't silently hang on prompts.
    //
    // `StrictHostKeyChecking=accept-new` avoids requiring a manual `limactl shell` run solely to
    // accept the VM's host key. Use a Substrate-scoped known_hosts file to avoid mutating the
    // user's global SSH state.
    let ssh_args: Vec<String> = vec![
        "-F".to_string(),
        ssh_config_str.to_string(),
        "-o".to_string(),
        "ControlMaster=no".to_string(),
        "-o".to_string(),
        "ControlPath=none".to_string(),
        "-o".to_string(),
        "BatchMode=yes".to_string(),
        "-o".to_string(),
        "ConnectTimeout=5".to_string(),
        "-o".to_string(),
        "ExitOnForwardFailure=yes".to_string(),
        "-o".to_string(),
        "StreamLocalBindUnlink=yes".to_string(),
        "-o".to_string(),
        "StrictHostKeyChecking=accept-new".to_string(),
        "-o".to_string(),
        format!("UserKnownHostsFile={}", known_hosts_path.display()),
        "-o".to_string(),
        "GlobalKnownHostsFile=/dev/null".to_string(),
        "-L".to_string(),
        socket_forward,
        vm_host,
        "-N".to_string(),
    ];

    debug!("Running SSH command: ssh {:?}", ssh_args);

    // Start SSH forwarding
    let mut cmd = Command::new("ssh");
    cmd.args(&ssh_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;

        // Keep the forwarding tunnel out of the shell foreground process group so Ctrl-C meant
        // for the in-world workload does not also kill the SSH transport underneath it.
        cmd.process_group(0);
    }
    let mut child = cmd.spawn().context("Failed to start SSH UDS forwarding")?;

    debug!("SSH process spawned with PID: {:?}", child.id());

    // Wait for the local socket and a successful capabilities probe (up to ~10s).
    let mut saw_socket = false;
    let mut stderr_buf = String::new();
    for i in 0..20 {
        if socket_path.exists() {
            saw_socket = true;
        }
        if saw_socket && probe_caps_uds(&socket_path) {
            info!(
                "SSH UDS forwarding established at {} and passed capabilities health check",
                socket_path.display()
            );
            return Ok(ForwardingHandle {
                kind: ForwardingKind::SshUds { path: socket_path },
                child: Some(child),
            });
        }

        // If ssh already exited, surface stderr to make failures actionable.
        match child.try_wait() {
            Ok(Some(status)) => {
                if let Some(mut stderr) = child.stderr.take() {
                    use std::io::Read as _;
                    let _ = stderr.read_to_string(&mut stderr_buf);
                }
                anyhow::bail!(
                    "SSH UDS forwarding failed (ssh exited: {status})\nSSH command: ssh {args}\nSSH stderr:\n{stderr}",
                    status = status,
                    args = ssh_args.join(" "),
                    stderr = stderr_buf.trim()
                );
            }
            Ok(None) => {}
            Err(err) => {
                warn!("Failed to poll SSH forwarding process: {err}");
            }
        }

        debug!(
            "Waiting for SSH UDS forwarding health check... attempt {}/20",
            i + 1
        );
        std::thread::sleep(Duration::from_millis(500));
    }

    // Best-effort: if ssh is still running but socket never appeared, capture any stderr already emitted.
    if let Some(mut stderr) = child.stderr.take() {
        use std::io::Read as _;
        let _ = stderr.read_to_string(&mut stderr_buf);
    }

    anyhow::bail!(
        "SSH UDS forwarding failed to establish - capabilities probe never succeeded at {} (socket_seen={})\nSSH command: ssh {args}\nSSH stderr:\n{stderr}",
        socket_path.display(),
        saw_socket,
        args = ssh_args.join(" "),
        stderr = stderr_buf.trim()
    )
}

#[allow(dead_code)]
fn create_ssh_tcp_forwarding(vm_name: &str) -> Result<ForwardingHandle> {
    let port = 17788u16;

    let ssh_config = lima_ssh_config_path(vm_name)?;
    let ssh_config_str = ssh_config.to_string_lossy();
    let port_forward = format!("127.0.0.1:{}:/run/substrate.sock", port);
    let vm_host = format!("lima-{}", vm_name);

    // Start SSH TCP forwarding (note: this requires a TCP<->UDS bridge in the guest to be usable)
    let child = Command::new("ssh")
        .args([
            "-F",
            &ssh_config_str,
            "-o",
            "ControlMaster=no",
            "-o",
            "ControlPath=none",
            "-o",
            "ExitOnForwardFailure=yes",
            "-L",
            &port_forward,
            &vm_host,
            "-N",
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start SSH TCP forwarding")?;

    // Wait for port to be available
    std::thread::sleep(std::time::Duration::from_millis(1000));

    // Try to connect
    if let Err(e) = std::net::TcpStream::connect(format!("127.0.0.1:{}", port)) {
        anyhow::bail!("SSH TCP forwarding failed to establish: {}", e);
    }

    Ok(ForwardingHandle {
        kind: ForwardingKind::SshTcp { port },
        child: Some(child),
    })
}

fn lima_home_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("LIMA_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(".lima"))
}

fn lima_ssh_config_path(vm_name: &str) -> Result<PathBuf> {
    let path = lima_home_dir()?.join(vm_name).join("ssh.config");

    if !path.exists() {
        anyhow::bail!("Lima SSH config not found at: {}", path.display());
    }

    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use std::fs;
    use std::io::{Read as _, Write as _};
    use std::os::unix::net::UnixListener;
    use tempfile::tempdir;

    #[test]
    fn test_vsock_detection() {
        let supported = vsock_supported();
        println!("VSock supported: {}", supported);
    }

    #[test]
    fn test_ssh_detection() {
        let available = ssh_available();
        println!("SSH available: {}", available);
        assert!(available, "SSH should be available on dev machines");
    }

    #[test]
    fn test_forwarding_kind_debug() {
        let kinds = vec![
            ForwardingKind::Vsock { port: 17788 },
            ForwardingKind::SshUds {
                path: PathBuf::from("/tmp/test.sock"),
            },
            ForwardingKind::SshTcp { port: 17788 },
        ];

        for kind in kinds {
            println!("Forwarding kind: {:?}", kind);
        }
    }

    #[test]
    fn probe_caps_uds_returns_true_for_http_200_response() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("agent.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind listener");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 256];
            let _ = stream.read(&mut buf);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\n\r\n{}")
                .expect("write response");
        });

        assert!(probe_caps_uds(&socket_path));
        server.join().expect("join server");
    }

    #[test]
    fn probe_caps_uds_returns_false_for_http_error_response() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("agent.sock");
        let listener = UnixListener::bind(&socket_path).expect("bind listener");

        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("accept");
            let mut buf = [0u8; 256];
            let _ = stream.read(&mut buf);
            stream
                .write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n")
                .expect("write response");
        });

        assert!(!probe_caps_uds(&socket_path));
        server.join().expect("join server");
    }

    #[test]
    fn probe_caps_uds_returns_false_when_socket_missing() {
        let dir = tempdir().expect("tempdir");
        let socket_path = dir.path().join("missing.sock");
        assert!(!probe_caps_uds(&socket_path));
        assert!(!fs::exists(&socket_path).expect("exists check"));
    }

    #[test]
    fn ssh_uds_forwarding_requires_capabilities_probe() {
        let temp = tempdir().expect("tempdir");
        let home = temp.path().join("home");
        let bin = temp.path().join("bin");
        fs::create_dir_all(&bin).expect("bin dir");
        fs::create_dir_all(home.join(".lima/substrate")).expect("ssh config dir");
        fs::write(
            home.join(".lima/substrate/ssh.config"),
            "Host lima-substrate\n  User stub\n",
        )
        .expect("write ssh.config");

        // Stub ssh: create the forwarded socket file immediately.
        let ssh_stub = bin.join("ssh");
        fs::write(
            &ssh_stub,
            r#"#!/usr/bin/env bash
set -euo pipefail
mkdir -p "$HOME/.substrate/sock"
touch "$HOME/.substrate/sock/agent.sock"
exit 0
"#,
        )
        .expect("write ssh stub");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = fs::metadata(&ssh_stub)
                .expect("ssh stub metadata")
                .permissions();
            perms.set_mode(0o755);
            fs::set_permissions(&ssh_stub, perms).expect("set ssh stub perms");
        }

        let prev_home = env::var_os("HOME");
        let prev_lima_home = env::var_os("LIMA_HOME");
        let prev_path = env::var_os("PATH");
        env::set_var("HOME", &home);
        env::set_var("LIMA_HOME", home.join(".lima"));
        let new_path = match prev_path.as_ref() {
            Some(p) => format!("{}:{}", bin.display(), p.to_string_lossy()),
            None => bin.display().to_string(),
        };
        env::set_var("PATH", &new_path);

        match create_ssh_uds_forwarding("substrate") {
            Ok(_) => panic!("ssh uds forwarding should fail without a healthy capabilities probe"),
            Err(err) => assert!(
                err.to_string().contains("SSH UDS forwarding failed"),
                "unexpected error: {err:#}"
            ),
        }

        match prev_home {
            Some(v) => env::set_var("HOME", v),
            None => env::remove_var("HOME"),
        }
        match prev_lima_home {
            Some(v) => env::set_var("LIMA_HOME", v),
            None => env::remove_var("LIMA_HOME"),
        }
        match prev_path {
            Some(v) => env::set_var("PATH", v),
            None => env::remove_var("PATH"),
        }
    }

    #[test]
    fn lima_ssh_config_path_prefers_lima_home_override() {
        let temp = tempdir().expect("tempdir");
        let override_home = temp.path().join("lima-home");
        fs::create_dir_all(override_home.join("substrate")).expect("lima home dir");
        fs::write(
            override_home.join("substrate/ssh.config"),
            "Host lima-substrate\n  User stub\n",
        )
        .expect("write ssh config");

        let prev_lima_home = env::var_os("LIMA_HOME");
        env::set_var("LIMA_HOME", &override_home);

        let path = lima_ssh_config_path("substrate").expect("ssh config path");
        assert_eq!(path, override_home.join("substrate/ssh.config"));

        match prev_lima_home {
            Some(v) => env::set_var("LIMA_HOME", v),
            None => env::remove_var("LIMA_HOME"),
        }
    }
}
