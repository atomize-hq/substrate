# llm-and-agent-identity-tuple-and-deployment-posture — workstream triage

## Canonical inputs used

- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`

## Evidence links

- Stable sentinels:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/spec-manifest/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/impact-map/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/min-spec-draft/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/CI-checkpoint/last_message.md`
- Canonical artifacts relied on:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`

## Boundary decision

- Accepted slice prefix: `LAITDP`
- Accepted slice order: `LAITDP0`, `LAITDP1`, `LAITDP2`
- Accepted workstream count: `5`
- Boundary rationale:
  - `LAITDP0` isolates operator wording, token grammar, and absence semantics.
  - `LAITDP1` isolates routing-hint policy, additive status publication, redaction, and trace vocabulary alignment.
  - `LAITDP2` isolates platform parity, compatibility posture, and manual validation evidence.

<!-- PM_PWS_INDEX:BEGIN -->
```json
{
  "pws_index_version": 2,
  "slice_prefix": "LAITDP",
  "accepted_slice_order": [
    "LAITDP0",
    "LAITDP1",
    "LAITDP2"
  ],
  "draft_slice_order": [
    "LAITDP0",
    "LAITDP1",
    "LAITDP2"
  ],
  "pws": [
    {
      "id": "LAITDP-PWS-contract",
      "role": "contract",
      "depends_on": [],
      "assumes": [
        "ADR-0027 remains the sole owner of config and policy key paths.",
        "backend_id remains an adapter selector separate from tuple meaning."
      ],
      "owns": [
        "pre-planning/spec_manifest.md",
        "contract.md",
        "identity-tuple-schema-spec.md"
      ]
    },
    {
      "id": "LAITDP-PWS-schema_inventory",
      "role": "schema_inventory",
      "depends_on": [
        "LAITDP-PWS-contract"
      ],
      "assumes": [
        "Gateway status-schema ownership stays external to this pack.",
        "ADR-0043 remains the only owner of tuple-axis policy keys under llm.constraints."
      ],
      "owns": [
        "policy-spec.md",
        "telemetry-spec.md"
      ]
    },
    {
      "id": "LAITDP-PWS-docs_validation",
      "role": "docs_validation",
      "depends_on": [
        "LAITDP-PWS-contract",
        "LAITDP-PWS-schema_inventory"
      ],
      "assumes": [
        "Linux, macOS, and Windows keep one operator-visible tuple and placement semantics contract."
      ],
      "owns": [
        "platform-parity-spec.md",
        "compatibility-spec.md",
        "manual_testing_playbook.md"
      ]
    },
    {
      "id": "LAITDP-PWS-implementation_seams",
      "role": "slice_spec",
      "depends_on": [
        "LAITDP-PWS-contract",
        "LAITDP-PWS-schema_inventory",
        "LAITDP-PWS-docs_validation"
      ],
      "assumes": [
        "The accepted slice order keeps one dominant behavior delta per slice."
      ],
      "owns": [
        "slices/LAITDP0/LAITDP0-spec.md",
        "slices/LAITDP1/LAITDP1-spec.md",
        "slices/LAITDP2/LAITDP2-spec.md"
      ]
    },
    {
      "id": "LAITDP-PWS-tasks_checkpoints",
      "role": "tasks_checkpoints",
      "depends_on": [
        "LAITDP-PWS-contract",
        "LAITDP-PWS-schema_inventory",
        "LAITDP-PWS-docs_validation",
        "LAITDP-PWS-implementation_seams"
      ],
      "assumes": [
        "Checkpoint cadence stays split between CP1 after LAITDP1 and CP2 after LAITDP2 unless ci_checkpoint_plan.md changes first."
      ],
      "owns": [
        "pre-planning/ci_checkpoint_plan.md",
        "plan.md",
        "tasks.json",
        "session_log.md",
        "quality_gate_report.md",
        "kickoff_prompts/",
        "slices/LAITDP0/kickoff_prompts/",
        "slices/LAITDP1/kickoff_prompts/",
        "slices/LAITDP2/kickoff_prompts/"
      ]
    }
  ]
}
```
<!-- PM_PWS_INDEX:END -->

## Proposed planning workstreams

### LAITDP-PWS-contract — contract and tuple schema lock
- Goal: lock tuple meaning, token grammar, placement-posture vocabulary, router identity, and tuple absence rules before full planning touches policy, status, trace, or parity work.
- Owned surfaces:
  - `pre-planning/spec_manifest.md`
  - `contract.md`
  - `identity-tuple-schema-spec.md`
- Dependencies: none.
- Proposed slices/triads:
  - `LAITDP0-{code,test,integ}`

### LAITDP-PWS-schema_inventory — policy, status, and trace alignment
- Goal: lock routing-hint evaluation semantics, additive tuple publication, redaction lines, and the ADR-0043 policy handoff while preserving the external owner line for gateway status JSON and canonical trace vocabulary.
- Owned surfaces:
  - `policy-spec.md`
  - `telemetry-spec.md`
- Dependencies:
  - `LAITDP-PWS-contract`
- Proposed slices/triads:
  - `LAITDP1-{code,test,integ}`

### LAITDP-PWS-docs_validation — parity, compatibility, and validation closure
- Goal: lock platform parity guarantees, compatibility wording, and the validation matrix for operator, status, trace, and parity evidence surfaces.
- Owned surfaces:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
- Dependencies:
  - `LAITDP-PWS-contract`
  - `LAITDP-PWS-schema_inventory`
- Proposed slices/triads:
  - `LAITDP2-{code,test,integ}`

### LAITDP-PWS-implementation_seams — execution-ready slice specs
- Goal: convert the accepted three-slice spine into execution-ready slice specs with disjoint acceptance criteria.
- Owned surfaces:
  - `slices/LAITDP0/LAITDP0-spec.md`
  - `slices/LAITDP1/LAITDP1-spec.md`
  - `slices/LAITDP2/LAITDP2-spec.md`
- Dependencies:
  - `LAITDP-PWS-contract`
  - `LAITDP-PWS-schema_inventory`
  - `LAITDP-PWS-docs_validation`
- Proposed slices/triads:
  - `LAITDP0-{code,test,integ}`
  - `LAITDP1-{code,test,integ}`
  - `LAITDP2-{code,test,integ}`

### LAITDP-PWS-tasks_checkpoints — single writer for tasks and gates
- Goal: translate the accepted slice order into `plan.md`, `tasks.json`, kickoff prompts, session logging, and checkpoint wiring.
- Owned surfaces:
  - `pre-planning/ci_checkpoint_plan.md`
  - `plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
  - `kickoff_prompts/`
  - `slices/LAITDP0/kickoff_prompts/`
  - `slices/LAITDP1/kickoff_prompts/`
  - `slices/LAITDP2/kickoff_prompts/`
