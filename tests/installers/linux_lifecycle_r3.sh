#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="linux-lifecycle-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
BINARY="${REPO_ROOT}/target/debug/substrate-lifecycle-linux"
KEEP_ROOT=0

log() {
  printf '[%s] %s\n' "${SCRIPT_NAME}" "$*" >&2
}

fail() {
  log "ERROR: $*"
  exit 1
}

usage() {
  cat <<'USAGE' >&2
Usage: tests/installers/linux_lifecycle_r3.sh [--keep-root]

Builds and exercises the Linux managed lifecycle provider in a non-privileged
temp root. The harness stubs systemctl, drives the stream and socket-activated
seqpacket transport paths plus relay behavior, verifies exact retry/restore
joins, and runs the guest pairing path through a real PTY with transcript and
replay negatives.
USAGE
}

record_skip() {
  log "Skipping: $1"
  exit 0
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --keep-root)
      KEEP_ROOT=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fail "Unknown argument: $1"
      ;;
  esac
done

if [[ "$(uname -s 2>/dev/null || true)" != "Linux" ]]; then
  record_skip "non-Linux platform"
fi

command -v cargo >/dev/null 2>&1 || fail "cargo is required"
command -v openssl >/dev/null 2>&1 || fail "openssl is required"
command -v python3 >/dev/null 2>&1 || fail "python3 is required"

WORK_ROOT="$(mktemp -d "/tmp/substrate-linux-lifecycle-r3.XXXXXX")"
STUB_BIN="${WORK_ROOT}/stub-bin"
SYSTEMCTL_STATE_DIR="${WORK_ROOT}/systemctl-state"
HELPER_PY="${WORK_ROOT}/helper.py"
mkdir -p "${STUB_BIN}" "${SYSTEMCTL_STATE_DIR}"

declare -a CLEANUP_PIDS=()

cleanup() {
  for pid in "${CLEANUP_PIDS[@]:-}"; do
    if kill -0 "${pid}" 2>/dev/null; then
      kill "${pid}" 2>/dev/null || true
      wait "${pid}" 2>/dev/null || true
    fi
  done
  if [[ "${KEEP_ROOT}" -eq 0 ]]; then
    rm -rf -- "${WORK_ROOT}"
  else
    log "Preserving artifacts under ${WORK_ROOT}"
  fi
}
trap cleanup EXIT

cat >"${STUB_BIN}/systemctl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
state_dir="${SUBSTRATE_LIFECYCLE_TEST_STATE_DIR:?}"

key_for() {
  local unit="$1"
  local key="${unit//[^A-Za-z0-9_.-]/_}"
  printf '%s' "${key}"
}

state_path() {
  local unit="$1"
  local kind="$2"
  printf '%s/%s.%s\n' "${state_dir}" "$(key_for "${unit}")" "${kind}"
}

read_state() {
  local path="$1"
  local fallback="$2"
  if [[ -f "${path}" ]]; then
    cat "${path}"
  else
    printf '%s\n' "${fallback}"
  fi
}

write_state() {
  local path="$1"
  local value="$2"
  printf '%s\n' "${value}" >"${path}"
}

verb="${1:-}"
shift || true
runtime=0
if [[ "${1:-}" == "--runtime" ]]; then
  runtime=1
  shift
fi
unit="${1:-}"
[[ -n "${verb}" && -n "${unit}" ]] || exit 1

case "${verb}" in
  is-active)
    state="$(read_state "$(state_path "${unit}" active)" "inactive")"
    printf '%s\n' "${state}"
    case "${state}" in
      active|activating|reloading)
        exit 0
        ;;
      *)
        exit 3
        ;;
    esac
    ;;
  is-enabled)
    state="$(read_state "$(state_path "${unit}" enabled)" "disabled")"
    printf '%s\n' "${state}"
    case "${state}" in
      enabled|enabled-runtime|static)
        exit 0
        ;;
      *)
        exit 1
        ;;
    esac
    ;;
  start)
    enabled_state="$(read_state "$(state_path "${unit}" enabled)" "disabled")"
    case "${enabled_state}" in
      masked|masked-runtime)
        exit 1
        ;;
    esac
    write_state "$(state_path "${unit}" active)" "active"
    ;;
  stop)
    write_state "$(state_path "${unit}" active)" "inactive"
    ;;
  enable)
    if [[ "${runtime}" -eq 1 ]]; then
      write_state "$(state_path "${unit}" enabled)" "enabled-runtime"
    else
      write_state "$(state_path "${unit}" enabled)" "enabled"
    fi
    ;;
  disable)
    write_state "$(state_path "${unit}" enabled)" "disabled"
    ;;
  mask)
    if [[ "${runtime}" -eq 1 ]]; then
      write_state "$(state_path "${unit}" enabled)" "masked-runtime"
    else
      write_state "$(state_path "${unit}" enabled)" "masked"
    fi
    ;;
  unmask)
    write_state "$(state_path "${unit}" enabled)" "disabled"
    ;;
  status)
    printf '%s active=%s enabled=%s\n' \
      "${unit}" \
      "$(read_state "$(state_path "${unit}" active)" "inactive")" \
      "$(read_state "$(state_path "${unit}" enabled)" "disabled")"
    ;;
  *)
    exit 1
    ;;
esac
EOF
chmod +x "${STUB_BIN}/systemctl"

cat >"${HELPER_PY}" <<'PY'
import base64
import fcntl
import hashlib
import json
import os
import pathlib
import pty
import socket
import subprocess
import sys
import termios
import time

P256_ORDER = int(
    "FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551",
    16,
)
PREFIX = b"SUBSTRATE-LIFECYCLE-SIGNATURE-V1\0"
PAIRING_LITERAL = "PAIR EXACT SUBSTRATE GUEST PUBLISHER"
MAX_FRAME_BYTES = 1024 * 1024


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


def b64url_decode(text: str) -> bytes:
    raw = text.encode("ascii")
    return base64.urlsafe_b64decode(raw + b"=" * ((-len(raw)) % 4))


def canonical(value):
    if value is None:
        return "null"
    if value is True:
        return "true"
    if value is False:
        return "false"
    if isinstance(value, int):
        return str(value)
    if isinstance(value, str):
        return json.dumps(value, ensure_ascii=False, separators=(",", ":"))
    if isinstance(value, list):
        return "[" + ",".join(canonical(item) for item in value) + "]"
    if isinstance(value, dict):
        items = []
        for key in sorted(value):
            items.append(
                json.dumps(key, ensure_ascii=False, separators=(",", ":"))
                + ":"
                + canonical(value[key])
            )
        return "{" + ",".join(items) + "}"
    raise TypeError(f"unsupported canonical value: {type(value)!r}")


def canonical_bytes(value) -> bytes:
    return canonical(value).encode("utf-8")


