#!/usr/bin/env bash
set -euo pipefail

readonly INSTALLER_NAME="substrate-install"
readonly INSTALLER_VERSION="0.1.0-dev"
readonly DEFAULT_VERSION="0.2.0-beta"
readonly DEFAULT_PREFIX="${HOME}/.substrate"
readonly DEFAULT_BASE_URL="https://github.com/atomize-hq/substrate/releases/download"

VERSION_RAW=""
VERSION=""
VERSION_TAG=""
PREFIX="$DEFAULT_PREFIX"
NO_WORLD=0
NO_SHIMS=0
DRY_RUN=0
ARCHIVE_OVERRIDE="${SUBSTRATE_INSTALL_ARCHIVE:-}"
BASE_URL="${SUBSTRATE_INSTALL_BASE_URL:-$DEFAULT_BASE_URL}"
TMPDIR=""
PLATFORM=""
ARCH=""
IS_WSL=0
ORIGINAL_PATH="${PATH}"

log() {
  printf '[%s] %s\n' "${INSTALLER_NAME}" "$*" >&2
}

warn() {
  printf '[%s][WARN] %s\n' "${INSTALLER_NAME}" "$*" >&2
}

fatal() {
  printf '[%s][ERROR] %s\n' "${INSTALLER_NAME}" "$*" >&2
  exit 1
}

print_usage() {
  cat <<'EOF'
Substrate Installer
Usage:
  curl -fsSL https://raw.githubusercontent.com/atomize-hq/substrate/main/scripts/substrate/install-substrate.sh | bash
  # (Windows host) powershell -ExecutionPolicy Bypass -File install-substrate.ps1

Options:
  --version <semver>   Install a specific release (default: 0.2.0-beta)
  --prefix <path>      Installation prefix (default: ~/.substrate)
  --no-world           Skip world backend provisioning
  --no-shims           Skip shim deployment
  --dry-run            Print actions without executing
  --archive <path>     Use a local archive instead of downloading (dev/test)
  -h, --help           Show this message
EOF
}

cleanup() {
  if [[ -n "${TMPDIR}" && -d "${TMPDIR}" && "${DRY_RUN}" -eq 0 ]]; then
    rm -rf "${TMPDIR}"
  fi
}

run_cmd() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s\n' "${INSTALLER_NAME}" "$*" >&2
    return 0
  fi
  "$@"
}

require_cmd() {
  local cmd="$1"
  command -v "${cmd}" >/dev/null 2>&1 || fatal "Required command '${cmd}' not found. Please install it and re-run."
}

compute_file_sha256() {
  local file="$1"
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "${file}" | awk '{print $1}'
  elif command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "${file}" | awk '{print $1}'
  else
    fatal "Neither sha256sum nor shasum found; cannot verify checksums."
  fi
}

sanitize_env_path() {
  if [[ -n "${SHIM_ORIGINAL_PATH:-}" ]]; then
    PATH="${SHIM_ORIGINAL_PATH}"
  else
    local shim_dir="${HOME}/.substrate/shims"
    local IFS=':'
    local parts=($PATH)
    local filtered=()
    for entry in "${parts[@]}"; do
      if [[ "${entry}" == "${shim_dir}" ]]; then
        continue
      fi
      filtered+=("${entry}")
    done
    PATH="$(IFS=':'; printf '%s' "${filtered[*]}")"
  fi
  export PATH
  ORIGINAL_PATH="${PATH}"
}

detect_platform() {
  local uname_s
  uname_s="$(uname -s)"
  ARCH="$(uname -m)"

  case "${uname_s}" in
    Darwin)
      PLATFORM="macos"
      ;;
    Linux)
      PLATFORM="linux"
      if grep -qi microsoft /proc/version 2>/dev/null; then
        IS_WSL=1
      fi
      ;;
    MINGW*|MSYS*|CYGWIN*)
      PLATFORM="windows"
      ;;
    *)
      fatal "Unsupported operating system: ${uname_s}"
      ;;
  esac
}

