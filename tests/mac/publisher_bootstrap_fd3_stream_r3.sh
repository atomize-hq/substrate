#!/usr/bin/env bash
# Live Darwin regression for the one-shot retained publisher-bootstrap FD3 channel. It uses only
# temporary socketpairs/processes and never invokes sudo, an installer, Lima, launchd, Keychain,
# codesign mutation, the privileged helper, publisher bootstrap, pairing, or native evidence.
set -euo pipefail

SCRIPT_NAME="publisher-bootstrap-fd3-stream-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '[%s] SKIP: Darwin-only AF_UNIX SOCK_STREAM regression\n' "${SCRIPT_NAME}"
  exit 0
fi

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-fd3-stream-r3.XXXXXX")"
cleanup() {
  rm -rf "${WORK_ROOT}"
}
trap cleanup EXIT

# Compile and execute the exact client framing/parser/child-guard helper bodies from the product
# source. The extraction is deliberate: crates/shell's unrelated legacy cfg(test) surface does not
# currently compile, while this bounded harness proves the implementation bytes rather than a
# Python reimplementation. Cargo is offline and every generated artifact stays under WORK_ROOT.
mkdir -p "${WORK_ROOT}/rust-client/src"
python3 - "${REPO_ROOT}" "${WORK_ROOT}/rust-client/src/main.rs" <<'PY'
import pathlib
import sys

repo = pathlib.Path(sys.argv[1])
destination = pathlib.Path(sys.argv[2])
source = (repo / "crates/shell/src/execution/managed_lifecycle/macos_client.rs").read_text()
start_marker = '#[cfg(target_os = "macos")]\nfn set_retained_bootstrap_fd_cloexec_v1'
end_marker = 'pub fn issue_guest_publisher_pairing_ticket_v1'
start = source.index(start_marker)
end = source.index(end_marker, start)
helpers = source[start:end]

preamble = r'''
use anyhow::{bail, Context, Result};
use serde_json::Value;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::process::Command;
use std::time::{Duration, Instant};

const MAC_BOOTSTRAP_FD3_MAX_BYTES_V1: usize = 1024 * 1024;
'''

