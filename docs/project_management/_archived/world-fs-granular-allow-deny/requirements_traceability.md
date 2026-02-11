# Requirements Traceability — World FS Granular Allow/Deny (V2) + Strict Deny

This document maps every MUST / MUST NOT in the authoritative spec pack to:
- an explicit implementation task (from `docs/project_management/_archived/world-fs-granular-allow-deny/tasks.json`), and
- an explicit validation step (test or manual verification).

Authoritative spec pack (no drift allowed):
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/contract.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/PROTOCOL.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/ENV.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/SECURITY.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/decision_register.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/impact_map.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/manual_testing_playbook.md`

## Canonical requirements (stable IDs)

| Req ID | Level | Canonical requirement (summary) | Implemented by | Validated by |
|---|---|---|---|---|
| R-001 | MUST | Policy schema is breaking: legacy keys are hard errors; no silent ignore of invalid patterns | `WFGAD0-code` | `WFGAD0-test` |
| R-002 | MUST | Deny lists and enforcement mode are supported only in `world_fs.isolation=full`; workspace usage is a hard error | `WFGAD0-code` | `WFGAD0-test` |
| R-003 | MUST | World-agent rejects `PolicySnapshotV1`; only `PolicySnapshotV2` allowed (schema_version=2) | `WFGAD1-code` | `WFGAD1-test`, `WFGAD1-integ` |
| R-004 | MUST | Pattern normalization and syntax are deterministic (no engine-dependent behavior); `allow_list` forbids wildcards; `deny_list` supports only `*`/`**`; unsupported metacharacters are hard errors | `WFGAD0-code` | `WFGAD0-test` |
| R-005 | MUST | `allow_list` is required and non-empty for any configured dimension | `WFGAD0-code` | `WFGAD0-test` |
| R-006 | MUST | Wildcard denies are enforced as “snapshot at exec start” (no overpromised dynamic guarantees) | `WFGAD3-code` | `WFGAD3-integ`; manual playbook Case 4 |
| R-007 | MUST | `discover` is an optional dimension; if omitted it mirrors `read` (no ambiguous defaults) | `WFGAD4-code` | `WFGAD4-test`, `WFGAD4-integ`; manual playbook Case 3 |
| R-008 | MUST | Deny masks are applied inside the per-command mount namespace before user code executes | `WFGAD3-code` | `WFGAD3-integ`; manual playbook Case 1 |
| R-009 | MUST | `enforcement=strict` makes deny rules a hard security boundary (workload cannot undo masks via mount/umount/remount) | `WFGAD5-code` | `WFGAD5-integ`; manual playbook Case 2 |
| R-010 | MUST | Deny masks apply to all nameable in-world project views in full isolation (both `/project/...` and `$SUBSTRATE_MOUNT_PROJECT_DIR/...`) | `WFGAD3-code` | `WFGAD3-integ`; manual playbook Case 1 |
| R-011 | MUST | World-agent validates `PolicySnapshotV2` strictly: wrong schema_version, unknown fields, invalid key combinations, and invalid patterns are rejected | `WFGAD1-code` | `WFGAD1-test`, `WFGAD1-integ` |
| R-012 | MUST | Protocol surfaces carry V2 only: HTTP `/v1/execute` and WS `start_session` reject invalid snapshots (fail closed). WS `start_session` failures return a fatal `error` frame and close the connection. | `WFGAD1-code` | `WFGAD1-test`, `WFGAD1-integ` |
| R-013 | MUST | Helper invocation is mandatory whenever helper-side enforcement is required (deny-only configs must still execute the helper) | `WFGAD2-code` | `WFGAD2-test`, `WFGAD2-integ` |
| R-014 | MUST | Strict-mode lockdown is explicit: helper drops mount-related capability authority and installs seccomp denying mount-family syscalls (kernel-missing syscalls are N/A; otherwise must be denied) | `WFGAD5-code` | `WFGAD5-integ` |
| R-015 | MUST | Strict-mode lockdown applies to the entire process tree (workload + descendants) | `WFGAD5-code` | `WFGAD5-integ` |
| R-016 | MUST | “Hard error” is defined: command must not execute; failure occurs during validation (exit code 2 on host; HTTP 400 with `{ \"error\": \"...\" }`; WS fatal `error` frame + close) | `WFGAD0-code` | `WFGAD0-test` |
| R-017 | MUST | No compatibility / lockstep: V2-emitting shell and V2-only world-agent must be upgraded together; V1↔V2 mismatches fail closed | `WFGAD1-code` | `WFGAD1-test`, `WFGAD1-integ` |
| R-018 | MUST | `world_fs.enforcement` is tied to denies: it is present iff any deny_list is non-empty (reject `enforcement` when all deny_list values are empty) | `WFGAD0-code` | `WFGAD0-test` |
| R-019 | MUST | Denies require a security boundary: if any deny_list is non-empty, `world_fs.require_world` MUST be true | `WFGAD0-code` | `WFGAD0-test` |
| R-020 | MUST | Wildcard deny snapshot scanning does not follow symlinks (deterministic behavior; no traversal outside project) | `WFGAD3-code` | `WFGAD3-test`, `WFGAD3-integ` |
| R-021 | MUST | Denied operation error semantics are deterministic: discover/read denies return `EACCES`; write denies return `EROFS`; strict syscall blocks return `EPERM` without killing the process | `WFGAD3-code`, `WFGAD5-code` | `WFGAD3-integ`, `WFGAD5-integ`; manual playbook Case 5 |
| R-022 | MUST | Enforcement plan env var schema is strict and validated: base64 JSON must match v1 shape; unknown fields rejected; plan represents effective (defaulted) denies; helper fails closed (exit 4) on parse/validation failure | `WFGAD2-code` | `WFGAD2-test`, `WFGAD2-integ` |

## Coverage notes (to avoid drift)
- `contract.md` MUST statements are covered by: R-001, R-002, R-005, R-008, R-009, R-010, R-016.
- `SCHEMA.md` MUST / MUST NOT statements are covered by: R-001, R-002, R-003, R-004, R-005, R-006, R-007, R-010, R-011.
- `PROTOCOL.md` MUST statements are covered by: R-003, R-011, R-012, R-017.
- `ENV.md` MUST statements are covered by: R-013, R-014.
- `SECURITY.md` MUST statements are covered by: R-009, R-014, R-015.
