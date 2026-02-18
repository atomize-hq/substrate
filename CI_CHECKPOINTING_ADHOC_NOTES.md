# CI Checkpointing (Ad-Hoc Notes)

Status: **implemented (CI checkpoints + boundary-only platform-fix + single wrapper)**

This document captures an operator decision made during an interactive planning-system iteration, so we do not lose context.

## Problem

Cross-platform CI dispatch (compile parity, Feature Smoke, CI Testing) has become too frequent when executing long features with many slices/triads.

Today, the default integration kickoff guidance effectively encourages cross-platform CI to be run per slice/triad, which creates:
- High latency (many redundant CI runs).
- High operational load (runners, queue time, repeated triage).
- Slower iteration despite the workflow being “autonomous”.

## Decision

Adopt **CI checkpoints**: run cross-platform CI at **bounded checkpoints** between groups of triads, rather than per triad.

Key properties:
- CI checkpoints are **explicit planning artifacts** (not ad-hoc operator judgment).
- Checkpoints are chosen using **code-grounded boundaries** (subsystem seams, contract completion points) plus mechanical bounds.
- Per-slice local integration gates remain mandatory (fmt/clippy/tests + `make integ-checks`), but cross-platform CI is not dispatched for every slice.

### Default bounds (non-negotiable defaults)

- Default **minimum** triads per checkpoint: **2**
- Default **maximum** triads per checkpoint: **4**

Notes:
- The plan must still be deterministic for very small features. If the total slice count is `< 2`, a single checkpoint may cover the entire feature.
- High-risk seams may justify earlier checkpoints even if it produces smaller groups (explicitly documented in the checkpoint plan).

## New planning artifact

Add a required planning-pack document:
- `docs/project_management/_archived/next/<feature>/ci_checkpoint_plan.md`

This document:
- Partitions the feature’s slices into checkpoint groups.
- Names the checkpoint task(s) that run CI.
- Defines which CI gates run at each checkpoint (compile parity vs smoke vs CI testing).
- Records the rationale for each checkpoint boundary.

The plan must include a machine-readable section, and lint should validate:
- Every slice belongs to exactly one checkpoint group.
- Each group size respects min/max defaults (except the “total slices < min” case).
- The checkpoint tasks referenced by the plan exist in `tasks.json`.

## Execution behavior

- **CI audit (`scripts/ci-audit/ci_audit.sh`) remains in use**, but it becomes a **checkpoint tool**:
  - Run it inside checkpoint tasks to decide skip/run and to record evidence (ledger + run ids).
  - Do not run it as a default “every slice” requirement.

## Implementation outline (repo changes)

1) Add `PLANNING_CI_CHECKPOINT_STANDARD.md` and a `ci_checkpoint_plan.md` template.
2) Update planning prompts to require `ci_checkpoint_plan.md` and to use it when building tasks/kickoffs.
3) Introduce a checkpoint task template (`kickoff_ci_checkpoint.md.tmpl`) and scaffold at least one checkpoint task.
4) Update integration kickoff templates to:
   - Always run local integration gates.
   - Dispatch cross-platform CI only when the checkpoint plan says this slice is a checkpoint.
5) Update planning lint to fail when checkpoint plan and tasks drift (missing tasks, missing coverage, bounds violations).

## Implemented (CI checkpoints)

The following items are implemented in the repo:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`
- `docs/project_management/standards/templates/ci_checkpoint_plan.md.tmpl`
- `docs/project_management/standards/templates/kickoff_ci_checkpoint.md.tmpl`
- `docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py`
- Planning lint requires and validates `ci_checkpoint_plan.md` for automation-enabled cross-platform packs.
- Integration kickoff templates explicitly remove “run cross-platform CI per slice” defaults (CI is a checkpoint-only activity).
- Cross-platform dispatch scripts support validating an exact commit (checkout-ref) so checkpoints can validate the merged state deterministically.

## Implemented: boundary-only platform-fix + single wrapper

### Problem (follow-on)

Even with checkpoints, a cross-platform pack currently encourages per-slice cross-platform task fan-out:
- `<slice>-integ-core`
- `<slice>-integ-<platform>`
- `<slice>-integ` (final aggregator)

That “platform-fix task explosion” is the slow part for many-triad features and doesn’t match the intent of checkpoints.

### Decision

Adopt **boundary-only platform-fix**:
- For **normal slices**: only `X-code`, `X-test`, `X-integ` exist (single per-slice integration merge task).
- For **checkpoint-boundary slices only**: full cross-platform structure exists:
  - `B-integ-core`
  - `B-integ-<platform>` (for each CI parity platform; plus WSL if required/separate)
  - `B-integ` (final aggregator)

### Detection primitive (machine-readable)

Add `tasks.json` meta:
- `meta.checkpoint_boundaries`: array of slice ids that are **the last slice** in each checkpoint group.

Rules:
- `meta.checkpoint_boundaries` must match `ci_checkpoint_plan.md` boundaries (lint/validation enforced).
- Only slices in `meta.checkpoint_boundaries` may define `*-integ-core` / `*-integ-<platform>` tasks.
- Code/test tasks’ `integration_task` must point to:
  - `X-integ` for normal slices
  - `B-integ-core` for boundary slices

### Operator UX: single wrapper entrypoint

Add a single automation entrypoint to run a slice “start → complete” end-to-end:
- `make triad-task-start-complete FEATURE_DIR="docs/project_management/_archived/next/<feature>" SLICE_ID="<slice>"`

Requirements:
- Wrapper runs from the orchestration checkout and uses Codex-enabled automation internally (no extra flags required in the common case).
- Wrapper writes a deterministic log + summary under `{{FEATURE_DIR}}/logs/<slice>/wrapper/` rather than only printing to stdout.
- Wrapper selects the correct per-slice integration merge task dynamically based on `tasks.json` (via the code/test task’s `integration_task` field).

Notes:
- CI checkpoint tasks (e.g. `CPk-ci-checkpoint`) remain explicit ops tasks; this wrapper’s primary responsibility is the slice’s code/test/merge closure.
