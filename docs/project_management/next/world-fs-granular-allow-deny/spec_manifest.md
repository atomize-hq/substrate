# world-fs-granular-allow-deny — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny`
- ADR(s):
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Required spec documents (authoritative)

- `docs/project_management/next/world-fs-granular-allow-deny/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/next/world-fs-granular-allow-deny/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/next/world-fs-granular-allow-deny/plan.md` — execution runbook + sequencing overview
- `docs/project_management/next/world-fs-granular-allow-deny/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/next/world-fs-granular-allow-deny/session_log.md` — planning + execution audit log
- `docs/project_management/next/world-fs-granular-allow-deny/WFGAD0-spec.md` — WFGAD0 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny/WFGAD1-spec.md` — WFGAD1 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny/WFGAD2-spec.md` — WFGAD2 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny/WFGAD3-spec.md` — WFGAD3 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny/WFGAD4-spec.md` — WFGAD4 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny/WFGAD5-spec.md` — WFGAD5 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny/contract.md` — operator-facing contract (inputs, invariants, hard errors)
- `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md` — policy and snapshot schema (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md` — host↔world-agent protocol requirements (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny/ENV.md` — environment variable contract for helper invocation (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny/SECURITY.md` — security posture and fail-closed invariants (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny/decision_register.md` — A/B decisions with explicit selection
- `docs/project_management/next/world-fs-granular-allow-deny/requirements_traceability.md` — MUST/MUST NOT mapping to tasks and validation steps
- `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md` — manual validation cases (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny/quality_gate_report.md` — planning quality gate outcome
- `docs/project_management/next/world-fs-granular-allow-deny/ci_checkpoint_plan.md` — bounded CI checkpoint plan (cross-platform automation packs)

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Operator-facing contract | `docs/project_management/next/world-fs-granular-allow-deny/contract.md` | supported modes, hard errors, precedence rules, deterministic failure behavior |
| Policy schema (YAML/patch) | `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md` | complete schema, constraints, validation rules, no-compat stance |
| Policy snapshot schema | `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md` | versioning, required/forbidden fields, canonicalization rules |
| Host↔world-agent protocol | `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md` | request/response shapes, error model, fail-closed rules for invalid snapshots |
| Environment variables | `docs/project_management/next/world-fs-granular-allow-deny/ENV.md` | names, types, required/optional behavior, parse/validation failure posture |
| Security posture (strict deny) | `docs/project_management/next/world-fs-granular-allow-deny/SECURITY.md` | strict prerequisites, bypass prevention requirements, fail-closed conditions |
| Decision points | `docs/project_management/next/world-fs-granular-allow-deny/decision_register.md` | A/B options, explicit selection, rationale |
| Requirements mapping | `docs/project_management/next/world-fs-granular-allow-deny/requirements_traceability.md` | stable requirement IDs and validation mapping |
| Manual validation | `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md` | deterministic cases and expected outcomes |

## Determinism checklist (must be satisfied before quality gate)

For every selected spec document, confirm it explicitly defines:
- Inputs (all) and precedence order (if multiple inputs exist)
- Defaults (all) and absence semantics
- Data model (types/constraints) for every serialized boundary
- Error model (exit codes and protocol errors) and failure posture
- Ordering/atomicity/concurrency rules (when applicable)
- Security invariants and redaction constraints (when applicable)
- Platform guarantees (Linux/macOS/Windows/WSL as applicable)
