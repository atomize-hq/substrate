# Manual Testing Playbook — add-non-apt-system-package-provisioning-support (NASP)

This playbook is aligned to:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP3/NASP3-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP4/NASP4-spec.md`

Exit codes:
- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

Smoke scripts:
- Linux: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`

## 0) Preconditions

1. Verify `substrate` is runnable:

```bash
substrate --version
```

Expected:
- Exit `0`.

2. Run the platform smoke script first:

Linux:

```bash
bash docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh
```

macOS:

```bash
bash docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh
```

Windows:

```powershell
pwsh -File docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1
```

Expected:
- Exit `0`.

3. Runtime cases require a healthy world backend on macOS and Windows:

```bash
substrate world doctor
```

Expected:
- macOS: exit `0`
- Windows: exit `0`

## 1) Fixture setup

The cases below use a deterministic local inventory under `SUBSTRATE_HOME`:
- `smoke-hello` — script-backed package for explicit-item scope checks
- `smoke-pacman-a` — pacman-backed prerequisite with two package names in reverse order
- `smoke-pacman-b` — pacman-backed prerequisite that repeats one package name for dedup validation
- `smoke-apt-a` — APT-backed prerequisite for mixed-manager validation on macOS

### Linux and macOS fixture setup (bash)

```bash
export NASP_TMP="$(mktemp -d)"
export SUBSTRATE_HOME="$NASP_TMP/substrate-home"
export NASP_WS="$NASP_TMP/ws"

mkdir -p "$SUBSTRATE_HOME/deps/packages" "$NASP_WS"
substrate workspace init "$NASP_WS" >/dev/null

cat >"$SUBSTRATE_HOME/deps/packages/smoke-hello.yaml" <<'YAML'
version: 1
name: smoke-hello
description: NASP smoke fixture (script install).
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

cat >"$SUBSTRATE_HOME/deps/packages/smoke-pacman-a.yaml" <<'YAML'
version: 1
name: smoke-pacman-a
description: NASP smoke fixture (pacman list in authored reverse order).
runnable: false
install:
  method: pacman
  pacman:
    - nasp-pacman-b
    - nasp-pacman-a
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-pacman-b.yaml" <<'YAML'
version: 1
name: smoke-pacman-b
description: NASP smoke fixture (pacman duplicate for normalization).
runnable: false
install:
  method: pacman
  pacman:
    - nasp-pacman-a
probe:
  command: "sh -c 'exit 1'"
YAML

cat >"$SUBSTRATE_HOME/deps/packages/smoke-apt-a.yaml" <<'YAML'
version: 1
name: smoke-apt-a
description: NASP smoke fixture (APT item for mixed-manager validation).
runnable: false
install:
  method: apt
  apt:
    - name: nasp-apt-a
probe:
  command: "sh -c 'exit 1'"
YAML

pushd "$NASP_WS" >/dev/null
substrate world deps global reset >/dev/null 2>&1 || true
substrate world deps workspace reset >/dev/null 2>&1 || true
substrate world deps workspace add smoke-hello smoke-pacman-a smoke-pacman-b >/dev/null
```

Expected:
- Exit `0`.
- `substrate world deps current list enabled` includes `smoke-pacman-a` and `smoke-pacman-b`.

### Windows fixture setup (PowerShell)

```powershell
$NaspTmp = Join-Path $env:TEMP ("nasp-" + [guid]::NewGuid().ToString("N"))
$SubstrateHome = Join-Path $NaspTmp "substrate-home"
$NaspWs = Join-Path $NaspTmp "ws"

New-Item -ItemType Directory -Force -Path (Join-Path $SubstrateHome "deps\\packages"), $NaspWs | Out-Null
$env:SUBSTRATE_HOME = $SubstrateHome

& substrate workspace init $NaspWs | Out-Null

@"
version: 1
name: smoke-hello
description: NASP smoke fixture (script install).
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
name: smoke-pacman-a
description: NASP smoke fixture (pacman list in authored reverse order).
runnable: false
install:
  method: pacman
  pacman:
    - nasp-pacman-b
    - nasp-pacman-a
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $SubstrateHome "deps\\packages\\smoke-pacman-a.yaml") -NoNewline

@"
version: 1
name: smoke-pacman-b
description: NASP smoke fixture (pacman duplicate for normalization).
runnable: false
install:
  method: pacman
  pacman:
    - nasp-pacman-a
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $SubstrateHome "deps\\packages\\smoke-pacman-b.yaml") -NoNewline

@"
version: 1
name: smoke-apt-a
description: NASP smoke fixture (APT item for mixed-manager validation).
runnable: false
install:
  method: apt
  apt:
    - name: nasp-apt-a
probe:
  command: "sh -c 'exit 1'"
"@ | Set-Content -Path (Join-Path $SubstrateHome "deps\\packages\\smoke-apt-a.yaml") -NoNewline

Push-Location $NaspWs
& substrate world deps global reset | Out-Null
& substrate world deps workspace reset | Out-Null
& substrate world deps workspace add smoke-hello smoke-pacman-a smoke-pacman-b | Out-Null
```

