# best-effort-distro-package-manager — workstream triage

## Lift And Boundary Decision

- Strict-pack status:
  - `tasks.json` has `meta.slice_spec_version = 2`, so pack-derived lift is authoritative sizing input.
- Pack-derived lift v1 (`logs/workstream-triage/pm_lift_pack.json`):
  - `lift_score = 49`
  - `estimated_slices = 5`
  - `confidence = low`
  - split signals:
    - `likely_split:lift_score>24`
    - `likely_split:touch_files_sum>12`
    - `split_required:estimated_slices>3`
  - touch evidence:
    - `derived.impact_map_touch_counts.create.effective_count = 11`
    - `derived.impact_map_touch_counts.edit.effective_count = 8`
- Discovery-time lift v1 (`logs/workstream-triage/pm_lift_intake.json`):
  - `lift_score = 16`
  - `estimated_slices = 2`
  - `confidence = high`
- Decision:
  - accept a **4-slice** planning skeleton for full planning
  - do not jump to 5 slices yet because pack-lift confidence is low and the missing inputs are mostly non-touch structured fields
  - add one extra seam to isolate wrapper/env-doc propagation from installer-local override logic

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "BEDPM",
  "accepted_slice_order": [
    "BEDPM0",
    "BEDPM1",
    "BEDPM2",
    "BEDPM3"
  ],
  "draft_slice_order": [
    "BEDPM0",
    "BEDPM1",
    "BEDPM2"
  ],
  "pws": [
    {
      "id": "BEDPM-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [],
      "owns": [
        "pre-planning/spec_manifest.md",
        "contract.md",
        "decision_register.md"
      ]
    },
    {
      "id": "BEDPM-PWS-implementation_seams",
      "role": "slice_spec",
      "depends_on": [
        "BEDPM-PWS-contract"
      ],
      "assumes": [
        "The accepted split keeps wrapper and env-doc propagation separate from installer-local override logic."
      ],
      "owns": [
        "slices/BEDPM0/BEDPM0-spec.md",
        "slices/BEDPM1/BEDPM1-spec.md",
        "slices/BEDPM2/BEDPM2-spec.md"
      ]
    },
    {
      "id": "BEDPM-PWS-tests_ci",
      "role": "docs_validation",
      "depends_on": [
        "BEDPM-PWS-contract"
      ],
      "assumes": [
        "The authoritative repo test path remains tests/installers/pkg_manager_detection_smoke.sh.",
        "The feature-local Linux smoke script stays a thin wrapper over the repo test."
      ],
      "owns": [
        "manual_testing_playbook.md",
        "smoke/linux-smoke.sh",
        "slices/BEDPM3/BEDPM3-spec.md"
      ]
    },
    {
      "id": "BEDPM-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "BEDPM-PWS-contract",
        "BEDPM-PWS-implementation_seams",
        "BEDPM-PWS-tests_ci"
      ],
      "assumes": [
        "CI remains a single end-of-feature checkpoint unless full planning raises checkpoint count explicitly."
      ],
      "owns": [
        "pre-planning/ci_checkpoint_plan.md",
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/BEDPM0/kickoff_prompts/",
        "slices/BEDPM1/kickoff_prompts/",
        "slices/BEDPM2/kickoff_prompts/",
        "slices/BEDPM3/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed Planning Workstreams

### `BEDPM-PWS-contract` — contract and DR lock

- Goal:
  - lock the authoritative contract wording before any slice spec or task graph mirrors it
- Owned surfaces:
  - `pre-planning/spec_manifest.md`
  - `contract.md`
  - `decision_register.md`
- Dependencies:
  - none
- Proposed slices or triads to enable:
  - all accepted slices inherit precedence, source vocabulary, `<unknown>` semantics, PATH order, and the `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contract from this lane

### `BEDPM-PWS-implementation_seams` — implementation-facing slice specs

- Goal:
  - author the three non-validation slice specs with the extra propagation seam made explicit
- Owned surfaces:
  - `slices/BEDPM0/BEDPM0-spec.md`
  - `slices/BEDPM1/BEDPM1-spec.md`
  - `slices/BEDPM2/BEDPM2-spec.md`
- Dependencies:
  - `BEDPM-PWS-contract`
- Proposed slices or triads to create:
  - `BEDPM0` for distro detection, mapping, `<unknown>`, and the stable decision line
  - `BEDPM1` for installer-local override precedence, fixed PATH fallback order, and fail-closed selection
  - `BEDPM2` for wrapper exit-status pass-through plus operator/env-doc propagation on `scripts/substrate/install.sh`, `docs/INSTALLATION.md`, and `docs/reference/env/contract.md`

### `BEDPM-PWS-tests_ci` — validation and evidence seam

- Goal:
  - keep the validation-authoring lane separate from the implementation-spec lane while proving the same contract
- Owned surfaces:
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `slices/BEDPM3/BEDPM3-spec.md`
- Dependencies:
  - `BEDPM-PWS-contract`
- Proposed slices or triads to create:
  - `BEDPM3` for `tests/installers/pkg_manager_detection_smoke.sh`, the thin Linux smoke wrapper, and manual evidence for precedence, warning, remediation, and wrapper pass-through

### `BEDPM-PWS-tasks_checkpoints` — single writer for task graph and gates

- Goal:
  - keep sequencing, checkpoints, kickoff prompts, and planning-gate artifacts in one lane
- Owned surfaces:
  - `pre-planning/ci_checkpoint_plan.md`
  - `plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `kickoff_prompts/`
  - `slices/BEDPM0/kickoff_prompts/`
  - `slices/BEDPM1/kickoff_prompts/`
  - `slices/BEDPM2/kickoff_prompts/`
  - `slices/BEDPM3/kickoff_prompts/`
- Dependencies:
  - `BEDPM-PWS-contract`
  - `BEDPM-PWS-implementation_seams`
  - `BEDPM-PWS-tests_ci`
- Proposed slices or triads to create:
  - task wiring for `BEDPM0` through `BEDPM3`
  - one `CP1-ci-checkpoint` task after the accepted last slice
  - kickoff prompts and quality-gate wiring for the accepted slice set

## Sequencing And Gates

- Hard ordering:
  - `BEDPM-PWS-contract` lands first
  - `BEDPM-PWS-implementation_seams` and `BEDPM-PWS-tests_ci` can run after contract lock
  - `BEDPM-PWS-tasks_checkpoints` lands last
- Why the extra seam exists:
  - the impact map adds shared wrapper and operator-doc surfaces (`scripts/substrate/install.sh`, `docs/INSTALLATION.md`, `docs/reference/env/contract.md`) on top of installer-local logic
  - isolating those surfaces in `BEDPM2` reduces churn across the detection slice and the validation slice
- CI-checkpoint implication:
  - current `pre-planning/ci_checkpoint_plan.md` still defines `CP1` after `BEDPM2`
  - accepted triage order moves the last slice to `BEDPM3`
  - `BEDPM-PWS-tasks_checkpoints` must update:
    - `pre-planning/ci_checkpoint_plan.md` to checkpoint after `BEDPM3`
    - `tasks.json` checkpoint metadata to end on `BEDPM3`
  - keep the current platform split unless the CI plan changes first:
    - behavior smoke: `linux`
    - parity: `linux`, `macos`, `windows`

## Risk And Unknowns

- High-churn seam:
  - `SUBSTRATE_INSTALL_OS_RELEASE_PATH` is now selected by the impact map and minimal spec, but `contract.md`, `decision_register.md`, and downstream docs still need the exact path-validation and absence semantics
- High-churn seam:
  - wrapper exit-status pass-through and operator/env-doc propagation justify the added `BEDPM2` boundary
- High-churn seam:
  - `pre-planning/spec_manifest.md` still says `ci_checkpoint_plan.md` is unselected and only names three slice specs; full planning must reconcile that drift with the accepted slice order above
- Unknown:
  - pack-lift confidence is low because structured counts for CLI/docs/ops/QA fields are still missing; full planning must recompute lift after those fields are filled and compare the result against the accepted 4-slice order
- Unknown:
  - the exact warning string for multi-manager PATH detection still needs one final authoritative wording in `contract.md`

## Slice Skeleton Recommendations

- `NO CHANGE` `BEDPM0`
  - keep detection, mapping, `<unknown>`, and the stable decision line together
- `SPLIT` draft `BEDPM1`
  - keep `BEDPM1` for installer-local override precedence, fixed PATH fallback order, and fail-closed selection in `scripts/substrate/install-substrate.sh`
  - `ADD BEDPM2` for wrapper exit-status pass-through and operator/env-doc propagation on `scripts/substrate/install.sh`, `docs/INSTALLATION.md`, and `docs/reference/env/contract.md`
- `RENAME` draft `BEDPM2` to `BEDPM3`
  - keep the final slice as the validation and evidence seam after the operator-facing contract is complete
- Accepted slice order for full planning:
  - `BEDPM0`
  - `BEDPM1`
  - `BEDPM2`
  - `BEDPM3`

## Evidence Links

- Step sentinels:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/logs/spec-manifest/last_message.md`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/logs/impact-map/last_message.md`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/logs/min-spec-draft/last_message.md`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/logs/CI-checkpoint/last_message.md`
- Canonical pre-planning artifacts used:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/impact_map.md`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/pre-planning/ci_checkpoint_plan.md`
- Lift evidence:
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/logs/workstream-triage/pm_lift_pack.json`
  - `docs/project_management/packs/implemented/best-effort-distro-package-manager/logs/workstream-triage/pm_lift_intake.json`

## Follow-ups

- Reconcile `pre-planning/spec_manifest.md` to the accepted 4-slice order and the now-present `pre-planning/ci_checkpoint_plan.md`.
- When full planning adds structured task and checkpoint metadata, set the end-of-checkpoint boundary to `BEDPM3` unless the accepted slice order changes first.
- Re-open the `BEDPM2` boundary when lift recomputation reports `estimated_slices >= 5` after the missing structured fields are filled.
