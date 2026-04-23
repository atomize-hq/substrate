# LAITDP0-spec — identity contract and schema lock

## Behavior delta (single)
- Existing: ADR-0042, `contract.md`, and `identity-tuple-schema-spec.md` define the tuple vocabulary, but the pack does not yet have a first execution-ready slice that locks the operator wording, machine-readable object names, token grammar, omission semantics, and router or posture invariants as one implementation unit.
- New: `LAITDP0` becomes the authoritative first slice for the identity contract and schema layer, fixing the tuple field meanings, `identity_tuple` and `placement_posture` object names, required-versus-omittable field rules, and the rule that `direct_provider_path` is routing authority only and requires `host_only`.
- Why: later slices cannot safely define routing evaluation, additive status publication, or platform parity until the tuple vocabulary and schema rules are frozen.

## Scope
- Lock the operator-visible meanings of `client`, `router`, `provider`, `auth_authority`, and `protocol`.
- Lock the operator-visible meanings of `in_world`, `host_only`, and `host_to_world_bridge`.
- Lock the canonical machine-readable object names `identity_tuple` and `placement_posture`.
- Lock the tuple token grammar, required field set, independent omission rules for `provider` and `auth_authority`, and the ban on placeholder omission values.
- Lock the illustrative-only rule for example credential-source paths such as `~/.codex/auth.json`.
- Keep `backend_id`, `status --json`, `client_wiring.*`, ADR-0043 tuple-axis keys, and ADR-0028 correlation keys owned by their existing authorities.

## Inputs (authoritative)
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/pre-planning/minimal_spec_draft.md`

## Behavior (authoritative)

### Contract and schema boundary
- `contract.md` is the operator wording owner for tuple and posture meanings.
- `identity-tuple-schema-spec.md` is the machine-readable owner for object names, field names, requiredness, token grammar, and omission rules.
- `identity_tuple` publishes required fields `client`, `router`, and `protocol`.
- `provider` and `auth_authority` publish only when resolved or applicable and omit by field absence only.
- `placement_posture.execution` publishes `in_world` or `host_only`; `host_to_world_bridge` publishes only as the literal boolean `true` when bridge transport participates.

### Router and posture invariants
- `router=direct_provider_path` records direct provider fulfillment with no `substrate_gateway` mediation.
- `router=direct_provider_path` requires `placement_posture.execution="host_only"`.
- `router=direct_provider_path` is invalid when `placement_posture.host_to_world_bridge=true`.
- `host_to_world_bridge` remains transport-only and never becomes router identity or a second control plane.
- `backend_id` remains the `<kind>:<name>` adapter selector and never substitutes for tuple semantics.

### Human-readable and example-path boundary
- Human-readable status and diagnostics reuse the labels `originating client`, `routing authority`, `fulfillment provider`, `auth authority`, `protocol`, `deployment posture`, and `bridge transport`.
- Missing optional tuple fields stay absent in human-readable output; writers do not render `unknown`, `n/a`, or other filler text.
- Example credential paths such as `~/.codex/auth.json` remain illustrative examples only and never become required Substrate-owned path contracts.

## Acceptance criteria
- AC-LAITDP0-01: Active planning and implementation surfaces for this slice use the exact tuple field names `client`, `router`, `provider`, `auth_authority`, and `protocol`, plus the exact posture vocabulary `in_world`, `host_only`, and `host_to_world_bridge`.
- AC-LAITDP0-02: Machine-readable tuple publication uses only the object names `identity_tuple` and `placement_posture`, requires `client`, `router`, and `protocol`, and omits unresolved `provider` and `auth_authority` by field absence only.
- AC-LAITDP0-03: Tuple ids use normalized lowercase snake_case for `client`, `router`, `provider`, and `auth_authority`, while `protocol` uses normalized lowercase dotted ids and never reuses backend-id grammar.
- AC-LAITDP0-04: `direct_provider_path` is defined as routing authority only, requires `host_only` when published with posture, and is never valid with `host_to_world_bridge=true`.
- AC-LAITDP0-05: Human-readable wording for this slice uses the contract-owned labels and never overloads `backend_id` into tuple meaning or renames `host_only` to `host gateway`.
- AC-LAITDP0-06: Example credential-source paths remain illustrative only and are not promoted into required filesystem, config, or policy contract paths.

## Out of scope
- Routing-hint evaluation order and tuple-axis constraint enforcement.
- Additive `status --json`, diagnostics, or trace publication rules.
- Linux, macOS, and Windows parity proof or compatibility rollout validation.
