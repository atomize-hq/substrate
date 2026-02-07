# world_process_exec_tracing_parity — manual testing playbook (Authoritative)

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (section 6)

## Scope
- Validates ADR-0028 operator-visible behavior:
  - in-world process exec/exit telemetry persistence into canonical trace,
  - explicit degrade diagnostics on unsupported platforms,
  - span correctness/joinability ergonomics and preexec safety posture.

## Prerequisites
- `substrate` available on PATH, or set `SUBSTRATE_BIN=/path/to/substrate`.
- World backend healthy for the platform under test:
  - Run `substrate world doctor` and fix any reported issues before proceeding.
- Tools:
  - `jq`
  - `bash` (Linux/macOS)
  - PowerShell (Windows)

## Smoke scripts (required)
- Linux: `docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`

## Case 1 — Smoke (Linux)
Run:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/next/world_process_exec_tracing_parity/smoke/linux-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 2 — Smoke (macOS)
Run:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/next/world_process_exec_tracing_parity/smoke/macos-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 3 — Smoke (Windows)
Run:
```powershell
$env:SUBSTRATE_BIN = $env:SUBSTRATE_BIN ?? "substrate"
pwsh -File docs/project_management/next/world_process_exec_tracing_parity/smoke/windows-smoke.ps1
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 4 — Manual validation: process events persisted and joinable (Linux)

Run in a temp home/workspace:
```bash
set -euo pipefail
tmp_root="$(mktemp -d)"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

substrate workspace init --force >/dev/null

substrate world doctor >/dev/null
substrate --world --command 'bash -lc "echo parent; sh -lc true; echo done"' >/dev/null

trace="$SUBSTRATE_HOME/trace.jsonl"
test -f "$trace"
```

Assertions:
- A shell `command_complete` record exists for the world command and contains `span_id`.
- At least one `world_process_start` record exists with `parent_span` equal to that `span_id`.

Example jq probes:
```bash
span_id="$(
  jq -r 'select(.component=="shell" and .event_type=="command_complete" and (.command|tostring|contains("bash -lc"))) | .span_id' \
    "$trace" | tail -n 1
)"
test -n "$span_id"

jq -e --arg sp "$span_id" '
  any(select(.component=="world-agent" and .event_type=="world_process_start" and .parent_span==$sp))
' "$trace" >/dev/null
```

Expected:
- Both probes succeed (exit `0`).

## Case 5 — Manual validation: argv omission vs argv capture (Linux-backed backends)

WPEP2 expectation:
- `world_process_*` records emit `argv_omitted: true`.

WPEP3 expectation:
- At least one `world_process_start` record includes `argv` (array) and no `argv_omitted` fields exist.

## Case 6 — Manual validation: explicit degrade diagnostics (Windows)

Run a simple world command and confirm the shell completion record includes an explicit “unavailable” diagnostic for process telemetry.

Expected:
- World command completes successfully.
- The corresponding `component: "shell"` `event_type: "command_complete"` record includes:
  - `process_events_status: "unavailable"`
  - `process_events_reason: "not_supported_platform"`

## Case 7 — Manual validation: preexec canonical trace omits bodies

Run:
```bash
set -euo pipefail
tmp_root="$(mktemp -d)"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

substrate workspace init --force >/dev/null
SUBSTRATE_ENABLE_PREEXEC=1 substrate --command 'echo hello' >/dev/null

trace="$SUBSTRATE_HOME/trace.jsonl"
test -f "$trace"

jq -e '
  any(select(.event_type=="builtin_command") | (.command_omitted==true))
' "$trace" >/dev/null
```

Expected:
- Exit code `0` and at least one `builtin_command` record has `command_omitted: true`.
