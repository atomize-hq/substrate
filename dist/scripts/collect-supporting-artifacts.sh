#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT_DIR}/dist/supporting-artifacts"
STAGING_DIR="${OUT_DIR}/staging"

rm -rf "${OUT_DIR}"
mkdir -p "${STAGING_DIR}"

# Documentation bundled with every release.
docs=(
  "docs/INSTALLATION.md"
  "docs/CONFIGURATION.md"
  "docs/WORLD.md"
)

mkdir -p "${STAGING_DIR}/docs"
for doc in "${docs[@]}"; do
  cp "${ROOT_DIR}/${doc}" "${STAGING_DIR}/docs/"
done

# Runtime configuration (manager manifest, etc.) bundled with releases.
if [[ -d "${ROOT_DIR}/config" ]]; then
  mkdir -p "${STAGING_DIR}/config"
  cp -R "${ROOT_DIR}/config/." "${STAGING_DIR}/config/"
fi

# Platform helper scripts required by the rewritten installers.
script_dirs=(
  "scripts/linux"
  "scripts/mac"
  "scripts/windows"
  "scripts/wsl"
  "scripts/substrate"
)

for dir in "${script_dirs[@]}"; do
  src="${ROOT_DIR}/${dir}"
  dest="${STAGING_DIR}/${dir}"
  if [[ -d "${src}" ]]; then
    mkdir -p "${dest}"
    cp -R "${src}/." "${dest}/"
  fi
done

mkdir -p "${OUT_DIR}"

# Tarball variant (POSIX shells, Linux/macOS installers).
tar -czf "${OUT_DIR}/substrate-support.tar.gz" -C "${STAGING_DIR}" .

# Zip variant (PowerShell convenience on Windows).
SUPPORT_OUT_DIR="${OUT_DIR}" SUPPORT_STAGING_DIR="${STAGING_DIR}" python3 - <<'PY'
import os, pathlib, zipfile
root = pathlib.Path(os.environ['SUPPORT_OUT_DIR'])
staging = pathlib.Path(os.environ['SUPPORT_STAGING_DIR'])
zip_path = root / 'substrate-support.zip'
with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zf:
    for path in staging.rglob('*'):
        if path.is_file():
            zf.write(path, path.relative_to(staging))
PY

rm -rf "${STAGING_DIR}"
echo "supporting release artifacts staged in ${OUT_DIR}" >&2
