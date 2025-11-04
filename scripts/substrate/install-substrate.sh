#!/usr/bin/env bash
set -euo pipefail

# ============================================================================
# SUBSTRATE INSTALLER - Main installation script for Substrate
# ============================================================================
# This script handles the complete installation process for Substrate across
# macOS and Linux platforms. It downloads binaries, sets up shell integration,
# deploys shims, and provisions the world backend (Lima VM on macOS, systemd
# service on Linux).
#
# Key responsibilities:
# - Platform detection (macOS, Linux, WSL)
# - Prerequisite checking and package installation
# - Binary download and checksum verification
# - Shell environment configuration
# - Shim deployment for command interception
# - World backend provisioning (virtualization layer)
# ============================================================================

# ----------------------------------------------------------------------------
# CONSTANTS - Immutable configuration values
# ----------------------------------------------------------------------------
readonly INSTALLER_NAME="substrate-install"
readonly INSTALLER_VERSION="0.1.0-dev"
readonly DEFAULT_VERSION="0.2.0-beta"
readonly DEFAULT_PREFIX="${HOME}/.substrate"
readonly DEFAULT_BASE_URL="https://github.com/atomize-hq/substrate/releases/download"

# ----------------------------------------------------------------------------
# GLOBAL VARIABLES - Mutable state throughout installation
# ----------------------------------------------------------------------------
VERSION_RAW=""          # Version string as provided by user (may include 'v' prefix)
VERSION=""              # Normalized version without 'v' prefix
VERSION_TAG=""          # Git tag format (v0.2.0-beta)
PREFIX="$DEFAULT_PREFIX"  # Installation directory (default: ~/.substrate)
NO_WORLD=0              # Flag to skip world backend provisioning
NO_SHIMS=0              # Flag to skip shim deployment
DRY_RUN=0               # Flag for dry-run mode (print actions without executing)
ARCHIVE_OVERRIDE="${SUBSTRATE_INSTALL_ARCHIVE:-}"  # Use local archive instead of downloading
BASE_URL="${SUBSTRATE_INSTALL_BASE_URL:-$DEFAULT_BASE_URL}"  # Release download URL
TMPDIR=""               # Temporary directory for downloads and extraction
PLATFORM=""             # Detected platform (macos, linux, windows)
ARCH=""                 # Detected architecture (arm64, x86_64, aarch64)
IS_WSL=0                # Flag indicating Windows Subsystem for Linux
ORIGINAL_PATH="${PATH}" # Original PATH before any modifications
PKG_MANAGER=""          # Detected package manager (apt-get, dnf, yum, pacman, zypper)
APT_UPDATED=0           # Flag to track if apt-get update has been run
SUDO_CMD=()             # Array holding sudo command if needed (empty if root)

# ============================================================================
# LOGGING FUNCTIONS - Output formatting and error handling
# ============================================================================

# log: Print informational messages to stderr
log() {
  printf '[%s] %s\n' "${INSTALLER_NAME}" "$*" >&2
}

# warn: Print warning messages to stderr (non-fatal issues)
warn() {
  printf '[%s][WARN] %s\n' "${INSTALLER_NAME}" "$*" >&2
}

# fatal: Print error message to stderr and exit with code 1
# This immediately terminates the installation process
fatal() {
  printf '[%s][ERROR] %s\n' "${INSTALLER_NAME}" "$*" >&2
  exit 1
}

# print_usage: Display help text with available options
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

# cleanup: Remove temporary directory on script exit
# This is called automatically via trap on EXIT
cleanup() {
  if [[ -n "${TMPDIR}" && -d "${TMPDIR}" && "${DRY_RUN}" -eq 0 ]]; then
    rm -rf "${TMPDIR}"
  fi
}

# run_cmd: Execute a command, or print it in dry-run mode
# In dry-run mode, commands are printed but not executed
run_cmd() {
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] %s\n' "${INSTALLER_NAME}" "$*" >&2
    return 0
  fi
  "$@"
}

