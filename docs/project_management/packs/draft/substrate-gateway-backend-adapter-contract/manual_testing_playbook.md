# Manual testing playbook - substrate-gateway-backend-adapter-contract

This playbook is the authoritative manual validation checklist for this feature pack.
It consumes the landed contracts below and does not redefine them.

## Contracts consumed

- `docs/contracts/substrate-gateway-operator-contract.md` - operator command family and exit taxonomy
- `docs/contracts/substrate-gateway-status-schema.md` - machine-readable gateway status wiring surface
- `docs/contracts/substrate-gateway-policy-evaluation.md` - policy evaluation and trust boundary
- `docs/project_management/adrs/draft/ADR-0017-agent-hub-concurrent-execution-and-output-routing.md` - event-envelope owner line
- `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md` - canonical trace vocabulary owner line
- `docs/contracts/substrate-gateway-runtime-parity.md` - typed runtime boundary and platform parity

## One-owner-per-surface audit

Use this checklist to confirm each surfaced validation area has one landed owner and no second owner is implied elsewhere.

### 1) Operator and status surface

Validate that operator entrypoints and their status wiring are owned by the published operator and status schema contracts, not by this playbook.

Check:
- The playbook treats `substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`, and `substrate world gateway status --json` as one operator-facing surface family.
- Human-readable status wording is treated as presentation, not as a new contract owner.
- JSON wiring checks stay limited to the machine-readable status schema owner and do not invent alternate discovery semantics.
- Exit behavior is evaluated as a consequence of the operator and status ownership lines, not as a separate local rule.

Pass condition:
- The operator/status family reads as one surfaced contract set with one owner per surface and no duplicate ownership language.

### 2) Policy surface

Validate that adapter selection policy, allowlist gating, and trusted-input boundaries remain owned by the policy-evaluation contract.

Check:
- Fail-closed selection behavior stays under the policy owner.
- Gateway-local config, admin mutation surfaces, token persistence, and session storage are treated as non-trusted inputs.
- Invalid selection, dependency unavailability, and policy denial remain distinct outcomes.
- The playbook does not elevate gateway-local state into a policy source of truth.

Pass condition:
- Policy evaluation is consumed as a single owner surface with no second owner and no added policy matrix.

### 3) Event surface

Validate that adapter-local event translation stops at the external event-envelope boundary and does not define a second envelope.

Check:
- Local adapter event translation is treated as bounded gateway-local behavior.
- The playbook does not restate or widen the external event envelope owned by ADR-0017.
- No new event categories, event fields, or routing semantics are introduced here.
- Any review language that sounds like event ownership is explicitly framed as validation only.

Pass condition:
- Event handling reads as one local-to-external boundary with no duplicate event ownership.

### 4) Trace surface

Validate that trace-adjacent validation stays under the canonical trace vocabulary owner and does not create a second trace model.

Check:
- Canonical trace vocabulary and correlation semantics remain external ownership under ADR-0028.
- The playbook only verifies that adapter behavior aligns with the trace owner line.
- No new trace field family, correlation rule, or proof category is introduced.
- Trace checks remain diagnostic validation, not a redefinition of the trace contract.

Pass condition:
- Trace validation maps to one owner and the playbook remains a consumer of that contract.

## Validation checklist

1. Read the six contract or owner-line files listed above and confirm the playbook matches their current wording at the boundary level.
2. Confirm each surfaced validation area below maps to exactly one contract owner:
   - operator and status
   - policy evaluation
   - event translation boundary
   - canonical trace vocabulary
3. Confirm this feature pack’s playbook does not claim ownership of command semantics, schema semantics, policy semantics, event-envelope semantics, or trace semantics outside those contract files.
4. Confirm the playbook does not introduce new proof categories, alternate validation owners, or hidden control-plane language.
5. Confirm the playbook reads as a validation checklist rather than as a specification.

## Stale-reference checks

Check for stale or archived wording before calling the playbook complete.

- Search the feature pack and adjacent docs for ownership wording that reintroduces ambiguity.
- Search for wording that implies a second owner for operator, status, policy, event, or trace surfaces.
- Search for wording that treats `status --json` as anything other than the machine-readable authority for gateway wiring discovery.
- Search for wording that collapses fail-closed behavior, dependency unavailability, and policy denial into one bucket.
- Search for wording that turns trace-adjacent validation into a second trace contract.

Accept only if:
- stale ownership wording is removed, or
- any unavoidable reference is explicitly called out as historical evidence and not as current ownership.

## Manual pass/fail summary

Pass when:
- each surfaced validation area has one landed owner,
- the playbook consumes the contract files without redefining them,
- and no stale ownership wording remains active in this feature pack.

Fail when:
- any surface claims two owners,
- any contract boundary is widened in the playbook,
- or stale archived references remain unqualified.
