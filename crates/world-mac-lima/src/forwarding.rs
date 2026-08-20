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
    SshUds {
        path: PathBuf,
        mapped_attempt: Option<MappedSshUdsAttemptV1>,
    },
    /// SSH TCP forwarding
    SshTcp { port: u16 },
}

/// Handle for an active forwarding process.
pub struct ForwardingHandle {
    kind: ForwardingKind,
    child: Option<Child>,
}

/// Receipt-bound state for the typed SSH-UDS forwarding attempt.
///
/// Compatibility forwarding deliberately has no instance of this type.  The typed path records
/// the exact before-state so Drop can retire only the socket and known-host mutation it created.
#[derive(Debug, Clone)]
pub struct MappedSshUdsAttemptV1 {
    socket_path: PathBuf,
    socket_device: u64,
    socket_inode: u64,
    known_hosts_path: PathBuf,
    known_hosts_existed: bool,
    known_hosts_before: Vec<u8>,
    known_hosts_device: Option<u64>,
    known_hosts_inode: Option<u64>,
    known_hosts_created_device: Option<u64>,
    known_hosts_created_inode: Option<u64>,
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

        let mapped_attempt = match &mut self.kind {
            ForwardingKind::SshUds { mapped_attempt, .. } => mapped_attempt.take(),
            ForwardingKind::Vsock { .. } | ForwardingKind::SshTcp { .. } => None,
        };
        if let Some(attempt) = mapped_attempt {
            let mut child_reaped = true;
            if let Some(mut child) = self.child.take() {
                match child.try_wait() {
                    Ok(Some(_)) => {}
                    Ok(None) => {
                        if let Err(error) = child.kill() {
                            warn!("Failed to kill mapped SSH forwarding process: {error}");
                            child_reaped = false;
                        }
                        if child_reaped {
                            if let Err(error) = child.wait() {
                                warn!("Failed to wait for mapped SSH forwarding process: {error}");
                                child_reaped = false;
                            }
                        }
                    }
                    Err(error) => {
                        warn!("Failed to poll mapped SSH forwarding process: {error}");
                        child_reaped = false;
                    }
                }
            }

            if !child_reaped {
                warn!(
                    "Preserving mapped SSH socket because its child could not be reaped; reconciling known_hosts independently"
                );
            } else if let Err(error) = remove_exact_mapped_ssh_socket_v1(&attempt) {
                warn!(
                    "Preserving replaced mapped SSH socket {}: {error}",
                    attempt.socket_path.display()
                );
            }

            // Socket replacement never bypasses independent A-local known-host rollback.
            if let Err(error) = restore_exact_mapped_known_hosts_entry_v1(&attempt) {
                warn!(
                    "Failed to restore mapped known_hosts {}: {error}",
                    attempt.known_hosts_path.display()
                );
            }
            return;
        }

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
        if let ForwardingKind::SshUds { ref path, .. } = self.kind {
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

#[cfg(unix)]
fn unix_regular_file_identity_v1(path: &Path) -> Result<Option<(u64, u64)>> {
    use std::os::unix::fs::MetadataExt as _;
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_file() => {
            Ok(Some((metadata.dev(), metadata.ino())))
        }
        Ok(_) => anyhow::bail!("mapped lifecycle path is not an exact regular file"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

#[cfg(unix)]
fn mapped_socket_identity_v1(path: &Path) -> Result<Option<(u64, u64)>> {
    use std::os::unix::fs::{FileTypeExt as _, MetadataExt as _};
    match std::fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() => {
            Ok(Some((metadata.dev(), metadata.ino())))
        }
        Ok(_) => anyhow::bail!("mapped SSH forwarding path is not an exact Unix socket"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("inspect {}", path.display())),
    }
}

#[cfg(not(unix))]
fn unix_regular_file_identity_v1(_path: &Path) -> Result<Option<(u64, u64)>> {
    Ok(None)
}

#[cfg(not(unix))]
fn mapped_socket_identity_v1(_path: &Path) -> Result<Option<(u64, u64)>> {
    Ok(None)
}

fn remove_exact_mapped_ssh_socket_v1(attempt: &MappedSshUdsAttemptV1) -> Result<()> {
    let Some((device, inode)) = mapped_socket_identity_v1(&attempt.socket_path)? else {
        return Ok(());
    };
    if device != attempt.socket_device || inode != attempt.socket_inode {
        anyhow::bail!("mapped SSH socket identity changed")
    }
    std::fs::remove_file(&attempt.socket_path).with_context(|| {
        format!(
            "remove exact mapped SSH socket {}",
            attempt.socket_path.display()
        )
    })
}

fn restore_exact_mapped_known_hosts_entry_v1(attempt: &MappedSshUdsAttemptV1) -> Result<()> {
    let current = unix_regular_file_identity_v1(&attempt.known_hosts_path)?;
    if attempt.known_hosts_existed {
        if current != attempt.known_hosts_device.zip(attempt.known_hosts_inode) {
            anyhow::bail!("mapped known_hosts identity changed")
        }
        return std::fs::write(&attempt.known_hosts_path, &attempt.known_hosts_before)
            .with_context(|| {
                format!(
                    "restore mapped known_hosts {}",
                    attempt.known_hosts_path.display()
                )
            });
    }
    match (
        current,
        attempt
            .known_hosts_created_device
            .zip(attempt.known_hosts_created_inode),
    ) {
        (None, _) => Ok(()),
        (Some(identity), Some(created_identity)) if identity == created_identity => {
            std::fs::remove_file(&attempt.known_hosts_path).with_context(|| {
                format!(
                    "remove exact created mapped known_hosts {}",
                    attempt.known_hosts_path.display()
                )
            })
        }
        (Some(_), _) => anyhow::bail!(
            "mapped known_hosts was not captured as an exact child-created regular file"
        ),
    }
}

fn mapped_ssh_attempt_v1(
    socket_path: PathBuf,
    socket_identity: Option<(u64, u64)>,
    known_hosts_path: PathBuf,
    known_hosts_existed: bool,
    known_hosts_before: Vec<u8>,
    known_hosts_identity: Option<(u64, u64)>,
    known_hosts_created_identity: Option<(u64, u64)>,
) -> MappedSshUdsAttemptV1 {
    MappedSshUdsAttemptV1 {
        socket_path,
        socket_device: socket_identity.map_or(0, |(device, _)| device),
        socket_inode: socket_identity.map_or(0, |(_, inode)| inode),
        known_hosts_path,
        known_hosts_existed,
        known_hosts_before,
        known_hosts_device: known_hosts_identity.map(|(device, _)| device),
        known_hosts_inode: known_hosts_identity.map(|(_, inode)| inode),
        known_hosts_created_device: known_hosts_created_identity.map(|(device, _)| device),
        known_hosts_created_inode: known_hosts_created_identity.map(|(_, inode)| inode),
    }
}

/// Retire a failed mapped attempt before returning its error.
///
/// Both state projections are reconciled independently: a replaced socket must not suppress
/// known-host rollback, and an inability to reap the child preserves only the socket it might
/// still own. Ownership identities are captured only while the child is observed live, so a new
/// pathname occupant can never be adopted or unlinked after an early-exit race.
fn cleanup_failed_mapped_ssh_attempt_v1(
    child: Option<Child>,
    socket_path: PathBuf,
    owned_socket_identity: Option<(u64, u64)>,
    known_hosts_path: PathBuf,
    known_hosts_existed: bool,
    known_hosts_before: Vec<u8>,
    known_hosts_identity: Option<(u64, u64)>,
    known_hosts_created_identity: Option<(u64, u64)>,
) -> Result<()> {
    let child_reaped = match child {
        None => true,
        Some(mut child) => match child
            .try_wait()
            .context("poll failed mapped SSH forwarding")?
        {
            Some(_) => true,
            None => match child.kill() {
                Ok(()) => child
                    .wait()
                    .context("wait for failed mapped SSH forwarding")
                    .is_ok(),
                Err(error) => {
                    warn!("failed to kill mapped SSH forwarding during cleanup: {error}");
                    false
                }
            },
        },
    };

    let socket_result = if child_reaped {
        match owned_socket_identity {
            Some(identity) => remove_exact_mapped_ssh_socket_v1(&mapped_ssh_attempt_v1(
                socket_path.clone(),
                Some(identity),
                known_hosts_path.clone(),
                known_hosts_existed,
                known_hosts_before.clone(),
                known_hosts_identity,
                known_hosts_created_identity,
            )),
            None => {
                // An early exit before a live child observation has no ownership proof. Preserve
                // any pathname occupant rather than converting a replacement race into unlink.
                Ok(())
            }
        }
    } else {
        warn!(
            "preserving mapped SSH socket because failed child could not be reaped; reconciling known_hosts independently"
        );
        Ok(())
    };
    let known_hosts_result = restore_exact_mapped_known_hosts_entry_v1(&mapped_ssh_attempt_v1(
        socket_path,
        None,
        known_hosts_path,
        known_hosts_existed,
        known_hosts_before,
        known_hosts_identity,
        known_hosts_created_identity,
    ));

    match (socket_result, known_hosts_result) {
        (Ok(()), Ok(())) => Ok(()),
        (Err(socket_error), Ok(())) => {
            Err(socket_error).context("failed mapped SSH socket cleanup")
        }
        (Ok(()), Err(known_hosts_error)) => {
            Err(known_hosts_error).context("failed mapped known_hosts cleanup")
        }
        (Err(socket_error), Err(known_hosts_error)) => Err(anyhow::anyhow!(
            "failed mapped SSH cleanup: socket={socket_error:#}; known_hosts={known_hosts_error:#}"
        )),
    }
}

/// Record the exact A-local known-hosts before-state before `accept-new` can mutate it.
pub fn record_mapped_known_hosts_entry_v1(path: &Path) -> Result<(bool, Vec<u8>)> {
    match unix_regular_file_identity_v1(path)? {
        Some(_) => std::fs::read(path)
            .map(|bytes| (true, bytes))
            .with_context(|| format!("read mapped known_hosts {}", path.display())),
        None => Ok((false, Vec::new())),
    }
}

/// Restore the recorded A-local known-hosts before-state without touching any global SSH file.
pub fn restore_mapped_known_hosts_entry_v1(path: &Path, existed: bool, bytes: &[u8]) -> Result<()> {
    if existed {
        std::fs::write(path, bytes)
            .with_context(|| format!("restore mapped known_hosts {}", path.display()))?;
    } else {
        match std::fs::remove_file(path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| {
                    format!("remove created mapped known_hosts {}", path.display())
                });
            }
        }
    }
    Ok(())
}

/// Create the only typed-product forwarding path: the already-selected PM SSH-UDS mapping.
///
/// The function accepts projections that were validated by `new_with_mapping`; it never consults
/// `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, auto-select, VSock, or TCP fallback state.
pub fn create_mapped_ssh_uds_forwarding_v1(
    vm_name: &str,
    socket_path: PathBuf,
    host_account_home: &Path,
    lima_control_root: &Path,
) -> Result<ForwardingHandle> {
    let expected_control_root = host_account_home.join(".lima");
    if lima_control_root != expected_control_root {
        anyhow::bail!("mapped Lima control root does not match the validated host account home");
    }
    let socket_parent = socket_path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("mapped host socket has no parent"))?;
    let socket_parent_metadata = std::fs::symlink_metadata(socket_parent)
        .with_context(|| format!("inspect mapped socket parent {}", socket_parent.display()))?;
    if !socket_parent_metadata.file_type().is_dir() {
        anyhow::bail!("mapped host socket parent is not a directory");
    }
    match std::fs::symlink_metadata(&socket_path) {
        Ok(_) => anyhow::bail!("mapped host socket must be absent before a typed SSH-UDS attempt"),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => {
            return Err(error)
                .with_context(|| format!("inspect mapped host socket {}", socket_path.display()))
        }
    }

