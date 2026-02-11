# world-sync — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-sync`
- `docs/project_management/next/world-sync/impact_map.md`
- `docs/project_management/next/world-sync/spec_manifest.md`
- Slice specs: `docs/project_management/next/world-sync/*-spec*.md`
- Required platforms (authoritative):
  - Behavior smoke platforms: `tasks.json` → `meta.behavior_platforms_required` (`linux`, `macos`)
  - CI parity platforms: `tasks.json` → `meta.ci_parity_platforms_required` (`linux`, `macos`)

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
      "slices": ["WS0", "WS1", "WS2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after the first end-to-end manual sync apply seam: CLI + pending diff discovery + non-PTY from_world apply + safety rails. This is the earliest slice where behavioral smoke can assert a real user workflow."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WS3", "WS4", "WS5"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after lifecycle + PTY + direction expansion seam: auto-sync trigger, PTY pending diff discovery, and host→world pre-sync plus both-direction semantics. This stabilizes the bidirectional contract before internal checkpoint/rollback is added."
    },
    {
      "id": "CP3",
      "task_id": "CP3-ci-checkpoint",
      "slices": ["WS6", "WS7"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Final checkpoint after internal git checkpoint + rollback seam. This is a safety-critical mutation surface (restore semantics), so CI testing runs in full mode."
    }
  ]
}
```

## Human-readable rationale (required)

For each checkpoint, explain:
- Why the boundary is code-grounded (subsystem seam / contract completion / enabling refactor / UX seam).
- What surfaces are stabilized by this checkpoint (from `spec_manifest.md`).
- What risk is reduced by running cross-platform CI here (from `impact_map.md`).

### CP1 — pending diff discovery + non-PTY from_world apply seam
- Code-grounded boundary:
  - Completes the “manual apply” workflow for non-PTY sessions: discovery → preview → apply.
- Stabilized surfaces:
  - `contract.md` (workspace sync CLI and exit codes)
  - `filesystem-semantics-spec.md` (protected paths, exclude matching, size guards)
- Risk reduced:
  - Ensures we do not regress safety rails or accidentally mutate protected paths across platforms; validates basic UX output.

### CP2 — lifecycle/PTY + direction expansion seam
- Code-grounded boundary:
  - Completes “session lifecycle” behavior: auto-sync policy + PTY pending-diff visibility + bidirectional semantics.
- Stabilized surfaces:
  - `contract.md` (direction/conflict policy defaults and flag overrides)
  - `platform-parity-spec.md` (PTY support and backend capability gates)
- Risk reduced:
  - Prevents subtle drift between PTY and non-PTY implementations and between from_world vs from_host flows.

### CP3 — internal checkpoint/rollback seam
- Code-grounded boundary:
  - Completes internal git: checkpoint creation and restore semantics.
- Stabilized surfaces:
  - `internal-git-spec.md` (repo init, tag ids, restore rules, safety rails)
- Risk reduced:
  - Ensures rollback cannot touch protected paths and is stable across OS/filesystems; validates the most safety-sensitive operator surface.
