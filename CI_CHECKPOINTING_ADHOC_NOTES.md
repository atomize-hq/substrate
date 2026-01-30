# CI Checkpointing (Ad-Hoc Notes)

Status: **draft (implementation in-progress)**

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
- `docs/project_management/next/<feature>/ci_checkpoint_plan.md`

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

