# add-non-apt-system-package-provisioning-support — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/minimal_spec_draft.md`
- Slice specs: see `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` for the canonical slice IDs `NASP0`, `NASP1`, `NASP2`, `NASP3`, and `NASP4`.

## Operator rules
- This plan is authoritative for **CI cadence** during planning and execution wiring.
- If slice IDs, platform scope, or subsystem seams change during full planning, update this plan first, then update `tasks.json` and kickoff prompts.
- This is a pre-planning first pass. The machine-readable JSON below uses the accepted slice order from `pre-planning/workstream_triage.md` because `tasks.json` does not yet define slice tasks.
- Do not claim mechanical validity yet. After slice tasks exist, wire `CP1-ci-checkpoint`, set `tasks.json` `meta.checkpoint_boundaries`, and run the validator.

## Machine-readable plan (draft; not yet mechanically validated)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 4,
    "max_triads_per_checkpoint": 8
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["NASP0", "NASP1", "NASP2", "NASP3", "NASP4"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Draft single-checkpoint grouping across NASP0-NASP4. The accepted slice order now spans five contiguous slices, and the probe contract, schema boundary, provisioning wiring, runtime fail-early behavior, and validation/doc reconciliation still converge into one end-to-end manager-aware workflow."
    }
  ]
}
```

## Human-readable rationale

### CP1 — `NASP0` through `NASP4`
- Boundary type:
  - Whole-feature checkpoint across the accepted contiguous slice order.
- Why this boundary is code-grounded:
  - `NASP0` stabilizes the in-world probe and support gate across `world_enable`, dispatch, and `world-agent`.
  - `NASP1` isolates schema and inventory-view changes from provisioning execution.
  - `NASP2` consumes the probe and schema contract to drive provisioning routing, request-profile use, and pacman execution.
  - `NASP3` locks the runtime fail-early contract and explicit-item scoping.
  - `NASP4` closes the contract-reconciliation, platform-parity, and manual/smoke evidence seam needed to validate the full manager-aware workflow.
- What surfaces are stabilized here:
  - Shared manager-aware `--provision-deps` contract and runtime no-system-package-mutation posture.
  - Inventory/schema support for `install.method=pacman` and `install.pacman`.
  - Platform parity evidence across Linux, macOS, and Windows, including supported, unsupported, and mismatch paths.
- What risk is reduced by running cross-platform CI here:
  - Prevents a false green on a partial seam where the probe contract exists but the provisioning path or runtime remediation is still incomplete.
  - Forces the compile/runtime/docs evidence to converge once the world-manager probe, pacman schema, and fail-early wording all interact.
  - Avoids redundant multi-OS dispatch before the feature exposes one coherent end-to-end behavior to validate.

## Gate cadence
- Compile parity:
  - Run `make ci-compile-parity ...` at `CP1-ci-checkpoint`.
  - Reason: `impact_map.md` shows shell, world-enable, dispatch, and `world-agent` code changes that must compile on all CI parity platforms.
- Feature smoke:
  - Run `make feature-smoke ... PLATFORM=behavior` at `CP1-ci-checkpoint`.
  - Reason: `tasks.json` currently requires behavior validation on `linux`, `macos`, and `windows`, and the accepted slice order now ends the validation-evidence seam at `NASP4`.
- CI testing:
  - Run `scripts/ci/dispatch_ci_testing.sh --mode full` at `CP1-ci-checkpoint`.
  - Reason: this checkpoint covers the full feature seam, including contract/schema/runtime behavior and cross-platform docs evidence, so the first-pass conservative mode is `full`.
- CI audit:
  - Run `scripts/ci-audit/ci_audit.sh` inside the checkpoint task before dispatching cross-platform gates.
  - Record skip/run evidence with the checkpoint task outputs.

## `tasks.json` mapping
- Current status:
  - `tasks.json` already satisfies the schema-v4 automation and cross-platform metadata baseline.
  - `tasks.json` does not yet define slice tasks or checkpoint tasks, so no staged `tasks.json` change is required in this pre-planning pass.
- Full-planning target:
  - Add `CP1-ci-checkpoint`.
  - When slice tasks are created, set `meta.checkpoint_boundaries` to `["NASP4"]` unless checkpoint grouping changes.
  - Keep `meta.ci_parity_platforms_required` as `["linux", "macos", "windows"]`.
  - Keep `meta.behavior_platforms_required` as `["linux", "macos", "windows"]` unless the accepted platform parity spec narrows behavior scope explicitly.

## Follow-ups
- Ensure slice ids in `tasks.json` match the draft slice skeleton, or update both to the accepted ids.
- Replace any remaining placeholder or draft-only slice ids in the plan JSON with real slice ids once slice tasks exist.
- Set `tasks.json` `meta.checkpoint_boundaries` to match the accepted checkpoint boundaries.
- Add the `CP1-ci-checkpoint` task with dependencies on the ending slice integration task for the accepted boundary slice.
- If full planning adopts the triage recommendation to split checkpoint wiring across `NASP2` and `NASP4`, regroup checkpoints contiguously before wiring dependencies.
- Then run:
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
