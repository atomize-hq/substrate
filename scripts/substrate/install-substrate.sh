#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${INSTALLER_NAME:-}" ]]; then
  INSTALLER_NAME="substrate-install"
fi
readonly INSTALLER_NAME
# shellcheck disable=SC2034 # used for release metadata
readonly INSTALLER_VERSION="0.1.0-dev"
readonly DEFAULT_FALLBACK_VERSION="0.2.2"
readonly LATEST_RELEASE_API="${SUBSTRATE_INSTALL_LATEST_API:-https://api.github.com/repos/atomize-hq/substrate/releases/latest}"
readonly DEFAULT_PREFIX="${HOME}/.substrate"
readonly DEFAULT_BASE_URL="https://github.com/atomize-hq/substrate/releases/download"

VERSION_RAW=""
VERSION=""
VERSION_TAG=""
PREFIX="$DEFAULT_PREFIX"
NO_WORLD=0
NO_SHIMS=0
DRY_RUN=0
SYNC_DEPS=0
ARTIFACT_DIR="${SUBSTRATE_INSTALL_ARTIFACT_DIR:-${SUBSTRATE_INSTALL_ARCHIVE:-}}"
BASE_URL="${SUBSTRATE_INSTALL_BASE_URL:-$DEFAULT_BASE_URL}"
TMPDIR=""
PLATFORM=""
ARCH=""
IS_WSL=0
ORIGINAL_PATH="${PATH}"
PKG_MANAGER=""
APT_UPDATED=0
SUDO_CMD=()
MANAGER_ENV_PATH=""
MANAGER_INIT_PATH=""
INSTALL_CONFIG_PATH=""

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
  --version <semver>   Install a specific release (default: latest GitHub release)
  --prefix <path>      Installation prefix (default: ~/.substrate)
  --no-world           Skip world backend provisioning
  --no-shims           Skip shim deployment
  --sync-deps          Run 'substrate world deps sync --all' after provisioning completes
  --dry-run            Print actions without executing
  --artifact-dir <dir> Use pre-downloaded host bundle + SHA256SUMS
  --archive <dir>      Alias for --artifact-dir (deprecated)
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

command_exists() {
  local cmd="$1"
  if command -v "${cmd}" >/dev/null 2>&1; then
    return 0
  fi

  local fallback=""
  case "${cmd}" in
    nft|ip)
      fallback="/usr/sbin/${cmd}"
      ;;
    systemctl)
      fallback="/usr/bin/systemctl"
      ;;
  esac

  if [[ -n "${fallback}" && -x "${fallback}" ]]; then
    return 0
  fi

  return 1
}

require_cmd() {
  local cmd="$1"
  command_exists "${cmd}" || fatal "Required command '${cmd}' not found. Please install it and re-run."
}

initialize_sudo() {
  if [[ ${#SUDO_CMD[@]} -gt 0 ]]; then
    return
  fi

  if [[ "${EUID}" -ne 0 ]]; then
    if command -v sudo >/dev/null 2>&1; then
      SUDO_CMD=(sudo)
    else
      fatal "This installer requires 'sudo' when run as a non-root user. Install sudo or re-run the installer as root."
    fi
  fi
}

detect_package_manager() {
  if [[ -n "${PKG_MANAGER}" ]]; then
    return 0
  fi

  if command -v apt-get >/dev/null 2>&1; then
    PKG_MANAGER="apt-get"
    return 0
  fi
  if command -v dnf >/dev/null 2>&1; then
    PKG_MANAGER="dnf"
    return 0
  fi
  if command -v yum >/dev/null 2>&1; then
    PKG_MANAGER="yum"
    return 0
  fi
  if command -v pacman >/dev/null 2>&1; then
    PKG_MANAGER="pacman"
    return 0
  fi
  if command -v zypper >/dev/null 2>&1; then
    PKG_MANAGER="zypper"
    return 0
  fi

  return 1
}

resolve_package_for_command() {
  local cmd="$1"

  case "${PKG_MANAGER}" in
    apt-get)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs fuse3" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute2" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    dnf|yum)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    pacman)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute2" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    zypper)
      case "${cmd}" in
        curl) echo "curl" ;;
        tar) echo "tar" ;;
        jq) echo "jq" ;;
        fuse-overlayfs) echo "fuse-overlayfs" ;;
        nft) echo "nftables" ;;
        ip) echo "iproute2" ;;
        sha256sum) echo "coreutils" ;;
        systemctl) echo "systemd" ;;
        *) echo "" ;;
      esac
      ;;
    *)
      echo ""
      ;;
  esac
}

