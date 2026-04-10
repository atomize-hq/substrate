# persist-macos-host-os-install-state — workstream triage

## Summary

- Candidate prefix: `PMHOS`
- Planning pressure: `high`
- Recommended downstream planning workstream count: `6`
- Draft candidate order basis from `minimal_spec_draft.md`: `PMHOS-01`, `PMHOS-02`, `PMHOS-03`
- Recommended downstream planning order:
  1. `PMHOS-FWS-decision_spine`
  2. `PMHOS-FWS-contract_surface`
  3. `PMHOS-FWS-schema_compatibility`
  4. `PMHOS-FWS-filesystem_rules`
  5. `PMHOS-FWS-platform_parity`
  6. `PMHOS-FWS-validation_reconciliation`

Planning rationale:
- The canonical impact map already locks the three open A/B decisions to one path, but downstream planning still needs an explicit decision lane so later spec authors inherit one stable source.
- The canonical minimal spec draft compresses `PMHOS-02` across contract posture, filesystem semantics, and parity guarantees. Those seams have different dependency shapes and different owning docs.
- The canonical checkpoint plan keeps one later verification checkpoint. That structure supports a front-loaded planning split and a single late validation gate.

<!-- PM_FSE_WORKSTREAM_INDEX:BEGIN -->
```json
{
  "index_version": 1,
  "candidate_prefix": "PMHOS",
  "recommended_workstream_order": [
    "PMHOS-FWS-decision_spine",
    "PMHOS-FWS-contract_surface",
    "PMHOS-FWS-schema_compatibility",
    "PMHOS-FWS-filesystem_rules",
    "PMHOS-FWS-platform_parity",
    "PMHOS-FWS-validation_reconciliation"
  ],
  "draft_candidate_order": [
    "PMHOS-01",
    "PMHOS-02",
    "PMHOS-03"
  ],
  "workstreams": [
    {
      "id": "PMHOS-FWS-decision_spine",
      "role": "Freeze ADR follow-up decisions before topic-spec authoring diverges.",
      "depends_on": [],
      "assumes": [
        "ADR-0039 remains the authority for additive macOS host OS persistence.",
        "The canonical pre-planning inputs remain the basis for downstream planning."
      ],
      "owns": [
        "decision_register.md"
      ],
      "outcomes": [
        "Producer scope is frozen as hosted installer plus dev installer.",
        "Partial-capture serialization is frozen as host_state.os.family plus collected leaves.",
        "Automated evidence is frozen with tests/mac/installer_parity_fixture.sh primary and tests/installers/install_state_smoke.sh secondary."
      ]
    },
    {
      "id": "PMHOS-FWS-contract_surface",
      "role": "Author the operator-facing contract for metadata path, warning posture, exit-code posture, and future-consumer precedence.",
      "depends_on": [
        "PMHOS-FWS-decision_spine"
      ],
      "assumes": [
        "effective_prefix resolution remains owned by existing installer logic outside this pack.",
        "The feature keeps one canonical metadata file at <effective_prefix>/install_state.json."
      ],
      "owns": [
        "contract.md"
      ],
      "outcomes": [
        "The path rule, no-new-command rule, no-new-flag rule, and no-new-env rule are explicit.",
        "Warning-only behavior is explicit for metadata-side failures on otherwise successful installs.",
        "Future-consumer precedence is explicit for persisted values then runtime fallback."
      ]
    },
    {
      "id": "PMHOS-FWS-schema_compatibility",
      "role": "Author the macOS schema and additive compatibility boundary without reopening Linux-owned fields.",
      "depends_on": [
        "PMHOS-FWS-decision_spine"
      ],
      "assumes": [
        "The Linux pack remains the sole owner of host_state.platform.* semantics.",
        "schema_version stays integer 1."
      ],
      "owns": [
        "install-state-schema-spec.md",
        "compatibility-spec.md"
      ],
      "outcomes": [
        "The exact host_state.os.* field contract and source mapping are explicit.",
        "Leaf absence semantics are explicit.",
        "Merge preservation is explicit for host_state.group, host_state.linger, host_state.platform.*, and unknown keys.",
        "Reader tolerance and additive-only compatibility are explicit."
      ]
    },
    {
      "id": "PMHOS-FWS-filesystem_rules",
      "role": "Author the write algorithm, temp-file replacement sequence, and recovery semantics for install_state.json.",
      "depends_on": [
        "PMHOS-FWS-decision_spine",
        "PMHOS-FWS-schema_compatibility"
      ],
      "assumes": [
        "The macOS subtree shape is frozen before filesystem recovery examples are finalized.",
        "The feature preserves prior canonical files on failed writes when a prior file exists."
      ],
      "owns": [
        "filesystem-semantics-spec.md"
      ],
      "outcomes": [
        "The write trigger for successful macOS producer flows is explicit.",
        "The same-directory temp path and replace-after-complete-document sequence are explicit.",
        "Parse-failure recovery and failed-write cleanup behavior are explicit.",
        "Directory-creation and protected-path boundaries are explicit."
      ]
    },
    {
      "id": "PMHOS-FWS-platform_parity",
      "role": "Author platform guarantees and no-change boundaries after contract, schema, and filesystem semantics are stable.",
      "depends_on": [
        "PMHOS-FWS-contract_surface",
        "PMHOS-FWS-schema_compatibility",
        "PMHOS-FWS-filesystem_rules"
      ],
      "assumes": [
        "Windows remains on the no-write boundary for this ADR.",
        "Uninstall cleanup behavior remains Linux-only."
      ],
      "owns": [
        "platform-parity-spec.md"
      ],
      "outcomes": [
        "The macOS producer matrix is explicit for hosted install, hosted --no-world, and dev-install coverage.",
        "Linux no-change guarantees are explicit for host_state.platform.* and cleanup-reader behavior.",
        "Windows no-write and no-change guarantees are explicit.",
        "Harness-role mapping is explicit for later validation planning."
      ]
    },
    {
      "id": "PMHOS-FWS-validation_reconciliation",
      "role": "Author the manual validation playbook and operator-doc reconciliation after parity is frozen.",
      "depends_on": [
        "PMHOS-FWS-platform_parity"
      ],
      "assumes": [
        "The parity lane has already fixed the producer matrix and no-change boundaries.",
        "Checkpoint execution wiring remains outside this planning workstream."
      ],
      "owns": [
        "manual_testing_playbook.md",
        "operator-doc reconciliation surface",
        "validation evidence mapping surface"
      ],
      "outcomes": [
        "Manual validation is explicit for hosted macOS, hosted macOS --no-world, and dev-install producer coverage.",
        "Warning-only degradation checks are explicit for collector failure and malformed-file recovery.",
        "The operator-doc reconciliation set is explicit for docs/INSTALLATION.md.",
        "The final evidence mapping is explicit for later checkpoint consumption."
      ]
    }
  ]
}
```
<!-- PM_FSE_WORKSTREAM_INDEX:END -->

