# add-non-apt-system-package-provisioning-support — workstream triage

Goal: propose pack-internal planning workstreams (PWS) and the accepted slice order for full planning.

## Inputs (authoritative)

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`
- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`

## Work Lift evidence

### Discovery-time (ADR-derived)

Source: `make pm-lift-intake FILE="docs/project_management/adrs/draft/ADR-0033-routing-weasel.md"`
- `lift_score=27`
- `estimated_slices=3`
- `confidence=low`
- triggers:
  - `likely_split:lift_score>24`
  - missing-input triggers for `touch.create_files`, `touch.boundary_crossings`, `qa.*`, and `risk.unknowns_high`

### Planning-time (pack-derived; strict pack)

This pack is strict (`tasks.json.meta.slice_spec_version=2`), so the pack-derived lift reflects the tracked `pre-planning/impact_map.md`.

Source: `make pm-lift-pack PACK="docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support"`
- `lift_score=106`
- `estimated_slices=9`
- `confidence=low`
- triggers:
  - `likely_split:lift_score>24`
  - `likely_split:touch_files_sum>12`
  - `split_required:estimated_slices>3`
- derived touch counts from `pm_lift_pack.json`:
  - create=`16`
  - edit=`25`
  - delete=`0`
  - deprecate=`0`

Interpretation:
- Keep one planning pack.
- Do not keep the draft three-slice skeleton unchanged.
- Split the original `NASP1` and `NASP2` seams before full planning so schema, provisioning wiring, runtime fail-early behavior, and validation/doc reconciliation can move with lower churn.

## Slice prefix + accepted slice order

- Slice prefix: `NASP`
- Draft slice order from `pre-planning/minimal_spec_draft.md`:
  - `NASP0`
  - `NASP1`
  - `NASP2`
