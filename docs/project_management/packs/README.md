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
Policy:
- Tools should **write** only to the canonical path.
