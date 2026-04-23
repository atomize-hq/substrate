# Workstream Triage

## Status
- Phase: `phase_b`
- Shared seam prefix: `LAITDP`
- Recommended downstream planning workstream count: `3`
- Draft seam skeleton action: `NO CHANGE`
- Canonical inputs reread:
  - `pre-planning/spec_manifest.md`
  - `pre-planning/impact_map.md`
  - `pre-planning/minimal_spec_draft.md`
  - `pre-planning/ci_checkpoint_plan.md`
- Planning-pressure refresh:
  - overall pressure remains `medium_high`
  - `CP1` groups `LAITDP-01` and `LAITDP-02`
  - `CP2` isolates `LAITDP-03`

## Triage Summary
- `LAITDP-FWS-contract_surface` opens first and resolves the semantic blockers that fan into every later lane.
- `LAITDP-FWS-policy_observability` opens after the contract freeze and closes the status-schema, trace-vocabulary, redaction, and ADR-0043 handoff boundaries captured in `CP1`.
- `LAITDP-FWS-platform_rollout` opens after the contract freeze, runs in parallel with the publication lane on parity and compatibility framing, and closes in `CP2` after publication field placement freezes.
- The draft seam skeleton already matches the required downstream seam-planning doc set in `spec_manifest.md` and the checkpoint groups in `ci_checkpoint_plan.md`.

<!-- PM_FSE_WORKSTREAM_INDEX:BEGIN -->
```json
{
  "index_version": 1,
  "seam_prefix": "LAITDP",
  "recommended_workstream_order": [
    "LAITDP-FWS-contract_surface",
    "LAITDP-FWS-policy_observability",
    "LAITDP-FWS-platform_rollout"
  ],
  "draft_seam_order": [
    "LAITDP-01",
    "LAITDP-02",
    "LAITDP-03"
  ],
  "workstreams": [
    {
      "id": "LAITDP-FWS-contract_surface",
      "role": "Freeze tuple meaning, token grammar, placement posture, router identity, and tuple absence rules.",
      "depends_on": [],
      "assumes": [
        "ADR-0027 remains the sole owner of config and policy key paths.",
        "backend_id remains an adapter selector separate from tuple meaning."
      ],
      "owns": [
        "contract.md",
        "identity-tuple-schema-spec.md",
        "seam-planning/identity-contract-and-schema.md",
        "contract surface: operator tuple meaning and placement-posture vocabulary",
        "contract surface: tuple token grammar and absence semantics"
      ],
      "outcomes": [
        "Resolved ruling for host-only router identity versus direct_provider_path wording.",
        "Frozen tuple grammar, placement-posture vocabulary, and tuple absence semantics.",
        "Frozen no-new-command, no-new-config, and no-new-exit-code boundary text."
      ]
    },
    {
      "id": "LAITDP-FWS-policy_observability",
      "role": "Freeze routing-hint policy semantics, additive publication rules, redaction lines, and the ADR-0043 handoff boundary.",
      "depends_on": [
        "LAITDP-FWS-contract_surface"
      ],
      "assumes": [
        "Gateway status-schema ownership stays external to this pack.",
        "ADR-0043 remains the only owner of tuple-axis policy keys under llm.constraints."
      ],
      "owns": [
        "policy-spec.md",
        "telemetry-spec.md",
        "seam-planning/policy-and-observability-alignment.md",
        "contract surface: additive status and diagnostics tuple publication outside client_wiring.*",
        "contract surface: additive trace tuple publication and redaction rules"
      ],
      "outcomes": [
        "Frozen routing-hint evaluation boundary for accepted, rejected, and denied paths.",
        "Frozen additive status and diagnostics field placement outside client_wiring.*.",
        "Frozen trace field placement, redaction rules, and ADR-0043 handoff text."
      ]
    },
    {
      "id": "LAITDP-FWS-platform_rollout",
      "role": "Freeze platform parity guarantees, compatibility retirement posture, and the manual validation matrix.",
      "depends_on": [
        "LAITDP-FWS-contract_surface"
      ],
      "assumes": [
        "Policy and observability planning freezes the publication surfaces before parity evidence closes.",
        "Linux, macOS, and Windows keep one operator-visible tuple and placement semantics contract."
      ],
      "owns": [
        "platform-parity-spec.md",
        "compatibility-spec.md",
        "manual_testing_playbook.md",
        "seam-planning/platform-rollout-and-validation.md",
        "contract surface: parity evidence matrix",
        "contract surface: overloaded-backend-language retirement posture"
      ],
      "outcomes": [
        "Frozen Linux, macOS, and Windows parity assertions for tuple and placement semantics.",
        "Frozen compatibility language for retiring overloaded backend labels in new docs and diagnostics.",
        "Frozen manual validation matrix and runtime review result for WSL scope."
      ]
    }
  ]
}
```
<!-- PM_FSE_WORKSTREAM_INDEX:END -->

## Proposed Downstream Planning Workstreams

### LAITDP-FWS-contract_surface — Contract Surface and Tuple Semantics
- Goal: lock tuple meaning, token grammar, placement-posture vocabulary, router identity, and tuple absence rules before downstream planning touches policy, status, trace, or parity closeout work.
- Owned surfaces:
  - `contract.md`
  - `identity-tuple-schema-spec.md`
  - `seam-planning/identity-contract-and-schema.md`
  - contract surfaces for operator tuple meaning, placement posture, and tuple absence semantics
