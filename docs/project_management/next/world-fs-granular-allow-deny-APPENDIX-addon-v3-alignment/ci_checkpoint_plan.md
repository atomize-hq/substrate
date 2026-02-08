# world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/impact_map.md`
- `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/spec_manifest.md`
- Slice specs: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/*-spec*.md`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (new slice added, new platform scope, new contract surface), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted).

## Machine-readable plan (linted)

```json
{
  "version": 1,
  "defaults": {
    "min_triads_per_checkpoint": 2,
    "max_triads_per_checkpoint": 4
  },
  "checkpoints": [
    {
      "id": "CP1",
      "task_id": "CP1-ci-checkpoint",
      "slices": ["WFGADAXA0", "WFGADAXA1", "WFGADAXA2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Single bounded checkpoint after all add-on slices land (3 triads): the work is cross-cutting (broker+shell+agent-api-types+world-agent+docs) and version-lockstep; running cross-platform compile parity + feature smoke once at the end provides maximum signal with minimal repeated dispatch."
    }
  ]
}
```

## Human-readable rationale (required)

For each checkpoint, explain:
- Why the boundary is code-grounded (subsystem seam / contract completion / enabling refactor / UX seam).
- What surfaces are stabilized by this checkpoint (from `spec_manifest.md`).
- What risk is reduced by running cross-platform CI here (from `impact_map.md`).
