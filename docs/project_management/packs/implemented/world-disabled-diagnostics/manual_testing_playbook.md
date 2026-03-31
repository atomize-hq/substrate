# world-disabled-diagnostics — manual testing playbook (Authoritative)

Standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (section 6)

Exit code taxonomy:
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Scope

Validates the contract defined by:
- `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md`
- `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`

Commands in scope:
- `substrate shim doctor` (text + `--json`)
- `substrate health` (text + `--json`)

This playbook focuses on the two operator-relevant postures:
1) World disabled (`effective world.enabled=false`) ⇒ explicit disabled/skipped statuses, **no probes**, exit `0`.
2) World enabled but backend unreachable ⇒ fail-visible “needs attention”, exit `0`.

It also includes a deterministic config-resolution failure case (exit `2`) aligned to `contract.md`.

## Prerequisites

- `substrate` available on PATH, or set `SUBSTRATE_BIN=/path/to/substrate` (`SUBSTRATE_EXE=...` on Windows).
- Run from a directory that is **not** an enabled workspace (so `SUBSTRATE_OVERRIDE_WORLD` is not ignored).
  - Recommended: use a temporary directory as the working directory for all cases.

## Smoke scripts (required)

These scripts implement the same assertions as this playbook and are slice-scoped via `SUBSTRATE_SMOKE_SLICE_ID`:

- Linux: `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/draft/world-disabled-diagnostics/smoke/windows-smoke.ps1`

Slice selection:
- `SUBSTRATE_SMOKE_SLICE_ID=WDD0|WDD1|WDD2` (default: `WDD2`)

## Case 1 — World disabled: explicit disabled/skipped + no probes (exit 0)

Preconditions:
- Effective config resolves `world.enabled=false` (example input: `SUBSTRATE_OVERRIDE_WORLD=disabled` outside an enabled workspace).

### Commands (Linux/macOS — bash)

```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export WORKDIR="$(mktemp -d)"
export SUBSTRATE_HOME="$(mktemp -d)"
export SUBSTRATE_OVERRIDE_WORLD=disabled
cd "$WORKDIR"

"$SUBSTRATE_BIN" shim doctor
"$SUBSTRATE_BIN" shim doctor --json > shim_doctor.disabled.json

"$SUBSTRATE_BIN" health
"$SUBSTRATE_BIN" health --json > health.disabled.json
```

Expected exit codes:
- All four commands exit `0`.

Expected text output assertions (verbatim required lines from `contract.md`):
`substrate shim doctor` includes:

```text
World backend:
  Status: disabled
  Next: run `substrate world enable` to provision
World deps:
  Status: skipped (world disabled)
```

`substrate health` includes:

```text
World backend: disabled
  Next: run `substrate world enable` to provision
World deps: skipped (world disabled)
```

`substrate health` MUST NOT print enabled-world world-deps remediation hints (for example, MUST NOT contain ``substrate world deps current``).

Expected JSON assertions (paths from `world-disabled-diagnostics-json-schema-spec.md`):

```bash
python3 - <<'PY'
import json

with open("shim_doctor.disabled.json", "r", encoding="utf-8") as f:
    d = json.load(f)
assert d["world"]["status"] == "disabled"
assert d["world_deps"]["status"] == "skipped_disabled"
for key in ("error", "stderr", "exit_code", "details"):
    assert key not in d["world"], f"world.{key} must be omitted when disabled"
for key in ("error", "report"):
    assert key not in d["world_deps"], f"world_deps.{key} must be omitted when skipped_disabled"

with open("health.disabled.json", "r", encoding="utf-8") as f:
    h = json.load(f)
assert h["shim"]["world"]["status"] == "disabled"
assert h["shim"]["world_deps"]["status"] == "skipped_disabled"
summary = h["summary"]
assert summary["world_ok"] is None
assert "world_error" not in summary
assert "world_deps_error" not in summary
assert summary["world_deps_missing"] == []
assert summary["world_deps_blocked"] == []
PY
```

### Commands (Windows — PowerShell)

```powershell
$Substrate = if ($env:SUBSTRATE_EXE) { $env:SUBSTRATE_EXE } elseif ($env:SUBSTRATE_BIN) { $env:SUBSTRATE_BIN } else { "substrate" }
$env:SUBSTRATE_HOME = Join-Path $env:TEMP ("wdd-home-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $env:SUBSTRATE_HOME | Out-Null
$env:SUBSTRATE_OVERRIDE_WORLD = "disabled"
$WorkDir = Join-Path $env:TEMP ("wdd-work-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $WorkDir | Out-Null
Push-Location $WorkDir

& $Substrate shim doctor | Out-String | Out-File -Encoding utf8 shim_doctor.disabled.txt
& $Substrate shim doctor --json | Out-File -Encoding utf8 shim_doctor.disabled.json

& $Substrate health | Out-String | Out-File -Encoding utf8 health.disabled.txt
& $Substrate health --json | Out-File -Encoding utf8 health.disabled.json

Pop-Location
```