install_packages() {
  local packages=()
  packages=("$@")
  if [[ ${#packages[@]} -eq 0 ]]; then
    return
  fi

  case "${PKG_MANAGER}" in
    apt-get)
      log "Installing packages: ${packages[*]}"
      if [[ "${DRY_RUN}" -eq 1 ]]; then
        printf '[%s][dry-run] %s apt-get update\n' "${INSTALLER_NAME}" "${SUDO_CMD[*]:-}" >&2
        printf '[%s][dry-run] %s apt-get install -y %s\n' "${INSTALLER_NAME}" "${SUDO_CMD[*]:-}" "${packages[*]}" >&2
        return
      fi
      if [[ ${APT_UPDATED} -eq 0 ]]; then
        run_cmd "${SUDO_CMD[@]}" apt-get update
        APT_UPDATED=1
      fi
      run_cmd "${SUDO_CMD[@]}" apt-get install -y "${packages[@]}"
      ;;
    dnf)
      log "Installing packages: ${packages[*]}"
      run_cmd "${SUDO_CMD[@]}" dnf install -y "${packages[@]}"
      ;;
    yum)
      log "Installing packages: ${packages[*]}"
      run_cmd "${SUDO_CMD[@]}" yum install -y "${packages[@]}"
      ;;
    pacman)
      log "Installing packages: ${packages[*]}"
      run_cmd "${SUDO_CMD[@]}" pacman -Sy --noconfirm --needed "${packages[@]}"
      ;;
    zypper)
      log "Installing packages: ${packages[*]}"
      run_cmd "${SUDO_CMD[@]}" zypper --non-interactive install "${packages[@]}"
      ;;
    *)
      fatal "Unsupported package manager. Install required commands manually and re-run."
      ;;
  esac
}

