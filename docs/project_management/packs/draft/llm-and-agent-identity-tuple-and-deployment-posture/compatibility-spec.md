# llm-and-agent-identity-tuple-and-deployment-posture — compatibility spec

Owner standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Scope

- This spec is authoritative for the overloaded-backend-label retirement posture and the rollout end state for new operator-facing docs, status wording, and diagnostics introduced by ADR-0042.
- This spec owns the compatibility story between the new tuple vocabulary and the existing backend-id, status-schema, and runtime-boundary contracts.
- This spec does not redefine backend-id grammar, tuple object shapes, policy keys, or platform guarantees.

Canonical references:
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/contract.md`
- `docs/project_management/packs/implemented/llm_and_agent_config_policy_surface/SCHEMA.md`
- `docs/contracts/gateway/status-schema.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Contract boundary

Owned here:

- the retirement posture for overloaded backend labels in new operator-facing docs and diagnostics
- the rollout rule that tuple vocabulary is the semantic layer while `backend_id` remains the adapter selector
- the compatibility proof that additive tuple publication does not redefine `status --json`, `client_wiring.*`, or backend inventory semantics
- the historical-evidence-only treatment for older wording that predates ADR-0042

Not owned here:

- backend-id grammar, backend inventory, or backend-selection realization
- tuple field names, token grammar, omission rules, or placement-posture object shape
- policy-evaluation ordering or tuple-axis constraint semantics
- Linux, macOS, or Windows parity guarantees
- operator command spellings, exit taxonomy, or status-schema field ownership

## Compatibility posture

- Existing operator commands remain unchanged.
- Existing `status --json` readers remain compatible because tuple publication is additive and stays outside `client_wiring.*`.
- Existing backend inventory and allowlist semantics remain unchanged because `backend_id` stays the `<kind>:<name>` adapter selector.
- Existing traces and diagnostics remain compatible because tuple metadata augments canonical correlation vocabulary rather than replacing it.
- Existing docs that already use `backend_id` as adapter selection remain correct when they keep that narrow meaning.

## Terminology retirement rules

- New operator-facing docs, planning specs, playbooks, examples, and diagnostics must not use `backend_id` as a proxy for `client`, `router`, `provider`, `auth_authority`, or `protocol`.
- New human-readable status or diagnostic wording must use the contract-owned labels:
  - `originating client`
  - `routing authority`
  - `fulfillment provider`
  - `auth authority`
  - `protocol`
  - `deployment posture`
  - `bridge transport`
- New machine-readable examples must use `identity_tuple` and `placement_posture` when they need tuple or placement semantics.
- New docs may mention `backend_id` only when they mean adapter selection, backend inventory identity, or allowlist selection.
- Archived or superseded docs may retain pre-ADR-0042 wording only when the document is clearly marked historical evidence or superseded context.

## End-state rule

The compatibility rollout is complete only when all of these conditions hold:

- new docs and diagnostics use tuple vocabulary for semantic identity
- `backend_id` remains the only adapter-selection token and is never presented as a substitute for tuple semantics
- additive tuple publication remains outside `client_wiring.*`
- additive tuple publication keeps ADR-0028 correlation keys unchanged
- `direct_provider_path` remains routing authority only and is not collapsed into backend identity
- example credential paths remain illustrative only and are not promoted into backend-id or auth-authority aliases

## Boundary alignment with adjacent ADRs

- ADR-0040 remains the owner of the Substrate versus `substrate-gateway` runtime-ownership split.
- ADR-0041 remains the owner of the stable backend-id contract and the one-backend-id-to-one-adapter posture.
- ADR-0046 remains the owner of backend-selection realization and integrated runtime behavior.
- This spec consumes those ADRs as compatibility anchors and keeps the tuple-vocabulary rollout from reopening their ownership lines.

## Historical evidence handling

- ADR-0023 and ADR-0024 remain historical evidence only where they are explicitly marked superseded.
- Backup packs, archived planning artifacts, and stale references may be cited only as historical context.
- Any active draft or active implementation doc that reintroduces overloaded backend wording is a compatibility defect.
- Any active doc that treats `backend_id` as semantic identity rather than adapter selection is a compatibility defect.

## Verification anchors

- `contract.md`
- `identity-tuple-schema-spec.md`
- `policy-spec.md`
- `telemetry-spec.md`
- `platform-parity-spec.md`
- `manual_testing_playbook.md`
- `docs/project_management/adrs/draft/ADR-0040-substrate-gateway-boundary-and-runtime-ownership.md`
- `docs/project_management/adrs/draft/ADR-0041-substrate-gateway-backend-adapter-contract.md`
- `docs/project_management/adrs/draft/ADR-0046-gateway-backend-selection-runtime-integration.md`

## Acceptance criteria

- The tuple vocabulary and `backend_id` vocabulary have one non-overlapping meaning each.
- New docs and diagnostics do not overload `backend_id`.
- Additive tuple publication remains backward compatible with status-schema and trace-owner boundaries.
- Historical wording is explicitly marked historical when it remains visible.
- The compatibility posture consumes ADR-0040, ADR-0041, and ADR-0046 without becoming a shadow contract for any of them.