tests = r'''
fn pair_v1() -> Result<(UnixStream, UnixStream)> {
    let (left, right) = UnixStream::pair().context("create Rust regression stream pair")?;
    set_retained_bootstrap_fd_cloexec_v1(left.as_raw_fd())?;
    set_retained_bootstrap_fd_cloexec_v1(right.as_raw_fd())?;
    set_retained_bootstrap_no_sigpipe_v1(left.as_raw_fd())?;
    set_retained_bootstrap_no_sigpipe_v1(right.as_raw_fd())?;
    Ok((left, right))
}

fn send_and_read_v1(chunks: Vec<Vec<u8>>, timeout: Duration) -> Result<Vec<u8>> {
    let (reader, writer) = pair_v1()?;
    let sender = std::thread::spawn(move || -> Result<()> {
        for chunk in chunks {
            write_retained_bootstrap_stream_v1(
                writer.as_raw_fd(),
                &chunk,
                Instant::now() + Duration::from_secs(4),
                "Rust regression write",
            )?;
        }
        if unsafe { libc::shutdown(writer.as_raw_fd(), libc::SHUT_WR) } != 0 {
            return Err(std::io::Error::last_os_error()).context("finish Rust regression write");
        }
        Ok(())
    });
    let result = read_retained_bootstrap_stream_v1(
        reader.as_raw_fd(),
        Instant::now() + timeout,
        "Rust regression read",
    );
    sender.join().expect("Rust regression sender panicked")?;
    result
}

fn assert_reaped_v1(pid: libc::pid_t) -> Result<()> {
    let kill_result = unsafe { libc::kill(pid, 0) };
    if kill_result == 0 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ESRCH) {
        bail!("guarded child still exists after error cleanup");
    }
    let mut status = 0;
    let wait_result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
    if wait_result != -1 || std::io::Error::last_os_error().raw_os_error() != Some(libc::ECHILD) {
        bail!("guarded child was killed but not reaped");
    }
    Ok(())
}

fn main() -> Result<()> {
    let canonical = br#"{"bootstrap_channel_bound":true,"status":"bootstrapped"}"#.to_vec();
    let fragments = canonical.iter().map(|byte| vec![*byte]).collect();
    let fragmented = send_and_read_v1(fragments, Duration::from_secs(4))?;
    parse_canonical_direct_bootstrap_response_v1(&fragmented)?;

    let coalesced = send_and_read_v1(vec![canonical.clone()], Duration::from_secs(4))?;
    if coalesced != canonical {
        bail!("coalesced Rust stream changed bytes");
    }
    for invalid in [
        [canonical.clone(), b" ".to_vec()].concat(),
        [canonical.clone(), canonical.clone()].concat(),
        br#"{"bootstrap_channel_bound":"#.to_vec(),
    ] {
        if parse_canonical_direct_bootstrap_response_v1(&invalid).is_ok() {
            bail!("Rust canonical parser accepted an invalid document");
        }
    }

    let exact = vec![b'x'; MAC_BOOTSTRAP_FD3_MAX_BYTES_V1];
    if send_and_read_v1(vec![exact.clone()], Duration::from_secs(8))? != exact {
        bail!("exact 1 MiB Rust frame changed bytes");
    }
    if send_and_read_v1(
        vec![vec![b'x'; MAC_BOOTSTRAP_FD3_MAX_BYTES_V1 + 1]],
        Duration::from_secs(8),
    )
    .is_ok()
    {
        bail!("Rust frame reader accepted 1 MiB plus one byte");
    }

    let (empty_reader, empty_writer) = pair_v1()?;
    unsafe { libc::shutdown(empty_writer.as_raw_fd(), libc::SHUT_WR) };
    if read_retained_bootstrap_stream_v1(
        empty_reader.as_raw_fd(),
        Instant::now() + Duration::from_secs(1),
        "empty Rust response",
    )
    .is_ok()
    {
        bail!("Rust frame reader accepted an empty response");
    }

    let (timeout_reader, _timeout_writer) = pair_v1()?;
    let child = Command::new("/bin/sleep")
        .arg("30")
        .spawn()
        .context("spawn harmless guard regression child")?;
    let pid = child.id() as libc::pid_t;
    {
        let _guard = RetainedBootstrapChildGuardV1::new(child);
        if read_retained_bootstrap_stream_v1(
            timeout_reader.as_raw_fd(),
            Instant::now() + Duration::from_millis(100),
            "missing EOF Rust response",
        )
        .is_ok()
        {
            bail!("Rust frame reader accepted a missing EOF");
        }
    }
    assert_reaped_v1(pid)?;

    let child = Command::new("/usr/bin/true")
        .spawn()
        .context("spawn successful guard regression child")?;
    let mut guard = RetainedBootstrapChildGuardV1::new(child);
    guard.wait_for_success_v1(Instant::now() + Duration::from_secs(2))?;

    println!("publisher-bootstrap-fd3-stream-r3-rust-client: PASS");
    Ok(())
}
'''

destination.write_text(preamble + helpers + tests)
PY

cat >"${WORK_ROOT}/rust-client/Cargo.toml" <<'TOML'
[package]
name = "publisher-bootstrap-fd3-stream-r3-client-regression"
version = "0.0.0"
edition = "2021"

[dependencies]
anyhow = "1.0"
libc = "0.2"
serde_json = "1.0"
TOML

cargo run --offline --quiet --manifest-path "${WORK_ROOT}/rust-client/Cargo.toml"

cat >"${WORK_ROOT}/fd3_stream_regression.py" <<'PY'
import ctypes
import errno
import fcntl
import gc
import json
import os
import pathlib
import pty
import select
import signal
import socket
import struct
import subprocess
import sys
import threading
import time

