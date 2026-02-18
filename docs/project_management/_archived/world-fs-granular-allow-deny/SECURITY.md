# Security Posture — World FS Allow/Deny + Strict Deny

This document is authoritative for the security meaning of:
- `world_fs.enforcement=strict|best_effort`

It supplements (but must not contradict):
- `docs/project_management/adrs/implemented/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`
- `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`

## Threat model

### Assets
- Sensitive project content that must not be readable by world workloads (e.g., `./secrets/**`, `**/*.pem`).

### Adversary
- Potentially malicious or compromised workload executing inside the world (including child processes).

### Enforcement mechanisms in scope
- Landlock (allowlist-only; cannot represent exceptions under broad allows like `.`).
- Mount namespace masking / remounting (can subtract visibility/access by covering paths).

## Why strict mode exists

Mount masking is only a reliable deny mechanism if the workload cannot later undo it with:
- `umount(2)` / `umount2(2)`
- `mount(2)` (including `remount` options)
- newer mount APIs (`open_tree(2)`, `move_mount(2)`, `fsopen(2)`, `fsmount(2)`, `fspick(2)`)

If the workload retains mount authority in its namespace (e.g., due to userns root mapping), it can potentially remove the masks and regain access to denied content.

## Enforcement modes

### `best_effort`
- Meaning:
  - Substrate applies deny masks before executing the inner command.
  - Substrate does not guarantee the workload cannot remove/alter those masks after it starts.
- Intended use:
  - Compatibility mode for tooling that requires mount operations inside the world.
- Explicit non-guarantee:
  - Deny rules are not a hard security boundary under adversarial workloads.

### `strict`
- Meaning:
  - Deny rules are a hard security boundary.
  - After Substrate applies deny masks, the workload must not be able to remove/alter them.
- Required properties:
  - The runtime MUST prevent mount/umount/remount operations by the workload after deny masks are applied.
  - The enforcement must apply to the entire process tree (workload + descendants).
- Implementation-level requirements (contract, not implementation prescription):
  - Drop mount-related capability authority for the workload.
  - Block mount-family syscalls for the workload (seccomp).

## Scope constraints
- Strict/best-effort enforcement is supported only in `world_fs.isolation=full`.
- Any attempt to configure strict/best-effort in `world_fs.isolation=workspace` is a hard error (no “ignore” behavior).
- If any `deny_list` is non-empty, `world_fs.require_world` MUST be `true` (deny rules cannot be configured in a mode that allows fallback/degradation).

## Known limitations
- Wildcard denies (e.g., `**/*.pem`) are enforced as “snapshot at exec start” per `SCHEMA.md`.
- This feature does not guarantee denial of files created/renamed and accessed within a single long-running process after exec.
