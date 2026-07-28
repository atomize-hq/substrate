//! Forwarding management for host-VM communication.

use crate::transport::{
    compatibility_tcp_endpoint, managed_host_socket_path, CANONICAL_GUEST_SOCKET_PATH,
    CANONICAL_GUEST_SOCKET_URI, COMPATIBILITY_TCP_HOST, COMPATIBILITY_TCP_PORT,
};
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tracing::{debug, info, warn};
use transport_api_types::{InstallBootstrapContextCarrierV1, PlatformBootstrapMappingV1};

#[cfg(test)]
use crate::transport::managed_host_socket_path_for_mapping;

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

/// Diagnostic compatibility auto-selection for ambient forwarding.
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
        match create_ssh_uds_forwarding(vm_name, None, None, None) {
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
    let port = COMPATIBILITY_TCP_PORT;

    // Start vsock-proxy
    let child = Command::new("vsock-proxy")
        .args([
            "--vm",
            vm_name,
            &port.to_string(),
            CANONICAL_GUEST_SOCKET_URI,
        ])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .context("Failed to start vsock-proxy")?;

    // Wait for proxy to be ready
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Health check
    let url = format!("http://{}:{}/v1/capabilities", COMPATIBILITY_TCP_HOST, port);
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

fn create_ssh_uds_forwarding(
    vm_name: &str,
    socket_path: Option<PathBuf>,
    ssh_config: Option<PathBuf>,
    known_hosts_path: Option<PathBuf>,
) -> Result<ForwardingHandle> {
    let socket_path = socket_path.unwrap_or_else(managed_host_socket_path);
    let socket_dir = socket_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("managed host socket path missing parent"))?
        .to_path_buf();

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

    // Remove old socket if exists
    if socket_path.exists() {
        debug!("Removing old socket file");
        std::fs::remove_file(&socket_path)?;
    }

    let ssh_config = match ssh_config {
        Some(path) => path,
        None => lima_ssh_config_path(vm_name)?,
    };
    debug!("Using SSH config: {}", ssh_config.display());

    let ssh_config_str = ssh_config.to_string_lossy();
    let socket_forward = format!("{}:{}", socket_path.display(), CANONICAL_GUEST_SOCKET_PATH);
    let vm_host = format!("lima-{}", vm_name);
    let known_hosts_path = match known_hosts_path {
        Some(path) => path,
        None => dirs::home_dir()
            .ok_or_else(|| anyhow::anyhow!("No home directory"))?
            .join(".substrate/lima_known_hosts"),
    };

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

    // Timed out waiting for the forwarded socket to become healthy. Terminate the ssh helper before
    // draining stderr so we do not block forever on an otherwise-live tunnel process.
    match child.try_wait() {
        Ok(None) => {
            if let Err(err) = child.kill() {
                warn!("Failed to kill timed out SSH forwarding process: {err}");
            }
            if let Err(err) = child.wait() {
                warn!("Failed to wait on timed out SSH forwarding process: {err}");
            }
        }
        Ok(Some(_)) => {}
        Err(err) => {
            warn!("Failed to poll timed out SSH forwarding process: {err}");
        }
    }

    // Best-effort: capture any stderr now that the child has terminated.
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
    let port = COMPATIBILITY_TCP_PORT;

    let ssh_config = lima_ssh_config_path(vm_name)?;
    let ssh_config_str = ssh_config.to_string_lossy();
    let port_forward = format!(
        "{}:{}:{}",
        COMPATIBILITY_TCP_HOST, port, CANONICAL_GUEST_SOCKET_PATH
    );
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
    if let Err(e) = std::net::TcpStream::connect(compatibility_tcp_endpoint()) {
        anyhow::bail!("SSH TCP forwarding failed to establish: {}", e);
    }

    Ok(ForwardingHandle {
        kind: ForwardingKind::SshTcp { port },
        child: Some(child),
    })
}

