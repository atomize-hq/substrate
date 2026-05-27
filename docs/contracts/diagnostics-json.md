# Diagnostics JSON Contract

This document is the durable machine-readable contract reference for:

- `substrate shim doctor --json`
- `substrate health --json`

Related references:
- `docs/USAGE.md`
- `docs/reference/env/contract.md`
- `docs/adr/implemented/ADR-0036-world-disabled-first-class-status-in-health-and-shim-doctor.md`
- `docs/adr/implemented/ADR-0037-doctor-health-attribute-why-world-is-disabled.md`

## Compatibility Policy

- additive-only
- existing fields must not be renamed or removed
- consumers must ignore unknown fields and unknown enum values

## Shim Doctor Status Fields

### `.world.status`

- type: string enum
- required when `.world` is present
- allowed values:
  - `healthy`
  - `needs_attention`
  - `disabled`
  - `unknown`

Semantics:

- `disabled`: effective config resolved `world.enabled=false`; backend probing was skipped
- `healthy`: effective config resolved `world.enabled=true` and the backend probe succeeded
- `needs_attention`: effective config resolved `world.enabled=true` and the backend probe failed in
  an actionable way
- `unknown`: reporting could not determine the state

Legacy field rule:

- `.world.ok` remains additive/legacy and is not the canonical disabled-state classifier

### `.world_deps.status`

- type: string enum
- required when `.world_deps` is present
- allowed values:
  - `ok`
  - `error`
  - `skipped_disabled`
  - `unknown`

Semantics:

- `skipped_disabled`: effective config resolved `world.enabled=false`; applied probing was skipped
- `ok`: snapshot collection and applied probing succeeded
- `error`: snapshot collection or applied probing failed
- `unknown`: reporting could not determine the state

Legacy field rule:

- `.world_deps.error` and `.world_deps.report.applied_error` remain legacy error surfaces, but
  `.world_deps.status` is the canonical machine-readable classifier

## Health JSON Shape

`substrate health --json` contains:

- top-level `shim`: the full `substrate shim doctor --json` payload
- top-level `summary`: a derived summary

Canonical summary inputs for world-disabled behavior are:

- `.shim.world.status`
- `.shim.world_deps.status`

## Disabled-World Omission Rules

When effective config resolves `world.enabled=false`:

- shim doctor:
  - `.world.status` must be `disabled`
  - `.world.details` must be omitted
  - `.world.error` must be omitted
  - `.world_deps.status` must be `skipped_disabled`
  - `.world_deps.report` must be omitted
  - `.world_deps.error` must be omitted
- health summary:
  - `.summary.world_ok` must be `null`
  - `.summary.world_error` must be omitted
  - `.summary.world_deps_error` must be omitted
  - `.summary.world_deps_missing` must be `[]`
  - `.summary.world_deps_blocked` must be `[]`
  - `.summary.failures` must not include failures caused solely by the disabled short-circuit

## Enabled-World Rules

When effective config resolves `world.enabled=true`:

- `.world.status` must not be `disabled`
- `.world_deps.status` must not be `skipped_disabled`
- `.world_deps.status` must be `error` when either:
  - `.world_deps.error` is present
  - `.world_deps.report.applied_error` is present
