#!/usr/bin/env bash
set -euo pipefail

# Example script install that fetches nvm into $HOME.
# The runnable `nvm` entrypoint is provided by `wrappers[]`, not by this script.

export NVM_DIR="${NVM_DIR:-$HOME/.nvm}"
if [ ! -d "${NVM_DIR}" ]; then
  curl -fsSL https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
else
  # Best-effort update.
  if [ -d "${NVM_DIR}/.git" ]; then
    (cd "${NVM_DIR}" && git pull --ff-only) || true
  fi
fi
