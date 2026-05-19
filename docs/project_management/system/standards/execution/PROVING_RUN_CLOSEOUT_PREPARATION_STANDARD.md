# Proving-Run Closeout Preparation Standard

Goal:
- Eliminate freehand create-lane closeout authoring by generating a repo-owned `proving-run-closeout.json` draft from lifecycle and publication truth, then leaving only the minimum human-owned fields for completion.

## Canonical lifecycle states
- `published_baseline` is the only legal source state for closeout preparation.
- `closed_baseline` is the terminal target state after closeout is completed.
- A create-mode proving run must not skip directly from green publication to `closed_baseline` without a prepared closeout artifact.

## Source-of-truth inputs
- Machine-owned lifecycle/publication facts live in one JSON object with:
  - `schema_version = 1`
  - `mode = "create"`
  - `run_id`
  - `lifecycle.current_state = "published_baseline"`
  - `publication.status = "green"`
  - `publication.published_at`
  - `publication.artifact_path`
- Optional human inputs may be supplied separately and merged during preparation.

## Ownership split
- Machine-owned fields:
  - lifecycle source state and target state
  - publication status and publication timestamps
  - publication artifact path and evidence refs
  - the full input facts snapshot
  - preparation transaction metadata and input file hashes
- Human-owned fields:
  - `human_owned.residual_friction`
  - `human_owned.manual_edits`
  - `human_owned.operator_notes`
  - `human_owned.follow_ups`

## Preparation flow
- Run `docs/project_management/system/scripts/execution/prepare_proving_run_closeout.py`.
- The script must:
  - fail closed unless publication is green and lifecycle state is `published_baseline`
  - write `proving-run-closeout.json` atomically
  - preserve machine-owned truth exactly
  - scaffold missing human-owned fields as `null` or empty arrays
  - record `handoff.status = "awaiting_human_inputs"` until the required human fields are present
  - record `handoff.status = "ready_to_close"` only when `residual_friction` and `manual_edits` are explicitly supplied

## Output contract
- Canonical schema: `docs/project_management/system/schemas/proving_run_closeout.schema.json`
- Canonical script: `docs/project_management/system/scripts/execution/prepare_proving_run_closeout.py`
- Default output filename: `proving-run-closeout.json`
