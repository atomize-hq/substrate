# Trace Schema (Internal)

This document is the stable internal schema reference for canonical trace record families.

Related sources:
- [docs/TRACE.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md)
- `docs/adr/implemented/ADR-0028-in-world-process-execution-tracing-parity.md`

## 1. Span records (`command_start` / `command_complete`)

### Required invariants

- `parent_span` on `command_complete` must equal the parent captured at span start.
- Deny completion spans must include `outcome: "denied"`.
- `duration_ms` must be present on completion spans.
- `policy_decision` must be present on completion spans whenever the command was policy-evaluated.
- `span_id` must be present on shell `command_start` and `command_complete` events when a span exists.
- `parent_cmd_id`, when present, must equal `SHIM_PARENT_CMD_ID` for the execution.

### Shell command summary fields

Shell `command_complete` events must include:
- `world_fs_strategy_primary`
- `world_fs_strategy_final`
- `world_fs_strategy_fallback_reason`

For world-backed executions, shell completion events must also include:
- `process_events_status`
- `process_events_reason` when status is not `ok`
- `process_events_dropped` when status is `truncated`

## 2. World process event family

This family is the canonical subprocess exec/exit telemetry path.

Event types:
- `world_process_start`
- `world_process_exit`

### Required fields

- `ts`
- `ts_unix_ns`
- `event_type`
- `component: "world-service"`
- `session_id`
- `world_id`
- `pid`
- `ppid`
- `cwd`
- exactly one of:
  - `argv`
  - `argv_omitted: true`
- `parent_span`
- `parent_cmd_id` when available

### Exit-only fields

- `exit_code` or `signal`
- `duration_ms`

### Optional fields

- `env`
- `exe`

### Truncation caps

- max events per execution: 10,000 by default
- max env value length: 4 KB per value by default

Protocol-level truncation reporting is owned by
[protocol.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/internals/trace/protocol.md).

## 3. Builtin and preexec trace posture

When `SUBSTRATE_ENABLE_PREEXEC=1`:
- `builtin_command` records in canonical trace must omit command bodies
- canonical records must include:
  - `command_omitted: true`
  - `parent_cmd_id` when available

When `SUBSTRATE_PREEXEC_RAW_LOG` is set:
- raw debug-only records are written as `builtin_command_raw`
- raw records must include `may_contain_secrets: true`

## 4. Router-derived and toolbox event families

Phase 8 additive correlation extends canonical trace beyond shell and world-service spans.

### Router-derived event examples

- `workflow_router_rule_match`
- `workflow_router_request_enqueued`
- `workflow_router_request_denied`
- `workflow_router_request_pending_approval`
- `workflow_router_action_enqueued`
- `workflow_router_action_executed`
- `workflow_router_cursor_gap_detected`

Required join keys for router-derived rows:
- `request_id`
- `idempotency_key`
- `workspace_id`
- one explicit cause reference:
  - `source_span_id`, and/or
  - `source_cmd_id`
- `rule_id`

### Toolbox tool-call event examples

- `toolbox_tool_call_start`
- `toolbox_tool_call_complete`

Required fields:
- `component: "agent-toolbox"`
- `session_id`
- `orchestration_session_id`
- `run_id`
- `agent_id`
- `backend_id`
- `tool_call_id`
- `tool_name`

Completion rows must also include:
- `outcome`
- `duration_ms`

Safe-by-default payload rule:
- canonical toolbox records must not embed full request args or full tool response bodies
- use omission markers such as `args_omitted: true` and `result_omitted: true`
