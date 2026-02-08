# world_process_exec_tracing_parity — contract

This document is the operator-facing contract summary for ADR-0028.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Schema: `docs/project_management/next/world_process_exec_tracing_parity/SCHEMA.md`
- Protocol: `docs/project_management/next/world_process_exec_tracing_parity/PROTOCOL.md`
- Security posture: `docs/project_management/next/world_process_exec_tracing_parity/SECURITY.md`

## What changes
- Canonical trace (`~/.substrate/trace.jsonl`) gains:
  - joinability improvements across shell/shim/replay records, and
  - a new `world_process_*` event family for subprocess exec/exit telemetry (Linux backend; degrade elsewhere).

## Non-negotiable invariants
- Canonical trace is safe-by-default:
  - no secrets in argv/env capture; apply shared redaction rules and data minimization.
  - preexec/builtin tracing MUST NOT write raw command bodies into canonical trace.
- Deny is unambiguous from completion records:
  - completion spans MUST reflect denied vs executed clearly (`outcome: "denied"` on deny).
- Degrade is explicit:
  - if in-world process capture is unavailable, world-agent responses MUST include deterministic diagnostics; omission must not be silent.

