# world-fs-granular-allow-deny-appendix — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX`
- ADR(s):
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md` (Appendix A + B)

## Required spec documents (authoritative)

- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md` — spec selection + ownership map (this file)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/impact_map.md` — touch set + cascading implications + cross-queue conflicts
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/plan.md` — execution runbook + sequencing overview
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/tasks.json` — triad task graph + acceptance criteria
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/session_log.md` — planning + execution audit log
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md` — bounded CI checkpoint plan (cross-platform automation packs)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX0-spec.md` — WFGADAX0 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX1-spec.md` — WFGADAX1 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX2-spec.md` — WFGADAX2 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/WFGADAX3-spec.md` — WFGADAX3 slice definition and acceptance routing
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` — operator-facing contract (inputs, invariants, hard errors)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` — policy patch and snapshot schema (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md` — host↔world-agent protocol requirements (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md` — environment variable contract (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SECURITY.md` — security posture and fail-closed invariants (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` — A/B decisions with explicit selection
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/requirements_traceability.md` — MUST/MUST NOT mapping to tasks and validation steps
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` — manual validation cases (authoritative)
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/execution_preflight_report.md` — execution-time preflight evidence log
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/quality_gate_report.md` — planning quality gate outcome
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/kickoff_prompts/F0-exec-preflight.md` — execution preflight kickoff
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/kickoff_prompts/FZ-feature-cleanup.md` — feature cleanup kickoff

## Coverage matrix (surface → authoritative doc)

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Operator-facing contract | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` | supported keys, defaults, hard errors, exit codes, deterministic failure behavior |
| Effective policy display (`substrate policy show`) | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` | Appendix A.6 rendering rules for `discover`/`read`/`write` (including explicit `deny_list: []` when empty and explicit `discover` when defaulted) |
| Config key `repl.exit_cwd` | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` | values, default, fallback behavior, required note line contract |
| Policy/config compatibility constraints | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` | `world_fs.caged_required=true` compatibility with `world.caged` and `world.anchor_mode`; `world_fs.fail_closed.routing=true` compatibility with effective world disable |
| Shell integration hook contract (`repl.exit_cwd=last_world`) | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` | machine-parseable note line contract and application rules for shell wrappers |
| Routing fallback warning contract (stderr substrings) | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` | Appendix B.2.1 trigger condition and required stderr substrings when routing falls back to host while `world_fs.host_visible=false` was requested |
| Policy patch schema (YAML/patch) | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` | complete V3 schema, constraints, validation rules, no-compat stance |
| Policy snapshot schema | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md` | versioning, required/forbidden fields, canonicalization rules |
| Host↔world-agent protocol | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md` | request/response shapes, error model, fail-closed rules for invalid snapshots |
| Environment variables | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md` | names, types, output-only rules, parse/validation failure posture |
| Security posture | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SECURITY.md` | fail-closed routing, deny enforcement posture semantics, caging boundary invariants |
| Decision points | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/decision_register.md` | A/B options, explicit selection, rationale |
| Requirements mapping | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/requirements_traceability.md` | stable requirement IDs and validation mapping |
| Manual validation | `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md` | deterministic cases and expected outcomes |
