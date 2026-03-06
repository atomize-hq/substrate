# best-effort-distro-package-manager — CI checkpoint plan

This file defines **when** cross-platform CI gates run for this feature.

Standard:
- `docs/project_management/system/standards/ci/PLANNING_CI_CHECKPOINT_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/best-effort-distro-package-manager/`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md` (draft slice skeleton + invariants)
- Slice specs: see `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`.
- Required platforms (authoritative; from `tasks.json`):
  - Behavior smoke platforms: `linux`
  - CI parity platforms: `linux`, `macos`, `windows`

## Operator rules
- This plan is authoritative for **CI cadence**.
- If you discover a mismatch between the plan and reality (slice ids, platform scope, contract surfaces), update this plan first, then update `tasks.json` and kickoff prompts.
- For schema v4+ cross-platform automation packs: update `tasks.json` `meta.checkpoint_boundaries` to list the **last slice** in each checkpoint group (this is linted once slice tasks exist in `tasks.json`).
- Pre-planning note: `tasks.json` does not define slice tasks yet, so this plan is not mechanically validated yet; the slice list below is derived from the canonical slice inventory in `pre-planning/spec_manifest.md`.

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
      "slices": ["BEDPM0"],
      "gates": {
        "compile_parity": true,
        "feature_smoke": false,
        "ci_testing": "quick"
      },
      "rationale": "Single checkpoint for the whole feature (1 slice < defaults.min=4). Boundary after BEDPM0 ensures the installer pkg-manager selection contract + hermetic detection harness are complete before running cross-platform gates."
    }
  ]
}
```

## Human-readable rationale (required)

For each checkpoint, explain:
- Why the boundary is code-grounded (subsystem seam / contract completion / enabling refactor / UX seam).
- What surfaces are stabilized by this checkpoint (from `spec_manifest.md`).
- What risk is reduced by running cross-platform CI here (from `impact_map.md`).

### CP1 (`BEDPM0`) — installer pkg-manager selection + hermetic detection tests seam

- Code-grounded boundary:
  - Contract completion seam: by the end of `BEDPM0`, the Linux installer’s pkg-manager selection contract is implemented end-to-end and a hermetic detection harness exists to assert precedence, outputs, warnings/errors, and exit codes deterministically.
- Stabilized surfaces (from `spec_manifest.md` ownership):
  - `scripts/substrate/install-substrate.sh` (installer entrypoint; Linux package-manager selection contract surface).
  - `tests/installers/pkg_manager_detection_test.sh` (hermetic contract enforcement for precedence, one-liner shape, and exit codes).
  - `contract.md` (authoritative CLI/env contract, selection algorithm, mapping table, determinism + warning rules, exit codes).
  - `decision_register.md` (DR-0001/2/3 decisions that pin parsing, ambiguity policy, and the hermetic test seam).
  - Slice spec `slices/BEDPM0/BEDPM0-spec.md` and the manual playbook.
- Risk reduced (from `impact_map.md`):
  - Detects cross-platform regressions from shared script/CLI changes by running compile parity + CI testing at a single, code-grounded seam.
  - Prevents user-facing installer drift in the decision one-liner and deterministic selection warnings.
  - Constrains exit-code taxonomy changes (`2|3|4`) to the intended override/selection failure modes without unintentionally reclassifying unrelated installer failures.
  - Reduces `world-enable.sh` sourcing/UX drift risk by validating the post-change installer surface at a single checkpoint seam.

## Follow-ups

- Mechanical validity (when slice tasks exist in `tasks.json`):
  - Replace the draft slice ids in this plan with the final slice ids and keep them in deterministic order with contiguous groups.
  - Add `CP1-ci-checkpoint` to `tasks.json` and wire it per the CI checkpoint standard.
  - Set `tasks.json` `meta.checkpoint_boundaries` to `["BEDPM0"]` (or the accepted final last-slice id for CP1).
  - Run (must pass): `python3 docs/project_management/system/scripts/planning/validate_ci_checkpoint_plan.py --feature-dir "docs/project_management/packs/draft/best-effort-distro-package-manager"`
- Gate selection:
  - If `FEATURE_DIR/smoke/` is added for this feature, flip CP1 gate `feature_smoke` to `true` and ensure smoke scripts exist for the selected behavior platforms.
  - Ensure `tasks.json` `meta.behavior_platforms_required` remains Linux-only (keep macOS/Windows as CI parity platforms only).
