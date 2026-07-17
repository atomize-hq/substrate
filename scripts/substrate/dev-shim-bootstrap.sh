#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-shim-bootstrap"

log() {
  printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"
}

warn() {
  printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2
}

fatal() {
  printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2
  exit 1
}

resolve_install_bootstrap_context() {
  local declared="$1"
  local raw_prefix="$2"
  local supplied_carrier="$3"
  local context_fd

  exec {context_fd}< <(python3 - "${declared}" "${raw_prefix}" "${supplied_carrier}" <<'PY'
import base64
import hashlib
import os
import pwd
import re
import sys

DOMAIN = "substrate.install_bootstrap_context"
KEYS = (
    "domain", "version", "selected_host_prefix", "host_substrate_home",
    "host_substrate_root", "principal_kind", "principal_account",
    "principal_uid", "host_context_commitment",
)


def fail():
    raise ValueError("invalid install bootstrap context")


def normalize_path(raw):
    if not raw or raw == "/" or not raw.startswith("/") or raw.startswith("//") or "\0" in raw:
        fail()
    parts = []
    for part in raw[1:].split("/"):
        if not part:
            continue
        if part in (".", ".."):
            fail()
        parts.append(part)
    if not parts:
        fail()
    return "/" + "/".join(parts)


def b64_encode(value):
    return base64.urlsafe_b64encode(value).rstrip(b"=")


def b64_decode(value):
    if not value or not re.fullmatch(rb"[A-Za-z0-9_-]+", value):
        fail()
    decoded = base64.urlsafe_b64decode(value + b"=" * ((-len(value)) % 4))
    if b64_encode(decoded) != value:
        fail()
    return decoded.decode("utf-8")


def frame(prefix, account, uid):
    encoded = b64_encode(prefix.encode("utf-8")).decode("ascii")
    return (
        f"domain={DOMAIN}\nversion=1\nselected_host_prefix={encoded}\n"
        f"host_substrate_home={encoded}\nhost_substrate_root={encoded}\n"
        f"principal_kind=unix\nprincipal_account={b64_encode(account.encode('utf-8')).decode('ascii')}\n"
        f"principal_uid={uid}\n"
    ).encode("ascii")


def current_principal():
    uid = os.geteuid()
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    entry = pwd.getpwuid(uid)
    if not entry.pw_name or pwd.getpwnam(entry.pw_name).pw_uid != uid:
        fail()
    if any(ch in entry.pw_name for ch in "\0\r\n"):
        fail()
    return entry, uid


def decode_carrier(encoded):
    raw_encoded = encoded.encode("ascii")
    if not raw_encoded or not re.fullmatch(rb"[A-Za-z0-9_-]+", raw_encoded):
        fail()
    record = base64.urlsafe_b64decode(raw_encoded + b"=" * ((-len(raw_encoded)) % 4))
    if b64_encode(record) != raw_encoded or not record.endswith(b"\n") or b"\r" in record or b"\0" in record:
        fail()
    lines = record[:-1].split(b"\n")
    if len(lines) != len(KEYS):
        fail()
    values = {}
    for expected, line in zip(KEYS, lines):
        if line.count(b"=") != 1:
            fail()
        key, value = line.split(b"=", 1)
        if key.decode("ascii") != expected:
            fail()
        values[expected] = value
    if values["domain"] != DOMAIN.encode("ascii") or values["version"] != b"1" or values["principal_kind"] != b"unix":
        fail()
    prefix = normalize_path(b64_decode(values["selected_host_prefix"]))
    if b64_decode(values["host_substrate_home"]) != prefix or b64_decode(values["host_substrate_root"]) != prefix:
        fail()
    account = b64_decode(values["principal_account"])
    raw_uid = values["principal_uid"]
    if not re.fullmatch(rb"0|[1-9][0-9]*", raw_uid):
        fail()
    uid = int(raw_uid)
    if uid == 0 or uid > 0xFFFFFFFF:
        fail()
    commitment = values["host_context_commitment"].decode("ascii")
    if not re.fullmatch(r"[0-9a-f]{64}", commitment):
        fail()
    expected_frame = frame(prefix, account, uid)
    if record != expected_frame + b"host_context_commitment=" + commitment.encode("ascii") + b"\n":
        fail()
    if hashlib.sha256(expected_frame).hexdigest() != commitment:
        fail()
    return prefix, account, uid, commitment


try:
    declared = sys.argv[1] == "1"
    raw_prefix = sys.argv[2]
    supplied = sys.argv[3]
    entry, current_uid = current_principal()
    if supplied:
        prefix, account, uid, commitment = decode_carrier(supplied)
        if account != entry.pw_name or uid != current_uid:
            fail()
        if declared and normalize_path(raw_prefix) != prefix:
            fail()
        expected = {
            "SUBSTRATE_HOME": prefix,
            "SUBSTRATE_ROOT": prefix,
            "SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT": commitment,
            "SUBSTRATE_INSTALL_PRIMARY_USER": account,
            "SUBSTRATE_INSTALL_PRIMARY_UID": str(uid),
            "SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1": supplied,
        }
        for key, value in expected.items():
            if key in os.environ and os.environ[key] != value:
                fail()
        carrier = supplied
    else:
        prefix = normalize_path(raw_prefix if declared else entry.pw_dir.rstrip("/") + "/.substrate")
        account = entry.pw_name
        uid = current_uid
        commitment_input = frame(prefix, account, uid)
        commitment = hashlib.sha256(commitment_input).hexdigest()
        carrier = b64_encode(commitment_input + f"host_context_commitment={commitment}\n".encode("ascii")).decode("ascii")
    for value in (prefix, carrier, commitment, account, str(uid)):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid install bootstrap context", file=sys.stderr)
    raise SystemExit(2)
PY
  )
  IFS= read -r -d '' PREFIX <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_CONTEXT_V1 <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_COMMITMENT <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  IFS= read -r -d '' INSTALL_BOOTSTRAP_UID <&"${context_fd}" || fatal "Unable to resolve install bootstrap context."
  exec {context_fd}<&-

  export SUBSTRATE_HOME="${PREFIX}"
  export SUBSTRATE_ROOT="${PREFIX}"
  export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}"
  export SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}"
  export SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}"
  export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
}

