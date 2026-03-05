# world-deps-apt-provisioning — workstream triage (pre-planning)

Goal: propose pack-internal planning workstreams (PWS) + hard sequencing gates for full planning.

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
  - `touch.create_files=11` (pack artifacts to be created during planning)
  - `touch.edit_files=30` (implementation + docs + scripts surfaces)
- Split signals:
  - `split_required:estimated_slices>3`
  - `likely_split:lift_score>24`

Implication: treat the current 2-slice skeleton (`WDAP0`, `WDAP1`) as under-split for execution; see “Slice skeleton recommendations”.

## Slice prefix + baseline skeleton (from `pre-planning/minimal_spec_draft.md`)

- Slice prefix: `WDAP`
- Draft slices: `WDAP0`, `WDAP1`

## Planning workstreams (PWS)

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 1,
  "slice_prefix": "WDAP",
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
        "Provisioning remains guest-backend-only and fails closed on Linux host-native (no host OS mutation)"
      ],
      "owns": [
        "slices/WDAP0/WDAP0-spec.md",
        "slices/WDAP2/WDAP2-spec.md"
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
      "id": "WDAP-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "WDAP-PWS-contract",
        "WDAP-PWS-provisioning_wiring",
        "WDAP-PWS-runtime_fail_early"
      ],
      "assumes": [
        "Operator-doc updates link to `contract.md` and do not duplicate contract tables"
      ],
      "owns": [
        "slices/WDAP3/WDAP3-spec.md"
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
        "Behavior smoke remains required on linux/macos/windows (per tasks.json meta)"
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
        "WDAP-PWS-docs_validation",
        "WDAP-PWS-tests_ci"
      ],
      "assumes": [
        "Checkpoint boundaries in tasks.json match the final slice ordering and the CI checkpoint plan"
      ],
      "owns": [
        "tasks.json",
        "plan.md",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/WDAP0/kickoff_prompts/",
        "slices/WDAP1/kickoff_prompts/",
        "slices/WDAP2/kickoff_prompts/",
        "slices/WDAP3/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

### WDAP-PWS-contract — Contract + decision register

- Goal: write the operator-facing contract and resolve the DRs that gate slice specs + validation.
- Owns: `contract.md`, `decision_register.md`
- Dependencies: none
- Must land first:
  - DR-0001 conflict policy for APT requirement derivation (version pins; de-dup; ordering)
  - DR-0003 provisioning isolation model + host-mutation guard rails + request `profile` value(s)
  - Windows posture (supported vs unsupported) and deterministic messaging/exit mapping

### WDAP-PWS-provisioning_wiring — Provisioning slice specs (`WDAP0`, `WDAP2`)

- Goal: specify provisioning-time behavior + acceptance criteria for the APT provisioning workflow (guest-only; fail closed on Linux host-native).
- Owns:
  - `slices/WDAP0/WDAP0-spec.md` (core provisioning contract + routing + guard rails)
  - `slices/WDAP2/WDAP2-spec.md` (scripts/installer integration + cross-platform provisioning entrypoints)
- Dependencies: `WDAP-PWS-contract`
- Proposed slice/triads for full planning:
  - `WDAP0-{code,test,integ}`
  - `WDAP2-{code,test,integ}`

### WDAP-PWS-runtime_fail_early — Runtime fail-early slice spec (`WDAP1`)

- Goal: specify runtime prohibition of APT/dpkg under `world deps current sync|install`, with deterministic remediation.
- Owns: `slices/WDAP1/WDAP1-spec.md`
- Dependencies: `WDAP-PWS-contract`
- Proposed slice/triads for full planning:
  - `WDAP1-{code,test,integ}`

### WDAP-PWS-docs_validation — Docs + upstream contract reconciliation slice spec (`WDAP3`)

- Goal: specify and validate the operator-doc + upstream contract updates required to remove runtime-APT contradictions and keep docs coherent.
- Owns: `slices/WDAP3/WDAP3-spec.md`
- Dependencies: `WDAP-PWS-contract`, `WDAP-PWS-provisioning_wiring`, `WDAP-PWS-runtime_fail_early`
- Proposed slice/triads for full planning:
  - `WDAP3-{code,test,integ}` (doc updates are executed/validated as part of the slice integ + planning lint gates)

### WDAP-PWS-tests_ci — Manual playbook + smoke scripts

- Goal: author deterministic validation artifacts aligned to the contract + slice acceptance criteria.
- Owns: `manual_testing_playbook.md`, `smoke/`
- Dependencies: `WDAP-PWS-contract`, `WDAP-PWS-provisioning_wiring`, `WDAP-PWS-runtime_fail_early`
- Proposed artifacts to cover:
  - Guest provisioning success path(s) + guest-only gating
  - Linux host-native fail-closed posture (“no host OS mutation”)
  - Runtime fail-early + remediation text invariants

### WDAP-PWS-tasks_checkpoints — `tasks.json` + plan/runbook + quality gate

- Goal: be the single writer for `tasks.json`, wire checkpoints, and author the execution runbook scaffolding for the pack.
- Owns:
  - `tasks.json`, `plan.md`, `session_log.md`, `quality_gate_report.md`
  - `kickoff_prompts/` + per-slice `slices/<SLICE_ID>/kickoff_prompts/`
- Dependencies: all other PWS (needs final AC IDs + validation commands)
- Must encode:
  - `meta.checkpoint_boundaries` aligned to `pre-planning/ci_checkpoint_plan.md` (updated if slice skeleton changes)
  - Triad tasks per slice, each referencing `AC-<SLICE_ID>-*` IDs from slice specs

## Sequencing + gates (hard ordering)

1) `WDAP-PWS-contract` (DR selections + contract invariants)
2) In parallel:
   - `WDAP-PWS-provisioning_wiring`
   - `WDAP-PWS-runtime_fail_early`