    let ssh_config = lima_control_root.join(vm_name).join("ssh.config");
    if !ssh_config.is_file() {
        anyhow::bail!("mapped Lima SSH config is not an exact regular file");
    }
    let selected_prefix = socket_parent
        .parent()
        .ok_or_else(|| anyhow::anyhow!("mapped host socket parent has no selected prefix"))?;
    let known_hosts_path = selected_prefix.join("lima_known_hosts");
    let (known_hosts_existed, known_hosts_before) =
        record_mapped_known_hosts_entry_v1(&known_hosts_path)?;
    let known_hosts_identity = unix_regular_file_identity_v1(&known_hosts_path)?;

    let ssh_args = vec![
        "-F".to_string(),
        ssh_config.to_string_lossy().to_string(),
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
        format!("{}:{}", socket_path.display(), CANONICAL_GUEST_SOCKET_PATH),
        format!("lima-{vm_name}"),
        "-N".to_string(),
    ];
    let mut command = Command::new("ssh");
    command
        .args(&ssh_args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt as _;
        command.process_group(0);
    }
    let mut child = match command.spawn() {
        Ok(child) => child,
        Err(error) => {
            let cleanup = restore_exact_mapped_known_hosts_entry_v1(&mapped_ssh_attempt_v1(
                socket_path.clone(),
                None,
                known_hosts_path.clone(),
                known_hosts_existed,
                known_hosts_before.clone(),
                known_hosts_identity,
                None,
            ));
            return match cleanup {
                Ok(()) => Err(error).context("start mapped SSH UDS forwarding"),
                Err(cleanup_error) => Err(anyhow::anyhow!(
                    "start mapped SSH UDS forwarding failed: {error}; known_hosts rollback failed: {cleanup_error:#}"
                )),
            };
        }
    };
    let mut stderr = String::new();
    // Identities become owned only when observed while the SSH child is still live. This keeps an
    // early-exit/replacement race fail-closed: cleanup never adopts a pathname at exit time.
    let mut owned_socket_identity = None;
    let mut created_known_hosts_identity = None;
    for _ in 0..20 {
        if let Some(status) = child.try_wait().context("poll mapped SSH forwarding")? {
            if let Some(mut output) = child.stderr.take() {
                use std::io::Read as _;
                let _ = output.read_to_string(&mut stderr);
            }
            let cleanup = cleanup_failed_mapped_ssh_attempt_v1(
                None,
                socket_path.clone(),
                owned_socket_identity,
                known_hosts_path.clone(),
                known_hosts_existed,
                known_hosts_before.clone(),
                known_hosts_identity,
                created_known_hosts_identity,
            );
            return match cleanup {
                Ok(()) => Err(anyhow::anyhow!(
                    "mapped SSH UDS forwarding exited {status}: {}",
                    stderr.trim()
                )),
                Err(cleanup_error) => Err(anyhow::anyhow!(
                    "mapped SSH UDS forwarding exited {status}: {}; cleanup failed: {cleanup_error:#}",
                    stderr.trim()
                )),
            };
        }

        if owned_socket_identity.is_none() {
            match mapped_socket_identity_v1(&socket_path) {
                Ok(identity) => owned_socket_identity = identity,
                Err(error) => {
                    let _ = cleanup_failed_mapped_ssh_attempt_v1(
                        Some(child),
                        socket_path.clone(),
                        owned_socket_identity,
                        known_hosts_path.clone(),
                        known_hosts_existed,
                        known_hosts_before.clone(),
                        known_hosts_identity,
                        created_known_hosts_identity,
                    );
                    return Err(error)
                        .context("observe exact mapped SSH socket while child is live");
                }
            }
        }
        if !known_hosts_existed && created_known_hosts_identity.is_none() {
            match unix_regular_file_identity_v1(&known_hosts_path) {
                Ok(identity) => created_known_hosts_identity = identity,
                Err(error) => {
                    let _ = cleanup_failed_mapped_ssh_attempt_v1(
                        Some(child),
                        socket_path.clone(),
                        owned_socket_identity,
                        known_hosts_path.clone(),
                        known_hosts_existed,
                        known_hosts_before.clone(),
                        known_hosts_identity,
                        created_known_hosts_identity,
                    );
                    return Err(error)
                        .context("observe exact mapped known_hosts while child is live");
                }
            }
        }

        if probe_caps_uds(&socket_path) {
            let socket_identity = match mapped_socket_identity_v1(&socket_path)? {
                Some(identity) => identity,
                None => {
                    let _ = cleanup_failed_mapped_ssh_attempt_v1(
                        Some(child),
                        socket_path.clone(),
                        owned_socket_identity,
                        known_hosts_path.clone(),
                        known_hosts_existed,
                        known_hosts_before.clone(),
                        known_hosts_identity,
                        created_known_hosts_identity,
                    );
                    anyhow::bail!("mapped SSH forwarding health probe had no exact Unix socket");
                }
            };
            if owned_socket_identity != Some(socket_identity) {
                let _ = cleanup_failed_mapped_ssh_attempt_v1(
                    Some(child),
                    socket_path.clone(),
                    owned_socket_identity,
                    known_hosts_path.clone(),
                    known_hosts_existed,
                    known_hosts_before.clone(),
                    known_hosts_identity,
                    created_known_hosts_identity,
                );
                anyhow::bail!(
                    "mapped SSH forwarding socket identity changed before health acceptance"
                );
            }
            return Ok(ForwardingHandle {
                kind: ForwardingKind::SshUds {
                    path: socket_path.clone(),
                    mapped_attempt: Some(mapped_ssh_attempt_v1(
                        socket_path,
                        owned_socket_identity,
                        known_hosts_path,
                        known_hosts_existed,
                        known_hosts_before,
                        known_hosts_identity,
                        created_known_hosts_identity,
                    )),
                },
                child: Some(child),
            });
        }
        std::thread::sleep(Duration::from_millis(500));
    }

