# ADR-0015 — Full Isolation Landlock Compatibility With OverlayFS Backing Dirs

## Status
- Status: Approved
- Date (UTC): 2026-01-20
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Policy snapshot spec: `docs/project_management/_archived/world-agent-policy-snapshot/policy-snapshot-spec.md`
- Decision Register: `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/decision_register.md`
- Related ADRs:
  - `docs/project_management/adrs/implemented/ADR-0014-world-agent-policy-resolution-and-concurrency.md`
- Grounding code references:
  - Full isolation mount wrapper + allowlist remounting: `crates/world/src/exec.rs`
  - Landlock exec wrapper (full isolation policy): `crates/world-agent/src/internal_exec.rs`
  - Snapshot allowlist canonicalization + env injection: `crates/world-agent/src/service.rs`, `crates/world-agent/src/pty.rs`

## Executive Summary (Operator)

ADR_BODY_SHA256: 7fb450fcafcf3cb53fd576b167a8bc26ab6aded352f141b91afec7f08cb50d77
### Changes (operator-facing)
- Full isolation writable allowlists stop failing with EPERM on Linux
  - Existing: With `world_fs.isolation=full` and `world_fs.mode=writable`, project writes can fail with `Operation not permitted` even when `world_fs.write_allowlist` explicitly allows the target project prefix.
  - New: Allowlisted project writes succeed consistently; non-allowlisted project writes remain denied. Failures surface a high-signal diagnostic when the runtime cannot derive required internal write roots.
  - Why: Landlock enforcement currently blocks overlayfs’ internal upper/work backing directory writes (not user policy writes), which breaks the operator contract for `world_fs.write_allowlist`.
  - Links:
    - `docs/ISOLATION_SUPPORT_MATRIX.md`
    - `docs/project_management/_archived/world-agent-policy-snapshot/decision_register.md`
    - `crates/world-agent/src/internal_exec.rs`
    - `crates/world/src/exec.rs`

## Problem / Context
- In `world_fs.isolation=full`, Substrate uses overlayfs to present an isolated project view while keeping the host project directory immutable.
- In full isolation, Substrate also applies a Landlock policy (best-effort hardening).
- Overlayfs writes are serviced via the overlay backing directories (`upperdir` / `workdir`), which live under `/var/lib/substrate/overlay/<WORLD_ID>/...`.
- Today, the Landlock policy allowlists project paths (e.g. `/project/<prefix>`), but does not allow write access to the overlay backing dirs.
- Result: allowlisted project writes fail with `EPERM` despite correct policy snapshot resolution and correct mount-time remounting of the allowlisted subtree.

## Goals
- Ensure `world_fs.write_allowlist` works in `isolation=full` on Linux without spurious `EPERM` failures.
- Preserve the existing enforcement contract:
  - allowlisted project writes succeed in full isolation + writable mode
  - non-allowlisted project writes remain denied
- Keep policy snapshot hashes deterministic (runtime-derived internal paths must not alter snapshot schema/hash).
- Provide high-signal diagnostics when runtime requirements cannot be met.

## Non-Goals
- Introducing new user-facing allowlist syntaxes (regex / extensions / file matchers).
- Changing policy snapshot schema versions or hash computation semantics.
- Redesigning the full isolation filesystem strategy (overlay → alternate) as part of this body of work.

## User Contract (Authoritative)

### CLI
- In `world_fs.isolation=full` and `world_fs.mode=writable`:
  - If `world_fs.write_allowlist` contains a pattern that covers a project-relative prefix, writing under that prefix MUST succeed (subject to normal command behavior).
  - If a write target is not covered by `world_fs.write_allowlist`, the write MUST be denied.
- Failure behavior:
  - If full isolation requires runtime-derived internal write roots and they cannot be derived, Substrate MUST fail closed (no host fallback when `world_fs.require_world=true`) and emit a high-signal diagnostic indicating the missing runtime requirement.

### Config
- No schema changes. `world_fs.write_allowlist` remains a list of project-relative glob patterns (already merged and snapshotted).

### Platform guarantees
- Linux:
  - Full isolation + overlayfs + Landlock MUST not regress allowlisted writability.
  - Landlock remains best-effort; if Landlock is unsupported by the kernel, Substrate continues relying on mount semantics (no new failure mode introduced solely due to Landlock unavailability).
- macOS / Windows:
  - No behavior changes required by this ADR (guest backends may still differ based on guest kernel capabilities).

## Architecture Shape
- Components:
  - `crates/world-agent/src/internal_exec.rs`: extends the full-isolation Landlock policy to include runtime-derived internal write roots required by the active filesystem strategy (overlayfs backing dirs).
  - `crates/world` (new helper module or existing procfs utilities): provides a Linux-only helper that inspects `/proc/self/mountinfo` (preferred) to resolve the backing dirs for the project mountpoint.
- End-to-end flow:
  - Inputs:
    - `SUBSTRATE_MOUNT_PROJECT_DIR` (project mountpoint inside the mount namespace)
    - policy-derived allowlist env vars (`SUBSTRATE_WORLD_FS_LANDLOCK_*_ALLOWLIST`, `SUBSTRATE_WORLD_FS_WRITE_ALLOWLIST`)
    - isolation mode (`SUBSTRATE_WORLD_FS_ISOLATION=full`)
  - Derived state:
    - `overlay_upperdir`, `overlay_workdir` (and/or an enclosing overlay state dir)
  - Actions:
    - Extend the Landlock `write_paths` allowlist to include the derived internal write roots needed for overlayfs to perform writes.
  - Outputs:
    - allowlisted project writes succeed; denied writes remain denied; diagnostics improve on failure.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` (world-agent policy snapshot parity workstream)
- Prerequisites:
  - Policy snapshot allowlist canonicalization and env injection must already be in place (covered by ADR-0014 and the policy snapshot implementation).

## Security / Safety Posture
- Fail-closed rules:
  - If `isolation=full` and Landlock is supported but overlay backing dirs cannot be derived for the active strategy, treat this as a world execution failure (no silent host fallback when world is required).
- Invariants:
  - Runtime-derived internal paths MUST NOT affect policy snapshot schema or hashing.
  - Derived internal write roots must be scoped to the active session’s backing dirs (avoid broad allowlists like `/var/lib/substrate/overlay`).
- Observability:
  - Trace metadata remains based on policy snapshot fields; runtime-derived internal allowlists are not part of snapshot hashes.
  - Diagnostics should include enough detail to identify missing procfs data or mount parsing failures without leaking secrets.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Add Linux-only tests for the mountinfo parsing helper using fixture mountinfo text.
- Integration tests:
  - Add/extend a Linux world-agent integration test gated on Landlock support that asserts:
    - allowlisted writes succeed in full isolation (`./prim/*` style prefixes)
    - denied writes remain denied
    - failure modes surface high-signal diagnostics when prerequisites are missing

### Manual validation
- Use the policy snapshot manual testing playbook for the feature directory and include a “full isolation write_allowlist” case.

### Smoke scripts
- Linux: validate in the policy snapshot smoke (feature directory smoke scripts).

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none (behavior is a bug fix; contracts remain the same)

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/decision_register.md`:
    - DR-0001