parse_args() {
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --version)
        [[ $# -lt 2 ]] && fatal "Missing value for --version"
        VERSION_RAW="$2"
        shift 2
        ;;
      --prefix)
        [[ $# -lt 2 ]] && fatal "Missing value for --prefix"
        PREFIX="$2"
        shift 2
        ;;
      --no-world)
        NO_WORLD=1
        shift
        ;;
      --no-shims)
        NO_SHIMS=1
        shift
        ;;
      --dry-run)
        DRY_RUN=1
        shift
        ;;
      --archive)
        [[ $# -lt 2 ]] && fatal "Missing value for --archive"
        ARCHIVE_OVERRIDE="$2"
        shift 2
        ;;
      -h|--help)
        print_usage
        exit 0
        ;;
      *)
        fatal "Unknown option: $1"
        ;;
    esac
  done

  if [[ -z "${VERSION_RAW}" ]]; then
    VERSION_RAW="${DEFAULT_VERSION}"
    warn "No --version provided; defaulting to ${VERSION_RAW}"
  fi

  VERSION="${VERSION_RAW#v}"
  if [[ -z "${VERSION}" ]]; then
    fatal "Unable to determine version from '${VERSION_RAW}'"
  fi
  VERSION_TAG="v${VERSION}"
}

prepare_tmpdir() {
  TMPDIR="$(mktemp -d -t substrate-install.XXXXXX)"
  trap cleanup EXIT
}

ensure_macos_prereqs() {
  require_cmd sw_vers
  require_cmd sysctl
  require_cmd curl
  require_cmd tar
  require_cmd shasum
  require_cmd jq
  require_cmd limactl
  require_cmd envsubst

  local hv_support
  hv_support="$(sysctl -n kern.hv_support 2>/dev/null || true)"
  if [[ "${hv_support}" != "1" ]]; then
    fatal "macOS virtualization not available. Enable Virtualization Framework in System Settings."
  fi

  if [[ "${ARCH}" != "arm64" ]]; then
    fatal "Only macOS arm64 is currently supported."
  fi
}

ensure_linux_prereqs() {
  require_cmd curl
  require_cmd tar
  require_cmd jq
  require_cmd sudo

  if ! command -v sha256sum >/dev/null 2>&1 && ! command -v shasum >/dev/null 2>&1; then
    fatal "Missing sha256sum (preferred) or shasum for checksum verification. Install coreutils/perl-Digest-SHA or rerun with --dry-run."
  fi

  if [[ "${NO_WORLD}" -eq 0 ]]; then
    if ! command -v systemctl >/dev/null 2>&1; then
      fatal "systemctl not found. Install systemd tooling or re-run with --no-world to skip world provisioning."
    fi

    local init_comm
    init_comm="$(ps -p 1 -o comm= 2>/dev/null || true)"
    if [[ "${init_comm}" != "systemd" ]]; then
      if [[ "${IS_WSL}" -eq 1 ]]; then
        fatal "WSL distribution not running systemd (pid 1: ${init_comm:-unknown}). Enable systemd in /etc/wsl.conf or re-run with --no-world."
      else
        fatal "Systemd is not PID 1 (detected '${init_comm:-unknown}'). Boot into a systemd-based userland or install with --no-world."
      fi
    fi
  fi
}