- Accepted slice order for full planning:
  - `NASP0`
  - `NASP1`
  - `NASP2`
  - `NASP3`
  - `NASP4`

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "NASP",
  "accepted_slice_order": [
    "NASP0",
    "NASP1",
    "NASP2",
    "NASP3",
    "NASP4"
  ],
  "draft_slice_order": [
    "NASP0",
    "NASP1",
    "NASP2"
  ],
  "pws": [
    {
      "id": "NASP-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "Shared manager-aware CLI and runtime semantics stay authoritative in this pack contract once overlapping APT wording is reconciled"
      ],
      "owns": [
        "contract.md",
        "decision_register.md"
      ]
    },
    {
      "id": "NASP-PWS-os_probe",
      "role": "os_probe",
      "depends_on": [
        "NASP-PWS-contract"
      ],
      "assumes": [
        "Manager detection stays in-world using os-release plus command availability rather than host PATH or host installer state"
      ],
      "owns": [
        "slices/NASP0/NASP0-spec.md"
      ]
    },
    {
      "id": "NASP-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "NASP-PWS-contract"
      ],
      "assumes": [
        "The chosen schema posture stays additive with install.method pacman rather than a manager-agnostic remap layer"
      ],
      "owns": [
        "world-deps-pacman-schema-spec.md",
        "slices/NASP1/NASP1-spec.md"
      ]
    },
    {
      "id": "NASP-PWS-provisioning_wiring",
      "role": "provisioning_wiring",
      "depends_on": [
        "NASP-PWS-contract",
        "NASP-PWS-os_probe",
        "NASP-PWS-schema_inventory"
      ],
      "assumes": [
        "Provisioning remains explicit through substrate world enable --provision-deps and never mutates the host OS"
      ],
      "owns": [
        "slices/NASP2/NASP2-spec.md"
      ]
    },
    {
      "id": "NASP-PWS-runtime_fail_early",
      "role": "runtime_fail_early",
      "depends_on": [
        "NASP-PWS-contract",
        "NASP-PWS-schema_inventory",
        "NASP-PWS-provisioning_wiring"
      ],
      "assumes": [
        "Runtime remediation keeps the exact substrate world enable --provision-deps command string and mirrors the selected provisioning contract"
      ],
      "owns": [
        "slices/NASP3/NASP3-spec.md"
      ]
    },
    {
      "id": "NASP-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "NASP-PWS-contract",
        "NASP-PWS-os_probe",
        "NASP-PWS-schema_inventory",
        "NASP-PWS-provisioning_wiring",
        "NASP-PWS-runtime_fail_early"
      ],
      "assumes": [
        "Behavior smoke and manual evidence remain required on linux macos and windows"
      ],
      "owns": [
        "pre-planning/spec_manifest.md",
        "slices/NASP4/NASP4-spec.md",
        "platform-parity-spec.md",
        "manual_testing_playbook.md",
        "smoke/linux-smoke.sh",
        "smoke/macos-smoke.sh",
        "smoke/windows-smoke.ps1"
      ]
    },
    {
      "id": "NASP-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "NASP-PWS-contract",
        "NASP-PWS-os_probe",
        "NASP-PWS-schema_inventory",
        "NASP-PWS-provisioning_wiring",
        "NASP-PWS-runtime_fail_early",
        "NASP-PWS-docs_validation"
      ],
      "assumes": [
        "Two checkpoints ending at NASP2 and NASP4 reduce churn without splitting contract ownership"
      ],
      "owns": [
        "pre-planning/ci_checkpoint_plan.md",
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/NASP0/kickoff_prompts/",
        "slices/NASP1/kickoff_prompts/",
        "slices/NASP2/kickoff_prompts/",
        "slices/NASP3/kickoff_prompts/",
        "slices/NASP4/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed planning workstreams

### NASP-PWS-contract — Contract + decision register

- Goal:
  - lock the operator-facing contract and remove ADR ambiguity before slice specs or checkpoint wiring settle
- Owned surfaces:
  - `contract.md`
  - `decision_register.md`
- Dependencies:
  - none
- Proposed slices/triads to create during full planning:
  - contract inputs for `NASP0`, `NASP1`, `NASP2`, `NASP3`, and `NASP4`
- Must resolve first:
  - exact world OS probe tie-break semantics
  - exact mixed-manager enabled-set rule
  - pacman command-construction and idempotency rule
  - exact Windows support or unsupported posture
  - pacman runnable-wrapper or present-semantics scope

### NASP-PWS-os_probe — Probe + support-gate slice spec

- Goal:
  - author the `NASP0` slice spec for in-world manager detection and provisioning eligibility
- Owned surfaces:
  - `slices/NASP0/NASP0-spec.md`
- Dependencies:
  - `NASP-PWS-contract`
- Proposed slices/triads to create during full planning:
  - `NASP0-{code,test,integ}`
- Scope:
  - `/etc/os-release` inputs
  - `command -v pacman`
  - mismatch and absence rules
  - in-world-only probe invariant

### NASP-PWS-schema_inventory — Schema spec + schema slice

- Goal:
  - author the inventory/schema boundary and isolate the schema-view seam from provisioning execution
- Owned surfaces:
  - `world-deps-pacman-schema-spec.md`
  - `slices/NASP1/NASP1-spec.md`
- Dependencies:
  - `NASP-PWS-contract`
- Proposed slices/triads to create during full planning:
  - `NASP1-{code,test,integ}`
- Scope:
  - `install.method=apt | pacman | script | manual`
  - `install.pacman`
  - mutual exclusion rules
  - valid and invalid YAML examples
  - inventory list/show JSON and YAML view expectations

### NASP-PWS-provisioning_wiring — Provisioning execution slice spec

- Goal:
  - author the `NASP2` slice spec for pacman requirement derivation and provisioning-time execution
- Owned surfaces:
  - `slices/NASP2/NASP2-spec.md`
- Dependencies:
  - `NASP-PWS-contract`
  - `NASP-PWS-os_probe`
  - `NASP-PWS-schema_inventory`
- Proposed slices/triads to create during full planning:
  - `NASP2-{code,test,integ}`
- Scope:
  - manager-aware requirement derivation
  - package de-duplication and ordering
  - request-profile usage boundaries
  - pacman command construction
  - mismatch fail-closed behavior during provisioning

### NASP-PWS-runtime_fail_early — Runtime remediation slice spec

- Goal:
  - author the `NASP3` slice spec for runtime fail-early behavior and explicit-item scoping
- Owned surfaces:
  - `slices/NASP3/NASP3-spec.md`
- Dependencies:
  - `NASP-PWS-contract`
  - `NASP-PWS-schema_inventory`
  - `NASP-PWS-provisioning_wiring`
- Proposed slices/triads to create during full planning:
  - `NASP3-{code,test,integ}`
- Scope:
  - `current sync` and `current install` fail-early rules
  - explicit-item scope rule for `current install <ITEM...>`
  - remediation wording invariants
  - runtime no-system-package-mutation contract

### NASP-PWS-docs_validation — Validation evidence + doc reconciliation slice

- Goal:
  - author the validation-evidence slice and reconcile pack-local documentation to the accepted manager-aware contract
- Owned surfaces:
  - `pre-planning/spec_manifest.md`
  - `slices/NASP4/NASP4-spec.md`
  - `platform-parity-spec.md`
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
- Dependencies:
  - `NASP-PWS-contract`
  - `NASP-PWS-os_probe`
  - `NASP-PWS-schema_inventory`
  - `NASP-PWS-provisioning_wiring`
  - `NASP-PWS-runtime_fail_early`
- Proposed slices/triads to create during full planning:
  - `NASP4-{code,test,integ}`
- Scope:
  - platform parity posture
  - manual validation evidence
  - smoke-script assertions
  - spec-manifest slice-id reconciliation if the accepted split is adopted

### NASP-PWS-tasks_checkpoints — `tasks.json` + runbook + checkpoint wiring

- Goal:
  - act as the single writer for `tasks.json` and convert the accepted slice model into executable triads and checkpoints
- Owned surfaces:
  - `pre-planning/ci_checkpoint_plan.md`
  - `plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `kickoff_prompts/`
  - `slices/NASP0/kickoff_prompts/`
  - `slices/NASP1/kickoff_prompts/`
  - `slices/NASP2/kickoff_prompts/`
  - `slices/NASP3/kickoff_prompts/`
  - `slices/NASP4/kickoff_prompts/`
- Dependencies:
  - all other PWS
- Proposed slices/triads to create during full planning:
  - `NASP0-{code,test,integ}`
  - `NASP1-{code,test,integ}`
  - `NASP2-{code,test,integ}`
  - `NASP3-{code,test,integ}`
  - `NASP4-{code,test,integ}`
  - `CP1-ci-checkpoint`
  - `CP2-ci-checkpoint`

## Sequencing + gates

Hard ordering:
1. `NASP-PWS-contract`
2. In parallel:
   - `NASP-PWS-os_probe`
   - `NASP-PWS-schema_inventory`
3. `NASP-PWS-provisioning_wiring`
4. `NASP-PWS-runtime_fail_early`
5. `NASP-PWS-docs_validation`
6. `NASP-PWS-tasks_checkpoints`

Checkpoint implications:
- The current tracked `pre-planning/ci_checkpoint_plan.md` is still a draft single-checkpoint plan over `NASP0` through `NASP2`.
- If full planning accepts this triage result, update `pre-planning/ci_checkpoint_plan.md` first, then wire `tasks.json`.
- Recommended checkpoint boundaries for the accepted split:
  - `CP1-ci-checkpoint`
    - boundary slice: `NASP2`
    - scope: probe + schema + provisioning wiring
  - `CP2-ci-checkpoint`
    - boundary slice: `NASP4`
    - scope: runtime fail-early + doc reconciliation + validation evidence

## Risks + unknowns

- Probe tie-break remains unresolved between `/etc/os-release` and package-manager presence checks.
- Mixed-manager enabled-set behavior is still unresolved; full planning must reject fallback, partial success, or silent skipping unless the decision register says otherwise.
- Pacman command-construction, idempotency, and runnable-wrapper scope remain unresolved high-churn seams.
- Windows posture is still not pinned beyond pre-planning follow-up language.
- `pre-planning/spec_manifest.md` and `pre-planning/ci_checkpoint_plan.md` currently assume only `NASP0` through `NASP2`; full planning must reconcile those docs immediately if the accepted split is adopted.
- Overlap with `world-deps-apt-provisioning` and `world-deps-packages-bundles-contract` remains a contract-authority risk until shared wording is reconciled.

## Slice skeleton recommendations

Starting point from `pre-planning/minimal_spec_draft.md`:
- `NASP0` — probe world manager and support gate
- `NASP1` — add pacman schema and provisioning path
- `NASP2` — lock runtime fail-early and reconcile docs

Recommended changes:
- `SPLIT` `NASP1`
  - keep `NASP1` as pacman schema extension, inventory validation, and inventory-view updates
  - add `NASP2` as provisioning routing, request-profile use, and pacman command execution
- `SPLIT` `NASP2`
  - add `NASP3` as runtime fail-early behavior, explicit-item scoping, and remediation wording
  - add `NASP4` as contract reconciliation, platform parity, and manual/smoke validation evidence
- `ADD`: `NASP3`, `NASP4`
- `MERGE`: none
- `RENAME`: none

Reason:
- pack-derived lift reports `lift_score=106`, `estimated_slices=9`, and `split_required:estimated_slices>3`
- the impact map separates schema, provisioning wiring, runtime fail-early semantics, and doc/validation reconciliation into distinct high-churn seams
- a five-slice model captures the dominant seams without over-fragmenting the pack

## Evidence links

Step completion sentinels:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/CI-checkpoint/last_message.md`

Canonical artifacts relied on:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/tasks.json`

Lift outputs captured for this triage:
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_intake.txt`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_intake.json`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_pack.txt`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/logs/workstream-triage/pm_lift_pack.json`

## Follow-ups

- If full planning accepts the five-slice model, update `pre-planning/spec_manifest.md` and `pre-planning/ci_checkpoint_plan.md` before writing slice tasks and kickoff prompts.
- If full planning rejects the five-slice model, record the rejection explicitly and justify how the original three-slice plan will manage the `split_required` lift signal and the 41-file touch set.
