# persist-detected-linux-distro-pkg-manager — workstream triage

Goal: propose pack-internal planning workstreams (PWS) and sequencing gates for full planning.

## Evidence

Canonical artifacts relied on:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`

Stable sentinels:
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/CI-checkpoint/last_message.md`

Work Lift evidence:
- Pack-derived: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_pack.txt`
- Pack-derived JSON: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_pack.json`
- Intake/ADR-derived: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_intake.txt`
- Intake/ADR-derived JSON: `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/logs/workstream-triage/pm_lift_intake.json`

## Work Lift Summary

- Strict-pack eligibility: `tasks.json.meta.slice_spec_version=2`, so pack-derived lift is authoritative.
- Intake/ADR lift (ADR-0032):
  - `lift_score=9`
  - `estimated_slices=1`
  - `confidence=high`
  - triggers: none
- Pack-derived lift:
  - `lift_score=33`
  - `estimated_slices=3`
  - `confidence=low`
  - triggers:
    - `likely_split:lift_score>24`
    - `likely_split:touch_files_sum>12`
  - derived touch counts (`derived.impact_map_touch_counts`):
    - create=`7`
    - edit=`6`
    - delete=`0`
    - deprecate=`0`

Interpretation:
- Keep one pack.
- Keep the three-slice skeleton `PDLDPM0..PDLDPM2`.
- Treat the pack-lift split triggers as a warning to keep planning ownership disjoint across contract/schema, reliable write semantics, and smoke/operator validation.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "PDLDPM",
  "accepted_slice_order": [
    "PDLDPM0",
    "PDLDPM1",
    "PDLDPM2"
  ],
  "pws": [
    {
      "id": "PDLDPM-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0031 contract remains authoritative for selected-manager and pkg_manager.source vocabulary",
        "Hosted install and dev install remain one producer contract"
      ],
      "owns": [
        "contract.md",
        "decision_register.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "PDLDPM-PWS-contract"
      ],
      "assumes": [
        "schema_version stays 1 with additive fields only"
      ],
      "owns": [
        "install-state-schema-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory"
      ],
      "assumes": [
        "Hosted uninstaller HOME-vs-prefix mismatch remains follow-up-only unless scope is explicitly expanded"
      ],
      "owns": [
        "pre-planning/spec_manifest.md",
        "pre-planning/impact_map.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-slice_spec_pdldpm0",
      "role": "slice_spec",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory"
      ],
      "assumes": [
        "Persistence stores upstream detection outputs verbatim"
      ],
      "owns": [
        "slices/PDLDPM0/PDLDPM0-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-slice_spec_pdldpm1",
      "role": "slice_spec",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory"
      ],
      "assumes": [
        "The accepted write matrix covers hosted install, hosted --no-world, dev install, dev --no-world, and dry-run"
      ],
      "owns": [
        "slices/PDLDPM1/PDLDPM1-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-slice_spec_pdldpm2",
      "role": "slice_spec",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory"
      ],
      "assumes": [
        "Linux behavior smoke remains the only behavior-changing platform surface"
      ],
      "owns": [
        "slices/PDLDPM2/PDLDPM2-spec.md"
      ]
    },
    {
      "id": "PDLDPM-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "PDLDPM-PWS-contract",
        "PDLDPM-PWS-schema_inventory",
        "PDLDPM-PWS-docs_validation",
        "PDLDPM-PWS-slice_spec_pdldpm0",
        "PDLDPM-PWS-slice_spec_pdldpm1",
        "PDLDPM-PWS-slice_spec_pdldpm2"
      ],
      "assumes": [
        "Single checkpoint CP1 remains after PDLDPM2 unless full planning explicitly changes the slice skeleton"
      ],
      "owns": [
        "plan.md",
        "tasks.json",
        "pre-planning/ci_checkpoint_plan.md",
        "session_log.md",
        "kickoff_prompts/",
        "slices/PDLDPM0/kickoff_prompts/",
        "slices/PDLDPM1/kickoff_prompts/",
        "slices/PDLDPM2/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed Planning Workstreams (PWS)

### PDLDPM-PWS-contract — Contract + decision register

Goal:
- Lock the installer-facing persistence contract before slice specs or checkpoint wiring become stable.
- Pin the equivalence rule between effective install prefix and `$SUBSTRATE_HOME/install_state.json`.
- Pin Linux-only guarantees, dry-run/no-write behavior, best-effort metadata failure posture, and the future-consumer fallback rule.
- Pin the installer-scope choice selected in `pre-planning/impact_map.md`: hosted install and dev install share one metadata contract.

Owned surfaces:
- `contract.md`
- `decision_register.md`

Dependencies:
- None

Proposed slices/triads to create during full planning:
- Contract inputs for `PDLDPM0`, `PDLDPM1`, and `PDLDPM2`

### PDLDPM-PWS-schema_inventory — install-state schema + compatibility

Goal:
- Author the schema boundary for `install_state.json`.
- Pin exact field paths, type/absence semantics, preservation of existing `host_state.group` and `host_state.linger`, additive compatibility, and canonical JSON examples.

Owned surfaces:
- `install-state-schema-spec.md`

Dependencies:
- `PDLDPM-PWS-contract`

Proposed slices/triads to create during full planning:
- Schema inputs for `PDLDPM0`, `PDLDPM1`, and `PDLDPM2`

### PDLDPM-PWS-docs_validation — pre-planning doc coherence + touch-set validity

Goal:
- Keep `pre-planning/spec_manifest.md` and `pre-planning/impact_map.md` aligned with the accepted slice order and the cross-pack authority boundary.
- Preserve the selected out-of-scope boundary from `pre-planning/impact_map.md`: hosted uninstaller HOME-vs-prefix alignment stays follow-up-only.
- Keep the touch set mechanically stable for planning-lint and future lift recomputation.

Owned surfaces:
- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`

