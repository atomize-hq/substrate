use anyhow::{bail, Context, Result};
use libc;
use serde_json::Value;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::UnixDatagram;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use substrate_common::{
    GuestPublisherPairingTicketV1, ManagedLifecyclePublisherRequestV1,
    PublisherBootstrapAuthorizationV1,
};

use super::provider_unavailable_error;

const DEFAULT_EXECUTOR_PATH_V1: &str = "/usr/libexec/substrate/substrate-lifecycle-linux";
const DEFAULT_ENDPOINT_PATH_V1: &str = "/run/substrate-lifecycle-publisher-v1.sock";
const FIXED_SUDO_PATH_V1: &str = "/usr/bin/sudo";
const FIXED_ENV_PATH_V1: &str = "/usr/bin/env";
const PRIVILEGED_PATH_V1: &str = "/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin";
const EXECUTOR_CHANNEL_FD_V1: &str = "1";
const MAX_FRAME_BYTES_V1: usize = 1024 * 1024;
const SINGLE_FRAME_FINISH_TIMEOUT_V1: Duration = Duration::from_secs(1);

pub fn bootstrap_publisher_v1(authorization: &PublisherBootstrapAuthorizationV1) -> Result<Value> {
    if !linux_provider_installed_v1() {
        return Err(provider_unavailable_error(
            "linux",
            "bootstrap_publisher_v1",
        ));
    }
    let request_bytes =
        serde_json::to_vec(authorization).context("encode bootstrap authorization")?;
    let response_bytes = run_linux_executor_frame_v1(
        &[
            "bootstrap-publisher",
            "--publisher-bootstrap-fd",
            EXECUTOR_CHANNEL_FD_V1,
        ],
        &request_bytes,
    )
    .context("dispatch linux publisher bootstrap")?;
    serde_json::from_slice(&response_bytes).context("decode bootstrap response")
}

pub fn submit_publisher_request_v1(request: &ManagedLifecyclePublisherRequestV1) -> Result<Value> {
    if !linux_provider_installed_v1() {
        return Err(provider_unavailable_error(
            "linux",
            "submit_publisher_request_v1",
        ));
    }
    let request_bytes = serde_json::to_vec(request).context("encode publisher request")?;
    let response_bytes = run_linux_executor_frame_v1(
        &["relay-request", "--relay-fd", EXECUTOR_CHANNEL_FD_V1],
        &request_bytes,
    )
    .context("dispatch linux publisher relay")?;
    attest_linux_publisher_response_v1(&response_bytes)?;
    serde_json::from_slice(&response_bytes).context("decode publisher response")
}

pub fn issue_guest_publisher_pairing_ticket_v1(
    _request: &ManagedLifecyclePublisherRequestV1,
) -> Result<GuestPublisherPairingTicketV1> {
    Err(provider_unavailable_error(
        "linux",
        "issue_guest_publisher_pairing_ticket_v1",
    ))
}

pub fn open_linux_seqpacket_channel_v1() -> Result<(UnixDatagram, OwnedFd)> {
    let mut fds = [-1_i32; 2];
    // SAFETY: libc::socketpair is called with fixed AF_UNIX/SOCK_SEQPACKET flags and a
    // two-element output array.
    let rc = unsafe {
        libc::socketpair(
            libc::AF_UNIX,
            libc::SOCK_SEQPACKET | libc::SOCK_CLOEXEC,
            0,
            fds.as_mut_ptr(),
        )
    };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("open SOCK_SEQPACKET socketpair");
    }
    // SAFETY: both descriptors are freshly returned by socketpair.
    let left = unsafe { OwnedFd::from_raw_fd(fds[0]) };
    // SAFETY: both descriptors are freshly returned by socketpair.
    let right = unsafe { OwnedFd::from_raw_fd(fds[1]) };
    let parent = owned_fd_to_unix_datagram_v1(left)?;
    Ok((parent, right))
}

pub fn attest_linux_publisher_response_v1(response_bytes: &[u8]) -> Result<()> {
    let response: Value =
        serde_json::from_slice(response_bytes).context("decode linux publisher response")?;
    let attestation = response
        .get("relay_attestation")
        .and_then(Value::as_object)
        .ok_or_else(|| anyhow::anyhow!("linux publisher response is missing relay_attestation"))?;
    let caller_pid = attestation
        .get("caller_pid")
        .and_then(Value::as_u64)
        .ok_or_else(|| anyhow::anyhow!("linux publisher response is missing caller_pid"))?;
    if caller_pid != u64::from(std::process::id()) {
        bail!("linux publisher relay caller_pid does not match the current process");
    }

    let caller_exe = attestation
        .get("caller_exe")
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("linux publisher response is missing caller_exe"))?;
    let current_exe = std::env::current_exe()
        .context("resolve current executable for linux publisher attestation")?;
    let current_exe = canonicalize_best_effort_v1(&current_exe);
    let remote_exe = canonicalize_best_effort_v1(Path::new(caller_exe));
    if remote_exe != current_exe {
        bail!("linux publisher relay caller_exe does not match the current executable");
    }
    Ok(())
}

