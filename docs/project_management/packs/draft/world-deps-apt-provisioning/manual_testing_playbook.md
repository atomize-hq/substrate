# Manual Testing Playbook — world-deps APT provisioning (WDAP)

This playbook is aligned to:
- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`

Exit codes:
- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

Smoke scripts (mirror the automatable subset of the cases below):
- Linux: `smoke/linux-smoke.sh`
- macOS: `smoke/macos-smoke.sh`
- Windows: `smoke/windows-smoke.ps1`

## 0) Preconditions (all platforms)

1. Verify `substrate` is runnable:

```bash
substrate --version
```

Expected:
- Exit `0`.

2. Runtime tests require a healthy world backend:

```bash
substrate world doctor
```

Expected:
- Exit `0`.

## 1) Fixture setup (enable at least one APT-backed dep)

The cases below use a deterministic local inventory under `SUBSTRATE_HOME`:
- `smoke-hello` (script install; non-APT)
- `smoke-apt-a` (APT; unpinned)
- `smoke-apt-b` (APT; pinned `version: "1"`)

### Linux/macOS fixture setup (bash)

```bash
export WDAP_TMP="$(mktemp -d)"
export SUBSTRATE_HOME="$WDAP_TMP/substrate-home"
export WDAP_WS="$WDAP_TMP/ws"

mkdir -p "$SUBSTRATE_HOME/deps/packages" "$WDAP_WS"
substrate workspace init "$WDAP_WS" >/dev/null

