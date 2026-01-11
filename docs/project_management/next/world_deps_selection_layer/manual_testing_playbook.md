# World Deps Selection Layer — Manual Testing Playbook (WDL0/WDL1/WDL2)

Authoritative specs:
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Which sections must be run (per slice closeout)

This playbook is organized so slice closeout is deterministic:

| Closeout target | Required sections |
|---|---|
| `WDL0-closeout_report.md` | Sections `0`, `1`, `2` |
| `WDL1-closeout_report.md` | Sections `0`, `1`, `2`, `3` |
| `WDL2-closeout_report.md` | Sections `0`, `1`, `2`, `3`, `4` |

Constraint:
- Section `4` must not be executed unless `substrate world deps --help` contains the `provision` subcommand (WDL2 capability).

---

## Automated smoke scripts (preferred first step)

Run the platform smoke script locally (fast preflight):
- Linux: `bash docs/project_management/next/world_deps_selection_layer/smoke/linux-smoke.sh`
- macOS: `bash docs/project_management/next/world_deps_selection_layer/smoke/macos-smoke.sh`
- Windows: `pwsh -File docs/project_management/next/world_deps_selection_layer/smoke/windows-smoke.ps1`

Cross-platform smoke via CI (preferred for audit + parity):
- Behavioral smoke (all behavior platforms, from the orchestration ref):
  - `make feature-smoke FEATURE_DIR="docs/project_management/next/world_deps_selection_layer" PLATFORM=behavior RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world_deps_selection_layer" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`
- Compile parity (GitHub-hosted runners; fast fail for macOS/Windows compilation):
  - `make ci-compile-parity CI_WORKFLOW_REF="feat/world_deps_selection_layer" CI_REMOTE=origin CI_CLEANUP=1`

---

## 0) Test fixtures (all platforms)

This playbook must not modify your real `~/.substrate` state. Use an isolated `SUBSTRATE_HOME`.

### Linux/macOS (bash)

```bash
set -euo pipefail

export WDL_FIXTURE_HOME="$(mktemp -d)"
export WDL_FIXTURE_WS="$(mktemp -d)"
export SUBSTRATE_HOME="$WDL_FIXTURE_HOME/.substrate"

mkdir -p "$SUBSTRATE_HOME"
cd "$WDL_FIXTURE_WS"
```

### Windows (PowerShell)

```powershell
$ErrorActionPreference = "Stop"

$env:WDL_FIXTURE_HOME = Join-Path $env:TEMP ("substrate-wdl-home-" + [guid]::NewGuid().ToString("N"))
$env:WDL_FIXTURE_WS = Join-Path $env:TEMP ("substrate-wdl-ws-" + [guid]::NewGuid().ToString("N"))
$env:SUBSTRATE_HOME = Join-Path $env:WDL_FIXTURE_HOME ".substrate"

New-Item -ItemType Directory -Force -Path $env:SUBSTRATE_HOME | Out-Null
New-Item -ItemType Directory -Force -Path $env:WDL_FIXTURE_WS | Out-Null
Set-Location $env:WDL_FIXTURE_WS
```

---

## 1) WDL0 — Selection gating (unconfigured selection is a no-op)

Goal:
- Prove the “selection missing” no-op contract (exit `0`, actionable message, no world backend calls).

### 1.1 Clear selection files

Linux/macOS:
```bash
rm -rf .substrate
rm -f "$SUBSTRATE_HOME/world-deps.selection.yaml"
```

Windows:
```powershell
Remove-Item -Recurse -Force -ErrorAction SilentlyContinue .substrate
Remove-Item -Force -ErrorAction SilentlyContinue (Join-Path $env:SUBSTRATE_HOME "world-deps.selection.yaml")
```

### 1.2 Prove “no world calls” using an invalid socket override

Linux/macOS:
```bash
export SUBSTRATE_WORLD_SOCKET="$WDL_FIXTURE_HOME/does-not-exist.sock"
```

Windows:
```powershell
$env:SUBSTRATE_WORLD_SOCKET = Join-Path $env:WDL_FIXTURE_HOME "does-not-exist.sock"
```

