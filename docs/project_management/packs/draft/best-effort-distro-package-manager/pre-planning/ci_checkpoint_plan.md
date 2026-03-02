# best-effort-distro-package-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/minimal_spec_draft.md` (draft slice skeleton + invariants)
- Slice specs: see `docs/project_management/packs/draft/best-effort-distro-package-manager/spec_manifest.md`.

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice tasks yet, so this plan’s slice list is based on the draft slice skeleton in `minimal_spec_draft.md` and is not mechanically validated yet.

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
      "slices": ["BEDPM0", "BEDPM1", "BEDPM2"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Single checkpoint after the end-to-end installer manager-selection seam is implemented and hermetic detection tests exist. Total slices (3) is below the default min=4."
    }
  ]
}
```

## Human-readable rationale (required)

For each checkpoint, explain:
- Why the boundary is code-grounded (subsystem seam / contract completion / enabling refactor / UX seam).
- What surfaces are stabilized by this checkpoint (from `spec_manifest.md`).
- What risk is reduced by running cross-platform CI here (from `impact_map.md`).

### CP1 — installer pkg-manager selection + hermetic detection tests seam

- Code-grounded boundary:
  - Contract completion seam: distro detection + precedence pipeline + failure posture + stable stderr one-liner are implemented, and the hermetic detection test harness exists to assert the contract deterministically.
- Stabilized surfaces (from `spec_manifest.md` ownership):
  - `scripts/substrate/install-substrate.sh` (installer entrypoint; Linux package-manager selection contract surface).
  - `tests/installers/pkg_manager_detection_test.sh` (hermetic contract enforcement for precedence, one-liner shape, and exit codes).
  - `contract.md` (authoritative CLI/env contract, selection algorithm, mapping table, determinism + warning rules, exit codes).
  - `decision_register.md` (DR-0001/2/3 decisions that pin parsing, ambiguity policy, and the hermetic test seam).
  - Slice specs for the three draft slices (`BEDPM0`/`BEDPM1`/`BEDPM2`) and the manual playbook.
- Risk reduced (from `impact_map.md`):
  - Detects cross-platform regressions from shared argument parsing and shell changes (CI Testing runs shell lint on Linux and workspace tests across Linux/macOS/Windows).
  - Prevents user-facing installer drift in the decision one-liner and deterministic selection warnings.
  - Constrains exit-code taxonomy changes (`2|3|4`) to the intended override/selection failure modes without unintentionally reclassifying unrelated installer failures.
  - Reduces `world-enable.sh` sourcing/UX drift risk by validating the post-change installer surface at a single checkpoint seam.

## Follow-ups

- Mechanical validity (when slice tasks exist in `tasks.json`):
  - Replace the draft slice ids in this plan with the final slice ids and keep them in deterministic order with contiguous groups.
  - Add `CP1-ci-checkpoint` to `tasks.json` and wire it per the CI checkpoint standard.
  - Set `tasks.json` `meta.checkpoint_boundaries` to `["BEDPM2"]` (or the accepted final last-slice id for CP1).
  - Run (must pass): `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/best-effort-distro-package-manager"`
- Gate selection:
  - If `FEATURE_DIR/smoke/` is added for this feature, flip CP1 gate `feature_smoke` to `true` and ensure smoke scripts exist for the selected behavior platforms.
  - If the installer behavior guarantee is Linux-only, narrow `tasks.json` `meta.behavior_platforms_required` and keep macOS/Windows as CI parity platforms only.

