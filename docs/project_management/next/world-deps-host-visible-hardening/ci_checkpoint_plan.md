# world-deps-host-visible-hardening — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/next/world-deps-host-visible-hardening`
- `docs/project_management/next/world-deps-host-visible-hardening/impact_map.md`
- `docs/project_management/next/world-deps-host-visible-hardening/spec_manifest.md`
- Slice specs: `docs/project_management/next/world-deps-host-visible-hardening/*-spec*.md`
- Required platforms (authoritative):
  - Behavior smoke platforms: `tasks.json` → `meta.behavior_platforms_required` (`linux`, `macos`)
  - CI parity platforms: `tasks.json` → `meta.ci_parity_platforms_required` (`linux`, `macos`, `windows`)
  - WSL coverage: `tasks.json` → `meta.wsl_required=true` (`bundled` via Linux smoke)

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
      "slices": ["WDH0", "WDH1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Checkpoint after the first contract-complete seam: deterministic world env construction (WDH0) plus deterministic runnable wrappers and host-path-independent present semantics (WDH1). This is the earliest point where cross-platform smoke validates the hardened operator posture without relying on later slices."
    },
    {
      "id": "CP2",
      "task_id": "CP2-ci-checkpoint",
      "slices": ["WDH2", "WDH3"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "full"
      },
      "rationale": "Final checkpoint after the exec-guard hardening surface (WDH2) and the installer/first-run scaffolding UX surface (WDH3). This boundary validates the full end-to-end operator workflow and the safety posture across required platforms."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 — env normalization + wrappers/present seam
- Code-grounded boundary:
  - Completes deterministic PATH/HOME/XDG construction and wrapper-anchored runnable semantics.
- Stabilized surfaces:
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH1-spec.md`
- Risk reduced:
  - Prevents host toolchain leakage via PATH on required platforms and makes `present/missing/blocked` meaningful under `world_fs.host_visible=true`.

### CP2 — exec-guard + installer scaffold seam
- Code-grounded boundary:
  - Completes the explicit-path bypass closure and the operator-facing inventory scaffolding UX.
- Stabilized surfaces:
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH2-spec.md`
  - `docs/project_management/next/world-deps-host-visible-hardening/WDH3-spec.md`
- Risk reduced:
  - Validates the hardened posture and the discovery scaffolding across required platforms before feature completion.
