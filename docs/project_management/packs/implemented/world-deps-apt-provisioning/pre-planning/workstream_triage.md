# world-deps-apt-provisioning — workstream triage (pre-planning)

Goal: propose pack-internal planning workstreams (PWS) and record the accepted slice order that full planning converged on for this pack.

## Inputs (authoritative)

- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/ci_checkpoint_plan.md`
- `tasks.json`
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`

## Work Lift evidence (v1)

### Discovery-time (ADR-derived)

Source: `make pm-lift-intake FILE="docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md"` (logs: `logs/workstream-triage/pm_lift_intake.txt`)
- Lift score: `24`
- Estimated slices: `2`
- Confidence: `low` (missing: `touch.create_files`, `touch.boundary_crossings`, `qa.*`, `risk.unknowns_high`)

### Planning-time (pack-derived; strict pack)

This pack is strict (`tasks.json` → `meta.slice_spec_version=2`), so `pm-lift-pack` reflects the authored Touch Set in `pre-planning/impact_map.md`.

Source: `make pm-lift-pack PACK="docs/project_management/packs/draft/world-deps-apt-provisioning"` (logs: `logs/workstream-triage/pm_lift_pack.txt`)
- Lift score: `101`
- Estimated slices: `9`
- Confidence: `low` (contract/QA/ops counts are not derivable from the Touch Set alone)
- Derived Touch counts (from `pm_lift_pack.json`):
  - `touch.create_files=11` (pack artifacts created during planning)
  - `touch.edit_files=30` (implementation + docs + scripts surfaces)
- Split signals:
  - `split_required:estimated_slices>3`
  - `likely_split:lift_score>24`

Convergence decision:
- Full planning retained the two-slice execution model from `minimal_spec_draft.md`.
- Shared script and installer ordering lands in `WDAP0`.
- Operator-doc and upstream contract reconciliation lands in `WDAP1`.
- The accepted execution slice order is `WDAP0` then `WDAP1`.

## Slice prefix + accepted slice order

- Slice prefix: `WDAP`
- Draft slice order (from `pre-planning/minimal_spec_draft.md`): `WDAP0`, `WDAP1`
- Accepted full-planning slice order: `WDAP0`, `WDAP1`