- Dependencies: none
- Expected downstream deliverables:
  - a seam-planning doc that freezes the semantic boundary for operator wording and schema work
  - a resolved ruling for `direct_provider_path` versus host-only router identity
  - frozen absence semantics for `provider` and `auth_authority`

### LAITDP-FWS-policy_observability — Policy, Status, and Trace Alignment
- Goal: lock routing-hint evaluation semantics, additive tuple publication, redaction lines, and the ADR-0043 policy handoff while preserving the external owner line for gateway status JSON and canonical trace vocabulary.
- Owned surfaces:
  - `policy-spec.md`
  - `telemetry-spec.md`
  - `seam-planning/policy-and-observability-alignment.md`
  - contract surfaces for additive status and diagnostics publication outside `client_wiring.*`
  - contract surfaces for additive trace publication and tuple redaction
- Dependencies:
  - `LAITDP-FWS-contract_surface`
- Expected downstream deliverables:
  - a seam-planning doc that freezes the external owner line for gateway status JSON and trace vocabulary
  - frozen additive tuple field placement in status, diagnostics, and trace surfaces
  - explicit ADR-0043 boundary text that blocks tuple-semantic drift

### LAITDP-FWS-platform_rollout — Platform Parity, Compatibility, and Validation
- Goal: lock platform parity guarantees, compatibility wording, and the validation matrix for operator, status, trace, and parity evidence surfaces.
- Owned surfaces:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
  - `seam-planning/platform-rollout-and-validation.md`
  - contract surfaces for parity evidence and compatibility retirement language
- Dependencies:
  - `LAITDP-FWS-contract_surface`
- Expected downstream deliverables:
  - a seam-planning doc that freezes the parity proof boundary for Linux, macOS, and Windows
  - frozen compatibility wording for overloaded backend-label retirement in new docs and diagnostics
  - a manual validation matrix that checks status, trace, and operator wording against the frozen semantic contract

## Sequencing and Gates
- Hard ordering constraint 1: `LAITDP-FWS-contract_surface` closes first. This lane freezes `direct_provider_path`, tuple absence semantics, token grammar, placement posture, and the no-new-surface boundary.
- Hard ordering constraint 2: `LAITDP-FWS-policy_observability` opens after the contract freeze and closes before `CP1` exits. This lane freezes status and trace publication boundaries and the ADR-0043 handoff.
- Hard ordering constraint 3: `LAITDP-FWS-platform_rollout` opens after the contract freeze, runs in parallel with `LAITDP-FWS-policy_observability`, and closes after the publication freeze lands.
- CI checkpoint implication 1: `CP1` groups `LAITDP-01` and `LAITDP-02`. Run document validation and the external-alignment gate against `docs/contracts/substrate-gateway-status-schema.md` and ADR-0028 before downstream planning depends on widened publication fields.
- CI checkpoint implication 2: `CP2` isolates `LAITDP-03`. Run the manual validation playbook review after parity, compatibility, and validation evidence land.

## Draft Seam Skeleton Recommendations
- No change.
- `NO CHANGE`: keep `LAITDP-01`, `LAITDP-02`, and `LAITDP-03` in the current order.
- Reason: `minimal_spec_draft.md` already maps one-to-one to the downstream seam-planning doc set in `spec_manifest.md`, and `ci_checkpoint_plan.md` groups those same seams without exposing a missing boundary.

## Risks and Unknowns
- The host-only router identity conflict around `direct_provider_path` remains open.
- The absence semantics for `provider` and `auth_authority` remain open for pre-provider-selection paths and for pure-agent follow-on flows.
- The exact status and diagnostics field family for tuple publication outside `client_wiring.*` remains open against `docs/contracts/substrate-gateway-status-schema.md`.
- The exact trace field family and field placement remains open against ADR-0028.
- The boundary line with ADR-0044 and ADR-0045 for pure-agent and toolbox tuple publication remains open.
- WSL scope remains a follow-up until downstream runtime review names the concrete surface set.
- High-churn boundary: additive tuple publication across `telemetry-spec.md`, `docs/contracts/substrate-gateway-status-schema.md`, ADR-0028, and the typed-model surface named in `impact_map.md`.
- High-churn boundary: compatibility and rollout wording across `compatibility-spec.md`, ADR-0040, ADR-0041, and ADR-0046.

## Evidence Links
- Stable sentinels:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/spec-manifest/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/impact-map/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/min-spec-draft/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/CI-checkpoint/last_message.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/logs/workstream-triage/planning_pressure_assessment.md`
- Canonical artifacts relied on:
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/impact_map.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`
  - `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/ci_checkpoint_plan.md`

## Follow-ups
- Freeze host-only router identity text in contract and schema planning before policy and telemetry planning finalizes provider-authority wording.
- Freeze the status and diagnostics tuple field family outside `client_wiring.*` before downstream planning touches typed-model or status-reader surfaces.
- Freeze the trace field family and the ADR-0044 and ADR-0045 boundary before downstream planning touches pure-agent or toolbox tuple publication.
- Confirm WSL scope after downstream runtime review names the concrete surface set.
