# substrate-gateway-boundary-and-runtime-ownership — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/pre-planning/spec_manifest.md`
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for CI cadence.
- If slice ids, platform scope, or checkpoint boundaries change, update this plan first, then update `tasks.json` and kickoff prompts.
- `tasks.json` now defines the accepted slice tasks, the checkpoint ops task, and the checkpoint boundary metadata.
- Any future reordering of slices or checkpoint boundaries must update this plan before the task graph.

## Machine-readable plan (linted)

The machine-readable slice list now uses the accepted five-slice spine from `workstream_triage.md`.
Mechanical checkpoint validation starts after the task graph and prompt artifacts are present.

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
      "slices": ["SGBRO0", "SGBRO1", "SGBRO2", "SGBRO3", "SGBRO4"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single end-of-feature checkpoint after SGBRO4. The accepted spine has five slices, and the checkpoint must verify the full planning graph, the validator-backed prompt/spec support artifacts, and the final task/checkpoint boundary after docs-validation and quality-gate lock-in."
    }
  ]
}
```

## Human-readable rationale

### CP1 (`SGBRO4`) — gateway ownership completion seam

Why this boundary is code-grounded:
- `workstream_triage.md` accepts the five-slice spine `SGBRO0` through `SGBRO4`, and `SGBRO4` is the last accepted slice.
- `spec_manifest.md` now names the same five-slice order and the same planning-support artifacts, so the checkpoint belongs after the task graph has been normalized to that order.
- `impact_map.md` identifies the joined risk seam across planning scaffolding, slice specs, kickoff prompts, and checkpoint wiring; the checkpoint belongs after that seam is coherent.

What surfaces are stabilized at CP1:
- `plan.md` for accepted slice order and checkpoint rules
- `tasks.json` for triad wiring, checkpoint metadata, and kickoff prompt references
- `session_log.md` for planning audit trail
- `quality_gate_report.md` for the accept/flag outcome
- `pre-planning/spec_manifest.md` for the canonical five-slice spine and required planning-support artifacts
- `pre-planning/ci_checkpoint_plan.md` for the one-checkpoint boundary after `SGBRO4`

What risk CP1 reduces:
- drift between the accepted slice order in `workstream_triage.md` and the planning pack files
- drift between `tasks.json` and the validator-required kickoff prompt paths
- drift between the checkpoint plan and the actual checkpoint boundary slice
- drift where slice-spec files are missing, misnamed, or unreferenced by the task graph

Why these gates run at CP1:
- Run the planning validators because the accepted spine and checkpoint boundary are now encoded in machine-readable planning artifacts.
- Keep the checkpoint prompt in the feature-level `kickoff_prompts/` directory so the validator-backed ops task has one canonical entrypoint.
- Keep this as the only checkpoint because the feature pack now has one end-of-feature gate after the final slice.

## Follow-ups

- Keep this plan aligned with `tasks.json` if any future planning change touches the slice order, boundary slice, or checkpoint prompt path.
- Keep the plan aligned with `session_log.md` and `quality_gate_report.md` whenever the checkpoint validation results change.
