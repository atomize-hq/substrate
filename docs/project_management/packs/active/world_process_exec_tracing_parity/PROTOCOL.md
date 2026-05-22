# Protocol — World process exec/exit telemetry (ADR-0028) (Authoritative)

This document defines the authoritative host↔world-service protocol surfaces introduced by ADR-0028.

## Scope
- Endpoints:
  - HTTP `POST /v1/execute`
  - WebSocket `GET /v1/stream` (Exit frame only for v1; no per-event streaming)
- Platforms:
  - Linux (native): process capture supported (ptrace-based; backend-dependent).
  - macOS (Lima): process capture supported because world-service runs inside a Linux guest (`docs/WORLD.md`).
    - Transport differences (VSock vs SSH UDS/TCP) MUST NOT change payload semantics; caps/truncation MUST bound payload size for all transports.
  - Windows: process capture is out of scope for this feature; responses MUST degrade explicitly (never omit silently).

## 1) Data model: process event payload

`process_events` is a list of normalized process lifecycle records emitted by world-service and persisted by the host shell.

### 1.1 ProcessEvent (base fields; required)
- `event_type`: `"world_process_start" | "world_process_exit"`
- `ts`: RFC3339 timestamp (UTC)
- `ts_unix_ns`: integer nanoseconds since Unix epoch (UTC)
- `session_id`: string
- `world_id`: string
- `pid`: integer
- `ppid`: integer (0 if unknown)
- `cwd`: string (redacted/path-sanitized if required by policy)
- Correlation:
  - `parent_span`: string host span id (`"spn_..."`)
  - `parent_cmd_id`: string host command id (`"cmd_..."`) when available, else omitted

### 1.1.1 argv capture (required by the full feature; may be explicitly omitted)
- `argv`: array of strings (redacted)
- `argv_omitted`: boolean

Rules:
- Exactly one of `argv` or `argv_omitted` MUST be present.
- If `argv_omitted` is present, it MUST be `true`.

### 1.2 Start-only fields (optional)
- `exe`: string (best-effort; may be omitted)

### 1.3 Exit-only fields (required for `world_process_exit`)
- `exit_code`: integer when a normal exit is observed
- `signal`: integer when a signal termination is observed
- `duration_ms`: integer

Exit semantics:
- Exactly one of `exit_code` or `signal` MUST be present on `world_process_exit`.

### 1.4 Env (optional; allowlist-only by default)
- `env`: object map of string→string (redacted)

Env rules:
- World-agent MUST only include env keys explicitly allowlisted for process telemetry.
- Unknown env keys MUST be omitted (not empty-stringed).

### 1.5 Ordering and determinism
- World-agent MUST return `process_events` sorted by:
  1) `ts_unix_ns` ascending, then
  2) `pid` ascending, then
  3) `event_type` (`world_process_start` before `world_process_exit`).

## 2) Diagnostics: explicit degrade vs ok vs truncated vs error

World-agent responses/frames that carry `process_events` MUST include:
- `process_events_status`: `"ok" | "unavailable" | "truncated" | "error"`
- `process_events_reason`: string (required when status != `"ok"`)

Optional summaries (status-dependent):
- When `process_events_status="truncated"`:
  - `process_events_dropped`: integer count of events not included in `process_events`
  - `process_events_max`: integer cap that triggered truncation
- When `process_events_status="unavailable"`:
  - `process_events_backend`: string (optional; e.g. `"world-linux"`), when known
- When `process_events_status="error"`:
  - `process_events_error`: string human-readable diagnostic

Reason codes (non-exhaustive; stable strings):
- `"not_supported_platform"`
- `"backend_disabled"`
- `"ptrace_not_permitted"`
- `"capture_overflow"`
- `"internal_error"`

## 3) HTTP `POST /v1/execute` (response additions)

`ExecuteResponse` MUST include:
- `process_events` (array; MAY be empty)
- `process_events_status`
- `process_events_reason` when status != `"ok"`

Rules:
- On Linux-backed backends where capture is supported and enabled (native Linux or macOS Lima guest), status MUST be `"ok"` or `"truncated"`.
- On Windows for this feature, status MUST be `"unavailable"` with reason `"not_supported_platform"`.
- `process_events` MUST be omitted only if `process_events_status="error"` and the system cannot safely emit any records.

## 4) WebSocket `GET /v1/stream` (Exit frame additions; v1 behavior)

For the v1 stream protocol:
- World-agent MUST include `process_events*` fields only on the Exit frame (batched-on-exit).
- World-agent MUST NOT stream per-event process telemetry frames in v1.

Exit frame MUST include:
- `process_events` (array; MAY be empty)
- `process_events_status`
- `process_events_reason` when status != `"ok"`

## 5) Host persistence contract

The host shell MUST persist each ProcessEvent as a JSONL record in canonical trace:
- `component: "world-service"`
- `event_type` copied from ProcessEvent
- all required ProcessEvent fields preserved after host-side normalization (no additional secrets introduced)

Host MUST NOT synthesize raw argv/env:
- if `argv`/`env` are absent or redacted by world-service, host persists them as-is.

Additionally, the host shell MUST persist the diagnostics summary for each world execution on the corresponding shell completion record:
- `component: "shell"`, `event_type: "command_complete"` MUST include:
  - `process_events_status`
  - `process_events_reason` when status != `"ok"`
  - `process_events_dropped` when status == `"truncated"`
