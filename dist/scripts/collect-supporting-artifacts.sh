#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/dist/supporting-artifacts"

# Recreate the output directory on every run so stale files never leak between releases.
rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}/docs" "${OUT_DIR}/installers"

# Docs that ship with each release to point users at deep configuration guidance.
install_docs=(
  "docs/INSTALLATION.md"
  "docs/CONFIGURATION.md"
  "docs/WORLD.md"
)

for doc in "${install_docs[@]}"; do
  cp "${ROOT_DIR}/${doc}" "${OUT_DIR}/docs/"
done

# Curated installer helpers for each platform family.
cp "${ROOT_DIR}/scripts/substrate/install-substrate.sh" \
   "${OUT_DIR}/installers/install-substrate.sh"

cp "${ROOT_DIR}/scripts/mac/lima-warm.sh" \
   "${OUT_DIR}/installers/lima-warm.sh"

cp "${ROOT_DIR}/scripts/windows/wsl-warm.ps1" \
   "${OUT_DIR}/installers/wsl-warm.ps1"

echo "supporting release artifacts staged in ${OUT_DIR}"
