# persist-macos-host-os-install-state — workstream triage

## Outcome

- Pack-derived Work Lift v1 is active for this pack because `tasks.json` sets `meta.slice_spec_version = 2`.
- Latest pack-derived lift:
  - `lift_score = 42`
  - `estimated_slices = 4`
  - `confidence = low`
  - triggers:
    - `likely_split:lift_score>24`
    - `likely_split:touch_files_sum>12`
    - `split_required:estimated_slices>3`
- Derived touch counts from `impact_map.md`:
  - create = `10`
  - edit = `6`
- Accepted slice order for full planning:
  - `PMHOIS0`
  - `PMHOIS1`
  - `PMHOIS2`
  - `PMHOIS3`
- Draft slice order from `pre-planning/minimal_spec_draft.md`:
  - `PMHOIS0`
  - `PMHOIS1`
  - `PMHOIS2`
- Triage decision:
  - expand the planning shape from 3 slices to 4 slices
  - keep the stable slice prefix `PMHOIS`
  - move the end-of-feature checkpoint boundary from `PMHOIS2` to `PMHOIS3`

## Machine-Readable PWS Index

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "PMHOIS",
  "accepted_slice_order": [
    "PMHOIS0",
    "PMHOIS1",
    "PMHOIS2",
    "PMHOIS3"
  ],
  "draft_slice_order": [
    "PMHOIS0",
    "PMHOIS1",
    "PMHOIS2"
  ],
  "pws": [
    {
      "id": "PMHOIS-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0039 keeps scope on hosted macOS installs in scripts/substrate/install-substrate.sh",
        "install_state.json remains schema_version 1"
      ],
      "owns": [
        "contract.md",
        "decision_register.md",
        "slices/PMHOIS0/PMHOIS0-spec.md"
      ]
    },
    {
      "id": "PMHOIS-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "PMHOIS-PWS-contract"
      ],
      "assumes": [
        "host_state.os field names stay family, product_version, build_version, and arch",
        "unknown top-level keys and existing host_state legacy keys remain preserved"
      ],
      "owns": [
        "install-state-schema-spec.md",
        "compatibility-spec.md",
        "slices/PMHOIS1/PMHOIS1-spec.md"
      ]
    },
    {
      "id": "PMHOIS-PWS-provisioning_wiring",
      "role": "implementation",
      "depends_on": [
        "PMHOIS-PWS-contract"
      ],
      "assumes": [
        "the shared installer writer stays in scripts/substrate/install-substrate.sh",
        "dev installer scope remains out of pack"
      ],
      "owns": [
        "filesystem-semantics-spec.md",
        "platform-parity-spec.md",
        "slices/PMHOIS2/PMHOIS2-spec.md"
      ]
    },
    {
      "id": "PMHOIS-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "PMHOIS-PWS-contract",
        "PMHOIS-PWS-schema_inventory",
        "PMHOIS-PWS-provisioning_wiring"
      ],
      "assumes": [
        "tests/mac/installer_parity_fixture.sh owns macOS execution-path assertions",
        "tests/installers/install_state_smoke.sh owns shared JSON and atomic-write assertions"
      ],
      "owns": [
        "plan.md",
        "slices/PMHOIS3/PMHOIS3-spec.md"
      ]
    },
    {
      "id": "PMHOIS-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "PMHOIS-PWS-contract",
        "PMHOIS-PWS-schema_inventory",
        "PMHOIS-PWS-provisioning_wiring",
        "PMHOIS-PWS-docs_validation"
      ],
      "assumes": [
        "one end-of-feature checkpoint remains enough after the slice split",
        "behavior platform scope remains macOS only"
      ],
      "owns": [
        "tasks.json",
        "session_log.md",
        "kickoff_prompts/",
        "slices/PMHOIS0/kickoff_prompts/",
        "slices/PMHOIS1/kickoff_prompts/",
        "slices/PMHOIS2/kickoff_prompts/",
        "slices/PMHOIS3/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Slice Skeleton Recommendations

- `SPLIT PMHOIS0`
  - `PMHOIS0` = freeze contract boundary and decision outcomes
  - `PMHOIS1` = freeze schema inventory, compatibility, and platform guarantees
- `RENAME draft PMHOIS1`
  - new id: `PMHOIS2`
  - title: implement hosted macOS persistence writer
- `RENAME draft PMHOIS2`
  - new id: `PMHOIS3`
  - title: close validation, checkpoint evidence, and operator docs

## Proposed Planning Workstreams

### PMHOIS-PWS-contract — freeze contract boundary and decision outcomes

- Goal: lock hosted-mac scope, canonical file path semantics, `--no-world` coverage, `--dry-run` no-write semantics, warning-only metadata posture, and the decision register outputs that feed every later planning artifact.
- Owned surfaces:
  - pack files:
    - `contract.md`
    - `decision_register.md`
    - `slices/PMHOIS0/PMHOIS0-spec.md`
  - implementation and contract surfaces:
    - canonical metadata path rule for `<effective_prefix>/install_state.json`
    - hosted installer scope boundary in `scripts/substrate/install-substrate.sh`
    - future-consumer read semantics for persisted `host_state.os.*`
- Dependencies:
  - none
- Proposed slices or triads:
  - `PMHOIS0`

### PMHOIS-PWS-schema_inventory — freeze schema inventory and compatibility

- Goal: lock the exact additive `install_state.json` field inventory, normalization rules, preserved legacy content, unknown-key preservation, and platform guarantee inventory for macOS, Linux, and Windows.
- Owned surfaces:
  - pack files:
    - `install-state-schema-spec.md`
    - `compatibility-spec.md`
    - `slices/PMHOIS1/PMHOIS1-spec.md`
  - contract surfaces:
    - `host_state.os.family`
    - `host_state.os.product_version`
    - `host_state.os.build_version`
    - `host_state.os.arch`
    - preservation of `host_state.group`
    - preservation of `host_state.linger`
    - preservation of `host_state.platform`
- Dependencies:
  - `PMHOIS-PWS-contract`
- Proposed slices or triads:
  - `PMHOIS1`

### PMHOIS-PWS-provisioning_wiring — freeze writer semantics and implementation seam

- Goal: lock the hosted installer writer path, same-directory temp-file and replace ordering, warning-only degradation, and the implementation acceptance boundary for the shared installer script.
- Owned surfaces:
  - pack files:
    - `filesystem-semantics-spec.md`
    - `platform-parity-spec.md`
    - `slices/PMHOIS2/PMHOIS2-spec.md`
  - implementation surfaces:
    - `scripts/substrate/install-substrate.sh`
    - same-directory temp path `<effective_prefix>/install_state.json.tmp`
    - macOS normal install write path
    - macOS `--no-world` write path
    - macOS `--dry-run` no-write path
- Dependencies:
  - `PMHOIS-PWS-contract`
- Proposed slices or triads:
  - `PMHOIS2`

### PMHOIS-PWS-docs_validation — freeze validation and operator alignment

- Goal: lock the validation split across `tests/mac/installer_parity_fixture.sh` and `tests/installers/install_state_smoke.sh`, the manual evidence flow, the checkpoint evidence flow, and the `docs/INSTALLATION.md` conformance text.
- Owned surfaces:
  - pack files:
    - `plan.md`
    - `slices/PMHOIS3/PMHOIS3-spec.md`
  - validation and docs surfaces:
    - `tests/mac/installer_parity_fixture.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
- Dependencies:
  - `PMHOIS-PWS-contract`
  - `PMHOIS-PWS-schema_inventory`
  - `PMHOIS-PWS-provisioning_wiring`
- Proposed slices or triads:
  - `PMHOIS3`

### PMHOIS-PWS-tasks_checkpoints — serialize the task graph and checkpoint wiring

- Goal: write the single authoritative task graph, kickoff prompts, and checkpoint wiring after the accepted slice order and every upstream planning decision are stable.
- Owned surfaces:
  - pack files:
    - `tasks.json`
    - `session_log.md`
    - `kickoff_prompts/`
    - `slices/PMHOIS0/kickoff_prompts/`
    - `slices/PMHOIS1/kickoff_prompts/`
    - `slices/PMHOIS2/kickoff_prompts/`
    - `slices/PMHOIS3/kickoff_prompts/`
- Dependencies:
  - `PMHOIS-PWS-contract`
  - `PMHOIS-PWS-schema_inventory`
  - `PMHOIS-PWS-provisioning_wiring`
  - `PMHOIS-PWS-docs_validation`
- Proposed slices or triads:
  - task graph and kickoff prompts for `PMHOIS0`
  - task graph and kickoff prompts for `PMHOIS1`
  - task graph and kickoff prompts for `PMHOIS2`
  - task graph and kickoff prompts for `PMHOIS3`
- Single-writer rule:
  - this workstream is the only writer for `tasks.json`

## Sequencing And Gates

- Hard ordering:
  - `PMHOIS-PWS-contract` lands first.
  - `PMHOIS-PWS-schema_inventory` and `PMHOIS-PWS-provisioning_wiring` start after `PMHOIS-PWS-contract`.
  - `PMHOIS-PWS-docs_validation` starts after `PMHOIS-PWS-schema_inventory` and `PMHOIS-PWS-provisioning_wiring`.
  - `PMHOIS-PWS-tasks_checkpoints` lands last.
- Hard gates:
  - freeze the partial-emission answer in `decision_register.md` before `PMHOIS1`, `PMHOIS2`, and `PMHOIS3` acceptance text closes
  - keep `scripts/substrate/dev-install-substrate.sh` outside this pack unless ADR-0039 scope changes explicitly
  - keep the additive `schema_version = 1` contract fixed across all slice specs and task wiring
- CI checkpoint implications:
  - keep one checkpoint group
  - change the checkpoint slice list from `PMHOIS0..PMHOIS2` to `PMHOIS0..PMHOIS3`
  - change `meta.checkpoint_boundaries` from `["PMHOIS2"]` to `["PMHOIS3"]`
  - keep `compile_parity = true`
  - keep `feature_smoke = true`
  - keep `ci_testing = "quick"`

## Risks And Unknowns

- The ADR has no embedded `PM_LIFT_VECTOR`, so discovery-time lift evidence is absent.
- `decision_register.md` still needs the final answer for partial `host_state.os` emission when one source command fails.
- The current `pre-planning/spec_manifest.md` and `pre-planning/ci_checkpoint_plan.md` still describe a 3-slice shape.
- `scripts/substrate/install-substrate.sh`, `tests/installers/install_state_smoke.sh`, and `docs/INSTALLATION.md` overlap with adjacent installer work, so full planning needs explicit shared-file sequencing.
- `docs/INSTALLATION.md` still contains a macOS contradiction that full planning needs to replace with diagnostic-state wording plus the Linux-only cleanup boundary.

## Evidence Links

- Canonical artifacts used:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/tasks.json`
- Stable completion sentinel paths:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/spec-manifest/last_message.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/impact-map/last_message.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/min-spec-draft/last_message.md`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/CI-checkpoint/last_message.md`
- Lift evidence:
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/workstream-triage/pm_lift_pack.txt`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/workstream-triage/pm_lift_pack.json`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/workstream-triage/pm_lift_intake.txt`
  - `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/workstream-triage/pm_lift_intake.json`

## Follow-ups

- Align the next full-planning pass to the accepted 4-slice order before kickoff prompts and triad tasks land.
- Update the checkpoint boundary references to `PMHOIS3` when full planning rewires `tasks.json`.
- Record the intake-lift gap as backlog debt on ADR-0039 or its intake doc.
