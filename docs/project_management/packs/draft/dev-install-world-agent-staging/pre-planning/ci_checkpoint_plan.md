# dev-install-world-agent-staging — CI checkpoint plan (pre-planning)

This file defines **when** cross-platform CI gates run for this feature.

The canonical checkpoint-plan path for this pack is `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/ci_checkpoint_plan.md`; the extracted `-fse` seam docs are planning inputs only.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/minimal_spec_draft.md` (draft slice skeleton + invariants)
- Slice specs: see `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`.

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice tasks yet, so this plan’s slice list is based on the draft slice skeleton in `pre-planning/minimal_spec_draft.md` and is not mechanically validated yet.

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
      "slices": ["DIWAS0", "DIWAS1"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": true,
        "ci_testing": "quick"
      },
      "rationale": "Single checkpoint for the full feature (2 slices < min_triads_per_checkpoint=4). Boundary after DIWAS1 ensures dev-install staging + enable preflight contract is complete before running cross-platform gates."
    }
  ]
}
```

## Human-readable rationale (required)

### CP1 (DIWAS0 → DIWAS1)

Why this boundary is code-grounded:
- **Contract completion seam:** the user-visible “dev install with `--no-world`, enable later” workflow is only coherent once BOTH are true:
  - `DIWAS1` stages `world-agent` into the contract-defined version-dir layout, and
  - `DIWAS0` implements the early missing-artifact preflight + deterministic remediation (before any privileged steps).

What surfaces this checkpoint stabilizes (from `spec_manifest.md` ownership):
- Dev-install `--no-world` staging behavior (paths + profile mapping + overwrite posture; contract-defined).
- Missing-`world-agent` enable preflight behavior (ordering + exit code + remediation; contract-defined).
- Cross-cutting config invariants (`$SUBSTRATE_HOME/config.yaml`, `world.enabled` flip only after verification).
- Platform guarantees: Linux behavior delta; macOS no-change posture; Windows unsupported posture (but still included in CI parity).
- Installer smoke (`tests/installers/install_smoke.sh`) remains C-04 regression evidence for dev-install staging only; it does not create an additional Linux behavior-delta proof surface.

What risk is reduced by running cross-platform CI here (from `impact_map.md`):
- Catches cross-platform compile regressions from `crates/shell/src/builtins/world_enable/runner.rs` changes (Windows/macOS parity).
- Exercises the Linux-only “enable later” behavior end-to-end via feature smoke (staging + preflight + remediation posture), aligned to the contract and exit-code taxonomy.
- Runs CI testing in `quick` mode to reduce the chance of CI-only breakage from shared shell + installer surfaces (`scripts/substrate/*`, `tests/installers/install_smoke.sh`) without paying full-suite cost during a 2-slice feature.

## Follow-ups

- Path canonicalization:
  - The canonical checkpoint-plan path is `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/ci_checkpoint_plan.md`.
  - Keep any impact-map or manifest references aligned to that path if they still point at the legacy root-level location.
- Mechanical validity (when slice tasks exist in `tasks.json`):
  - Ensure the final slice ids match `DIWAS0`, `DIWAS1` (or update both this plan and `tasks.json` consistently).
  - Add slice triad tasks (`DIWAS{0,1}-{code,test,integ}` plus any `-integ-core` / platform-fix tasks required by schema v4 cross-platform standards).
  - Add `CP1-ci-checkpoint` (type `ops`) and wire dependencies per the CI checkpoint standard.
  - Ensure `tasks.json` `meta.checkpoint_boundaries = ["DIWAS1"]` continues to match this plan once the final slice list is accepted.
  - Run (must pass): `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/dev-install-world-agent-staging"`.
