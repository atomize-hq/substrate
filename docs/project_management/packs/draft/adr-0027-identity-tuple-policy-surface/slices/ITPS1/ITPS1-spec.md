# ITPS1-spec — runtime fail-early policy ordering lock

## Behavior delta (single)
- Existing: the pack has the tuple-axis key family and the first slice lock, but it does not yet have one authoritative runtime-planning slice that fixes evaluation order, fail-early deny behavior, explain-message families, and reused failure buckets for tuple-aware routing.
- New: `ITPS1` becomes the authoritative runtime-policy slice, fixing lifecycle config gates, backend allowlists, tuple-axis narrowing order, unresolved constrained-field denials, auth-source gating, and the exact deny-message families that downstream telemetry and validation work inherit.
- Why: telemetry, compatibility, and validation planning cannot stay coherent unless the runtime deny path and failure buckets are locked first.

## Scope
- Publish the runtime policy contract in `policy-spec.md`.
- Publish the A/B decisions in `decision_register.md`.
- Lock the fail-early deny taxonomy for backend, router, protocol, provider, and auth-authority mismatches.
- Lock the explain-surface rule that keeps `llm.constraints.*` on `substrate policy current show --explain`.
- Leave telemetry-field inventories, compatibility rollout text, manual validation steps, and task/checkpoint wiring to later slices.

## Inputs (authoritative)
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/contract.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/tuple-policy-schema-spec.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/impact_map.md`
- `docs/project_management/packs/draft/adr-0027-identity-tuple-policy-surface/pre-planning/workstream_triage.md`
- `docs/contracts/gateway/policy-evaluation.md`
- `docs/project_management/adrs/draft/ADR-0042-llm-and-agent-identity-tuple-and-deployment-posture.md`
- `docs/project_management/adrs/draft/ADR-0043-adr-0027-identity-tuple-policy-surface.md`

## Behavior (authoritative)

### Ordered runtime gates
- Gateway lifecycle config gates run before backend or tuple evaluation.
- `llm.allowed_backends` gates the selected backend before tuple-axis narrowing begins.
- Tuple-axis narrowing runs in this exact order:
  1. `llm.constraints.routers`
  2. `llm.constraints.protocols`
  3. `llm.constraints.providers`
  4. `llm.constraints.auth_authorities`
- Env-auth gating and host credential-read gating run after tuple derivation and before final runtime dispatch.
- Missing world components remain component-unavailable outcomes; transient socket or timeout failures remain transient-runtime outcomes.

### Fail-early deny rules
- Backend allowlist denial stops evaluation before tuple-axis narrowing.
- Non-empty provider or auth-authority constraints deny unresolved effective values.
- Blocked env auth denies the request instead of falling through to host credential-file reads.
- Partial env auth fails as invalid integration instead of merging with another auth source.
- `host_to_world_bridge` and `net_allowed` remain downstream transport and egress gates; neither surface can authorize a tuple-axis mismatch.

### Operator explain requirements
- `substrate policy current show --explain` is the authoritative merged inspection surface for `llm.constraints.*`.
- Deny explanations must name the exact denying policy key.
- Deny explanations must use tuple labels:
  - `routing authority`
  - `protocol`
  - `provider`
  - `auth authority`
- Deny explanations must not relabel `router` as `backend`.

### Decision closure
- `DR-ITPS-01` selects reuse of `identity_tuple` and `placement_posture`.
- `DR-ITPS-02` selects `substrate policy current show --explain` as the authoritative inspection surface for tuple-policy keys.

## Acceptance criteria
- AC-ITPS1-01: `policy-spec.md` defines one ordered runtime flow that starts with lifecycle config gates, applies backend allowlisting before tuple narrowing, and places auth-source gating before final runtime dispatch.
- AC-ITPS1-02: `policy-spec.md` defines the exact tuple-axis narrowing order as `routers`, `protocols`, `providers`, `auth_authorities`.
- AC-ITPS1-03: `policy-spec.md` states that non-empty provider and auth-authority constraints deny unresolved effective values instead of degrading to an unconstrained route.
- AC-ITPS1-04: `policy-spec.md` states that blocked env auth denies the request and that partial env auth is invalid integration rather than a host-credential fallback path.
- AC-ITPS1-05: `policy-spec.md` defines the exact deny-message families for backend, router, protocol, provider, and auth-authority failures and requires the denying policy key in each explanation.
- AC-ITPS1-06: `policy-spec.md` states that missing world components remain component-unavailable outcomes and that connection or timeout failures remain transient-runtime outcomes.
- AC-ITPS1-07: `decision_register.md` records `DR-ITPS-01` as reuse of `identity_tuple` and `placement_posture` instead of a trace-only tuple schema.
- AC-ITPS1-08: `decision_register.md` records `DR-ITPS-02` as `substrate policy current show --explain` ownership for `llm.constraints.*` instead of a config-view or dual-view contract.

## Out of scope
- Tuple-key grammar, YAML shape, and reserved-key ownership from `ITPS0`.
- Tuple-aware telemetry field inventories and redaction tables.
- Additive rollout guarantees for operators who omit `llm.constraints.*`.
- Manual validation commands, platform evidence, CI checkpoints, and `tasks.json` wiring.
