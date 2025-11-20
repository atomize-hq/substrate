#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UPSTREAM_INSTALL="${SCRIPT_DIR}/install-substrate.sh"
LOADER_DIR="${SCRIPT_DIR}/loader"

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
    bash) echo "~/.bashrc" ;;
    zsh) echo "~/.zshrc" ;;
    fish) echo "~/.config/fish/config.fish" ;;
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