MAX_BYTES = 1024 * 1024
SO_NOSIGPIPE = 0x1022
SOL_LOCAL = 0
LOCAL_PEERPID = 0x002
HARNESS = pathlib.Path(__file__).resolve()
LIBC = ctypes.CDLL(None, use_errno=True)
LIBC.getpeereid.argtypes = [ctypes.c_int, ctypes.POINTER(ctypes.c_uint), ctypes.POINTER(ctypes.c_uint)]
LIBC.getpeereid.restype = ctypes.c_int
LIBC.tcgetsid.argtypes = [ctypes.c_int]
LIBC.tcgetsid.restype = ctypes.c_int
LIBC.getsid.argtypes = [ctypes.c_int]
LIBC.getsid.restype = ctypes.c_int


class FrameError(Exception):
    pass


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def set_no_sigpipe(sock):
    sock.setsockopt(socket.SOL_SOCKET, SO_NOSIGPIPE, 1)
    require(sock.getsockopt(socket.SOL_SOCKET, SO_NOSIGPIPE) == 1, "SO_NOSIGPIPE did not stick")


def read_eof_frame(sock, timeout):
    deadline = time.monotonic() + timeout
    output = bytearray()
    while True:
        remaining_time = deadline - time.monotonic()
        if remaining_time <= 0:
            raise TimeoutError("absolute frame deadline expired")
        readable, _, _ = select.select([sock], [], [], remaining_time)
        if not readable:
            raise TimeoutError("absolute frame deadline expired")
        remaining = MAX_BYTES - len(output)
        chunk = sock.recv(1 if remaining == 0 else min(16384, remaining))
        if not chunk:
            if not output:
                raise FrameError("empty EOF-delimited frame")
            return bytes(output)
        if remaining == 0:
            raise FrameError("frame exceeds 1 MiB")
        output.extend(chunk)


def canonical_json_document(raw):
    def reject_float(_value):
        raise FrameError("non-integer number")

    try:
        value = json.loads(
            raw.decode("utf-8"),
            parse_float=reject_float,
            parse_constant=reject_float,
        )
    except (UnicodeDecodeError, json.JSONDecodeError, FrameError) as error:
        raise FrameError("invalid JSON document") from error
    canonical = json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=False,
    ).encode("utf-8")
    if canonical != raw:
        raise FrameError("noncanonical JSON document")
    return value


def threaded_send(sock, chunks, shutdown=True, delay=0.0, hold=0.0):
    def run():
        try:
            for chunk in chunks:
                sock.sendall(chunk)
                if delay:
                    time.sleep(delay)
            if shutdown:
                sock.shutdown(socket.SHUT_WR)
            if hold:
                time.sleep(hold)
        except OSError:
            pass
        finally:
            sock.close()

    thread = threading.Thread(target=run, daemon=True)
    thread.start()
    return thread


def frame_pair():
    left, right = socket.socketpair(socket.AF_UNIX, socket.SOCK_STREAM)
    require(left.getsockopt(socket.SOL_SOCKET, socket.SO_TYPE) == socket.SOCK_STREAM, "left SO_TYPE")
    require(right.getsockopt(socket.SOL_SOCKET, socket.SO_TYPE) == socket.SOCK_STREAM, "right SO_TYPE")
    set_no_sigpipe(left)
    set_no_sigpipe(right)
    return left, right


def expect_frame_error(label, operation, expected):
    try:
        operation()
    except expected:
        return
    raise AssertionError(f"{label}: expected {expected}")


