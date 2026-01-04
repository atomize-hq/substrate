# Environment Variables Contract (Operator-Facing)

This file is scaffolding: it will define the supported environment variable interface for operators and automation.

Non-goals for this file:
- It must not try to be exhaustive across internal/test usage.
- It must not list standard env vars unless Substrate requires a specific semantic beyond the OS default.

## Taxonomy (contract-level)

Intended categories to document here:
- Config override inputs (operator-settable)
- Diagnostics toggles (operator-settable, explicitly non-stable if applicable)

Explicitly out of scope for this contract:
- Exported state variables that Substrate writes for propagation (`SUBSTRATE_*` state exports)
- Internal coordination variables for shim/world/trace (`SHIM_*`, `WORLD_*`, `TRACE_*`)

## Related docs
- Entry point: `docs/ENVIRONMENT_VARIABLES.md`
- Inventory (developer): `docs/internals/env/inventory.md`
- Configuration overview: `docs/CONFIGURATION.md`
- Governing ADR(s): `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

## Contract table (to be filled)

This table should list only supported operator inputs (example shape):

| Variable | Type / Allowed values | Default | Precedence notes | Purpose |
| --- | --- | --- | --- | --- |
| `SUBSTRATE_OVERRIDE_POLICY_MODE` | `disabled|observe|enforce` | *(none)* | CLI flags > workspace config > overrides > global config > defaults | Override effective policy mode |