Run (all platforms):
- `substrate world deps status`
- `substrate world deps sync`
- `substrate world deps install nvm`
- If `substrate world deps --help` contains `provision`: also run `substrate world deps provision`

Expected (all platforms):
- Each command exits `0`.
- Each command prints the exact substring: `world deps not configured (selection file missing)`.

---

## 2) WDL0 — Selection init/select semantics (configured but empty selection)

Goal:
- Prove `init/select` write the selection file deterministically and that empty selection is valid configuration.

### 2.1 Initialize workspace selection

Run:
- `substrate world deps init --workspace --force`

Expected:
- Exit `0`.
- Creates `.substrate/world-deps.selection.yaml` (and creates `.substrate/` if missing).

### 2.2 Validate configured-but-empty behavior (no world calls)

Keep `SUBSTRATE_WORLD_SOCKET` pointing to a non-existent path (from section `1.2`).

Run:
- `substrate world deps status --json`
- `substrate world deps sync`
- `substrate world deps install nvm`

Expected:
- `status --json` exits `0` and includes:
  - `selection.configured=true`
  - `selection.active_scope="workspace"`
  - `selection.active_path=".substrate/world-deps.selection.yaml"`
  - `selection.selected` is an empty list
- `sync` exits `0` and prints the exact substring: `No tools selected; nothing to do.`
- `install nvm` exits `2` and prints the exact substring: `tool not selected`

JSON assertion examples:

Linux/macOS:
```bash
substrate world deps status --json | jq -e '.selection.configured==true and .selection.active_scope=="workspace" and (.selection.selected|length)==0' >/dev/null
```

Windows:
```powershell
$s = (substrate world deps status --json | ConvertFrom-Json)
if (-not $s.selection.configured) { throw "expected selection.configured=true" }
if ($s.selection.active_scope -ne "workspace") { throw "expected selection.active_scope=workspace" }
if ($s.selection.selected.Count -ne 0) { throw "expected empty selection.selected" }
```

### 2.3 Select tools (workspace scope) and validate selection normalization

Run:
- `substrate world deps select --workspace nvm bun`
- `substrate world deps status --json`

Expected:
- Exit `0`.
- JSON includes `selection.selected` containing `["nvm","bun"]` (lower-case normalization).

---

## 3) WDL1 — Install class visibility + routing signals (status/sync/install)

Goal:
- Prove install class metadata is surfaced and that `system_packages` tools are routed to provisioning (not runtime installs).

Precondition (must be true before running this section):
- `substrate world deps status --json` includes `tools[].install_class` (WDL1 capability).

Linux/macOS:
```bash
substrate world deps status --all --json | jq -e '.tools[0].install_class? != null' >/dev/null
```

Windows:
```powershell
$s = (substrate world deps status --all --json | ConvertFrom-Json)
if ($null -eq $s.tools[0].install_class) { throw "expected tools[].install_class to be present" }
```

### 3.1 Assert install class values (inventory expectations)

Run:
- `substrate world deps status --all --json`

Expected (must hold simultaneously):
- Tool `bun` has `install_class="user_space"`.
- Tool `pyenv` has `install_class="system_packages"`.

Linux/macOS:
```bash
substrate world deps status --all --json | jq -e '
  ( .tools[] | select(.name=="bun") | .install_class=="user_space" )
  and
  ( .tools[] | select(.name=="pyenv") | .install_class=="system_packages" )
' >/dev/null
```

Windows:
```powershell
$s = (substrate world deps status --all --json | ConvertFrom-Json)
$bun = $s.tools | Where-Object { $_.name -eq "bun" } | Select-Object -First 1
$pyenv = $s.tools | Where-Object { $_.name -eq "pyenv" } | Select-Object -First 1
if ($bun.install_class -ne "user_space") { throw "expected bun.install_class=user_space" }
if ($pyenv.install_class -ne "system_packages") { throw "expected pyenv.install_class=system_packages" }
```

