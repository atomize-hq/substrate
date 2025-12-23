#!/usr/bin/env bash
set -euo pipefail

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
SCRIPT_DIR=""
if [[ -n "${SCRIPT_SOURCE}" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
fi

ASSET_TMP_DIR=""
ASSET_ROOT="${SCRIPT_DIR}"
if [[ -z "${ASSET_ROOT}" || ! -f "${ASSET_ROOT}/install-substrate.sh" ]]; then
  # When running via curl|bash there is no local tree, so fetch the helpers on demand.
  if ! command -v curl >/dev/null 2>&1; then
    echo "[substrate-install] 'curl' is required to download installer assets." >&2
    exit 1
  fi
  ASSET_TMP_DIR="$(mktemp -d -t substrate-install-assets.XXXXXX)"
  INSTALL_REF="${SUBSTRATE_INSTALL_REF:-}"
  if [[ -z "${INSTALL_REF}" ]]; then
    # If the caller specified a version, pin helper scripts to that tag.
    VERSION_PIN=""
    for arg in "$@"; do
      case "${arg}" in
        --version=*)
          VERSION_PIN="${arg#--version=}"
          ;;
      esac
    done
    if [[ -z "${VERSION_PIN}" ]]; then
      for ((i=1; i<=$#; i++)); do
        if [[ "${!i}" == "--version" ]]; then
          j=$((i+1))
          VERSION_PIN="${!j:-}"
          break
        fi
      done
    fi

    if [[ -n "${VERSION_PIN}" ]]; then
      VERSION_PIN="${VERSION_PIN#v}"
      INSTALL_REF="v${VERSION_PIN}"
    else
      # No version requested: default to latest GitHub release tag (not main) to avoid drift.
      latest_url="$(curl -fsSL -o /dev/null -w '%{url_effective}' https://github.com/atomize-hq/substrate/releases/latest || true)"
      if [[ "${latest_url}" =~ /tag/([^/]+)$ ]]; then
        INSTALL_REF="${BASH_REMATCH[1]}"
      else
        INSTALL_REF="main"
        echo "[substrate-install][WARN] Unable to resolve latest release tag; falling back to '${INSTALL_REF}' for installer helpers." >&2
      fi
    fi
  fi

  ASSET_BASE="${SUBSTRATE_INSTALL_WRAPPER_BASE_URL:-https://raw.githubusercontent.com/atomize-hq/substrate/${INSTALL_REF}/scripts/substrate}"
  mkdir -p "${ASSET_TMP_DIR}/loader"
  curl -fsSL "${ASSET_BASE}/install-substrate.sh" -o "${ASSET_TMP_DIR}/install-substrate.sh"
  curl -fsSL "${ASSET_BASE}/loader/bash_loading_animations.sh" -o "${ASSET_TMP_DIR}/loader/bash_loading_animations.sh"
  chmod +x "${ASSET_TMP_DIR}/install-substrate.sh"
  ASSET_ROOT="${ASSET_TMP_DIR}"
fi

UPSTREAM_INSTALL="${ASSET_ROOT}/install-substrate.sh"
LOADER_DIR="${ASSET_ROOT}/loader"

if [[ ! -x "${UPSTREAM_INSTALL}" ]]; then
  echo "[substrate-install] missing installer at ${UPSTREAM_INSTALL}" >&2
  exit 1
fi
if [[ ! -f "${LOADER_DIR}/bash_loading_animations.sh" ]]; then
  echo "[substrate-install] missing loader at ${LOADER_DIR}/bash_loading_animations.sh" >&2
  exit 1
fi
source "${LOADER_DIR}/bash_loading_animations.sh"

detect_rc_file() {
  case "$(basename "${SHELL:-}")" in
    bash) echo "${HOME}/.bashrc" ;;
    zsh) echo "${HOME}/.zshrc" ;;
    fish) echo "${HOME}/.config/fish/config.fish" ;;
    *) echo "your shell's rc file" ;;
  esac
}
RC_TARGET="$(detect_rc_file)"

TMP_LOG="$(mktemp -t substrate-install-log.XXXXXX)"
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

printf "\033[32mSubstrate installer running…\033[0m\n"
if [[ -n "${BLA_braille_fill_bar[*]}" ]]; then
  BLA::start_loading_animation "${BLA_braille_fill_bar[@]}"
  LOADER_STARTED=1
fi
if "${UPSTREAM_INSTALL}" "$@" >"${TMP_LOG}" 2>&1; then
  stop_loader
  printf "\033[32mSubstrate install successful!\033[0m\n"
  printf "Open a new shell or run 'source %s' so PATH changes take effect.\n" "${RC_TARGET}"
else
  stop_loader
  echo "[substrate-install] Failed. See ${TMP_LOG} for details." >&2
  cat "${TMP_LOG}" >&2
  exit 1
fi
