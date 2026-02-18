# Kickoff: WDH1-integ-windows (integration)

## Scope
- Windows CI parity fixes for CP1 after `WDH1-integ-core`.
- Windows must build without errors; Feature Smoke is not required on Windows for this feature.

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-deps-host-visible-hardening-wdh1-integ-windows` and `.taskmeta.json`.
2. Merge `world-deps-host-visible-hardening-wdh1-integ-core` into this branch.
3. Fix Windows-only compile/test parity issues surfaced by CP1 checkpoint gates.

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WDH1-integ-windows"`
