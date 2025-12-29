# ADR-0004 — World OverlayFS Directory Enumeration Reliability

## Status
- Status: Draft
- Date (UTC): 2025-12-29
- Owner(s): spenser

## Scope
- Feature directory: `docs/project_management/next/world-overlayfs-enumeration/`
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

## Problem / Context
- Substrate’s Linux world execution uses overlayfs to present an isolated, writable view of the project and to compute filesystem diffs.
- On some hosts, the overlay mount can succeed and writes can occur, but directory enumeration in the merged overlay view returns an empty listing (e.g., `ls` shows nothing) even when files exist and can be `stat`’d.
- This breaks the fundamental world contract: interactive workflows (shell, editors, build tools) depend on accurate directory listings, and Substrate becomes confusing/unsafe to use because “writes happened” but the user cannot observe them inside the world.

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
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: command succeeded
  - `3`: world filesystem strategy unavailable when required (no viable strategy passes the enumeration health check)
  - `4`: host prerequisites missing for the requested world/isolation mode and Substrate is not allowed to degrade

### Config
- Files and locations (precedence): unchanged by this ADR.
- Schema: unchanged by this ADR.

### Platform guarantees
- Linux:
  - Substrate MUST NOT run a “world” command on a filesystem strategy that cannot enumerate directories correctly.
  - Substrate MUST attempt the selected primary world filesystem strategy first; if it fails the enumeration health check, Substrate MUST attempt the configured fallback strategy chain (see “Security / Safety Posture”).
  - Substrate MUST record the chosen strategy and any fallback reason in trace spans for reproducibility.
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
  - If world execution is selected but not required, and no viable strategy exists, Substrate MAY fall back to host execution, but MUST record the fallback reason in trace/telemetry and MUST surface a clear warning to the user.
- Protected paths/invariants:
  - Mount operations MUST remain confined to Substrate-managed overlay roots and the target project path within a private mount namespace.
  - The project bind enforcement MUST prevent absolute-path escapes back into the host project when running in “project” isolation mode.

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