Expected:
- Exit `0`.
- `substrate world deps current list enabled` includes `smoke-pacman-a` and `smoke-pacman-b`.

## 2) Linux host-native — unsupported provisioning

Run:

```bash
substrate world enable --provision-deps --dry-run 1>"$NASP_TMP/linux.stdout" 2>"$NASP_TMP/linux.stderr"
echo "exit=$?"
cat "$NASP_TMP/linux.stderr"
```

Expected:
- Exit `4`.
- Stderr contains:
  - `Substrate will not mutate the host OS`
  - `substrate world enable --provision-deps`

## 3) macOS default Lima guest — mixed-manager rejection

Setup addition:

```bash
substrate world deps workspace add smoke-apt-a >/dev/null
substrate world doctor >/dev/null
```

Run:

```bash
substrate world enable --provision-deps --dry-run --verbose 1>"$NASP_TMP/macos-mixed.stdout" 2>"$NASP_TMP/macos-mixed.stderr"
echo "exit=$?"
cat "$NASP_TMP/macos-mixed.stderr"
```

Expected:
- Exit `4`.
- Stdout contains `world-deps-provision`.
- Stderr contains:
  - `incompatible system-package methods`
  - `apt`

## 4) macOS Arch-family guest — supported pacman provisioning success

This case is manual-only evidence on a non-default Arch-family Lima VM named `substrate`. It is valid only when the fixture assumptions below are true; if any check fails, do not use the result as Arch-family success evidence for this pack.

Fixture assumptions:
- The guest reports an Arch-family identity through `/etc/os-release` by setting `ID=arch` or `ID_LIKE=arch`.
- `pacman` is available on `PATH` inside the guest.
- `/usr/local/bin/substrate` exists and is executable.
- `/usr/local/bin/substrate-world-service` exists and is executable.
- `/run/substrate.sock` exists and is reachable as a socket.

Validate the fixture with all of the following commands:

```bash
limactl shell substrate sh -lc 'grep -E "^(ID|ID_LIKE)=" /etc/os-release'
limactl shell substrate sh -lc 'command -v pacman'
limactl shell substrate sh -lc 'test -x /usr/local/bin/substrate'
limactl shell substrate sh -lc 'test -x /usr/local/bin/substrate-world-service'
limactl shell substrate sudo test -S /run/substrate.sock
```

Expected:
- Exit `0`.
- `/etc/os-release` output contains `ID=arch` or `ID_LIKE=arch`.

In the workspace from fixture setup, leave only pacman-backed items enabled:

```bash
substrate world deps workspace reset >/dev/null
substrate world deps workspace add smoke-pacman-a smoke-pacman-b >/dev/null
```

Run:

```bash
substrate world enable --provision-deps --dry-run --verbose 1>"$NASP_TMP/arch-success.stdout" 2>"$NASP_TMP/arch-success.stderr"
echo "exit=$?"
cat "$NASP_TMP/arch-success.stdout"
```

Expected:
- Exit `0`.
- Stdout contains:
  - `pacman`
  - `world-deps-provision`
  - `nasp-pacman-a`
  - `nasp-pacman-b`
  - `pacman -Sy --noconfirm --needed nasp-pacman-a nasp-pacman-b`

## 5) Runtime fail-early for pacman-backed items

Run:

```bash
substrate world deps current sync --dry-run 1>"$NASP_TMP/runtime.stdout" 2>"$NASP_TMP/runtime.stderr"
echo "exit=$?"
cat "$NASP_TMP/runtime.stdout"
```

Expected:
- Exit `4`.
- Stdout contains, in order:
  - `nasp-pacman-a`
  - `nasp-pacman-b`
- Stderr contains `substrate world enable --provision-deps`.
- Linux: stderr contains `Substrate will not mutate the host OS`.
- Windows: stderr contains `unsupported on Windows`.

## 6) Explicit-item scope

Run:

```bash
substrate world deps current install smoke-hello
echo "exit=$?"
```

Expected:
- Exit `0`.

Run:

```bash
substrate world deps current install smoke-pacman-a --dry-run 1>"$NASP_TMP/install.stdout" 2>"$NASP_TMP/install.stderr"
echo "exit=$?"
cat "$NASP_TMP/install.stdout"
```

Expected:
- Exit `4`.
- Stdout contains:
  - `nasp-pacman-a`
  - `nasp-pacman-b`
- Stderr contains `substrate world enable --provision-deps`.

## 7) Windows unsupported provisioning

Run:

```powershell
& substrate world enable --provision-deps --dry-run 1> "$NaspTmp\\windows.stdout" 2> "$NaspTmp\\windows.stderr"
$LASTEXITCODE
Get-Content -Raw -Path "$NaspTmp\\windows.stderr"
```

Expected:
- Exit `4`.
- Stderr contains:
  - `unsupported on Windows`
  - `substrate world enable --provision-deps`

## 8) Cleanup

Linux and macOS:

```bash
popd >/dev/null
rm -rf "$NASP_TMP"
```

Windows:

```powershell
Pop-Location
Remove-Item -Recurse -Force $NaspTmp
```
