#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-install-substrate"

log()   { printf '[%s] %s\n' "${SCRIPT_NAME}" "$1"; }
warn()  { printf '[%s][WARN] %s\n' "${SCRIPT_NAME}" "$1" >&2; }
fatal() { printf '[%s][ERROR] %s\n' "${SCRIPT_NAME}" "$1" >&2; exit 1; }

usage() {
  cat <<'USAGE'
Substrate Dev Installer

Build Substrate from the current repository and wire development shims to the
freshly built binaries. This is intended for local iteration after removing any
production installation.

Usage:
  dev-install-substrate.sh [--prefix <path>] [--profile <debug|release>] [--version-label <name>] [--no-world] [--world-root-mode <mode>] [--world-root-path <path>] [--no-shims]
  dev-install-substrate.sh --help

Options:
  --prefix <path>           Installation prefix for shims/env helper (default: ~/.substrate)
  --profile <name>          Cargo profile to build (debug or release; default: debug)
  --version-label <name>    Version directory label under <prefix>/versions (default: dev)
  --no-world                Mark install metadata as world_disabled (skips provisioning entirely)
  --world-root-mode <mode>  Default world root mode (project|follow-cwd|custom; default: project)
  --world-root-path <path>  Default world root path (for custom mode)
  --no-shims                Skip shim deployment (only run cargo build)
  --help                    Show this message
USAGE
}

PREFIX="${HOME}/.substrate"
PROFILE="debug"
DEPLOY_SHIMS=1
WORLD_ENABLED=1
WORLD_ROOT_MODE="project"
WORLD_ROOT_PATH=""
VERSION_LABEL="dev"

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
    --version-label)
      [[ $# -ge 2 ]] || fatal "--version-label requires a value"
      VERSION_LABEL="$2"
      shift 2
      ;;
    --no-world)
      WORLD_ENABLED=0
      shift
      ;;
    --world-root-mode)
      [[ $# -ge 2 ]] || fatal "--world-root-mode requires a value"
      WORLD_ROOT_MODE="$2"
      shift 2
      ;;
    --world-root-path)
      [[ $# -ge 2 ]] || fatal "--world-root-path requires a value"
      WORLD_ROOT_PATH="$2"
      shift 2
      ;;
    --no-shims)
      DEPLOY_SHIMS=0
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

case "${PROFILE}" in
  debug|release) ;;
  *) fatal "Unsupported profile '${PROFILE}'. Use 'debug' or 'release'." ;;
esac

case "${WORLD_ROOT_MODE}" in
  project|follow-cwd|custom) ;;
  *) fatal "Unsupported world root mode '${WORLD_ROOT_MODE}'. Use project, follow-cwd, or custom." ;;
esac

if [[ "${WORLD_ROOT_MODE}" == "custom" && -z "${WORLD_ROOT_PATH}" ]]; then
  fatal "--world-root-path is required when --world-root-mode=custom"
fi

if ! command -v cargo >/dev/null 2>&1; then
  fatal "cargo not found on PATH. Install the Rust toolchain before running this script."
fi

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "${REPO_ROOT}"

TARGET_DIR="${PROFILE}"
BUILD_FLAGS=(build --bin substrate --bin substrate-shim)
if [[ "${PROFILE}" == "release" ]]; then
  BUILD_FLAGS+=(--release)
fi

log "Building Substrate (${PROFILE})..."
cargo "${BUILD_FLAGS[@]}"

SUBSTRATE_BIN="${REPO_ROOT}/target/${TARGET_DIR}/substrate"
if [[ ! -x "${SUBSTRATE_BIN}" ]]; then
  fatal "Expected substrate binary at ${SUBSTRATE_BIN}, but it was not found."
fi

BIN_DIR="${PREFIX%/}/bin"
SHIMS_DIR="${PREFIX%/}/shims"
ENV_FILE="${PREFIX%/}/dev-shim-env.sh"
VERSION_DIR="${PREFIX%/}/versions/${VERSION_LABEL}"
VERSION_CONFIG_DIR="${VERSION_DIR}/config"
MANAGER_INIT_PATH="${PREFIX%/}/manager_init.sh"
MANAGER_ENV_PATH="${PREFIX%/}/manager_env.sh"
INSTALL_CONFIG_PATH="${PREFIX%/}/config.toml"

mkdir -p "${PREFIX}" "${BIN_DIR}" "${VERSION_CONFIG_DIR}"

# Stage config assets to mirror the production bundle layout.
if [[ -d "${REPO_ROOT}/config" ]]; then
  cp -R "${REPO_ROOT}/config/." "${VERSION_CONFIG_DIR}/"
fi
if [[ -f "${REPO_ROOT}/scripts/substrate/world-deps.yaml" ]]; then
  cp "${REPO_ROOT}/scripts/substrate/world-deps.yaml" "${VERSION_CONFIG_DIR}/world-deps.yaml"
fi
if [[ ! -f "${VERSION_CONFIG_DIR}/manager_hooks.yaml" ]]; then
  fatal "manager manifest missing (expected ${VERSION_CONFIG_DIR}/manager_hooks.yaml)"
