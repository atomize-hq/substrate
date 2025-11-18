#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-uninstall-substrate"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Substrate Dev Uninstaller

Removes development shims and helper files produced by dev-install-substrate.sh.

Usage:
  dev-uninstall-substrate.sh [--prefix <path>] [--profile <debug|release>] [--bin <path>]
  dev-uninstall-substrate.sh --help

Options:
  --prefix <path>   Installation prefix that was used during dev install (default: ~/.substrate)
  --profile <name>  Cargo profile whose binary should be used for shim removal
  --bin <path>      Explicit path to substrate binary to invoke for shim removal
  --help            Show this message

If neither --profile nor --bin is provided the script will look for
`target/release/substrate` first, then `target/debug/substrate`.
USAGE
}

PREFIX="${HOME}/.substrate"
PROFILE=""
SUBSTRATE_BIN=""

while [[ $# -gt 0 ]]; do
  case "$1" in
    --prefix)
      [[ $# -ge 2 ]] || fatal "--prefix requires a value"
      PREFIX="$2"
      shift 2
      ;;
    --profile)
      [[ $# -ge 2 ]] || fatal "--profile requires a value"
      PROFILE="$2"
      shift 2
      ;;
    --bin)
      [[ $# -ge 2 ]] || fatal "--bin requires a value"
      SUBSTRATE_BIN="$2"
      shift 2
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

BIN_DIR="${PREFIX%/}/bin"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

if [[ -z "${SUBSTRATE_BIN}" ]]; then
  case "${PROFILE}" in
    release)
      SUBSTRATE_BIN="${REPO_ROOT}/target/release/substrate"
      ;;
    debug)
      SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
      ;;
    "")
      if [[ -x "${REPO_ROOT}/target/release/substrate" ]]; then
        SUBSTRATE_BIN="${REPO_ROOT}/target/release/substrate"
      elif [[ -x "${REPO_ROOT}/target/debug/substrate" ]]; then
        SUBSTRATE_BIN="${REPO_ROOT}/target/debug/substrate"
      fi
      ;;
    *)
      fatal "Unsupported profile '${PROFILE}'. Use 'debug' or 'release'."
      ;;
  esac
fi

if [[ -n "${SUBSTRATE_BIN}" && ! -x "${SUBSTRATE_BIN}" ]]; then
  warn "Specified substrate binary (${SUBSTRATE_BIN}) is not executable; shim removal may be incomplete."
  SUBSTRATE_BIN=""
fi

if [[ -n "${SUBSTRATE_BIN}" ]]; then
  log "Removing shims via ${SUBSTRATE_BIN}"
  if ! SHIM_ORIGINAL_PATH="${PATH}" SUBSTRATE_ROOT="${PREFIX}" SUBSTRATE_WORLD=disabled SUBSTRATE_WORLD_ENABLED=0 "${SUBSTRATE_BIN}" --shim-remove; then
    warn "substrate --shim-remove returned an error"
  fi
else
  warn "No substrate binary found; skipping shim-remove invocation."
fi

SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"

if [[ -d "${SHIMS_DIR}" ]]; then
  log "Deleting ${SHIMS_DIR}"
  rm -rf "${SHIMS_DIR}"
fi

if [[ -f "${ENV_FILE}" ]]; then
  log "Removing ${ENV_FILE}"
  rm -f "${ENV_FILE}"
fi

if [[ -d "${BIN_DIR}" ]]; then
  log "Cleaning dev symlinks in ${BIN_DIR}"
  for binary in substrate substrate-shim substrate-forwarder host-proxy world-agent; do
    for candidate in "${binary}" "${binary}.exe"; do
      target_path="${BIN_DIR}/${candidate}"
      if [[ -L "${target_path}" ]]; then
        target="$(readlink "${target_path}")"
        if [[ -n "${target}" && "${target}" == "${REPO_ROOT}"/* ]]; then
          rm -f "${target_path}"
        fi
      fi
    done
  done
  rmdir "${BIN_DIR}" 2>/dev/null || true
fi

cat <<'MSG'

Dev shims removed. Open a new shell (or run `hash -r`) to clear cached commands.
Built artifacts under target/ are left untouched.
MSG
