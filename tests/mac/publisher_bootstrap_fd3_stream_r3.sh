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
use std::io::Write;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use std::process::{Command, Stdio};
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

fn send_and_read_v1(
    chunks: Vec<Vec<u8>>,
    production_timeout: Duration,
    transfer_timeout: Duration,
) -> Result<Vec<u8>> {
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
        Instant::now() + production_timeout,
        transfer_timeout,
        "Rust regression read",
        || {},
    );
    sender.join().expect("Rust regression sender panicked")?;
    result
}

/// Exercise the exact product reader with short, deterministic writer delays. Writer errors are
/// expected when a deadline closes the reader before a deliberately late chunk arrives.
fn scheduled_read_v1(
    schedule: Vec<(Duration, Vec<u8>)>,
    production_timeout: Duration,
    transfer_timeout: Duration,
    eof_delay: Duration,
) -> Result<Vec<u8>> {
    let (reader, writer) = pair_v1()?;
    let sender = std::thread::spawn(move || {
        for (delay, chunk) in schedule {
            std::thread::sleep(delay);
            if write_retained_bootstrap_stream_v1(
                writer.as_raw_fd(),
                &chunk,
                Instant::now() + Duration::from_secs(1),
                "scheduled Rust regression write",
            )
            .is_err()
            {
                return;
            }
        }
        std::thread::sleep(eof_delay);
        let _ = unsafe { libc::shutdown(writer.as_raw_fd(), libc::SHUT_WR) };
    });
    let result = read_retained_bootstrap_stream_v1(
        reader.as_raw_fd(),
        Instant::now() + production_timeout,
        transfer_timeout,
        "scheduled Rust regression read",
        || {},
    );
    sender.join().expect("scheduled Rust regression sender panicked");
    result
}

fn blocked_milestone_sink_does_not_delay_reader_v1() -> Result<()> {
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let (sink_started_sender, sink_started_receiver) = std::sync::mpsc::channel();
    let (release_sender, release_receiver) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        let mut first = true;
        run_retained_bootstrap_milestone_sink_v1(receiver, move |_| {
            if first {
                first = false;
                sink_started_sender
                    .send(())
                    .expect("signal blocked milestone sink");
                release_receiver
                    .recv()
                    .expect("release blocked milestone sink");
            }
        });
    });
    try_enqueue_retained_bootstrap_milestone_v1(&sender, "first".to_string());
    sink_started_receiver
        .recv_timeout(Duration::from_secs(1))
        .context("wait for blocked milestone sink")?;
    try_enqueue_retained_bootstrap_milestone_v1(&sender, "fills queue".to_string());

    let started = Instant::now();
    try_enqueue_retained_bootstrap_milestone_v1(&sender, "must drop".to_string());
    let (reader, writer) = pair_v1()?;
    unsafe { libc::shutdown(writer.as_raw_fd(), libc::SHUT_WR) };
    if read_retained_bootstrap_stream_v1(
        reader.as_raw_fd(),
        Instant::now() + Duration::from_millis(100),
        Duration::from_millis(50),
        "blocked sink empty response",
        || {},
    )
    .is_ok()
    {
        bail!("reader accepted empty response while milestone sink was blocked");
    }
    if started.elapsed() > Duration::from_millis(500) {
        bail!("full milestone queue delayed an immediate FD3 reader operation");
    }
    release_sender
        .send(())
        .context("release blocked milestone sink")?;
    drop(sender);
    worker.join().expect("blocked milestone worker panicked");
    Ok(())
}

fn assert_child_reaped_exactly_once_v1(pid: libc::pid_t) -> Result<()> {
    let mut status = 0;
    if unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) } != -1
        || std::io::Error::last_os_error().raw_os_error() != Some(libc::ECHILD)
    {
        bail!("a second wait unexpectedly observed the retained executor");
    }
    Ok(())
}

