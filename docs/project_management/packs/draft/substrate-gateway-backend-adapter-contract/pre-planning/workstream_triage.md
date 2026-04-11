# Workstream Triage — substrate-gateway-backend-adapter-contract

## Decision summary
- Shared seam prefix: `SGBA`
- Planning pressure: high
- Recommended downstream planning workstreams: `3`
- Draft seam skeleton result: no change
- Downstream planning order:
  1. `SGBA-FWS-contract_surface`
  2. `SGBA-FWS-protocol_schema`
  3. `SGBA-FWS-parity_validation`

<!-- PM_FSE_WORKSTREAM_INDEX:BEGIN -->
```json
{
  "index_version": 1,
  "seam_prefix": "SGBA",
  "recommended_workstream_order": [
    "SGBA-FWS-contract_surface",
    "SGBA-FWS-protocol_schema",
    "SGBA-FWS-parity_validation"
  ],
  "draft_seam_order": [
    "SGBA-01",
    "SGBA-02",
    "SGBA-03"
  ],
  "workstreams": [
    {
      "id": "SGBA-FWS-contract_surface",
      "role": "Lock the stable backend-id contract, selection boundary, failure taxonomy, and published adapter visibility subset.",
      "depends_on": [],
      "assumes": [
        "Stable backend ids remain the only Substrate-facing backend identity.",
        "Gateway operator command ownership stays external to this pack.",
        "Gateway policy-evaluation ownership stays external to this pack."
      ],
      "owns": [
        "contract.md",
        "policy-spec.md",
        "seam-planning/adapter-selection-boundary.md",
        "contract-surfaces/stable_backend_id_contract",
        "contract-surfaces/adapter_selection_failure_taxonomy",
        "contract-surfaces/published_gateway_status_adapter_visibility_subset"
      ],
      "outcomes": [
        "One fixed backend-id contract reused by operator, policy, and inventory references.",
        "One fixed classification line for invalid selection, dependency unavailable, and policy denial.",
        "One fixed publication boundary for adapter-visible gateway status data."
      ]
    },
    {
      "id": "SGBA-FWS-protocol_schema",
      "role": "Define adapter dispatch lifecycle, schema inventory, capability subset, extension-key subset, and local-to-external handoff boundaries.",
      "depends_on": [
        "SGBA-FWS-contract_surface"
      ],
      "assumes": [
        "The published gateway status subset stays inside one external status-schema owner line.",
        "ADR-0017 remains the single owner for event-envelope semantics.",
        "ADR-0028 remains the single owner for canonical trace vocabulary."
      ],
      "owns": [
        "gateway-backend-adapter-protocol-spec.md",
        "gateway-backend-adapter-schema-spec.md",
        "seam-planning/adapter-protocol-and-schema.md",
        "protocol-surfaces/adapter_dispatch_lifecycle",
        "protocol-surfaces/capability_and_extension_key_subset",
        "protocol-surfaces/session_handle_boundary",
        "protocol-surfaces/event_and_trace_handoff_boundary"
      ],
      "outcomes": [
        "One fixed dispatch and normalization order from backend selection to adapter response emission.",
        "One fixed field inventory for adapter descriptor, capability advertisement, request, response, error, and session-handle objects.",
        "One fixed handoff line between local adapter translation and the external event and trace owners."
      ]
    },
    {
      "id": "SGBA-FWS-parity_validation",
      "role": "Lock cross-platform guarantees, ADR-0024 compatibility proof, and deterministic validation evidence for the adapter contract.",
      "depends_on": [
        "SGBA-FWS-contract_surface",
        "SGBA-FWS-protocol_schema"
      ],
      "assumes": [
        "The first two workstreams settle contract, protocol, schema, and handoff boundaries before parity proof closes.",
        "ADR-0040 remains the boundary owner for general gateway runtime ownership.",
        "The canonical checkpoint cadence stays anchored on CP1 and CP2."
      ],
      "owns": [
        "platform-parity-spec.md",
        "compatibility-spec.md",
        "manual_testing_playbook.md",
        "seam-planning/parity-and-validation.md",
        "validation-surfaces/cross_platform_guarantee_matrix",
        "validation-surfaces/adr_0024_supersession_proof",
        "validation-surfaces/final_doc_validation_gate"
      ],
      "outcomes": [
        "One fixed Linux, macOS, and Windows parity guarantee matrix for adapter-backed execution.",
        "One fixed compatibility statement that keeps ADR-0024 in historical-evidence status and preserves the no-second-control-plane invariant.",
        "One fixed validation gate covering owner-line review, ambiguity scans, and parity-proof evidence."
      ]
    }
  ]
}
```
<!-- PM_FSE_WORKSTREAM_INDEX:END -->

## Proposed downstream planning workstreams

### SGBA-FWS-contract_surface — Contract Surface and Selection Boundary
- Goal: lock the stable backend-id contract, failure buckets, trusted-input boundary, and the published adapter-visible gateway status subset.
- Owned surfaces:
  - `contract.md`
  - `policy-spec.md`
  - `seam-planning/adapter-selection-boundary.md`
  - `contract-surfaces/stable_backend_id_contract`
  - `contract-surfaces/adapter_selection_failure_taxonomy`
  - `contract-surfaces/published_gateway_status_adapter_visibility_subset`
- Dependencies: none
- Expected downstream deliverables:
  - one-owner-per-surface boundary table for backend-id semantics, allowlist gating, invalid selection, dependency unavailable, and policy denial
  - fixed publication boundary for any adapter-visible `status --json` subset
  - downstream seam-planning packet for `seam-planning/adapter-selection-boundary.md`

