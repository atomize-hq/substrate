# world_process_exec_tracing_parity — schema

This document defines the authoritative trace schema surfaces introduced or tightened by ADR-0028.
It captures the landed command-span fixes and the planned `world_process_*` family in a single place so the pack stays aligned with the current implementation.

## 1) Span records (command_start / command_complete)

### Required invariants
- `parent_span` on `command_complete` MUST equal the parent captured at span start (never re-read env at finish).
- Deny MUST be unambiguous on completion records:
  - `outcome: "denied"` MUST be present on deny completion spans.

### Completion ergonomics
- `duration_ms` MUST be present on completion spans.
- `policy_decision` MUST be present on completion spans whenever the command was policy-evaluated; commands running with policy disabled MAY omit it.

### Joinability fields
- `span_id` MUST be present on shell `command_start` and `command_complete` events when a span exists.
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
- When a span exists for the command, shell `command_start` and `command_complete` events MUST include:
  - `span_id: "spn_..."` (string)

### world_fs_strategy contract fields
- Shell `command_complete` events MUST include:
  - `world_fs_strategy_primary`
  - `world_fs_strategy_final`
  - `world_fs_strategy_fallback_reason`

## 3) World process event family (planned; reserved by ADR-0028 / WPEP1-WPEP3)

This family is the target schema for the new world subprocess telemetry path. The current runtime does not emit these records yet; this section exists so downstream schema and docs can align before implementation lands.

### Event types
- `world_process_start`
- `world_process_exit`

### Required fields
- `ts` (timestamp)
- `ts_unix_ns` (integer nanoseconds since Unix epoch (UTC))
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

## 5) Router-derived event families (Phase 8 additive; owned by ADR-0029)

ADR-0028’s Phase 8 additive correlation vocabulary extends beyond `world_process_*` to cover router-derived events emitted by the host workflow router daemon (ADR-0029). This planning pack does not implement the router, but the canonical trace schema reserves and documents the derived event families so downstream joins are deterministic and non-heuristic.

Authoritative sources:
- ADR-0028 Phase 8 additive correlation vocabulary + matrix: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Router derived-event taxonomy + required fields: `docs/project_management/adrs/draft/ADR-0029-host-event-bus-and-router-daemon.md`

### Event types (v1; additive-only list)
Router-derived events appended to canonical `trace.jsonl` MUST use explicit `event_type` values such as:
- `workflow_router_rule_match`
- `workflow_router_request_enqueued`
- `workflow_router_request_denied`
- `workflow_router_request_pending_approval`
- `workflow_router_action_enqueued`
- `workflow_router_action_executed`
- `workflow_router_cursor_gap_detected`

### Required fields (all router-derived events)
- `ts`
- `event_type` (one of the above)
- `component: "workflow-router"`
- `session_id`
- Correlation / join keys (must be sufficient to avoid heuristic joins):
  - `request_id` (UUID string)
  - `idempotency_key` (hex string)
  - `workspace_id` (source workspace id; UUID string)
  - one cause reference:
    - `source_span_id` when available, and/or
    - `source_cmd_id`
  - `rule_id`

### Optional fields (router-derived events)
- `orchestration_session_id` (when tied to orchestration)
- `target_workspace_id` (when routing targets a distinct workspace)
- `backend_id` (when a specific backend is involved)

## 6) Toolbox tool-call event family (Phase 8 additive; owned by ADR-0026)

The internal orchestration toolbox (ADR-0026) emits tool-call audit records into canonical `trace.jsonl` so control-plane activity is joinable without heuristics. This planning pack does not implement the toolbox, but the canonical trace schema must reserve and document the tool-call event family so later additions do not reshape existing records.

Authoritative sources:
- Tool surface + emission rules: `docs/project_management/adrs/draft/ADR-0026-orchestration-toolbox-mcp.md`
- Correlation vocabulary + required/optional matrix: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

### Event types (v1; additive-only list)
- `toolbox_tool_call_start`
- `toolbox_tool_call_complete`

### Required fields (all toolbox tool-call records)
- `ts`
- `event_type` (one of the above)
- `component: "agent-toolbox"`
- `session_id`
- Correlation / join keys:
  - `orchestration_session_id`
  - `run_id`
  - `agent_id`
  - `backend_id`
  - `tool_call_id`
- Tool identity:
  - `toolbox_version` (v1: `1`)
  - `tool_name` (stable `substrate.*` tool name)

### Completion-only required fields
- `outcome: "ok" | "error" | "denied"`
- `duration_ms`

### Safe-by-default payload posture
- Tool-call records MUST NOT embed full tool request args or full tool response bodies in v1.
  - Records MUST include:
    - `args_omitted: true`
    - `result_omitted: true`
