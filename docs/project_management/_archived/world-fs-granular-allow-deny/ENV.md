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

### 2.1 Enforcement plan (required iff any deny_list is non-empty)
- `SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`
  - Encoding: base64 JSON (UTF-8).
  - Producer: world-agent (non-PTY: `crates/world-agent/src/service.rs`; PTY: `crates/world-agent/src/pty.rs`).
  - Consumer: helper (`crates/world-agent/src/internal_exec.rs`), prior to executing the inner command.
  - Purpose:
    - carry deny patterns (project-relative),
    - carry strict/best-effort enforcement choice,
    - carry any helper-side scan/mask parameters needed for deterministic behavior.

Plan schema (v1; hard errors on violation):
- The decoded value MUST be a JSON object.
- The object MUST contain only the fields listed below (unknown fields MUST be rejected).
- `version` MUST be the integer `1`.
- `enforcement` MUST be `strict` or `best_effort`.
- `read_deny`, `discover_deny`, and `write_deny` MUST be present and MUST each be an array (lists are allowed to be empty).
- Each deny entry MUST be a valid deny pattern per `docs/project_management/_archived/world-fs-granular-allow-deny/SCHEMA.md`.
- The plan MUST represent the effective (defaulted) denies:
  - If `discover` is omitted in the snapshot (meaning it mirrors `read`), world-agent MUST set `discover_deny` equal to `read_deny`.
- If parsing or validation fails, the helper MUST fail closed (world enforcement failure; exit code `4`).

Canonical JSON shape:
```json
{
  "version": 1,
  "enforcement": "strict|best_effort",
  "read_deny": ["./secrets/**", "**/*.pem"],
  "discover_deny": ["./secrets/**", "**/*.pem"],
  "write_deny": ["./outputs/private/**"]
}
```

### 2.2 Landlock discover allowlist (optional)
- `SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST`
  - Encoding: newline-separated absolute allow paths.
  - Producer: world-agent.
  - Consumer: helper.
  - Purpose: allow directory listing/visibility (`READ_DIR`) without implying file reads (`READ_FILE`).
  - If unset, the helper MUST treat the discover allowlist as equal to the read allowlist.

## 3) Helper invocation rule (critical)
Mechanism (grounded in `crates/world/src/exec.rs`):
- The mount script executes the helper iff `SUBSTRATE_LANDLOCK_HELPER_PATH` is set and executable.
- In `world_fs.isolation=full`, the script sets `SUBSTRATE_LANDLOCK_HELPER_PATH` iff `SUBSTRATE_LANDLOCK_HELPER_SRC` is set and exists (it bind-mounts the helper into the isolated rootfs).

Therefore, world-agent MUST set `SUBSTRATE_LANDLOCK_HELPER_SRC` whenever helper-side enforcement is required, including when any of the following are present:
- `SUBSTRATE_WORLD_FS_LANDLOCK_READ_ALLOWLIST`
- `SUBSTRATE_WORLD_FS_LANDLOCK_WRITE_ALLOWLIST`
- `SUBSTRATE_WORLD_FS_LANDLOCK_DISCOVER_ALLOWLIST`
- `SUBSTRATE_WORLD_FS_ENFORCEMENT_PLAN_B64`

Under ADR-0018, deny enforcement and strict lockdown live in the helper, so deny-only configurations MUST still cause the helper to be executed.

## 4) Strict deny lockdown (security boundary)
When `enforcement=strict`:
- The helper MUST ensure the workload cannot undo deny masks.
- Requirements:
  - Drop mount-related capability authority for the workload (e.g., clear effective/permitted caps in the namespace).
  - The helper MUST install a seccomp filter that denies mount-family syscalls for the workload, specifically:
    - `mount`, `umount2`, `pivot_root`
    - `open_tree`, `move_mount`, `fsopen`, `fsmount`, `fspick`
  - If any of the listed syscalls do not exist on the running kernel, denying them is N/A; otherwise they MUST be denied.
  - Denied mount-family syscalls MUST fail with `EPERM` (`Operation not permitted`) and MUST NOT terminate the process.
- These actions must occur **after** the helper applies deny mounts and **before** it `exec`s `sh -c/-lc $SUBSTRATE_INNER_CMD`.

In `best_effort` mode:
- The helper applies deny mounts but does not enforce the strict lockdown.
