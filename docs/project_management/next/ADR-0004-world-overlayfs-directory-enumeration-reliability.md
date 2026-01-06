# ADR-0004 — World OverlayFS Directory Enumeration Reliability

## Status
- Status: Accepted
- Date (UTC): 2025-12-29
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/next/world-overlayfs-enumeration/`
- Orchestration branch: `feat/world-overlayfs-enumeration`
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

## Related Docs
- Plan: `docs/project_management/next/world-overlayfs-enumeration/plan.md`
- Tasks: `docs/project_management/next/world-overlayfs-enumeration/tasks.json`
- Specs:
  - `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
- Decision Register: `docs/project_management/next/world-overlayfs-enumeration/decision_register.md`
- Integration Map: `docs/project_management/next/world-overlayfs-enumeration/integration_map.md`
- Manual Playbook: `docs/project_management/next/world-overlayfs-enumeration/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 5869e26e0dcaea4e2cd25daf4fe5c1b93fdfba697798993472632b13c8c6c33a
### Changes (operator-facing)
- World overlay directory enumeration reliability (Linux)
  - Existing: On some Linux hosts, a world overlay mount can succeed but directory enumeration in the merged view is broken (e.g., `ls` shows empty even though `stat` works), making worlds unsafe/confusing.
  - New: Substrate refuses to run a world command on any filesystem strategy that fails an enumeration health check; it selects a strategy + fallback chain deterministically and records the choice/fallback reason in trace and doctor output.
  - Why: Restores the core world contract (observable filesystem state) and makes host variance reproducible and debuggable.
  - Links:
    - `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
    - `docs/project_management/next/world-overlayfs-enumeration/manual_testing_playbook.md`
    - `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`

## Problem / Context
- Substrate’s Linux world execution uses overlayfs to present an isolated, writable view of the project and to compute filesystem diffs.
- On some hosts, the overlay mount can succeed and writes can occur, but directory enumeration in the merged overlay view returns an empty listing (e.g., `ls` shows nothing) even when files exist and can be `stat`’d.
- This breaks the fundamental world contract: interactive workflows (shell, editors, build tools) depend on accurate directory listings, and Substrate becomes confusing/unsafe to use because “writes happened” but the user cannot observe them inside the world.

## Reality Check / Evidence (Grounding)
- This ADR is grounded in the current Linux project isolation implementation, which today:
  - Mounts an overlay at a Substrate-managed merged directory, then
  - Enters a private mount namespace and bind-mounts the merged directory onto the host project path for “project cage” enforcement.
- Implementation references (for grounding only; this ADR remains authoritative):
  - Project cage enforcement script: `crates/world/src/exec.rs` (mount namespace wrapper; bind-mount enforcement)
  - Overlay mount + fuse fallback plumbing: `crates/world/src/overlayfs/`
- The “enumeration broken but stat works” failure mode is consistent with a merged-view `readdir(3)` failure while direct path lookup continues to succeed; Substrate’s operator-visible symptom is `ls`/`find` returning empty while tools can still access known paths.
- This ADR treats directory enumeration correctness as a hard safety prerequisite for world execution: if enumeration is unhealthy, world execution is refused (or degrades to host only when world is optional).

## Goals
- Ensure directory enumeration inside the world overlay is correct and stable (files created in the world are visible via `readdir`/`ls`).
- Make the world filesystem strategy deterministic and observable (strategy selection + fallback reason recorded in trace/doctor output).
- Preserve Substrate’s fail-closed posture when world execution is required (CLI `--world` or enforce-mode “requires world” constraints).