usage() {
  cat <<'USAGE'
Substrate Dev Shim Bootstrap (Linux only)

Usage:
  dev-shim-bootstrap.sh --install [--bin <path>] [--prefix <path>]
  dev-shim-bootstrap.sh --uninstall [--bin <path>] [--prefix <path>]

Options:
  --install           Deploy dev shims using the specified substrate binary.
  --uninstall         Remove dev shims that were previously deployed.
  --bin <path>        Path to substrate executable (default: <repo>/target/debug/substrate).
  --prefix <path>     Base directory for shims (default: ~/.substrate).
  --dry-run           Print actions without executing them.
  -h, --help          Show this help message.
USAGE
}

ACTION=""
SUBSTRATE_BIN="${SUBSTRATE_DEV_BIN:-}"
PREFIX=""
PREFIX_DECLARED=0
INSTALL_BOOTSTRAP_CONTEXT_V1=""
DRY_RUN=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --install)
      ACTION="install"
      shift
      ;;
    --uninstall)
      ACTION="uninstall"
      shift
      ;;
    --bin)
      [[ $# -ge 2 ]] || fatal "--bin requires a value"
      SUBSTRATE_BIN="$2"
      shift 2
      ;;
    --prefix)
      [[ $# -ge 2 ]] || fatal "--prefix requires a value"
      PREFIX="$2"
      PREFIX_DECLARED=1
      shift 2
      ;;
    --install-bootstrap-context-v1)
      [[ $# -ge 2 ]] || fatal "--install-bootstrap-context-v1 requires a value"
      INSTALL_BOOTSTRAP_CONTEXT_V1="$2"
      shift 2
      ;;
    --dry-run)
      DRY_RUN=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      fatal "Unknown argument: $1"
      ;;
  esac
done

[[ -n "${ACTION}" ]] || { usage; exit 1; }

if [[ "$(uname -s)" != "Linux" ]]; then
  fatal "This helper currently supports Linux only."
fi

resolve_install_bootstrap_context "${PREFIX_DECLARED}" "${PREFIX}" "${INSTALL_BOOTSTRAP_CONTEXT_V1}"

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ -z "${SUBSTRATE_BIN}" ]]; then
  SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
fi

SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"

run_substrate() {
  if [[ ${DRY_RUN} -eq 1 ]]; then
    printf '[%s][dry-run] substrate <authenticated-context> %s\n' "${SCRIPT_NAME}" "$*"
    return 0
  fi

  if [[ -x "${SUBSTRATE_BIN}" ]]; then
    SHIM_ORIGINAL_PATH="${PATH}" "${SUBSTRATE_BIN}" --no-world \
      --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "$@"
    return $?
  fi

  if command -v substrate >/dev/null 2>&1; then
    SHIM_ORIGINAL_PATH="${PATH}" substrate --no-world \
      --install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}" "$@"
    return $?
  fi

  warn "substrate binary not found (expected ${SUBSTRATE_BIN}); skipping '$*'"
  return 1
}

