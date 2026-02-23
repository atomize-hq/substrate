# Work Lift v1 — seam extraction (WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md)

Source: `WORKSTREAM_TRIAGE_AND_LIFT_DECISIONS.md` and `.codex/handoffs/2026-02-22-175724-workstream-triage-lift.md`

This directory contains **seam** artifacts extracted to make the Work Lift v1 rollout owner-assignable and parallelizable without hiding coupling.
These files are planning aids; they are not normative contracts (authoritative contracts remain in the PM system docs + schemas).

- Start here: `scope_brief.md`
- Seam overview: `seam_map.md`
- Threading (contracts + dependencies + workstreams): `threading.md`
- Execution plan (threaded decompositions): `threaded-seams/README.md`

## Reading order (canonical navigation)

1. `scope_brief.md` — what “Work Lift v1” is and what is explicitly out-of-scope.
2. `seam_map.md` — what seams exist and why they were cut.
3. `threading.md` — canonical contract IDs + dependency edges + workstreams.
4. `threaded-seams/README.md` — per-seam `seam.md` + `slice-*.md` files (the executable slice/task breakdowns).

Notes:
- The **threaded seam slices** (`threaded-seams/**/slice-*.md`) are the canonical execution plan artifacts for implementers.
- When a doc references a `CONTRACT-*`, treat `threading.md` as the canonical registry for the exact artifact path and the producing/consuming seams.
