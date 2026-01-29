# Requirements Traceability — World FS Granular Allow/Deny (V2) + Strict Deny

This document maps every MUST in the authoritative spec pack to:
- an explicit implementation task (from `docs/project_management/next/world-fs-granular-allow-deny/tasks.json`), and
- an explicit validation step (test or manual verification).

Authoritative spec pack (no drift allowed):
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- `docs/project_management/next/world-fs-granular-allow-deny/contract.md`
- `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
- `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
- `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`
- `docs/project_management/next/world-fs-granular-allow-deny/SECURITY.md`
- `docs/project_management/next/world-fs-granular-allow-deny/decision_register.md`
- `docs/project_management/next/world-fs-granular-allow-deny/integration_map.md`
- `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`

## Canonical requirements (stable IDs)

| Req ID | Level | Canonical requirement (summary) | Implemented by | Validated by |
|---|---|---|---|---|
| R-001 | MUST | Policy schema is breaking: legacy keys are hard errors; no silent ignore of invalid patterns | `C0-code` | `C0-test` |
| R-002 | MUST | Deny lists and enforcement mode are supported only in `world_fs.isolation=full`; workspace usage is a hard error | `C0-code` | `C0-test` |
| R-003 | MUST | World-agent rejects `PolicySnapshotV1`; only `PolicySnapshotV2` allowed (schema_version=2) | `C0-code` | `C0-test`, `C0-integ` |
| R-004 | MUST | Deny masks are applied inside the per-command mount namespace before user code executes | `C0-code` | `C0-integ`; manual playbook Case 1 |
| R-005 | MUST | `enforcement=strict` makes deny rules a hard security boundary (workload cannot undo masks via mount/umount/remount) | `C0-code` | `C0-integ`; manual playbook Case 2 |
| R-006 | MUST | `discover` is an optional dimension; if omitted it mirrors `read` (no ambiguous defaults) | `C0-code` | `C0-test`, `C0-integ`; manual playbook Case 3 |
| R-007 | MUST | Filename-glob denies (`**/*.pem`) are enforced as “snapshot at exec start” (no overpromised dynamic guarantees) | `C0-code` | `C0-integ`; manual playbook Case 4 |

