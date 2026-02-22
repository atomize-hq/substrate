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
  - `bash` (Linux/macOS/WSL)

## Smoke scripts (required)
- Linux: `docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/macos-smoke.sh`
- WSL: run the Linux smoke inside WSL: `docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh`

## Case 1 — Smoke (WDP2 checkpoint boundary: inventory + enabled + applied + explain)

Run your platform’s smoke with:
- `SUBSTRATE_SMOKE_SLICE_ID=WDP2`

Linux:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export SUBSTRATE_SMOKE_SLICE_ID="WDP2"
bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh
```

macOS:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export SUBSTRATE_SMOKE_SLICE_ID="WDP2"
bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/macos-smoke.sh
```

WSL (run inside a WSL shell):
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export SUBSTRATE_SMOKE_SLICE_ID="WDP2"
bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 2 — Smoke (WDP5 checkpoint boundary: install planning + sync apply)

Run your platform’s smoke with:
- `SUBSTRATE_SMOKE_SLICE_ID=WDP5` (default)

Linux:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export SUBSTRATE_SMOKE_SLICE_ID="WDP5"
bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh
```

macOS:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export SUBSTRATE_SMOKE_SLICE_ID="WDP5"
bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/macos-smoke.sh
```

WSL (run inside a WSL shell):
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export SUBSTRATE_SMOKE_SLICE_ID="WDP5"
bash docs/project_management/packs/active/world-deps-packages-bundles-contract/smoke/linux-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## What the smoke scripts enforce (authoritative summary)

The smoke scripts are designed to be deterministic and enforce:
- Legacy-path replacement completeness (intentionally-invalid YAML is placed at legacy file paths; `world deps` must ignore it).
- Non-world views do not require the backend (`current list available|enabled`, `current show` succeed even if `SUBSTRATE_WORLD_SOCKET` points to a missing socket).
- World-backed views fail-closed when the backend is unavailable (`current list applied` and `current show --explain` exit `3` under a missing socket override).
- `current list applied --all` includes non-enabled inventory items.
- `--json` output is parseable and contains required fields (shape stability).
- `blocked` status behavior via a `method=manual` fixture package.
- Install planning (`current install ... --dry-run`) includes apt and script plan content.
- Sync apply (`current sync`) results in `world=present` for a deterministic script-installed entrypoint.

## Optional spot-check — Example inventory (Linux/macOS)

If you want to validate the example inventory definitions under `deps_examples/`:
```bash
set -euo pipefail
tmp_root="$(mktemp -d)"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

substrate workspace init --force >/dev/null

mkdir -p "$SUBSTRATE_HOME/deps/packages" "$SUBSTRATE_HOME/deps/bundles" "$SUBSTRATE_HOME/deps/scripts"
cp -R docs/project_management/packs/active/world-deps-packages-bundles-contract/deps_examples/packages/. "$SUBSTRATE_HOME/deps/packages/"
cp -R docs/project_management/packs/active/world-deps-packages-bundles-contract/deps_examples/bundles/. "$SUBSTRATE_HOME/deps/bundles/"
cp -R docs/project_management/packs/active/world-deps-packages-bundles-contract/deps_examples/scripts/. "$SUBSTRATE_HOME/deps/scripts/"

substrate world deps current list available
```
