# Seam Map — Work Lift v1

Primary extraction axis: **workflow-first** (intake/ADR → compute lift → triage/splitting → plan slices/checkpoints), with an integration-focused pass for contracts/config.

## Capability inventory (no seams yet)

- Lift Vector v1 definition (fields + meaning) with discovery-time `null` semantics
- Lift Score v1 formula + risk multipliers + rounding
- Split triggers v1 (ADR vs workstream)
- Lift→slices→checkpoints mapping (time-free)
- Directory/prefix handling for Impact Map Touch Sets (raw counts + optional deterministic expansion)
- Machine-readable artifacts (schema + model config) and human rubric docs
- `pm_lift.py` computation tool (intake, impact_map, git-diff calibration) + stable JSON output
- Advisory-first integration into planning workflow (Makefile/docs guidance; strict gating deferred)

## Seams (final)

### SEAM-1 — Lift Vector schema + human rubric
- Type: **integration**
- Value: makes “fill vector” and “read vector” deterministic and consistent; produces authoritative docs for humans/agents.
- Output artifacts: schema + rubric doc.

### SEAM-2 — Lift model config v1 (weights/triggers/versioning)
- Type: **integration**
- Value: makes scoring tunable without code edits; creates a single machine-readable source of truth for weights and thresholds.
- Output artifacts: versioned model config JSON.

### SEAM-3 — `pm_lift` core engine + stable output contract
- Type: **capability**
- Value: deterministic computation from (a) embedded vector block and (b) derived counts, producing stable CLI + JSON outputs for downstream tooling.

### SEAM-4 — Pack-derived lift inputs (Impact Map + prefix expansion)
- Type: **capability**
- Value: compute lift from Planning Pack `impact_map.md` using existing validation output; handle directory prefixes safely (discount/cap + confidence).

### SEAM-5 — Advisory-first workflow integration (+ strict-mode onramp)
- Type: **platform**
- Value: makes lift usable in real planning flow without breaking legacy; defines (but does not prematurely enforce) strict-mode gates and rollout steps.

## Seam briefs

- `seam-1-lift-vector-schema-and-rubric.md`
- `seam-2-lift-model-config-v1.md`
- `seam-3-pm-lift-core-engine.md`
- `seam-4-pack-derived-lift-inputs.md`
- `seam-5-advisory-workflow-integration.md`

## Threaded decompositions (execution plan)

The seam briefs above are high-level overviews. The canonical execution plan artifacts live under:

- `threaded-seams/README.md` (per-seam `seam.md` + `slice-*.md` breakdowns)
