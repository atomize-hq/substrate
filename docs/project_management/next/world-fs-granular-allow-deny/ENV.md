# Env Var Contract — World FS Allow/Deny + Strict Deny (Authoritative)

This document is authoritative for:
- `docs/project_management/next/ADR-0018-world-fs-granular-allow-deny-and-strict-deny.md`

It defines the env var interfaces between:
- host shell → world-agent,
- world-agent → world backend exec,
- mount script (`PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT`) → helper (`__substrate_world_landlock_exec`),
- helper → inner command (`sh -c/-lc $SUBSTRATE_INNER_CMD`).

## 1) Existing env vars (grounded in current code)

Mount/exec wrapper inputs (consumed by `crates/world/src/exec.rs` script):
- `SUBSTRATE_WORLD_FS_ISOLATION` (`workspace|full`)
- `SUBSTRATE_MOUNT_MERGED_DIR`
- `SUBSTRATE_MOUNT_PROJECT_DIR`
- `SUBSTRATE_MOUNT_CWD`
- `SUBSTRATE_MOUNT_FS_MODE` (`read_only|writable`)
- `SUBSTRATE_INNER_CMD`
- `SUBSTRATE_INNER_LOGIN_SHELL` (`0|1`)
- `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST` (newline-separated project-relative prefixes; legacy mount-based write allowlist)
- `SUBSTRATE_LANDLOCK_HELPER_SRC` (host path to world-agent binary to bind into full isolation root)

Helper inputs (consumed by `crates/world-agent/src/internal_exec.rs`):
- `SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST` (newline-separated absolute allow paths)
- `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST` (newline-separated absolute allow paths)

## 2) New env vars (V2 feature)

### 2.1 Enforcement plan (required when V2 world_fs is used)
- `SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`
  - Encoding: base64 JSON (UTF-8).
  - Producer: world-agent (non-PTY: `crates/world-agent/src/service.rs`; PTY: `crates/world-agent/src/pty.rs`).
  - Consumer: helper (`crates/world-agent/src/internal_exec.rs`), prior to executing the inner command.
  - Purpose:
    - carry deny patterns (project-relative),
    - carry strict/best-effort enforcement choice,
    - carry any helper-side scan/mask parameters needed for deterministic behavior.

Minimal JSON shape:
```json
{
  "version": 1,
  "enforcement": "strict|best_effort",
  "read_deny": ["./secrets/**", "**/*.pem"],
  "discover_deny": ["./secrets/**"],
  "write_deny": ["./outputs/private/**"]
}
```

### 2.2 Landlock discover allowlist (optional)
- `SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST`
  - Encoding: newline-separated absolute allow paths.
  - Producer: world-agent.
  - Consumer: helper.
  - Purpose: allow directory listing/visibility (`READ_DIR`) without implying file reads (`READ_FILE`).

## 3) Helper invocation rule (critical)
The mount script MUST execute the helper whenever V2 enforcement is required.

Specifically, in `crates/world/src/exec.rs`:
- The script currently executes helper only when:
  - `SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST` or `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST` is set.
- Under ADR-0018, the script MUST also execute helper when:
  - `SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64` is set (deny enforcement and strict lockdown live in helper).

## 4) Strict deny lockdown (security boundary)
When `enforcement=strict`:
- The helper MUST ensure the workload cannot undo deny masks.
- Requirements:
  - Drop mount-related capability authority for the workload (e.g., clear effective/permitted caps in the namespace).
  - Block mount-family syscalls for the workload (seccomp), including at minimum:
    - `mount`, `umount2`, `pivot_root`
    - and modern mount APIs (`open_tree`, `move_mount`, `fsopen`, `fsmount`, `fspick`) when available.
- These actions must occur **after** the helper applies deny mounts and **before** it `exec`s `sh -c/-lc $SUBSTRATE_INNER_CMD`.

In `best_effort` mode:
- The helper applies deny mounts but does not enforce the strict lockdown.