def sha256_hex(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def load_json(path):
    with open(path, "r", encoding="utf-8") as handle:
        return json.load(handle)


def dump_json(path, value):
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(value, handle, sort_keys=True, separators=(",", ":"))
        handle.write("\n")


def get_path(value, path):
    if not path:
        return value
    current = value
    for part in path.split("."):
        if part == "":
            continue
        if isinstance(current, list):
            current = current[int(part)]
        else:
            current = current[part]
    return current


def set_path_value(value, path, replacement):
    parts = path.split(".")
    target = value
    for part in parts[:-1]:
        target = target[int(part)] if isinstance(target, list) else target[part]
    target[parts[-1]] = replacement


def managed_request_sha(path):
    value = load_json(path)
    if value.get("platform_mapping_commitment") is None:
        value.pop("platform_mapping_commitment", None)
    object_identity = value.get("object_identity")
    if isinstance(object_identity, dict) and object_identity.get("metadata") is None:
        object_identity.pop("metadata", None)
    return sha256_hex(canonical_bytes(value))


def sign_record(key_path: str, owner: str, record: dict) -> str:
    unsigned = dict(record)
    unsigned.pop("signature", None)
    payload = PREFIX + owner.encode("utf-8") + b"\0" + canonical_bytes(unsigned)
    result = subprocess.run(
        ["openssl", "dgst", "-sha256", "-sign", key_path, "-binary"],
        input=payload,
        capture_output=True,
        check=True,
    )
    der = result.stdout
    if len(der) < 8 or der[0] != 0x30:
        raise RuntimeError("unexpected DER signature encoding")
    total_len = der[1]
    payload = der[2 : 2 + total_len]
    if len(payload) != total_len or payload[0] != 0x02:
        raise RuntimeError("unexpected DER integer encoding")
    r_len = payload[1]
    r_bytes = payload[2 : 2 + r_len]
    rest = payload[2 + r_len :]
    if len(rest) < 2 or rest[0] != 0x02:
        raise RuntimeError("unexpected DER second integer encoding")
    s_len = rest[1]
    s_bytes = rest[2 : 2 + s_len]
    if len(s_bytes) != s_len:
        raise RuntimeError("truncated DER signature")
    r = int.from_bytes(r_bytes, "big")
    s = int.from_bytes(s_bytes, "big")
    if s > P256_ORDER // 2:
        s = P256_ORDER - s
    raw = r.to_bytes(32, "big") + s.to_bytes(32, "big")
    return b64url(raw)


def current_target_triple() -> str:
    result = subprocess.run(
        ["rustc", "-vV"],
        capture_output=True,
        text=True,
        check=True,
    )
    for line in result.stdout.splitlines():
        if line.startswith("host: "):
            return line.split(": ", 1)[1].strip()
    raise RuntimeError("rustc -vV did not report a host target triple")


def write_bootstrap_auth(
    out_path: str,
    scope_id: str,
    attempt_nonce: str,
    artifact_sha: str,
    artifact_identity: str,
):
    component_id = "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a92"
    target = f"/var/lib/substrate/.substrate-lifecycle-bootstrap-intent-v1.{scope_id}.json"
    now_ns = time.time_ns()
    target_triple = current_target_triple()
    value = {
        "schema_owner": "substrate.publisher-bootstrap-authorization",
        "schema_version": 1,
        "authority_domain": "linux_system",
        "scope_id": scope_id,
        "host_context_commitment": "1" * 64,
        "platform_mapping_commitment": None,
        "requester_principal": "tester:1000",
        "source_commit": "a" * 40,
        "source_tree": "b" * 40,
        "source_ref": "refs/heads/test",
        "issued_at_unix_ns": now_ns,
        "expires_at_unix_ns": now_ns + 600_000_000_000,
        "attempt_nonce": attempt_nonce,
        "publisher_expected_absent": True,
        "executor_build_evidence": {
            "schema_owner": "substrate.executor-build-evidence",
            "schema_version": 1,
            "source_commit": "a" * 40,
            "source_tree": "b" * 40,
            "source_ref": "refs/heads/test",
            "artifact_sha256": artifact_sha,
            "artifact_identity": os.path.realpath(artifact_identity),
            "target_triple": target_triple,
            "tool_versions": {},
            "code_identity": None,
        },
        "components": [
            {
                "component_id": component_id,
                "role": "LinuxBootstrapIntent",
                "target_identity": target,
                "object_type": "regular-file",
                "expected_before": None,
                "source_artifact_sha256": None,
                "source_signature_sha256": None,
                "owner": "root",
                "group_name": None,
                "mode": "0600",
                "acl_or_security": None,
                "code_requirement": None,
                "dependency_component_ids": [],
                "durability_method": "otmpfile-linkat-fsync",
            }
        ],
        "test_retirement_commitment": None,
        "confirmation": "CREATE EXACT SUBSTRATE LIFECYCLE PUBLISHER",
    }
    dump_json(out_path, value)


def build_request(
    out_path: str,
    state_path: str,
    role: str,
    action: str,
    attempt_nonce: str,
    physical_identity: str,
    metadata_text: str,
):
    state = load_json(state_path)
    anchor = state["current_anchor"]
    metadata = None if metadata_text == "-" else json.loads(metadata_text)
    request = {
        "host_context_commitment": anchor["host_context_commitment"],
        "platform_mapping_commitment": anchor.get("platform_mapping_commitment"),
        "scope_id": anchor["scope_id"],
        "current_anchor_counter": state["counter"],
        "current_anchor_sha256": sha256_hex(canonical_bytes(anchor)),
        "manifest_generation": anchor["manifest_generation"],
        "manifest_sha256": anchor["manifest_sha256"],
        "role": role,
        "action": action,
        "object_identity": {
            "scope_id": anchor["scope_id"],
            "parent_identity": os.path.dirname(physical_identity) or "/",
            "name_identity": os.path.basename(physical_identity) or role,
            "physical_identity": physical_identity,
            "metadata": metadata,
        },
        "requester_principal": anchor["requester_principal"],
        "attempt_nonce": attempt_nonce,
        "expected_executor_build": anchor["executor_identity"],
    }
    dump_json(out_path, request)


def write_ticket(
    out_path: str,
    state_path: str,
    key_path: str,
    spki_path: str,
    challenge_id: str,
    expires_at_unix_ns=None,
):
    state = load_json(state_path)
    anchor = state["current_anchor"]
    spki_der = pathlib.Path(spki_path).read_bytes()
    spki_b64 = b64url(spki_der)
    anchor_sha = sha256_hex(canonical_bytes(anchor))
    if expires_at_unix_ns is None:
        expires_at_unix_ns = time.time_ns() + 600_000_000_000
    challenge = {
        "schema_owner": "substrate.guest-publisher-pairing-challenge",
        "schema_version": 1,
        "challenge_id": challenge_id,
        "challenge": b64url(b"challenge-value"),
        "expires_at_unix_ns": expires_at_unix_ns,
        "host_key_fingerprint_sha256": sha256_hex(spki_der),
        "current_anchor_sha256": anchor_sha,
        "host_context_commitment": anchor["host_context_commitment"],
        "platform_mapping_commitment": anchor.get("platform_mapping_commitment"),
        "guest_machine_identity": "linux-guest-test",
        "source_commit": anchor["executor_identity"]["source_commit"],
        "source_tree": anchor["executor_identity"]["source_tree"],
        "source_ref": anchor["executor_identity"]["source_ref"],
        "executor_build_evidence_sha256": anchor["executor_identity"]["artifact_sha256"],
        "guest_component_commitment_sha256": "d" * 64,
    }
    if challenge["platform_mapping_commitment"] is None:
        challenge.pop("platform_mapping_commitment")
    ticket = {
        "schema_owner": "substrate.guest-publisher-pairing-ticket",
        "schema_version": 1,
        "challenge": challenge,
        "signer_spki_der": spki_b64,
        "current_anchor": anchor,
        "current_anchor_sha256": anchor_sha,
        "challenge_sha256": sha256_hex(canonical_bytes(challenge)),
        "host_generation": anchor["manifest_generation"],
        "host_counter": state["counter"],
        "guest_test_retirement_commitment": None,
        "signature": {
            "algorithm": "ecdsa-p256-sha256-p1363-low-s-v1",
            "public_key": spki_b64,
            "signature": "",
        },
    }
    if ticket["guest_test_retirement_commitment"] is None:
        ticket.pop("guest_test_retirement_commitment")
    ticket["signature"]["signature"] = sign_record(
        key_path, ticket["schema_owner"], ticket
    )
    dump_json(out_path, ticket)


def write_transcript(out_path: str, ticket_path: str, hello_path: str, key_path: str):
    ticket = load_json(ticket_path)
    hello = load_json(hello_path)
    public_key_sha = sha256_hex(b64url_decode(hello["guest_public_key"]))
    transcript = {
        "schema_owner": "substrate.guest-publisher-bootstrap-transcript",
        "schema_version": 1,
        "ticket_sha256": sha256_hex(canonical_bytes(ticket)),
        "hello_sha256": sha256_hex(canonical_bytes(hello)),
        "guest_test_retirement_commitment": None,
        "guest_machine_identity": hello["guest_machine_identity"],
        "staged_executor_sha256": hello["guest_artifact_sha256"],
        "guest_nonce": hello["nonce"],
        "guest_public_key_sha256": public_key_sha,
        "signature": {
            "algorithm": "ecdsa-p256-sha256-p1363-low-s-v1",
            "public_key": ticket["signer_spki_der"],
            "signature": "",
        },
    }
    if transcript["guest_test_retirement_commitment"] is None:
        transcript.pop("guest_test_retirement_commitment")
    transcript["signature"]["signature"] = sign_record(
        key_path, transcript["schema_owner"], transcript
    )
    dump_json(out_path, transcript)


def run_begin_pty(
    out_json: str,
    out_prompt: str,
    binary: str,
    state_root: str,
    ticket_path: str,
    kill_at: str = "none",
    allow_prompt: bool = True,
):
    ticket = load_json(ticket_path)
    master_fd, slave_fd = pty.openpty()
    slave_path = os.ttyname(slave_fd)

    def prepare_tty():
        os.setsid()
        fcntl.ioctl(0, termios.TIOCSCTTY, 0)

    command = [
        binary,
        "begin-guest-bootstrap",
        "--state-root",
        state_root,
        "--tty-path",
        slave_path,
        "--ticket-file",
        ticket_path,
    ]
    if kill_at != "none":
        command.extend(["--kill-at", kill_at])
    proc = subprocess.Popen(
        command,
        stdin=slave_fd,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        preexec_fn=prepare_tty,
    )
    os.close(slave_fd)
    prompt = bytearray()
    deadline = time.time() + 10.0
    saw_literal = False
    while time.time() < deadline:
        if proc.poll() is not None:
            break
        try:
            chunk = os.read(master_fd, 4096)
        except OSError as exc:
            if exc.errno == 5:
                break
            raise
        if not chunk:
            break
        prompt.extend(chunk)
        if b"LITERAL PAIR EXACT SUBSTRATE GUEST PUBLISHER" in prompt:
            saw_literal = True
            if not allow_prompt:
                proc.kill()
                raise RuntimeError("unexpected guest pairing prompt during durable retry")
            break
    if saw_literal and proc.poll() is None:
        response = (
            ticket["challenge"]["host_key_fingerprint_sha256"]
            + "\n"
            + ticket["challenge"]["challenge"]
            + "\n"
            + PAIRING_LITERAL
            + "\n"
            + "\x04"
        ).encode("utf-8")
        os.write(master_fd, response)
    stdout, stderr = proc.communicate(timeout=10.0)
    os.close(master_fd)
    if proc.returncode != 0:
        raise RuntimeError(stderr.strip() or "begin-guest-bootstrap failed")
    pathlib.Path(out_json).write_text(stdout, encoding="utf-8")
    pathlib.Path(out_prompt).write_text(prompt.decode("utf-8", errors="replace"), encoding="utf-8")


def relay_request(
    out_json: str,
    out_meta: str,
    binary: str,
    state_root: str,
    socket_path: str,
    request_path: str,
    transport: str,
):
    socket_kind = socket.SOCK_SEQPACKET if transport == "seqpacket" else socket.SOCK_STREAM
    parent, child = socket.socketpair(socket.AF_UNIX, socket_kind)
    proc = subprocess.Popen(
        [
            binary,
            "relay-request",
            "--state-root",
            state_root,
            "--socket",
            socket_path,
            "--transport",
            transport,
            "--relay-fd",
            str(child.fileno()),
        ],
        pass_fds=(child.fileno(),),
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    child.close()
    request_bytes = pathlib.Path(request_path).read_bytes()
    if transport == "seqpacket":
        written = parent.send(request_bytes)
        if written != len(request_bytes):
            raise RuntimeError("seqpacket relay request write was truncated")
        parent.shutdown(socket.SHUT_WR)
        response = parent.recv(MAX_FRAME_BYTES + 1)
        if not response:
            raise RuntimeError("seqpacket relay did not return a response")
        if len(response) > MAX_FRAME_BYTES:
            raise RuntimeError("seqpacket relay response exceeded the packet contract")
    else:
        parent.sendall(request_bytes)
        parent.shutdown(socket.SHUT_WR)
        response = bytearray()
        while True:
            chunk = parent.recv(4096)
            if not chunk:
                break
            response.extend(chunk)
        response = bytes(response)
    parent.close()
    stdout, stderr = proc.communicate(timeout=10.0)
    if proc.returncode != 0:
        raise RuntimeError((stderr or b"").decode("utf-8", errors="replace").strip())
    pathlib.Path(out_json).write_bytes(response)
    pathlib.Path(out_meta).write_text(
        json.dumps(
            {
                "caller_pid": os.getpid(),
                "caller_exe": os.path.realpath(sys.executable),
                "child_stdout": (stdout or b"").decode("utf-8", errors="replace"),
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        + "\n",
        encoding="utf-8",
    )


def start_seqpacket_publisher(
    binary: str,
    state_root: str,
    socket_path: str,
    kill_at: str,
    log_path: str,
):
    socket_target = pathlib.Path(socket_path)
    socket_target.parent.mkdir(parents=True, exist_ok=True)
    if socket_target.exists():
        socket_target.unlink()

    listener = socket.socket(socket.AF_UNIX, socket.SOCK_SEQPACKET)
    listener.bind(socket_path)
    listener.listen(16)
    listener_fd = listener.fileno()
    log_handle = open(log_path, "wb")

    def configure_listener():
        os.dup2(listener_fd, 3, inheritable=True)
        os.environ["LISTEN_FDS"] = "1"
        os.environ["LISTEN_PID"] = str(os.getpid())

    proc = subprocess.Popen(
        [
            binary,
            "run-publisher",
            "--state-root",
            state_root,
            "--socket",
            socket_path,
            "--transport",
            "seqpacket",
            "--kill-at",
            kill_at,
        ],
        pass_fds=(listener_fd,),
        stdout=log_handle,
        stderr=subprocess.STDOUT,
        preexec_fn=configure_listener,
    )
    listener.close()
    log_handle.close()
    print(proc.pid)


def main(argv):
    command = argv[1]
    if command == "bootstrap-auth":
        write_bootstrap_auth(argv[2], argv[3], argv[4], argv[5], argv[6])
    elif command == "request":
        build_request(argv[2], argv[3], argv[4], argv[5], argv[6], argv[7], argv[8])
    elif command == "ticket":
        expiry = int(argv[7]) if len(argv) > 7 else None
        write_ticket(argv[2], argv[3], argv[4], argv[5], argv[6], expiry)
    elif command == "transcript":
        write_transcript(argv[2], argv[3], argv[4], argv[5])
    elif command == "begin-pty":
        kill_at = argv[7] if len(argv) > 7 else "none"
        allow_prompt = len(argv) <= 8 or argv[8] != "forbid-prompt"
        run_begin_pty(argv[2], argv[3], argv[4], argv[5], argv[6], kill_at, allow_prompt)
    elif command == "relay":
        relay_request(argv[2], argv[3], argv[4], argv[5], argv[6], argv[7], argv[8])
    elif command == "start-seqpacket":
        start_seqpacket_publisher(argv[2], argv[3], argv[4], argv[5], argv[6])
    elif command == "json-get":
        value = get_path(load_json(argv[2]), argv[3])
        if isinstance(value, bool):
          print("true" if value else "false")
        elif value is None:
          print("null")
        else:
          print(value)
    elif command == "canonical-sha":
        value = get_path(load_json(argv[2]), argv[3] if len(argv) > 3 else "")
        print(sha256_hex(canonical_bytes(value)))
    elif command == "request-sha":
        print(managed_request_sha(argv[2]))
    elif command == "set-field":
        value = load_json(argv[2])
        set_path_value(value, argv[3], json.loads(argv[4]))
        dump_json(argv[2], value)
    elif command == "set-repeat-string":
        value = load_json(argv[2])
        set_path_value(value, argv[3], argv[4] + argv[5] * int(argv[6]))
        dump_json(argv[2], value)
    else:
        raise SystemExit(f"unknown command: {command}")


if __name__ == "__main__":
    main(sys.argv)
PY

export PATH="${STUB_BIN}:${PATH}"
export SUBSTRATE_LIFECYCLE_TEST_STATE_DIR="${SYSTEMCTL_STATE_DIR}"

json_get() {
  python3 "${HELPER_PY}" json-get "$1" "$2"
}

json_sha() {
  if [[ $# -eq 1 ]]; then
    python3 "${HELPER_PY}" canonical-sha "$1"
  else
    python3 "${HELPER_PY}" canonical-sha "$1" "$2"
  fi
}

set_json_field() {
  python3 "${HELPER_PY}" set-field "$1" "$2" "$3"
}

assert_eq() {
  local expected="$1"
  local actual="$2"
  local message="$3"
  if [[ "${expected}" != "${actual}" ]]; then
    fail "${message}: expected '${expected}', got '${actual}'"
  fi
}

assert_contains() {
  local needle="$1"
  local path="$2"
  local message="$3"
  if ! grep -Fq -- "${needle}" "${path}"; then
    fail "${message} (missing '${needle}' in ${path})"
  fi
}

assert_not_exists() {
  local path="$1"
  local message="$2"
  if [[ -e "${path}" ]]; then
    fail "${message}: ${path}"
  fi
}

sha256_text() {
  printf '%s' "$1" | sha256sum | awk '{print $1}'
}

set_unit_state() {
  local unit="$1"
  local enabled="$2"
  local active="$3"
  local key="${unit//[^A-Za-z0-9_.-]/_}"
  printf '%s\n' "${enabled}" >"${SYSTEMCTL_STATE_DIR}/${key}.enabled"
  printf '%s\n' "${active}" >"${SYSTEMCTL_STATE_DIR}/${key}.active"
}

start_publisher() {
  local state_root="$1"
  local socket_path="$2"
  local kill_at="$3"
  local log_path="$4"

  "${BINARY}" run-publisher \
    --state-root "${state_root}" \
    --socket "${socket_path}" \
    --transport stream \
    --kill-at "${kill_at}" >"${log_path}" 2>&1 &
  local pid=$!
  CLEANUP_PIDS+=("${pid}")

  for _ in $(seq 1 100); do
    if [[ -S "${socket_path}" ]]; then
      printf '%s\n' "${pid}"
      return 0
    fi
    if ! kill -0 "${pid}" 2>/dev/null; then
      sed 's/^/[publisher] /' "${log_path}" >&2 || true
      fail "publisher exited before binding ${socket_path}"
    fi
    sleep 0.05
  done

  sed 's/^/[publisher] /' "${log_path}" >&2 || true
  fail "publisher did not bind ${socket_path}"
}

start_seqpacket_publisher() {
  local state_root="$1"
  local socket_path="$2"
  local kill_at="$3"
  local log_path="$4"
  local pid

  pid="$(
    python3 "${HELPER_PY}" start-seqpacket \
      "${BINARY}" \
      "${state_root}" \
      "${socket_path}" \
      "${kill_at}" \
      "${log_path}"
  )"

  for _ in $(seq 1 100); do
    if [[ -S "${socket_path}" ]] && kill -0 "${pid}" 2>/dev/null; then
      printf '%s\n' "${pid}"
      return 0
    fi
    if ! kill -0 "${pid}" 2>/dev/null; then
      sed 's/^/[publisher] /' "${log_path}" >&2 || true
      fail "seqpacket publisher exited before becoming ready on ${socket_path}"
    fi
    sleep 0.05
  done

  sed 's/^/[publisher] /' "${log_path}" >&2 || true
  fail "seqpacket publisher did not become ready on ${socket_path}"
}

start_direct_seqpacket_publisher() {
  local state_root="$1"
  local socket_path="$2"
  local kill_at="$3"
  local log_path="$4"
  local pid

  "${BINARY}" run-publisher \
    --state-root "${state_root}" \
    --socket "${socket_path}" \
    --transport seqpacket \
    --kill-at "${kill_at}" >"${log_path}" 2>&1 &
  pid=$!

  for _ in $(seq 1 100); do
    if [[ -S "${socket_path}" ]] && kill -0 "${pid}" 2>/dev/null; then
      printf '%s\n' "${pid}"
      return 0
    fi
    if ! kill -0 "${pid}" 2>/dev/null; then
      sed 's/^/[publisher] /' "${log_path}" >&2 || true
      fail "direct seqpacket publisher exited before becoming ready on ${socket_path}"
    fi
    sleep 0.05
  done

  sed 's/^/[publisher] /' "${log_path}" >&2 || true
  fail "direct seqpacket publisher did not become ready on ${socket_path}"
}

stop_publisher() {
  local pid="$1"
  if kill -0 "${pid}" 2>/dev/null; then
    kill "${pid}" 2>/dev/null || true
    wait "${pid}" 2>/dev/null || true
  fi
}

run_expect_fail() {
  local stderr_path="$1"
  shift
  set +e
  "$@" > /dev/null 2>"${stderr_path}"
  local status=$?
  set -e
  if [[ "${status}" -eq 0 ]]; then
    fail "expected failure for command: $*"
  fi
}

log "Building substrate-lifecycle-linux"
cargo build -p substrate --bin substrate-lifecycle-linux >/dev/null
[[ -x "${BINARY}" ]] || fail "binary missing at ${BINARY}"

BINARY_SHA="$(sha256sum "${BINARY}" | awk '{print $1}')"

BOOTSTRAP_AUTH="${WORK_ROOT}/bootstrap-auth.json"
python3 "${HELPER_PY}" bootstrap-auth \
  "${BOOTSTRAP_AUTH}" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a91" \
  "${BINARY_SHA}" \
  "${BINARY}"

BOOTSTRAP_ROOT="${WORK_ROOT}/bootstrap-root"
mkdir -p "${BOOTSTRAP_ROOT}"
chmod 0750 "${BOOTSTRAP_ROOT}"
printf 'keep-me\n' >"${BOOTSTRAP_ROOT}/unrelated-sibling"

log "Checking bootstrap create and pre-existing reuse"
"${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_ROOT}" <"${BOOTSTRAP_AUTH}" >"${WORK_ROOT}/bootstrap-create.json"
assert_eq "GenerationOneAnchored" "$(json_get "${WORK_ROOT}/bootstrap-create.json" "state")" "bootstrap create state"
assert_eq "keep-me" "$(<"${BOOTSTRAP_ROOT}/unrelated-sibling")" "bootstrap must preserve unrelated sibling state"
assert_eq "750" "$(stat -c '%a' "${BOOTSTRAP_ROOT}")" "bootstrap must preserve pre-existing state root mode"
BOOTSTRAP_INTENT_PATH="$(json_get "${WORK_ROOT}/bootstrap-create.json" "bootstrap_intent_path")"
assert_eq \
  "${BOOTSTRAP_ROOT}/.substrate-lifecycle-bootstrap-intent-v1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90.json" \
  "${BOOTSTRAP_INTENT_PATH}" \
  "bootstrap intent path"
[[ -f "${BOOTSTRAP_INTENT_PATH}" ]] || fail "bootstrap intent missing at ${BOOTSTRAP_INTENT_PATH}"
assert_eq "KeyDurable" "$(json_get "${BOOTSTRAP_INTENT_PATH}" "state")" "bootstrap intent durable state"
assert_eq \
  "$(json_get "${WORK_ROOT}/bootstrap-create.json" "request_sha256")" \
  "$(json_get "${BOOTSTRAP_INTENT_PATH}" "request_sha256")" \
  "bootstrap intent request sha"

"${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_ROOT}" <"${BOOTSTRAP_AUTH}" >"${WORK_ROOT}/bootstrap-reuse.json"
assert_eq "true" "$(json_get "${WORK_ROOT}/bootstrap-reuse.json" "reused")" "bootstrap retry must reuse existing protected state"
assert_eq \
  "$(json_get "${WORK_ROOT}/bootstrap-create.json" "protected_state_sha256")" \
  "$(json_get "${WORK_ROOT}/bootstrap-reuse.json" "protected_state_sha256")" \
  "bootstrap retry protected_state_sha256"
assert_eq \
  "${BOOTSTRAP_INTENT_PATH}" \
  "$(json_get "${WORK_ROOT}/bootstrap-reuse.json" "bootstrap_intent_path")" \
  "bootstrap retry intent path"

BOOTSTRAP_INTENT_KILL_ROOT="${WORK_ROOT}/bootstrap-intent-kill-root"
mkdir -p "${BOOTSTRAP_INTENT_KILL_ROOT}"
chmod 0750 "${BOOTSTRAP_INTENT_KILL_ROOT}"
run_expect_fail "${WORK_ROOT}/bootstrap-intent-kill.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_INTENT_KILL_ROOT}" --kill-at intent <"${BOOTSTRAP_AUTH}"
INTENT_KILL_PATH="${BOOTSTRAP_INTENT_KILL_ROOT}/.substrate-lifecycle-bootstrap-intent-v1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90.json"
[[ -f "${INTENT_KILL_PATH}" ]] || fail "bootstrap intent kill-point must publish the durable intent"
assert_not_exists "${BOOTSTRAP_INTENT_KILL_ROOT}/.substrate-lifecycle-v1/publisher/current-anchor.v1.json" \
  "bootstrap intent kill-point must not publish current-anchor before retry"
assert_not_exists "${BOOTSTRAP_INTENT_KILL_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1" \
  "bootstrap intent kill-point must not publish signing-key before retry"
"${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_INTENT_KILL_ROOT}" <"${BOOTSTRAP_AUTH}" >"${WORK_ROOT}/bootstrap-intent-kill-retry.json"
assert_eq "true" "$(json_get "${WORK_ROOT}/bootstrap-intent-kill-retry.json" "reused_pending")" \
  "bootstrap retry must resume from a durable intent-only crash window"

BOOTSTRAP_COMMIT_KILL_ROOT="${WORK_ROOT}/bootstrap-commit-kill-root"
mkdir -p "${BOOTSTRAP_COMMIT_KILL_ROOT}"
chmod 0750 "${BOOTSTRAP_COMMIT_KILL_ROOT}"
run_expect_fail "${WORK_ROOT}/bootstrap-commit-kill.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_COMMIT_KILL_ROOT}" --kill-at commit <"${BOOTSTRAP_AUTH}"
COMMIT_KILL_INTENT="${BOOTSTRAP_COMMIT_KILL_ROOT}/.substrate-lifecycle-bootstrap-intent-v1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a90.json"
[[ -f "${COMMIT_KILL_INTENT}" ]] || fail "bootstrap commit kill-point must preserve the durable intent"
[[ -f "${BOOTSTRAP_COMMIT_KILL_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1" ]] \
  || fail "bootstrap commit kill-point must preserve the durable signing key"
assert_not_exists "${BOOTSTRAP_COMMIT_KILL_ROOT}/.substrate-lifecycle-v1/publisher/current-anchor.v1.json" \
  "bootstrap commit kill-point must not publish current-anchor before retry"
"${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_COMMIT_KILL_ROOT}" <"${BOOTSTRAP_AUTH}" >"${WORK_ROOT}/bootstrap-commit-kill-retry.json"
assert_eq "true" "$(json_get "${WORK_ROOT}/bootstrap-commit-kill-retry.json" "reused_pending")" \
  "bootstrap retry must resume from a durable intent-plus-signing-key crash window"
assert_eq "KeyDurable" "$(json_get "${COMMIT_KILL_INTENT}" "state")" \
  "bootstrap commit kill-point must record durable signing-key identity before retry"

BOOTSTRAP_COMMIT_REPLACE_KEY_ROOT="${WORK_ROOT}/bootstrap-commit-replace-key-root"
mkdir -p "${BOOTSTRAP_COMMIT_REPLACE_KEY_ROOT}"
chmod 0750 "${BOOTSTRAP_COMMIT_REPLACE_KEY_ROOT}"
run_expect_fail "${WORK_ROOT}/bootstrap-commit-replace-key-initial.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_COMMIT_REPLACE_KEY_ROOT}" --kill-at commit <"${BOOTSTRAP_AUTH}"
COMMIT_REPLACE_KEY_PATH="${BOOTSTRAP_COMMIT_REPLACE_KEY_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1"
openssl genpkey -algorithm ed25519 -out "${WORK_ROOT}/bootstrap-replacement-key.pem" >/dev/null 2>&1
cp "${WORK_ROOT}/bootstrap-replacement-key.pem" "${COMMIT_REPLACE_KEY_PATH}"
chmod 0600 "${COMMIT_REPLACE_KEY_PATH}"
run_expect_fail "${WORK_ROOT}/bootstrap-commit-replace-key-retry.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_COMMIT_REPLACE_KEY_ROOT}" <"${BOOTSTRAP_AUTH}"
assert_contains "does not exact-join the durable bootstrap intent" "${WORK_ROOT}/bootstrap-commit-replace-key-retry.stderr" \
  "bootstrap retry must reject a replaced durable signing key"

BOOTSTRAP_ANCHORED_REPLACE_KEY_ROOT="${WORK_ROOT}/bootstrap-anchored-replace-key-root"
mkdir -p "${BOOTSTRAP_ANCHORED_REPLACE_KEY_ROOT}"
chmod 0750 "${BOOTSTRAP_ANCHORED_REPLACE_KEY_ROOT}"
"${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_ANCHORED_REPLACE_KEY_ROOT}" <"${BOOTSTRAP_AUTH}" >"${WORK_ROOT}/bootstrap-anchored-create.json"
ANCHORED_REPLACE_KEY_PATH="${BOOTSTRAP_ANCHORED_REPLACE_KEY_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1"
openssl genpkey -algorithm ed25519 -out "${WORK_ROOT}/bootstrap-anchored-replacement-key.pem" >/dev/null 2>&1
cp "${WORK_ROOT}/bootstrap-anchored-replacement-key.pem" "${ANCHORED_REPLACE_KEY_PATH}"
chmod 0600 "${ANCHORED_REPLACE_KEY_PATH}"
run_expect_fail "${WORK_ROOT}/bootstrap-anchored-replace-key-retry.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_ANCHORED_REPLACE_KEY_ROOT}" <"${BOOTSTRAP_AUTH}"
assert_contains "does not exact-join the anchored bootstrap state" "${WORK_ROOT}/bootstrap-anchored-replace-key-retry.stderr" \
  "anchored bootstrap retry must reject a replaced final signing key"

cp "${BOOTSTRAP_AUTH}" "${WORK_ROOT}/bootstrap-expired.json"
set_json_field "${WORK_ROOT}/bootstrap-expired.json" "issued_at_unix_ns" "1"
set_json_field "${WORK_ROOT}/bootstrap-expired.json" "expires_at_unix_ns" "2"
run_expect_fail "${WORK_ROOT}/bootstrap-expired.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_ROOT}" <"${WORK_ROOT}/bootstrap-expired.json"
assert_contains \
  "publisher bootstrap authorization is expired or not yet valid" \
  "${WORK_ROOT}/bootstrap-expired.stderr" \
  "bootstrap must reject expired authorization windows"
cp "${BOOTSTRAP_AUTH}" "${WORK_ROOT}/bootstrap-mismatch.json"
set_json_field "${WORK_ROOT}/bootstrap-mismatch.json" "attempt_nonce" '"018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3aff"'
run_expect_fail "${WORK_ROOT}/bootstrap-mismatch.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${BOOTSTRAP_ROOT}" <"${WORK_ROOT}/bootstrap-mismatch.json"
assert_contains \
  "request digest does not match the bootstrap authorization" \
  "${WORK_ROOT}/bootstrap-mismatch.stderr" \
  "bootstrap retry must reject non-identical authorization bytes"

INVALID_ROOT="${WORK_ROOT}/invalid-root"
mkdir -p "${INVALID_ROOT}/.substrate-lifecycle-v1/publisher"
printf 'reserved\n' >"${INVALID_ROOT}/.substrate-lifecycle-v1/publisher/current-anchor.v1.json"
printf 'preserve-invalid\n' >"${INVALID_ROOT}/sibling"
run_expect_fail "${WORK_ROOT}/bootstrap-invalid.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${INVALID_ROOT}" <"${BOOTSTRAP_AUTH}"
assert_eq "preserve-invalid" "$(<"${INVALID_ROOT}/sibling")" "invalid pre-existing state must not mutate siblings"

STALE_KEY_ROOT="${WORK_ROOT}/stale-key-root"
mkdir -p "${STALE_KEY_ROOT}/.substrate-lifecycle-v1/publisher"
printf 'preserve-stale-key\n' >"${STALE_KEY_ROOT}/sibling"
openssl genpkey -algorithm ed25519 -out "${STALE_KEY_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1" >/dev/null 2>&1
run_expect_fail "${WORK_ROOT}/bootstrap-stale-key.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${STALE_KEY_ROOT}" <"${BOOTSTRAP_AUTH}"
assert_contains \
  "publisher_expected_absent rejects pre-existing linux managed residue" \
  "${WORK_ROOT}/bootstrap-stale-key.stderr" \
  "bootstrap must reject stale signing key residue"
assert_eq "preserve-stale-key" "$(<"${STALE_KEY_ROOT}/sibling")" "stale signing-key residue must not mutate siblings"

FORGED_EVIDENCE_ROOT="${WORK_ROOT}/forged-evidence-root"
mkdir -p "${FORGED_EVIDENCE_ROOT}"
printf 'preserve-forged-evidence\n' >"${FORGED_EVIDENCE_ROOT}/sibling"
cp "${BOOTSTRAP_AUTH}" "${WORK_ROOT}/bootstrap-forged-evidence.json"
set_json_field "${WORK_ROOT}/bootstrap-forged-evidence.json" "executor_build_evidence.artifact_sha256" '"ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"'
run_expect_fail "${WORK_ROOT}/bootstrap-forged-evidence.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${FORGED_EVIDENCE_ROOT}" <"${WORK_ROOT}/bootstrap-forged-evidence.json"
assert_contains \
  "executor build evidence artifact_sha256 does not match the running executor" \
  "${WORK_ROOT}/bootstrap-forged-evidence.stderr" \
  "bootstrap must reject forged executor artifact digests"
assert_eq "preserve-forged-evidence" "$(<"${FORGED_EVIDENCE_ROOT}/sibling")" "forged build evidence must not mutate siblings"

SYMLINK_REAL_PARENT="${WORK_ROOT}/symlink-real-parent"
SYMLINK_PARENT_LINK="${WORK_ROOT}/symlink-parent-link"
mkdir -p "${SYMLINK_REAL_PARENT}"
ln -s "${SYMLINK_REAL_PARENT}" "${SYMLINK_PARENT_LINK}"
run_expect_fail "${WORK_ROOT}/bootstrap-symlink.stderr" \
  "${BINARY}" bootstrap-publisher --state-root "${SYMLINK_PARENT_LINK}/nested-state-root" <"${BOOTSTRAP_AUTH}"
assert_contains \
  "symlinked path component is not allowed" \
  "${WORK_ROOT}/bootstrap-symlink.stderr" \
  "bootstrap must reject symlinked ancestor state roots"
assert_not_exists "${SYMLINK_REAL_PARENT}/nested-state-root" \
  "symlinked ancestor bootstrap must not mutate the symlink target"

PUBLISHER_ROOT="${WORK_ROOT}/publisher-root"
PUBLISHER_STATE="${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/current-anchor.v1.json"
mkdir -p "${PUBLISHER_ROOT}"

python3 "${HELPER_PY}" bootstrap-auth \
  "${WORK_ROOT}/publisher-auth.json" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3aa0" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3aa1" \
  "${BINARY_SHA}" \
  "${BINARY}"
"${BINARY}" bootstrap-publisher --state-root "${PUBLISHER_ROOT}" <"${WORK_ROOT}/publisher-auth.json" >/dev/null

set_unit_state "substrate-lifecycle-publisher-v1.service" "enabled" "active"
set_unit_state "substrate-lifecycle-publisher-v1.socket" "enabled" "active"
set_unit_state "substrate-world-service.service" "enabled" "active"
set_unit_state "substrate-world-service.socket" "enabled" "active"

PUBLISHER_SOCKET="${WORK_ROOT}/publisher.sock"
PUBLISHER_PID="$(start_publisher "${PUBLISHER_ROOT}" "${PUBLISHER_SOCKET}" "none" "${WORK_ROOT}/publisher.log")"
CLEANUP_PIDS+=("${PUBLISHER_PID}")

log "Checking prepared record, receipt publication, exact retry, and restore joins"
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-stop.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "stop" \
  "attempt-stop-service" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
STOP_ATTEMPT_HASH="$(sha256_text "attempt-stop-service")"
STOP_RESPONSE_CACHE="${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/responses/${STOP_ATTEMPT_HASH}.json"
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-stop.json" >"${WORK_ROOT}/response-stop.json"
[[ -f "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/prepared/${STOP_ATTEMPT_HASH}.json" ]] \
  || fail "prepared record missing for service stop"
STOP_RECEIPT_PATH="$(json_get "${WORK_ROOT}/response-stop.json" "receipt_path")"
[[ -f "${STOP_RECEIPT_PATH}" ]] || fail "receipt missing for service stop"
assert_eq "1" "$(json_get "${WORK_ROOT}/response-stop.json" "counter")" "service stop counter"
[[ "${STOP_RECEIPT_PATH}" == */receipts/1/receipt.*.json ]] \
  || fail "service stop receipt path must be canonical"
assert_eq "inactive" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.active")" "service stop must drive systemctl inactive"

"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-stop.json" >"${WORK_ROOT}/response-stop-retry.json"
cmp -s "${WORK_ROOT}/response-stop.json" "${WORK_ROOT}/response-stop-retry.json" \
  || fail "exact retry must return the cached publisher response"
cp "${WORK_ROOT}/request-stop.json" "${WORK_ROOT}/request-stop-mismatch.json"
set_json_field "${WORK_ROOT}/request-stop-mismatch.json" "action" '"start"'
run_expect_fail "${WORK_ROOT}/request-stop-mismatch.stderr" \
  "${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-stop-mismatch.json"
assert_contains \
  "cached response request_sha256 does not match the supplied request" \
  "${WORK_ROOT}/publisher.log" \
  "reused attempt_nonce must reject changed request bytes"
PUBLISHER_PID="$(start_publisher "${PUBLISHER_ROOT}" "${PUBLISHER_SOCKET}" "none" "${WORK_ROOT}/publisher-restarted.log")"
rm -f -- "${STOP_RESPONSE_CACHE}"
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-stop.json" >"${WORK_ROOT}/response-stop-reconstructed.json"
cmp -s "${WORK_ROOT}/response-stop.json" "${WORK_ROOT}/response-stop-reconstructed.json" \
  || fail "missing response cache must reconstruct the committed publisher response exactly"

python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-restore.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "restore" \
  "attempt-restore-service" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  '{"restore_state":{"active":"active","enabled":"enabled"}}'
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-restore.json" >"${WORK_ROOT}/response-restore.json"
assert_eq "active" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.active")" "restore must drive service back to active"
assert_eq "2" "$(json_get "${WORK_ROOT}/response-restore.json" "counter")" "service restore counter"
RESTORE_RECEIPT_PATH="$(json_get "${WORK_ROOT}/response-restore.json" "receipt_path")"
[[ -f "${RESTORE_RECEIPT_PATH}" ]] || fail "receipt missing for service restore"
[[ "${RESTORE_RECEIPT_PATH}" == */receipts/1/receipt.*.json ]] \
  || fail "service restore receipt path must be canonical"

python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-socket-stop.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(socket)" \
  "stop" \
  "attempt-stop-socket" \
  "/run/substrate-lifecycle-publisher-v1.sock" \
  "-"
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-socket-stop.json" >"${WORK_ROOT}/response-socket-stop.json"
assert_eq "3" "$(json_get "${WORK_ROOT}/response-socket-stop.json" "counter")" "socket stop counter"
SOCKET_STOP_RECEIPT="$(json_get "${WORK_ROOT}/response-socket-stop.json" "receipt_path")"
[[ -f "${SOCKET_STOP_RECEIPT}" ]] || fail "receipt missing for socket stop"
[[ "${SOCKET_STOP_RECEIPT}" == */receipts/1/receipt.*.json ]] \
  || fail "socket stop receipt path must be canonical"
assert_eq "socket" "$(json_get "${SOCKET_STOP_RECEIPT}" "effect_observation.observation.before.endpoint.kind")" "socket stop must observe the publisher endpoint"
assert_eq "socket" "$(json_get "${SOCKET_STOP_RECEIPT}" "effect_observation.observation.after.endpoint.kind")" "socket stop must keep endpoint identity"

python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-socket-restore.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(socket)" \
  "restore" \
  "attempt-restore-socket" \
  "/run/substrate-lifecycle-publisher-v1.sock" \
  '{"restore_state":{"active":"active","enabled":"enabled"}}'
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-socket-restore.json" >"${WORK_ROOT}/response-socket-restore.json"
assert_eq "4" "$(json_get "${WORK_ROOT}/response-socket-restore.json" "counter")" "socket restore counter"
SOCKET_RESTORE_RECEIPT="$(json_get "${WORK_ROOT}/response-socket-restore.json" "receipt_path")"
[[ -f "${SOCKET_RESTORE_RECEIPT}" ]] || fail "receipt missing for socket restore"
[[ "${SOCKET_RESTORE_RECEIPT}" == */receipts/1/receipt.*.json ]] \
  || fail "socket restore receipt path must be canonical"
assert_eq "socket" "$(json_get "${SOCKET_RESTORE_RECEIPT}" "effect_observation.before.endpoint.kind")" "socket restore must observe the publisher endpoint"
assert_eq "socket" "$(json_get "${SOCKET_RESTORE_RECEIPT}" "effect_observation.after.endpoint.kind")" "socket restore must keep endpoint identity"

log "Checking relay attestation and response publication"
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-relay.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "start" \
  "attempt-relay-service" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
python3 "${HELPER_PY}" relay \
  "${WORK_ROOT}/relay-response.json" \
  "${WORK_ROOT}/relay-meta.json" \
  "${BINARY}" \
  "${PUBLISHER_ROOT}" \
  "${PUBLISHER_SOCKET}" \
  "${WORK_ROOT}/request-relay.json" \
  "stream"
assert_eq "$(json_get "${WORK_ROOT}/relay-meta.json" "caller_pid")" \
  "$(json_get "${WORK_ROOT}/relay-response.json" "relay_attestation.caller_pid")" \
  "relay caller PID"
assert_eq "$(json_get "${WORK_ROOT}/relay-meta.json" "caller_exe")" \
  "$(json_get "${WORK_ROOT}/relay-response.json" "relay_attestation.caller_exe")" \
  "relay caller executable"
assert_eq "5" "$(json_get "${WORK_ROOT}/relay-response.json" "counter")" "relay request counter"

log "Checking observe kill-point retry join"
stop_publisher "${PUBLISHER_PID}"
KILL_SOCKET="${WORK_ROOT}/publisher-kill.sock"
KILL_PID="$(start_publisher "${PUBLISHER_ROOT}" "${KILL_SOCKET}" "observe" "${WORK_ROOT}/publisher-kill.log")"
CLEANUP_PIDS+=("${KILL_PID}")
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-kill.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "start" \
  "attempt-kill-observe" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
KILL_ATTEMPT_HASH="$(sha256_text "attempt-kill-observe")"
run_expect_fail "${WORK_ROOT}/request-kill.stderr" \
  "${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${KILL_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-kill.json"
wait "${KILL_PID}" 2>/dev/null || true
[[ -f "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/prepared/${KILL_ATTEMPT_HASH}.json" ]] \
  || fail "prepared record missing after observe kill-point"
KILL_RECEIPT_RELATIVE="$(json_get "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/prepared/${KILL_ATTEMPT_HASH}.json" "receipt_relative_path")"
assert_not_exists "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/${KILL_RECEIPT_RELATIVE}" \
  "observe kill-point must not publish a receipt"

RETRY_PID="$(start_publisher "${PUBLISHER_ROOT}" "${KILL_SOCKET}" "none" "${WORK_ROOT}/publisher-retry.log")"
CLEANUP_PIDS+=("${RETRY_PID}")
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${KILL_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-kill.json" >"${WORK_ROOT}/response-kill-retry.json"
KILL_RETRY_RECEIPT="$(json_get "${WORK_ROOT}/response-kill-retry.json" "receipt_path")"
[[ -f "${KILL_RETRY_RECEIPT}" ]] || fail "retry after observe kill-point must publish the receipt"
assert_eq "6" "$(json_get "${WORK_ROOT}/response-kill-retry.json" "counter")" "observe kill-point retry counter"
stop_publisher "${RETRY_PID}"

log "Checking receipt kill-point retry join"
RECEIPT_SOCKET="${WORK_ROOT}/publisher-receipt.sock"
RECEIPT_PID="$(start_publisher "${PUBLISHER_ROOT}" "${RECEIPT_SOCKET}" "receipt" "${WORK_ROOT}/publisher-receipt.log")"
CLEANUP_PIDS+=("${RECEIPT_PID}")
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-receipt-kill.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "stop" \
  "attempt-kill-receipt" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
RECEIPT_ATTEMPT_HASH="$(sha256_text "attempt-kill-receipt")"
run_expect_fail "${WORK_ROOT}/request-receipt-kill.stderr" \
  "${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${RECEIPT_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-receipt-kill.json"
wait "${RECEIPT_PID}" 2>/dev/null || true
RECEIPT_KILL_RELATIVE="$(json_get "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/prepared/${RECEIPT_ATTEMPT_HASH}.json" "receipt_relative_path")"
RECEIPT_KILL_PATH="${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/${RECEIPT_KILL_RELATIVE}"
[[ -f "${RECEIPT_KILL_PATH}" ]] || fail "receipt kill-point must durably publish the receipt before exit"
RECEIPT_RETRY_PID="$(start_publisher "${PUBLISHER_ROOT}" "${RECEIPT_SOCKET}" "none" "${WORK_ROOT}/publisher-receipt-retry.log")"
CLEANUP_PIDS+=("${RECEIPT_RETRY_PID}")
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${RECEIPT_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-receipt-kill.json" >"${WORK_ROOT}/response-receipt-kill-retry.json"
assert_eq "7" "$(json_get "${WORK_ROOT}/response-receipt-kill-retry.json" "counter")" "receipt kill-point retry counter"
assert_eq "${RECEIPT_KILL_PATH}" "$(json_get "${WORK_ROOT}/response-receipt-kill-retry.json" "receipt_path")" \
  "receipt kill-point retry must join the published receipt exactly"
assert_eq "inactive" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.active")" \
  "receipt kill-point retry must preserve the stopped service state"
stop_publisher "${RECEIPT_RETRY_PID}"

log "Checking socket-activated seqpacket relay path"
SEQPACKET_SOCKET="${WORK_ROOT}/publisher-seqpacket.sock"
SEQPACKET_PID="$(start_seqpacket_publisher "${PUBLISHER_ROOT}" "${SEQPACKET_SOCKET}" "none" "${WORK_ROOT}/publisher-seqpacket.log")"
CLEANUP_PIDS+=("${SEQPACKET_PID}")
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-seqpacket.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "stop" \
  "attempt-relay-seqpacket" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
python3 "${HELPER_PY}" set-repeat-string \
  "${WORK_ROOT}/request-seqpacket.json" \
  "attempt_nonce" \
  "attempt-relay-seqpacket-" \
  "x" \
  "$((128 * 1024))"
python3 "${HELPER_PY}" relay \
  "${WORK_ROOT}/relay-seqpacket-response.json" \
  "${WORK_ROOT}/relay-seqpacket-meta.json" \
  "${BINARY}" \
  "${PUBLISHER_ROOT}" \
  "${SEQPACKET_SOCKET}" \
  "${WORK_ROOT}/request-seqpacket.json" \
  "seqpacket"
assert_eq "$(json_get "${WORK_ROOT}/relay-seqpacket-meta.json" "caller_pid")" \
  "$(json_get "${WORK_ROOT}/relay-seqpacket-response.json" "relay_attestation.caller_pid")" \
  "seqpacket relay caller PID"
assert_eq "$(python3 "${HELPER_PY}" request-sha "${WORK_ROOT}/request-seqpacket.json")" \
  "$(json_get "${WORK_ROOT}/relay-seqpacket-response.json" "request_sha256")" \
  "seqpacket relay request digest"
assert_eq "8" "$(json_get "${WORK_ROOT}/relay-seqpacket-response.json" "counter")" "seqpacket relay request counter"
assert_eq "inactive" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.active")" "seqpacket relay must drive systemctl inactive"

log "Checking live seqpacket service start preserves the published socket"
set_unit_state "substrate-lifecycle-publisher-v1.service" "enabled" "inactive"
SEQPACKET_INODE_BEFORE="$(stat -Lc '%i' "${SEQPACKET_SOCKET}")"
"${BINARY}" service-state \
  --state-root "${PUBLISHER_ROOT}" \
  --socket "${SEQPACKET_SOCKET}" \
  --transport seqpacket \
  --service-unit "substrate-lifecycle-publisher-v1.service" \
  --action start >"${WORK_ROOT}/service-state-live-seqpacket.json"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-live-seqpacket.json" "observation.before.active")" "live seqpacket start must observe inactive service state before activation"
assert_eq "socket" "$(json_get "${WORK_ROOT}/service-state-live-seqpacket.json" "observation.before.endpoint.kind")" "live seqpacket start must observe the published endpoint"
assert_eq "active" "$(json_get "${WORK_ROOT}/service-state-live-seqpacket.json" "observation.after.active")" "live seqpacket start must restore active service state"
assert_eq "${SEQPACKET_INODE_BEFORE}" "$(stat -Lc '%i' "${SEQPACKET_SOCKET}")" "live seqpacket service start must preserve the published socket inode"

run_expect_fail "${WORK_ROOT}/direct-seqpacket-collision.stderr" \
  "${BINARY}" run-publisher --state-root "${PUBLISHER_ROOT}" --socket "${SEQPACKET_SOCKET}" --transport seqpacket --kill-at none
assert_contains "publisher endpoint already has a live listener" "${WORK_ROOT}/direct-seqpacket-collision.stderr" \
  "direct seqpacket bind must fail closed when the published socket is already live"
assert_eq "${SEQPACKET_INODE_BEFORE}" "$(stat -Lc '%i' "${SEQPACKET_SOCKET}")" "failed direct seqpacket bind must not replace the published socket"
stop_publisher "${SEQPACKET_PID}"

log "Checking direct seqpacket publisher start path"
DIRECT_SEQPACKET_SOCKET="${WORK_ROOT}/publisher-direct-seqpacket.sock"
DIRECT_SEQPACKET_PID="$(start_direct_seqpacket_publisher "${PUBLISHER_ROOT}" "${DIRECT_SEQPACKET_SOCKET}" "none" "${WORK_ROOT}/publisher-direct-seqpacket.log")"
CLEANUP_PIDS+=("${DIRECT_SEQPACKET_PID}")
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-direct-seqpacket.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "start" \
  "attempt-direct-seqpacket" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${DIRECT_SEQPACKET_SOCKET}" --transport seqpacket \
  <"${WORK_ROOT}/request-direct-seqpacket.json" >"${WORK_ROOT}/response-direct-seqpacket.json"
assert_eq "9" "$(json_get "${WORK_ROOT}/response-direct-seqpacket.json" "counter")" "direct seqpacket request counter"
assert_eq "active" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.active")" "direct seqpacket publisher must support service start without socket activation"
stop_publisher "${DIRECT_SEQPACKET_PID}"
PUBLISHER_PID="$(start_publisher "${PUBLISHER_ROOT}" "${PUBLISHER_SOCKET}" "none" "${WORK_ROOT}/publisher-resumed.log")"
CLEANUP_PIDS+=("${PUBLISHER_PID}")

log "Checking request-derived path confinement"
TRAVERSAL_NONCE="../escape"
TRAVERSAL_HASH="$(printf '%s' "${TRAVERSAL_NONCE}" | sha256sum | awk '{print $1}')"
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-traversal.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "start" \
  "${TRAVERSAL_NONCE}" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  "-"
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-traversal.json" >"${WORK_ROOT}/response-traversal.json"
[[ -f "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/prepared/${TRAVERSAL_HASH}.json" ]] \
  || fail "traversal nonce prepared record must stay under the prepared directory"
[[ -f "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/responses/${TRAVERSAL_HASH}.json" ]] \
  || fail "traversal nonce response cache must stay under the responses directory"
assert_not_exists "${PUBLISHER_ROOT}/.substrate-lifecycle-v1/publisher/escape.json" \
  "traversal nonce must not escape into the publisher root"
assert_eq "10" "$(json_get "${WORK_ROOT}/response-traversal.json" "counter")" "traversal request counter"

log "Checking exact runtime and masked restore states"
python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-runtime-enable.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "restore" \
  "attempt-runtime-enable" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  '{"restore_state":{"active":"inactive","enabled":"enabled-runtime"}}'
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-runtime-enable.json" >"${WORK_ROOT}/response-runtime-enable.json"
RUNTIME_ENABLE_RECEIPT="$(json_get "${WORK_ROOT}/response-runtime-enable.json" "receipt_path")"
assert_eq "enabled-runtime" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.enabled")" "runtime enable restore must preserve runtime-only enablement"
assert_eq "enabled-runtime" "$(json_get "${RUNTIME_ENABLE_RECEIPT}" "effect_observation.after.enabled")" "runtime enable restore observation"
assert_eq "11" "$(json_get "${WORK_ROOT}/response-runtime-enable.json" "counter")" "runtime enable restore counter"

python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-masked.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "restore" \
  "attempt-masked-service" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  '{"restore_state":{"active":"inactive","enabled":"masked"}}'
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-masked.json" >"${WORK_ROOT}/response-masked.json"
MASKED_RECEIPT="$(json_get "${WORK_ROOT}/response-masked.json" "receipt_path")"
assert_eq "masked" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.enabled")" "masked restore must preserve masked enablement"
assert_eq "masked" "$(json_get "${MASKED_RECEIPT}" "effect_observation.after.enabled")" "masked restore observation"
assert_eq "12" "$(json_get "${WORK_ROOT}/response-masked.json" "counter")" "masked restore counter"

python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-masked-active.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "restore" \
  "attempt-masked-active-service" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  '{"restore_state":{"active":"active","enabled":"masked"}}'
"${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-masked-active.json" >"${WORK_ROOT}/response-masked-active.json"
MASKED_ACTIVE_RECEIPT="$(json_get "${WORK_ROOT}/response-masked-active.json" "receipt_path")"
assert_eq "masked" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.enabled")" "masked active restore must preserve masked enablement"
assert_eq "active" "$(<"${SYSTEMCTL_STATE_DIR}/substrate-lifecycle-publisher-v1.service.active")" "masked active restore must preserve active service state"
assert_eq "masked" "$(json_get "${MASKED_ACTIVE_RECEIPT}" "effect_observation.after.enabled")" "masked active restore enabled observation"
assert_eq "active" "$(json_get "${MASKED_ACTIVE_RECEIPT}" "effect_observation.after.active")" "masked active restore active observation"
assert_eq "13" "$(json_get "${WORK_ROOT}/response-masked-active.json" "counter")" "masked active restore counter"

python3 "${HELPER_PY}" request \
  "${WORK_ROOT}/request-failed-restore.json" \
  "${PUBLISHER_STATE}" \
  "linux.publisher.service-state(service)" \
  "restore" \
  "attempt-failed-restore" \
  "/etc/systemd/system/substrate-lifecycle-publisher-v1.service" \
  '{"restore_state":{"active":"failed","enabled":"disabled"}}'
run_expect_fail "${WORK_ROOT}/request-failed-restore.stderr" \
  "${BINARY}" submit-request --state-root "${PUBLISHER_ROOT}" --socket "${PUBLISHER_SOCKET}" --transport stream \
  <"${WORK_ROOT}/request-failed-restore.json"
assert_contains "unsupported exact restore_state.active value failed" "${WORK_ROOT}/publisher-resumed.log" \
  "failed restore targets must stop preserving instead of collapsing to inactive"

log "Checking endpoint non-requestability and zero-change observations"
set_unit_state "substrate-lifecycle-publisher-v1.service" "disabled" "inactive"
MISSING_SOCKET="${WORK_ROOT}/missing.sock"
"${BINARY}" service-state \
  --state-root "${PUBLISHER_ROOT}" \
  --socket "${MISSING_SOCKET}" \
  --service-unit "substrate-lifecycle-publisher-v1.service" \
  --action restore >"${WORK_ROOT}/service-state-missing.json"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-missing.json" "observation.before.active")" "missing endpoint must preserve inactive service state"
assert_eq "disabled" "$(json_get "${WORK_ROOT}/service-state-missing.json" "observation.before.enabled")" "missing endpoint must preserve disabled enablement"
assert_eq "missing" "$(json_get "${WORK_ROOT}/service-state-missing.json" "observation.before.endpoint.kind")" "missing endpoint kind"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-missing.json" "observation.after.active")" "missing endpoint must preserve inactive service state after restore"
assert_eq "disabled" "$(json_get "${WORK_ROOT}/service-state-missing.json" "observation.after.enabled")" "missing endpoint must preserve disabled enablement after restore"
assert_eq "missing" "$(json_get "${WORK_ROOT}/service-state-missing.json" "observation.after.endpoint.kind")" "missing endpoint must remain unchanged"

REPLACED_SOCKET="${WORK_ROOT}/replaced.sock"
printf 'not-a-socket\n' >"${REPLACED_SOCKET}"
"${BINARY}" service-state \
  --state-root "${PUBLISHER_ROOT}" \
  --socket "${REPLACED_SOCKET}" \
  --service-unit "substrate-lifecycle-publisher-v1.service" \
  --action restore >"${WORK_ROOT}/service-state-file.json"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-file.json" "observation.before.active")" "replaced endpoint must preserve inactive service state"
assert_eq "disabled" "$(json_get "${WORK_ROOT}/service-state-file.json" "observation.before.enabled")" "replaced endpoint must preserve disabled enablement"
assert_eq "file" "$(json_get "${WORK_ROOT}/service-state-file.json" "observation.before.endpoint.kind")" "replaced endpoint kind"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-file.json" "observation.after.active")" "replaced endpoint must preserve inactive service state after restore"
assert_eq "disabled" "$(json_get "${WORK_ROOT}/service-state-file.json" "observation.after.enabled")" "replaced endpoint must preserve disabled enablement after restore"
assert_eq "file" "$(json_get "${WORK_ROOT}/service-state-file.json" "observation.after.endpoint.kind")" "replaced endpoint must remain unchanged"

mkdir -p "${WORK_ROOT}/endpoint-dir"
"${BINARY}" service-state \
  --state-root "${PUBLISHER_ROOT}" \
  --socket "${WORK_ROOT}/endpoint-dir" \
  --service-unit "substrate-lifecycle-publisher-v1.service" \
  --action restore >"${WORK_ROOT}/service-state-dir.json"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-dir.json" "observation.before.active")" "directory endpoint must preserve inactive service state"
assert_eq "disabled" "$(json_get "${WORK_ROOT}/service-state-dir.json" "observation.before.enabled")" "directory endpoint must preserve disabled enablement"
assert_eq "directory" "$(json_get "${WORK_ROOT}/service-state-dir.json" "observation.before.endpoint.kind")" "directory endpoint kind"
assert_eq "inactive" "$(json_get "${WORK_ROOT}/service-state-dir.json" "observation.after.active")" "directory endpoint must preserve inactive service state after restore"
assert_eq "disabled" "$(json_get "${WORK_ROOT}/service-state-dir.json" "observation.after.enabled")" "directory endpoint must preserve disabled enablement after restore"
assert_eq "directory" "$(json_get "${WORK_ROOT}/service-state-dir.json" "observation.after.endpoint.kind")" "directory endpoint must remain unchanged"

log "Checking guest bootstrap challenge, transcript, and replay negatives"
GUEST_ROOT="${WORK_ROOT}/guest-root"
mkdir -p "${GUEST_ROOT}"
chmod 0750 "${GUEST_ROOT}"
python3 "${HELPER_PY}" bootstrap-auth \
  "${WORK_ROOT}/guest-auth.json" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3ab0" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3ab1" \
  "${BINARY_SHA}" \
  "${BINARY}"
"${BINARY}" bootstrap-publisher --state-root "${GUEST_ROOT}" <"${WORK_ROOT}/guest-auth.json" >/dev/null
GUEST_STATE="${GUEST_ROOT}/.substrate-lifecycle-v1/publisher/current-anchor.v1.json"
GUEST_BOOTSTRAP_STATE_SHA="$(json_sha "${GUEST_STATE}")"
GUEST_BOOTSTRAP_SIGNING_KEY="${GUEST_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1"
GUEST_BOOTSTRAP_SIGNING_KEY_SHA="$(sha256sum "${GUEST_BOOTSTRAP_SIGNING_KEY}" | awk '{print $1}')"
assert_eq "750" "$(stat -c '%a' "${GUEST_ROOT}")" "guest bootstrap must preserve pre-existing state root mode"

openssl genpkey -algorithm EC -pkeyopt ec_paramgen_curve:P-256 -out "${WORK_ROOT}/host-key.pem" >/dev/null 2>&1
openssl pkey -in "${WORK_ROOT}/host-key.pem" -pubout -outform DER >"${WORK_ROOT}/host-spki.der" 2>/dev/null

python3 "${HELPER_PY}" ticket \
  "${WORK_ROOT}/pairing-ticket.json" \
  "${GUEST_STATE}" \
  "${WORK_ROOT}/host-key.pem" \
  "${WORK_ROOT}/host-spki.der" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99"
python3 "${HELPER_PY}" ticket \
  "${WORK_ROOT}/pairing-ticket-expired.json" \
  "${GUEST_STATE}" \
  "${WORK_ROOT}/host-key.pem" \
  "${WORK_ROOT}/host-spki.der" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a98" \
  "1"
python3 "${HELPER_PY}" ticket \
  "${WORK_ROOT}/pairing-ticket-intent-retry.json" \
  "${GUEST_STATE}" \
  "${WORK_ROOT}/host-key.pem" \
  "${WORK_ROOT}/host-spki.der" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a97"
python3 "${HELPER_PY}" ticket \
  "${WORK_ROOT}/pairing-ticket-hello-retry.json" \
  "${GUEST_STATE}" \
  "${WORK_ROOT}/host-key.pem" \
  "${WORK_ROOT}/host-spki.der" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a96"
SHORT_EXPIRY_UNIX_NS="$(($(date +%s%N) + 5000000000))"
python3 "${HELPER_PY}" ticket \
  "${WORK_ROOT}/pairing-ticket-hello-expired-retry.json" \
  "${GUEST_STATE}" \
  "${WORK_ROOT}/host-key.pem" \
  "${WORK_ROOT}/host-spki.der" \
  "018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a95" \
  "${SHORT_EXPIRY_UNIX_NS}"

printf 'not-a-tty\n' >"${WORK_ROOT}/not-a-tty"
run_expect_fail "${WORK_ROOT}/begin-non-tty.stderr" \
  "${BINARY}" begin-guest-bootstrap \
    --state-root "${GUEST_ROOT}" \
    --tty-path "${WORK_ROOT}/not-a-tty" \
    --ticket-file "${WORK_ROOT}/pairing-ticket.json"
assert_contains "not a controlling tty" "${WORK_ROOT}/begin-non-tty.stderr" "non-tty path must be rejected"
run_expect_fail "${WORK_ROOT}/begin-expired-ticket.stderr" \
  python3 "${HELPER_PY}" begin-pty \
    "${WORK_ROOT}/guest-hello-expired.json" \
    "${WORK_ROOT}/guest-prompt-expired.txt" \
    "${BINARY}" \
    "${GUEST_ROOT}" \
    "${WORK_ROOT}/pairing-ticket-expired.json"
assert_contains "guest pairing ticket expired" "${WORK_ROOT}/begin-expired-ticket.stderr" "expired ticket must be rejected before guest intent publication"

run_expect_fail "${WORK_ROOT}/begin-intent-kill.stderr" \
  python3 "${HELPER_PY}" begin-pty \
    "${WORK_ROOT}/guest-hello-intent-kill.json" \
    "${WORK_ROOT}/guest-prompt-intent-kill.txt" \
    "${BINARY}" \
    "${GUEST_ROOT}" \
    "${WORK_ROOT}/pairing-ticket-intent-retry.json" \
    "intent"
INTENT_RETRY_PATH="${GUEST_ROOT}/.substrate-lifecycle-pairing-intent-v1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a97.json"
assert_eq "PairingIntentDurable" "$(json_get "${INTENT_RETRY_PATH}" "state")" \
  "guest intent kill-point must leave a durable intent before retry"
assert_not_exists "${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a97/hello.json" \
  "guest intent kill-point must not publish hello before retry"
python3 "${HELPER_PY}" begin-pty \
  "${WORK_ROOT}/guest-hello-intent-retry.json" \
  "${WORK_ROOT}/guest-prompt-intent-retry.txt" \
  "${BINARY}" \
  "${GUEST_ROOT}" \
  "${WORK_ROOT}/pairing-ticket-intent-retry.json" \
  "none" \
  "forbid-prompt"
assert_eq "HelloDurable" "$(json_get "${INTENT_RETRY_PATH}" "state")" \
  "guest intent retry must promote the durable intent to HelloDurable"

run_expect_fail "${WORK_ROOT}/begin-hello-kill.stderr" \
  python3 "${HELPER_PY}" begin-pty \
    "${WORK_ROOT}/guest-hello-hello-kill.json" \
    "${WORK_ROOT}/guest-prompt-hello-kill.txt" \
    "${BINARY}" \
    "${GUEST_ROOT}" \
    "${WORK_ROOT}/pairing-ticket-hello-retry.json" \
    "hello"
HELLO_RETRY_PATH="${GUEST_ROOT}/.substrate-lifecycle-pairing-intent-v1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a96.json"
HELLO_RETRY_FILE="${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a96/hello.json"
assert_eq "HelloDurable" "$(json_get "${HELLO_RETRY_PATH}" "state")" \
  "guest hello kill-point must leave a durable hello before retry"
[[ -f "${HELLO_RETRY_FILE}" ]] || fail "guest hello kill-point must preserve the durable hello file"
python3 "${HELPER_PY}" begin-pty \
  "${WORK_ROOT}/guest-hello-hello-retry.json" \
  "${WORK_ROOT}/guest-prompt-hello-retry.txt" \
  "${BINARY}" \
  "${GUEST_ROOT}" \
  "${WORK_ROOT}/pairing-ticket-hello-retry.json" \
  "none" \
  "forbid-prompt"
assert_eq "$(json_sha "${HELLO_RETRY_FILE}")" "$(json_sha "${WORK_ROOT}/guest-hello-hello-retry.json")" \
  "guest hello retry must reuse the exact durable hello payload"

run_expect_fail "${WORK_ROOT}/begin-hello-expired-kill.stderr" \
  python3 "${HELPER_PY}" begin-pty \
    "${WORK_ROOT}/guest-hello-expired-kill.json" \
    "${WORK_ROOT}/guest-prompt-expired-kill.txt" \
    "${BINARY}" \
    "${GUEST_ROOT}" \
    "${WORK_ROOT}/pairing-ticket-hello-expired-retry.json" \
    "hello"
HELLO_EXPIRED_RETRY_FILE="${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a95/hello.json"
[[ -f "${HELLO_EXPIRED_RETRY_FILE}" ]] || fail "guest hello expiry retry kill-point must preserve the durable hello file"
sleep 6
python3 "${HELPER_PY}" begin-pty \
  "${WORK_ROOT}/guest-hello-expired-retry.json" \
  "${WORK_ROOT}/guest-prompt-expired-retry.txt" \
  "${BINARY}" \
  "${GUEST_ROOT}" \
  "${WORK_ROOT}/pairing-ticket-hello-expired-retry.json" \
  "none" \
  "forbid-prompt"
assert_eq "$(json_sha "${HELLO_EXPIRED_RETRY_FILE}")" "$(json_sha "${WORK_ROOT}/guest-hello-expired-retry.json")" \
  "expired ticket retry must reuse the exact durable hello payload without re-prompting"

python3 "${HELPER_PY}" transcript \
  "${WORK_ROOT}/guest-transcript-intent-retry.json" \
  "${WORK_ROOT}/pairing-ticket-intent-retry.json" \
  "${WORK_ROOT}/guest-hello-intent-retry.json" \
  "${WORK_ROOT}/host-key.pem"
INTENT_RETRY_GUEST_KEY="${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a97/guest-signing-key.v1"
openssl genpkey -algorithm ed25519 -out "${INTENT_RETRY_GUEST_KEY}" >/dev/null 2>&1
run_expect_fail "${WORK_ROOT}/commit-intent-retry-key-replaced.stderr" \
  "${BINARY}" commit-guest-bootstrap \
    --state-root "${GUEST_ROOT}" \
    --ticket-file "${WORK_ROOT}/pairing-ticket-intent-retry.json" \
    --transcript-file "${WORK_ROOT}/guest-transcript-intent-retry.json"
assert_contains "guest signing key does not match the durable guest pairing intent" \
  "${WORK_ROOT}/commit-intent-retry-key-replaced.stderr" \
  "guest commit must reject a replaced durable guest signing key before final publication"
assert_eq "${GUEST_BOOTSTRAP_SIGNING_KEY_SHA}" "$(sha256sum "${GUEST_BOOTSTRAP_SIGNING_KEY}" | awk '{print $1}')" \
  "guest commit must preserve the bootstrap signing key when the durable guest signing key was replaced"
assert_eq "${GUEST_BOOTSTRAP_STATE_SHA}" "$(json_sha "${GUEST_STATE}")" \
  "guest commit must preserve the bootstrap anchor when the durable guest signing key was replaced"

python3 "${HELPER_PY}" begin-pty \
  "${WORK_ROOT}/guest-hello.json" \
  "${WORK_ROOT}/guest-prompt.txt" \
  "${BINARY}" \
  "${GUEST_ROOT}" \
  "${WORK_ROOT}/pairing-ticket.json"
GUEST_INTENT_PATH="${GUEST_ROOT}/.substrate-lifecycle-pairing-intent-v1.018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99.json"
assert_eq "750" "$(stat -c '%a' "${GUEST_ROOT}")" "guest pairing intent publication must preserve state root mode"
assert_contains "HOST_KEY_FINGERPRINT_SHA256" "${WORK_ROOT}/guest-prompt.txt" "pairing prompt must print fingerprint"
assert_contains "CHALLENGE" "${WORK_ROOT}/guest-prompt.txt" "pairing prompt must print challenge"
assert_contains "PAIR EXACT SUBSTRATE GUEST PUBLISHER" "${WORK_ROOT}/guest-prompt.txt" "pairing prompt must print the literal confirmation"
assert_eq "HelloDurable" "$(json_get "${GUEST_INTENT_PATH}" "state")" "guest pairing hello must be recorded durably in the intent"
assert_eq "$(json_get "${WORK_ROOT}/guest-hello.json" "guest_public_key")" \
  "$(json_get "${GUEST_INTENT_PATH}" "hello.guest_public_key")" \
  "durable guest intent must retain the signed hello payload"

python3 "${HELPER_PY}" transcript \
  "${WORK_ROOT}/guest-transcript.json" \
  "${WORK_ROOT}/pairing-ticket.json" \
  "${WORK_ROOT}/guest-hello.json" \
  "${WORK_ROOT}/host-key.pem"
cp "${WORK_ROOT}/guest-transcript.json" "${WORK_ROOT}/guest-transcript-bad.json"
set_json_field "${WORK_ROOT}/guest-transcript-bad.json" "guest_nonce" '"wrong-nonce"'

run_expect_fail "${WORK_ROOT}/commit-bad.stderr" \
  "${BINARY}" commit-guest-bootstrap \
    --state-root "${GUEST_ROOT}" \
    --ticket-file "${WORK_ROOT}/pairing-ticket.json" \
    --transcript-file "${WORK_ROOT}/guest-transcript-bad.json"
assert_contains "guest_nonce does not match" "${WORK_ROOT}/commit-bad.stderr" "transcript substitution must be rejected"

run_expect_fail "${WORK_ROOT}/commit-kill.stderr" \
  "${BINARY}" commit-guest-bootstrap \
    --state-root "${GUEST_ROOT}" \
    --ticket-file "${WORK_ROOT}/pairing-ticket.json" \
    --transcript-file "${WORK_ROOT}/guest-transcript.json" \
    --kill-at commit
assert_not_exists "${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99/consumed.marker" \
  "guest commit kill-point must not publish the consumed marker before retry"
cmp -s \
  "${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99/guest-signing-key.v1" \
  "${GUEST_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1" \
  || fail "guest commit kill-point must preserve the exact final signing key for retry"
assert_eq \
  "$(json_get "${WORK_ROOT}/guest-hello.json" "guest_public_key")" \
  "$(json_get "${GUEST_STATE}" "current_anchor.signature.public_key")" \
  "guest commit kill-point must preserve the exact guest-signed anchor for retry"
"${BINARY}" commit-guest-bootstrap \
  --state-root "${GUEST_ROOT}" \
  --ticket-file "${WORK_ROOT}/pairing-ticket.json" \
  --transcript-file "${WORK_ROOT}/guest-transcript.json" >"${WORK_ROOT}/guest-anchor.json"
[[ -f "${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99/consumed.marker" ]] \
  || fail "guest pairing commit must publish a consumed marker"
[[ -f "${GUEST_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1" ]] \
  || fail "guest pairing commit must materialize the final signing key"
cmp -s \
  "${GUEST_ROOT}/.substrate-lifecycle-guest-pairings/018f3e4a-7b2c-7c91-8a6f-2e1d5c4b3a99/guest-signing-key.v1" \
  "${GUEST_ROOT}/.substrate-lifecycle-v1/publisher/signing-key.v1" \
  || fail "guest pairing commit must materialize the exact durable guest signing key at the final publisher path"
assert_eq \
  "$(json_get "${WORK_ROOT}/guest-hello.json" "guest_public_key")" \
  "$(json_get "${WORK_ROOT}/guest-anchor.json" "signature.public_key")" \
  "guest pairing commit must sign the generation-one anchor with the durable guest key"
assert_eq "TranscriptDurable" "$(json_get "${GUEST_INTENT_PATH}" "state")" \
  "guest pairing transcript must be recorded durably in the intent before final key publication"
assert_eq "$(json_get "${WORK_ROOT}/guest-transcript.json" "guest_nonce")" \
  "$(json_get "${GUEST_INTENT_PATH}" "transcript.guest_nonce")" \
  "durable guest intent must retain the committed transcript payload"

run_expect_fail "${WORK_ROOT}/commit-replay.stderr" \
  "${BINARY}" commit-guest-bootstrap \
    --state-root "${GUEST_ROOT}" \
    --ticket-file "${WORK_ROOT}/pairing-ticket.json" \
    --transcript-file "${WORK_ROOT}/guest-transcript.json"
assert_contains "consumed terminal state" "${WORK_ROOT}/commit-replay.stderr" "guest pairing replay must be rejected"

stop_publisher "${PUBLISHER_PID}"

log "All checks passed."
log "Artifacts: ${WORK_ROOT}"
