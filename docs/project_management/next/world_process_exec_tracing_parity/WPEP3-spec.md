# WPEP3 — spec — redaction hardening + argv/env capture for process events

## Scope (explicit)
- Implement shared redaction helpers suitable for process-granularity capture:
  - argv redaction handles flag-paired secrets (`--flag value`) and equals form (`--flag=value`).
  - env redaction handles URL credentials, headers, and common token patterns.
- Upgrade Linux-backed capture (native Linux + macOS Lima guest) to emit:
  - `argv` (redacted array) on `world_process_*` records (do not emit `argv_omitted` once argv is available),
  - `env` allowlist-only capture (optional field; keys outside allowlist omitted),
  - per-value env cap (default 4KB/value) with truncation behavior defined in PROTOCOL/SCHEMA.
- Ensure transport stability on macOS (VSock/SSH UDS/TCP):
  - payload semantics identical across transports,
  - payload caps prevent oversized responses/frames.

## Acceptance (explicit)
- On Linux-backed backends (native Linux and macOS Lima):
  - at least one `world_process_start` record includes `argv` as an array,
  - no `world_process_*` record includes `argv_omitted`.
- Env capture is allowlist-only:
  - env keys not in the allowlist are omitted.
- Caps:
  - if caps trigger truncation, diagnostics include `process_events_status: "truncated"` and `process_events_dropped: <n>`.

## Out of scope (explicit)
- Windows/WSL process capture.
- Per-event streaming of process telemetry over `/v1/stream` (follow-on optimization).

