# ADR Registry

This directory is the canonical home for ADRs (Architecture Decision Records) across the repo.

It exists to make cross-queue discovery deterministic for planning steps like:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`

## Directory taxonomy

- `docs/project_management/adrs/draft/`
  - ADRs that are not accepted yet.
  - Status in the file remains `Draft`.

- `docs/project_management/adrs/queued/`
  - ADRs that are accepted (or ready to be accepted) and are not implemented yet.
  - Status in the file is typically `Accepted`.

- `docs/project_management/adrs/implemented/`
  - ADRs that are fully implemented/landed.

- `docs/project_management/adrs/superseded/`
  - ADRs that are obsolete and replaced by another ADR.
  - The ADR must link to the replacing ADR.

## Legacy locations (still supported)

New ADRs should prefer `docs/project_management/adrs/` unless there is a strong reason to keep an ADR feature-local.

Legacy ADR locations may still be referenced by older planning artifacts. Use the migration tooling to move them into
this registry so repo-wide scans can be strict and deterministic.

## Stable Curated ADRs

Curated ADRs that survive the retirement of `docs/project_management/**` now live under:

- `docs/adr/`

Use that tree for stable operator/runtime decision records. Keep this registry for planning-rich,
historical, or not-yet-curated ADR material.