### 3.2 Assert runtime routing signal for `system_packages`

Goal:
- Validate routing deterministically without relying on the guest image’s preinstalled package set.

Requirement:
- This step requires a reachable world backend because `sync` is an action command (exit `3` when backend unavailable).

Create a deterministic `system_packages` fixture tool in the user overlay (scoped to `SUBSTRATE_HOME`):

Linux/macOS:
```bash
cat > "$SUBSTRATE_HOME/manager_hooks.local.yaml" <<'YAML'
version: 2
managers:
  - name: wdl-smoke-system-packages
    guest_detect:
      command: "dpkg -s cowsay >/dev/null 2>&1"
    guest_install:
      class: system_packages
      system_packages:
        apt:
          - cowsay
YAML
```

Windows:
```powershell
$overlayPath = Join-Path $env:SUBSTRATE_HOME "manager_hooks.local.yaml"
$overlay = @"
version: 2
managers:
  - name: wdl-smoke-system-packages
    guest_detect:
      command: "dpkg -s cowsay >/dev/null 2>&1"
    guest_install:
      class: system_packages
      system_packages:
        apt:
          - cowsay
"@
Set-Content -LiteralPath $overlayPath -Value $overlay -Encoding UTF8
```

Ensure backend readiness:

Linux/macOS:
```bash
unset SUBSTRATE_WORLD_SOCKET
substrate world doctor --json | jq -e '.world.ok==true' >/dev/null
```

Windows:
```powershell
Remove-Item Env:SUBSTRATE_WORLD_SOCKET -ErrorAction SilentlyContinue
$d = (substrate world doctor --json | ConvertFrom-Json)
if (-not $d.world.ok) { throw "expected world.ok=true before running sync checks" }
```

Select the fixture tool and inspect its current guest status:

Linux/macOS:
```bash
substrate world deps select --workspace wdl-smoke-system-packages
substrate world deps status --all --json | jq '.tools[] | select(.name=="wdl-smoke-system-packages") | {install_class, guest: .guest.status}'
```

Windows:
```powershell
substrate world deps select --workspace wdl-smoke-system-packages
$s = (substrate world deps status --all --json | ConvertFrom-Json)
($s.tools | Where-Object { $_.name -eq "wdl-smoke-system-packages" } | Select-Object -First 1) | Select-Object install_class, @{Name="guest_status";Expression={$_.guest.status}}
```

Expected (must hold):
- `install_class="system_packages"`.
- If the tool’s `guest.status` is `skipped` (probe failing):
  - `substrate world deps sync` exits `4` and output contains the exact substring: `substrate world deps provision`.
- If the tool’s `guest.status` is `present` (probe already satisfied):
  - `substrate world deps sync` exits `0`.

---

## 4) WDL2 — Provisioning (`world deps provision`)

Goal:
- Prove `system_packages` are fulfilled only by explicit provisioning, and that provisioning behavior matches the platform strategy.

Preconditions (must hold before running this section):
- `substrate world deps --help` contains the `provision` subcommand (WDL2 capability).
- The workspace selection includes at least one `system_packages` tool (use `pyenv`).

### 4.1 Linux host backend: explicit unsupported error

Run:
- `substrate world deps provision --all`

Expected:
- Exit `4`.
- Output contains the exact substring: `unsupported on Linux host backend`.
- Output prints a non-empty package list under a “required system packages” heading.

### 4.2 macOS (Lima) and Windows (WSL): provisioning succeeds and is idempotent

Requirement:
- World backend must be reachable:
  - `substrate world doctor --json` must indicate `.world.ok==true`.

Run:
- `substrate world deps provision`
- Re-run: `substrate world deps provision`

Expected:
- Both runs exit `0`.
- Output contains a non-empty apt package list and the exact substring: `system packages installed`.

### 4.3 Proof that `system_packages` becomes “present”

Run:
- `substrate world deps status --json`

Expected:
- Tool `wdl-smoke-system-packages` is reported as `guest.status="present"` only if its `guest_detect.command` probe succeeds (DR-0014).
