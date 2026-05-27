# ADR Tree

This directory is the stable home for curated Architecture Decision Records that still matter to
the current Substrate product, runtime, and operator contract.

Use this tree for ADRs that should survive the retirement of `docs/project_management/**`.

## Structure

- `docs/adr/implemented/`
  - Curated ADRs whose decisions are accepted and materially reflected in shipped code, stable
    contract docs, or operator/runtime behavior.
- `docs/adr/draft/`
  - Curated ADRs that are still active design inputs but are not implemented yet.
- `docs/adr/historical/`
  - Curated ADRs kept only for historical reasoning, supersession context, or audit trail.

## Curation Policy

- Do not blindly move every ADR from `docs/project_management/adrs/**`.
- Curate first, then promote only the keepers.
- Keep an ADR in this tree only when at least one of these is true:
  - it remains normative for the current product, runtime, or operator contract
  - it is implemented and still explains current behavior
  - stable docs or code still depend on it as a named decision anchor
  - it records a still-relevant supersession boundary
- Leave planning-only or pack-coupled artifacts in `docs/project_management/**` until they are
  either promoted here or intentionally archived.

## Migration Rule

Use `restate + supersede`, not a blind filesystem move, when curating ADRs out of
`docs/project_management/adrs/**`.

That means:

1. Create the curated ADR under `docs/adr/**`.
2. Rewrite it so links and status reflect the stable post-project-management world.
3. Leave a short compatibility stub or supersession note behind in
   `docs/project_management/adrs/**` when existing planning/history docs still point there.
4. Repoint stable docs and current contracts to `docs/adr/**`.

This avoids dragging planning-pack assumptions, feature-directory references, and stale status
labels into the stable ADR tree unchanged.

## Current Ledger

The active classification ledger lives in `docs/adr/CURATION.md`.
