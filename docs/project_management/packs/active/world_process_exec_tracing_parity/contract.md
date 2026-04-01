# world_process_exec_tracing_parity — contract

This document is the operator-facing contract summary for ADR-0028.

Authoritative inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- Schema: `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`
- Protocol: `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`
- Security posture: `docs/project_management/packs/active/world_process_exec_tracing_parity/SECURITY.md`

## What changes
- Canonical trace (`~/.substrate/trace.jsonl`) gains:
  - joinability improvements across shell/shim/replay records, and
  - a planned `world_process_*` event family for subprocess exec/exit telemetry (Linux backend; degrade elsewhere when implemented).
- Phase 8 additive note (cross-feature): ADR-0028 also defines a shared correlation vocabulary and reserves/adds additional trace families (router/toolbox/workflow). This planning pack implements the landed joinability and preexec posture, while the `world_process_*` family remains the planned target for the later world-process slices; router-derived families are owned by ADR-0029 and documented in the router decision register.

## Non-negotiable invariants
- Canonical trace is safe-by-default:
  - no secrets in argv/env capture; apply shared redaction rules and data minimization.
  - preexec/builtin tracing MUST NOT write raw command bodies into canonical trace.
- Deny is unambiguous from completion records:
  - completion spans MUST reflect denied vs executed clearly (`outcome: "denied"` on deny).
- Degrade is explicit:
  - if in-world process capture is unavailable, world-agent responses MUST include deterministic diagnostics; omission must not be silent.