# ============================================================================
# COMMAND AVAILABILITY CHECKING
# ============================================================================

# command_exists: Check if a command is available in PATH or at known locations
# Some system utilities (nft, ip, systemctl) may not be in user PATH but exist
# at standard locations like /usr/sbin or /usr/bin
command_exists() {
  local cmd="$1"
  if command -v "${cmd}" >/dev/null 2>&1; then
    return 0
  fi

  # Check known fallback locations for system utilities
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

# require_cmd: Ensure a command exists or exit with fatal error
# Used for critical dependencies that cannot be auto-installed
require_cmd() {
  local cmd="$1"
  command_exists "${cmd}" || fatal "Required command '${cmd}' not found. Please install it and re-run."
}

# ============================================================================
# PRIVILEGE ESCALATION - Configure sudo if needed
# ============================================================================

# initialize_sudo: Set up sudo command array for operations requiring root
# If already root (EUID=0), SUDO_CMD remains empty
# If not root, verify sudo is available and populate SUDO_CMD array
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

# ============================================================================
# PACKAGE MANAGER DETECTION AND OPERATIONS
# ============================================================================
# Supports multiple Linux package managers: apt-get, dnf, yum, pacman, zypper
# Enables automatic installation of missing dependencies

# detect_package_manager: Identify which package manager is available
# Sets PKG_MANAGER global variable to the detected package manager
# Returns 0 if found, 1 if no supported package manager is available
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

# resolve_package_for_command: Map command names to distribution-specific package names
# Different Linux distributions name packages differently (e.g., iproute vs iproute2)
# Args: $1 = command name (e.g., "curl", "nft", "systemctl")
# Returns: Space-separated list of package names for the current PKG_MANAGER
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

# install_packages: Install packages using the detected package manager
# Handles package manager-specific commands and flags
# For apt-get, runs update once before first installation
# Args: $@ = list of package names to install
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
      # Run apt-get update once per installation session
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

# ensure_linux_packages_for_commands: High-level dependency resolution
# Checks which commands are missing, maps them to packages, installs packages,
# then verifies installation succeeded
# Args: $@ = list of command names that must be available
#
# Flow:
# 1. Check which commands are missing
# 2. Map commands to packages via resolve_package_for_command
# 3. Deduplicate packages (multiple commands may come from same package)
# 4. Install all packages
# 5. Verify commands are now available
ensure_linux_packages_for_commands() {
  initialize_sudo
  local commands=("$@")
  local missing_cmds=()

  # Identify which commands are not available
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

  # Map commands to packages and deduplicate using associative array
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

  # Convert associative array keys to regular array
  local packages=()
  for pkg in "${!pkg_set[@]}"; do
    packages+=("${pkg}")
  done

  install_packages "${packages[@]}"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    return
  fi

  # Verify installation: re-check if commands are now available
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

# ============================================================================
# FILE UTILITIES - Checksum verification and PATH manipulation
# ============================================================================

# compute_file_sha256: Calculate SHA256 hash of a file
# Tries sha256sum first (Linux standard), falls back to shasum (macOS)
# Args: $1 = file path
# Returns: SHA256 hash string
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

# sanitize_env_path: Remove shim directory from PATH to avoid recursive calls
# When substrate binaries call system commands, we need the "real" PATH
# without our shim directory to prevent infinite loops
# Uses SHIM_ORIGINAL_PATH if available, otherwise filters out shim dir
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

# ============================================================================
# PLATFORM DETECTION - Identify OS and architecture
# ============================================================================

# detect_platform: Determine operating system and architecture
# Sets global variables: PLATFORM (macos|linux|windows), ARCH (arm64|x86_64|aarch64)
# Also detects WSL (Windows Subsystem for Linux) by checking /proc/version
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
      # Detect WSL by looking for "microsoft" in kernel version
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

# ============================================================================
# COMMAND-LINE ARGUMENT PARSING
# ============================================================================

# parse_args: Process command-line flags and options
# Sets global variables based on provided arguments
# Normalizes version string (removes 'v' prefix, adds it back for VERSION_TAG)
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
        NO_WORLD=1  # Skip Lima VM (macOS) or systemd service (Linux) setup
        shift
        ;;
      --no-shims)
        NO_SHIMS=1  # Skip shim binary deployment
        shift
        ;;
      --dry-run)
        DRY_RUN=1  # Print actions without executing them
        shift
        ;;
      --archive)
        [[ $# -lt 2 ]] && fatal "Missing value for --archive"
        ARCHIVE_OVERRIDE="$2"  # Use local archive for testing
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

  # Default to DEFAULT_VERSION if not specified
  if [[ -z "${VERSION_RAW}" ]]; then
    VERSION_RAW="${DEFAULT_VERSION}"
    warn "No --version provided; defaulting to ${VERSION_RAW}"
  fi

  # Normalize version: strip 'v' prefix if present
  VERSION="${VERSION_RAW#v}"
  if [[ -z "${VERSION}" ]]; then
    fatal "Unable to determine version from '${VERSION_RAW}'"
  fi
  # VERSION_TAG is used for GitHub release URLs (includes 'v' prefix)
  VERSION_TAG="v${VERSION}"
}

# ============================================================================
# TEMPORARY DIRECTORY MANAGEMENT
# ============================================================================

# prepare_tmpdir: Create temporary directory for downloads
# Sets up cleanup trap to remove directory on script exit
prepare_tmpdir() {
  TMPDIR="$(mktemp -d -t substrate-install.XXXXXX)"
  trap cleanup EXIT
}

# ============================================================================
# PREREQUISITE CHECKING - Platform-specific dependency verification
# ============================================================================

# ensure_macos_prereqs: Verify all macOS requirements are met
# Required tools: sw_vers, sysctl, curl, tar, shasum, jq, limactl, envsubst
# System requirements:
#   - Virtualization Framework enabled (kern.hv_support=1)
#   - arm64 architecture (Apple Silicon)
ensure_macos_prereqs() {
  require_cmd sw_vers    # macOS version info
  require_cmd sysctl     # System configuration
  require_cmd curl       # Download files
  require_cmd tar        # Extract archives
  require_cmd shasum     # Checksum verification
  require_cmd jq         # JSON parsing for doctor output
  require_cmd limactl    # Lima VM manager (critical for world backend)
  require_cmd envsubst   # Environment variable substitution

  # Check if Virtualization Framework is available
  local hv_support
  hv_support="$(sysctl -n kern.hv_support 2>/dev/null || true)"
  if [[ "${hv_support}" != "1" ]]; then
    fatal "macOS virtualization not available. Enable Virtualization Framework in System Settings."
  fi

  # Currently only arm64 (Apple Silicon) is supported
  if [[ "${ARCH}" != "arm64" ]]; then
    fatal "Only macOS arm64 is currently supported."
  fi
}

# ensure_linux_prereqs: Verify all Linux requirements are met
# Basic requirements: curl, tar, jq, sha256sum/shasum
# World backend requirements (if NO_WORLD=0):
#   - systemctl, fuse-overlayfs, nft, ip
#   - systemd as PID 1
ensure_linux_prereqs() {
  # Install basic tools if missing (auto-installs via package manager)
  ensure_linux_packages_for_commands curl tar jq
  require_cmd curl
  require_cmd tar
  require_cmd jq

  # Verify sudo availability for non-root users
  if [[ "${EUID}" -ne 0 ]]; then
    if ! command_exists sudo; then
      fatal "This installer requires 'sudo' when run as a non-root user. Install sudo or re-run the installer as root."
    fi
  fi

  # Ensure checksum verification tool is available
  if ! command_exists sha256sum && ! command_exists shasum; then
    ensure_linux_packages_for_commands sha256sum
    if ! command_exists sha256sum && ! command_exists shasum; then
      fatal "Missing sha256sum (preferred) or shasum for checksum verification. Install coreutils/perl-Digest-SHA or rerun with --dry-run."
    fi
  fi

  # World backend prerequisites (Linux container runtime requirements)
  if [[ "${NO_WORLD}" -eq 0 ]]; then
    ensure_linux_packages_for_commands systemctl fuse-overlayfs nft ip
    require_cmd systemctl       # systemd service manager
    require_cmd fuse-overlayfs  # Rootless container filesystem
    require_cmd nft             # nftables for network filtering
    require_cmd ip              # Network configuration

    # Verify systemd is the init system (PID 1)
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

# ============================================================================
# DOWNLOAD AND VERIFICATION - Fetch artifacts and verify integrity
# ============================================================================

# download_file: Download a file from URL or copy from local path
# Args: $1 = source (URL or local path), $2 = destination path
# Supports both HTTP(S) URLs and local file paths
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

# download_artifact: Download a specific release artifact
# If ARCHIVE_OVERRIDE is set, uses local archive instead of downloading
# Args: $1 = artifact name, $2 = destination path
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

  # Construct GitHub release URL
  local url="${BASE_URL}/${VERSION_TAG}/${artifact_name}"
  log "Downloading ${artifact_name} from ${url}"
  download_file "${url}" "${dest_path}"
}

# download_checksums: Download SHA256SUMS file from release
# Skipped if using local archive override
# Args: $1 = destination path for SHA256SUMS file
# Returns: 0 on success, 1 if checksum file unavailable
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

# verify_checksum: Verify archive integrity using SHA256
# Compares expected checksum from SHA256SUMS with actual file hash
# Args: $1 = archive path, $2 = SHA256SUMS path, $3 = artifact name
verify_checksum() {
  local archive_path="$1"
  local checksums_path="$2"
  local artifact_name="$3"

  if [[ ! -f "${checksums_path}" ]]; then
    warn "Checksum file missing; skipping verification."
    return
  fi

  # Extract expected checksum for this artifact
  local expected
  expected="$(grep "  ${artifact_name}$" "${checksums_path}" | awk '{print $1}' || true)"
  if [[ -z "${expected}" ]]; then
    warn "Checksum entry for ${artifact_name} not found; skipping verification."
    return
  fi

  # Calculate actual checksum
  local actual
  actual="$(compute_file_sha256 "${archive_path}")"

  if [[ "${expected}" != "${actual}" ]]; then
    fatal "Checksum mismatch for ${artifact_name}: expected ${expected}, got ${actual}"
  fi
  log "Checksum verified for ${artifact_name}"
}

# ============================================================================
# ARCHIVE EXTRACTION - Unpack release artifacts
# ============================================================================

# extract_archive: Extract tar.gz archive to destination directory
# Args: $1 = archive path, $2 = destination directory
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

# find_extracted_root: Locate root directory of extracted archive
# Some archives extract to a single directory, others extract contents directly
# Args: $1 = extraction directory
# Returns: Path to the root directory containing the release files
find_extracted_root() {
  local dest_dir="$1"
  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '%s\n' "${dest_dir}/SIMULATED_ROOT"
    return
  fi

  # Find all top-level entries in extraction directory
  local entries=()
  while IFS= read -r entry; do
    entries+=("${entry}")
  done < <(find "${dest_dir}" -mindepth 1 -maxdepth 1 -print)

  if [[ ${#entries[@]} -eq 0 ]]; then
    fatal "Failed to determine extracted archive root."
  fi

  # If single directory, use it; otherwise use extraction directory itself
  if [[ ${#entries[@]} -eq 1 && -d "${entries[0]}" ]]; then
    printf '%s\n' "${entries[0]}"
  else
    printf '%s\n' "${dest_dir}"
  fi
}

# ============================================================================
# BINARY INSTALLATION - Link binaries into PATH
# ============================================================================

# link_binaries: Create symlinks from version-specific bin to main bin directory
# Structure: ~/.substrate/versions/X.Y.Z/bin/* -> ~/.substrate/bin/*
# This allows version switching by changing which version dir is linked
# Args: $1 = version-specific directory, $2 = main bin directory
link_binaries() {
  local version_dir="$1"
  local bin_dir="$2"

  if [[ "${DRY_RUN}" -eq 1 ]]; then
    printf '[%s][dry-run] Linking binaries from %s into %s\n' "${INSTALLER_NAME}" "${version_dir}/bin" "${bin_dir}" >&2
    return
  fi

  mkdir -p "${bin_dir}"
  # Remove old symlinks before creating new ones
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

# ============================================================================
# SHELL INTEGRATION - Configure shell environment for Substrate
# ============================================================================

# ensure_shell_integration: Set up PATH and BASH_ENV for shell integration
# Creates ~/.substrate_bashenv with PATH modifications
# Handles BASH_ENV chaining if user already has one configured
# Adds sourcing snippet to shell rc files (.zshrc, .bashrc, .bash_profile)
#
# Key concepts:
# - SHIM_ORIGINAL_PATH: Stores the original PATH before shim dir is added
# - BASH_ENV: Makes non-interactive shells (like subprocess spawns) have proper PATH
# - Trampoline: Chains existing BASH_ENV with substrate's BASH_ENV
#
# Args: $1 = shim directory, $2 = bin directory
ensure_shell_integration() {
  local shim_dir="$1"
  local bin_dir="$2"
  local bashenv="${HOME}/.substrate_bashenv"
  local tramp="${HOME}/.substrate_bashenv_trampoline"
  local existing_be="${BASH_ENV:-}"
  local today
  today="$(date)"

  # Create the main bashenv file that sets PATH
  if [[ "${DRY_RUN}" -eq 0 ]]; then
    mkdir -p "$(dirname "${bashenv}")"
    cat > "${bashenv}.tmp" <<EOF
# Generated by ${INSTALLER_NAME} on ${today}
export SUBSTRATE_ROOT="${PREFIX}"
if [ -z "\${SHIM_ORIGINAL_PATH:-}" ]; then
  export SHIM_ORIGINAL_PATH="\${PATH}"
fi
export PATH="${shim_dir}:${bin_dir}:\${SHIM_ORIGINAL_PATH}"
export SHIM_ORIGINAL_PATH
EOF
    mv "${bashenv}.tmp" "${bashenv}"
  else
    printf '[%s][dry-run] Update %s with PATH exports\n' "${INSTALLER_NAME}" "${bashenv}" >&2
  fi

  # If user already has BASH_ENV set, create trampoline to chain them
  local target_bash_env="${bashenv}"

  if [[ -n "${existing_be}" ]]; then
    target_bash_env="${tramp}"
    local expanded_be="${existing_be}"
    # Expand tilde to home directory
    if [[ "${expanded_be}" == ~* ]]; then
      expanded_be="${expanded_be/#~/$HOME}"
    fi
    # Expand environment variables if envsubst available
    if command -v envsubst >/dev/null 2>&1; then
      expanded_be="$(envsubst <<<"${expanded_be}")"
    fi
    local escaped_be
    escaped_be="$(printf '%q' "${expanded_be}")"

    # Create trampoline that sources both existing BASH_ENV and substrate's
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

  # Snippet to add to shell rc files
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

  # Add snippet to all common shell rc files
  local shells=( "${HOME}/.zshrc" "${HOME}/.bashrc" "${HOME}/.bash_profile" )
  for shell_rc in "${shells[@]}"; do
    if [[ "${DRY_RUN}" -eq 1 ]]; then
      printf '[%s][dry-run] Ensure PATH/BASH_ENV snippet in %s\n' "${INSTALLER_NAME}" "${shell_rc}" >&2
      continue
    fi

    if [[ -f "${shell_rc}" ]]; then
      # Only add if not already present
      if ! grep -Fq 'source "$HOME/.substrate_bashenv"' "${shell_rc}"; then
        printf '\n%s\n' "${snippet}" >> "${shell_rc}"
      fi
    else
      # Create new rc file with snippet
      printf '#!/usr/bin/env bash\n%s\n' "${snippet}" > "${shell_rc}"
    fi
  done

  # Apply environment changes to current shell session
  if [[ "${DRY_RUN}" -eq 0 ]]; then
    if [ -f "${bashenv}" ]; then
      # shellcheck disable=SC1090
      source "${bashenv}" 2>/dev/null || true
    fi
    export SUBSTRATE_ORIGINAL_BASH_ENV="${existing_be}"
    export BASH_ENV="${target_bash_env}"
  fi
}

# ============================================================================
# SHIM MANAGEMENT - Deploy and configure command interception shims
# ============================================================================

# deploy_shims: Call substrate binary to deploy shims
# Shims intercept common commands (node, python, etc.) and route them through substrate
# Skipped if --no-shims flag is set
# Args: $1 = path to substrate binary
deploy_shims() {
  local substrate_bin="$1"
  if [[ "${NO_SHIMS}" -eq 1 ]]; then
    log "Skipping shim deployment (--no-shims)."
    return
  fi

  log "Deploying shims..."
  run_cmd "${substrate_bin}" --shim-deploy
}

# harden_shim_symlinks: Convert shim symlinks to hard links or copies
# Symlinks can break if the target is moved; hard links are more robust
# If hard linking fails (e.g., across filesystems), falls back to copying
# This ensures shims remain functional even if substrate installation changes
# Args: $1 = shims directory
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
  # Find all symlinks in shims directory
  while IFS= read -r -d '' shim_path; do
    local link_target
    link_target="$(readlink "${shim_path}")" || continue

    # Resolve relative symlinks to absolute paths
    local resolved_target
    if [[ "${link_target}" == /* ]]; then
      resolved_target="${link_target}"
    else
      local shim_dirname
      shim_dirname="$(cd "$(dirname "${shim_path}")" && pwd -P)"
      resolved_target="${shim_dirname}/${link_target}"
    fi

    # Skip if target doesn't exist
    if [[ ! -e "${resolved_target}" ]]; then
      continue
    fi

    # Replace symlink with hard link, or copy if hard link fails
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

# ============================================================================
# WORLD BACKEND PROVISIONING - Set up virtualization layer
# ============================================================================
# The "world" backend is the virtualization/containerization layer that allows
# substrate to run isolated environments:
# - macOS: Uses Lima (Linux VM running under macOS Virtualization Framework)
# - Linux: Uses systemd service with fuse-overlayfs for rootless containers

# provision_macos_world: Set up Lima VM and install world-agent inside it
# Lima provides a lightweight Linux VM that substrate uses for containerization
# The world-agent systemd service inside Lima handles container operations
# Args: $1 = release root directory containing scripts and binaries
provision_macos_world() {
  local release_root="$1"

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world provisioning (--no-world)."
    return
  fi

  log "Provisioning macOS Lima world backend..."

  # Run Lima warm-up script to create and configure VM
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

  # Locate Linux world-agent binary (runs inside Lima VM, not on macOS host)
  local linux_agent=""
  if [[ -f "${release_root}/bin/linux/world-agent" ]]; then
    linux_agent="${release_root}/bin/linux/world-agent"
  elif [[ -f "${release_root}/bin/world-agent-linux" ]]; then
    linux_agent="${release_root}/bin/world-agent-linux"
  elif [[ -f "${release_root}/bin/world-agent" ]]; then
    # Disambiguate: is this ELF (Linux) or Mach-O (macOS)?
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

  # Install world-agent inside Lima VM and enable systemd service
  log "Installing Linux world agent inside Lima..."
  run_cmd limactl copy "${linux_agent}" substrate:/tmp/world-agent
  run_cmd limactl shell substrate sudo mv /tmp/world-agent /usr/local/bin/substrate-world-agent
  run_cmd limactl shell substrate sudo chmod 755 /usr/local/bin/substrate-world-agent
  run_cmd limactl shell substrate sudo systemctl enable --now substrate-world-agent
}

# provision_linux_world: Install world-agent as systemd service on Linux host
# On Linux, substrate runs directly on the host (no VM needed like macOS)
# The world-agent service handles containerization using fuse-overlayfs
# Requires elevated capabilities for network/container management
# Args: $1 = version directory containing world-agent binary
provision_linux_world() {
  local version_dir="$1"

  if [[ "${NO_WORLD}" -eq 1 ]]; then
    log "Skipping world provisioning (--no-world)."
    return
  fi

  # Locate world-agent binary in release bundle
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

  # Install binary and create required directories
  run_cmd sudo install -Dm0755 "${world_agent}" /usr/local/bin/substrate-world-agent
  run_cmd sudo install -d -m0750 /run/substrate      # Runtime state
  run_cmd sudo install -d -m0750 /var/lib/substrate  # Persistent state

  # Determine user's home directory for ReadWritePaths
  local home_path
  if [[ -n "${HOME}" ]]; then
    home_path="$(cd "${HOME}" && pwd)"
  else
    home_path="/home"
  fi

  # Create systemd unit file with security hardening
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
# Capabilities needed for container/network management
CapabilityBoundingSet=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE
AmbientCapabilities=CAP_NET_ADMIN CAP_NET_BIND_SERVICE CAP_SYS_ADMIN CAP_SYS_CHROOT CAP_DAC_OVERRIDE

[Install]
WantedBy=multi-user.target
UNIT

  # Install and start service
  run_cmd sudo install -Dm0644 "${unit_file}" "${service_path}"
  run_cmd sudo systemctl daemon-reload
  run_cmd sudo systemctl enable --now substrate-world-agent
  run_cmd sudo systemctl restart substrate-world-agent
  run_cmd sudo systemctl status substrate-world-agent --no-pager --lines=10 || true
}

# ============================================================================
# HEALTH CHECKS - Verify installation and world backend status
# ============================================================================

# run_world_checks: Run substrate world doctor to verify world backend health
# The doctor command checks:
# - VM/systemd service status
# - Network connectivity
# - Container runtime functionality
# Args: $1 = path to substrate binary
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

# ============================================================================
# PLATFORM-SPECIFIC INSTALLATION FUNCTIONS
# ============================================================================

# install_macos: Complete installation workflow for macOS
# Flow:
#   1. Check prerequisites (limactl, virtualization, etc.)
#   2. Download and verify release artifact
#   3. Extract to versioned directory (~/.substrate/versions/X.Y.Z/)
#   4. Link binaries to ~/.substrate/bin
#   5. Configure shell integration
#   6. Deploy shims
#   7. Provision Lima VM with world-agent
#   8. Run health checks
install_macos() {
  ensure_macos_prereqs

  # Download release artifact for macOS arm64
  local artifact="substrate-v${VERSION}-macos_arm64.tar.gz"
  local archive_path="${TMPDIR}/${artifact}"
  download_artifact "${artifact}" "${archive_path}"

  # Verify checksum if available
  local checksums_path="${TMPDIR}/SHA256SUMS"
  if download_checksums "${checksums_path}"; then
    verify_checksum "${archive_path}" "${checksums_path}" "${artifact}"
  fi

  # Extract archive
  local extract_dir="${TMPDIR}/extract"
  extract_archive "${archive_path}" "${extract_dir}"
  local release_root
  release_root="$(find_extracted_root "${extract_dir}")"

  # Set up installation directories
  local versions_dir="${PREFIX}/versions"
  local version_dir="${versions_dir}/${VERSION}"
  local bin_dir="${PREFIX}/bin"
  local shim_dir="${PREFIX}/shims"

  # Install to versioned directory
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

  # Link binaries and configure shell
  link_binaries "${version_dir}" "${bin_dir}"
  ensure_shell_integration "${shim_dir}" "${bin_dir}"

  # Deploy shims and provision world backend
  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  harden_shim_symlinks "${shim_dir}"
  provision_macos_world "${version_dir}"

  # Run health checks with proper PATH (bin_dir before original PATH)
  local doctor_original_path="${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_original_path}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"

  # Source environment for current shell and print completion message
  if [[ "${DRY_RUN}" -eq 0 ]]; then
    if [ -f "${HOME}/.substrate_bashenv" ]; then
      # shellcheck disable=SC1090
      source "${HOME}/.substrate_bashenv" 2>/dev/null || true
    fi
    log "Sourced ~/.substrate_bashenv; environment ready."
    log "Installation complete. Start using Substrate in this shell or any new terminals."
  else
    log "Installation complete (dry run). Open a new terminal or 'source ~/.substrate_bashenv' when running for real."
  fi
}

# linux_artifact_name: Determine correct artifact name for Linux architecture
# Maps architecture to release artifact filename
# Returns: artifact filename for the current architecture
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

# install_linux: Complete installation workflow for Linux
# Flow:
#   1. Check prerequisites (systemctl, fuse-overlayfs, networking tools)
#   2. Download and verify architecture-specific release artifact
#   3. Extract to versioned directory (~/.substrate/versions/X.Y.Z/)
#   4. Link binaries to ~/.substrate/bin
#   5. Configure shell integration
#   6. Deploy shims
#   7. Install world-agent systemd service
#   8. Run health checks
install_linux() {
  ensure_linux_prereqs

  # Determine and download correct artifact for architecture
  local artifact
  artifact="$(linux_artifact_name)"
  local archive_path="${TMPDIR}/${artifact}"
  download_artifact "${artifact}" "${archive_path}"

  # Verify checksum if available
  local checksums_path="${TMPDIR}/SHA256SUMS"
  if download_checksums "${checksums_path}"; then
    verify_checksum "${archive_path}" "${checksums_path}" "${artifact}"
  fi

  # Extract archive
  local extract_dir="${TMPDIR}/extract"
  extract_archive "${archive_path}" "${extract_dir}"
  local release_root
  release_root="$(find_extracted_root "${extract_dir}")"

  # Set up installation directories
  local versions_dir="${PREFIX}/versions"
  local version_dir="${versions_dir}/${VERSION}"
  local bin_dir="${PREFIX}/bin"
  local shim_dir="${PREFIX}/shims"

  # Install to versioned directory
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

  # Link binaries and configure shell
  link_binaries "${version_dir}" "${bin_dir}"
  ensure_shell_integration "${shim_dir}" "${bin_dir}"

  # Deploy shims and provision world backend
  local substrate_bin="${bin_dir}/substrate"
  deploy_shims "${substrate_bin}"
  harden_shim_symlinks "${shim_dir}"
  provision_linux_world "${version_dir}"

  # Run health checks with proper PATH (bin_dir before original PATH)
  local doctor_original_path="${bin_dir}:${ORIGINAL_PATH}"
  log "Doctor PATH: ${doctor_original_path}"
  PATH="${doctor_original_path}" SHIM_ORIGINAL_PATH="${ORIGINAL_PATH}" SUBSTRATE_ROOT="${PREFIX}" run_world_checks "${substrate_bin}"

  # WSL-specific note
  if [[ "${IS_WSL}" -eq 1 ]]; then
    log "Detected WSL environment. Windows host components (forwarder, uninstall) must be managed via PowerShell scripts."
  fi

  log "Installation complete. Open a new terminal or 'source ~/.substrate_bashenv' to refresh PATH."
}

# ============================================================================
# MAIN ENTRY POINT
# ============================================================================

# main: Orchestrate the entire installation process
# Flow:
#   1. Sanitize PATH to avoid recursive shim calls
#   2. Parse command-line arguments
#   3. Detect platform (macOS/Linux/Windows)
#   4. Prepare temporary directory for downloads
#   5. Call platform-specific installation function
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

# Execute main function with all script arguments
main "$@"