fn linux_provider_installed_v1() -> bool {
    let executor = Path::new(DEFAULT_EXECUTOR_PATH_V1);
    let endpoint_parent = Path::new(DEFAULT_ENDPOINT_PATH_V1)
        .parent()
        .map(Path::exists)
        .unwrap_or(false);
    executor.is_file() && endpoint_parent
}

fn run_linux_executor_frame_v1(args: &[&str], request_bytes: &[u8]) -> Result<Vec<u8>> {
    run_linux_executor_frame_with_paths_v1(
        Path::new(FIXED_SUDO_PATH_V1),
        Path::new(FIXED_ENV_PATH_V1),
        Path::new(DEFAULT_EXECUTOR_PATH_V1),
        args,
        request_bytes,
    )
}

fn run_linux_executor_frame_with_paths_v1(
    sudo_path: &Path,
    env_path: &Path,
    executor_path: &Path,
    args: &[&str],
    request_bytes: &[u8],
) -> Result<Vec<u8>> {
    let (parent_socket, child_fd) = open_linux_seqpacket_channel_v1()?;

    let mut command = Command::new(sudo_path);
    // Route the framed channel through stdout because sudo always preserves stdio, while
    // -C/--close-from depends on closefrom_override in sudoers.
    command
        .arg("--")
        .arg(env_path)
        .arg("-i")
        .arg(format!("PATH={PRIVILEGED_PATH_V1}"))
        .arg("HOME=/root")
        .arg("USER=root")
        .arg("LOGNAME=root")
        .arg(executor_path)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::from(child_fd))
        .stderr(Stdio::piped());

    let child = command.spawn().context("spawn fixed sudo linux executor")?;

    send_seqpacket_frame_v1(&parent_socket, request_bytes)
        .context("write request to linux executor channel")?;
    let response_bytes = read_seqpacket_frame_v1(&parent_socket)
        .context("read response from linux executor channel")?;

    let output = child
        .wait_with_output()
        .context("wait for fixed sudo linux executor")?;
    if !output.status.success() {
        bail!(
            "linux executor failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    if response_bytes.is_empty() {
        bail!("linux executor did not return a response");
    }
    Ok(response_bytes)
}

fn owned_fd_to_unix_datagram_v1(fd: OwnedFd) -> Result<UnixDatagram> {
    Ok(UnixDatagram::from(fd))
}

fn read_seqpacket_frame_v1(socket: &UnixDatagram) -> Result<Vec<u8>> {
    let mut bytes = vec![0u8; MAX_FRAME_BYTES_V1 + 1];
    let read_len = socket.recv(&mut bytes).context("read seqpacket frame")?;
    if read_len == 0 {
        bail!("frame was empty");
    }
    if read_len > MAX_FRAME_BYTES_V1 {
        bail!("frame exceeded the 1 MiB packet contract");
    }
    bytes.truncate(read_len);
    let prior_timeout = socket.read_timeout().context("read seqpacket timeout")?;
    socket
        .set_read_timeout(Some(SINGLE_FRAME_FINISH_TIMEOUT_V1))
        .context("set seqpacket finish timeout")?;
    let mut probe = [0u8; 1];
    let trailer = socket.recv(&mut probe);
    socket
        .set_read_timeout(prior_timeout)
        .context("restore seqpacket timeout")?;
    match trailer {
        Ok(0) => {}
        Ok(_) => bail!("seqpacket channel carried more than one frame"),
        Err(error)
            if matches!(
                error.kind(),
                std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
            ) =>
        {
            bail!("seqpacket peer did not terminate the single-frame exchange");
        }
        Err(error) => return Err(error).context("verify seqpacket frame termination"),
    }
    Ok(bytes)
}

fn send_seqpacket_frame_v1(socket: &UnixDatagram, bytes: &[u8]) -> Result<()> {
    if bytes.len() > MAX_FRAME_BYTES_V1 {
        bail!("frame exceeded the 1 MiB packet contract");
    }
    let written = socket.send(bytes).context("write seqpacket frame")?;
    if written != bytes.len() {
        bail!("seqpacket frame write was truncated");
    }
    shutdown_seqpacket_write_v1(socket)?;
    Ok(())
}

fn shutdown_seqpacket_write_v1(socket: &UnixDatagram) -> Result<()> {
    // SAFETY: shutdown only affects the supplied connected socket descriptor.
    let rc = unsafe { libc::shutdown(socket.as_raw_fd(), libc::SHUT_WR) };
    if rc != 0 {
        return Err(std::io::Error::last_os_error()).context("shutdown seqpacket write");
    }
    Ok(())
}

