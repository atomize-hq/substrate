# ADR-0018 — World FS Granular Allow/Deny (Read/Discover/Write) + Strict Deny (Full Isolation Only)

## Status
- Status: Draft
- Date (UTC): 2026-01-29
- Owner(s): spenser, Substrate maintainers

## Scope
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

## Related Docs
- Plan: `docs/project_management/next/world-fs-granular-allow-deny/plan.md`
- Decision Register: `docs/project_management/next/world-fs-granular-allow-deny/decision_register.md`
- Schema (authoritative): `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
- Protocol (authoritative): `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
- Env var contract (authoritative): `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`
- Integration map: `docs/project_management/next/world-fs-granular-allow-deny/integration_map.md`
- Manual playbook: `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`
- Related ADRs / grounding:
  - Policy snapshot direction and threat model: `docs/project_management/next/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
  - Full isolation mount/exec chokepoint: `crates/world/src/exec.rs`
  - Landlock exec wrapper: `crates/world-agent/src/internal_exec.rs`
  - Snapshot resolution on host + drift handling (REPL): `crates/shell/src/execution/policy_snapshot.rs`, `crates/shell/src/repl/async_repl.rs`
  - Full isolation Landlock overlayfs compatibility: `docs/project_management/next/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 64ac13a560ad971697a15a9014565a944d30e2b146b80730223a2ed1f1e9cdb6
### Changes (operator-facing)
- Add granular `allow_list` + `deny_list` for world filesystem reads/writes (and optional directory visibility)
  - Existing: `world_fs.read_allowlist` / `world_fs.write_allowlist` are allowlist-only; invalid patterns (e.g., `..`) can be accepted but ignored; there is no deny list; “allow all except secrets” cannot be expressed.
  - New: `world_fs.read|discover|write.{allow_list,deny_list}` with explicit deny-overrides-allow semantics in `world_fs.isolation=full`.
  - Why: Operators need explicit, enforceable “allow all except X” controls and reliable failure on invalid patterns (no silent ignore).
  - Links:
    - `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`
    - `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`

- Make deny enforcement a true security boundary in full isolation via `world_fs.enforcement=strict`
  - Existing: Any mount-based masking (if added) would be bypassable if the workload can later `umount`/`mount` in its namespace; Landlock cannot subtract an exception once `.` is allowed.
  - New: In `world_fs.isolation=full`, deny rules are enforced via mount masking plus a strict post-setup lockdown that prevents the workload from undoing denies (cap drop + mount syscall blocking).
  - Why: Deny rules are intended to protect secrets under compromise, not only prevent accidental reads.
  - Links:
    - `docs/project_management/next/world-fs-granular-allow-deny/ENV.md`
    - `docs/project_management/next/world-fs-granular-allow-deny/integration_map.md`

- Break policy snapshot and policy YAML schemas (no backwards compatibility)
  - Existing: World-agent accepts `PolicySnapshotV1` with `read_allowlist`/`write_allowlist`, and YAML/patch formats match that.
  - New: Introduce `PolicySnapshotV2` and a V2 policy YAML schema; old keys and old snapshots become hard errors.
  - Why: This body of work cannot be expressed safely in the V1 shape; “accepted but not enforced” must be structurally impossible.
  - Links:
    - `docs/project_management/next/world-fs-granular-allow-deny/PROTOCOL.md`
    - `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`

## Problem / Context
- Operators need “deny overrides allow” (e.g., allow `.` but deny `./secrets/**`) to prevent accidental or malicious access to sensitive project content.
- Landlock (allowlist-only) cannot express “allow everything except X” once broad allows are granted.
- The current codebase has a known foot-gun: allowlist patterns containing `..` can be accepted by higher layers but are silently ignored during allowlist resolution, which can disable enforcement while policy *appears* set.
  - See: `crates/world-agent/src/service.rs` (`resolve_landlock_allowlist_paths` drops `..` segments).