def framing_cases():
    canonical = b'{"bootstrap_channel_bound":true,"status":"bootstrapped"}'

    left, right = frame_pair()
    thread = threaded_send(right, [canonical[index:index + 1] for index in range(len(canonical))])
    require(canonical_json_document(read_eof_frame(left, 2.0))["bootstrap_channel_bound"], "fragmented JSON")
    left.close(); thread.join(1.0)

    left, right = frame_pair()
    thread = threaded_send(right, [canonical[:7], canonical[7:]])
    require(read_eof_frame(left, 2.0) == canonical, "coalesced stream bytes changed")
    left.close(); thread.join(1.0)

    left, right = frame_pair()
    right.shutdown(socket.SHUT_WR); right.close()
    expect_frame_error("empty", lambda: read_eof_frame(left, 1.0), FrameError)
    left.close()

    exact = b"x" * MAX_BYTES
    left, right = frame_pair()
    thread = threaded_send(right, [exact])
    require(read_eof_frame(left, 3.0) == exact, "exact 1 MiB did not succeed")
    left.close(); thread.join(1.0)

    left, right = frame_pair()
    thread = threaded_send(right, [exact + b"x"])
    expect_frame_error("1 MiB plus one", lambda: read_eof_frame(left, 3.0), FrameError)
    left.close(); thread.join(1.0)

    for label, invalid in (
        ("trailing whitespace", canonical + b" "),
        ("concatenated JSON", canonical + canonical),
        ("premature EOF", b'{"bootstrap_channel_bound":'),
    ):
        left, right = frame_pair()
        thread = threaded_send(right, [invalid])
        raw = read_eof_frame(left, 1.0)
        expect_frame_error(label, lambda raw=raw: canonical_json_document(raw), FrameError)
        left.close(); thread.join(1.0)

    left, right = frame_pair()
    thread = threaded_send(right, [canonical], shutdown=False, hold=0.5)
    expect_frame_error("missing EOF", lambda: read_eof_frame(left, 0.15), TimeoutError)
    left.close(); thread.join(1.0)

    left, right = frame_pair()
    thread = threaded_send(right, [canonical[index:index + 1] for index in range(12)], shutdown=False, delay=0.04)
    started = time.monotonic()
    expect_frame_error("trickle", lambda: read_eof_frame(left, 0.16), TimeoutError)
    require(time.monotonic() - started < 0.4, "trickle extended the absolute deadline")
    left.close(); thread.join(1.0)

    left, right = frame_pair()
    thread = threaded_send(right, [canonical[:9]])
    raw = read_eof_frame(left, 1.0)
    expect_frame_error("disconnect partial", lambda: canonical_json_document(raw), FrameError)
    left.close(); thread.join(1.0)

    left, right = frame_pair()
    expect_frame_error("timeout without bytes", lambda: read_eof_frame(left, 0.1), TimeoutError)
    left.close(); right.close()


def fd_snapshot(limit=512):
    result = set()
    for fd in range(limit):
        try:
            fcntl.fcntl(fd, fcntl.F_GETFD)
        except OSError as error:
            if error.errno != errno.EBADF:
                raise
        else:
            result.add(fd)
    return result


def spawn_failure_cleanup():
    before = fd_snapshot()
    for mode in ("spawn", "preexec"):
        left, right = frame_pair()
        left.set_inheritable(False)
        right.set_inheritable(False)
        try:
            if mode == "spawn":
                subprocess.Popen(["/definitely/absent/substrate-helper"], pass_fds=(right.fileno(),))
            else:
                def fail_preexec():
                    raise OSError(errno.EPERM, "injected pre-exec failure")
                subprocess.Popen(["/usr/bin/true"], pass_fds=(right.fileno(),), preexec_fn=fail_preexec)
        except (FileNotFoundError, subprocess.SubprocessError):
            pass
        else:
            raise AssertionError(f"{mode} failure injection unexpectedly succeeded")
        finally:
            left.close(); right.close()
    gc.collect()
    require(fd_snapshot() == before, "spawn/pre-exec failure leaked a descriptor")