## Non-Goals
- Redesigning the entire Linux isolation model (cgroups/netns/full-cage are out of scope).
- Implementing new policy semantics for filesystem allowlists beyond the existing world mounts.
- Changing config/policy file formats, naming, or precedence.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate` interactive shell and `substrate --world ...` MUST present a coherent filesystem view:
    - A file created in the project directory inside the world MUST be discoverable via directory enumeration (e.g., `ls`, `find`, `readdir`) for the duration of that world session/command.
  - `substrate world doctor --json` MUST report whether the host supports a functional overlay-based world filesystem strategy for the current environment (see “Platform guarantees”).
- World requirement:
  - World is **required** if either is true:
    - The CLI explicitly requests world execution (`--world`), or
    - Policy mode is `enforce` and the command is under a “requires world” constraint, including any of:
      - `world_fs.require_world=true`
      - `world_fs.mode=read_only`
      - `world_fs.isolation=full`
      - command matched `allow_with_restrictions` (isolated match)
  - World is **optional** otherwise.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: command succeeded (in world, or in host fallback when world is optional)
  - `2`: usage/config error (invalid flags/config that prevent execution)
  - `3`: world filesystem strategy unavailable when required (no viable in-world strategy passes the enumeration health check)
  - `5`: safety/policy violation (policy deny, protected-path violation)

### Config
- Files and locations (precedence): unchanged by this ADR.
- Schema: unchanged by this ADR.

### Platform guarantees
- Linux:
  - Substrate MUST NOT run a “world” command on a filesystem strategy that cannot enumerate directories correctly.
  - Substrate MUST attempt the selected primary world filesystem strategy first; if it fails the enumeration health check, Substrate MUST attempt the configured fallback strategy chain (see “Security / Safety Posture”).
  - Substrate MUST record the chosen strategy and any fallback reason in trace spans for reproducibility.
  - Enumeration probe contract (authoritative):
    - Probe id: `enumeration_v1`
    - Probe mechanism:
      1. Create a dedicated probe overlay mount (not the user session overlay) whose merged directory represents the project root.
      2. In the probe merged directory, create a probe file at `./.substrate_enum_probe` (exact filename).
      3. Enumerate the directory entries using `ls -a1` and assert the output contains a line exactly equal to `.substrate_enum_probe`.
      4. Remove the probe file.
    - A strategy is “enumeration healthy” if and only if the probe succeeds.
    - Any failure in steps (1)-(4) is an unhealthy result for that strategy.
  - Strategy selection contract (authoritative):
    - Primary strategy: kernel overlayfs (`overlay`)
    - Fallback strategy: fuse-overlayfs (`fuse`)
    - If primary is unavailable or probe-unhealthy: attempt fallback exactly once.
    - If neither strategy is viable:
      - If world is required: exit `3` and do not execute on host.
      - If world is optional: execute on host and emit the warning line defined below.
  - Warning line contract (authoritative):
    - If and only if Substrate executes on host due to “world optional + no viable strategy”, it MUST emit exactly one warning line to stderr:
      - `substrate: warn: world unavailable; falling back to host`
- macOS:
  - No behavior changes required by this ADR (Lima-backed worlds are out of scope).
- Windows:
  - No behavior changes required by this ADR (WSL-backed worlds are out of scope).

## Architecture Shape
- Components:
  - `crates/world`:
    - Own the Linux world filesystem strategy selection (overlayfs vs fuse-overlayfs vs other) and health checking.
    - Ensure mounts are created in a topology that is stable under bind mounts and mount namespaces.
  - `crates/world-agent`:
    - Carry strategy metadata into execution responses/trace context.
    - Surface health/strategy diagnostics through doctor endpoints.
  - `crates/shell` + `crates/trace`:
    - Record strategy selection and fallback reasons in trace spans (no schema-breaking changes; add fields only if strictly required).
- End-to-end flow:
  - Inputs:
    - world selection (CLI/env/config), policy-derived “requires world” constraints, and platform capability probes
  - Derived state:
    - chosen world filesystem strategy + fallback chain; “world required” vs “world optional”
  - Actions:
    - create mounts in a private mount namespace using a stable lower snapshot
    - run the command with project bind enforcement
    - validate directory enumeration health; retry with fallback strategy on failure
  - Outputs:
    - correct directory listings inside the world
    - trace spans annotate strategy + fallback reason
    - doctor output reports strategy health

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `world_overlayfs_enumeration`
- Prerequisite integration task IDs:
  - `WO0-integ` before `WO1-code` (if later triads extend into PTY parity or full-cage)

## Security / Safety Posture
- Fail-closed rules:
  - If world execution is required (CLI `--world` or enforce-mode “requires world”) and no viable world filesystem strategy passes the enumeration health check, Substrate MUST fail closed with exit code `3`.
- Degrade rules:
  - If world execution is selected but not required, and no viable strategy exists, Substrate MUST fall back to host execution, MUST record the fallback reason in trace/telemetry, and MUST surface a clear warning to the user.
- Protected paths/invariants:
  - Mount operations MUST remain confined to Substrate-managed overlay roots and the target project path within a private mount namespace.
  - The project bind enforcement MUST prevent absolute-path escapes back into the host project when running in “project” isolation mode.
- Observability contract (authoritative):
  - Trace spans (on `command_complete` events) MUST include these keys:
    - `world_fs_strategy_primary`: `overlay|fuse`
    - `world_fs_strategy_final`: `overlay|fuse|host`
    - `world_fs_strategy_fallback_reason`: one of:
      - `none` (no fallback occurred)
      - `primary_unavailable`
      - `primary_mount_failed`
      - `primary_probe_failed`
      - `fallback_unavailable`
      - `fallback_mount_failed`
      - `fallback_probe_failed`
      - `world_optional_fallback_to_host`
  - `substrate world doctor --json` MUST include these keys:
    - `world_fs_strategy_primary`: `overlay`
    - `world_fs_strategy_fallback`: `fuse`
    - `world_fs_strategy_probe`:
      - `id`: `enumeration_v1`
      - `probe_file`: `.substrate_enum_probe`
      - `result`: `pass|fail`
      - `failure_reason`: string or null

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - `crates/world` tests validating strategy selection and the enumeration health check behavior.
- Integration tests:
  - A Linux integration test that runs a world command which creates a file and asserts the file appears in directory enumeration inside the world view.
  - A regression test that simulates an unhealthy overlay strategy (forced via test hook) and asserts fallback strategy selection and trace annotation.

### Manual validation
- Manual playbook: `docs/project_management/next/world-overlayfs-enumeration/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`
- macOS: `docs/project_management/next/world-overlayfs-enumeration/smoke/macos-smoke.sh`
- Windows: `docs/project_management/next/world-overlayfs-enumeration/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none

## Decision Summary
- Decision Register entries:
  - `docs/project_management/next/world-overlayfs-enumeration/decision_register.md`:
    - DR-0001, DR-0002
