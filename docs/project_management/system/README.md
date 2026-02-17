# Project Management System (Canonical)

This directory is the **portable system root** for PM “system” assets: standards, prompts, templates, and schemas.

## System vs artifacts

- **System** = reusable, canonical references used by humans/agents (standards, prompts, templates, schemas).
- **Artifacts** = feature- or run-specific outputs produced by planning/execution (Planning Packs, reports, tasks).

Canonical artifact roots live elsewhere (for example `docs/project_management/next/<feature>/`).

## Canonical locations

- Standards: `docs/project_management/system/standards/**`
- Prompts: `docs/project_management/system/prompts/**`
- Templates: `docs/project_management/system/templates/**`
- Schemas: `docs/project_management/system/schemas/**`

## Environment roots contract

The PM tooling uses these environment roots:

- `PM_ROOT`: repo PM root (typically `docs/project_management/`)
- `PM_SYSTEM_ROOT`: canonical system root (this directory)
- `PM_ADRS_ROOT`: ADRs root (typically `docs/project_management/adrs/`)
- `PM_PACKS_ROOT`: planning packs root (typically `docs/project_management/next/`)
- `PM_DEFAULT_PACK_BUCKET`: default bucket name under packs root (typically `next`)

Path resolver helper:

- `scripts/planning/pm_paths.py`
  - `print-roots`
  - `resolve-feature-dir`

## Migration note

Legacy locations remain during migration. The **source of truth** is `docs/project_management/system/**`.

