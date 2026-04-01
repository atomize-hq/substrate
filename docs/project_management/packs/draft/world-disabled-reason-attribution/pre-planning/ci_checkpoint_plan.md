# world-disabled-reason-attribution — CI checkpoint plan

This file defines when cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/minimal_spec_draft.md`
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`, `macos`, `windows`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for CI cadence.
- If slice ids or checkpoint boundaries change, update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4 cross-platform packs, `tasks.json` `meta.checkpoint_boundaries` lists the last slice in each checkpoint group.

## Machine-readable plan (linted)

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
      "slices": ["WDRA0", "WDRA1", "WDRA2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single end-of-feature checkpoint after WDRA2. Total slices is 3, which is below defaults.min_triads_per_checkpoint. CP1 validates shared disable-attribution semantics, replay stderr copy, replay_strategy telemetry, docs alignment, and cross-platform parity across Linux, macOS, and Windows."
    }
  ]
}
```

## Human-readable rationale

### CP1 (`WDRA0..WDRA2`) — replay attribution completion seam

Why this boundary is code-grounded:
- The full behavior contract is not coherent until the helper seam, replay stderr surfaces, telemetry fields, tests, and docs all land together.
- The pack has 3 slices. One checkpoint is valid under the checkpoint standard when total slices is below the default minimum.

What surfaces are stabilized at CP1:
- `contract.md`
- `telemetry-spec.md`
- `platform-parity-spec.md`
- `slices/WDRA0/WDRA0-spec.md`
- `slices/WDRA1/WDRA1-spec.md`
- `slices/WDRA2/WDRA2-spec.md`
- `manual_testing_playbook.md`
- `smoke/linux-smoke.sh`
- `smoke/macos-smoke.sh`
- `smoke/windows-smoke.ps1`

What risk CP1 reduces:
- multi-platform drift in replay stderr copy or trace fields
- redaction regressions that leak absolute paths or raw env values
- docs drift across `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/COMMANDS.md`
