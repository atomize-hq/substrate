# world_process_exec_tracing_parity — schema

This document defines the authoritative trace schema surfaces introduced or tightened by ADR-0028.

## 1) Span records (command_start / command_complete)

### Required invariants
- `parent_span` on `command_complete` MUST equal the parent captured at span start (never re-read env at finish).
- Deny MUST be unambiguous on completion records:
  - `outcome: "denied"` MUST be present on deny completion spans.

### Completion ergonomics (recommended / required by spec slices)
- `duration_ms` SHOULD be present on completion spans.
- `policy_decision` SHOULD be present on completion spans when known at start.

### Joinability fields
- `parent_cmd_id` (optional):
  - When present, it MUST equal `SHIM_PARENT_CMD_ID` for the execution.

## 2) Shell command summary events (shell telemetry)

Shell emits command summary events via `log_command_event`:
- `event_type: command_start`
- `event_type: command_complete`

### Process telemetry diagnostics (world executions)
For any world execution summary (`component: "shell"` and `event_type: "command_complete"` where the command ran via world-agent):
- Shell MUST include:
  - `process_events_status`: `"ok" | "unavailable" | "truncated" | "error"`
  - `process_events_reason`: string when status != `"ok"`
- When `process_events_status="truncated"`, shell MUST include:
  - `process_events_dropped`: integer

### Joinability
- When a span exists for the command, shell `command_start` and `command_complete` events SHOULD include:
  - `span_id: "spn_..."` (string)

### world_fs_strategy contract fields
- Shell `command_complete` events MUST include:
  - `world_fs_strategy_primary`
  - `world_fs_strategy_final`
  - `world_fs_strategy_fallback_reason`

## 3) World process event family (new)

### Event types
- `world_process_start`
- `world_process_exit`

### Required fields
- `ts` (timestamp)
- `event_type` (one of the above)
- `component: "world-agent"`
- `session_id`
- `world_id`
- `pid`, `ppid`
- `cwd`
- One of:
  - `argv` (redacted array)
  - `argv_omitted: true`
- Correlation:
  - `parent_span` (host span id)
  - `parent_cmd_id` (optional)

### Exit-only fields
- `exit_code` (or signal termination)
- `duration_ms`

### Optional fields
- `env` (redacted map; allowlist-only by default)
- `exe` (best-effort)
- `signal`

### Caps/truncation
- Max events per execution: 10,000 (default).
- Max env value length: 4KB/value (default).

Truncation reporting lives in the protocol layer (`process_events_status: "truncated"` and `process_events_dropped`).

## 4) Preexec/builtin command tracing

### Canonical trace
When `SUBSTRATE_ENABLE_PREEXEC=1`:
- `event_type: builtin_command` records in `trace.jsonl` MUST omit command bodies.
- Canonical record MUST include:
  - `command_omitted: true`
  - `parent_cmd_id` when available

### Debug-only raw log
When `SUBSTRATE_PREEXEC_RAW_LOG` is set:
- Write `event_type: builtin_command_raw` records to that file path.
- Raw records MUST include:
  - `may_contain_secrets: true`
