# adr-0027-identity-tuple-policy-surface — execution plan

## Scope
- Keep the accepted slice set exactly as `ITPS0`, `ITPS1`, `ITPS2`, `ITPS3`.
- Keep schema v4 triad automation and cross-platform wiring enabled.
- Model the only checkpoint boundary at `ITPS3`, with `CP1-ci-checkpoint` after `ITPS3-integ-core`.
- Keep `ac_ids` sourced directly from each slice spec’s `## Acceptance criteria` section.

## Guardrails
- Planning docs are edited only on the orchestration checkout.
- Slice tasks use canonical kickoff paths under `slices/<SLICE_ID>/kickoff_prompts/`.
- Feature-level ops tasks use canonical kickoff paths under `kickoff_prompts/`.
- `ITPS0` through `ITPS2` use the schema v4 non-boundary shape: `code`, `test`, `integ`.
- `ITPS3` uses the schema v4 checkpoint-boundary shape: `code`, `test`, `integ-core`, `integ-linux`, `integ-macos`, `integ-windows`, `integ`.

## Sequencing
- `ITPS0` starts the pack and closes the contract/schema seam.
- `ITPS1` depends on `ITPS0-integ`.
- `ITPS2` depends on `ITPS1-integ`.
- `ITPS3-code` and `ITPS3-test` depend on `ITPS2-integ`.
- `ITPS3-integ-core` depends on `ITPS3-code` and `ITPS3-test`.
- `CP1-ci-checkpoint` depends on `ITPS3-integ-core`.
- The `ITPS3` platform-fix tasks depend on both `ITPS3-integ-core` and `CP1-ci-checkpoint`.
- `ITPS3-integ` aggregates the core branch plus the three platform-fix branches.
- `FZ-feature-cleanup` runs only after `ITPS3-integ`.

## Checkpoint wiring
- `meta.checkpoint_boundaries = ["ITPS3"]`.
- The checkpoint plan remains authoritative at `pre-planning/ci_checkpoint_plan.md`.
- `CP1-ci-checkpoint` validates the `ITPS3-integ-core` branch SHA and covers Linux, macOS, and Windows behavior scope.
- No earlier slice gets platform-fix fanout or a second checkpoint boundary.

## Execution-gate note
- `meta.execution_gates` is intentionally `false` in the current tracked pack because the required tracked outputs are outside the dispatcher allowlist:
  - `execution_preflight_report.md`
  - `slices/ITPS0/ITPS0-closeout_report.md`
  - `slices/ITPS1/ITPS1-closeout_report.md`
  - `slices/ITPS2/ITPS2-closeout_report.md`
  - `slices/ITPS3/ITPS3-closeout_report.md`
- An allowlist request for those optional execution-gate surfaces is recorded under `logs/pws/ITPS-PWS-tasks_checkpoints/`.

## Owned outputs
- `plan.md`
- `tasks.json`
- `session_log.md`
- `kickoff_prompts/`
- `slices/ITPS0/kickoff_prompts/`
- `slices/ITPS1/kickoff_prompts/`
- `slices/ITPS2/kickoff_prompts/`
- `slices/ITPS3/kickoff_prompts/`
