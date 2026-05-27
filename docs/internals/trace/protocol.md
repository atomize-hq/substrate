# Trace Protocol (Internal)

This document is the stable internal protocol reference for process-event payloads returned by
world-service and persisted by the host shell.

## Scope

- HTTP `POST /v1/execute`
- WebSocket `GET /v1/stream`
  - v1 uses exit-frame batching only for process-event payloads

Platform posture:
- Linux: supported when process capture is enabled
- macOS Lima: supported because world-service runs inside a Linux guest
- Windows: explicit degrade-only posture for this feature

## 1. Process event payload

`process_events` is a list of normalized process lifecycle records.

### Base fields

- `event_type`: `world_process_start` or `world_process_exit`
- `ts`
- `ts_unix_ns`
- `session_id`
- `world_id`
- `pid`
- `ppid`
- `cwd`
- `parent_span`
- `parent_cmd_id` when available

### Argv capture

Exactly one of the following must be present:
- `argv`
- `argv_omitted: true`

### Start-only fields

- `exe` may be present as best effort

### Exit-only fields

For `world_process_exit`, exactly one of the following must be present:
- `exit_code`
- `signal`

`duration_ms` is also required on exit records.

### Optional env capture

- `env` is optional
- env keys are allowlist-only and may be redacted

### Ordering

`process_events` must be sorted by:
1. `ts_unix_ns` ascending
2. `pid` ascending
3. `event_type`, with `world_process_start` before `world_process_exit`

## 2. Diagnostics and degrade posture

Responses and frames that carry `process_events` must include:
- `process_events_status`: `ok | unavailable | truncated | error`
- `process_events_reason` when status is not `ok`

Optional summaries:
- when status is `truncated`:
  - `process_events_dropped`
  - `process_events_max`
- when status is `unavailable`:
  - `process_events_backend`
- when status is `error`:
  - `process_events_error`

Stable non-ok reason examples:
- `not_supported_platform`
- `backend_disabled`
- `ptrace_not_permitted`
- `capture_overflow`
- `internal_error`

## 3. HTTP execute response additions

`ExecuteResponse` must include:
- `process_events`
- `process_events_status`
- `process_events_reason` when status is not `ok`

Rules:
- on Linux-backed supported backends, status must be `ok` or `truncated`
- on Windows for this feature, status must be `unavailable` with reason `not_supported_platform`
- `process_events` may be omitted only when status is `error` and no safe records can be emitted

## 4. WebSocket stream exit-frame additions

For the v1 stream protocol:
- `process_events*` fields appear only on the exit frame
- per-event streaming of process telemetry is not part of v1

Exit frames must include:
- `process_events`
- `process_events_status`
- `process_events_reason` when status is not `ok`

## 5. Host persistence contract

The host shell persists each process event as a canonical JSONL record with:
- `component: "world-service"`
- `event_type` copied from the process event payload
- required process-event fields preserved after host normalization

The host must not synthesize raw argv or env values that were omitted or redacted by world-service.

For each world execution, the host shell also persists protocol diagnostics on the corresponding
shell completion record:
- `process_events_status`
- `process_events_reason` when status is not `ok`
- `process_events_dropped` when status is `truncated`
