# Plan — World FS Granular Allow/Deny (V2) + Strict Deny

This planning pack supports:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

## Scope
- Linux full isolation only (`world_fs.isolation=full`).
- Breaking schema changes (no backwards compatibility).

## Deliverables
- ADR (accepted): `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- Schema (authoritative): `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
- Protocol (authoritative): `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
- Env contract (authoritative): `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`
- Integration map: `docs/project_management/next/world-fs-granular-allow-deny/integration_map.md`
- Manual testing playbook: `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`

## Implementation phases (high-level)
1) Schema + snapshot plumbing
   - Broker policy schema v2 (no compat)
   - `agent-api-types` snapshot v2 + request updates
   - Shell emits v2 snapshots (non-PTY + persistent session)
2) World-agent enforcement
   - Helper applies deny masks + strict lockdown before `exec sh -c`
   - Landlock allowlists updated for discover/read split
3) Validation + ergonomics
   - Unit/integration tests
   - Manual playbook + smoke scripts