- Full isolation command execution routes through a single chokepoint:
  - `unshare --mount … sh -c PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT` → conditional Landlock exec wrapper (`__substrate_world_landlock_exec`) → `sh -c/-lc $SUBSTRATE_INNER_CMD`.
  - See: `crates/world/src/exec.rs`, `crates/world-agent/src/internal_exec.rs`.
- If Substrate introduces mount-based deny masking without preventing later mount/umount by the workload, deny rules are bypassable in an adversarial model.

## Goals
- Provide a new `world_fs` policy contract with:
  - `read`, `discover`, `write` dimensions, each supporting `allow_list` and `deny_list`.
  - Explicit “deny overrides allow” semantics in `world_fs.isolation=full`.
  - A strict enforcement mode where denies are a hard security boundary.
- Remove “accepted but ignored” policy states:
  - Invalid patterns are rejected (hard error) at config/policy resolution time and at snapshot ingestion time.
- Keep behavior deterministic in PTY and non-PTY routes:
  - The same `policy_snapshot` schema and enforcement logic is used for `/v1/execute` and `/v1/stream start_session`.
  - Existing REPL snapshot drift handling continues to work after migrating snapshot schema versions.

## Non-Goals
- Supporting strict deny enforcement in `world_fs.isolation=workspace` (this ADR explicitly forbids it).
- Guaranteeing “dynamic” wildcard denies within a single long-running command (e.g., creating `x.pem` then reading it later in the same process invocation). Wildcard denies are snapshot-scanned per exec boundary (documented in schema/guarantees).
- Cross-platform parity for strict deny (Linux full isolation is the initial scope; other platforms are out of scope for this ADR).

## User Contract (Authoritative)

### CLI
- Operators edit policy via existing patch file mechanisms (ADR-0008/ADR-0012) and `substrate policy ...` surfaces.
- New keys (V2) are authoritative; old keys are invalid and MUST hard error.
  - Canonical keys:
    - `world_fs.enforcement` (`strict|best_effort`, full isolation only)
    - `world_fs.read.allow_list`, `world_fs.read.deny_list`
    - `world_fs.discover.allow_list`, `world_fs.discover.deny_list` (optional; default mirrors `read`)
    - `world_fs.write.allow_list`, `world_fs.write.deny_list` (required only when `mode=writable`)
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: policy set/show succeeded / execution succeeded
  - `2`: user/config error (invalid schema, invalid pattern, disallowed mode combination)
  - `4`: world enforcement failure (e.g., strict deny prerequisites not met and `require_world=true`)

### Config
- Policy schema is defined in `docs/project_management/next/world-fs-granular-allow-deny/SCHEMA.md`.
- Hard schema constraints (fail closed; no silent ignore):
  - Patterns MUST be project-relative; absolute paths and `..` segments are invalid.
  - `allow_list` MUST be non-empty for all configured dimensions.
  - `deny_list` defaults to empty.
  - `world_fs.enforcement` MUST be present iff at least one `deny_list` is non-empty.
  - If any `deny_list` is non-empty, `world_fs.require_world` MUST be `true`.
- Isolation constraints:
  - `world_fs.enforcement=strict` is valid only when `world_fs.isolation=full`.
  - Any `deny_list` usage is valid only when `world_fs.isolation=full`.

### Platform guarantees
- Linux:
  - Full isolation (`world_fs.isolation=full`) is the only supported mode for deny enforcement.
  - Strict mode MUST prevent the workload from undoing deny mounts (security boundary).
  - When strict prerequisites are unavailable, behavior MUST fail closed (no silent downgrade).
- macOS/Windows:
  - Out of scope for this ADR (future work: guests can add support later; until then, fail closed when strict is requested).