3) `WDAP-PWS-docs_validation` (depends on both slice-spec seams)
4) `WDAP-PWS-tests_ci` (playbook + smoke scripts)
5) `WDAP-PWS-tasks_checkpoints` (tasks + checkpoints + plan + quality gate report template)

## Slice skeleton recommendations (execution planning)

Evidence: pack-derived lift estimates `9` slices (`pm_lift_pack.txt`) and emits `split_required:estimated_slices>3`.

Recommended edits to the draft skeleton in `pre-planning/minimal_spec_draft.md` (recommendations only; do not edit that file here):

- ADD `WDAP2` — Provisioning scripts + installer integration seam
  - Boundary: `scripts/**` surfaces listed in `pre-planning/impact_map.md` (world-enable/install flows; platform warmers; service templates).
  - Motivation: isolate high-conflict shared script edits (multiple overlapping packs) from core Rust behavior.

- ADD `WDAP3` — Docs + upstream contract reconciliation seam
  - Boundary: docs targets enumerated in `pre-planning/minimal_spec_draft.md` “Operator-doc update targets”.
  - Motivation: isolate cross-document contradiction resolution and reduce churn while core behavior stabilizes.

- Ordering recommendation (single explicit order):
  - `WDAP0` → `WDAP2` → `WDAP1` → `WDAP3`

- CI checkpoint plan update (if the above ADDs are accepted):
  - Update CP1 to end at `WDAP2` (provisioning + scripts seam)
  - Update CP2 to end at `WDAP3` (runtime fail-early + docs/contract reconciliation seam)

## Risks + unknowns (planning follow-ups)

- Windows posture is still a decision: `tasks.json` requires Windows behavior smoke parity; contract + playbooks must encode one deterministic supported/unsupported posture.
- DR-0001 and DR-0003 are high-churn: slice specs and tests depend on deterministic conflict policy and isolation/guard-rail posture.
- Shared script overlap is high (see `pre-planning/impact_map.md` cross-queue scan); prefer minimal edits per file and isolate changes by slice boundary.
- Upstream contract/doc contradictions exist today (runtime APT mutation vs provisioning-time-only); `WDAP3` must leave exactly one coherent truth.

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

