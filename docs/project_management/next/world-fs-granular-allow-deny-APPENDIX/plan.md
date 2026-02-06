# world-fs-granular-allow-deny-appendix — plan

## Scope
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX`
- Orchestration branch: `feat/world-fs-granular-allow-deny-appendix`
- ADR (Appendix A + B are authoritative inputs):
  - `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Spec ownership map: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/spec_manifest.md`
- Impact map: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/impact_map.md`
- CI checkpoint plan: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ci_checkpoint_plan.md`

## Preconditions (explicit)
- `feat/world-fs-granular-allow-deny` is merged and released as the baseline implementation for world-fs allow/deny.
- Appendix A + B changes are implemented as a breaking policy surface update (no backwards compatibility).

## Goal
- Implement Appendix A + B of ADR-0018 as an execution-ready, cross-platform-safe change set:
  - V3 policy patch schema rename for `world_fs` (intent-driven keys).
  - Routing fail-closed semantics (`world_fs.fail_closed.routing`) and exported state env var rename.
  - Policy-level caging requirement (`world_fs.caged_required`) with deterministic cage root.
  - REPL exit transparency and `repl.exit_cwd` semantics.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.
- No backwards compatibility is provided for replaced policy keys and snapshot schema.
- Cross-platform CI parity is required (linux/macos/windows) to prevent compile regressions from landing.

## Deliverables (authoritative)
- Contract: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md`
- Schema: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SCHEMA.md`
- Protocol: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/PROTOCOL.md`
- Env var contract: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/ENV.md`
- Security invariants: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/SECURITY.md`
- Requirements mapping: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/requirements_traceability.md`
- Manual testing playbook: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/manual_testing_playbook.md`

## Slices
- WFGADAX0: policy patch schema V3 (rename + defaults + validation) + broker wiring
- WFGADAX1: routing fail-closed + exported state env var rename + failure taxonomy wiring
- WFGADAX2: policy-level caging requirement + cage root derivation + config compatibility
- WFGADAX3: REPL exit transparency + `repl.exit_cwd` + shell integration hook contract