#[allow(dead_code)]
fn lima_home_dir_for_mapping(
    host_carrier: &InstallBootstrapContextCarrierV1,
    mapping: &PlatformBootstrapMappingV1,
) -> Result<PathBuf> {
    let (_, _, _, control_root) = crate::validate_typed_lima_backend_mapping(host_carrier, mapping)
        .context("typed Lima mapping did not validate for future control-root projection")?;
    Ok(control_root)
}

#[allow(dead_code)]
fn lima_ssh_config_path_for_mapping(
    host_carrier: &InstallBootstrapContextCarrierV1,
    mapping: &PlatformBootstrapMappingV1,
) -> Result<PathBuf> {
    let (vm_name, _, _, control_root) =
        crate::validate_typed_lima_backend_mapping(host_carrier, mapping)
            .context("typed Lima mapping did not validate for future SSH-config projection")?;
    Ok(control_root.join(vm_name).join("ssh.config"))
}

#[allow(dead_code)]
fn lima_known_hosts_path_for_mapping(
    host_carrier: &InstallBootstrapContextCarrierV1,
    mapping: &PlatformBootstrapMappingV1,
) -> Result<PathBuf> {
    crate::validate_typed_lima_backend_mapping(host_carrier, mapping)
        .context("typed Lima mapping did not validate for future known-hosts projection")?;
    Ok(PathBuf::from(&host_carrier.context.selected_host_prefix).join("lima_known_hosts"))
}

/// Diagnostic compatibility helper. R2-3M3 proves the future typed control-root projection via
/// `lima_home_dir_for_mapping`, while the live typed path remains gated on R3.
fn lima_home_dir() -> Result<PathBuf> {
    if let Some(path) = std::env::var_os("LIMA_HOME").filter(|value| !value.is_empty()) {
        return Ok(PathBuf::from(path));
    }

    Ok(dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("No home directory"))?
        .join(".lima"))
}