- Dependencies:
  - `LAITDP-PWS-contract`
  - `LAITDP-PWS-schema_inventory`
  - `LAITDP-PWS-docs_validation`
  - `LAITDP-PWS-implementation_seams`
- Proposed slices/triads:
  - `LAITDP0-{code,test,integ}`
  - `LAITDP1-{code,test,integ}`
  - `LAITDP2-{code,test,integ}`
  - `CP1-ci-checkpoint`
  - `CP2-ci-checkpoint`

## Sequencing and gates

- Hard ordering: `LAITDP-PWS-contract` lands first.
- Hard ordering: `LAITDP-PWS-schema_inventory` lands after `LAITDP-PWS-contract`.
- Hard ordering: `LAITDP-PWS-docs_validation` lands after contract and schema inventory lock.
- Hard ordering: `LAITDP-PWS-implementation_seams` lands after contract, schema inventory, and docs-validation surfaces are pinned.
- Hard ordering: `LAITDP-PWS-tasks_checkpoints` lands last.
- CI checkpoint implication: `CP1` groups `LAITDP0` and `LAITDP1`.
- CI checkpoint implication: `CP2` isolates `LAITDP2`.
- CI checkpoint implication: `tasks.json` must set `meta.checkpoint_boundaries=["LAITDP1","LAITDP2"]`.

## Risks and unknowns

- The host-only router identity conflict around `direct_provider_path` remains open.
- The absence semantics for `provider` and `auth_authority` remain open for pre-provider-selection paths and for pure-agent follow-on flows.
- The exact status and diagnostics field family for tuple publication outside `client_wiring.*` remains open against `docs/contracts/gateway/status-schema.md`.
- The exact trace field family and field placement remains open against ADR-0028.
- The boundary line with ADR-0044 and ADR-0045 for pure-agent and toolbox tuple publication remains open.
- WSL scope remains a follow-up until downstream runtime review names the concrete surface set.
- High-churn boundary: additive tuple publication across `telemetry-spec.md`, `docs/contracts/gateway/status-schema.md`, ADR-0028, and the typed-model surface named in `impact_map.md`.
- High-churn boundary: compatibility and rollout wording across `compatibility-spec.md`, ADR-0040, ADR-0041, and ADR-0046.

## Slice skeleton recommendations

- `NO CHANGE`: keep `LAITDP0`, `LAITDP1`, and `LAITDP2` in the current order.
- `LAITDP0` owns contract and tuple-schema lock.
- `LAITDP1` owns policy and observability alignment lock.
- `LAITDP2` owns platform rollout and validation lock.
- `ACCEPTED SLICE ORDER`: `LAITDP0`, `LAITDP1`, `LAITDP2`.

## Follow-ups

- Freeze host-only router identity text in contract and schema planning before policy and telemetry planning finalizes provider-authority wording.
- Freeze the status and diagnostics tuple field family outside `client_wiring.*` before downstream planning touches typed-model or status-reader surfaces.
- Freeze the trace field family and the ADR-0044 and ADR-0045 boundary before downstream planning touches pure-agent or toolbox tuple publication.
- Confirm WSL scope after downstream runtime review names the concrete surface set.
