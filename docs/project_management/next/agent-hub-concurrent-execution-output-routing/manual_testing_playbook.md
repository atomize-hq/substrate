# agent-hub-concurrent-execution-output-routing — manual testing playbook (Authoritative)

Standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (section 6)

## Scope

Validates ADR-0017 operator-visible behavior for concurrent structured agent events during interactive use:
- structured agent events are persisted to canonical trace as `event_type="agent_event"` records with stable join keys,
- PTY passthrough never receives injected structured output,
- structured events during PTY passthrough are buffered up to a deterministic cap and dropped beyond the cap,
- a deterministic suppression warning is emitted when drops occur,
- config clamp and warning record semantics for `repl.max_pty_buffered_lines`.

## Prerequisites

- Rust toolchain available for running targeted test suites (`cargo test`).
- `substrate` available on PATH, or set `SUBSTRATE_BIN=/path/to/substrate`.
- Tools:
  - `jq`
  - `bash` (Linux/macOS)
  - PowerShell 7 (`pwsh`) (Windows)

Exit code taxonomy for Substrate CLI behavior:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Smoke scripts (required)

- Linux: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1`

## Case 1 — Smoke (Linux)

Run:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/linux-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 2 — Smoke (macOS)

Run:
```bash
export SUBSTRATE_BIN="${SUBSTRATE_BIN:-substrate}"
bash docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/macos-smoke.sh
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 3 — Smoke (Windows)

Run:
```powershell
$env:SUBSTRATE_BIN = $env:SUBSTRATE_BIN ?? "substrate"
pwsh -NoProfile -File docs/project_management/next/agent-hub-concurrent-execution-output-routing/smoke/windows-smoke.ps1
```

Expected:
- Exit code `0`.
- Output contains an `OK:` line.

## Case 4 — Interactive validation (Linux/macOS): no PTY injection + drop warning emitted

Run in a temporary workspace and temporary Substrate home:
```bash
set -euo pipefail
tmp_root="$(mktemp -d)"
export SUBSTRATE_HOME="$tmp_root/substrate-home"
workspace="$tmp_root/ws"
mkdir -p "$workspace"
cd "$workspace"

substrate workspace init --force >/dev/null

# Force deterministic drop behavior for this manual case:
cat >"$workspace/.substrate/workspace.yaml" <<'YAML'
repl:
  max_pty_buffered_lines: 0
YAML

substrate --no-world
```

In the interactive REPL session, run:
1) Start the demo structured-event producer:
   - `:demo-agent`
2) Immediately start a PTY passthrough command that overlaps with demo output:
   - `:pty bash -lc 'echo PTY_START; sleep 2; echo PTY_END'`
3) Exit the REPL:
   - `:quit`

Expected:
- Between `PTY_START` and `PTY_END`, no structured agent event lines are printed.
- After `PTY_END`, a human-readable warning line is printed indicating structured output was suppressed during PTY passthrough.

Trace assertions (after exiting REPL):
```bash
trace="$SUBSTRATE_HOME/trace.jsonl"
test -f "$trace"

# Agent events exist and are flattened (no nested payload object).
jq -e 'any(select(.event_type=="agent_event" and .component=="agent-hub" and (.orchestration_session_id|type=="string") and (.run_id|type=="string") and (.data|type=="object")))' \
  "$trace" >/dev/null

# Drop warning exists with required fields.
jq -e 'any(select(.event_type=="warning" and .component=="shell" and .code=="pty_structured_event_drops" and (.dropped_structured_event_lines|type=="number") and (.max_pty_buffered_lines|type=="number")))' \
  "$trace" >/dev/null
```

Expected:
- Both `jq -e` probes succeed (exit `0`).

## Case 5 — Windows validation: agent_event persistence (PTY passthrough not applicable)

Run in a temporary workspace and temporary Substrate home:
```powershell
$ErrorActionPreference = "Stop"
$tmp = New-Item -ItemType Directory -Force -Path ([System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.Guid]::NewGuid().ToString()))
$env:SUBSTRATE_HOME = Join-Path $tmp "substrate-home"
$workspace = Join-Path $tmp "ws"
New-Item -ItemType Directory -Force -Path $workspace | Out-Null
Set-Location $workspace

substrate workspace init --force | Out-Null
substrate --no-world --command ":demo-agent" | Out-Null

$trace = Join-Path $env:SUBSTRATE_HOME "trace.jsonl"
if (!(Test-Path $trace)) { throw "trace.jsonl missing: $trace" }

Get-Content $trace | jq -e 'any(select(.event_type=="agent_event" and .component=="agent-hub"))' | Out-Null
```

Expected:
- Exit code `0`.
- At least one `event_type="agent_event"` record exists.