## Architecture Shape
- Components:
  - `crates/broker`: policy YAML schema update to V2 (no compat); validation of pattern grammar and isolation constraints.
  - `crates/agent-api-types`: introduce `PolicySnapshotV2` and update `ExecuteRequest` and WS `start_session` payloads (no compat).
  - `crates/shell`: emit `PolicySnapshotV2` for world-agent requests; REPL drift continues via snapshot hash.
  - `crates/world-agent`:
    - Convert V2 snapshot to env inputs for the mount script + helper.
    - Extend the exec helper to:
      1) apply deny mounts (masking) inside the per-command mount namespace,
      2) apply Landlock allowlists (including `discover` vs `read` split),
      3) enforce strict mode lockdown (cap drop + mount syscall blocking),
      4) exec the inner command (`sh -c/-lc $SUBSTRATE_INNER_CMD`).
  - `crates/world`: maintain the mount/exec chokepoint (`PROJECT_BIND_MOUNT_ENFORCEMENT_SCRIPT`) and ensure the helper is invoked whenever V2 enforcement is required (even if Landlock allowlists are empty).
- End-to-end flow (full isolation):
  - Inputs:
    - `PolicySnapshotV2` (host-resolved effective policy)
    - mount script env vars (`SUBSTRATE_MOUNT_*`, `SUBSTRATE_INNER_*`)
  - Derived state:
    - absolute allowlists for Landlock under both in-namespace project roots (`/project` and `$SUBSTRATE_MOUNT_PROJECT_DIR`)
    - deny plans (project-relative patterns) applied by helper after mounts exist
  - Actions:
    - mount script sets up minimal root + overlay bind-mounts
    - helper applies deny mounts and strict lockdown before executing the workload
  - Outputs:
    - denied operations are deterministic:
      - discover/read denies return `EACCES` (`Permission denied`)
      - write denies return `EROFS` (`Read-only file system`)
    - strict mode blocks bypass via `mount/umount` by the workload

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` (new workstream to be added for ADR-0018)
- Dependencies:
  - Requires ADR-0014 snapshot direction (host-resolved policy snapshot is authoritative).
  - Must remain compatible with ADR-0015 overlayfs backing dir allowlist derivation for full isolation writable mode.

## Security / Safety Posture
- Fail-closed rules (full isolation):
  - If `world_fs.enforcement=strict` is requested but strict prerequisites cannot be applied, the world execution MUST fail (no best-effort fallback).
  - If the policy requires Landlock (e.g., `read.allow_list` narrower than `.`), but Landlock is unsupported, execution MUST fail closed when `require_world=true`.
- Invariants:
  - No silent ignore of invalid patterns (`..`, absolute paths).
  - Deny masks must be applied before any user code runs.
  - In strict mode, the workload must not retain mount authority in its namespace.
- Observability:
  - Execution failures must surface clear diagnostics explaining which prerequisite failed (Landlock unsupported, strict lockdown unavailable, etc.).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - V2 policy schema parsing/validation in broker (invalid keys rejected, invalid patterns rejected).
  - V2 snapshot serialization/hashing stability in shell.
- Integration tests (Linux):
  - Full isolation: allow `.` + deny `./secrets/**` returns `EACCES` for reads and cannot be bypassed by attempted `umount` in strict mode.
  - Discover/read split: allow discover `.` but deny read for specific file(s) produces “visible but not readable” behavior (when configured).
  - Wildcard denies (`**/*.pem`) enforced for matching files present at exec start (documented limitation for within-process creation).

### Manual validation
- Manual playbook is required and authoritative: `docs/project_management/next/world-fs-granular-allow-deny/manual_testing_playbook.md`

### Smoke scripts
- Linux smoke script(s) MUST live under `docs/project_management/next/world-fs-granular-allow-deny/smoke/` (added during execution triads).

## Rollout / Backwards Compatibility
- No backwards compatibility is provided:
  - Old policy YAML keys are invalid and MUST hard error.
  - `PolicySnapshotV1` is rejected by world-agent once this ADR is implemented.
- Operators must update:
  - policy patch file schema to V2, and
  - host shell + world-agent together (version lockstep).

## Decision Summary
- Decision Register: `docs/project_management/next/world-fs-granular-allow-deny/decision_register.md`
  - DR-0001 through DR-0008
