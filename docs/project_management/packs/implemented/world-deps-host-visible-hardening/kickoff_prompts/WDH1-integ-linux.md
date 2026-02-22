# Kickoff: WDH1-integ-linux (integration)

## Scope
- Linux (and bundled WSL) parity fixes for CP1 after `WDH1-integ-core`.
- Smoke script: `docs/project_management/packs/active/world-deps-host-visible-hardening/smoke/linux-smoke.sh`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-deps-host-visible-hardening-wdh1-integ-linux` and `.taskmeta.json`.
2. Merge `world-deps-host-visible-hardening-wdh1-integ-core` into this branch.
3. Fix Linux/WSL-only failures surfaced by CP1 checkpoint gates (Feature Smoke / CI Testing / compile parity).

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WDH1-integ-linux"`