def no_sigpipe_case():
    pid = os.fork()
    if pid == 0:
        signal.signal(signal.SIGPIPE, signal.SIG_DFL)
        writer, peer = frame_pair()
        peer.close()
        try:
            writer.send(b"x")
        except OSError as error:
            os._exit(0 if error.errno == errno.EPIPE else 2)
        os._exit(3)
    waited, result = os.waitpid(pid, 0)
    require(waited == pid and os.WIFEXITED(result) and os.WEXITSTATUS(result) == 0,
            "SO_NOSIGPIPE did not convert closed-peer send to EPIPE")


def child_role(require_tty):
    inherited_flags = fcntl.fcntl(3, fcntl.F_GETFD)
    require(inherited_flags & fcntl.FD_CLOEXEC == 0, "FD3 did not cross the one intended exec")
    fcntl.fcntl(3, fcntl.F_SETFD, inherited_flags | fcntl.FD_CLOEXEC)
    open_fds = sorted(fd_snapshot(128))
    require(open_fds == [0, 1, 2, 3], f"unexpected inherited descriptors: {open_fds}")

    channel = socket.socket(fileno=3)
    require(channel.getsockopt(socket.SOL_SOCKET, socket.SO_TYPE) == socket.SOCK_STREAM, "child SO_TYPE")
    require(channel.getsockopt(socket.SOL_SOCKET, SO_NOSIGPIPE) == 1, "child SO_NOSIGPIPE")

    uid = ctypes.c_uint()
    gid = ctypes.c_uint()
    require(LIBC.getpeereid(3, ctypes.byref(uid), ctypes.byref(gid)) == 0, "getpeereid failed")
    require((uid.value, gid.value) == (os.getuid(), os.getgid()), "getpeereid mismatch")
    peer_pid = struct.unpack("i", channel.getsockopt(SOL_LOCAL, LOCAL_PEERPID, 4))[0]
    require(peer_pid == os.getppid(), f"LOCAL_PEERPID mismatch: {peer_pid} != {os.getppid()}")

    tty_bound = False
    try:
        tty_fd = os.open("/dev/tty", os.O_RDONLY)
    except OSError:
        if require_tty:
            raise AssertionError("PTY test expected a controlling terminal")
    else:
        try:
            terminal_session = LIBC.tcgetsid(tty_fd)
            peer_session = LIBC.getsid(peer_pid)
            tty_bound = terminal_session > 0 and terminal_session == peer_session
            require(tty_bound, "peer does not join the controlling-terminal session")
        finally:
            os.close(tty_fd)

    descendant = subprocess.run(
        [sys.executable, str(HARNESS), "grandchild"],
        close_fds=False,
        check=False,
        capture_output=True,
        text=True,
    )
    require(descendant.returncode == 0, f"descendant inherited FD3: {descendant.stderr}")

    response = json.dumps(
        {
            "bootstrap_channel_bound": True,
            "getpeereid": True,
            "local_peerpid": peer_pid,
            "only_fd3_inherited": True,
            "terminal_bound": tty_bound,
        },
        sort_keys=True,
        separators=(",", ":"),
    ).encode()
    channel.sendall(response)
    channel.shutdown(socket.SHUT_WR)
    channel.close()


def grandchild_role():
    try:
        fcntl.fcntl(3, fcntl.F_GETFD)
    except OSError as error:
        if error.errno == errno.EBADF:
            return
        raise
    raise AssertionError("executor descendant inherited re-armed FD3")


def fd3_exec_roundtrip(require_tty=False):
    retained, peer = frame_pair()
    retained.set_inheritable(False)
    peer.set_inheritable(False)
    extra = os.open("/dev/null", os.O_RDONLY)
    os.set_inheritable(extra, True)
    pid = os.fork()
    if pid == 0:
        try:
            os.dup2(peer.fileno(), 3, inheritable=True)
            for fd in range(4, 256):
                try:
                    os.close(fd)
                except OSError:
                    pass
            os.execve(
                sys.executable,
                [sys.executable, str(HARNESS), "child", "tty" if require_tty else "optional-tty"],
                {"PATH": "/usr/bin:/bin"},
            )
        finally:
            os._exit(127)
    peer.close()
    os.close(extra)
    response = canonical_json_document(read_eof_frame(retained, 3.0))
    retained.close()
    waited, result = os.waitpid(pid, 0)
    require(waited == pid and os.WIFEXITED(result) and os.WEXITSTATUS(result) == 0,
            "FD3 fork/exec child failed")
    require(response["only_fd3_inherited"] is True, "FD3 inventory result")
    require(response["getpeereid"] is True, "getpeereid result")
    if require_tty:
        require(response["terminal_bound"] is True, "controlling-terminal binding result")


