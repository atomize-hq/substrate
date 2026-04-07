# WPEP2 — spec — Linux in-world process capture (ptrace) + caps/truncation

## Scope (explicit)
- Land Linux process tree capture for in-world executions (ptrace-based).
  - Supported backends: native Linux and macOS Lima guest (world-agent runs on Linux; `docs/WORLD.md`).
- Provisioning requirement (Linux-backed backends):
  - The `substrate-world-agent` systemd service MUST include `CAP_SYS_PTRACE` in `CapabilityBoundingSet` and `AmbientCapabilities`.
  - If the capability is missing (or ptrace is blocked by kernel policy), world-agent MUST degrade explicitly:
    - `process_events_status: "unavailable"`
    - `process_events_reason: "ptrace_not_permitted"`
- World-agent returns a capped `process_events` list for:
  - HTTP `/v1/execute` (response body), and
  - WebSocket `/v1/stream` (Exit frame; batched-on-exit).
- Enforce caps/truncation deterministically:
  - max events per execution (default 10,000),
  - truncation diagnostics (`process_events_status: "truncated"`, `process_events_dropped`).
- Privacy posture for this slice:
  - `argv` MAY be omitted, but only by emitting `argv_omitted: true` on each process event.

## Acceptance (explicit)
- For a deterministic child-spawning world command on Linux:
  - `process_events_status` is `"ok"` or `"truncated"`.
  - canonical trace includes at least one `world_process_start` and one `world_process_exit`.
  - `world_process_*` records are joinable via `parent_span` to the originating host span id.
- Truncation:
  - when the cap is exceeded, diagnostics include `process_events_status: "truncated"` and `process_events_dropped: <n>`.
 - `argv` omission is explicit:
   - every emitted `world_process_*` record contains `argv_omitted: true` (until WPEP3 adds redacted argv capture).

## Out of scope (explicit)
- Per-event streaming of process telemetry over `/v1/stream` (follow-on optimization).
- Windows capture mechanisms.
- Emitting redacted `argv` and allowlisted `env` (WPEP3).
