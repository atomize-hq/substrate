# Planning packs (`docs/project_management/packs/`)

This directory is the canonical home for planning packs (feature directories) and planning-pack metadata.

## Buckets

- `draft/`: early-stage packs (not yet sequenced / not ready for execution)
- `queued/`: ready to start but not yet active
- `active/`: in-progress work
- `implemented/`: completed work (historical reference; generally read-only)
- `superseded/`: abandoned or replaced work (historical reference; read-only)

## Sequencing spine (canonical)

- Canonical: `docs/project_management/packs/sequencing.json`
- Legacy compatibility mirror (during migration only): `docs/project_management/next/sequencing.json`

Policy:
- Tools should **write** only to the canonical path.
- The legacy path is a generated compatibility mirror (kept in sync by `make pm-sync-sequencing`).
- Do not use symlinks for the mirror (Windows compatibility).

