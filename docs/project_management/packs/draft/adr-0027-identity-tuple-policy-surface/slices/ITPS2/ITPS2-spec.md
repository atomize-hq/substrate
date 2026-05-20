# ITPS2-spec — telemetry publication and compatibility closure

## Behavior delta (single)
- Existing: the pack locks tuple-policy contract and runtime deny ordering, but it does not yet have one authoritative execution slice that binds tuple-aware status and trace publication to the additive rollout boundary that operators and downstream consumers inherit.
- New: `ITPS2` becomes the authoritative telemetry-and-compatibility slice, fixing reuse of `identity_tuple` and `placement_posture`, deny-versus-allow publication rules, `backend_id` separation, additive rollout behavior when `llm.constraints.*` is absent, and the future-extension invariants that later docs and code changes must preserve.
- Why: validation and promotion work cannot close cleanly unless telemetry placement and compatibility posture are frozen as one implementation seam.

## Scope
- Lock the tuple-aware telemetry publication seam for status, diagnostics, and trace output.
- Lock the additive compatibility posture for `llm.constraints.routers`, `llm.constraints.providers`, `llm.constraints.protocols`, and `llm.constraints.auth_authorities`.
- Lock the rule that `backend_id` remains adapter identity and correlation only.
- Lock the future-extension invariants that keep tuple-axis additions additive and compatible with ADR-0042 and ADR-0028 owner lines.
- Leave runtime gate ordering and deny-taxonomy ownership with `ITPS1`.
- Leave manual validation commands, CI checkpoint wiring, and promotion packaging to `ITPS3`.

## Inputs (authoritative)
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/telemetry-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/compatibility-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/policy-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/decision_register.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/workstream_triage.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`
- `docs/contracts/substrate-gateway-status-schema.md`
- `docs/TRACE.md`

## Behavior (authoritative)

### Telemetry publication seam
- `ITPS2` owns the slice-local closure that reuses `identity_tuple` and `placement_posture` across gateway status, diagnostics, and trace publication.
- `identity_tuple` and `placement_posture` stay top-level additive objects and never move under `client_wiring.*`.
- Allow-path publication emits tuple-aware metadata without deny-only detail fields.
- Deny-path publication emits the denying policy key and exact deny-detail text while preserving the same tuple object names.
- `backend_id` remains a separate selector and correlation field on status and trace surfaces and never becomes a tuple surrogate.
- Human-readable diagnostics keep the contract-owned tuple label order and omission rules rather than inventing telemetry-local wording.

### Compatibility seam
- `ITPS2` locks the additive rollout rule that tuple-axis keys extend the existing ADR-0027 policy surface instead of creating a second policy root, config root, or trace-local tuple family.
- When all four `llm.constraints.*` keys are absent or `[]`, backend selection, policy inspection, gateway lifecycle behavior, and backend correlation semantics remain unchanged apart from additive visibility of tuple-aware metadata.
- Tuple-axis constraints narrow an already selected backend path and do not widen a backend denied by `llm.allowed_backends`.
- `backend_id` remains the adapter-selection token, backend inventory identity, and existing consumer correlation field.
- Promotion into the implemented ADR-0027 pack must extend the existing contract and schema surfaces rather than fork them into a second policy system.

### Future-extension boundary
- Any later tuple-axis key must remain under `llm.constraints.*`.
- Any later tuple-axis key must preserve absent-or-empty compatibility behavior.
- Any later tuple-axis key must reuse `identity_tuple` and `placement_posture` and keep secret-bearing auth-source detail redacted or omitted.
- Any later tuple-axis key must keep `substrate policy current show --explain` as the authoritative merged inspection surface.
- This slice does not authorize tuple-field renaming, status-envelope widening under `client_wiring.*`, or a second trace-only tuple schema.

## Acceptance criteria
- AC-ITPS2-01: `ITPS2` defines one telemetry seam that reuses `identity_tuple` and `placement_posture` for gateway status, diagnostics, and trace publication instead of inventing a second tuple family.
- AC-ITPS2-02: `ITPS2` states that tuple-aware publication remains additive and top-level, keeps `identity_tuple` and `placement_posture` outside `client_wiring.*`, and preserves ADR-0028 correlation keys unchanged.
- AC-ITPS2-03: `ITPS2` states that allow publication omits deny-only fields while deny publication requires the denying policy key and exact deny-detail text.
- AC-ITPS2-04: `ITPS2` states that `backend_id` remains adapter identity and correlation only and never substitutes for `router`, `provider`, `protocol`, or `auth_authority`.
- AC-ITPS2-05: `ITPS2` states that absent or empty `llm.constraints.*` keys preserve existing backend-selection and operator workflows apart from additive tuple-aware visibility.
- AC-ITPS2-06: `ITPS2` states that promotion into the implemented ADR-0027 pack extends the existing policy system and does not create a second policy root, config root, or trace-only schema.
- AC-ITPS2-07: `ITPS2` states that future tuple-axis additions must stay under `llm.constraints.*`, preserve additive compatibility behavior, and reuse `identity_tuple` plus `placement_posture`.

## Out of scope
- Tuple-axis evaluation order, fail-early deny sequencing, and explain-surface ownership from `ITPS1`.
- Manual validation commands, cross-platform evidence capture, CI checkpoint task wiring, and promotion-closeout procedure from `ITPS3`.
- Tuple semantics, tuple-field grammar, and schema ownership from ADR-0042 and `ITPS0`.