## Proposed downstream planning workstreams

### PMHOS-FWS-decision_spine — decision register lock

- Goal: freeze DR-0001, DR-0002, and DR-0003 before topic-spec authoring diverges.
- Owned surfaces:
  - `decision_register.md`
- Dependencies:
  - none
- Expected downstream deliverables:
  - hosted installer plus dev installer producer scope
  - `host_state.os.family = "macos"` plus collected-leaf serialization rule
  - `tests/mac/installer_parity_fixture.sh` primary and `tests/installers/install_state_smoke.sh` secondary evidence split

### PMHOS-FWS-contract_surface — operator contract and read precedence

- Goal: freeze the user-visible contract for metadata path, warning posture, exit-code posture, and future-consumer precedence.
- Owned surfaces:
  - `contract.md`
- Dependencies:
  - `PMHOS-FWS-decision_spine`
- Expected downstream deliverables:
  - canonical `<effective_prefix>/install_state.json` path rule
  - no-new-command, no-new-flag, and no-new-env rule
  - warning-only metadata failure posture
  - persisted-value-first then runtime-fallback rule for future consumers

### PMHOS-FWS-schema_compatibility — schema and additive merge boundary

- Goal: freeze the `host_state.os.*` schema and reader-tolerance boundary without reopening Linux-owned fields.
- Owned surfaces:
  - `install-state-schema-spec.md`
  - `compatibility-spec.md`
- Dependencies:
  - `PMHOS-FWS-decision_spine`
- Expected downstream deliverables:
  - exact field list, types, and source-command mapping
  - exact leaf absence semantics
  - additive-only compatibility contract
  - merge preservation for `host_state.group`, `host_state.linger`, `host_state.platform.*`, and unknown keys

### PMHOS-FWS-filesystem_rules — writer flow and recovery semantics

- Goal: freeze the create-or-update algorithm, temp-file replacement, and recovery semantics around `install_state.json`.
- Owned surfaces:
  - `filesystem-semantics-spec.md`
- Dependencies:
  - `PMHOS-FWS-decision_spine`
  - `PMHOS-FWS-schema_compatibility`
- Expected downstream deliverables:
  - write trigger for successful macOS producer flows
  - same-directory temp path rule for `install_state.json.tmp`
  - replace-after-complete-document rule
  - parse-failure recovery and prior-file preservation rule
  - directory-creation and protected-path boundary for metadata writes

### PMHOS-FWS-platform_parity — platform guarantees and no-change boundaries

- Goal: freeze macOS producer guarantees plus Linux, Windows, and uninstaller no-change guarantees.
- Owned surfaces:
  - `platform-parity-spec.md`
