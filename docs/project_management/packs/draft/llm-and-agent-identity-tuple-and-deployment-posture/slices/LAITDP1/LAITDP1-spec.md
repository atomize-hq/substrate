# LAITDP1-spec — policy and observability alignment lock

## Behavior delta (single)
- Existing: the tuple vocabulary is pinned, but the pack does not yet have a mid-chain execution slice that binds routing-hint evaluation, unresolved-field denial rules, additive status publication, trace enrichment, and redaction boundaries into one deterministic implementation unit.
- New: `LAITDP1` becomes the authoritative policy-and-observability slice, fixing routing-hint handling, direct-provider gating, top-level status and trace placement for `identity_tuple` and `placement_posture`, stable human-readable diagnostics order, and non-secret publication rules.
- Why: operator-visible tuple publication is not safe unless policy decisions and observability placement follow one deterministic contract.

## Scope
- Lock routing-hint evaluation order and denial semantics using the existing ADR-0027 and ADR-0043 inputs.
- Lock the direct-provider gate, including the requirement that `direct_provider_path` requires `host_only` and forbids bridge transport.
- Lock additive publication of `identity_tuple` and `placement_posture` on status, diagnostics, and trace surfaces.
- Lock the redaction line that permits normalized ids only and forbids secrets, raw credential material, and raw credential paths.
- Keep the `status --json` envelope, `client_wiring.*`, ADR-0043 key ownership, and ADR-0028 correlation keys owned by their existing authorities.
- Leave platform parity, compatibility rollout, and manual validation evidence to the next slice.

## Inputs (authoritative)
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/policy-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/telemetry-spec.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/contract.md`
- `docs/project_management/packs/draft/llm-and-agent-identity-tuple-and-deployment-posture/identity-tuple-schema-spec.md`
- `docs/contracts/substrate-gateway-policy-evaluation.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Behavior (authoritative)

### Policy-evaluation boundary
- Routing hints are requests only. They never provide `client`, `router`, `auth_authority`, or placement authority by themselves.
- Backend allowlist and placement-floor evaluation run before tuple-aware routing-hint validation.
- Accepted routing hints may narrow the effective provider selection only after backend allowlists, tuple-axis constraints, and host-credential gates succeed.
- Rejected routing hints never rewrite `client`, `router`, or `auth_authority`, and they never create implicit provider authority.
- If `llm.constraints.providers` is non-empty and the effective provider is unresolved, the request is denied.
- If `llm.constraints.auth_authorities` is non-empty and the effective auth authority is unresolved, the request is denied.

### Status and diagnostics publication
- `status --json` publishes `identity_tuple` and `placement_posture` as top-level siblings of `status` and `client_wiring`.
- `identity_tuple` and `placement_posture` never publish inside `client_wiring.*`.
- Human-readable status and diagnostics render tuple and posture metadata in this exact order:
  1. `originating client`
  2. `routing authority`
  3. `fulfillment provider`
  4. `auth authority`
  5. `protocol`
  6. `deployment posture`
  7. `bridge transport`
- Missing optional tuple fields produce no placeholder lines.

### Trace and redaction boundary
- Trace records publish `identity_tuple` and `placement_posture` as additive top-level objects on the same records that already carry ADR-0028 correlation keys.
- Tuple and posture publication never replaces `session_id`, `span_id`, `cmd_id`, `world_id`, `backend_id`, or any other existing join key.
- Published tuple values are normalized ids only.
- Status, diagnostics, and trace publication never emits access tokens, API keys, session cookies, raw credential files, raw credential paths, or duplicated endpoint URLs from secret-bearing sources.

## Acceptance criteria
- AC-LAITDP1-01: Routing-hint evaluation runs after backend allowlist resolution and before final route selection, and an accepted hint narrows provider selection without supplying `client`, `router`, or `auth_authority`.
- AC-LAITDP1-02: A rejected routing hint never rewrites `client`, `router`, or `auth_authority`, never creates implicit provider authority, and never bypasses backend allowlists, tuple-axis constraints, or host-credential gates.
- AC-LAITDP1-03: `router=direct_provider_path` is valid only with `placement_posture.execution="host_only"`, is invalid with `placement_posture.host_to_world_bridge=true`, and does not become valid from `provider`, `protocol`, or `auth_authority` alone.
- AC-LAITDP1-04: `status --json` publishes `identity_tuple` and `placement_posture` as top-level additive siblings outside `client_wiring.*`, and unavailable-status responses keep the existing `client_wiring.*` owner line unchanged.
- AC-LAITDP1-05: Human-readable status and diagnostics use the exact tuple and posture label order defined by this slice, omit missing optional fields without placeholders, and never rename `router` to `backend`.
- AC-LAITDP1-06: Trace records carry the same object names as additive top-level metadata, preserve ADR-0028 correlation keys unchanged, and never emit secret material, illustrative credential paths, or duplicated endpoint-discovery URLs.

## Out of scope
- Tuple field meanings, token grammar, or omission-shape ownership from `LAITDP0`.
- Linux, macOS, and Windows parity guarantees.
- Compatibility rollout proof and manual cross-document review procedures.
