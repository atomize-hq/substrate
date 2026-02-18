# Kickoff: WDH3-integ-core (integration)

## Scope
- Merge WDH3 code+test and make the slice green on the primary dev platform before CP2 checkpoint CI.
- Spec: `docs/project_management/packs/active/world-deps-host-visible-hardening/WDH3-spec.md`
- CI cadence: `docs/project_management/packs/active/world-deps-host-visible-hardening/ci_checkpoint_plan.md`

## Start Checklist
Do not edit planning docs inside the worktree.

1. Verify worktree `wt/world-deps-host-visible-hardening-wdh3-integ-core` and `.taskmeta.json`.
2. Merge the WDH3 code+test branches if needed.
3. Run: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make integ-checks`.
4. If possible, run local smoke for your platform (slice-scoped):
   - Linux: `SUBSTRATE_SMOKE_SLICE_ID=WDH3 bash docs/project_management/packs/active/world-deps-host-visible-hardening/smoke/linux-smoke.sh`
   - macOS: `SUBSTRATE_SMOKE_SLICE_ID=WDH3 bash docs/project_management/packs/active/world-deps-host-visible-hardening/smoke/macos-smoke.sh`

## End Checklist
1. From inside the worktree: `make triad-task-finish TASK_ID="WDH3-integ-core"`
2. Hand off the HEAD SHA to the operator for CP2 checkpoint dispatch.