- Dependencies:
  - `PMHOS-FWS-contract_surface`
  - `PMHOS-FWS-schema_compatibility`
  - `PMHOS-FWS-filesystem_rules`
- Expected downstream deliverables:
  - exact macOS producer matrix for hosted install, hosted `--no-world`, and dev-install coverage
  - exact Linux no-change guarantee for `host_state.platform.*`
  - exact Windows no-write and no-change guarantee
  - exact Linux-only cleanup boundary for uninstall readers
  - harness-role mapping carried into later validation work

### PMHOS-FWS-validation_reconciliation — validation playbook and operator-doc alignment

- Goal: freeze the validation evidence set and operator-doc reconciliation after the parity boundary is stable.
- Owned surfaces:
  - `manual_testing_playbook.md`
  - operator-doc reconciliation surface for `docs/INSTALLATION.md`
  - validation evidence mapping surface
- Dependencies:
  - `PMHOS-FWS-platform_parity`
- Expected downstream deliverables:
  - manual validation procedure for hosted macOS, hosted macOS `--no-world`, and dev-install macOS producer coverage
  - warning-only degradation checks for collector failure and malformed-file recovery
  - operator-doc reconciliation plan for `docs/INSTALLATION.md`
  - final evidence map consumed by the checkpoint plan and later decomposition

## Sequencing and gates

### Hard ordering constraints

- `PMHOS-FWS-decision_spine` runs first and completes first.
- `PMHOS-FWS-contract_surface` and `PMHOS-FWS-schema_compatibility` start after `PMHOS-FWS-decision_spine`.
- `PMHOS-FWS-filesystem_rules` starts after `PMHOS-FWS-schema_compatibility`.
- `PMHOS-FWS-platform_parity` starts after `PMHOS-FWS-contract_surface`, `PMHOS-FWS-schema_compatibility`, and `PMHOS-FWS-filesystem_rules`.
- `PMHOS-FWS-validation_reconciliation` starts after `PMHOS-FWS-platform_parity`.

### Parallelization guidance

- The only early parallel window is `PMHOS-FWS-contract_surface` plus `PMHOS-FWS-schema_compatibility`.
- `PMHOS-FWS-filesystem_rules` stays downstream of schema finalization because filesystem recovery examples depend on the exact subtree shape and absence semantics.
- `PMHOS-FWS-platform_parity` stays downstream of contract, schema, and filesystem outputs because parity guarantees consume all three surfaces.
- `PMHOS-FWS-validation_reconciliation` stays last because the playbook and operator-doc reconciliation consume the frozen parity matrix.

### CI checkpoint implications

- The canonical `ci_checkpoint_plan.md` keeps one later verification checkpoint across `PMHOS-01`, `PMHOS-02`, and `PMHOS-03`.
- That checkpoint consumes outputs from `PMHOS-FWS-contract_surface`, `PMHOS-FWS-schema_compatibility`, `PMHOS-FWS-filesystem_rules`, `PMHOS-FWS-platform_parity`, and `PMHOS-FWS-validation_reconciliation`.
- No extra planning-layer checkpoint adds value before the later verification checkpoint because the shared-file contract remains cross-coupled until parity and validation surfaces converge.

## Candidate skeleton recommendations

- `ADD`: add one explicit decision candidate ahead of `PMHOS-01` to own `decision_register.md` and freeze DR-0001, DR-0002, and DR-0003.
- `SPLIT`: split `PMHOS-02` into three downstream planning lanes:
  - contract surface
  - filesystem rules
  - platform parity
- `REORDER`: place the decision candidate first and place validation plus operator-guidance work after parity is frozen.
- `NO CHANGE`: keep `PMHOS-01` as the schema-and-compatibility candidate and keep `PMHOS-03` at the tail of the draft candidate order.

## Risks and unknowns

- The highest-churn seam is the shared `install_state.json` writer logic in both installer scripts.
- The next highest-churn seam is the wording boundary between `docs/INSTALLATION.md` and the implemented Linux persistence pack.
- The uninstaller-reader tolerance boundary stays sensitive because cleanup readers consume Linux cleanup fields from the same file that gains a new macOS subtree.
- The downstream planning graph now treats the producer scope, partial-leaf serialization rule, and harness split as fixed inputs. Any later reversal expands churn across contract, schema, filesystem, parity, and validation docs.

## Evidence links

### Stable step completion sentinels

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/CI-checkpoint/last_message.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/logs/workstream-triage/planning_pressure_assessment.md`

### Canonical artifacts used

- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/persist-macos-host-os-install-state/pre-planning/ci_checkpoint_plan.md`

## Follow-ups

- Promote this staged candidate after overlap-safe validation.
- Carry the draft candidate order `PMHOS-01`, `PMHOS-02`, `PMHOS-03` forward as the baseline candidate skeleton during downstream planning.
- Resolve execution wiring later in the subsystem that owns checkpoint execution and implementation sequencing.