fn canonicalize_best_effort_v1(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::thread;
    use tempfile::tempdir;

    fn write_test_script_v1(path: &Path, contents: &str) {
        fs::write(path, contents).unwrap();
        let mut permissions = fs::metadata(path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(path, permissions).unwrap();
    }

    fn run_fake_fixed_sudo_round_trip_v1(args: &[&str], request_bytes: &[u8]) -> Value {
        let temp = tempdir().unwrap();
        let sudo_path = temp.path().join("sudo-wrapper.sh");
        let executor_path = temp.path().join("executor.py");
        write_test_script_v1(
            &sudo_path,
            "#!/usr/bin/env bash\nset -euo pipefail\n[[ \"$1\" == \"--\" ]] || exit 91\nshift\nexec \"$@\"\n",
        );
        write_test_script_v1(
            &executor_path,
            "#!/usr/bin/env python3\nimport hashlib\nimport json\nimport socket\nimport sys\nsock = socket.fromfd(1, socket.AF_UNIX, socket.SOCK_SEQPACKET)\nrequest = sock.recv(1024 * 1024 + 1)\ntrailer = sock.recv(1)\nif trailer != b'':\n    raise SystemExit('request bridge accepted multiple frames')\nresponse = json.dumps({\n    'argv': sys.argv[1:],\n    'request_len': len(request),\n    'request_sha256': hashlib.sha256(request).hexdigest(),\n}, separators=(',', ':')).encode('utf-8')\nsock.send(response)\nsock.shutdown(socket.SHUT_WR)\n",
        );
        let response_bytes = run_linux_executor_frame_with_paths_v1(
            &sudo_path,
            Path::new(FIXED_ENV_PATH_V1),
            &executor_path,
            args,
            request_bytes,
        )
        .unwrap();
        serde_json::from_slice(&response_bytes).unwrap()
    }

    #[test]
    fn seqpacket_channel_preserves_large_request_and_response_v1() {
        let request_bytes = vec![b'r'; 128 * 1024];
        let response_bytes = vec![b's'; 96 * 1024];
        let expected_request = request_bytes.clone();
        let expected_response = response_bytes.clone();

        let (parent_socket, child_fd) = open_linux_seqpacket_channel_v1().unwrap();
        let worker = thread::spawn(move || {
            let child_socket = owned_fd_to_unix_datagram_v1(child_fd).unwrap();
            let received_request = read_seqpacket_frame_v1(&child_socket).unwrap();
            send_seqpacket_frame_v1(&child_socket, &response_bytes).unwrap();
            received_request
        });

        send_seqpacket_frame_v1(&parent_socket, &request_bytes).unwrap();
        let received_response = read_seqpacket_frame_v1(&parent_socket).unwrap();
        let received_request = worker.join().unwrap();

        assert_eq!(received_request, expected_request);
        assert_eq!(received_response, expected_response);
    }

    #[test]
    fn seqpacket_channel_rejects_additional_frame_v1() {
        let (parent_socket, child_fd) = open_linux_seqpacket_channel_v1().unwrap();
        let worker = thread::spawn(move || {
            let child_socket = owned_fd_to_unix_datagram_v1(child_fd).unwrap();
            child_socket.send(b"first").unwrap();
            child_socket.send(b"second").unwrap();
            shutdown_seqpacket_write_v1(&child_socket).unwrap();
        });

        let error = read_seqpacket_frame_v1(&parent_socket).unwrap_err();
        worker.join().unwrap();
        assert!(error.to_string().contains("more than one frame"));
    }

    #[test]
    fn bootstrap_frame_round_trip_through_fixed_sudo_v1() {
        let request_bytes = vec![b'b'; 128 * 1024];
        use sha2::{Digest, Sha256};
        let request_sha256 = format!("{:x}", Sha256::digest(&request_bytes));
        let response = run_fake_fixed_sudo_round_trip_v1(
            &[
                "bootstrap-publisher",
                "--publisher-bootstrap-fd",
                EXECUTOR_CHANNEL_FD_V1,
            ],
            &request_bytes,
        );
        assert_eq!(
            response,
            json!({
                "argv": ["bootstrap-publisher", "--publisher-bootstrap-fd", EXECUTOR_CHANNEL_FD_V1],
                "request_len": request_bytes.len(),
                "request_sha256": request_sha256,
            })
        );
    }

    #[test]
    fn relay_frame_round_trip_through_fixed_sudo_v1() {
        let request_bytes = vec![b'r'; 128 * 1024];
        use sha2::{Digest, Sha256};
        let request_sha256 = format!("{:x}", Sha256::digest(&request_bytes));
        let response = run_fake_fixed_sudo_round_trip_v1(
            &["relay-request", "--relay-fd", EXECUTOR_CHANNEL_FD_V1],
            &request_bytes,
        );
        assert_eq!(
            response,
            json!({
                "argv": ["relay-request", "--relay-fd", EXECUTOR_CHANNEL_FD_V1],
                "request_len": request_bytes.len(),
                "request_sha256": request_sha256,
            })
        );
    }
}