Expected:
- Same exit codes and assertions as the bash section (check the same required lines and JSON fields).

## Case 2 — World enabled, backend intentionally broken: fail-visible “needs attention” (exit 0)

Preconditions:
- Force enabled with `--world`.
- Ensure the backend is unreachable:
  - Linux/macOS: set `SUBSTRATE_WORLD_SOCKET` to a nonexistent path.
  - Windows: set `SUBSTRATE_FORWARDER_PIPE` to a nonexistent pipe.

### Commands (Linux/macOS — bash)

```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export WORKDIR="$(mktemp -d)"
export SUBSTRATE_HOME="$(mktemp -d)"
export SUBSTRATE_WORLD_SOCKET="$SUBSTRATE_HOME/does-not-exist.sock"
cd "$WORKDIR"

"$SUBSTRATE_BIN" --world shim doctor
"$SUBSTRATE_BIN" --world shim doctor --json > shim_doctor.broken.json

"$SUBSTRATE_BIN" --world health
"$SUBSTRATE_BIN" --world health --json > health.broken.json
```

Expected exit codes:
- All four commands exit `0` (report generated successfully).

Expected text output assertions (key lines; enabled-mode copy is not fully templated):
- `substrate shim doctor` includes:
  - `World backend:`
  - `  Status: needs attention`
  - at least one `  Error:` line in the world backend section
- `substrate health` includes:
  - `World backend: needs attention`
  - `  Error:`
  - `Overall status: attention required`

Expected JSON assertions:

```bash
python3 - <<'PY'
import json

with open("shim_doctor.broken.json", "r", encoding="utf-8") as f:
    d = json.load(f)
assert d["world"]["status"] == "needs_attention"
assert isinstance(d["world"].get("error"), str) and d["world"]["error"].strip()
assert d["world_deps"]["status"] == "error"
assert isinstance(d["world_deps"].get("error"), str) and d["world_deps"]["error"].strip()

with open("health.broken.json", "r", encoding="utf-8") as f:
    h = json.load(f)
assert h["shim"]["world"]["status"] == "needs_attention"
assert h["summary"]["world_ok"] is False
assert isinstance(h["summary"].get("world_error"), str) and h["summary"]["world_error"].strip()
assert h["shim"]["world_deps"]["status"] == "error"
assert isinstance(h["summary"].get("world_deps_error"), str) and h["summary"]["world_deps_error"].strip()
PY
```

### Commands (Windows — PowerShell)

```powershell
$Substrate = if ($env:SUBSTRATE_EXE) { $env:SUBSTRATE_EXE } elseif ($env:SUBSTRATE_BIN) { $env:SUBSTRATE_BIN } else { "substrate" }
$env:SUBSTRATE_HOME = Join-Path $env:TEMP ("wdd-home-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $env:SUBSTRATE_HOME | Out-Null
$env:SUBSTRATE_FORWARDER_PIPE = "\\\\.\\pipe\\substrate-agent-wdd-broken-$([guid]::NewGuid())"
$env:SUBSTRATE_FORWARDER_TCP = "0"
$WorkDir = Join-Path $env:TEMP ("wdd-work-" + [guid]::NewGuid())
New-Item -ItemType Directory -Path $WorkDir | Out-Null
Push-Location $WorkDir

& $Substrate --world shim doctor --json | Out-File -Encoding utf8 shim_doctor.broken.json
& $Substrate --world health --json | Out-File -Encoding utf8 health.broken.json

Pop-Location
```

Expected:
- Same exit codes and JSON assertions as the bash section.

## Case 3 — Effective-config resolution failure: fail fast (exit 2)

Preconditions:
- `$SUBSTRATE_HOME/config.yaml` contains invalid YAML.

Commands (Linux/macOS — bash):

```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
export WORKDIR="$(mktemp -d)"
export SUBSTRATE_HOME="$(mktemp -d)"
printf 'world: [\n' > "$SUBSTRATE_HOME/config.yaml"
cd "$WORKDIR"

set +e
"$SUBSTRATE_BIN" shim doctor >/dev/null 2>&1
test "$?" -eq 2
"$SUBSTRATE_BIN" health >/dev/null 2>&1
test "$?" -eq 2
set -e
```

Expected:
- Both commands exit `2`.
- Stderr names the offending path (must include `$SUBSTRATE_HOME/config.yaml` or `config.yaml`).
