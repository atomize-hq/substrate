# Manual Testing Playbook — best-effort-distro-package-manager (BEDPM)

This playbook validates the Linux hosted-installer contract defined by:
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/slices/BEDPM3/BEDPM3-spec.md`
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh`

Exit code taxonomy:
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Scope

- Linux hosted-installer package-manager detection, override precedence, decision-line output, and wrapper exit preservation for ADR-0031.
- macOS and Windows have no behavior delta under ADR-0031.

Validation authority:
- `tests/installers/pkg_manager_detection_smoke.sh` remains the authoritative assertion suite.
- `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh` is the non-interactive rerun entrypoint and must not define a second contract.

## Prerequisites

- Run the commands from a Linux shell in a local checkout of this repo.
- `bash` and `git` are available on `PATH`.
- The operator can write to a temp directory.

## Smoke script (required)

Run:

```bash
bash docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh
```

Expected:

- Exit `0`.

## Fixture setup

Run:

```bash
set -euo pipefail

export BEDPM_REPO_ROOT="$(git rev-parse --show-toplevel)"
export BEDPM_INSTALLER="$BEDPM_REPO_ROOT/scripts/substrate/install-substrate.sh"
export BEDPM_WRAPPER="$BEDPM_REPO_ROOT/scripts/substrate/install.sh"
export BEDPM_TMP="$(mktemp -d)"
export BEDPM_PREFIX="$BEDPM_TMP/prefix"
export BEDPM_ARTIFACTS="$BEDPM_TMP/artifacts"

mkdir -p "$BEDPM_PREFIX" "$BEDPM_ARTIFACTS"

case "$(uname -m)" in
  x86_64|amd64) export BEDPM_BUNDLE_LABEL="linux_x86_64" ;;
  aarch64|arm64) export BEDPM_BUNDLE_LABEL="linux_aarch64" ;;
  *) printf 'unsupported Linux arch: %s\n' "$(uname -m)" >&2; exit 1 ;;
esac

touch "$BEDPM_ARTIFACTS/substrate-v0.0.0-${BEDPM_BUNDLE_LABEL}.tar.gz"

bedpm_stub() {
  local name="$1"
  eval "${name}() { :; }"
  export -f "${name}"
}

bedpm_clear_manager_stubs() {
  unset -f apt-get dnf yum pacman zypper 2>/dev/null || true
}

bedpm_stub curl
bedpm_stub tar
bedpm_stub jq
bedpm_stub sudo
bedpm_stub sha256sum

unset PKG_MANAGER SUBSTRATE_INSTALL_OS_RELEASE_PATH
test -x "$BEDPM_INSTALLER"
test -x "$BEDPM_WRAPPER"
```

Expected:

- Exit `0`.
- `$BEDPM_INSTALLER` and `$BEDPM_WRAPPER` exist.

## Case 1 — Default Debian-family selection

Run:

```bash
bedpm_clear_manager_stubs
bedpm_stub apt-get

cat >"$BEDPM_TMP/os-release.debian" <<'EOF'
ID=debian
EOF

set +e
env SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.debian" \
  "$BEDPM_INSTALLER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  >"$BEDPM_TMP/debian.stdout" 2>"$BEDPM_TMP/debian.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "Detected distro: debian (like: <unknown>), using package manager: apt-get (source: os_release)" "$BEDPM_TMP/debian.stderr"
test "$status" -eq 0
```

Expected:

- Exit `0`.
- Stderr contains `Detected distro: debian (like: <unknown>), using package manager: apt-get (source: os_release)`.

## Case 2 — Default Arch-family selection

Run:

```bash
bedpm_clear_manager_stubs
bedpm_stub pacman

cat >"$BEDPM_TMP/os-release.arch" <<'EOF'
ID=arch
ID_LIKE=arch
EOF

set +e
env SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.arch" \
  "$BEDPM_INSTALLER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  >"$BEDPM_TMP/arch.stdout" 2>"$BEDPM_TMP/arch.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "Detected distro: arch (like: arch), using package manager: pacman (source: os_release)" "$BEDPM_TMP/arch.stderr"
test "$status" -eq 0
```

Expected:

- Exit `0`.
- Stderr contains `Detected distro: arch (like: arch), using package manager: pacman (source: os_release)`.

## Case 3 — Forced override via `--pkg-manager`

Run:

```bash
bedpm_clear_manager_stubs
bedpm_stub dnf

cat >"$BEDPM_TMP/os-release.override" <<'EOF'
ID=debian
EOF

set +e
env SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.override" \
  "$BEDPM_INSTALLER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  --pkg-manager dnf \
  >"$BEDPM_TMP/flag.stdout" 2>"$BEDPM_TMP/flag.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "Detected distro: debian (like: <unknown>), using package manager: dnf (source: flag)" "$BEDPM_TMP/flag.stderr"
test "$status" -eq 0
```

Expected:

- Exit `0`.
- Stderr contains `Detected distro: debian (like: <unknown>), using package manager: dnf (source: flag)`.

## Case 4 — Legacy override via `PKG_MANAGER`

Run:

```bash
bedpm_clear_manager_stubs
bedpm_stub yum

cat >"$BEDPM_TMP/os-release.legacy" <<'EOF'
ID=debian
EOF

set +e
env PKG_MANAGER="yum" \
  SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.legacy" \
  "$BEDPM_INSTALLER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  >"$BEDPM_TMP/env.stdout" 2>"$BEDPM_TMP/env.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "Detected distro: debian (like: <unknown>), using package manager: yum (source: env)" "$BEDPM_TMP/env.stderr"
test "$status" -eq 0
```

Expected:

