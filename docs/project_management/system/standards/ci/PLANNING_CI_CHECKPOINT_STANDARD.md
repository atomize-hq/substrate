# Planning CI Checkpoint Standard

Goal:
- Reduce redundant multi-OS CI by moving cross-platform validation to **bounded CI checkpoints** between groups of triads, while preserving deterministic, code-grounded safety.

This standard defines:
- the required planning artifact `ci_checkpoint_plan.md`,
- the default min/max triads per checkpoint,
- how to select code-grounded boundaries,
- and the required `tasks.json` wiring so execution is deterministic.

## When this applies

This standard is required for:
- automation-enabled Planning Packs (`tasks.json` meta.schema_version >= 3 and meta.automation.enabled=true), and
- cross-platform packs (`tasks.json` meta.cross_platform=true).

## Why checkpoints (not per-slice CI)

Running full cross-platform CI on every slice tends to be the dominant cost for long features, and often produces redundant “still green” signal.

Checkpoints preserve safety by:
- keeping local integration gates per slice (fmt/clippy/tests + `make integ-checks`),
- and adding explicit, documented “multi-OS sync points” at bounded, code-grounded seams.

## Required artifact

Create:
- `docs/project_management/next/<feature>/ci_checkpoint_plan.md`

Rules:
- Must include a **machine-readable JSON section** that is linted.
- Must partition slices into checkpoint groups with **no overlaps** and **no missing slices**.
- Must name the checkpoint task ids that exist in `tasks.json`.
- Must state which CI gates run at each checkpoint (compile parity vs smoke vs CI testing).

Template:
- `docs/project_management/system/templates/planning_pack/ci_checkpoint_plan.md.tmpl`

## Default bounds (required unless explicitly justified)

Defaults:
- `min_triads_per_checkpoint = 4`
- `max_triads_per_checkpoint = 8`

Exceptions:
- If the total slice count is `< min_triads_per_checkpoint`, a single checkpoint may cover the entire feature.
- If a slice is “high risk” (protocol/schema/FS semantics/platform guards/policy enforcement), a smaller group is allowed **only** when explicitly justified in `ci_checkpoint_plan.md`.

## Selecting checkpoint boundaries (code-grounded)

Use `impact_map.md`, `spec_manifest.md`, and the slice specs to choose boundaries that minimize churn and maximize safety.

Prefer boundaries at:
- **Contract completion seams**: after an end-to-end contract surface is fully defined + implemented + tested (CLI/config/env vars/schema/protocol).
- **Subsystem seams**: before crossing into a new major subsystem (shim ↔ broker ↔ world-agent ↔ world backend).
- **Enabling refactor seams**: after “refactor enabling change” lands, before “new behavior” starts.
- **UX seams**: after a user-visible workflow becomes coherent enough to be validated.

Avoid boundaries that:
- split “schema change” and “schema consumption” across different sides of a checkpoint unless that reduces risk,
- mix multiple unrelated behavioral deltas in the same checkpoint group.

## Execution rules

1) **Per-slice local integration is always required**
   - Integration tasks always run: `cargo fmt`, `cargo clippy ... -D warnings`, relevant tests, and `make integ-checks`.

2) **Cross-platform CI runs only at checkpoints (default)**
   - CI checkpoints run:
     - `make ci-compile-parity ...` (GitHub-hosted cross-platform parity)
     - `make feature-smoke ... PLATFORM=behavior` (self-hosted behavior smoke)
     - CI Testing (`scripts/ci/dispatch_ci_testing.sh`) in `quick` or `full` mode (per plan)

3) **ci_audit remains in use (checkpoint tool)**
   - Run `scripts/ci-audit/ci_audit.sh` inside checkpoint tasks to recommend skip/run and to record evidence.
   - Do not require running `ci_audit` on every slice by default.

## Required `tasks.json` wiring

For each checkpoint group `CPk`:
- Create an ops task `CPk-ci-checkpoint` (or similar) that:
  - depends on the **core integration task** of the checkpoint group’s ending slice (e.g., `WCU3-integ-core`),
  - has a kickoff prompt under `kickoff_prompts/`,
  - references `ci_checkpoint_plan.md`.
- The first slice of the next group must depend on `CPk-ci-checkpoint` (so work cannot proceed past the checkpoint without completing the CI gate).

Kickoff template:
- `docs/project_management/system/templates/kickoff/kickoff_ci_checkpoint.md.tmpl`

## Boundary-only platform-fix (schema v4+; recommended)

To avoid per-slice platform-fix task explosions, schema v4 cross-platform automation packs add:
- `tasks.json` `meta.checkpoint_boundaries`: the slice ids that are the **last slice** in each checkpoint group.

Rules:
- `meta.checkpoint_boundaries` must match the checkpoint group boundaries defined in `ci_checkpoint_plan.md` (mechanically validated by planning lint).
- Only slices listed in `meta.checkpoint_boundaries` may define `*-integ-core` / `*-integ-<platform>` tasks; normal slices use only `X-integ` as their integration merge task.