write_env_file() {
  [[ ${DRY_RUN} -eq 1 ]] && return
  mkdir -p "${PREFIX}"
  local commitment_literal account_literal uid_literal carrier_literal
  commitment_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_COMMITMENT}")"
  account_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_ACCOUNT}")"
  uid_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_UID}")"
  carrier_literal="$(printf '%q' "${INSTALL_BOOTSTRAP_CONTEXT_V1}")"
  cat >"${ENV_FILE}" <<ENV
# Generated by ${SCRIPT_NAME} on $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Source this file to enable Substrate dev shims for the current shell session.
substrate_home="\$(cd "\$(dirname "\${BASH_SOURCE[0]}")" && pwd)"
export SUBSTRATE_HOME="\${substrate_home}"
export SUBSTRATE_ROOT="\${substrate_home}"
export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT=${commitment_literal}
export SUBSTRATE_INSTALL_PRIMARY_USER=${account_literal}
export SUBSTRATE_INSTALL_PRIMARY_UID=${uid_literal}
export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1=${carrier_literal}
if [[ ":\$PATH:" != *":${SHIMS_DIR}:"* ]]; then
  export PATH="${SHIMS_DIR}:\$PATH"
fi
ENV
  log "Wrote dev shim helper to ${ENV_FILE}"
}

remove_env_file() {
  [[ ${DRY_RUN} -eq 1 ]] && return
  if [[ -f "${ENV_FILE}" ]]; then
    rm -f "${ENV_FILE}"
    log "Removed ${ENV_FILE}"
  fi
}

ensure_bin_exists() {
  if [[ ${DRY_RUN} -eq 1 ]]; then
    return 0
  fi
  if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
    warn "substrate binary not found at ${SUBSTRATE_BIN}; build it with 'cargo build -p substrate --bin substrate --bin substrate-shim' or pass --bin"
  fi
}

install_shims() {
  ensure_bin_exists
  mkdir -p "${SHIMS_DIR}"
  log "Deploying shims via ${SUBSTRATE_BIN}"
  if run_substrate --shim-deploy; then
    write_env_file
    cat <<MSG

Dev shims deployed to ${SHIMS_DIR}.
To activate them in this shell, run:
  source ${ENV_FILE}

MSG
  else
    warn "shim deployment reported an error"
  fi
}

uninstall_shims() {
  if run_substrate --shim-remove; then
    log "Removed shims via substrate CLI"
  else
    warn "Falling back to removing ${SHIMS_DIR} manually"
  fi

  if [[ ${DRY_RUN} -eq 0 && -d "${SHIMS_DIR}" ]]; then
    rm -rf "${SHIMS_DIR}"
    log "Deleted ${SHIMS_DIR}"
  fi

  remove_env_file
  cat <<MSG

Dev shims removed. Open a new shell or run 'hash -r' to clear command caches.

MSG
}

case "${ACTION}" in
  install)
    install_shims
    ;;
  uninstall)
    uninstall_shims
    ;;
  *)
    fatal "Unhandled action ${ACTION}"
    ;;
esac
