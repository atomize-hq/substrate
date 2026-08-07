#!/usr/bin/env bash
set -euo pipefail

SCRIPT_NAME="dev-install-compile-surface-r3"
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
INSTALLER="${REPO_ROOT}/scripts/substrate/dev-install-substrate.sh"

if [[ "$(uname -s)" != "Darwin" ]]; then
  printf '[%s] SKIP: macOS compile surface\n' "${SCRIPT_NAME}"
  exit 0
fi

WORK_ROOT="$(mktemp -d "${TMPDIR:-/tmp}/substrate-dev-install-compile.XXXXXX")"
chmod 0700 "${WORK_ROOT}"
cleanup() {
  rm -rf -- "${WORK_ROOT}"
}
trap cleanup EXIT

cd "${REPO_ROOT}"
# R4 deferred gate: substrate-lifecycle-macos currently has sha2, AuditTokenV1 visibility,
# and pointer-cast compile failures outside the R2 allowlist. R2 proves only the shell/ordinary
# host cfg surface and the installer source shape; R4 must compile this binary after its owner
# authorizes the lifecycle source/API/manifest closure.
CARGO_TARGET_DIR="${WORK_ROOT}/target-native" cargo check \
  -p substrate --bin substrate --bin substrate-shim \
  -p substrate-gateway --bin substrate-gateway
CARGO_TARGET_DIR="${WORK_ROOT}/target-x86_64-apple-darwin" cargo check \
  --target x86_64-apple-darwin \
  -p substrate --bin substrate --bin substrate-shim \
  -p substrate-gateway --bin substrate-gateway

python3 - "${INSTALLER}" <<'PY'
from pathlib import Path
import sys

source = Path(sys.argv[1]).read_text(encoding="utf-8")
start = source.index("BUILD_FLAGS=(build ")
end = source.index('\nif [[ "${IS_MAC}" -eq 1 ]]', start)
build_flags = source[start:end]
for token in (
    "-p substrate",
    "--bin substrate",
    "--bin substrate-shim",
    "-p substrate-gateway",
    "--bin substrate-gateway",
):
    if token not in build_flags:
        raise SystemExit(f"canonical installer no longer builds {token}")

expected_mac_build_flags = '''if [[ "${IS_MAC}" -eq 1 ]]; then
  BUILD_FLAGS+=(--bin substrate-lifecycle-control --bin substrate-lifecycle-macos)
fi'''
if expected_mac_build_flags not in source:
    raise SystemExit("canonical macOS lifecycle BUILD_FLAGS are not the exact R2 pair")

helper_start = source.index("stage_managed_mac_control_binary_copy() {")
helper_end = source.index("\n}\n", helper_start) + 3
helper = source[helper_start:helper_end]
for required in (
    '[[ -f "${src}" && -x "${src}" ]]',
    'path_is_managed_bundle_entry "${dest}" "${repo_root}" "${manifest_path}"',
    'fatal "Refusing to overwrite unmanaged ${label} at ${dest}"',
    'local binary_tmp',
    'local manifest_tmp',
    'binary_tmp="${dest}.tmp.$$"',
    'manifest_tmp="${manifest_path}.tmp.$$"',
    'cp "${src}" "${binary_tmp}"',
    'chmod 0755 "${binary_tmp}"',
    ': > "${manifest_tmp}"',
    'printf \'%s\\n\' "${dest}" >> "${manifest_tmp}"',
    'mv "${manifest_tmp}" "${manifest_path}"',
    'mv "${binary_tmp}" "${dest}"',
):
    if required not in helper:
        raise SystemExit(f"managed macOS fixed-copy helper lost required R2 invariant: {required}")

if helper.count('rm -f "${binary_tmp}" "${manifest_tmp}"') < 5:
    raise SystemExit("managed macOS fixed-copy helper no longer cleans both temporary paths on staging failures")

if helper.index('mv "${manifest_tmp}" "${manifest_path}"') > helper.index('mv "${binary_tmp}" "${dest}"'):
    raise SystemExit("managed macOS fixed-copy helper must publish the manifest before replacing the destination")

expected_copy_loop = '''if [[ "${IS_MAC}" -eq 1 ]]; then
  mkdir -p "${MANAGED_STATE_DIR}"
  for binary in substrate-lifecycle-control substrate-lifecycle-macos; do
    src="${REPO_ROOT}/target/${TARGET_DIR}/${binary}"
    stage_managed_mac_control_binary_copy \\
      "${src}" "${BIN_DIR}/${binary}" "${REPO_ROOT}" \\
      "${MANAGED_MAC_CONTROL_BINARIES_PATH}" "macOS ${binary}"
  done
fi'''
if expected_copy_loop not in source:
    raise SystemExit("canonical macOS fixed managed-copy branch is not the exact R2 pair")
PY
