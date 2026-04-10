# Planning CI Checkpoint Standard

Goal:
- reduce redundant multi-platform verification by identifying bounded checkpoint seams during FSE pre-planning,
- while keeping the pre-planning artifact advisory rather than execution-wired.

This standard defines:
- the required planning artifact `ci_checkpoint_plan.md`,
- the default bounds for candidate groups per checkpoint,
- how to select code-grounded boundaries,
- and what the artifact must say before downstream planning turns it into concrete execution behavior.

## When this applies

This standard is required when:
- the feature has cross-platform behavior,
- verification is expensive enough that explicit cadence matters,
- or the impact map identifies high-risk seams that need deliberate multi-platform sync points.

## Why checkpoints

Running the heaviest verification after every future slice or seam is usually wasteful.

Checkpoint planning preserves safety by:
- identifying where multi-platform confirmation will matter later,
- recording why those boundaries exist,
- and keeping the checkpoint contract visible before downstream execution planning starts.

## Required artifact

Create:
- `docs/project_management/packs/<bucket>/<feature>/pre-planning/ci_checkpoint_plan.md`

Rules:
- Must include a machine-readable JSON section.
- Must partition draft candidates into checkpoint groups with no overlaps and no unexplained omissions.
- Must state which verification gates are intended at each checkpoint.
- Must remain advisory at the pre-planning stage. It does not name task IDs or execution ownership.

Template:
- `docs/project_management/system/fse/templates/planning_pack/ci_checkpoint_plan.md.tmpl`

## Default bounds

Defaults:
- `min_candidates_per_checkpoint = 2`
- `max_candidates_per_checkpoint = 6`

Exceptions:
- If the total candidate count is below the minimum, a single checkpoint may cover the whole feature.
- If a candidate is high-risk, a smaller group is allowed when explicitly justified in `ci_checkpoint_plan.md`.

## Selecting checkpoint boundaries

Use `impact_map.md`, `spec_manifest.md`, and `minimal_spec_draft.md` to choose boundaries that minimize churn and maximize safety.

Prefer boundaries at:
- contract completion seams,
- subsystem seams,
- enabling-refactor seams,
- operator-UX seams,
- platform-divergence seams.

Avoid boundaries that:
- separate a schema change from the first place that consumes it unless the risk reduction is explicit,
- mix unrelated behavioral deltas into one checkpoint without a reason.

## Expected contents

Each checkpoint must state:
- the draft candidates it covers,
- the intended verification gates,
- the reason that boundary is code-grounded,
- what surfaces are stabilized by the checkpoint,
- what uncertainty still remains for downstream planning.

Typical gates:
- compile parity,
- feature smoke,
- deeper CI testing,
- targeted platform validation.

## Downstream handoff

The checkpoint plan feeds downstream planning and decomposition by:
- giving a first-pass cadence,
- highlighting where platform confirmation matters,
- surfacing unresolved platform-scope questions.

It does not by itself:
- create tasks,
- create kickoff prompts,
- define checkpoint task IDs,
- or assign execution ownership.
