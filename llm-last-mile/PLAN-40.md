# PLAN-40: Internal Retained Host Approval Response Bootstrap

Source spec: [SPEC-40-internal-retained-host-approval-response-bootstrap.md](./SPEC-40-internal-retained-host-approval-response-bootstrap.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-39 host-response widening slice  
Status: implemented on `2026-06-03`
Landed posture note: typed host `approval_response` delivery over `continue_world_worker`, deny-by-default approval-response policy gating, exact approval-obligation binding, and deterministic post-delivery closeout are landed repo-wide, but broader host control/fork directives, active-ephemeral exact task-identity widening, public approval UX, and Family-2 router/attach execution remain deferred.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope stabilization follow-up, and the landed slice stayed bounded to typed host approval-response bootstrap over the existing prompt seam.

## Objective

Land the first post-Slice-39 Family-1 host-response widening slice by allowing host orchestrators to send typed `approval_response` messages through the existing `continue_world_worker` seam, bound to exact unresolved `ApprovalRequired` obligations and exact retained-worker identity, without widening into generic control directives, fork commands, active-ephemeral task identity, Family-2 router execution, or a new public approval surface.

This slice is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed `approval_response` payload under exact world/session/participant truth,
2. a deny-by-default policy surface explicitly gates typed approval responses,
3. typed approval responses must bind to exact unresolved approval obligations and fail closed otherwise,
4. successful delivery resolves or dismisses the matching approval obligation deterministically,
5. the existing member-turn prompt seam is reused honestly rather than implying a broader typed transport has landed,
6. `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, `fork_command`, active-ephemeral inspect/cancel widening, and Family-2 router execution remain deferred.

## Plan Summary

After Slice `39`, the repo can now accept worker-originated `approval_request` obligations, but the host still lacks a typed sanctioned response path. The live `continue_world_worker` payload remains prompt text plus optional thread id, and the design stack still treats `approval_response` as a distinct host-to-worker class rather than something that should stay indefinitely buried in ad hoc prompt wording.

That makes typed approval response the narrowest honest next widening because:

1. it closes the most immediate producer-consumer gap introduced by Slice `39`,
2. it widens an already-landed verb rather than inventing an eighth verb,
3. it can stay smaller than a transport redesign by compiling the typed response onto the already-landed member-turn prompt submit seam,
4. it freezes obligation-consumer semantics before broader control directives or Family-2 router execution.

What the repo already has:

1. the seven-verb Family-1 internal dispatch surface,
2. typed worker-originated `approval_request` obligations,
3. canonical local `ApprovalRequired` ledger entries with review-state and resolved-at semantics,
4. exact retained-worker targeting and world-binding validation through `continue_world_worker`,
5. a live member-turn submit seam that already delivers exact retained follow-up prompts.

What the repo lacked before Packets `1` through `3` landed was:

1. a typed host-side approval-response contract at the dispatch boundary,
2. explicit deny-by-default policy truth for that typed host response,
3. sanctioned approval-obligation consumer behavior on the live host control-plane path,
4. regression coverage proving delivery ordering and obligation closeout semantics.

The narrowest honest implementation order was:

1. freeze the typed approval-response contract and policy surface first,
2. freeze approval-obligation consumer semantics second,
3. wire live delivery and deterministic prompt rendering third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. add a typed internal `approval_response` host payload to the `continue_world_worker` contract,
2. add the minimum deny-by-default policy gate needed for typed host approval responses,
3. require exact unresolved approval-obligation binding before delivery,
4. resolve or dismiss the matching approval obligation only after successful delivery,
5. keep prompt rendering deterministic and implementation-owned.

### What does not change

1. no new dispatch verb,
2. no public CLI or toolbox approval-response surface,
3. no `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, or `fork_command` in this slice,
4. no transport-api schema widening in this slice,
5. no active-ephemeral inspect/cancel widening,
6. no router/daemon execution redesign,
7. no host-global inbox or remote ingress layering,
8. no fork allocation or auto-fork behavior.

## Implementation Order

### Packet 1: Typed Approval Response Contract And Policy Surface

Goal:

1. freeze the first typed host-side approval-response payload shape,
2. add minimal deny-by-default policy parsing and denial buckets,
3. keep broader host-response classes deferred.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. targeted contract and policy tests

Why first:

1. delivery and obligation-consumer behavior should sit on a frozen typed contract,
2. the runtime needs explicit deny-by-default policy truth before typed approval responses can be accepted safely,
3. later packets should not infer host-response semantics from free-form prompts.

Verification checkpoint:

1. typed `approval_response` payloads validate,
2. the new policy key is deny-by-default,
3. explanation-ready denials exist for disallowed typed approval responses,
4. broader host-response/control classes remain deferred.

### Packet 2: Approval Obligation Consumer Semantics

Goal:

1. freeze how typed approval responses consume existing `ApprovalRequired` obligations,
2. keep review-state and resolved-at semantics explicit and durable,
3. fail closed on ambiguous or stale obligation targeting.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
3. targeted state-store and obligation tests

Why second:

1. live delivery should consume one authoritative approval-obligation rule,
2. this avoids ad hoc local file mutation or non-reviewable side effects,
3. approval closeout semantics need to be explicit before prompt rendering and delivery are considered complete.

Verification checkpoint:

1. approve-style responses resolve the matching approval obligation,
2. deny-style responses dismiss the matching approval obligation,
3. missing, resolved, cross-session, wrong-kind, or boundary-drifted obligations fail closed,
4. unresolved obligations remain unchanged when delivery has not occurred.

### Packet 3: Continue Routing, Deterministic Rendering, And Delivery Ordering

Goal:

1. wire the live `continue_world_worker` path to deliver typed approval responses,
2. compile the typed response onto the existing member-turn prompt seam deterministically,
3. prove that approval-obligation closeout happens only after successful delivery.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. targeted shell integration/regression tests

Why third:

1. live routing should consume the frozen contract and obligation-consumer semantics,
2. this packet proves the slice is real runtime behavior rather than doc-only schema work,
3. the key safety property is ordering: no delivery means no closeout.

Verification checkpoint:

1. typed approval responses submit through the existing retained member-turn seam,
2. rendered prompt content is deterministic from the typed payload,
3. delivery failure leaves the approval obligation unresolved,
4. successful delivery resolves or dismisses the matching obligation only once.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `40` scope,
2. keep broader host-response/control widening, active-ephemeral identity, and Family-2 work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-40-internal-retained-host-approval-response-bootstrap.md`
3. `llm-last-mile/PLAN-40.md`
4. `llm-last-mile/TASKS-40.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `40` as typed host approval-response bootstrap, not generic control-directive delivery or transport redesign,
2. docs keep `fork_command`, `control_directive`, and active-ephemeral identity widening explicitly deferred,
3. docs keep Family-2 router execution downstream.

Verification checkpoint:

1. docs remain honest about scope,
2. validation is green,
3. the next follow-on slice can sequence broader host-response/control widening or active-ephemeral identity work without reopening Slice `40`.

## Risks And Mitigations

1. Risk: the slice quietly widens into a broader host-message transport redesign.
   Mitigation: keep the typed contract at the dispatch boundary and reuse the existing prompt submit seam in v1.
2. Risk: approval obligations resolve before the worker actually receives the response.
   Mitigation: sequence closeout only after successful submit/delivery confirmation from the live member-turn path.
3. Risk: typed approval response drags in generic control directives or fork commands.
   Mitigation: limit the accepted host-side typed class to `approval_response` only.
4. Risk: approval targeting becomes heuristic and closes the wrong obligation.
   Mitigation: require exact unresolved approval-obligation binding plus exact retained-worker identity.
5. Risk: the slice reopens Family-2 attach/router behavior because approval obligations are attach-eligible.
   Mitigation: reuse the currently landed local obligation ledger only; do not redesign router ownership or attach execution.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because approval-obligation consumer behavior depends on a frozen typed approval-response contract and policy surface.
2. Packet 2 before Packet 3 because live routing should consume one authoritative approval closeout rule rather than inventing it inline.
3. Packet 3 before Packet 4 because docs and validation should reflect final live behavior.

### Can be parallelized later

1. broader typed host-response/control work after Slice `40` is stable,
2. active-ephemeral exact task identity work after the retained approval-response slice is closed,
3. Family-2 router execution after the Family-1 producer and consumer semantics are frozen enough for downstream attach behavior.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `40`

If Slice `40` lands cleanly as narrow typed approval-response bootstrap first, the next follow-on work should remain:

1. broader typed host-to-worker classes such as `clarification_response`, `progress_ack`, `control_directive`, `control_ack`, and `fork_command` only if still needed,
2. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
3. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `40` implementation scope.