cat >"$SUBSTRATE_HOME/deps/packages/smoke-hello.yaml" <<'YAML'
version: 1
name: smoke-hello
description: WDAP smoke fixture (script install).
runnable: true
entrypoints: ["smoke-hello"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p /var/lib/substrate/world-deps/bin
    cat > /var/lib/substrate/world-deps/bin/smoke-hello <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/smoke-hello
probe:
  command: "smoke-hello"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-a.yaml" <<'YAML'
version: 1
name: smoke-apt-a
description: WDAP smoke fixture (APT; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-a
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-b.yaml" <<'YAML'
version: 1
name: smoke-apt-b
description: WDAP smoke fixture (APT; pinned; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-b
      version: "1"
probe:
  command: "sh -c 'exit 1'"
YAML

pushd "$WDAP_WS" >/dev/null
substrate world deps global reset >/dev/null 2>&1 || true
substrate world deps workspace reset >/dev/null 2>&1 || true
substrate world deps workspace add smoke-hello smoke-apt-a smoke-apt-b >/dev/null
```

Expected:
- Exit `0`.
- `substrate world deps current list enabled` includes `smoke-apt-a` and `smoke-apt-b`.

### Windows fixture setup (PowerShell)

```powershell
$WdapTmp = Join-Path $env:TEMP ("wdap-" + [guid]::NewGuid().ToString("N"))
$SubstrateHome = Join-Path $WdapTmp "substrate-home"
$WdapWs = Join-Path $WdapTmp "ws"

New-Item -ItemType Directory -Force -Path (Join-Path $SubstrateHome "deps\\packages"), $WdapWs | Out-Null
$env:SUBSTRATE_HOME = $SubstrateHome

& substrate workspace init $WdapWs | Out-Null

@"
version: 1
name: smoke-hello
description: WDAP smoke fixture (script install).
runnable: true
entrypoints: ["smoke-hello"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p /var/lib/substrate/world-deps/bin
    cat > /var/lib/substrate/world-deps/bin/smoke-hello <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/smoke-hello
probe:
  command: "smoke-hello"
"@ | Set-Content -Path (Join-Path $SubstrateHome "deps\\packages\\smoke-hello.yaml") -NoNewline

@"
version: 1
name: smoke-apt-a
description: WDAP smoke fixture (APT; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-a
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $SubstrateHome "deps\\packages\\smoke-apt-a.yaml") -NoNewline

@"
version: 1
name: smoke-apt-b
description: WDAP smoke fixture (APT; pinned; missing by design).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-b
      version: "1"
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $SubstrateHome "deps\\packages\\smoke-apt-b.yaml") -NoNewline

Push-Location $WdapWs
& substrate world deps global reset | Out-Null
& substrate world deps workspace reset | Out-Null
& substrate world deps workspace add smoke-hello smoke-apt-a smoke-apt-b | Out-Null
```

Expected:
- Exit `0`.
- `substrate world deps current list enabled` includes `smoke-apt-a` and `smoke-apt-b`.

## 2) WDAP0 — Provisioning surface (`world enable --provision-deps`)

### Case 2.1 — Unsupported backends fail closed (Linux host-native, Windows)

Run:

```bash
substrate world enable --provision-deps --dry-run
echo "exit=$?"
```

Expected:
- Linux host-native:
  - Exit `4`.
  - Stderr contains `Substrate will not mutate the host OS`.
  - Stderr contains `substrate world enable --provision-deps`.
- Windows:
  - Exit `4`.
  - Stderr contains `unsupported on Windows`.
  - Stderr contains `substrate world enable --provision-deps`.

### Case 2.2 — `--dry-run` prints a normalized APT requirement set (macOS Lima)

Run (inside the workspace from Fixture setup):

```bash
substrate world enable --provision-deps --dry-run >"$WDAP_TMP/stdout.txt" 2>"$WDAP_TMP/stderr.txt"
echo "exit=$?"
cat "$WDAP_TMP/stdout.txt"
```

Expected:
- Exit `0`.
- Stdout is exactly:

  ```text
  smoke-apt-a
  smoke-apt-b=1
  ```

### Case 2.3 — Provisioning ignores `SUBSTRATE_WORLD_REQUEST_PROFILE` (macOS Lima)

Run:

```bash
SUBSTRATE_WORLD_REQUEST_PROFILE="wdap-smoke-profile" \
  substrate world enable --provision-deps --dry-run --verbose >"$WDAP_TMP/stdout-verbose.txt" 2>"$WDAP_TMP/stderr-verbose.txt"
echo "exit=$?"
```

Expected:
- Exit `0`.
- Stdout contains `world-deps-provision`.
- Stdout does not contain `wdap-smoke-profile`.

### Case 2.4 — Supported guest non-dry-run provisioning succeeds with a real APT package (macOS Lima)

This case intentionally performs guest OS mutation. Use a real package name that is absent in the
guest before the run. The default below uses `sl`; if your Lima guest already has `sl` installed,
set `WDAP_REAL_APT_PACKAGE` to another small apt package that is currently absent and rerun the
fixture creation step in this case.

When validating the current checkout rather than an already-installed host binary:
- run this case from the repository root,
- point `SUBSTRATE_BIN` at the checkout build you want to validate, and
- verify package presence from the Lima guest with `limactl shell substrate ... dpkg-query ...`.

A normal workspace-isolated `substrate --world` shell is not authoritative for this case because
the general workspace posture masks `/var/lib`, which hides `/var/lib/dpkg` from guest package
queries outside the dedicated world-deps flows.

Setup:

```bash
export WDAP_REAL_APT_PACKAGE="${WDAP_REAL_APT_PACKAGE:-sl}"
export WDAP_REAL_ROOT="${WDAP_REAL_ROOT:-$HOME/.substrate-wdap-real}"
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-$PWD/target/debug/substrate}"
export SUBSTRATE_WORLD_ENABLE_SCRIPT="${SUBSTRATE_WORLD_ENABLE_SCRIPT:-$PWD/scripts/substrate/world-enable.sh}"
rm -rf "$WDAP_REAL_ROOT"
export SUBSTRATE_HOME="$WDAP_REAL_ROOT/substrate-home"
export WDAP_WS="$WDAP_REAL_ROOT/ws"
mkdir -p "$SUBSTRATE_HOME/deps/packages" "$SUBSTRATE_HOME/bin" "$WDAP_WS"
ln -sf "$SUBSTRATE_BIN" "$SUBSTRATE_HOME/bin/substrate"
"$SUBSTRATE_BIN" workspace init "$WDAP_WS" >/dev/null

cat >"$SUBSTRATE_HOME/deps/packages/smoke-hello.yaml" <<'YAML'
version: 1
name: smoke-hello
description: WDAP smoke fixture (script install).
runnable: true
entrypoints: ["smoke-hello"]
install:
  method: script
  script: |
    set -euo pipefail
    mkdir -p /var/lib/substrate/world-deps/bin
    cat > /var/lib/substrate/world-deps/bin/smoke-hello <<'EOF'
    #!/bin/sh
    echo smoke-hello
    EOF
    chmod +x /var/lib/substrate/world-deps/bin/smoke-hello
probe:
  command: "smoke-hello"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-real.yaml" <<YAML
version: 1
name: smoke-apt-real
description: WDAP smoke fixture (real APT install).
runnable: false
install:
  method: apt
  apt:
    - name: ${WDAP_REAL_APT_PACKAGE}
probe:
  command: "sh -c 'exit 1'"
YAML

"$SUBSTRATE_BIN" world deps workspace reset >/dev/null 2>&1 || true
"$SUBSTRATE_BIN" world deps workspace add smoke-hello smoke-apt-real >/dev/null
limactl shell substrate "dpkg-query -W -f='\${Status}\\n' ${WDAP_REAL_APT_PACKAGE} 2>/dev/null || true" \
  >"$WDAP_TMP/real-apt-before.txt"
cat "$WDAP_TMP/real-apt-before.txt"
```

Expected before provisioning:
- Output does not contain `install ok installed`.

Run:

```bash
"$SUBSTRATE_BIN" world enable --home "$SUBSTRATE_HOME" --profile debug --provision-deps --verbose \
  >"$WDAP_TMP/stdout-real.txt" 2>"$WDAP_TMP/stderr-real.txt"
echo "exit=$?"
limactl shell substrate "dpkg-query -W -f='\${Status}\\n' ${WDAP_REAL_APT_PACKAGE}" \
  >"$WDAP_TMP/real-apt-after.txt"
cat "$WDAP_TMP/real-apt-after.txt"
```

Expected:
- Exit `0`.
- Stdout contains `world-deps-provision`.
- Stdout contains `${WDAP_REAL_APT_PACKAGE}`.
- Post-run `dpkg-query` output contains `install ok installed`.

### Case 2.5 — Version-pin conflicts exit `2` (macOS Lima)

Setup (inside the same workspace):

```bash
cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-conflict-1.yaml" <<'YAML'
version: 1
name: smoke-apt-conflict-1
description: WDAP smoke fixture (APT conflict 1).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-conflict
      version: "1"
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-conflict-2.yaml" <<'YAML'
version: 1
name: smoke-apt-conflict-2
description: WDAP smoke fixture (APT conflict 2).
runnable: false
install:
  method: apt
  apt:
    - name: smoke-apt-conflict
      version: "2"
probe:
  command: "sh -c 'exit 1'"
YAML

substrate world deps workspace add smoke-apt-conflict-1 smoke-apt-conflict-2 >/dev/null
```

Run:

```bash
substrate world enable --provision-deps --dry-run 1>"$WDAP_TMP/stdout-conflict.txt" 2>"$WDAP_TMP/stderr-conflict.txt"
echo "exit=$?"
cat "$WDAP_TMP/stderr-conflict.txt"
```

Expected:
- Exit `2`.
- Stderr contains:
  - `smoke-apt-conflict`
  - `1`
  - `2`

## 3) WDAP1 — Runtime fail-early (`world deps current sync|install`)

Preflight (required):

```bash
substrate world doctor
echo "exit=$?"
```

Expected:
- Exit `0`.

### Case 3.1 — `current sync --dry-run` exits `4` and emits remediation

Run (inside the workspace):

```bash
substrate world deps current sync --dry-run 1>"$WDAP_TMP/stdout-sync.txt" 2>"$WDAP_TMP/stderr-sync.txt"
echo "exit=$?"
```

Expected:
- Exit `4`.
- Stdout contains, in order:
  - `smoke-apt-a`
  - `smoke-apt-b=1`
- Stderr contains `substrate world enable --provision-deps`.
- Linux host-native: stderr contains `Substrate will not mutate the host OS`.
- Windows: stderr contains `unsupported on Windows`.

### Case 3.2 — `current sync --dry-run --verbose` includes the requirement set on stderr

Run:

```bash
substrate world deps current sync --dry-run --verbose 1>"$WDAP_TMP/stdout-sync-verbose.txt" 2>"$WDAP_TMP/stderr-sync-verbose.txt"
echo "exit=$?"
cat "$WDAP_TMP/stderr-sync-verbose.txt"
```

Expected:
- Exit `4`.
- Stderr contains:
  - `substrate world enable --provision-deps`
  - `smoke-apt-a`
  - `smoke-apt-b=1`

### Case 3.3 — `current install <ITEM...>` does not add enabled items implicitly

Run:

```bash
substrate world deps current install smoke-hello
echo "exit=$?"
```

Expected:
- Exit `0`.

### Case 3.4 — `current install <ITEM...> --dry-run` fails early for explicit APT-backed items

Run:

```bash
substrate world deps current install smoke-apt-a --dry-run 1>"$WDAP_TMP/stdout-install.txt" 2>"$WDAP_TMP/stderr-install.txt"
echo "exit=$?"
cat "$WDAP_TMP/stdout-install.txt"
```

Expected:
- Exit `4`.
- Stdout contains `smoke-apt-a`.
- Stderr contains `substrate world enable --provision-deps`.
