# Integration Map — world-overlayfs-enumeration (ADR-0004)

## Scope / Non-scope

Scope:
- Linux world filesystem strategy selection for overlay-based isolation and diff collection.
- Enumeration health check (`enumeration_v1`) gating world execution.
- Deterministic fallback chain (`overlay` → `fuse`) and explicit host fallback when world is optional.
- Observability: trace fields on `command_complete` spans and `substrate world doctor --json` keys.

Non-scope:
- macOS (Lima) and Windows (WSL) world backends.
- Policy/config format changes.
- Redesign of cgroups/netns/full-cage isolation.

## End-to-end data flow (inputs → derived state → actions → outputs)

Inputs:
- CLI world selection (`substrate --world ...`).
- Policy-derived “world required” constraint.
- Platform capability probes (overlayfs available, fuse-overlayfs available).

Derived state:
- `world_required`: `true|false`
- `world_fs_strategy_primary`: `overlay`
- `world_fs_strategy_fallback`: `fuse`
- `world_fs_strategy_final`: `overlay|fuse|host`
- `world_fs_strategy_fallback_reason`: enum (ADR-0004)

Actions:
- Create overlay mount topology for the selected strategy.
- Run `enumeration_v1` probe on a dedicated probe overlay mount.
- If probe fails: retry once with the fallback strategy.
- If both strategies fail:
  - If world required: fail closed (exit `3`).
  - If world optional: execute on host and emit the warning line contract.

Outputs:
- Correct directory listing semantics inside the world project view.
- Trace annotations for strategy selection and fallback reasons.
- Doctor JSON diagnostics for strategy availability and probe status (via `substrate world doctor --json` → `.world.world_fs_strategy`).

## Component map (what changes where)

- `crates/world`
  - Owns strategy selection and the enumeration probe implementation.
  - Owns mount choreography changes for project cage enforcement.
  - Emits strategy metadata to callers for trace and doctor surfacing.

- `crates/world-agent`
  - Surfaces strategy/probe diagnostics through doctor endpoints used by `substrate world doctor --json`.
  - Carries selected strategy metadata into execution responses where trace spans are written.

- `crates/shell`
  - Routes execution into the Linux world backend and prints the warning-line contract for host fallback (world optional).

- `crates/trace` / `crates/common`
  - Ensures trace fields are written on `command_complete` spans without breaking schema consumers.

## Composition with adjacent tracks

- `world_sync` and other world-dependent tracks assume the world filesystem contract is reliable for interactive tools and integration tests.
- This work is a hard prerequisite for using world-based flows as a validation environment.

## Sequencing alignment

- Sequencing entry: `docs/project_management/next/sequencing.json` → `world_overlayfs_enumeration` (`WO0`)
- This feature is sequenced before world-sync to avoid building new flows on an unreliable world filesystem contract.