Dependencies:
- `PDLDPM-PWS-contract`
- `PDLDPM-PWS-schema_inventory`

Proposed slices/triads to create during full planning:
- None; this workstream keeps the planning surfaces coherent

### PDLDPM-PWS-slice_spec_pdldpm0 — slice spec: persist `host_state.platform.*`

Goal:
- Author `PDLDPM0` acceptance criteria for persisting distro/package-manager metadata under the exact JSON nesting.
- Lock the rule that selected manager and source are copied verbatim from the upstream detection contract.

Owned surfaces:
- `slices/PDLDPM0/PDLDPM0-spec.md`

Dependencies:
- `PDLDPM-PWS-contract`
- `PDLDPM-PWS-schema_inventory`

Proposed slices/triads to create during full planning:
- `PDLDPM0` triad

### PDLDPM-PWS-slice_spec_pdldpm1 — slice spec: reliable file creation/update semantics

Goal:
- Author `PDLDPM1` acceptance criteria for successful-install write/update behavior, idempotence, no-write branches, and warning-only degradation on metadata write failure.
- Lock the per-branch rule for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`.

Owned surfaces:
- `slices/PDLDPM1/PDLDPM1-spec.md`

Dependencies:
- `PDLDPM-PWS-contract`
- `PDLDPM-PWS-schema_inventory`

Proposed slices/triads to create during full planning:
- `PDLDPM1` triad

### PDLDPM-PWS-slice_spec_pdldpm2 — slice spec: smoke coverage + operator evidence

Goal:
- Author `PDLDPM2` acceptance criteria for installer smoke assertions and operator-facing evidence.
- Lock exact assertions for file existence, field presence/absence, missing-`/etc/os-release` degradation, and `schema_version = 1` compatibility.

Owned surfaces:
- `slices/PDLDPM2/PDLDPM2-spec.md`

Dependencies:
- `PDLDPM-PWS-contract`
- `PDLDPM-PWS-schema_inventory`

Proposed slices/triads to create during full planning:
- `PDLDPM2` triad

### PDLDPM-PWS-tasks_checkpoints — plan/tasks/checkpoint wiring (single writer)

Goal:
- Act as the single writer for `tasks.json` and all automation-owned planning scaffolding.
- Convert the accepted slice order into executable triads, checkpoint wiring, session-log scaffolding, and kickoff prompts.
- Finalize the single-checkpoint plan already implied by `pre-planning/ci_checkpoint_plan.md`.

Owned surfaces:
- `plan.md`
- `tasks.json`
- `pre-planning/ci_checkpoint_plan.md`
- `session_log.md`
- `kickoff_prompts/`
- `slices/PDLDPM0/kickoff_prompts/`
- `slices/PDLDPM1/kickoff_prompts/`
- `slices/PDLDPM2/kickoff_prompts/`

Dependencies:
- `PDLDPM-PWS-contract`
- `PDLDPM-PWS-schema_inventory`
- `PDLDPM-PWS-docs_validation`
- `PDLDPM-PWS-slice_spec_pdldpm0`
- `PDLDPM-PWS-slice_spec_pdldpm1`
- `PDLDPM-PWS-slice_spec_pdldpm2`

Proposed slices/triads to create during full planning:
- `PDLDPM0-code`, `PDLDPM0-test`, `PDLDPM0-integ`
- `PDLDPM1-code`, `PDLDPM1-test`, `PDLDPM1-integ`
- `PDLDPM2-code`, `PDLDPM2-test`, `PDLDPM2-integ`
- `CP1-ci-checkpoint`

## Sequencing + Gates

1. Gate A: `PDLDPM-PWS-contract`
   - Required before schema authoring, slice specs, or task/checkpoint wiring become stable.
2. Gate B: `PDLDPM-PWS-schema_inventory`
   - Required before any slice spec that names exact JSON fields/examples is treated as stable.
3. Gate C: parallel fan-out
   - `PDLDPM-PWS-docs_validation`
   - `PDLDPM-PWS-slice_spec_pdldpm0`
   - `PDLDPM-PWS-slice_spec_pdldpm1`
   - `PDLDPM-PWS-slice_spec_pdldpm2`
4. Gate D: `PDLDPM-PWS-tasks_checkpoints`
   - `tasks.json` stays single-writer and lands after docs/schema/slice surfaces are concrete.

CI checkpoint implications:
- `pre-planning/ci_checkpoint_plan.md` currently selects one checkpoint, `CP1`, after `PDLDPM2`.
- Keep `meta.behavior_platforms_required=["linux"]`.
- Keep `meta.ci_parity_platforms_required=["linux","macos","windows"]`.
- `PDLDPM-PWS-tasks_checkpoints` adds `meta.checkpoint_boundaries=["PDLDPM2"]` and the `CP1-ci-checkpoint` ops task when it populates `tasks.json`.

## Risks + Unknowns

- High-churn seam: both installer producers (`install-substrate.sh` and `dev-install-substrate.sh`) touch `install_state.json`, but ADR wording still needs directory/path drift cleanup.
- High-churn seam: installer producers write by effective prefix, while uninstall consumers still read different paths today; current impact-map selection keeps that mismatch out of scope.
- Cross-pack dependency risk: this pack must mirror selected manager and `pkg_manager.source` vocabulary from `best-effort-distro-package-manager` exactly.
- Shared-file sequencing risk: `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, and `docs/INSTALLATION.md` also overlap with multiple neighboring packs.
- Unknown: whether `docs/INSTALLATION.md` reconciliation stays wording-only or forces broader operator-path clarification during full planning.

## Slice Skeleton Recommendations

Starting point from `pre-planning/minimal_spec_draft.md`:
- `PDLDPM0`
- `PDLDPM1`
- `PDLDPM2`

Recommendation:
- NO CHANGE.
- `accepted_slice_order = ["PDLDPM0", "PDLDPM1", "PDLDPM2"]`
- The strict-pack lift estimate (`estimated_slices=3`) matches the current skeleton, so no `ADD`, `SPLIT`, `MERGE`, or `RENAME` action is recommended.

## Follow-ups

- Fix ADR-0032 feature-dir and related-doc drift from `draft/stashing-ferret` to `draft/persist-detected-linux-distro-pkg-manager`.
- Pin the exact write/no-write matrix for hosted install, hosted `--no-world`, dev install, dev `--no-world`, and `--dry-run`.
- Pin the exact temp-file and replace rule for `install_state.json` updates.
- Reconcile `docs/INSTALLATION.md` wording for `schema_version`, effective metadata path, and shared hosted/dev installer scope.
- Keep hosted uninstaller HOME-vs-prefix alignment as a separate follow-up unless full planning explicitly expands scope.
