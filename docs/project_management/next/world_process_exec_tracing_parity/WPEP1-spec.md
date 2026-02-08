# WPEP1 — spec — world-agent API + persistence plumbing for process events

## Scope (explicit)
- Define and implement the host↔world-agent transport shape for process telemetry diagnostics:
  - `process_events_status`
  - `process_events_reason`
  - `process_events_dropped` (when truncated)
- Persist diagnostics on the shell completion record for world executions:
  - `component: "shell"`, `event_type: "command_complete"` includes `process_events_status` and `process_events_reason` (when status != "ok").
- Wire the shell persistence path for `process_events` when present:
  - shell appends `world_process_start`/`world_process_exit` records to canonical trace.

## Acceptance (explicit)
- For a world execution, shell trace includes a completion summary with `process_events_status`.
- On Windows (out of scope for capture in this feature):
  - `process_events_status: "unavailable"`
  - `process_events_reason: "not_supported_platform"`
- On Linux-backed backends (native Linux and macOS Lima) before capture exists/enabled:
  - `process_events_status: "unavailable"`
  - `process_events_reason: "backend_disabled"`

## Out of scope (explicit)
- Capturing the in-world process tree (ptrace or equivalent).
- Any change that makes process capture mandatory for execution.
