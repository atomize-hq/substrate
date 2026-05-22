# Manual testing playbook - substrate-gateway-boundary-and-runtime-ownership

This playbook is the authoritative manual validation checklist for the feature pack.
It is a consumer of the landed contracts below and does not redefine them.

## Contracts consumed

- `docs/contracts/substrate-gateway-operator-contract.md` - operator command family and exit taxonomy
- `docs/contracts/substrate-gateway-status-schema.md` - machine-readable gateway status wiring surface
- `docs/contracts/substrate-gateway-policy-evaluation.md` - policy evaluation and trust boundary
- `docs/contracts/substrate-gateway-runtime-parity.md` - typed runtime boundary and platform parity

## One-owner-per-surface audit

Use this checklist to confirm each operator-visible surface has one landed owner and no second owner is implied elsewhere.

### 1) Operator command family

Validate that the gateway command family is owned only by `docs/contracts/substrate-gateway-operator-contract.md`.

Check:
- The playbook treats `substrate world gateway sync`, `substrate world gateway status`, `substrate world gateway restart`, and `substrate world gateway status --json` as one operator surface.
- The human-readable status entrypoint is described only as a presentation surface and not as a separate contract owner.
- Exit behavior is checked only as a consequence of the operator command contract, not as a new local rule.
- Transitional lifecycle windows (startup, restart, ready-timeout, connection-refused handoff gaps) are checked as transient runtime failures with exit `3`, not as unavailable.
- Exit `4` is checked only for actual absent component outcomes, such as an explicit runtime `unavailable` result or a missing required gateway/world listener.
- No other doc in this feature pack claims ownership of the operator command family.

Pass condition:
- The command family reads as a single surfaced contract with one owner and no duplicate ownership language.

### 2) Machine-readable status wiring

Validate that the JSON wiring surface is owned only by `docs/contracts/substrate-gateway-status-schema.md`.

Check:
- `substrate world gateway status --json` is treated as the authoritative machine-readable wiring discovery surface.
- The playbook expects only the owned wiring fields described by the status schema contract.
- The playbook does not invent extra top-level fields, alternate discovery semantics, or placeholder values.
- The non-secret posture is checked as a contract requirement, not as an implementation guess.

Pass condition:
- The status wiring surface maps to one contract owner and the playbook does not restate schema details as if they were locally owned.

### 3) Policy evaluation and trust boundary

Validate that policy and trust boundary rules are owned only by `docs/contracts/substrate-gateway-policy-evaluation.md`.

Check:
- The playbook keeps fail-closed behavior, host-to-world secret delivery, and trust-boundary rules under the policy contract.
- Gateway-local config, admin mutation surfaces, and token persistence are treated as non-trusted surfaces.
- Invalid integration state, dependency unavailability, and policy denial remain distinct reasoning outcomes.
- The playbook does not elevate gateway-local state into a policy source of truth.

Pass condition:
- Policy evaluation is consumed as a single contract owner with no second owner and no added policy matrix.

### 4) Runtime and platform parity

Validate that typed lifecycle/status runtime behavior and platform parity are owned only by `docs/contracts/substrate-gateway-runtime-parity.md`.

Check:
- The playbook treats shell and world-service runtime interaction as a typed boundary, not raw probing or log scraping.
- Linux, macOS, and Windows are checked as one operator-facing lifecycle/status contract with permitted hidden transport divergence only.
- Provisioning and warm-flow mechanics are treated as evidence surfaces, not as the operator contract itself.
- Start/restart windows are validated as transient runtime failures in the operator surface even if the hidden transport symptom is a temporary connect failure.
- The playbook does not turn platform-specific transport detail into a second runtime contract.

Pass condition:
- Runtime/parity checks map to one owner and the playbook remains a consumer of that contract.

## Validation checklist

1. Read the four contract files listed above and confirm the playbook matches their current wording.
2. Confirm each operator-visible surface below maps to exactly one contract owner:
   - command family and exit taxonomy
   - machine-readable status wiring
   - policy evaluation and trust boundary
   - typed runtime and platform parity
3. Confirm this feature pack’s playbook does not claim ownership of schema, policy, runtime, or command semantics outside those contract files.
4. Confirm the playbook does not introduce any new command spellings, status fields, policy rules, or transport rules.
5. Confirm the playbook can be read as a validation checklist rather than as a specification.

## Stale-reference checks

Check for stale or archived wording before calling the playbook complete.

- Search the feature pack and adjacent docs for archived references that reintroduce ownership ambiguity.
- Search for wording that implies a second owner for command, status, policy, or runtime surfaces.
- Search for wording that treats `status --json` as anything other than the machine-readable authority for gateway wiring discovery.
- Search for wording that collapses fail-closed behavior, dependency unavailability, and policy denial into one bucket.
- Search for wording that labels startup or restart windows as exit `4` unavailable instead of exit `3` transient runtime failure.
- Search for wording that turns platform transport detail into a second operator-facing contract.

Accept only if:
- archived or stale ownership wording is removed, or
- any unavoidable reference is explicitly called out as historical evidence and not as current ownership.

## Manual pass/fail summary

Pass when:
- each operator-visible surface has one landed owner,
- the playbook consumes the four contract files without redefining them,
- and no stale archived ownership wording remains active in this feature pack.

Fail when:
- any surface claims two owners,
- any contract boundary is widened in the playbook,
- or stale archived references remain unqualified.