def terminal_binding_case():
    pid, master = pty.fork()
    if pid == 0:
        try:
            fd3_exec_roundtrip(require_tty=True)
            print("TTY_BOUND", flush=True)
            os._exit(0)
        except BaseException:
            import traceback
            traceback.print_exc()
            os._exit(1)
    output = bytearray()
    while True:
        try:
            chunk = os.read(master, 4096)
        except OSError as error:
            if error.errno == errno.EIO:
                break
            raise
        if not chunk:
            break
        output.extend(chunk)
    os.close(master)
    waited, result = os.waitpid(pid, 0)
    require(waited == pid and os.WIFEXITED(result) and os.WEXITSTATUS(result) == 0,
            f"PTY terminal-binding check failed: {output.decode(errors='replace')}")
    require(b"TTY_BOUND" in output, "PTY terminal-binding sentinel absent")


def child_kill_reap_case():
    retained, peer = frame_pair()
    child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(30)"])
    try:
        expect_frame_error("child timeout", lambda: read_eof_frame(retained, 0.1), TimeoutError)
    finally:
        child.kill()
        child.wait(timeout=2.0)
        retained.close(); peer.close()
    require(child.poll() is not None, "timed-out child was not reaped")
    try:
        os.waitpid(child.pid, os.WNOHANG)
    except ChildProcessError:
        pass
    else:
        raise AssertionError("timed-out child remains waitable")


def installer_parser_case(repo_root):
    installer = (repo_root / "scripts/substrate/dev-install-substrate.sh").read_text()
    marker = 'stage_one_authorization="$(python3 - "${bootstrap_response}" <<\'PY\'\n'
    start = installer.index(marker) + len(marker)
    end = installer.index(
        '\nPY\n)" || fatal "direct publisher-bootstrap returned an invalid Stage-1 result"',
        start,
    )
    parser = installer[start:end]
    sha = "a" * 64
    response = {
        "status": "bootstrapped",
        "scope_id": "018f0f38-7d8a-7a9a-8b7c-0123456789ab",
        "manifest_generation": 1,
        "manifest_sha256": sha,
        "authorization_sha256": sha,
        "anchor_sha256": sha,
        "lima_stage_one_authorization_v1": {
            "schema_owner": "substrate.lima-stage-one-authorization",
            "schema_version": 1,
            "expected_absent": True,
        },
        "bootstrap_channel_bound": True,
    }
    canonical = json.dumps(response, sort_keys=True, separators=(",", ":"))
    accepted = subprocess.run([sys.executable, "-c", parser, canonical], capture_output=True, text=True)
    require(accepted.returncode == 0, f"direct response without XPC attestation rejected: {accepted.stderr}")
    require(json.loads(accepted.stdout)["expected_absent"] is True, "Stage-1 extraction failed")

    response["xpc_attestation"] = {"audit_token_bound": True}
    extra_xpc = subprocess.run(
        [sys.executable, "-c", parser, json.dumps(response, sort_keys=True, separators=(",", ":"))],
        capture_output=True,
        text=True,
    )
    require(extra_xpc.returncode != 0, "direct parser accepted a fabricated XPC field")
    response.pop("xpc_attestation")
    response["bootstrap_channel_bound"] = False
    unbound = subprocess.run(
        [sys.executable, "-c", parser, json.dumps(response, sort_keys=True, separators=(",", ":"))],
        capture_output=True,
        text=True,
    )
    require(unbound.returncode != 0, "direct parser accepted an unbound bootstrap response")