fi
if [[ ! -f "${VERSION_CONFIG_DIR}/world-deps.yaml" ]]; then
  fatal "world-deps manifest missing (expected ${VERSION_CONFIG_DIR}/world-deps.yaml)"
fi

# Write install metadata (install + world tables) like the production installer.
cat > "${INSTALL_CONFIG_PATH}.tmp" <<EOF
[install]
world_enabled = $([[ "${WORLD_ENABLED}" -eq 1 ]] && echo "true" || echo "false")

[world]
root_mode = "${WORLD_ROOT_MODE}"
root_path = "${WORLD_ROOT_PATH}"
EOF
mv "${INSTALL_CONFIG_PATH}.tmp" "${INSTALL_CONFIG_PATH}"
chmod 0644 "${INSTALL_CONFIG_PATH}" || true

# Write manager init placeholder + env exporter.
cat > "${MANAGER_INIT_PATH}.tmp" <<'EOF'
#!/usr/bin/env bash
# Managed by dev-install-substrate

# Place per-manager snippets here if you need them for debugging.
EOF
mv "${MANAGER_INIT_PATH}.tmp" "${MANAGER_INIT_PATH}"
chmod 0644 "${MANAGER_INIT_PATH}" || true

today="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
manager_env_literal="$(printf '%q' "${MANAGER_ENV_PATH}")"
manager_init_literal="$(printf '%q' "${MANAGER_INIT_PATH}")"
cat > "${MANAGER_ENV_PATH}.tmp" <<EOF
#!/usr/bin/env bash
# Managed by ${SCRIPT_NAME} on ${today}
export SUBSTRATE_WORLD=$([[ "${WORLD_ENABLED}" -eq 1 ]] && echo "enabled" || echo "disabled")
export SUBSTRATE_WORLD_ENABLED=$([[ "${WORLD_ENABLED}" -eq 1 ]] && echo "1" || echo "0")
export SUBSTRATE_MANAGER_ENV=${manager_env_literal}
export SUBSTRATE_MANAGER_INIT=${manager_init_literal}

manager_init_path=${manager_init_literal}
if [[ -f "\${manager_init_path}" ]]; then
  # shellcheck disable=SC1090
  source "\${manager_init_path}"
fi
EOF
mv "${MANAGER_ENV_PATH}.tmp" "${MANAGER_ENV_PATH}"
chmod 0644 "${MANAGER_ENV_PATH}" || true

shim_note=""
if [[ ${DEPLOY_SHIMS} -eq 1 ]]; then
  log "Deploying shims via ${SUBSTRATE_BIN}"
  if ! SHIM_ORIGINAL_PATH="${PATH}" SUBSTRATE_ROOT="${PREFIX}" "${SUBSTRATE_BIN}" --shim-deploy; then
    fatal "Shim deployment failed"
  fi
  shim_note="Dev shims deployed to ${SHIMS_DIR}."
else
  warn "Shim deployment skipped (--no-shims)."
  shim_note="Shims were not deployed (--no-shims). Binaries are available under ${BIN_DIR}."
fi

for binary in substrate substrate-shim substrate-forwarder host-proxy world-agent; do
  src="${REPO_ROOT}/target/${TARGET_DIR}/${binary}"
  if [[ -x "${src}" ]]; then
    ln -sfn "${src}" "${BIN_DIR}/${binary}"
  elif [[ -x "${src}.exe" ]]; then
    ln -sfn "${src}.exe" "${BIN_DIR}/${binary}.exe"
  fi
done

cat >"${ENV_FILE}" <<EOF_ENV
# Generated by ${SCRIPT_NAME} on $(date -u +"%Y-%m-%dT%H:%M:%SZ")
# Source this file to enable Substrate dev shims for the current shell session.
export SUBSTRATE_ROOT="${PREFIX}"
export SUBSTRATE_MANAGER_ENV="${MANAGER_ENV_PATH}"
export SUBSTRATE_MANAGER_INIT="${MANAGER_INIT_PATH}"
if [[ -z "\${SHIM_ORIGINAL_PATH:-}" ]]; then
  export SHIM_ORIGINAL_PATH="\$PATH"
fi
if [[ ":\$PATH:" != *":${BIN_DIR}:"* ]]; then
  export PATH="${BIN_DIR}:\$PATH"
fi
if [[ ":\$PATH:" != *":${SHIMS_DIR}:"* ]]; then
  export PATH="${SHIMS_DIR}:\$PATH"
fi
EOF_ENV
log "Wrote dev shim helper to ${ENV_FILE}"

cat <<MSG

${shim_note}
To add the dev binaries/shims to PATH for this shell, run:
  source ${ENV_FILE}

MSG
log "Substrate dev install complete."
log "manager_init placeholder: ${MANAGER_INIT_PATH}"
log "manager_env script: ${MANAGER_ENV_PATH}"
log "install metadata: ${INSTALL_CONFIG_PATH}"
