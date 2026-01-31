# world-fs-granular-allow-deny — plan

## Scope
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny`
- Orchestration branch: `feat/world-fs-granular-allow-deny`
- Spec ownership map: `docs/project_management/next/world-fs-granular-allow-deny/spec_manifest.md`
- Impact map: `docs/project_management/next/world-fs-granular-allow-deny/impact_map.md`

- Linux full isolation only (`world_fs.isolation=full`).
- Breaking schema changes (no backwards compatibility).

## Goal
- Implement ADR-0018 for Linux full isolation only with no backwards compatibility.

## Guardrails (non-negotiable)
- Specs are the single source of truth.
- Planning Pack docs are edited only on the orchestration branch.
- Do not edit planning docs inside the worktree.

## Deliverables (authoritative)
- ADR: `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Contract: `docs/project_management/next/world-fs-granular-allow-deny/contract.md`
- Schema: `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
- Protocol: `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
- Env var contract: `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`
- Security invariants: `docs/project_management/next/world-fs-granular-allow-deny/SECURITY.md`
- Requirements mapping: `docs/project_management/next/world-fs-granular-allow-deny/requirements_traceability.md`
- Manual testing playbook: `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`

## Triads
- C0: V2 policy snapshot + strict deny (code/test/integ)
