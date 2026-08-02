#!/usr/bin/env bash
UNINSTALL_WRAPPER_INHERITED_XTRACE=0
if [[ $- == *x* ]]; then
  UNINSTALL_WRAPPER_INHERITED_XTRACE=1
  set +x
fi
set -euo pipefail

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
SCRIPT_DIR=""
if [[ -n "${SCRIPT_SOURCE}" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
fi

resolve_public_install_bootstrap_context() {
  local declared="$1"
  local raw_prefix="$2"
  local context_fd

  exec {context_fd}< <(python3 - "${declared}" "${raw_prefix}" <<'PY'
import base64
import hashlib
import os
import pwd
import sys

DOMAIN = "substrate.install_bootstrap_context"


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


def account_entry(account):
    entry = pwd.getpwnam(account)
    if entry.pw_uid == 0 or pwd.getpwuid(entry.pw_uid).pw_name != entry.pw_name:
        fail()
    if not entry.pw_name or any(ch in entry.pw_name for ch in "\0\r\n"):
        fail()
    return entry


def public_principal():
    effective_uid = os.geteuid()
    if effective_uid != 0:
        entry = pwd.getpwuid(effective_uid)
        if entry.pw_uid != effective_uid or pwd.getpwnam(entry.pw_name).pw_uid != effective_uid:
            fail()
        if not entry.pw_name or any(ch in entry.pw_name for ch in "\0\r\n"):
            fail()
        return entry

    explicit = os.environ.get("SUBSTRATE_INSTALL_PRIMARY_USER", "")
    if explicit:
        return account_entry(explicit)

    sudo_user = os.environ.get("SUDO_USER", "")
    sudo_uid = os.environ.get("SUDO_UID", "")
    if not sudo_user or not sudo_uid or not sudo_uid.isdecimal():
        fail()
    entry = account_entry(sudo_user)
    if str(entry.pw_uid) != sudo_uid:
        fail()
    return entry


def frame(prefix, account, uid):
    encoded_prefix = b64_encode(prefix.encode("utf-8")).decode("ascii")
    encoded_account = b64_encode(account.encode("utf-8")).decode("ascii")
    return (
        f"domain={DOMAIN}\n"
        "version=1\n"
        f"selected_host_prefix={encoded_prefix}\n"
        f"host_substrate_home={encoded_prefix}\n"
        f"host_substrate_root={encoded_prefix}\n"
        "principal_kind=unix\n"
        f"principal_account={encoded_account}\n"
        f"principal_uid={uid}\n"
    ).encode("ascii")


try:
    declared = sys.argv[1] == "1"
    raw_prefix = sys.argv[2]
    entry = public_principal()
    prefix = normalize_path(raw_prefix if declared else entry.pw_dir.rstrip("/") + "/.substrate")
    commitment_input = frame(prefix, entry.pw_name, entry.pw_uid)
    commitment = hashlib.sha256(commitment_input).hexdigest()
    carrier = b64_encode(
        commitment_input + f"host_context_commitment={commitment}\n".encode("ascii")
    ).decode("ascii")
    account_home = normalize_path(entry.pw_dir)
    for value in (prefix, carrier, commitment, entry.pw_name, str(entry.pw_uid), account_home):
        sys.stdout.buffer.write(value.encode("utf-8") + b"\0")
except Exception:
    print("invalid install bootstrap context", file=sys.stderr)
    raise SystemExit(2)
PY
  )
  IFS= read -r -d '' PREFIX <&"${context_fd}" || return 2
  IFS= read -r -d '' INSTALL_BOOTSTRAP_CONTEXT_V1 <&"${context_fd}" || return 2
  IFS= read -r -d '' INSTALL_BOOTSTRAP_COMMITMENT <&"${context_fd}" || return 2
  IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT <&"${context_fd}" || return 2
  IFS= read -r -d '' INSTALL_BOOTSTRAP_UID <&"${context_fd}" || return 2
  IFS= read -r -d '' INSTALL_BOOTSTRAP_ACCOUNT_HOME <&"${context_fd}" || return 2
  exec {context_fd}<&-

  export SUBSTRATE_HOME="${PREFIX}"
  export SUBSTRATE_ROOT="${PREFIX}"
  export SUBSTRATE_INSTALL_HOST_CONTEXT_COMMITMENT="${INSTALL_BOOTSTRAP_COMMITMENT}"
  export SUBSTRATE_INSTALL_PRIMARY_USER="${INSTALL_BOOTSTRAP_ACCOUNT}"
  export SUBSTRATE_INSTALL_PRIMARY_UID="${INSTALL_BOOTSTRAP_UID}"
  export SUBSTRATE_INSTALL_BOOTSTRAP_CONTEXT_V1="${INSTALL_BOOTSTRAP_CONTEXT_V1}"
}

PREFIX=""

resolve_latest_release_tag() {
  local latest_url
  latest_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' https://github.com/atomize-hq/substrate/releases/latest || true)"
  if [[ "${latest_url}" =~ /tag/([^/]+)$ ]]; then
    printf '%s\n' "${BASH_REMATCH[1]}"
    return 0
  fi
  return 1
}

resolve_local_uninstaller() {
  local root="$1"
  local substrate_link="${root}/bin/substrate"

  if [[ -x "${root}/versions/current/scripts/substrate/uninstall-substrate.sh" ]]; then
    printf '%s\n' "${root}/versions/current/scripts/substrate/uninstall-substrate.sh"
    return 0
  fi

  if command -v readlink >/dev/null 2>&1 && [[ -L "${substrate_link}" ]]; then
    local target
    target="$(readlink "${substrate_link}" || true)"
    if [[ -n "${target}" ]]; then
      if [[ "${target}" != /* ]]; then
        target="${root}/bin/${target}"
      fi
      # Resolve one more hop if the target is itself a symlink (best effort; avoids readlink -f).
      if [[ -L "${target}" ]]; then
        local hop
        hop="$(readlink "${target}" || true)"
        if [[ -n "${hop}" ]]; then
          if [[ "${hop}" != /* ]]; then
            target="$(cd "$(dirname "${target}")" && pwd)/${hop}"
          else
            target="${hop}"
          fi
        fi
      fi
      local version_dir
      version_dir="$(cd "$(dirname "${target}")/.." 2>/dev/null && pwd || true)"
      if [[ -n "${version_dir}" && -x "${version_dir}/scripts/substrate/uninstall-substrate.sh" ]]; then
        printf '%s\n' "${version_dir}/scripts/substrate/uninstall-substrate.sh"
        return 0
      fi
    fi
  fi

  if [[ -d "${root}/versions" ]]; then
    local dir
    while IFS= read -r dir; do
      if [[ -x "${dir}/scripts/substrate/uninstall-substrate.sh" ]]; then
        printf '%s\n' "${dir}/scripts/substrate/uninstall-substrate.sh"
        return 0
      fi
    done < <(ls -1dt "${root}/versions"/* 2>/dev/null || true)
  fi

  return 1
}

detect_prefix() {
  local prefix=""
  local arg=""
  for arg in "$@"; do
    case "${arg}" in
      --prefix=*)
        prefix="${arg#--prefix=}"
        ;;
    esac
  done
  if [[ -z "${prefix}" ]]; then
    local i j
    for ((i=1; i<=$#; i++)); do
      if [[ "${!i}" == "--prefix" ]]; then
        j=$((i+1))
        prefix="${!j:-}"
        break
      fi
    done
  fi
  printf '%s\n' "${prefix}"
}

args_declare_prefix() {
  local arg=""
  for arg in "$@"; do
    case "${arg}" in
      --prefix|--prefix=*)
        return 0
        ;;
    esac
  done
  return 1
}

args_request_help() {
  local arg=""
  for arg in "$@"; do
    case "${arg}" in
      -h|--help)
        return 0
        ;;
    esac
  done
  return 1
}

PREFIX_RAW="$(detect_prefix "$@")"
PREFIX_DECLARED=0
if args_declare_prefix "$@"; then
  PREFIX_DECLARED=1
fi
HELP_REQUESTED=0
if args_request_help "$@"; then
  HELP_REQUESTED=1
fi
if [[ "${HELP_REQUESTED}" -eq 0 ]]; then
  if ! resolve_public_install_bootstrap_context "${PREFIX_DECLARED}" "${PREFIX_RAW}"; then
    echo "[substrate-uninstall] unable to construct install bootstrap context." >&2
    exit 2
  fi
fi

ASSET_TMP_DIR=""
ASSET_ROOT="${SCRIPT_DIR}"
UPSTREAM_UNINSTALL="${ASSET_ROOT}/uninstall-substrate.sh"
LOADER_DIR="${ASSET_ROOT}/loader"

if [[ -z "${ASSET_ROOT}" || ! -x "${UPSTREAM_UNINSTALL}" ]]; then
  # Prefer running the uninstaller shipped with the installed version (keeps behavior/version aligned).
  if [[ "${HELP_REQUESTED}" -eq 0 ]] && local_uninstall="$(resolve_local_uninstaller "${PREFIX}" 2>/dev/null)"; then
    UPSTREAM_UNINSTALL="${local_uninstall}"
  else
    # When running via curl|bash, fetch the helpers on demand (pinned to the latest release tag).
    if ! command -v curl >/dev/null 2>&1; then
      echo "[substrate-uninstall] 'curl' is required to download uninstaller assets." >&2
      exit 1
    fi
    ASSET_TMP_DIR="$(mktemp -d -t substrate-uninstall-assets.XXXXXX)"
    UNINSTALL_REF="${SUBSTRATE_UNINSTALL_REF:-${SUBSTRATE_INSTALL_REF:-}}"
    if [[ -z "${UNINSTALL_REF}" ]]; then
      if ! UNINSTALL_REF="$(resolve_latest_release_tag)"; then
        UNINSTALL_REF="main"
        echo "[substrate-uninstall][WARN] Unable to resolve latest release tag; falling back to '${UNINSTALL_REF}' for uninstaller helpers." >&2
      fi
    fi
    ASSET_BASE="${SUBSTRATE_UNINSTALL_WRAPPER_BASE_URL:-https://raw.githubusercontent.com/atomize-hq/substrate/${UNINSTALL_REF}/scripts/substrate}"
    mkdir -p "${ASSET_TMP_DIR}/loader"
    curl -fsSL "${ASSET_BASE}/uninstall-substrate.sh" -o "${ASSET_TMP_DIR}/uninstall-substrate.sh"
    curl -fsSL "${ASSET_BASE}/loader/bash_loading_animations.sh" -o "${ASSET_TMP_DIR}/loader/bash_loading_animations.sh" || true
    chmod +x "${ASSET_TMP_DIR}/uninstall-substrate.sh"
    ASSET_ROOT="${ASSET_TMP_DIR}"
    UPSTREAM_UNINSTALL="${ASSET_ROOT}/uninstall-substrate.sh"
    LOADER_DIR="${ASSET_ROOT}/loader"
  fi
fi

if [[ ! -x "${UPSTREAM_UNINSTALL}" ]]; then
  echo "[substrate-uninstall] missing uninstaller at ${UPSTREAM_UNINSTALL}" >&2
  exit 1
fi

TMP_LOG="$(mktemp -t substrate-uninstall-log.XXXXXX)"
LOADER_STARTED=0
stop_loader() {
  if [[ "${LOADER_STARTED}" -eq 1 ]]; then
    BLA::stop_loading_animation
    LOADER_STARTED=0
  fi
}
cleanup() {
  stop_loader
  rm -f "${TMP_LOG}"
  if [[ -n "${ASSET_TMP_DIR}" && -d "${ASSET_TMP_DIR}" ]]; then
    rm -rf "${ASSET_TMP_DIR}"
  fi
}
trap cleanup EXIT

if [[ "${HELP_REQUESTED}" -eq 1 ]]; then
  if "${UPSTREAM_UNINSTALL}" "$@"; then
    if [[ "${UNINSTALL_WRAPPER_INHERITED_XTRACE}" -eq 1 ]]; then
      set -x
    fi
    exit 0
  else
    upstream_status=$?
    if [[ "${UNINSTALL_WRAPPER_INHERITED_XTRACE}" -eq 1 ]]; then
      set -x
    fi
    exit "${upstream_status}"
  fi
fi

UPSTREAM_ARGS=("$@")
if [[ "${PREFIX_DECLARED}" -eq 0 ]]; then
  UPSTREAM_ARGS+=(--prefix "${PREFIX}")
fi
UPSTREAM_ARGS+=(--install-bootstrap-context-v1 "${INSTALL_BOOTSTRAP_CONTEXT_V1}")

printf "\033[32mSubstrate uninstall running…\033[0m\n"
if [[ -f "${LOADER_DIR}/bash_loading_animations.sh" ]]; then
  # shellcheck disable=SC1090
  source "${LOADER_DIR}/bash_loading_animations.sh"
  if [[ -t 1 && -n "${BLA_braille_fill_bar[*]:-}" ]]; then
    BLA::start_loading_animation "${BLA_braille_fill_bar[@]}"
    LOADER_STARTED=1
  fi
fi

if "${UPSTREAM_UNINSTALL}" "${UPSTREAM_ARGS[@]}" >"${TMP_LOG}" 2>&1; then
  if [[ "${UNINSTALL_WRAPPER_INHERITED_XTRACE}" -eq 1 ]]; then
    set -x
  fi
  stop_loader
  printf "\033[32mSubstrate uninstall complete!\033[0m\n"
else
  if [[ "${UNINSTALL_WRAPPER_INHERITED_XTRACE}" -eq 1 ]]; then
    set -x
  fi
  stop_loader
  printf "\033[31mSubstrate uninstall failed.\033[0m See %s for details.\n" "${TMP_LOG}" >&2
  cat "${TMP_LOG}" >&2
  exit 1
fi
