# Manual Testing Playbook — best-effort-distro-package-manager (BEDPM)

This playbook is aligned to:
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM0/BEDPM0-spec.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM1/BEDPM1-spec.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/slices/BEDPM2/BEDPM2-spec.md`

Exit codes:
- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Linux installer pkg-manager decision exit codes: `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

Smoke scripts (mirror the cases below):
- Hermetic harness: `tests/installers/pkg_manager_detection_test.sh`
- Optional container sanity: `tests/installers/pkg_manager_container_smoke.sh`

## 0) Preconditions (Linux)

1. Verify the installer script is syntactically valid:

```bash
bash -n scripts/substrate/install-substrate.sh
```

Expected:
- Exit `0`.

2. Verify the hermetic harness passes:

```bash
bash tests/installers/pkg_manager_detection_test.sh
echo "exit=$?"
```

Expected:
- Exit `0`.

## 1) Fixture setup (PATH stubs + fake os-release)

The cases below run the installer in `--dry-run` mode with:
- a fake os-release file via `SUBSTRATE_INSTALL_OS_RELEASE_PATH`, and
- a controlled `PATH` that contains stub package-manager binaries.

Create a reusable temp directory:

```bash
export BEDPM_TMP="$(mktemp -d)"
export BEDPM_OS_RELEASE="$BEDPM_TMP/os-release"
export BEDPM_PATH_BIN="$BEDPM_TMP/path-bin"
export BEDPM_PM_BIN="$BEDPM_TMP/pm-bin"
mkdir -p "$BEDPM_PATH_BIN" "$BEDPM_PM_BIN"
```

Create a deterministic PATH sandbox (symlinks to common tools, no system package managers):

```bash
bedpm_link() {
  local cmd="$1"
  local p
  p="$(type -P "$cmd" || true)"
  if [[ -z "$p" ]]; then
    echo "missing required command: $cmd" >&2
    return 1
  fi
  ln -sf "$p" "$BEDPM_PATH_BIN/$cmd"
}

bedpm_link bash
bedpm_link cat
bedpm_link chmod
bedpm_link cp
bedpm_link curl
bedpm_link cut
bedpm_link date
bedpm_link dirname
bedpm_link getent
bedpm_link grep
bedpm_link head
bedpm_link id
bedpm_link jq
bedpm_link mkdir
bedpm_link mktemp
bedpm_link mv
bedpm_link rm
bedpm_link sed
bedpm_link sudo
bedpm_link tar
bedpm_link tr
bedpm_link uname

if type -P sha256sum >/dev/null 2>&1; then
  bedpm_link sha256sum
else
  bedpm_link shasum
fi
```

Helper to create a stub binary:

```bash
bedpm_stub() {
  local name="$1"
  cat >"$BEDPM_PM_BIN/$name" <<'EOF'
#!/usr/bin/env bash
exit 0
EOF
  chmod +x "$BEDPM_PM_BIN/$name"
}
```

Helper to run the installer and capture stderr:

```bash
bedpm_run() {
  local stderr_path="$1"
  shift
  (
    export SUBSTRATE_INSTALL_OS_RELEASE_PATH="$BEDPM_OS_RELEASE"
    export PATH="$BEDPM_PM_BIN:$BEDPM_PATH_BIN"
    "$BEDPM_PATH_BIN/bash" scripts/substrate/install-substrate.sh --dry-run --no-world --no-shims --version "v0.2.2" "$@"
  ) 1>/dev/null 2>"$stderr_path"
  echo "exit=$?"
}
```

Notes:
- `PATH` contains only the sandbox (`$BEDPM_PATH_BIN`) plus pkg-manager stubs (`$BEDPM_PM_BIN`).
- `--version "v0.2.2"` avoids network calls during manual runs.

## 2) BEDPM0/BEDPM1 — selection + decision one-liner cases

### Case 2.1 — Debian/Ubuntu family mapping selects `apt-get` (source=`os_release`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF
rm -f "$BEDPM_PM_BIN/"*
bedpm_stub apt-get
```

Run:

```bash
bedpm_run "$BEDPM_TMP/stderr.txt"
cat "$BEDPM_TMP/stderr.txt" | grep -Fxc "Detected distro: ubuntu (like: debian), using package manager: apt-get (source: os_release)"
```

Expected:
- First command prints `exit=0`.
- `grep` exits `0` and prints `1` (exactly one decision one-liner).

### Case 2.2 — Arch family mapping selects `pacman` (source=`os_release`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=arch
ID_LIKE=arch
EOF
rm -f "$BEDPM_PM_BIN/"*
bedpm_stub pacman
```

Run:

```bash
bedpm_run "$BEDPM_TMP/stderr.txt"
cat "$BEDPM_TMP/stderr.txt" | grep -Fxc "Detected distro: arch (like: arch), using package manager: pacman (source: os_release)"
```

Expected:
- First command prints `exit=0`.
- `grep` exits `0` and prints `1`.

### Case 2.3 — CLI override `--pkg-manager` forces manager (source=`flag`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF
rm -f "$BEDPM_PM_BIN/"*
bedpm_stub yum
```

Run:

```bash
bedpm_run "$BEDPM_TMP/stderr.txt" --pkg-manager yum
cat "$BEDPM_TMP/stderr.txt" | grep -Fxc "Detected distro: ubuntu (like: debian), using package manager: yum (source: flag)"
```

Expected:
- First command prints `exit=0`.
- `grep` exits `0` and prints `1`.

### Case 2.4 — Env override `PKG_MANAGER` forces manager (source=`env`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF
rm -f "$BEDPM_PM_BIN/"*
bedpm_stub dnf
```

Run:

```bash
(
  export PKG_MANAGER="dnf"
  bedpm_run "$BEDPM_TMP/stderr.txt"
)
cat "$BEDPM_TMP/stderr.txt" | grep -Fxc "Detected distro: ubuntu (like: debian), using package manager: dnf (source: env)"
```

Expected:
- First command prints `exit=0`.
- `grep` exits `0` and prints `1`.

### Case 2.5 — Invalid override value fails closed (exit `2`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF
rm -f "$BEDPM_PM_BIN/"*
```

Run:

```bash
bedpm_run "$BEDPM_TMP/stderr.txt" --pkg-manager "not-a-manager"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "not-a-manager"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "--pkg-manager"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "PKG_MANAGER"
```

Expected:
- First command prints `exit=2`.
- Each `grep` exits `0` (required content elements are present in stderr).

### Case 2.6 — Forced manager missing from PATH fails closed (exit `3`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=ubuntu
ID_LIKE=debian
EOF
rm -f "$BEDPM_PM_BIN/"*
```

Run:

```bash
bedpm_run "$BEDPM_TMP/stderr.txt" --pkg-manager "zypper"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "zypper"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "--pkg-manager"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "PKG_MANAGER"
```

Expected:
- First command prints `exit=3`.
- Each `grep` exits `0`.

### Case 2.7 — No supported manager selectable fails (exit `4`)

Fixture:

```bash
cat >"$BEDPM_OS_RELEASE" <<'EOF'
ID=
ID_LIKE=
EOF
rm -f "$BEDPM_PM_BIN/"*
```

Run:

```bash
bedpm_run "$BEDPM_TMP/stderr.txt"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "--pkg-manager"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "PKG_MANAGER"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "curl"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "tar"
cat "$BEDPM_TMP/stderr.txt" | grep -Fq "jq"
```

Expected:
- First command prints `exit=4`.
- Each `grep` exits `0` (remediation guidance includes override hints and the manual prereq command list elements).
