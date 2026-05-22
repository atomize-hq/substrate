# world_process_exec_tracing_parity — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:

- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/active/world_process_exec_tracing_parity`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

## Required spec documents (authoritative)

- `docs/project_management/packs/active/world_process_exec_tracing_parity/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/packs/active/world_process_exec_tracing_parity/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/packs/active/world_process_exec_tracing_parity/plan.md` — execution runbook + sequencing overview
- `docs/project_management/packs/active/world_process_exec_tracing_parity/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/packs/active/world_process_exec_tracing_parity/session_log.md` — planning + execution audit log
- `docs/project_management/packs/active/world_process_exec_tracing_parity/ci_checkpoint_plan.md` — bounded CI checkpoint plan (cross-platform automation packs)
- `docs/project_management/packs/active/world_process_exec_tracing_parity/quality_gate_report.md` — planning quality gate outcome

- Slice specs:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP0-spec.md` — span correctness + joinability + preexec safety
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP1-spec.md` — world-service API + types + shell persistence for process events
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP2-spec.md` — Linux in-world process capture + provisioning/caps/truncation
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP3-spec.md` — redaction hardening + argv/env capture

- Contracts:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md` — operator-facing contract summary + invariants
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md` — host↔world-service API and transport payloads
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md` — trace event families + fields + redaction/caps schema
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/SECURITY.md` — security posture, redaction, fail/degrade rules
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md` — A/B decisions with explicit selection
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md` — manual validation cases (authoritative)
  - `docs/internals/env/inventory.md` — env var inventory entries (SUBSTRATE_ENABLE_PREEXEC, SUBSTRATE_PREEXEC_RAW_LOG)

- Smoke scripts:
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/macos-smoke.sh`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/windows-smoke.ps1`

## Coverage matrix (surface → authoritative doc)

| Surface                       | Authoritative doc                                                                                   | What must be explicitly defined                                                   |
| ----------------------------- | --------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Operator-facing contract      | `docs/project_management/packs/active/world_process_exec_tracing_parity/contract.md`                | new event families, invariants, defaults, “safe-by-default” stance                |
| Trace schema (spans + events) | `docs/project_management/packs/active/world_process_exec_tracing_parity/SCHEMA.md`                  | event types, required/optional fields, join keys, caps/truncation                 |
| World-agent API contract      | `docs/project_management/packs/active/world_process_exec_tracing_parity/PROTOCOL.md`                | request/response/frames, error model, diagnostics, ordering                       |
| Security posture              | `docs/project_management/packs/active/world_process_exec_tracing_parity/SECURITY.md`                | redaction requirements, omit/raw rules, degrade vs fail-closed                    |
| Env var semantics (preexec)   | `docs/project_management/packs/active/world_process_exec_tracing_parity/SECURITY.md`                | SUBSTRATE_ENABLE_PREEXEC, SUBSTRATE_PREEXEC_RAW_LOG, defaults, safety constraints |
| Env var inventory entries     | `docs/internals/env/inventory.md`                                                                   | inventory rows updated for env vars introduced/modified by this feature           |
| Decision points               | `docs/project_management/packs/active/world_process_exec_tracing_parity/decision_register.md`       | A/B options, explicit selection, rationale, follow-ups                            |
| Slice definitions             | `docs/project_management/packs/active/world_process_exec_tracing_parity/WPEP*-spec.md`              | exact behaviors and acceptance criteria per slice                                 |
| Manual validation             | `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md` | deterministic commands + expected outcomes                                        |
| Smoke validation              | `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/*`                    | automated validation commands; expected pass/fail                                 |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:

- Inputs (all) and precedence order (if multiple inputs exist)
- Defaults (all) and absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes and protocol errors) and failure posture
- Ordering/atomicity/concurrency rules (when applicable)
- Security invariants and redaction constraints (when applicable)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)