def source_shape_case(repo_root):
    client = (repo_root / "crates/shell/src/execution/managed_lifecycle/macos_client.rs").read_text()
    executor = (repo_root / "src/bin/substrate-lifecycle-macos.rs").read_text()
    linux_client = (repo_root / "crates/shell/src/execution/managed_lifecycle/linux_client.rs").read_text()
    linux_executor = (repo_root / "src/bin/substrate-lifecycle-linux.rs").read_text()

    require("libc::SOCK_STREAM" in client and "libc::SOCK_SEQPACKET" not in client,
            "Darwin client transport selector")
    socketpair = client.index("libc::socketpair(libc::AF_UNIX, libc::SOCK_STREAM")
    retained_owned = client.index("OwnedFd::from_raw_fd(pair[0])", socketpair)
    peer_owned = client.index("OwnedFd::from_raw_fd(pair[1])", retained_owned)
    cloexec = client.index("set_retained_bootstrap_fd_cloexec_v1", peer_owned)
    spawn = client.index('Command::new("/usr/bin/sudo")', cloexec)
    require(socketpair < retained_owned < peer_owned < cloexec < spawn, "client RAII/CLOEXEC/spawn order")
    for token in ("SO_NOSIGPIPE", "MSG_DONTWAIT", "SHUT_WR", "RetainedBootstrapChildGuardV1",
                  "parse_canonical_direct_bootstrap_response_v1"):
        require(token in client, f"client missing {token}")

    consumer = executor[
        executor.index("fn consume_publisher_bootstrap_fd3_v1"):
        executor.index("struct MacBootstrapPeerIdentityV1")
    ]
    order = [
        consumer.index("if fd != 3"),
        consumer.index("mac_rearm_bootstrap_fd3_cloexec_v1(fd)?"),
        consumer.index("mac_require_stream_channel_v1(fd)?"),
        consumer.index("mac_bootstrap_peer_identity_v1"),
        consumer.index("mac_attest_bootstrap_control_peer_predecode_v1"),
        consumer.index("parse_publisher_bootstrap_authorization_v1"),
        consumer.index("mac_attest_running_executor_image_v1"),
    ]
    require(order == sorted(order), "executor FD3 CLOEXEC/admission/decode order")
    for token in ("libc::SOCK_STREAM", "libc::SO_NOSIGPIPE", "libc::MSG_DONTWAIT",
                  "mac_read_single_stream_document_v1", "mac_send_single_stream_document_v1"):
        require(token in executor, f"executor missing {token}")
    require("SOCK_SEQPACKET" in linux_client and "SOCK_SEQPACKET_V1" in linux_executor,
            "Linux publisher seqpacket endpoints changed")

    xpc = client[client.index("pub fn attest_mac_publisher_response_v1"):]
    require("xpc_attestation" in xpc and "audit_token_bound" in xpc,
            "actual XPC attestation requirements changed")


def main(repo_root):
    # Framing/parser/guard correctness is exercised above by compiling the exact Rust helper
    # bodies. These Python cases are supplemental Darwin kernel/exec/terminal integration proof.
    spawn_failure_cleanup()
    no_sigpipe_case()
    fd3_exec_roundtrip(require_tty=False)
    terminal_binding_case()
    installer_parser_case(repo_root)
    source_shape_case(repo_root)
    print("publisher-bootstrap-fd3-stream-r3: PASS")


if __name__ == "__main__":
    role = sys.argv[1] if len(sys.argv) > 1 else "main"
    if role == "child":
        child_role(len(sys.argv) > 2 and sys.argv[2] == "tty")
    elif role == "grandchild":
        grandchild_role()
    else:
        main(pathlib.Path(sys.argv[1]).resolve())
PY

python3 "${WORK_ROOT}/fd3_stream_regression.py" "${REPO_ROOT}"