    let cleanup = cleanup_failed_mapped_ssh_attempt_v1(
        Some(child),
        socket_path,
        owned_socket_identity,
        known_hosts_path,
        known_hosts_existed,
        known_hosts_before,
        known_hosts_identity,
        created_known_hosts_identity,
    );
    match cleanup {
        Ok(()) => anyhow::bail!(
            "mapped SSH UDS forwarding timed out before a healthy exact socket was observed"
        ),
        Err(cleanup_error) => anyhow::bail!(
            "mapped SSH UDS forwarding timed out before a healthy exact socket was observed; cleanup failed: {cleanup_error:#}"
        ),
    }
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
                kind: ForwardingKind::SshUds {
                    path: socket_path,
                    mapped_attempt: None,
                },
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
                mapped_attempt: None,
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
    fn r3_mapped_ssh_uds_lifecycle_is_exact_and_compatibility_is_preserved() {
        const SOURCE: &str = include_str!("forwarding.rs");
        let mapped_start = SOURCE
            .find("pub fn create_mapped_ssh_uds_forwarding_v1(")
            .expect("mapped SSH-UDS constructor");
        let mapped_end = SOURCE[mapped_start..]
            .find("\nfn create_ssh_uds_forwarding(")
            .map(|offset| mapped_start + offset)
            .expect("mapped SSH-UDS constructor end");
        let mapped = &SOURCE[mapped_start..mapped_end];
        assert!(mapped.contains("StreamLocalBindUnlink=yes"));
        assert!(mapped.contains("ConnectTimeout=5"));
        assert!(mapped.contains("cleanup_failed_mapped_ssh_attempt_v1"));
        assert!(mapped.contains("record_mapped_known_hosts_entry_v1"));
        assert!(mapped.contains("restore_exact_mapped_known_hosts_entry_v1"));
        assert!(!mapped.contains("auto_select("));
        assert!(!mapped.contains("create_ssh_tcp_forwarding("));
        assert!(!mapped.contains("create_vsock_forwarding("));

        let cleanup_start = SOURCE
            .find("fn cleanup_failed_mapped_ssh_attempt_v1(")
            .expect("mapped SSH cleanup helper");
        let cleanup_end = SOURCE[cleanup_start..]
            .find("\n/// Record the exact A-local")
            .map(|offset| cleanup_start + offset)
            .expect("mapped SSH cleanup helper end");
        let cleanup = &SOURCE[cleanup_start..cleanup_end];
        assert!(cleanup.contains("child.kill()"));
        assert!(cleanup.contains(".wait()"));
        assert!(cleanup.contains("owned_socket_identity"));
        assert!(cleanup.contains("remove_exact_mapped_ssh_socket_v1"));
        assert!(cleanup.contains("restore_exact_mapped_known_hosts_entry_v1"));
        assert!(mapped.contains("created_known_hosts_identity"));

        let drop_start = SOURCE
            .find("impl Drop for ForwardingHandle {")
            .expect("ForwardingHandle drop");
        let drop_end = SOURCE[drop_start..]
            .find("\nfn probe_caps_uds(")
            .map(|offset| drop_start + offset)
            .expect("ForwardingHandle drop end");
        let drop_body = &SOURCE[drop_start..drop_end];
        assert!(drop_body.contains("mapped_attempt.take()"));
        assert!(drop_body.contains("remove_exact_mapped_ssh_socket_v1"));
        assert!(drop_body.contains("Preserving replaced mapped SSH socket"));
        assert!(drop_body.contains("restore_exact_mapped_known_hosts_entry_v1"));
        assert!(drop_body.contains("reconciling known_hosts independently"));
    }

