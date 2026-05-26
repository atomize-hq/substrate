#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-bootstrap"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RUST_TOOLCHAIN="${RUST_TOOLCHAIN:-1.89.0}"
ENABLE_WIN_PREFLIGHT="${ENABLE_WIN_PREFLIGHT:-0}"

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

truthy() {
  case "${1,,}" in
    1|true|yes|y|on) return 0 ;;
    *) return 1 ;;
  esac
}

run_with_sudo() {
  if [[ ${EUID} -eq 0 ]]; then
    "$@"
    return
  fi
  if command -v sudo >/dev/null 2>&1; then
    sudo "$@"
    return
  fi
  fatal "sudo is required to install system packages: $*"
}

detect_linux_package_manager() {
  local manager=""
  for manager in apt-get dnf yum pacman zypper; do
    if command -v "${manager}" >/dev/null 2>&1; then
      printf '%s\n' "${manager}"
      return 0
    fi
  done
  return 1
}

install_linux_packages() {
  local manager="$1"
  shift
  case "${manager}" in
    apt-get)
      run_with_sudo apt-get update
      run_with_sudo apt-get install -y "$@"
      ;;
    dnf)
      run_with_sudo dnf install -y "$@"
      ;;
    yum)
      run_with_sudo yum install -y "$@"
      ;;
    pacman)
      run_with_sudo pacman -Sy --needed --noconfirm "$@"
      ;;
    zypper)
      run_with_sudo zypper install -y "$@"
      ;;
    *)
      fatal "unsupported Linux package manager: ${manager}"
      ;;
  esac
}

ensure_rustup_unix() {
  if command -v rustup >/dev/null 2>&1; then
    return
  fi
  if ! command -v curl >/dev/null 2>&1; then
    fatal "curl is required to install rustup; rerun after installing curl."
  fi
  log "Installing rustup (${RUST_TOOLCHAIN})"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain "${RUST_TOOLCHAIN}"
  export PATH="${HOME}/.cargo/bin:${PATH}"
  if ! command -v rustup >/dev/null 2>&1; then
    fatal "rustup installation completed, but rustup is not on PATH; open a new shell and rerun."
  fi
}

ensure_rust_toolchain() {
  log "Ensuring Rust toolchain ${RUST_TOOLCHAIN}"
  rustup toolchain install "${RUST_TOOLCHAIN}" --profile minimal
  rustup component add rustfmt clippy llvm-tools --toolchain "${RUST_TOOLCHAIN}"
}

ensure_cargo_xwin() {
  if command -v cargo-xwin >/dev/null 2>&1; then
    return
  fi
  log "Installing cargo-xwin"
  cargo install --locked cargo-xwin
}

print_linux_next_steps() {
  cat <<'EOF'

Next steps:
  1. Run `scripts/substrate/dev-install-substrate.sh --profile debug` to build and wire the local dev install.
  2. Run `make preflight` for the normal workspace checks.
  3. If ENABLE_WIN_PREFLIGHT=1 was used, run `make preflight-win` for Linux-host Windows parity.
EOF
}

print_macos_next_steps() {
  cat <<'EOF'

Next steps:
  1. Run `scripts/mac/lima-warm.sh` if you want the Lima-backed world enabled locally.
  2. Run `scripts/substrate/dev-install-substrate.sh --profile debug` to build and wire the local dev install.
  3. Run `make preflight` for the normal workspace checks.
EOF
}

bootstrap_linux() {
  local manager=""
  local base_packages=()
  local linux_specific_packages=()
  local win_preflight_packages=()

  manager="$(detect_linux_package_manager)" || fatal "no supported Linux package manager found (expected apt-get, dnf, yum, pacman, or zypper)."

  case "${manager}" in
    apt-get)
      base_packages=(curl jq pkg-config shellcheck shfmt)
      linux_specific_packages=(libseccomp-dev)
      win_preflight_packages=(clang wine64)
      ;;
    dnf|yum)
      base_packages=(curl jq pkgconf-pkg-config ShellCheck shfmt)
      linux_specific_packages=(libseccomp-devel)
      win_preflight_packages=(clang wine)
      ;;
    pacman)
      base_packages=(curl jq pkgconf shellcheck shfmt)
      linux_specific_packages=(libseccomp)
      win_preflight_packages=(clang wine)
      ;;
    zypper)
      base_packages=(curl jq pkg-config ShellCheck shfmt)
      linux_specific_packages=(libseccomp-devel)
      win_preflight_packages=(clang wine)
      ;;
  esac

  if truthy "${ENABLE_WIN_PREFLIGHT}"; then
    log "Installing Linux developer packages via ${manager} (including Windows preflight deps)"
    install_linux_packages "${manager}" "${base_packages[@]}" "${linux_specific_packages[@]}" "${win_preflight_packages[@]}"
  else
    log "Installing Linux developer packages via ${manager}"
    install_linux_packages "${manager}" "${base_packages[@]}" "${linux_specific_packages[@]}"
  fi

  ensure_rustup_unix
  ensure_rust_toolchain

  if truthy "${ENABLE_WIN_PREFLIGHT}"; then
    log "Enabling Linux-host Windows parity prerequisites"
    rustup target add x86_64-pc-windows-msvc --toolchain "${RUST_TOOLCHAIN}"
    ensure_cargo_xwin
  fi

  if [[ -x "${REPO_ROOT}/scripts/check-host-prereqs.sh" ]]; then
    log "Running Linux host prerequisite report"
    "${REPO_ROOT}/scripts/check-host-prereqs.sh" || true
  fi

  print_linux_next_steps
}

bootstrap_macos() {
  if truthy "${ENABLE_WIN_PREFLIGHT}"; then
    fatal "ENABLE_WIN_PREFLIGHT=1 is only supported on Linux hosts because make preflight-win is Linux-only."
  fi
  if ! command -v brew >/dev/null 2>&1; then
    fatal "Homebrew is required on macOS. Install brew first, then rerun make dev-bootstrap."
  fi

  log "Installing macOS developer packages via Homebrew"
  brew install curl jq lima pkg-config shellcheck shfmt

  ensure_rustup_unix
  ensure_rust_toolchain

  print_macos_next_steps
}

main() {
  case "$(uname -s)" in
    Linux)
      bootstrap_linux
      ;;
    Darwin)
      bootstrap_macos
      ;;
    *)
      fatal "unsupported host OS for scripts/dev/bootstrap.sh: $(uname -s)"
      ;;
  esac
}

main "$@"