fn wait_for_detached_reaper_v1(pid: libc::pid_t) -> Result<()> {
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        if unsafe { libc::kill(pid, 0) } != 0 {
            if std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH) {
                return Ok(());
            }
            bail!("detached child reaper observed an unexpected process error");
        }
        if Instant::now() >= deadline {
            bail!("detached child reaper did not reap the completed executor");
        }
        std::thread::yield_now();
    }
}

fn main() -> Result<()> {
    initialize_retained_bootstrap_milestone_logger_v1();
    let diagnostic = retained_bootstrap_milestone_line_v1(
        "client.start",
        Duration::from_millis(17),
    );
    if diagnostic != "substrate.bootstrap.milestone=client.start elapsed_ms=17" {
        bail!("client milestone diagnostic format changed");
    }
    emit_retained_bootstrap_milestone_v1(Instant::now(), "client.start");
    for forbidden in [
        "request",
        "authorization",
        "keychain",
        "signature",
        "identity",
        "service",
        "account",
        "path",
        "sha256",
        "digest",
        "confirmation",
        "peer",
    ] {
        if diagnostic.contains(forbidden) {
            bail!("client milestone diagnostic contains a forbidden field");
        }
    }

    let request_eof = Instant::now();
    let old_shared_deadline = request_eof + Duration::from_millis(5);
    let production_deadline = retained_bootstrap_response_production_deadline_v1(
        request_eof,
        Duration::from_millis(30),
    );
    let first_byte_after_old_shared_bound = request_eof + Duration::from_millis(6);
    let (production_phase, selected_production_deadline) =
        retained_bootstrap_response_read_deadline_v1(
            production_deadline,
            None,
            Duration::from_millis(5),
        );
    if production_phase != RetainedBootstrapResponseReadPhaseV1::Production
        || !retained_bootstrap_deadline_expired_v1(
            first_byte_after_old_shared_bound,
            old_shared_deadline,
        )
        || retained_bootstrap_deadline_expired_v1(
            first_byte_after_old_shared_bound,
            selected_production_deadline,
        )
        || !retained_bootstrap_deadline_expired_v1(
            request_eof + Duration::from_millis(31),
            selected_production_deadline,
        )
    {
        bail!("response production deadline no longer differs from the legacy shared bound");
    }
    let first_byte = request_eof + Duration::from_millis(29);
    let (transfer_phase, transfer_deadline) = retained_bootstrap_response_read_deadline_v1(
        production_deadline,
        Some(first_byte),
        Duration::from_millis(5),
    );
    if transfer_phase != RetainedBootstrapResponseReadPhaseV1::Transfer
        || !retained_bootstrap_deadline_expired_v1(
            request_eof + Duration::from_millis(31),
            production_deadline,
        )
        || retained_bootstrap_deadline_expired_v1(
            request_eof + Duration::from_millis(33),
            transfer_deadline,
        )
        || !retained_bootstrap_deadline_expired_v1(
            request_eof + Duration::from_millis(35),
            transfer_deadline,
        )
    {
        bail!("response transfer deadline is not a fixed window from the first byte");
    }
    let ingress_deadline = retained_bootstrap_request_ingress_deadline_v1(
        request_eof,
        Duration::from_millis(5),
    );
    let child_exit_deadline = retained_bootstrap_child_exit_deadline_v1(
        request_eof + Duration::from_millis(29),
        Duration::from_millis(5),
    );
    if ingress_deadline != request_eof + Duration::from_millis(5)
        || child_exit_deadline != request_eof + Duration::from_millis(34)
    {
        bail!("request ingress or child exit deadline was recombined with a response deadline");
    }
    blocked_milestone_sink_does_not_delay_reader_v1()?;

    let delayed_first = scheduled_read_v1(
        vec![(Duration::from_millis(90), b"delayed".to_vec())],
        Duration::from_millis(250),
        Duration::from_millis(100),
        Duration::ZERO,
    )?;
    if delayed_first != b"delayed" {
        bail!("actual reader rejected a first byte after the old bound but before production deadline");
    }
    let fresh_transfer = scheduled_read_v1(
        vec![
            (Duration::from_millis(100), b"first".to_vec()),
            (Duration::from_millis(110), b"-complete".to_vec()),
        ],
        Duration::from_millis(160),
        Duration::from_millis(160),
        Duration::ZERO,
    )?;
    if fresh_transfer != b"first-complete" {
        bail!("actual reader did not grant a fresh transfer window from the first byte");
    }
    if scheduled_read_v1(
        vec![(Duration::from_millis(140), b"late".to_vec())],
        Duration::from_millis(60),
        Duration::from_millis(100),
        Duration::ZERO,
    )
    .is_ok()
    {
        bail!("actual reader accepted a first byte after the production deadline");
    }
    if scheduled_read_v1(
        vec![
            (Duration::from_millis(20), b"a".to_vec()),
            (Duration::from_millis(45), b"b".to_vec()),
            (Duration::from_millis(45), b"c".to_vec()),
        ],
        Duration::from_millis(180),
        Duration::from_millis(70),
        Duration::ZERO,
    )
    .is_ok()
    {
        bail!("actual reader allowed trickled response bytes to extend the transfer deadline");
    }
    if scheduled_read_v1(
        vec![(Duration::ZERO, b"partial".to_vec())],
        Duration::from_millis(100),
        Duration::from_millis(60),
        Duration::from_millis(120),
    )
    .is_ok()
    {
        bail!("actual reader accepted a response without EOF");
    }

    let canonical = br#"{"bootstrap_channel_bound":true,"status":"bootstrapped"}"#.to_vec();
    let fragments = canonical.iter().map(|byte| vec![*byte]).collect();
    let fragmented = send_and_read_v1(
        fragments,
        Duration::from_secs(4),
        Duration::from_secs(4),
    )?;
    parse_canonical_direct_bootstrap_response_v1(&fragmented)?;

    let coalesced = send_and_read_v1(
        vec![canonical.clone()],
        Duration::from_secs(4),
        Duration::from_secs(4),
    )?;
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
    if send_and_read_v1(
        vec![exact.clone()],
        Duration::from_secs(8),
        Duration::from_secs(8),
    )? != exact {
        bail!("exact 1 MiB Rust frame changed bytes");
    }
    if send_and_read_v1(
        vec![vec![b'x'; MAC_BOOTSTRAP_FD3_MAX_BYTES_V1 + 1]],
        Duration::from_secs(8),
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
        Duration::from_secs(1),
        "empty Rust response",
        || {},
    )
    .is_ok()
    {
        bail!("Rust frame reader accepted an empty response");
    }

    let (disconnected_reader, disconnected_writer) = pair_v1()?;
    drop(disconnected_writer);
    if read_retained_bootstrap_stream_v1(
        disconnected_reader.as_raw_fd(),
        Instant::now() + Duration::from_secs(1),
        Duration::from_secs(1),
        "disconnected Rust response",
        || {},
    )
    .is_ok()
    {
        bail!("Rust frame reader accepted an empty peer disconnect");
    }

    let (timeout_reader, _timeout_writer) = pair_v1()?;
    let reaper_sender = start_retained_bootstrap_child_reaper_v1()
        .context("start harmless guard regression reaper before child launch")?;
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg("IFS= read -r _")
        .stdin(Stdio::piped())
        .spawn()
        .context("spawn harmless guard regression child")?;
    let pid = child.id() as libc::pid_t;
    let mut child_stdin = child.stdin.take().context("retain guard regression stdin")?;
    {
        let mut guard = RetainedBootstrapChildGuardV1::new(child, reaper_sender);
        if read_retained_bootstrap_stream_v1(
            timeout_reader.as_raw_fd(),
            Instant::now() + Duration::from_millis(100),
            Duration::from_millis(100),
            "missing EOF Rust response",
            || {},
        )
        .is_ok()
        {
            bail!("Rust frame reader accepted a missing EOF");
        }
        if guard.wait_for_success_v1(Instant::now())?
            != RetainedBootstrapChildExitV1::TimedOut
        {
            bail!("child exit timeout was not independent from the response transfer deadline");
        }
    }
    if unsafe { libc::kill(pid, 0) } != 0 {
        bail!("client timeout unexpectedly cancelled the harmless guard regression child");
    }
    child_stdin
        .write_all(b"release\n")
        .context("release harmless guard regression child")?;
    drop(child_stdin);
    wait_for_detached_reaper_v1(pid)?;
    assert_child_reaped_exactly_once_v1(pid)?;

    let reaper_sender = start_retained_bootstrap_child_reaper_v1()
        .context("start harmless drop-handoff reaper before child launch")?;
    let mut child = Command::new("/bin/sh")
        .arg("-c")
        .arg("IFS= read -r _")
        .stdin(Stdio::piped())
        .spawn()
        .context("spawn harmless drop-handoff regression child")?;
    let pid = child.id() as libc::pid_t;
    let mut child_stdin = child
        .stdin
        .take()
        .context("retain drop-handoff regression stdin")?;
    drop(RetainedBootstrapChildGuardV1::new(child, reaper_sender));
    if unsafe { libc::kill(pid, 0) } != 0 {
        bail!("client drop handoff unexpectedly cancelled the harmless child");
    }
    child_stdin
        .write_all(b"release\n")
        .context("release harmless drop-handoff regression child")?;
    drop(child_stdin);
    wait_for_detached_reaper_v1(pid)?;
    assert_child_reaped_exactly_once_v1(pid)?;

    let reaper_sender = start_retained_bootstrap_child_reaper_v1()
        .context("start successful guard reaper before child launch")?;
    let child = Command::new("/usr/bin/true")
        .spawn()
        .context("spawn successful guard regression child")?;
    let pid = child.id() as libc::pid_t;
    let mut guard = RetainedBootstrapChildGuardV1::new(child, reaper_sender);
    if guard.wait_for_success_v1(Instant::now() + Duration::from_secs(2))?
        != RetainedBootstrapChildExitV1::Reaped
    {
        bail!("successful guard did not report a reaped child");
    }
    drop(guard);
    assert_child_reaped_exactly_once_v1(pid)?;

    let reaper_sender = start_retained_bootstrap_child_reaper_v1()
        .context("start failed guard reaper before child launch")?;
    let child = Command::new("/usr/bin/false")
        .spawn()
        .context("spawn failed guard regression child")?;
    let pid = child.id() as libc::pid_t;
    let mut guard = RetainedBootstrapChildGuardV1::new(child, reaper_sender);
    if guard
        .wait_for_success_v1(Instant::now() + Duration::from_secs(2))
        .is_ok()
    {
        bail!("failed guard did not report executor failure");
    }
    drop(guard);
    assert_child_reaped_exactly_once_v1(pid)?;

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
    for token in (
        "MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1",
        "MAC_BOOTSTRAP_FD3_RESPONSE_PRODUCTION_TIMEOUT_V1",
        "MAC_BOOTSTRAP_FD3_RESPONSE_TRANSFER_TIMEOUT_V1",
        "MAC_BOOTSTRAP_FD3_CHILD_EXIT_TIMEOUT_V1",
        "retained_bootstrap_request_ingress_deadline_v1",
        "retained_bootstrap_response_production_deadline_v1",
        "retained_bootstrap_response_transfer_deadline_v1",
        "retained_bootstrap_child_exit_deadline_v1",
        "RetainedBootstrapResponseReadPhaseV1",
        "client.response.first_byte",
        "client.child_exit.complete",
        "client.child_exit.timeout",
    ):
        require(token in client, f"client missing distinct retained-FD3 deadline token {token}")
    for constant, seconds in (
        ("MAC_BOOTSTRAP_FD3_REQUEST_INGRESS_TIMEOUT_V1", 5),
        ("MAC_BOOTSTRAP_FD3_RESPONSE_PRODUCTION_TIMEOUT_V1", 30),
        ("MAC_BOOTSTRAP_FD3_RESPONSE_TRANSFER_TIMEOUT_V1", 5),
        ("MAC_BOOTSTRAP_FD3_CHILD_EXIT_TIMEOUT_V1", 5),
    ):
        require(
            f"const {constant}: std::time::Duration =\n    std::time::Duration::from_secs({seconds});" in client,
            f"retained-FD3 deadline changed: {constant}",
        )
    require("MAC_BOOTSTRAP_FD3_TIMEOUT_V1" not in client,
            "retained-FD3 deadlines collapsed back to one shared timeout")
    require("child.kill()" not in client,
            "client timeout must not hard-kill an in-flight executor")
    child_reaper = client[
        client.index("fn start_retained_bootstrap_child_reaper_v1"):
        client.index("impl RetainedBootstrapChildGuardV1")
    ]
    child_guard_drop = client[
        client.index("impl Drop for RetainedBootstrapChildGuardV1"):
        client.index("mod retained_bootstrap_fd3_deadline_tests")
    ]
    require(
        "std::sync::mpsc::channel" in child_reaper
        and ".spawn(move ||" in child_reaper
        and "run_retained_bootstrap_child_reaper_v1(receiver);" in child_reaper
        and "loop {" in child_reaper
        and "receiver.recv()" in child_reaper
        and "child.wait()" in child_reaper
        and ".join()" not in child_reaper
        and "child.kill()" not in child_reaper
        and ".send(child)" in child_reaper
        and "handoff_retained_bootstrap_child_to_reaper_v1(&self.reaper_sender, child);" in child_guard_drop
        and "None if std::time::Instant::now() >= deadline" in client
        and client.index("start_retained_bootstrap_child_reaper_v1()")
        < client.index("let child = command")
        and client.index("handoff_retained_bootstrap_child_to_reaper_v1(&self.reaper_sender, child);")
        < client.index("return Ok(RetainedBootstrapChildExitV1::TimedOut)"),
        "timed-out retained executor is not handed to a prelaunch wait-only reaper",
    )

    client_dispatch = client[
        client.index("pub fn send_publisher_bootstrap_authorization_fd3_v1"):
        client.index("fn set_retained_bootstrap_fd_cloexec_v1")
    ]
    require(
        client_dispatch.index("initialize_retained_bootstrap_milestone_logger_v1()")
        < client_dispatch.index("let milestones_started"),
        "client milestone sink is not initialized before protocol deadlines",
    )
    require(
        client_dispatch.index("let request_eof_observed")
        < client_dispatch.index("let response_production_deadline")
        < client_dispatch.index('"client.request.eof"'),
        "client production deadline is not armed at request EOF before diagnostics",
    )
    client_reader = client[
        client.index("fn read_retained_bootstrap_stream_v1"):
        client.index("fn parse_canonical_direct_bootstrap_response_v1")
    ]
    require(
        client_reader.index(
            "active_deadline = retained_bootstrap_response_transfer_deadline_v1"
        ) < client_reader.index("on_first_response_byte()"),
        "client transfer deadline is not armed before first-byte diagnostics",
    )
    client_logger = client[
        client.index("const RETAINED_BOOTSTRAP_MILESTONE_QUEUE_CAPACITY_V1"):
        client.index("fn set_retained_bootstrap_no_sigpipe_v1")
    ]
    client_emit = client[
        client.index("fn emit_retained_bootstrap_milestone_v1"):
        client.index("fn set_retained_bootstrap_no_sigpipe_v1")
    ]
    require(
        "std::sync::mpsc::sync_channel" in client_logger
        and "sender.try_send(line)" in client_logger
        and "run_retained_bootstrap_milestone_sink_v1" in client_logger
        and ".join()" not in client_logger
        and "eprintln!" not in client_emit,
        "client milestones can still block the FD3 protocol on stderr",
    )

    stage_one_milestones = (
        "executor.stage_one.derive.start",
        "executor.stage_one.input.preparation.start",
        "executor.stage_one.input.preparation.end",
        "executor.stage_one.signing_material.ready",
        "executor.stage_one.signer.call.start",
        "executor.stage_one.signer.p256.return",
        "executor.stage_one.signer.call.end",
    )
    for milestone in stage_one_milestones:
        for forbidden in ("request", "authorization", "keychain", "signature", "identity",
                          "service", "account", "path", "sha256", "digest", "confirmation",
                          "peer"):
            require(forbidden not in milestone,
                    f"Stage-1 milestone is material: {milestone}")
    stage_one_derivation = executor[
        executor.index("fn derive_mac_lima_stage_one_authorization_v1"):
        executor.index("fn bootstrap_mac_publisher_from_authorized_fd3_v1")
    ]
    require(
        stage_one_derivation.index("executor.stage_one.derive.start")
        < stage_one_derivation.index("executor.stage_one.input.preparation.start")
        < stage_one_derivation.index("executor.stage_one.input.preparation.end")
        < stage_one_derivation.index("sign_mac_lima_stage_one_authorization_v1"),
        "Stage-1 derive/input milestones are not ordered",
    )
    stage_one_signer = executor[
        executor.index("fn sign_mac_lima_stage_one_authorization_v1"):
        executor.index("fn derive_mac_lima_stage_one_authorization_v1")
    ]
    require(
        stage_one_signer.index("executor.stage_one.signing_material.ready")
        < stage_one_signer.index("executor.stage_one.signer.call.start")
        < stage_one_signer.index("mac_system_keychain_sign_p1363_low_s_v1")
        < stage_one_signer.index("executor.stage_one.signer.p256.return")
        < stage_one_signer.index("executor.stage_one.signer.call.end"),
        "Stage-1 signer milestones are not ordered",
    )

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
    require(
        consumer.index("initialize_mac_bootstrap_milestone_logger_v1()")
        < consumer.index("let milestones_started"),
        "executor milestone sink is not initialized before protocol deadlines",
    )
    require(
        consumer.index("let response_write_deadline")
        < consumer.index('"executor.response.serialization.complete"')
        < consumer.index("mac_send_single_stream_document_v1"),
        "executor response deadline is not armed before response diagnostics",
    )
    executor_logger = executor[
        executor.index("const MAC_BOOTSTRAP_MILESTONE_QUEUE_CAPACITY_V1"):
        executor.index("struct MacBootstrapPeerIdentityV1")
    ]
    executor_emit = executor[
        executor.index("fn emit_mac_bootstrap_milestone_v1"):
        executor.index("struct MacBootstrapPeerIdentityV1")
    ]
    require(
        "std::sync::mpsc::sync_channel" in executor_logger
        and "sender.try_send(line)" in executor_logger
        and "run_mac_bootstrap_milestone_sink_v1" in executor_logger
        and ".join()" not in executor_logger
        and "eprintln!" not in executor_emit,
        "executor milestones can still block the FD3 protocol on stderr",
    )
    require(
        '#[cfg(target_os = "macos")]\nfn emit_mac_bootstrap_milestone_v1' in executor
        and '#[cfg(not(target_os = "macos"))]\nfn emit_mac_bootstrap_milestone_v1' in executor,
        "executor milestone emitters are not cfg-symmetric",
    )
    non_macos_emit = executor[
        executor.index('#[cfg(not(target_os = "macos"))]\nfn emit_mac_bootstrap_milestone_v1'):
        executor.index("struct MacBootstrapPeerIdentityV1")
    ]
    require(
        "let _ = (started, milestone);" in non_macos_emit
        and "MAC_BOOTSTRAP_MILESTONE_SENDER_V1" not in non_macos_emit
        and "try_enqueue_mac_bootstrap_milestone_v1" not in non_macos_emit,
        "non-macOS milestone emitter references macOS-only logging state",
    )
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