ensure_linux_packages_for_commands() {
  initialize_sudo
  local commands=("$@")
  local missing_cmds=()
  for cmd in "${commands[@]}"; do
    if ! command_exists "${cmd}"; then
      missing_cmds+=("${cmd}")
    fi
  done

  if [[ ${#missing_cmds[@]} -eq 0 ]]; then
    return
  fi

  if ! detect_package_manager; then
    fatal "Unable to detect supported package manager. Install required commands (${missing_cmds[*]}) manually and re-run."
  fi

  declare -A pkg_set=()
  local cmd pkg_list
  for cmd in "${missing_cmds[@]}"; do
    pkg_list="$(resolve_package_for_command "${cmd}")"
    if [[ -z "${pkg_list}" ]]; then
      warn "No package mapping for '${cmd}' under ${PKG_MANAGER}; please install it manually."
      continue
    fi
    for pkg in ${pkg_list}; do
      pkg_set["${pkg}"]=1
    done
  done

  if [[ ${#pkg_set[@]} -eq 0 ]]; then
    return
  fi

  local packages=()
  for pkg in "${!pkg_set[@]}"; do
    packages+=("${pkg}")
  done

  install_packages "${packages[@]}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return
  fi

  # Re-check after installation.
  local remaining=()
  for cmd in "${missing_cmds[@]}"; do
    if ! command_exists "${cmd}"; then
      remaining+=("${cmd}")
    fi
  done
  if [[ ${#remaining[@]} -gt 0 ]]; then
    fatal "Unable to install required commands: ${remaining[*]}. Install them manually and re-run."
  fi
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
    local parts=()
    if [[ -n "${PATH}" ]]; then
      IFS=':' read -r -a parts <<< "${PATH}"
    fi
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
      --sync-deps)
        SYNC_DEPS=1
        shift
        ;;
      --artifact-dir|--archive)
        [[ $# -lt 2 ]] && fatal "Missing value for $1"
        ARTIFACT_DIR="$2"
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
}

fetch_latest_release_tag() {
  if ! command -v curl >/dev/null 2>&1; then
    return 1
  fi
  if ! command -v jq >/dev/null 2>&1; then
    return 1
  fi

  local curl_cmd=(curl -fsSL -H "Accept: application/vnd.github+json")
  if [[ -n "${SUBSTRATE_INSTALL_GITHUB_TOKEN:-}" ]]; then
    curl_cmd+=(-H "Authorization: Bearer ${SUBSTRATE_INSTALL_GITHUB_TOKEN}")
  fi

  local response
  if ! response="$("${curl_cmd[@]}" "${LATEST_RELEASE_API}")"; then
    return 1
  fi

  jq -r '.tag_name // empty' <<<"${response}"
}

ensure_version_selected() {
  if [[ -n "${VERSION_TAG}" ]]; then
    return
  fi

  if [[ -z "${VERSION_RAW}" ]]; then
    local resolved_tag=""
    if resolved_tag="$(fetch_latest_release_tag 2>/dev/null)" && [[ -n "${resolved_tag}" ]]; then
      VERSION_RAW="${resolved_tag}"
      log "No --version provided; defaulting to latest release ${resolved_tag}."
    else
      VERSION_RAW="v${DEFAULT_FALLBACK_VERSION}"
      warn "Unable to resolve latest release tag; falling back to ${VERSION_RAW}."
    fi
  else
    log "Using requested version ${VERSION_RAW}."
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

normalize_prefix() {
  if [[ "${PREFIX}" != "/" ]]; then
    PREFIX="${PREFIX%/}"
    if [[ -z "${PREFIX}" ]]; then
      PREFIX="/"
    fi
  fi
}

initialize_metadata_paths() {
  MANAGER_ENV_PATH="${PREFIX}/manager_env.sh"
  MANAGER_INIT_PATH="${PREFIX}/manager_init.sh"
  INSTALL_CONFIG_PATH="${PREFIX}/config.toml"
}

ensure_manager_init_placeholder() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Create manager_init placeholder at %s\n' "${INSTALLER_NAME}" "${MANAGER_INIT_PATH}" >&2
    return
  fi

  local init_dir
  init_dir="$(dirname "${MANAGER_INIT_PATH}")"
  mkdir -p "${init_dir}"
  if [[ -f "${MANAGER_INIT_PATH}" ]]; then
    return
  fi

  cat > "${MANAGER_INIT_PATH}.tmp" <<'EOF'
# Substrate manager init placeholder – this file is replaced at runtime by `substrate`.
EOF
  mv "${MANAGER_INIT_PATH}.tmp" "${MANAGER_INIT_PATH}"
  chmod 0644 "${MANAGER_INIT_PATH}" || true
}

write_manager_env_script() {
  local enabled="$1"
  local state="disabled"
  local enabled_flag="0"
  if [[ "${enabled}" -eq 1 ]]; then
    state="enabled"
    enabled_flag="1"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Write manager_env.sh at %s (world_enabled=%s)\n' "${INSTALLER_NAME}" "${MANAGER_ENV_PATH}" "${state}" >&2
    return
  fi

  local env_dir
  env_dir="$(dirname "${MANAGER_ENV_PATH}")"
  mkdir -p "${env_dir}"
  local today
  today="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
  local manager_env_literal manager_init_literal legacy_literal
  manager_env_literal="$(printf '%q' "${MANAGER_ENV_PATH}")"
  manager_init_literal="$(printf '%q' "${MANAGER_INIT_PATH}")"
  legacy_literal="\${HOME}/.substrate_bashenv"
  cat > "${MANAGER_ENV_PATH}.tmp" <<EOF
#!/usr/bin/env bash
# Managed by ${INSTALLER_NAME} on ${today}
export SUBSTRATE_WORLD=${state}
export SUBSTRATE_WORLD_ENABLED=${enabled_flag}
export SUBSTRATE_MANAGER_ENV=${manager_env_literal}
export SUBSTRATE_MANAGER_INIT=${manager_init_literal}

manager_init_path=${manager_init_literal}
if [[ -f "\${manager_init_path}" ]]; then
  # shellcheck disable=SC1090
  source "\${manager_init_path}"
fi

substrate_original="\${SUBSTRATE_ORIGINAL_BASH_ENV:-}"
if [[ -n "\${substrate_original}" && -f "\${substrate_original}" ]]; then
  # shellcheck disable=SC1090
  source "\${substrate_original}"
fi

legacy_bashenv="${legacy_literal}"
if [[ -f "\${legacy_bashenv}" ]]; then
  # shellcheck disable=SC1090
  source "\${legacy_bashenv}"
fi
EOF
  mv "${MANAGER_ENV_PATH}.tmp" "${MANAGER_ENV_PATH}"
  chmod 0644 "${MANAGER_ENV_PATH}" || true
}

write_install_config() {
  local enabled="$1"
  local flag="false"
  if [[ "${enabled}" -eq 1 ]]; then
    flag="true"
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Write install metadata to %s (world_enabled=%s)\n' "${INSTALLER_NAME}" "${INSTALL_CONFIG_PATH}" "${flag}" >&2
    return
  fi

  local config_dir
  config_dir="$(dirname "${INSTALL_CONFIG_PATH}")"
  mkdir -p "${config_dir}"
  cat > "${INSTALL_CONFIG_PATH}.tmp" <<EOF
[install]
world_enabled = ${flag}

[world]
root_mode = "project"
root_path = ""
caged = true
EOF
  mv "${INSTALL_CONFIG_PATH}.tmp" "${INSTALL_CONFIG_PATH}"
  chmod 0644 "${INSTALL_CONFIG_PATH}" || true
}

finalize_install_metadata() {
  local enabled="$1"
  ensure_manager_init_placeholder
  write_manager_env_script "${enabled}"
  write_install_config "${enabled}"
}

ensure_version_config_present() {
  local version_dir="$1"
  local config_dir="${version_dir}/config"
  local manager_manifest="${config_dir}/manager_hooks.yaml"
  local world_deps="${config_dir}/world-deps.yaml"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] ensure config manifests exist under %s\n' "${INSTALLER_NAME}" "${config_dir}" >&2
    return
  fi

  mkdir -p "${config_dir}"

  if [[ ! -f "${manager_manifest}" ]]; then
    fatal "manager manifest missing from bundle (expected ${manager_manifest})"
  fi

  if [[ ! -f "${world_deps}" ]]; then
    local scripts_world_deps
    scripts_world_deps="${version_dir}/scripts/substrate/world-deps.yaml"
    if [[ -f "${scripts_world_deps}" ]]; then
      cp "${scripts_world_deps}" "${world_deps}"
      log "Staged world-deps manifest under ${config_dir}"
    else
      fatal "world-deps manifest missing from bundle (expected ${world_deps})"
    fi
  fi
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
  ensure_linux_packages_for_commands curl tar jq
  require_cmd curl
  require_cmd tar
  require_cmd jq

  if [[ "${EUID}" -ne 0 ]]; then
    if ! command_exists sudo; then
      fatal "This installer requires 'sudo' when run as a non-root user. Install sudo or re-run the installer as root."
    fi
  fi

  if ! command_exists sha256sum && ! command_exists shasum; then
    ensure_linux_packages_for_commands sha256sum
    if ! command_exists sha256sum && ! command_exists shasum; then
      fatal "Missing sha256sum (preferred) or shasum for checksum verification. Install coreutils/perl-Digest-SHA or rerun with --dry-run."
    fi
  fi

  if [[ "${NO_WORLD}" -eq 0 ]]; then
    ensure_linux_packages_for_commands systemctl fuse-overlayfs nft ip
    require_cmd systemctl
    require_cmd fuse-overlayfs
    require_cmd nft
    require_cmd ip

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

  if [[ -n "${ARTIFACT_DIR}" ]]; then
    local local_path="${ARTIFACT_DIR}/${artifact_name}"
    if [[ ! -f "${local_path}" ]]; then
      fatal "Expected artifact '${artifact_name}' not found in ${ARTIFACT_DIR}."
    fi
    log "Using local artifact: ${local_path}"
    download_file "${local_path}" "${dest_path}"
    return
  fi

  local url="${BASE_URL}/${VERSION_TAG}/${artifact_name}"
  log "Downloading ${artifact_name} from ${url}"
  download_file "${url}" "${dest_path}"
}

download_checksums() {
  local dest_path="$1"

  if [[ -n "${ARTIFACT_DIR}" ]]; then
    local checksum_path="${ARTIFACT_DIR}/SHA256SUMS"
    if [[ -f "${checksum_path}" ]]; then
      download_file "${checksum_path}" "${dest_path}"
      return 0
    fi
    warn "SHA256SUMS not found in ${ARTIFACT_DIR}; skipping checksum verification."
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

target_triple_linux() {
  case "${ARCH}" in
    x86_64|amd64)
      printf 'x86_64-unknown-linux-gnu'
      ;;
    aarch64|arm64)
      printf 'aarch64-unknown-linux-gnu'
      ;;
    *)
      fatal "Unsupported Linux architecture: ${ARCH}"
      ;;
  esac
}

target_triple_macos() {
  case "${ARCH}" in
    arm64)
      printf 'aarch64-apple-darwin'
      ;;
    x86_64|amd64)
      fatal "macOS Intel installs are not supported; use an Apple Silicon host."
      ;;
    *)
      fatal "Unsupported macOS architecture: ${ARCH}"
      ;;
  esac
}

bundle_label_for_target() {
  local target="$1"
  case "${target}" in
    x86_64-unknown-linux-gnu)
      printf 'linux_x86_64'
      ;;
    aarch64-unknown-linux-gnu)
      printf 'linux_aarch64'
      ;;
    x86_64-apple-darwin)
      printf 'macos_x86_64'
      ;;
    aarch64-apple-darwin)
      printf 'macos_arm64'
      ;;
    *)
      fatal "Unsupported release target: ${target}"
      ;;
  esac
}