## Planning workstreams (PWS)

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "WDAP",
  "accepted_slice_order": ["WDAP0", "WDAP1"],
  "draft_slice_order": ["WDAP0", "WDAP1"],
  "pws": [
    {
      "id": "WDAP-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "Operator-facing contract remains pack-local and links to upstream authoritative inputs without restating their schemas"
      ],
      "owns": [
        "contract.md",
        "decision_register.md"
      ]
    },
    {
      "id": "WDAP-PWS-provisioning_wiring",
      "role": "slice_spec",
      "depends_on": ["WDAP-PWS-contract"],
      "assumes": [
        "Provisioning remains guest-backend-only and fails closed on Linux host-native"
      ],
      "owns": [
        "slices/WDAP0/WDAP0-spec.md"
      ]
    },
    {
      "id": "WDAP-PWS-runtime_fail_early",
      "role": "slice_spec",
      "depends_on": ["WDAP-PWS-contract"],
      "assumes": [
        "Runtime remediation continues to require the exact command string `substrate world enable --provision-deps`"
      ],
      "owns": [
        "slices/WDAP1/WDAP1-spec.md"
      ]
    },
    {
      "id": "WDAP-PWS-tests_ci",
      "role": "tests_ci",
      "depends_on": [
        "WDAP-PWS-contract",
        "WDAP-PWS-provisioning_wiring",
        "WDAP-PWS-runtime_fail_early"
      ],
      "assumes": [
        "Behavior smoke remains required on linux/macos/windows"
      ],
      "owns": [
        "manual_testing_playbook.md",
        "smoke/"
      ]
    },
    {
      "id": "WDAP-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "WDAP-PWS-contract",
        "WDAP-PWS-provisioning_wiring",
        "WDAP-PWS-runtime_fail_early",
        "WDAP-PWS-tests_ci"
      ],
      "assumes": [
        "Checkpoint boundaries in tasks.json match the accepted slice ordering and the CI checkpoint plan"
      ],
      "owns": [
        "tasks.json",
        "plan.md",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/WDAP0/kickoff_prompts/",
        "slices/WDAP1/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

### WDAP-PWS-contract — Contract + decision register

- Goal: write the operator-facing contract and resolve the DRs that gate slice specs and validation.
- Owns: `contract.md`, `decision_register.md`
- Dependencies: none
- Must land first:
  - DR-0001 conflict policy for APT requirement derivation (version pins; de-dup; ordering)
  - DR-0003 provisioning isolation model, host-mutation guard rails, and request `profile` value
  - Windows posture and deterministic messaging and exit mapping

### WDAP-PWS-provisioning_wiring — Provisioning slice spec (`WDAP0`)

- Goal: specify provisioning-time behavior and acceptance criteria for the APT provisioning workflow, including helper-script and installer ordering invariants.
- Owns:
  - `slices/WDAP0/WDAP0-spec.md`
- Dependencies: `WDAP-PWS-contract`
- Execution slice emitted by full planning:
  - `WDAP0-{code,test,integ}`

### WDAP-PWS-runtime_fail_early — Runtime fail-early slice spec (`WDAP1`)

- Goal: specify runtime prohibition of APT/dpkg under `world deps current sync|install`, deterministic remediation, and the required operator-doc reconciliation targets.
- Owns:
  - `slices/WDAP1/WDAP1-spec.md`
- Dependencies: `WDAP-PWS-contract`
- Execution slice emitted by full planning:
  - `WDAP1-{code,test,integ}`

### WDAP-PWS-tests_ci — Manual playbook + smoke scripts

- Goal: author deterministic validation artifacts aligned to the contract and slice acceptance criteria.
- Owns:
  - `manual_testing_playbook.md`
  - `smoke/`
- Dependencies:
  - `WDAP-PWS-contract`
  - `WDAP-PWS-provisioning_wiring`
  - `WDAP-PWS-runtime_fail_early`
- Coverage required:
  - guest provisioning success path and guest-only gating
  - Linux host-native fail-closed posture (`Substrate will not mutate the host OS`)
  - runtime fail-early and remediation text invariants

### WDAP-PWS-tasks_checkpoints — `tasks.json` + plan/runbook + quality gate

- Goal: be the single writer for `tasks.json`, wire checkpoints, and author the execution runbook scaffolding for the accepted slice order.
- Owns:
  - `tasks.json`, `plan.md`, `session_log.md`, `quality_gate_report.md`
  - `kickoff_prompts/` and the accepted per-slice kickoff prompt directories under `slices/`
- Dependencies: all other PWS
- Must encode:
  - `meta.checkpoint_boundaries = ["WDAP0", "WDAP1"]`
  - triad tasks for `WDAP0` and `WDAP1`
  - checkpoint tasks `CP1-ci-checkpoint` and `CP2-ci-checkpoint`

## Sequencing + gates (hard ordering)

1. `WDAP-PWS-contract`
2. In parallel:
   - `WDAP-PWS-provisioning_wiring`
   - `WDAP-PWS-runtime_fail_early`
3. `WDAP-PWS-tests_ci`
4. `WDAP-PWS-tasks_checkpoints`

## Accepted slice model

- `WDAP0` — provisioning-time APT surface, execution posture, helper and installer wiring
- `WDAP1` — runtime fail-early posture, remediation invariants, and operator-doc and upstream contract reconciliation

The accepted slice order is singular:
- `WDAP0` → `WDAP1`

Checkpoint alignment:
- `CP1` ends at `WDAP0`
- `CP2` ends at `WDAP1`

## Risks + unknowns (planning follow-ups)

- Windows posture remains explicit and unsupported in this pack; all validation artifacts and remediation text must encode that posture deterministically.
- DR-0001 and DR-0003 remain high-churn surfaces; slice specs and tests depend on the exact conflict policy and isolation posture.
- Shared script overlap remains high (see `pre-planning/impact_map.md` cross-queue scan); keep edits minimal per file and constrained to the accepted two-slice model.
- Upstream contract and operator docs must leave exactly one coherent truth for runtime APT behavior.

## Evidence links (sentinels + relied-on artifacts)

Step completion sentinels:
- `logs/spec-manifest/last_message.md`
- `logs/impact-map/last_message.md`
- `logs/min-spec-draft/last_message.md`
- `logs/CI-checkpoint/last_message.md`

Canonical artifacts relied on:
- `pre-planning/spec_manifest.md`
- `pre-planning/impact_map.md`
- `pre-planning/minimal_spec_draft.md`
- `pre-planning/ci_checkpoint_plan.md`
- `tasks.json`

Lift outputs captured for this triage:
- `logs/workstream-triage/pm_lift_intake.txt`
- `logs/workstream-triage/pm_lift_intake.json`
- `logs/workstream-triage/pm_lift_pack.txt`
- `logs/workstream-triage/pm_lift_pack.json`
