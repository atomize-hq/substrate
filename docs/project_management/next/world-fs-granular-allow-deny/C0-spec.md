# C0 — spec — world-fs-granular-allow-deny

## Scope (explicit)
- Implements ADR-0018 for Linux full isolation only (`world_fs.isolation=full`).
- This slice is breaking (no backwards compatibility):
  - `PolicySnapshotV1` is rejected.
  - Legacy policy keys are hard errors.

## Authoritative spec pack (no drift allowed)
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- `docs/project_management/next/world-fs-granular-allow-deny/contract.md`
- `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
- `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
- `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`
- `docs/project_management/next/world-fs-granular-allow-deny/SECURITY.md`
- `docs/project_management/next/world-fs-granular-allow-deny/decision_register.md`
- `docs/project_management/next/world-fs-granular-allow-deny/requirements_traceability.md`
- `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`

## Acceptance (explicit)
- All requirements R-001 through R-022 in `requirements_traceability.md` are implemented and validated as specified.
- Manual testing playbook Cases 1-6 pass on Linux full isolation.

## Out of scope (explicit)
- macOS support.
- Windows support (including WSL).
- `world_fs.isolation=workspace` support for deny/enforcement.