/// Diagnostic compatibility helper. R2-3M3 proves the future typed SSH-config projection via
/// `lima_ssh_config_path_for_mapping`, while the live typed path remains gated on R3.
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
    use transport_api_types::{
        InstallBootstrapContextCarrierV1, InstallBootstrapContextV1, PlatformBootstrapMappingV1,
        PlatformInstanceIdentityV1, PlatformTransportIdentityV1,
    };

    fn test_host_carrier() -> InstallBootstrapContextCarrierV1 {
        let (principal, _) =
            crate::current_unix_principal_and_home().expect("current Unix install principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/opt/substrate", &account, uid).unwrap(),
        )
        .unwrap()
    }

    fn test_lima_mapping() -> PlatformBootstrapMappingV1 {
        let (_, home) =
            crate::current_unix_principal_and_home().expect("current Unix install principal");
        PlatformBootstrapMappingV1::new_lima(
            &test_host_carrier(),
            "substrate",
            "0123456789abcdef0123456789abcdef",
            home.join(".lima").to_str().expect("UTF-8 home path"),
            "/home/substrate/.substrate",
            "substrate",
            1000,
            "/opt/substrate/sock/agent.sock",
            CANONICAL_GUEST_SOCKET_PATH,
        )
        .unwrap()
    }

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
            ForwardingKind::Vsock {
                port: COMPATIBILITY_TCP_PORT,
            },
            ForwardingKind::SshUds {
                path: PathBuf::from("/tmp/test.sock"),
            },
            ForwardingKind::SshTcp {
                port: COMPATIBILITY_TCP_PORT,
            },
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
        let _env_guard = crate::test_util::lock_env();
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

        match create_ssh_uds_forwarding("substrate", None, None, None) {
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
        let _env_guard = crate::test_util::lock_env();
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

    #[test]
    fn typed_mapping_projections_ignore_ambient_environment() {
        let _env_guard = crate::test_util::lock_env();
        let temp = tempdir().expect("tempdir");
        let ambient_home = temp.path().join("ambient-home-b");
        let ambient_lima = temp.path().join("ambient-lima-b");
        let ambient_substrate = temp.path().join("ambient-substrate-b");
        let ambient_bin = temp.path().join("ambient-bin");
        fs::create_dir_all(ambient_home.join(".substrate/sock")).expect("ambient socket dir");
        fs::create_dir_all(ambient_lima.join("substrate")).expect("ambient lima dir");
        fs::create_dir_all(&ambient_bin).expect("ambient bin");
        fs::write(ambient_home.join(".substrate/sock/agent.sock"), "").expect("ambient socket");
        fs::write(
            ambient_lima.join("substrate/ssh.config"),
            "Host lima-substrate\n  User ambient\n",
        )
        .expect("ambient ssh config");
        for tool in ["ssh", "vsock-proxy"] {
            let path = ambient_bin.join(tool);
            fs::write(&path, "#!/bin/sh\nexit 0\n").expect("tool stub");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&path).expect("tool metadata").permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&path, perms).expect("tool perms");
            }
        }

        let prev_home = env::var_os("HOME");
        let prev_lima_home = env::var_os("LIMA_HOME");
        let prev_substrate_home = env::var_os("SUBSTRATE_HOME");
        let prev_path = env::var_os("PATH");
        let prev_cwd = env::current_dir().expect("cwd");
        env::set_var("HOME", &ambient_home);
        env::set_var("LIMA_HOME", &ambient_lima);
        env::set_var("SUBSTRATE_HOME", &ambient_substrate);
        env::set_var("PATH", &ambient_bin);
        env::set_current_dir(temp.path()).expect("set cwd");
        let _ambient_tcp =
            std::net::TcpListener::bind((COMPATIBILITY_TCP_HOST, COMPATIBILITY_TCP_PORT)).ok();

        let host = test_host_carrier();
        let mapping = test_lima_mapping();
        let (_, current_home) =
            crate::current_unix_principal_and_home().expect("current Unix install principal");
        let current_lima_home = current_home.join(".lima");

        assert_eq!(
            managed_host_socket_path_for_mapping(&host, &mapping).unwrap(),
            PathBuf::from("/opt/substrate/sock/agent.sock")
        );
        assert_eq!(
            lima_home_dir_for_mapping(&host, &mapping).unwrap(),
            current_lima_home
        );
        assert_eq!(
            lima_ssh_config_path_for_mapping(&host, &mapping).unwrap(),
            current_home.join(".lima/substrate/ssh.config")
        );
        assert_eq!(
            lima_known_hosts_path_for_mapping(&host, &mapping).unwrap(),
            PathBuf::from("/opt/substrate/lima_known_hosts")
        );
        let PlatformTransportIdentityV1::Lima {
            host_socket,
            guest_socket,
        } = &mapping.realized_transport
        else {
            panic!("expected Lima transport");
        };
        assert_eq!(host_socket, "/opt/substrate/sock/agent.sock");
        assert_eq!(guest_socket, CANONICAL_GUEST_SOCKET_PATH);

        env::set_current_dir(prev_cwd).expect("restore cwd");
        match prev_home {
            Some(v) => env::set_var("HOME", v),
            None => env::remove_var("HOME"),
        }
        match prev_lima_home {
            Some(v) => env::set_var("LIMA_HOME", v),
            None => env::remove_var("LIMA_HOME"),
        }
        match prev_substrate_home {
            Some(v) => env::set_var("SUBSTRATE_HOME", v),
            None => env::remove_var("SUBSTRATE_HOME"),
        }
        match prev_path {
            Some(v) => env::set_var("PATH", v),
            None => env::remove_var("PATH"),
        }
    }

    #[test]
    fn typed_mapping_projection_helpers_reject_mismatches() {
        let host = test_host_carrier();
        let mapping = test_lima_mapping();

        let mut wrong_commitment = mapping.clone();
        wrong_commitment.host_context_commitment = "0".repeat(64);
        assert!(lima_home_dir_for_mapping(&host, &wrong_commitment).is_err());

        let mut wrong_platform = mapping.clone();
        wrong_platform.platform_instance = PlatformInstanceIdentityV1::Wsl {
            distro_name: "Substrate-WSL".to_string(),
            guest_machine_id: "0123456789abcdef0123456789abcdef".to_string(),
        };
        assert!(lima_home_dir_for_mapping(&host, &wrong_platform).is_err());

        let mut wrong_transport = mapping.clone();
        wrong_transport.realized_transport = PlatformTransportIdentityV1::Wsl {
            pipe_path: r"\\.\pipe\substrate-agent".to_string(),
            guest_socket: CANONICAL_GUEST_SOCKET_PATH.to_string(),
        };
        assert!(lima_home_dir_for_mapping(&host, &wrong_transport).is_err());

        let mut wrong_control_root = mapping.clone();
        wrong_control_root.host_platform_control_root = "/Users/other/.lima".to_string();
        assert!(lima_home_dir_for_mapping(&host, &wrong_control_root).is_err());

        let mut wrong_host_socket = mapping.clone();
        wrong_host_socket.realized_transport = PlatformTransportIdentityV1::Lima {
            host_socket: "/opt/other/sock/agent.sock".to_string(),
            guest_socket: CANONICAL_GUEST_SOCKET_PATH.to_string(),
        };
        assert!(lima_home_dir_for_mapping(&host, &wrong_host_socket).is_err());

        let mut wrong_guest_socket = mapping.clone();
        wrong_guest_socket.realized_transport = PlatformTransportIdentityV1::Lima {
            host_socket: "/opt/substrate/sock/agent.sock".to_string(),
            guest_socket: "/tmp/substrate.sock".to_string(),
        };
        assert!(lima_home_dir_for_mapping(&host, &wrong_guest_socket).is_err());

        let (principal, _) =
            crate::current_unix_principal_and_home().expect("current Unix install principal");
        let transport_api_types::PlatformPrincipalV1::Unix { account, uid } = principal else {
            panic!("expected Unix principal");
        };
        let selected_prefix_b = InstallBootstrapContextCarrierV1::from_context(
            InstallBootstrapContextV1::new_unix("/opt/other", &account, uid).unwrap(),
        )
        .unwrap();
        assert!(lima_home_dir_for_mapping(&selected_prefix_b, &mapping).is_err());
        assert!(lima_ssh_config_path_for_mapping(&selected_prefix_b, &mapping).is_err());
        assert!(lima_known_hosts_path_for_mapping(&selected_prefix_b, &mapping).is_err());
    }

    #[test]
    fn r3_lifecycle_blocks_remain_frozen() {
        const SOURCE: &str = include_str!("forwarding.rs");
        const DROP_BLOCK: &str = r#"impl Drop for ForwardingHandle {
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
}"#;
        const PRELAUNCH_UNLINK_BLOCK: &str = r#"    // Remove old socket if exists
    if socket_path.exists() {
        debug!("Removing old socket file");
        std::fs::remove_file(&socket_path)?;
    }"#;
        const STREAMLOCAL_BIND_UNLINK_BLOCK: &str = r#""-o".to_string(),
        "StreamLocalBindUnlink=yes".to_string(),"#;
        const TIMEOUT_BLOCK: &str = r#"    match child.try_wait() {
        Ok(None) => {
            if let Err(err) = child.kill() {
                warn!("Failed to kill timed out SSH forwarding process: {err}");
            }
            if let Err(err) = child.wait() {
                warn!("Failed to wait on timed out SSH forwarding process: {err}");
            }"#;

        assert!(SOURCE.contains(DROP_BLOCK), "drop block changed");
        assert!(
            SOURCE.contains(PRELAUNCH_UNLINK_BLOCK),
            "pre-launch unlink block changed"
        );
        assert!(
            SOURCE.contains(STREAMLOCAL_BIND_UNLINK_BLOCK),
            "StreamLocalBindUnlink block changed"
        );
        assert!(
            SOURCE.contains(TIMEOUT_BLOCK),
            "timeout kill/wait block changed"
        );
    }
}