bundle_archive_name() {
  local label="$1"
  printf 'substrate-v%s-%s.tar.gz' "${VERSION}" "${label}"
}

fetch_bundle_archive() {
  local archive_name="$1"
  local dest_path="$2"

  if [[ -n "${ARTIFACT_DIR}" ]]; then
    if [[ -d "${ARTIFACT_DIR}" && -f "${ARTIFACT_DIR}/${archive_name}" ]]; then
      cp "${ARTIFACT_DIR}/${archive_name}" "${dest_path}"
      return
    fi
    if [[ -f "${ARTIFACT_DIR}" && "$(basename "${ARTIFACT_DIR}")" == "${archive_name}" ]]; then
      cp "${ARTIFACT_DIR}" "${dest_path}"
      return
    fi
    fatal "Expected bundle '${archive_name}' not found in ${ARTIFACT_DIR}."
  fi

  download_artifact "${archive_name}" "${dest_path}"
}

prepare_bundle_payload() {
  local target_triple="$1"
  local release_root="$2"
  local checksums_path="$3"

  local label
  label="$(bundle_label_for_target "${target_triple}")"
  local archive_name
  archive_name="$(bundle_archive_name "${label}")"
  local archive_path="${TMPDIR}/${archive_name}"

  fetch_bundle_archive "${archive_name}" "${archive_path}"
  if [[ -n "${checksums_path}" ]]; then
    verify_checksum "${archive_path}" "${checksums_path}" "${archive_name}"
  fi

  local extract_dir="${TMPDIR}/bundle-${label}"
  rm -rf "${extract_dir}"
  extract_archive "${archive_path}" "${extract_dir}"
  local bundle_root
  bundle_root="$(find_extracted_root "${extract_dir}")"

  rm -rf "${release_root}"
  mkdir -p "${release_root}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] stage bundle contents from %s into %s\n' "${INSTALLER_NAME}" "${bundle_root}" "${release_root}" >&2
    return
  fi
  cp -R "${bundle_root}/." "${release_root}/"
}

