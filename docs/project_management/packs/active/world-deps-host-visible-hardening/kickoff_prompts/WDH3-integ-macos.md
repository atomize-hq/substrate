# Kickoff: WDH3-integ-macos (integration)

## Scope
- macOS parity fixes for CP2 after `WDH3-integ-core`.
- Smoke script: `docs/project_management/packs/active/world-deps-host-visible-hardening/smoke/macos-smoke.sh`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-deps-host-visible-hardening-wdh3-integ-macos` and `.taskmeta.json`.
2. Merge `world-deps-host-visible-hardening-wdh3-integ-core` into this branch.
3. Fix macOS-only failures surfaced by CP2 checkpoint gates (Feature Smoke / CI Testing / compile parity).

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WDH3-integ-macos"`