- Exit `0`.
- Stderr contains `Detected distro: debian (like: <unknown>), using package manager: yum (source: env)`.

## Case 5 — Wrapper failure path with actionable remediation

Run:

```bash
bedpm_clear_manager_stubs

set +e
"$BEDPM_WRAPPER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  --pkg-manager apk \
  >"$BEDPM_TMP/failure.stdout" 2>"$BEDPM_TMP/failure.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F -- "--pkg-manager" "$BEDPM_TMP/failure.stderr"
grep -F "apk" "$BEDPM_TMP/failure.stderr"
grep -F "apt-get, dnf, yum, pacman, zypper" "$BEDPM_TMP/failure.stderr"
test "$status" -eq 2
```

Expected:

- Exit `2`.
- Stderr names the invalid value `apk`.
- Stderr names the invalid input source `--pkg-manager`.
- Stderr lists the allowed values `apt-get, dnf, yum, pacman, zypper`.
- Stderr includes rerun guidance that points the operator back to the allowed values or removal of the invalid override.

## Case 6 — Wrapper preserves exit `3` and direct-installer remediation

Run:

```bash
bedpm_clear_manager_stubs
bedpm_stub apt-get

cat >"$BEDPM_TMP/os-release.wrapper-missing" <<'EOF'
ID=debian
EOF

set +e
env SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.wrapper-missing" \
  "$BEDPM_WRAPPER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  --pkg-manager pacman \
  >"$BEDPM_TMP/wrapper-missing.stdout" 2>"$BEDPM_TMP/wrapper-missing.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "Detected distro: debian (like: <unknown>), using package manager: pacman (source: flag)" "$BEDPM_TMP/wrapper-missing.stderr"
grep -F "was not found in PATH" "$BEDPM_TMP/wrapper-missing.stderr"
grep -F "Install that manager or rerun with another allowed manager (apt-get, dnf, yum, pacman, zypper)." "$BEDPM_TMP/wrapper-missing.stderr"
test "$status" -eq 3
```

Expected:

- Exit `3`.
- Stderr contains the same decision line the direct installer emits for a flag-selected manager.
- Stderr preserves the missing-manager remediation text instead of collapsing to a wrapper-only failure class.

## Case 7 — Multi-manager `PATH` probe warning remains fixed-order

Run:

```bash
bedpm_clear_manager_stubs
bedpm_stub apt-get
bedpm_stub dnf
bedpm_stub yum
bedpm_stub zypper

cat >"$BEDPM_TMP/os-release.path-probe" <<'EOF'
ID=endeavouros
ID_LIKE=arch
EOF

set +e
env SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.path-probe" \
  "$BEDPM_INSTALLER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  >"$BEDPM_TMP/path-probe.stdout" 2>"$BEDPM_TMP/path-probe.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "Multiple supported package managers found in PATH: apt-get, dnf, yum, zypper; selecting apt-get by fixed probe order (apt-get -> dnf -> yum -> pacman -> zypper)." "$BEDPM_TMP/path-probe.stderr"
grep -F "Detected distro: endeavouros (like: arch), using package manager: apt-get (source: path_probe)" "$BEDPM_TMP/path-probe.stderr"
test "$status" -eq 0
```

Expected:

- Exit `0`.
- Stderr emits the fixed-order multi-manager warning before the decision line.
- The selected manager remains `apt-get` from `path_probe`, matching the authoritative harness.

## Case 8 — No supported manager remains exit `4`

Run:

```bash
bedpm_clear_manager_stubs

cat >"$BEDPM_TMP/os-release.no-manager" <<'EOF'
ID=arch
ID_LIKE=arch
EOF

set +e
env SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_TMP/os-release.no-manager" \
  "$BEDPM_INSTALLER" \
  --version 0.0.0 \
  --artifact-dir "$BEDPM_ARTIFACTS" \
  --dry-run \
  --no-world \
  --no-shims \
  >"$BEDPM_TMP/no-manager.stdout" 2>"$BEDPM_TMP/no-manager.stderr"
status=$?
set -e

printf 'exit=%s\n' "$status"
grep -F "No supported package manager was detected." "$BEDPM_TMP/no-manager.stderr"
grep -F "Missing prerequisite commands for this installer branch:" "$BEDPM_TMP/no-manager.stderr"
grep -F -- "--pkg-manager <apt-get|dnf|yum|pacman|zypper>" "$BEDPM_TMP/no-manager.stderr"
grep -F "PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>" "$BEDPM_TMP/no-manager.stderr"
test "$status" -eq 4
```

Expected:

- Exit `4`.
- Stderr names the no-manager posture and the missing prerequisites.
- Stderr carries the same rerun guidance for `--pkg-manager` and `PKG_MANAGER` as the authoritative harness.

## macOS-hosted Lima verification

Run this only on a macOS host with Lima configured for Substrate:

```bash
PATH="$(pwd)/target/debug:$PATH" scripts/mac/smoke.sh --bedpm-installer-conformance
```

Expected:

- Exit `0`.
- The command runs `docs/project_management/packs/implemented/best-effort-distro-package-manager/smoke/linux-smoke.sh` through the Lima-backed Linux guest path.
- The resulting output matches the authoritative Linux smoke behavior instead of implying native macOS package-manager-selection behavior.

## Cleanup

Run:

```bash
rm -rf "$BEDPM_TMP"
unset BEDPM_REPO_ROOT BEDPM_INSTALLER BEDPM_WRAPPER BEDPM_TMP BEDPM_PREFIX BEDPM_ARTIFACTS BEDPM_BUNDLE_LABEL
```

Expected:

- Exit `0`.
