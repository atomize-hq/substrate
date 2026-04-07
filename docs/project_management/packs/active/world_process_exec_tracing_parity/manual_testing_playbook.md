# world_process_exec_tracing_parity — manual testing playbook (Authoritative)

Standard:

- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (section 6)

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

Exit code taxonomy for the smoke scripts:

- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Smoke scripts (required)

- Linux: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`

## Case 1 — Smoke (Linux)

Run:

```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh
```

Expected:

- Exit code `0`.
- Output contains an `OK:` line.

## Case 2 — Smoke (macOS)

Run:

```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/macos-smoke.sh
```

Expected:

- Exit code `0`.
- Output contains an `OK:` line.

## Case 3 — Smoke (Windows)

Run:

```powershell
$env:SUBSTRATE_BIN = $env:SUBSTRATE_BIN ?? "substrate"
pwsh -File docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/windows-smoke.ps1
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
  jq -r -s '.[] | select(.component=="shell" and .event_type=="command_complete" and (.command|tostring|contains("bash -lc"))) | .span_id' \
    "$trace" | tail -n 1
)"
test -n "$span_id"

jq -s -e --arg sp "$span_id" '
  any(select(.component=="world-agent" and .event_type=="world_process_start" and .parent_span==$sp))
' "$trace" >/dev/null
```

Expected:

- Both probes succeed (exit `0`).
- On Linux-backed execution, this is the authoritative place to assert that `world_process_*` records are present and joinable by `parent_span`.
- On non-Linux platforms, treat `world_process_*` as a degrade-only surface and do not require the joinability assertion.

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
- Windows does not assert `world_process_*` records for this feature; the degrade summary is the contract.

## Case 7 — Manual validation: wrap builtin canonical trace omits bodies

Run:

```bash
set -euo pipefail
tmp_root="$(mktemp -d)"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
workspace="$tmp_root/workspace"
mkdir -p "$workspace"
cd "$workspace"

substrate workspace init --force >/dev/null
SUBSTRATE_OVERRIDE_WORLD=disabled substrate --command 'export SUBSTRATE_SMOKE_WRAP=1' >/dev/null

trace="$SUBSTRATE_HOME/trace.jsonl"
test -f "$trace"

jq -s -e '
  any(select(.event_type=="builtin_command" and .mode=="wrap") | (.command_omitted==true))
' "$trace" >/dev/null

jq -s -e '
  all(select(.event_type=="builtin_command") | (.command_omitted==true))
' "$trace" >/dev/null

jq -s -e '
  (any(select(.event_type=="builtin_command") | has("command"))) | not
' "$trace" >/dev/null
```

Expected:

- Exit code `0`.
- The wrap command produces at least one `builtin_command` record with `mode: "wrap"` and `command_omitted: true`.
- Canonical trace must not contain raw `command` bodies for `builtin_command` records.
- This case intentionally asserts the observable wrap-mode builtin-routing path directly.
- Script-mode `SUBSTRATE_ENABLE_PREEXEC` wiring remains part of the published matrix, but the deterministic proof for that cell lives in `crates/shell/tests/shell_env.rs` (`shell_env_script_mode_sets_preexec_flag_for_bash`) rather than this operator-facing smoke flow.
- Interactive preexec removal remains a runtime/code-path invariant rather than a playbook assertion here.