    #[test]
    fn mapped_ssh_uds_early_exit_restores_exact_state_and_retries() {
        let _env_guard = crate::test_util::lock_env();
        let temp = tempdir().expect("tempdir");
        let host_home = temp.path().join("host-home");
        let control_root = host_home.join(".lima");
        let selected_prefix = temp.path().join("selected-prefix");
        let socket_path = selected_prefix.join("sock/agent.sock");
        let bin = temp.path().join("bin");
        fs::create_dir_all(socket_path.parent().expect("socket parent")).expect("socket parent");
        fs::create_dir_all(control_root.join("substrate")).expect("control root");
        fs::create_dir_all(&bin).expect("fake ssh bin");
        fs::write(
            control_root.join("substrate/ssh.config"),
            "Host lima-substrate\n  User fixture\n",
        )
        .expect("ssh config");
        let known_hosts = selected_prefix.join("lima_known_hosts");
        fs::write(&known_hosts, b"before\n").expect("seed known hosts");

        let ssh = bin.join("ssh");
        fs::write(
            &ssh,
            r##"#!/usr/bin/env bash
set -euo pipefail
known_hosts=""
socket_path=""
previous=""
for arg in "$@"; do
  case "$arg" in
    UserKnownHostsFile=*) known_hosts="${arg#UserKnownHostsFile=}" ;;
  esac
  if [[ "$previous" == "-L" ]]; then
    socket_path="${arg%:/run/substrate.sock}"
  fi
  previous="$arg"
done
mkdir -p "$(dirname "$known_hosts")"
printf 'mutated\n' >"$known_hosts"
python3 - "$socket_path" <<'EOF_FAKEPY'
import socket
import sys
listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
listener.bind(sys.argv[1])
listener.close()
EOF_FAKEPY
# Keep the child live long enough for the constructor to capture the identities it owns.
sleep 1
printf 'fixture ssh failed after state mutation\n' >&2
exit 23
"##,
        )
        .expect("fake ssh");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut permissions = fs::metadata(&ssh).expect("fake ssh metadata").permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&ssh, permissions).expect("fake ssh permissions");
        }
        let previous_path = env::var_os("PATH");
        let path = previous_path.as_ref().map_or_else(
            || bin.display().to_string(),
            |previous| format!("{}:{}", bin.display(), previous.to_string_lossy()),
        );
        env::set_var("PATH", path);

        for attempt in 0..2 {
            let error = match create_mapped_ssh_uds_forwarding_v1(
                "substrate",
                socket_path.clone(),
                &host_home,
                &control_root,
            ) {
                Ok(_) => panic!("fake ssh must fail after its socket and known-host mutation"),
                Err(error) => error,
            };
            assert!(
                error.to_string().contains("exited"),
                "unexpected attempt {attempt} error: {error:#}"
            );
            assert!(
                !socket_path.exists(),
                "failed attempt {attempt} retained its socket"
            );
            assert_eq!(
                fs::read(&known_hosts).expect("known hosts after failed attempt"),
                b"before\n"
            );
        }

        match previous_path {
            Some(value) => env::set_var("PATH", value),
            None => env::remove_var("PATH"),
        }
    }

    #[test]
    fn mapped_cleanup_preserves_replaced_and_symlinked_paths() {
        use std::os::unix::fs::{symlink, FileTypeExt as _, MetadataExt as _};

        let temp = tempdir().expect("tempdir");
        let socket_path = temp.path().join("agent.sock");
        let original_socket = UnixListener::bind(&socket_path).expect("original socket");
        let original_metadata = fs::symlink_metadata(&socket_path).expect("socket metadata");
        let known_hosts = temp.path().join("lima_known_hosts");
        fs::write(&known_hosts, b"before\n").expect("known hosts");
        let known_metadata = fs::symlink_metadata(&known_hosts).expect("known-host metadata");
        let attempt = mapped_ssh_attempt_v1(
            socket_path.clone(),
            Some((original_metadata.dev(), original_metadata.ino())),
            known_hosts.clone(),
            true,
            b"before\n".to_vec(),
            Some((known_metadata.dev(), known_metadata.ino())),
            None,
        );

        fs::remove_file(&socket_path).expect("unlink original socket pathname");
        let replacement_socket = UnixListener::bind(&socket_path).expect("replacement socket");
        let replacement = temp.path().join("replacement-known-hosts");
        fs::write(&replacement, b"replacement\n").expect("replacement known hosts");
        fs::rename(&replacement, &known_hosts).expect("replace known hosts atomically");

        assert!(remove_exact_mapped_ssh_socket_v1(&attempt).is_err());
        assert!(fs::symlink_metadata(&socket_path)
            .expect("replacement socket remains")
            .file_type()
            .is_socket());
        assert!(restore_exact_mapped_known_hosts_entry_v1(&attempt).is_err());
        assert_eq!(fs::read(&known_hosts).unwrap(), b"replacement\n");

        drop(replacement_socket);
        drop(original_socket);
        fs::remove_file(&socket_path).expect("remove replacement socket");
        let target = temp.path().join("external-target");
        fs::write(&target, b"external\n").expect("external target");
        fs::remove_file(&known_hosts).expect("remove replacement known hosts");
        symlink(&target, &known_hosts).expect("symlink known hosts");
        assert!(record_mapped_known_hosts_entry_v1(&known_hosts).is_err());
        assert!(restore_exact_mapped_known_hosts_entry_v1(&attempt).is_err());
        assert!(fs::symlink_metadata(&known_hosts)
            .expect("symlink remains")
            .file_type()
            .is_symlink());
    }

    #[test]
    fn mapped_cleanup_preserves_unobserved_early_exit_paths() {
        let temp = tempdir().expect("tempdir");
        let socket_path = temp.path().join("agent.sock");
        let listener =
            UnixListener::bind(&socket_path).expect("socket created after an unobserved exit");
        let known_hosts = temp.path().join("lima_known_hosts");
        fs::write(&known_hosts, b"unobserved-created\n").expect("created known hosts");

        let cleanup = cleanup_failed_mapped_ssh_attempt_v1(
            None,
            socket_path.clone(),
            None,
            known_hosts.clone(),
            false,
            Vec::new(),
            None,
            None,
        );
        assert!(
            cleanup.is_err(),
            "unobserved known_hosts must not be adopted for unlink"
        );
        assert!(socket_path.exists(), "unobserved socket must be preserved");
        assert_eq!(fs::read(&known_hosts).unwrap(), b"unobserved-created\n");

        drop(listener);
        fs::remove_file(&socket_path).expect("manual safe socket retirement");
    }

    #[test]
    fn mapped_known_hosts_before_state_round_trips() {
        let tempdir = tempfile::tempdir().expect("tempdir");
        let existing = tempdir.path().join("existing_known_hosts");
        std::fs::write(&existing, b"before\n").expect("seed existing known_hosts");
        let (existed, before) =
            record_mapped_known_hosts_entry_v1(&existing).expect("record existing");
        std::fs::write(&existing, b"after\n").expect("replace existing known_hosts");
        restore_mapped_known_hosts_entry_v1(&existing, existed, &before).expect("restore existing");
        assert_eq!(std::fs::read(&existing).unwrap(), b"before\n");

        let absent = tempdir.path().join("absent_known_hosts");
        let (existed, before) = record_mapped_known_hosts_entry_v1(&absent).expect("record absent");
        std::fs::write(&absent, b"created\n").expect("create known_hosts");
        restore_mapped_known_hosts_entry_v1(&absent, existed, &before).expect("restore absent");
        assert!(!absent.exists());
    }
}
