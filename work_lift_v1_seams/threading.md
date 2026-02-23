# Threading — Work Lift v1

This file makes coupling explicit: contracts, dependency edges, and conflict-safe workstreams.

## Contract registry

### CONTRACT-1 — Lift Vector block markers + JSON schema
- **Contract ID**: `CONTRACT-1:work_lift_vector_block_v1`
- **Type**: schema
- **Owner seam**: SEAM-1
- **Consumers (seams)**: SEAM-3, SEAM-5
- **Definition**:
  - Markers in markdown:
    - `<!-- PM_LIFT_VECTOR:BEGIN -->`
    - `<!-- PM_LIFT_VECTOR:END -->`
  - Within markers, a ` ```json { ... } ``` ` object matching `docs/project_management/system/schemas/work_lift_vector.schema.json`.
- **Versioning/compat**: schema MUST be versioned (via `model_version`) and allow additive evolution; strict validation MUST be opt-in and gated (see SEAM-5 S3).

### CONTRACT-2 — Lift model config v1
- **Contract ID**: `CONTRACT-2:work_lift_model_v1`
- **Type**: config
- **Owner seam**: SEAM-2
- **Consumers (seams)**: SEAM-3, SEAM-5
- **Definition**: `docs/project_management/system/schemas/work_lift_model.v1.json` contains:
  - weights (base points),
  - risk multipliers,
  - split trigger thresholds,
  - mapping constants (e.g., lift→estimated_slices divisor),
  - confidence rules for missing inputs and directory/prefix tokens.
- **Versioning/compat**: `work_lift_model.v1.json` is immutable once published; future revisions MUST add a new versioned config file and explicit selection rules (no “latest” scanning).

### CONTRACT-3 — `pm_lift --emit-json` output shape
- **Contract ID**: `CONTRACT-3:pm_lift_emit_json_v1`
- **Type**: API (CLI JSON)
- **Owner seam**: SEAM-3
- **Consumers (seams)**: SEAM-5 (workflow integration), future lint/enforcement tools
- **Definition**: stable JSON keys for:
  - `model_version`, `lift_score`, `estimated_slices`, `confidence`,
  - `triggers`, `missing_inputs`,
  - `vector` (input),
  - `derived` (debuggable intermediate computations).
- **Versioning/compat**: additive keys only; keep existing keys stable.

### CONTRACT-4 — `validate_impact_map.py --emit-json` contract (input to lift)
- **Contract ID**: `CONTRACT-4:impact_map_emit_json_v1`
- **Type**: API (CLI JSON)
- **Owner seam**: SEAM-4 (consumer-defined; source lives in existing tooling)
- **Consumers (seams)**: SEAM-4, SEAM-3
- **Definition**: JSON object containing per-action allowlists (e.g., `create/edit/delete/deprecate` arrays) and a signal of directory-prefix entries (e.g., `dir_prefixes`).
- **Versioning/compat**: treat as a stable internal interface; changes require coordinated update to `pm_lift.py`.

## Dependency graph (text)

- `SEAM-1 blocks SEAM-3` because `pm_lift` needs an authoritative schema/rubric to validate and explain lift inputs.
- `SEAM-2 blocks SEAM-3` because scoring must be driven by model config (weights/triggers) rather than hard-coded constants.
- `SEAM-3 blocks SEAM-5` because workflow integration needs a stable `pm_lift` output contract.
- `SEAM-4 blocks SEAM-5` because workflow integration for Planning Packs depends on pack-derived lift being computed reliably.

## Critical path

1. SEAM-1 (schema + rubric) + SEAM-2 (model config) — establish the contracts
2. SEAM-3 (pm_lift core) — implement config-backed scoring + stable output
3. SEAM-4 (pack-derived inputs) — reliable derived counts + prefix handling + confidence
4. SEAM-5 (advisory integration) — wire into workflow and document usage

## Parallelization notes / conflict-safe workstreams

Note on terminology:
- The **WS-A / WS-B / WS-C / WS-INT** labels in this file are **implementation workstreams** for executing this planning pack in parallel.
- They are **not** canonical Workstream IDs and must not be confused with the initiative-themed registry IDs described in `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` (see the “workstream IDs are initiative-themed” decision).

The work can be parallelized safely with a thin-contract-first approach:

- **WS-A (Artifacts)**: SEAM-1 + SEAM-2. Touch surface is new files under `docs/project_management/system/schemas/` and `docs/project_management/system/standards/shared/`.
- **WS-B (Tooling)**: SEAM-3. Touch surface: `docs/project_management/system/scripts/planning/pm_lift.py` (and optional new helper modules colocated with it).
- **WS-C (Workflow integration)**: SEAM-5. Touch surface: `Makefile`, planning docs, and any advisory “how to run” guidance; no hard enforcement.
- **WS-INT (Integration)**: after WS-A/WS-B/WS-C land, perform end-to-end validation:
  - run `pm_lift.py` against representative intake/ADR markdown and a strict Planning Pack,
  - verify error messages and JSON output stability,
  - confirm legacy packs are unaffected.

To reduce merge conflicts, WS-A MUST land first (new files only). WS-C can proceed in parallel with WS-B only if it references the stable `pm_lift` CLI/JSON contract (CONTRACT-3) without requiring implementation details.