### SGBA-FWS-protocol_schema — Adapter Protocol and Schema Boundary
- Goal: define adapter dispatch lifecycle, request and response normalization order, adopted capability subset, extension-key subset, error object, and session-handle boundary without widening ADR-0017 or ADR-0028 ownership.
- Owned surfaces:
  - `gateway-backend-adapter-protocol-spec.md`
  - `gateway-backend-adapter-schema-spec.md`
  - `seam-planning/adapter-protocol-and-schema.md`
  - `protocol-surfaces/adapter_dispatch_lifecycle`
  - `protocol-surfaces/capability_and_extension_key_subset`
  - `protocol-surfaces/session_handle_boundary`
  - `protocol-surfaces/event_and_trace_handoff_boundary`
- Dependencies:
  - `SGBA-FWS-contract_surface`
- Expected downstream deliverables:
  - exact field inventory for adapter descriptor, capability advertisement, request payload, response payload, adapter error object, and session-handle facets
  - exact owner line for local adapter translation versus ADR-0017 event-envelope ownership
  - exact owner line for local adapter diagnostics versus ADR-0028 trace vocabulary
  - downstream seam-planning packet for `seam-planning/adapter-protocol-and-schema.md`

### SGBA-FWS-parity_validation — Platform Parity, Compatibility, and Validation
- Goal: prove the adapter contract is additive, cross-platform, and reviewable through deterministic parity and document-validation gates.
- Owned surfaces:
  - `platform-parity-spec.md`
  - `compatibility-spec.md`
  - `manual_testing_playbook.md`
  - `seam-planning/parity-and-validation.md`
  - `validation-surfaces/cross_platform_guarantee_matrix`
  - `validation-surfaces/adr_0024_supersession_proof`
  - `validation-surfaces/final_doc_validation_gate`
- Dependencies:
  - `SGBA-FWS-contract_surface`
  - `SGBA-FWS-protocol_schema`
- Expected downstream deliverables:
  - fixed platform guarantee table for Linux, macOS, and Windows
  - explicit compatibility and supersession proof for ADR-0024 and ADR-0040 boundaries
  - final validation gate covering document review, ambiguity scan, and parity proof
  - downstream seam-planning packet for `seam-planning/parity-and-validation.md`

## Sequencing and gates

### Hard ordering constraints
- `SGBA-FWS-contract_surface` closes first because it fixes backend-id semantics, failure buckets, trusted-input boundaries, and the published gateway status subset.
- `SGBA-FWS-protocol_schema` closes second because protocol and schema work depends on the stable selection contract and the published visibility boundary.
- `SGBA-FWS-parity_validation` closes last because parity proof and compatibility proof depend on settled contract, protocol, schema, and handoff boundaries.

### Parallelizable evidence collection
- `SGBA-FWS-protocol_schema` starts external-owner inventory review and field inventory assembly while `SGBA-FWS-contract_surface` locks the publication boundary.
- `SGBA-FWS-parity_validation` starts runtime-parity evidence collection and compatibility-proof assembly while the first two workstreams close their planning packets.
- Final planning sign-off follows the hard ordering constraints above.

### CI checkpoint implications
- `CP1` covers the lock point for `SGBA-01` and `SGBA-02`.
- `CP1` gates:
  - document validation
  - doc-lint and ambiguity scan
  - compile parity on Linux, macOS, and Windows when downstream changes touch runtime or status surfaces
- `CP2` covers the final lock point for `SGBA-03`.
- `CP2` gates:
  - doc-lint and ambiguity scan
  - compile parity on Linux, macOS, and Windows
  - targeted gateway `status`, `sync`, `restart`, and `status --json` validation after the published subset is fixed
  - compatibility-proof review for ADR-0024 supersession and ADR-0040 boundary alignment

## Draft seam skeleton recommendations
- No change. Keep the draft seam order from `pre-planning/minimal_spec_draft.md`:
  - `SGBA-01` / `adapter-selection-boundary`
  - `SGBA-02` / `adapter-protocol-and-schema`
  - `SGBA-03` / `parity-and-validation`
- Workstream mapping:
  - `SGBA-01` -> `SGBA-FWS-contract_surface`
  - `SGBA-02` -> `SGBA-FWS-protocol_schema`
  - `SGBA-03` -> `SGBA-FWS-parity_validation`
- `SPLIT SGBA-02` if the published `status --json` subset, the adopted Universal Agent API subset, and the session-handle or error field inventory no longer fit one bounded planning packet.

## Risks and unknowns
- The exact published `status --json` subset for backend capability visibility remains open.
- The exact adopted Universal Agent API subset remains open:
  - capability ids
  - extension keys
  - session-handle facet fields
  - bounded adapter error detail
- The local-to-external owner line is now pinned by `C-03`; the remaining open items are the schema subset bullets above, while the follow-ups below are unrelated packaging or evidence decisions.
- ADR-0041 still carries `packs/active/...` path references while this checkout uses `packs/implemented/...`.
- ADR-0040 alignment stays in evidence-only status until downstream planning records a direct edit or confirms no direct edit.

## Evidence links

### Stable step sentinels
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/logs/spec-manifest/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/logs/impact-map/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/logs/min-spec-draft/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/logs/CI-checkpoint/last_message.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/logs/workstream-triage/planning_pressure_assessment.md`

### Canonical artifacts used
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/minimal_spec_draft.md`
- `docs/project_management/packs/draft/substrate-gateway-backend-adapter-contract/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`

## Follow-ups
- Fix one explicit published backend-visibility subset for `status --json`.
- Fix one explicit Universal Agent API subset for capability ids, extension keys, session-handle facets, and bounded adapter error detail.
- Confirm whether ADR-0040 planning-pack docs stay evidence-only or enter the downstream touch set as direct alignment edits.