download_file() {
  local source="$1"
  local destination="$2"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] download %s -> %s\n' "${INSTALLER_NAME}" "${source}" "${destination}" >&2
    return 0
  fi

  local dir
  dir="$(dirname "${destination}")"
  mkdir -p "${dir}"

  if [[ "${source}" =~ ^https?:// ]]; then
    curl -fL --progress-bar -o "${destination}" "${source}"
  else
    cp "${source}" "${destination}"
  fi
}

download_artifact() {
  local artifact_name="$1"
  local dest_path="$2"

  if [[ -n "${ARCHIVE_OVERRIDE}" ]]; then
    if [[ ! -f "${ARCHIVE_OVERRIDE}" ]]; then
      fatal "Override archive not found at ${ARCHIVE_OVERRIDE}"
    fi
    log "Using local archive override: ${ARCHIVE_OVERRIDE}"
    download_file "${ARCHIVE_OVERRIDE}" "${dest_path}"
    return
  fi

  local url="${BASE_URL}/${VERSION_TAG}/${artifact_name}"
  log "Downloading ${artifact_name} from ${url}"
  download_file "${url}" "${dest_path}"
}

download_checksums() {
  local dest_path="$1"

  if [[ -n "${ARCHIVE_OVERRIDE}" ]]; then
    warn "Skipping checksum verification for local override archive."
    return 1
  fi

  local url="${BASE_URL}/${VERSION_TAG}/SHA256SUMS"
  log "Downloading SHA256SUMS from ${url}"
  if ! download_file "${url}" "${dest_path}"; then
    warn "Failed to download SHA256SUMS; skipping checksum verification."
    return 1
  fi
  return 0
}

verify_checksum() {
  local archive_path="$1"
  local checksums_path="$2"
  local artifact_name="$3"

  if [[ ! -f "${checksums_path}" ]]; then
    warn "Checksum file missing; skipping verification."
    return
  fi

  local expected
  expected="$(grep "  ${artifact_name}$" "${checksums_path}" | awk '{print $1}' || true)"
  if [[ -z "${expected}" ]]; then
    warn "Checksum entry for ${artifact_name} not found; skipping verification."
    return
  fi

  local actual
  actual="$(compute_file_sha256 "${archive_path}")"

  if [[ "${expected}" != "${actual}" ]]; then
    fatal "Checksum mismatch for ${artifact_name}: expected ${expected}, got ${actual}"
  fi
  log "Checksum verified for ${artifact_name}"
}

extract_archive() {
  local archive_path="$1"
  local dest_dir="$2"

  mkdir -p "${dest_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] tar -xzf %s -C %s\n' "${INSTALLER_NAME}" "${archive_path}" "${dest_dir}" >&2
    return 0
  fi

  tar -xzf "${archive_path}" -C "${dest_dir}"
}

find_extracted_root() {
  local dest_dir="$1"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '%s\n' "${dest_dir}/SIMULATED_ROOT"
    return
  fi
  local entries=()
  while IFS= read -r entry; do
    entries+=("${entry}")
  done < <(find "${dest_dir}" -mindepth 1 -maxdepth 1 -print)
  if [[ ${#entries[@]} -eq 0 ]]; then
    fatal "Failed to determine extracted archive root."
  fi
  if [[ ${#entries[@]} -eq 1 && -d "${entries[0]}" ]]; then
    printf '%s\n' "${entries[0]}"
  else
    printf '%s\n' "${dest_dir}"
  fi
}

link_binaries() {
  local version_dir="$1"
  local bin_dir="$2"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Linking binaries from %s into %s\n' "${INSTALLER_NAME}" "${version_dir}/bin" "${bin_dir}" >&2
    return
  fi

  mkdir -p "${bin_dir}"
  find "${bin_dir}" -maxdepth 1 -type l -exec rm -f {} +
  if [[ -d "${version_dir}/bin" ]]; then
    for binary in "${version_dir}/bin/"*; do
      local name
      name="$(basename "${binary}")"
      ln -sfn "${binary}" "${bin_dir}/${name}"
    done
  else
    warn "No bin directory found in ${version_dir}"
  fi
}

ensure_shell_integration() {
  local shim_dir="$1"
  local bin_dir="$2"
  local bashenv="${HOME}/.substrate_bashenv"
  local tramp="${HOME}/.substrate_bashenv_trampoline"
  local existing_be="${BASH_ENV:-}"
  local today
  today="$(date)"

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    mkdir -p "$(dirname "${bashenv}")"
    cat > "${bashenv}.tmp" <<EOF
# Generated by ${INSTALLER_NAME} on ${today}
export SUBSTRATE_ROOT="${PREFIX}"
if [ -z "\${SHIM_ORIGINAL_PATH:-}" ]; then
  export SHIM_ORIGINAL_PATH="\${PATH}"
fi
export PATH="${shim_dir}:${bin_dir}:\${PATH}"
export SHIM_ORIGINAL_PATH
EOF
    mv "${bashenv}.tmp" "${bashenv}"
  else
    printf '[%s][dry-run] Update %s with PATH exports\n' "${INSTALLER_NAME}" "${bashenv}" >&2
  fi

  local target_bash_env="${bashenv}"

  if [[ -n "${existing_be}" ]]; then
    target_bash_env="${tramp}"
    local expanded_be="${existing_be}"
    if [[ "${expanded_be}" == ~* ]]; then
      expanded_be="${expanded_be/#~/$HOME}"
    fi
    if command -v envsubst >/dev/null 2>&1; then
      expanded_be="$(envsubst <<<"${expanded_be}")"
    fi
    local escaped_be
    escaped_be="$(printf '%q' "${expanded_be}")"

    if [[ "${DRY_RUN}" -eq 0 ]]; then
      cat > "${tramp}.tmp" <<EOF
#!/usr/bin/env bash
orig=${escaped_be}
if [ -n "\${orig}" ] && [ -f "\${orig}" ]; then
  # shellcheck disable=SC1090
  source "\${orig}"
fi
# shellcheck disable=SC1090
source "${bashenv}"
EOF
      mv "${tramp}.tmp" "${tramp}"
      chmod +x "${tramp}"
    else
      printf '[%s][dry-run] Create trampoline at %s chaining %s\n' "${INSTALLER_NAME}" "${tramp}" "${expanded_be}" >&2
    fi
  fi

  local snippet
  snippet="$(cat <<EOF
# Added by substrate installer
if [ -f "\$HOME/.substrate_bashenv" ]; then
  # shellcheck disable=SC1090
  source "\$HOME/.substrate_bashenv"
  export BASH_ENV="${target_bash_env}"
fi
EOF
)"

  local shells=( "${HOME}/.zshrc" "${HOME}/.bashrc" "${HOME}/.bash_profile" )
  for shell_rc in "${shells[@]}"; do
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      printf '[%s][dry-run] Ensure PATH/BASH_ENV snippet in %s\n' "${INSTALLER_NAME}" "${shell_rc}" >&2
      continue
    fi

    if [[ -f "${shell_rc}" ]]; then
      if ! grep -Fq 'source "$HOME/.substrate_bashenv"' "${shell_rc}"; then
        printf '\n%s\n' "${snippet}" >> "${shell_rc}"
      fi
    else
      printf '#!/usr/bin/env bash\n%s\n' "${snippet}" > "${shell_rc}"
    fi
  done

  if [[ "${DRY_RUN}" -eq 0 ]]; then
    if [ -f "${bashenv}" ]; then
      # shellcheck disable=SC1090
      source "${bashenv}" 2>/dev/null || true
    fi
    export SUBSTRATE_ORIGINAL_BASH_ENV="${existing_be}"
    export BASH_ENV="${target_bash_env}"
  fi
}

deploy_shims() {
  local substrate_bin="$1"
  if [[ "${NO_SHIMS}" -eq 1 ]]; then
    log "Skipping shim deployment (--no-shims)."
    return
  fi

  log "Deploying shims..."
  run_cmd "${substrate_bin}" --shim-deploy
}

provision_macos_world() {
  local release_root="$1"

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world provisioning (--no-world)."
    return
  fi

  log "Provisioning macOS Lima world backend..."

  local lima_script="${release_root}/scripts/mac/lima-warm.sh"
  if [[ "${DRY_RUN}" -eq 0 && ! -x "${lima_script}" ]]; then
    fatal "Expected Lima warm script not found at ${lima_script}"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] (cd %s && %s %s)\n' "${INSTALLER_NAME}" "${release_root}" "${lima_script}" "${release_root}" >&2
  else
    (
      cd "${release_root}" &&
      "${lima_script}" "${release_root}"
    )
  fi

  local linux_agent=""
  if [[ -f "${release_root}/bin/linux/world-agent" ]]; then
    linux_agent="${release_root}/bin/linux/world-agent"
  elif [[ -f "${release_root}/bin/world-agent-linux" ]]; then
    linux_agent="${release_root}/bin/world-agent-linux"
  elif [[ -f "${release_root}/bin/world-agent" ]]; then
    # Detect whether the bundled binary is ELF (Linux) or Mach-O (macOS host)
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      linux_agent="${release_root}/bin/world-agent"
    else
      local file_type
      file_type="$(file -b "${release_root}/bin/world-agent" 2>/dev/null || true)"
      if echo "${file_type}" | grep -q "ELF"; then
        linux_agent="${release_root}/bin/world-agent"
      fi
    fi
  fi

  if [[ -z "${linux_agent}" ]]; then
    warn "Linux world-agent binary not found in release bundle; skipping agent install. (Ensure release publishes a Linux build.)"
    return
  fi

  log "Installing Linux world agent inside Lima..."
  run_cmd limactl copy "${linux_agent}" substrate:/tmp/world-agent
  run_cmd limactl shell substrate sudo mv /tmp/world-agent /usr/local/bin/substrate-world-agent
  run_cmd limactl shell substrate sudo chmod 755 /usr/local/bin/substrate-world-agent
  run_cmd limactl shell substrate sudo systemctl enable --now substrate-world-agent
}

provision_linux_world() {
  local version_dir="$1"

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world provisioning (--no-world)."
    return
  fi

  local world_agent=""
  if [[ -x "${version_dir}/bin/world-agent" ]]; then
    world_agent="${version_dir}/bin/world-agent"
  elif [[ -x "${version_dir}/bin/linux/world-agent" ]]; then
    world_agent="${version_dir}/bin/linux/world-agent"
  fi

  if [[ -z "${world_agent}" ]]; then
    fatal "Linux world-agent binary not found in release bundle under ${version_dir}/bin."
  fi

  log "Installing Linux world agent systemd service..."

  local service_path="/etc/systemd/system/substrate-world-agent.service"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] sudo install -Dm0755 %s /usr/local/bin/substrate-world-agent\n' "${INSTALLER_NAME}" "${world_agent}" >&2
    printf '[%s][dry-run] sudo install -d -m0750 /run/substrate /var/lib/substrate\n' "${INSTALLER_NAME}" >&2
    printf '[%s][dry-run] Write systemd unit to %s\n' "${INSTALLER_NAME}" "${service_path}" >&2
    printf '[%s][dry-run] sudo systemctl daemon-reload && sudo systemctl enable --now substrate-world-agent\n' "${INSTALLER_NAME}" >&2
    return
  fi

  run_cmd sudo install -Dm0755 "${world_agent}" /usr/local/bin/substrate-world-agent
  run_cmd sudo install -d -m0750 /run/substrate
  run_cmd sudo install -d -m0750 /var/lib/substrate

  local unit_file
  unit_file="${TMPDIR}/substrate-world-agent.service"
  cat > "${unit_file}" <<'UNIT'
[Unit]
Description=Substrate World Agent
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/substrate-world-agent
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=SUBSTRATE_AGENT_TCP_PORT=61337
RuntimeDirectory=substrate
RuntimeDirectoryMode=0750
StateDirectory=substrate
StateDirectoryMode=0750
WorkingDirectory=/var/lib/substrate
StandardOutput=journal
StandardError=journal
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

[Install]
WantedBy=multi-user.target
UNIT

  run_cmd sudo install -Dm0644 "${unit_file}" "${service_path}"
  run_cmd sudo systemctl daemon-reload
  run_cmd sudo systemctl enable --now substrate-world-agent
  run_cmd sudo systemctl status substrate-world-agent --no-pager --lines=10 || true
}

run_world_checks() {
  local substrate_bin="$1"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s world doctor --json\n' "${INSTALLER_NAME}" "${substrate_bin}" >&2
    return
  fi

  log "Running substrate world doctor..."
  if ! "${substrate_bin}" world doctor --json | jq '.'; then
    warn "World doctor reported issues. Review output above."
  fi
}

install_macos() {
  ensure_macos_prereqs

  local artifact="substrate-v${VERSION}-macos_arm64.tar.gz"
  local archive_path="${TMPDIR}/${artifact}"
  download_artifact "${artifact}" "${archive_path}"

  local checksums_path="${TMPDIR}/SHA256SUMS"
  if download_checksums "${checksums_path}"; then
    verify_checksum "${archive_path}" "${checksums_path}" "${artifact}"
  fi

  local extract_dir="${TMPDIR}/extract"
  extract_archive "${archive_path}" "${extract_dir}"
  local release_root
  release_root="$(find_extracted_root "${extract_dir}")"

  local versions_dir="${PREFIX}/versions"
  local version_dir="${versions_dir}/${VERSION}"
  local bin_dir="${PREFIX}/bin"
  local shim_dir="${PREFIX}/shims"

  log "Installing to ${version_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] mkdir -p %s\n' "${INSTALLER_NAME}" "${versions_dir}" >&2
    printf '[%s][dry-run] rm -rf %s\n' "${INSTALLER_NAME}" "${version_dir}" >&2
    printf '[%s][dry-run] copy contents of %s into %s\n' "${INSTALLER_NAME}" "${release_root}" "${version_dir}" >&2
  else
    mkdir -p "${versions_dir}"
    rm -rf "${version_dir}"
    mkdir -p "${version_dir}"
    cp -R "${release_root}"/. "${version_dir}"/
  fi

  link_binaries "${version_dir}" "${bin_dir}"
  ensure_shell_integration "${shim_dir}" "${bin_dir}"

  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  provision_macos_world "${version_dir}"
  local doctor_path
  doctor_path="${shim_dir}:${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_path}"
  PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"

  log "Installation complete. Open a new terminal or 'source ~/.substrate_bashenv' to refresh PATH."
}

linux_artifact_name() {
  case "${ARCH}" in
    x86_64|amd64)
      printf 'substrate-v%s-linux_x86_64.tar.gz' "${VERSION}"
      ;;
    aarch64|arm64)
      printf 'substrate-v%s-linux_aarch64.tar.gz' "${VERSION}"
      ;;
    *)
      fatal "Unsupported Linux architecture: ${ARCH}"
      ;;
  esac
}

install_linux() {
  ensure_linux_prereqs

  local artifact
  artifact="$(linux_artifact_name)"
  local archive_path="${TMPDIR}/${artifact}"
  download_artifact "${artifact}" "${archive_path}"

  local checksums_path="${TMPDIR}/SHA256SUMS"
  if download_checksums "${checksums_path}"; then
    verify_checksum "${archive_path}" "${checksums_path}" "${artifact}"
  fi

  local extract_dir="${TMPDIR}/extract"
  extract_archive "${archive_path}" "${extract_dir}"
  local release_root
  release_root="$(find_extracted_root "${extract_dir}")"

  local versions_dir="${PREFIX}/versions"
  local version_dir="${versions_dir}/${VERSION}"
  local bin_dir="${PREFIX}/bin"
  local shim_dir="${PREFIX}/shims"

  log "Installing to ${version_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] mkdir -p %s\n' "${INSTALLER_NAME}" "${versions_dir}" >&2
    printf '[%s][dry-run] rm -rf %s\n' "${INSTALLER_NAME}" "${version_dir}" >&2
    printf '[%s][dry-run] copy contents of %s into %s\n' "${INSTALLER_NAME}" "${release_root}" "${version_dir}" >&2
  else
    mkdir -p "${versions_dir}"
    rm -rf "${version_dir}"
    mkdir -p "${version_dir}"
    cp -R "${release_root}"/. "${version_dir}"/
  fi

  link_binaries "${version_dir}" "${bin_dir}"
  ensure_shell_integration "${shim_dir}" "${bin_dir}"

  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  provision_linux_world "${version_dir}"
  local doctor_path
  doctor_path="${shim_dir}:${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_path}"
  PATH="${doctor_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"

  if [[ "${IS_WSL}" -eq 1 ]]; then
    log "Detected WSL environment. Windows host components (forwarder, uninstall) must be managed via PowerShell scripts."
  fi

  log "Installation complete. Open a new terminal or 'source ~/.substrate_bashenv' to refresh PATH."
}

main() {
  sanitize_env_path
  parse_args "$@"
  detect_platform
  prepare_tmpdir

  case "${PLATFORM}" in
    macos)
      install_macos
      ;;
    linux)
      install_linux
      ;;
    windows)
      warn "Automated Windows (PowerShell) installation flow not yet implemented. Refer to docs/install/windows.md."
      exit 2
      ;;
    *)
      fatal "Unsupported platform: ${PLATFORM}"
      ;;
  esac
}

main "$@"
