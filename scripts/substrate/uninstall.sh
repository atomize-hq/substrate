#!/usr/bin/env bash
set -euo pipefail

SCRIPT_SOURCE="${BASH_SOURCE[0]:-}"
SCRIPT_DIR=""
if [[ -n "${SCRIPT_SOURCE}" ]]; then
  SCRIPT_DIR="$(cd "$(dirname "${SCRIPT_SOURCE}")" && pwd)"
fi

DEFAULT_PREFIX="${HOME}/.substrate"
SUBSTRATE_ROOT="${SUBSTRATE_ROOT:-${DEFAULT_PREFIX}}"

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

ASSET_TMP_DIR=""
ASSET_ROOT="${SCRIPT_DIR}"
UPSTREAM_UNINSTALL="${ASSET_ROOT}/uninstall-substrate.sh"
LOADER_DIR="${ASSET_ROOT}/loader"

if [[ -z "${ASSET_ROOT}" || ! -x "${UPSTREAM_UNINSTALL}" ]]; then
  # Prefer running the uninstaller shipped with the installed version (keeps behavior/version aligned).
  if local_uninstall="$(resolve_local_uninstaller "${SUBSTRATE_ROOT}" 2>/dev/null)"; then
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

printf "\033[32mSubstrate uninstall running…\033[0m\n"
if [[ -f "${LOADER_DIR}/bash_loading_animations.sh" ]]; then
  # shellcheck disable=SC1090
  source "${LOADER_DIR}/bash_loading_animations.sh"
  if [[ -n "${BLA_braille_fill_bar[*]:-}" ]]; then
    BLA::start_loading_animation "${BLA_braille_fill_bar[@]}"
    LOADER_STARTED=1
  fi
fi

if "${UPSTREAM_UNINSTALL}" "$@" >"${TMP_LOG}" 2>&1; then
  stop_loader
  printf "\033[32mSubstrate uninstall complete!\033[0m\n"
else
  stop_loader
  printf "\033[31mSubstrate uninstall failed.\033[0m See %s for details.\n" "${TMP_LOG}" >&2
  cat "${TMP_LOG}" >&2
  exit 1
fi
