# Workstream Triage — agent-hub-core-successor-identity-tuple-compatible

## Decision
- Keep the accepted slice order at `AHCSITC0`, `AHCSITC1`, `AHCSITC2`, `AHCSITC3`.
- Keep the draft slice skeleton unchanged.
- Split full planning into seven pack-internal planning workstreams so contract, protocol, policy, parity, validation, slice specs, and task wiring move without file-owner overlap.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "AHCSITC",
  "accepted_slice_order": [
    "AHCSITC0",
    "AHCSITC1",
    "AHCSITC2",
    "AHCSITC3"
  ],
  "pws": [
    {
      "id": "AHCSITC-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0042 tuple semantics remain authoritative",
        "ADR-0043 nested gateway policy boundaries remain authoritative"
      ],
      "owns": [
        "contract.md"
      ]
    },
    {
      "id": "AHCSITC-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "AHCSITC-PWS-contract"
      ],
      "assumes": [
        "ADR-0017 event vocabulary remains authoritative",
        "ADR-0028 trace families remain authoritative"
      ],
      "owns": [
        "agent-hub-session-protocol-spec.md"
      ]
    },
    {
      "id": "AHCSITC-PWS-runtime_fail_early",
      "role": "runtime_fail_early",
      "depends_on": [
        "AHCSITC-PWS-contract",
        "AHCSITC-PWS-schema_inventory"
      ],
      "assumes": [
        "gateway nested-request approval stays under substrate_gateway ownership"
      ],
      "owns": [
        "policy-spec.md",
        "telemetry-spec.md"
      ]
    },
    {
      "id": "AHCSITC-PWS-world_agent_profile",
      "role": "world_agent_profile",
      "depends_on": [
        "AHCSITC-PWS-contract",
        "AHCSITC-PWS-schema_inventory"
      ],
      "assumes": [
        "cross-platform parity stays aligned with the existing Linux, macOS, and Windows posture"
      ],
      "owns": [
        "platform-parity-spec.md",
        "compatibility-spec.md"
      ]
    },
    {
      "id": "AHCSITC-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "AHCSITC-PWS-runtime_fail_early",
        "AHCSITC-PWS-world_agent_profile"
      ],
      "assumes": [
        "manual validation remains the canonical proof path for list, status, doctor, and nested-record checks"
      ],
      "owns": [
        "manual_testing_playbook.md"
      ]
    },
    {
      "id": "AHCSITC-PWS-implementation_seams",
      "role": "implementation",
      "depends_on": [
        "AHCSITC-PWS-contract",
        "AHCSITC-PWS-schema_inventory",
        "AHCSITC-PWS-runtime_fail_early",
        "AHCSITC-PWS-world_agent_profile"
      ],
      "assumes": [
        "execution stays in existing crates named by the impact map"
      ],
      "owns": [
        "slices/AHCSITC0/AHCSITC0-spec.md",
        "slices/AHCSITC1/AHCSITC1-spec.md",
        "slices/AHCSITC2/AHCSITC2-spec.md",
        "slices/AHCSITC3/AHCSITC3-spec.md"
      ]
    },
    {
      "id": "AHCSITC-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "AHCSITC-PWS-docs_validation",
        "AHCSITC-PWS-implementation_seams"
      ],
      "assumes": [
        "checkpoint boundaries stay at AHCSITC2 and AHCSITC3"
      ],
      "owns": [
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/AHCSITC0/kickoff_prompts/",
        "slices/AHCSITC1/kickoff_prompts/",
        "slices/AHCSITC2/kickoff_prompts/",
        "slices/AHCSITC3/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Lift Evidence
- Strict-pack status: `tasks.json` sets `meta.slice_spec_version = 2`, so `pm-lift-pack` is authoritative for pack-derived lift.
- Pack lift result:
  - `lift_score = 125`
  - `estimated_slices = 11`
  - `confidence = low`
  - trigger set includes `likely_split:crates_touched>2`, `likely_split:touch_files_sum>12`, and `split_required:estimated_slices>3`
- Impact-map touch counts:
  - create: `16`
  - edit: `26`
  - deprecate: `1`
  - crates touched: `6`
- Intake lift result: failed because ADR-0044 uses an invalid Lift Vector field type for `risk.unknowns_high`.

## Proposed Planning Workstreams

### AHCSITC-PWS-contract — Contract and operator identity lock
- Goal: lock `substrate agent list`, `substrate agent status`, and `substrate agent doctor` semantics, including additive compatibility wording for the plural validate leaf.
- Owned surfaces:
  - `contract.md`
- Dependencies: none
- Proposed slices and triads:
  - `AHCSITC0-{code,test,integ}`

### AHCSITC-PWS-schema_inventory — Session protocol and field inventory lock
- Goal: lock capability descriptors, session handles, lifecycle state names, and shared identity-field inventory used by policy, telemetry, and slice specs.
- Owned surfaces:
  - `agent-hub-session-protocol-spec.md`
- Dependencies:
  - `AHCSITC-PWS-contract`
- Proposed slices and triads:
  - `AHCSITC1-{code,test,integ}`

### AHCSITC-PWS-runtime_fail_early — Fail-closed routing and telemetry publication lock
- Goal: lock orchestrator eligibility, deny posture, nested gateway reuse, pure-agent omission rules, and correlated nested-record publication.
- Owned surfaces:
  - `policy-spec.md`
  - `telemetry-spec.md`
- Dependencies:
  - `AHCSITC-PWS-contract`
  - `AHCSITC-PWS-schema_inventory`
- Proposed slices and triads:
  - `AHCSITC2-{code,test,integ}`

### AHCSITC-PWS-world_agent_profile — Platform parity and compatibility closure
- Goal: lock host-orchestrator versus world-member placement guarantees, `world_id` and `world_generation` visibility, parity evidence, and ADR-0025 supersession wording.
- Owned surfaces:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
- Dependencies:
  - `AHCSITC-PWS-contract`
  - `AHCSITC-PWS-schema_inventory`
- Proposed slices and triads:
  - `AHCSITC3-{code,test,integ}`

### AHCSITC-PWS-docs_validation — Manual validation and one-owner proof
- Goal: lock deterministic operator validation for list, status, doctor, nested records, and cross-doc owner alignment.
- Owned surfaces:
  - `manual_testing_playbook.md`
- Dependencies:
  - `AHCSITC-PWS-runtime_fail_early`
  - `AHCSITC-PWS-world_agent_profile`
- Proposed slices and triads:
  - `AHCSITC3-{code,test,integ}`

### AHCSITC-PWS-implementation_seams — Execution-ready slice specs
- Goal: convert the accepted slice order into slice specs with explicit acceptance criteria and file-owner boundaries.
- Owned surfaces:
  - `slices/AHCSITC0/AHCSITC0-spec.md`
  - `slices/AHCSITC1/AHCSITC1-spec.md`
  - `slices/AHCSITC2/AHCSITC2-spec.md`
  - `slices/AHCSITC3/AHCSITC3-spec.md`
- Dependencies:
  - `AHCSITC-PWS-contract`
  - `AHCSITC-PWS-schema_inventory`
  - `AHCSITC-PWS-runtime_fail_early`
  - `AHCSITC-PWS-world_agent_profile`
- Proposed slices and triads:
  - `AHCSITC0-{code,test,integ}`
  - `AHCSITC1-{code,test,integ}`
  - `AHCSITC2-{code,test,integ}`
  - `AHCSITC3-{code,test,integ}`

### AHCSITC-PWS-tasks_checkpoints — Single writer for plan, tasks, and kickoff wiring
- Goal: translate the accepted slice order into plan wiring, checkpoint tasks, kickoff prompts, and pack-level planning evidence with one writer for `tasks.json`.
- Owned surfaces:
  - `plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `kickoff_prompts/`
  - `slices/AHCSITC0/kickoff_prompts/`
  - `slices/AHCSITC1/kickoff_prompts/`
  - `slices/AHCSITC2/kickoff_prompts/`
  - `slices/AHCSITC3/kickoff_prompts/`
- Dependencies:
  - `AHCSITC-PWS-docs_validation`
  - `AHCSITC-PWS-implementation_seams`
- Proposed slices and triads:
  - `AHCSITC0-{code,test,integ}`
  - `AHCSITC1-{code,test,integ}`
  - `AHCSITC2-{code,test,integ}`
  - `AHCSITC3-{code,test,integ}`
  - `CP1-ci-checkpoint`
  - `CP2-ci-checkpoint`

## Sequencing And Gates
- Hard ordering:
  - `AHCSITC-PWS-contract` starts first.
  - `AHCSITC-PWS-schema_inventory` starts after `AHCSITC-PWS-contract`.
  - `AHCSITC-PWS-runtime_fail_early` and `AHCSITC-PWS-world_agent_profile` start after `AHCSITC-PWS-schema_inventory` and can run in parallel.
  - `AHCSITC-PWS-docs_validation` starts after runtime and parity closures are locked.
  - `AHCSITC-PWS-implementation_seams` starts after contract, schema inventory, runtime fail-early, and world-profile surfaces are locked.
  - `AHCSITC-PWS-tasks_checkpoints` runs last.
- Sequencing gates:
  - Gate 1: contract wording for `backend_id`, pure-agent omission, nested-record presence, and CLI namespace is final.
  - Gate 2: session and routing vocabulary for capability descriptors, session handles, `world_id`, and `world_generation` is final.
  - Gate 3: fail-closed policy and telemetry publication rules are final.
  - Gate 4: parity, compatibility, and manual-validation wording is final.
  - Gate 5: slice specs are final before `tasks.json` and kickoff prompts are authored.
- CI checkpoint implications:
  - Keep `CP1` after `AHCSITC2`.
  - Keep `CP2` after `AHCSITC3`.
  - `AHCSITC-PWS-tasks_checkpoints` owns `tasks.json` checkpoint wiring and kickoff prompt generation from those boundaries.

## Slice Skeleton Recommendations
- `NO CHANGE`
- `ADD`: none
- `SPLIT`: none
- `MERGE`: none
- `RENAME`: none
- Rationale:
  - The four draft slices already isolate the four execution seams named by the canonical inputs.
  - The lift spike comes from pack-wide doc and crate breadth, not from a missing fifth execution seam.
  - Seven planning workstreams absorb that breadth without introducing extra slice overlap before full planning pins task ownership.

## Risks And Unknowns
- High-churn seam: `backend_id`, `client`, `router`, `protocol`, `provider`, and `auth_authority` cross `contract.md`, `policy-spec.md`, `telemetry-spec.md`, and CLI JSON output.
- High-churn seam: `world_id` and `world_generation` publication crosses protocol, telemetry, parity, and manual validation.
- Open gap: ADR-0044 still carries `crates/agent-hub` wording, while the impact map locks this feature to existing crates under `crates/shell`, `crates/common`, `crates/trace`, and `crates/agent-api-*`.
- Open gap: machine-readable output for `substrate agent list` and `substrate agent status` needs an explicit owner line in `contract.md`.
- Open gap: `pm-lift-intake` fails until ADR-0044 changes `risk.unknowns_high` to `integer` or `null`.

## Evidence Links
- Canonical pre-planning artifacts:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/ci_checkpoint_plan.md`
- Stable sentinels:
  - `logs/spec-manifest/last_message.md`
  - `logs/impact-map/last_message.md`
  - `logs/min-spec-draft/last_message.md`
  - `logs/CI-checkpoint/last_message.md`
- Lift evidence:
  - `logs/workstream-triage/pm_lift_pack.txt`
  - `logs/workstream-triage/pm_lift_pack.json`

## Follow-ups
- Repair the ADR-0044 Lift Vector field type for `risk.unknowns_high`, then rerun `pm-lift-intake`.
- Mirror the existing-crate implementation placement in ADR-0044 before full planning promotes slice specs.
- Pin the machine-readable list and status output contract inside `contract.md` during `AHCSITC-PWS-contract`.
- Pin the exact publication path for `world_generation` inside `telemetry-spec.md` during `AHCSITC-PWS-runtime_fail_early`.
