# adr-0027-identity-tuple-policy-surface — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature during pre-planning.

Standard:

- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs

- Feature directory: `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/minimal_spec_draft.md`

## Operator rules

- This plan is authoritative for checkpoint cadence during pre-planning.
- If slice ids, platform scope, or checkpoint boundaries change, update this plan first.
- `tasks.json` already carries the required schema v4 automation baseline and the full Linux, macOS, and Windows CI-parity and behavior-platform scope for this draft pack.
- Full planning must add `CP1-ci-checkpoint` and set `meta.checkpoint_boundaries = ["ITPS1"]` after slice tasks exist.
- Mechanical validation is deferred until `tasks.json` contains real slice integration tasks and the checkpoint ops task.

## Applicability

- Checkpoint planning applies because the authoritative inputs define one cross-platform tuple-policy contract across Linux, macOS, and Windows.
- `impact_map.md` ties the feature to broker policy parsing, shell denial and explain surfaces, trace publication, and operator documentation parity.
- `spec_manifest.md` keeps the feature on two contiguous slices that together close the contract, schema, policy, telemetry, compatibility, and validation surfaces.

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
      "slices": ["ITPS0", "ITPS1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "The draft slice set contains two slices only, so one checkpoint is valid under the minimum-size exception. ITPS0 locks the tuple-axis contract and schema surface. ITPS1 closes policy evaluation, telemetry publication, compatibility posture, and validation closure. Cross-platform validation after ITPS1 covers the completed broker, shell, trace, and operator-contract seam in one pass."
    }
  ]
}
```

## Human-readable rationale

### CP1 (`ITPS0` through `ITPS1`)

- Code-grounded boundary:
  - `ITPS0` owns the additive `llm.constraints.*` contract and schema surface, the authoritative explain surface, and naming and precedence rules.
  - `ITPS1` owns ordered policy evaluation, tuple-axis deny semantics, telemetry publication, compatibility guarantees, and validation closure.
  - The completed seam crosses `crates/broker`, `crates/shell`, `crates/trace`, and the cross-platform operator contract surfaces named in `impact_map.md`.
- Stabilized surfaces:
  - tuple-axis contract and schema publication
  - authoritative policy inspection and explain wording
  - backend-gate plus tuple-axis evaluation ordering
  - tuple-aware allow and deny telemetry publication
  - backend-id compatibility boundary
  - Linux, macOS, and Windows validation closure
- Risk reduced at this checkpoint:
  - contract drift between policy docs and effective policy output
  - deny-taxonomy drift across broker and shell surfaces
  - trace-field drift against the selected tuple vocabulary
  - platform-specific parity regressions in the final operator story
- Gate selection:
  - `compile_parity`: run at `CP1` because the feature touches shared broker, shell, and trace surfaces.
  - `feature_smoke`: run at `CP1` because the feature claims identical tuple-axis policy behavior on Linux, macOS, and Windows.
  - `ci_testing = "full"`: run at `CP1` because this is the only checkpoint and it closes the full contract-to-behavior seam.

## Follow-ups

- Add real slice tasks to `tasks.json` for `ITPS0` and `ITPS1`.
- Add `CP1-ci-checkpoint` to `tasks.json` with dependencies anchored to the ending slice integration task.
- Set `tasks.json` `meta.checkpoint_boundaries = ["ITPS1"]`.
- Replace the draft slice ids in the machine-readable JSON if full planning renames or splits the current slices.
- Run `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface"` after the real slice tasks and checkpoint task exist.