extract_archive() {
  local archive_path="$1"
  local dest_dir="$2"

  mkdir -p "${dest_dir}"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] extract %s -> %s\n' "${INSTALLER_NAME}" "${archive_path}" "${dest_dir}" >&2
    return 0
  fi

  case "${archive_path}" in
    *.tar.gz|*.tgz)
      tar -xzf "${archive_path}" -C "${dest_dir}"
      ;;
    *.tar.xz|*.txz)
      tar -xJf "${archive_path}" -C "${dest_dir}"
      ;;
    *.zip)
      unzip -q "${archive_path}" -d "${dest_dir}"
      ;;
    *)
      fatal "Unsupported archive format: ${archive_path}"
      ;;
  esac
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

deploy_shims() {
  local substrate_bin="$1"
  if [[ "${NO_SHIMS}" -eq 1 ]]; then
    log "Skipping shim deployment (--no-shims)."
    return
  fi

  log "Deploying shims..."
  run_cmd "${substrate_bin}" --shim-deploy
}

harden_shim_symlinks() {
  local shims_dir="$1"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Normalize shims in %s to real binaries\n' "${INSTALLER_NAME}" "${shims_dir}" >&2
    return
  fi

  if [[ ! -d "${shims_dir}" ]]; then
    return
  fi

  local converted=0
  while IFS= read -r -d '' shim_path; do
    local link_target
    link_target="$(readlink "${shim_path}")" || continue

    local resolved_target
    if [[ "${link_target}" == /* ]]; then
      resolved_target="${link_target}"
    else
      local shim_dirname
      shim_dirname="$(cd "$(dirname "${shim_path}")" && pwd -P)"
      resolved_target="${shim_dirname}/${link_target}"
    fi

    if [[ ! -e "${resolved_target}" ]]; then
      continue
    fi

    rm -f "${shim_path}"
    if ! ln "${resolved_target}" "${shim_path}" 2>/dev/null; then
      cp "${resolved_target}" "${shim_path}"
      chmod +x "${shim_path}" 2>/dev/null || true
    fi
    converted=1
  done < <(find "${shims_dir}" -maxdepth 1 -type l -print0 2>/dev/null)

  if [[ ${converted} -eq 1 ]]; then
    log "Normalized shim binaries in ${shims_dir}"
  fi
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
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      world_agent="${version_dir}/bin/world-agent"
      warn "Linux world-agent binary not found in release bundle; using placeholder path for dry run."
    else
      fatal "Linux world-agent binary not found in release bundle under ${version_dir}/bin."
    fi
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

  local home_path
  if [[ -n "${HOME}" ]]; then
    home_path="$(cd "${HOME}" && pwd)"
  else
    home_path="/home"
  fi

  local unit_file
  unit_file="${TMPDIR}/substrate-world-agent.service"
  cat > "${unit_file}" <<UNIT
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
ReadWritePaths=${home_path} /var/lib/substrate /run /run/substrate /sys/fs/cgroup /tmp
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

[Install]
WantedBy=multi-user.target
UNIT

  run_cmd sudo install -Dm0644 "${unit_file}" "${service_path}"
  run_cmd sudo systemctl daemon-reload
  run_cmd sudo systemctl enable --now substrate-world-agent
  run_cmd sudo systemctl restart substrate-world-agent
  run_cmd sudo systemctl status substrate-world-agent --no-pager --lines=10 || true
}

run_world_checks() {
  local substrate_bin="$1"
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world doctor (--no-world)."
    return
  fi
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s world doctor --json\n' "${INSTALLER_NAME}" "${substrate_bin}" >&2
    return
  fi

  log "Running substrate world doctor..."
  if ! "${substrate_bin}" world doctor --json | jq '.'; then
    warn "World doctor reported issues. Review output above."
  fi
}

sync_world_deps() {
  local substrate_bin="$1"
  if [[ "${SYNC_DEPS}" -ne 1 ]]; then
    return
  fi
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world dependency sync because --no-world was used."
    return
  fi
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s world deps sync --all --verbose\n' "${INSTALLER_NAME}" "${substrate_bin}" >&2
    return
  fi

  log "Syncing guest dependencies via 'substrate world deps sync --all'..."
  if ! "${substrate_bin}" world deps sync --all --verbose; then
    warn "world deps sync failed; run 'substrate world deps sync --all' later to finish provisioning."
  fi
}

install_macos() {
  ensure_macos_prereqs
  ensure_version_selected

  local target_triple
  target_triple="$(target_triple_macos)"

  local release_root="${TMPDIR}/payload"
  local checksums_path="${TMPDIR}/SHA256SUMS"
  if ! download_checksums "${checksums_path}"; then
    checksums_path=""
  fi

  prepare_bundle_payload "${target_triple}" "${release_root}" "${checksums_path}"

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

  ensure_version_config_present "${version_dir}"

  link_binaries "${version_dir}" "${bin_dir}"

  local world_enabled=1
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    world_enabled=0
  fi

  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  harden_shim_symlinks "${shim_dir}"
  provision_macos_world "${version_dir}"
  local doctor_original_path
  doctor_original_path="${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_original_path}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" sync_world_deps "${substrate_bin}"

  finalize_install_metadata "${world_enabled}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "Installation complete (dry run). After a real install add ${bin_dir} to your PATH or run ${bin_dir}/substrate directly."
  else
    log "Installation complete. Add ${bin_dir} to your PATH or invoke ${bin_dir}/substrate directly."
  fi
  log "manager_init placeholder: ${MANAGER_INIT_PATH}"
  log "manager_env script: ${MANAGER_ENV_PATH}"
  log "config manifests: ${version_dir}/config"
  log "install metadata: ${INSTALL_CONFIG_PATH}"

  if [[ "${world_enabled}" -eq 1 ]]; then
    log "World backend enabled; run '${bin_dir}/substrate world doctor --json' or '${bin_dir}/substrate world deps sync --all' as needed."
  else
    log "World backend disabled (--no-world). Run '${bin_dir}/substrate world enable --prefix \"${PREFIX}\"' when you are ready to provision."
  fi
}

install_linux() {
  ensure_linux_prereqs
  ensure_version_selected

  local target_triple
  target_triple="$(target_triple_linux)"

  local release_root="${TMPDIR}/payload"
  local checksums_path="${TMPDIR}/SHA256SUMS"
  if ! download_checksums "${checksums_path}"; then
    checksums_path=""
  fi

  prepare_bundle_payload "${target_triple}" "${release_root}" "${checksums_path}"

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

  ensure_version_config_present "${version_dir}"

  link_binaries "${version_dir}" "${bin_dir}"

  local world_enabled=1
  if [[ "${NO_WORLD}" -eq 1 ]]; then
    world_enabled=0
  fi

  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  harden_shim_symlinks "${shim_dir}"
  provision_linux_world "${version_dir}"
  local doctor_original_path
  doctor_original_path="${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_original_path}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" sync_world_deps "${substrate_bin}"

  finalize_install_metadata "${world_enabled}"

  if [[ "${IS_WSL}" -eq 1 ]]; then
    log "Detected WSL environment. Windows host components (forwarder, uninstall) must be managed via PowerShell scripts."
  fi

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    log "Installation complete (dry run). After a real install add ${bin_dir} to your PATH or run ${bin_dir}/substrate directly."
  else
    log "Installation complete. Add ${bin_dir} to your PATH or invoke ${bin_dir}/substrate directly."
  fi
  log "manager_init placeholder: ${MANAGER_INIT_PATH}"
  log "manager_env script: ${MANAGER_ENV_PATH}"
  log "config manifests: ${version_dir}/config"
  log "install metadata: ${INSTALL_CONFIG_PATH}"

  if [[ "${world_enabled}" -eq 1 ]]; then
    log "World backend enabled; run '${bin_dir}/substrate world doctor --json' for diagnostics or '${bin_dir}/substrate world deps sync --all' to mirror host tools."
  else
    log "World backend disabled (--no-world). Run '${bin_dir}/substrate world enable --prefix \"${PREFIX}\"' when you are ready to provision."
  fi
}

main() {
  sanitize_env_path
  parse_args "$@"
  normalize_prefix
  initialize_metadata_paths
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

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  main "$@"
fi
