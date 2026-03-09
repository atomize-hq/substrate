# add-non-apt-system-package-provisioning-support — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/workstream_triage.md`
- Slice specs: see `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md` for the canonical slice IDs `NASP0`, `NASP1`, `NASP2`, `NASP3`, and `NASP4`.

## Operator rules
- This plan is authoritative for **CI cadence** during planning and execution wiring.
- If slice IDs, platform scope, or subsystem seams change during full planning, update this plan first, then update `tasks.json` and kickoff prompts.
- The accepted slice order is `NASP0`, `NASP1`, `NASP2`, `NASP3`, `NASP4`.
- This pack uses schema-v4 checkpoint-boundary wiring:
  - checkpoint group 1 ends at `NASP2`
  - checkpoint group 2 ends at `NASP4`
- `tasks.json` MUST mirror this file with:
  - `meta.checkpoint_boundaries = ["NASP2", "NASP4"]`
  - checkpoint ops tasks `CP1-ci-checkpoint` and `CP2-ci-checkpoint`
  - boundary-only platform-fix integration tasks for `NASP2` and `NASP4`

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 2,
    "max_triads_per_checkpoint": 3
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["NASP0", "NASP1", "NASP2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Checkpoint after NASP2 closes the first contiguous seam where world-manager probe, pacman schema support, and provisioning routing converge into one end-to-end manager-aware provisioning path."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["NASP3", "NASP4"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Checkpoint after NASP4 closes the remaining contiguous seam where runtime fail-early behavior, platform parity evidence, smoke coverage, and reconciliation updates settle the full feature contract."
    }
  ]
}
```

## Human-readable rationale

### CP1 — `NASP0` through `NASP2`
- Boundary type:
  - First contiguous checkpoint group across the accepted slice order.
- Why this boundary is code-grounded:
  - `NASP0` stabilizes the in-world probe and support gate across `world_enable`, dispatch, and `world-agent`.
  - `NASP1` isolates schema and inventory-view changes from provisioning execution.
  - `NASP2` consumes the probe and schema contract to drive provisioning routing, request-profile use, and pacman execution.
- What surfaces are stabilized here:
  - Shared manager-aware `--provision-deps` contract through the provisioning path.
  - Inventory/schema support for `install.method=pacman` and `install.pacman`.
- What risk is reduced by running cross-platform CI here:
  - Prevents a false green on the provisioning seam before the first cross-platform compile and smoke signal is captured.
  - Forces the probe, schema, and provisioning-routing changes to converge before runtime remediation work starts in the next slice group.

### CP2 — `NASP3` through `NASP4`
- Boundary type:
  - Final contiguous checkpoint group across the accepted slice order.
- Why this boundary is code-grounded:
  - `NASP3` locks the runtime fail-early contract, explicit-item scoping, and manager-aware remediation wording.
  - `NASP4` closes the platform-parity spec, smoke/manual validation evidence, and contract-reconciliation targets needed for release-quality behavior.
- What surfaces are stabilized here:
  - Runtime no-system-package-mutation behavior for `deps current sync|install`.
  - Platform parity evidence across Linux, macOS, and Windows, including supported, unsupported, mismatch, and explicit-item-scope paths.
  - Upstream doc reconciliation targets for the shared manager-aware contract.
- What risk is reduced by running cross-platform CI here:
  - Prevents runtime wording, smoke coverage, and docs reconciliation from drifting after provisioning is already green.
  - Ensures the final slice group ships with one coherent cross-platform validation story instead of a code-only closure.

## Gate cadence
- Compile parity:
  - Run `make ci-compile-parity ...` at both `CP1-ci-checkpoint` and `CP2-ci-checkpoint`.
  - Reason: both checkpoint groups change shell, world-enable, dispatch, docs, and validation surfaces that must stay green on all CI parity platforms.
- Feature smoke:
  - Run `make feature-smoke ... PLATFORM=behavior` at both checkpoint tasks.
  - Reason: `tasks.json` requires behavior validation on `linux`, `macos`, and `windows`, and both checkpoint groups end at cross-platform behavior boundaries.
- CI testing:
  - Run `scripts/ci/dispatch_ci_testing.sh --mode full` at both checkpoint tasks.
  - Reason: each checkpoint seals a contiguous slice group whose branch tip must be validated before the next phase or feature cleanup proceeds.
- CI audit:
  - Run `scripts/ci-audit/ci_audit.sh` inside the checkpoint task before dispatching cross-platform gates.
  - Record skip/run evidence with the checkpoint task outputs.

## `tasks.json` mapping
- `tasks.json` MUST define:
  - non-boundary triads for `NASP0`, `NASP1`, and `NASP3`
  - boundary triads for `NASP2` and `NASP4` using `*-integ-core`, `*-integ-linux`, `*-integ-macos`, `*-integ-windows`, and final `*-integ`
  - checkpoint ops tasks `CP1-ci-checkpoint` and `CP2-ci-checkpoint`
  - `meta.checkpoint_boundaries = ["NASP2", "NASP4"]`
  - gating dependencies so `NASP3-code` and `NASP3-test` depend on `CP1-ci-checkpoint`
  - `FZ-feature-cleanup` after `NASP4-integ` and `CP2-ci-checkpoint`
- Keep `meta.ci_parity_platforms_required` as `["linux", "macos", "windows"]`.
- Keep `meta.behavior_platforms_required` as `["linux", "macos", "windows"]` unless the accepted platform parity spec narrows behavior scope explicitly.

## Follow-ups
- Keep this plan and `tasks.json` synchronized whenever checkpoint grouping changes.
- Do not add platform-fix tasks to non-boundary slices in schema v4.
- Re-run:
  - `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
