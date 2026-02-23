# Threaded Seams — Work Lift v1 (execution plan)

This directory contains the **threaded decompositions** for each seam: a `seam.md` overview plus `slice-*.md` documents with the executable slice/task breakdown.

Canonical navigation:

- Contract registry / canonical contract paths: `../threading.md`
- Seam overview (non-threaded briefs): `../seam_map.md`

## Seams (threaded)

- SEAM-1 — Lift Vector schema + human rubric
  - `seam-1-lift-vector-schema-and-rubric/seam.md`
- SEAM-2 — Lift model config v1 (weights/triggers/versioning)
  - `seam-2-lift-model-config-v1/seam.md`
- SEAM-3 — `pm_lift` core engine + stable output contract
  - `seam-3-pm-lift-core-engine/seam.md`
- SEAM-4 — Pack-derived lift inputs (Impact Map + prefix expansion)
  - `seam-4-pack-derived-lift-inputs/seam.md`
- SEAM-5 — Advisory-first workflow integration (+ strict-mode onramp)
  - `seam-5-advisory-workflow-integration/seam.md`

## How to read a threaded seam

1. Open the seam’s `seam.md` to see scope, owned/consumed contracts, and the slice index.
2. Execute slices in index order (`slice-1-*`, `slice-2-*`, …).

If a threaded slice references a contract by ID (e.g., `CONTRACT-2:work_lift_model_v1`), use `../threading.md` as the canonical registry for the artifact path and the producer/consumer seams.
