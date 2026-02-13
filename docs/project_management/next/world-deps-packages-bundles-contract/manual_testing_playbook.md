# world-deps-packages-bundles-contract — manual testing playbook (Authoritative)

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (section 6)

## Scope
Validates ADR-0011 operator-visible behavior for `substrate world deps`:
- inventory visibility (built-ins + global + workspace),
- enabled patch editing and effective enabled list resolution,
- world-backed status (`present|missing|blocked`),
- install/sync behavior (`apt` + `script` + `manual` blocked),
- legacy path removal (replacement completeness).

Exit code taxonomy:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Prerequisites
- `substrate` available on PATH, or set `SUBSTRATE_BIN=/path/to/substrate`.
- World backend healthy for the platform under test:
  - Run `substrate world doctor` and fix any reported issues before proceeding.
- Tools:
  - `bash` (Linux/macOS)
  - PowerShell (`pwsh`) (Windows)

## Smoke scripts (required)
- Linux: `docs/project_management/next/world-deps-packages-bundles-contract/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world-deps-packages-bundles-contract/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world-deps-packages-bundles-contract/smoke/windows-smoke.ps1`

## Case 1 — Smoke (Linux)
Run:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/next/world-deps-packages-bundles-contract/smoke/linux-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 2 — Smoke (macOS)
Run:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/next/world-deps-packages-bundles-contract/smoke/macos-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 3 — Smoke (Windows)
Run:
```powershell
$env:SUBSTRATE_BIN = $env:SUBSTRATE_BIN ?? "substrate"
pwsh -File docs/project_management/next/world-deps-packages-bundles-contract/smoke/windows-smoke.ps1
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 4 — Inventory merge sanity (all platforms)

Setup a temp workspace and SUBSTRATE_HOME:
```bash
set -euo pipefail
tmp_root="$(mktemp -d)"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

substrate workspace init --force >/dev/null
```

Install the example inventory under global scope:
```bash
mkdir -p "$SUBSTRATE_HOME/deps/packages" "$SUBSTRATE_HOME/deps/bundles" "$SUBSTRATE_HOME/deps/scripts"
cp -R docs/project_management/next/world-deps-packages-bundles-contract/deps_examples/packages/. "$SUBSTRATE_HOME/deps/packages/"
cp -R docs/project_management/next/world-deps-packages-bundles-contract/deps_examples/bundles/. "$SUBSTRATE_HOME/deps/bundles/"
cp -R docs/project_management/next/world-deps-packages-bundles-contract/deps_examples/scripts/. "$SUBSTRATE_HOME/deps/scripts/"
```

Run:
```bash
substrate world deps current list available
```

Expected:
- Exit `0`.
- Output includes `bun` and `node-runtime`.

## Case 5 — Enabled patch editing + effective enabled view (all platforms)

Run:
```bash
substrate world deps global add bun node-runtime
substrate world deps workspace add python-build-deps
substrate world deps current list enabled
```

Expected:
- `global add` and `workspace add` exit `0`.
- `current list enabled` exits `0` and includes (in-order): `bun`, `node-runtime`, `python-build-deps`.

## Case 6 — World-backed status (requires backend)

Run:
```bash
substrate world doctor >/dev/null
substrate world deps current list applied
```

Expected:
- Exit `0`.
- Output includes `world=present|missing|blocked` for each enabled item.

