# PLAN-44: Internal Retained Worker Control Ack Bootstrap

Source spec: [SPEC-44-internal-retained-worker-control-ack-bootstrap.md](./SPEC-44-internal-retained-worker-control-ack-bootstrap.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-43 worker-acknowledgement slice  
Status: drafted on `2026-06-04`

## Objective

Land the narrow worker-acknowledgement slice that admits typed retained-worker `control_ack` on the live `continue_world_worker` seam, but only as exact same-stream acknowledgement of an already-landed typed host `control_directive`, without widening into `fork_command`, optional `progress_ack`, a durable obligation kind, active-ephemeral task identity, Family-2 router execution, or a new public control surface.

This slice is complete only when all of the following are true:

1. the retained-worker event contract admits `control_ack` under exact session/participant/backend/world truth,
2. `control_ack` is accepted only for typed host `control_directive` turns and fails closed elsewhere,
3. `control_ack` remains non-attention-driving and non-durable,
4. live delivery outcomes stay honest about acknowledgement versus completion,
5. the existing `control_directives_allowed` host gate remains the only policy surface widened by implication,
6. `fork_command`, optional `progress_ack`, active-ephemeral inspect/cancel widening, and Family-2 router execution remain deferred.

## Plan Summary

After Slice `43`, the repo already has the host half of operational steering:

1. the host may issue typed `control_directive` over the existing `continue_world_worker` seam,
2. that path is deny-by-default under `control_directives_allowed`,
3. prompt rendering is deterministic and implementation-owned,
4. successful delivery remains truthful about delivery-only semantics and explicitly avoids implying a landed worker acknowledgement.

What remains is the smallest missing paired behaviour on that same seam: the retained worker should be allowed to emit `control_ack` back to the host on the same live stream when it is acknowledging the typed control directive it just received.

That is narrower than `fork_command` because:

1. it does not allocate a child worker,
2. it does not widen lineage or child-delivery semantics,
3. it does not need a new durable obligation kind,
4. it can stay on the existing immediate stream without transport redesign.

That is narrower than optional `progress_ack` because:

1. `progress_ack` remains optional in the design stack,
2. `progress_ack` still lacks the concrete same-stream pairing that the already-landed `control_directive` now provides,
3. the repo already documents `control_ack` as the explicit next follow-on after Slice `43`.

The narrowest honest implementation order is:

1. freeze the event-class contract and exact-causation guard first,
2. freeze live outcome semantics and no-durable-side-effect behaviour second,
3. prove fail-closed regression coverage third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. admit `control_ack` as an in-scope retained-worker event class,
2. accept `control_ack` only when the originating `continue_world_worker` request payload was the typed host `control_directive`,
3. surface `control_ack` through the existing live outcome path,
4. keep acknowledgement semantics explicit and non-terminal,
5. document the slice as immediate-only and non-durable.

### What does not change

1. no new dispatch verb,
2. no new dedicated `control_ack` policy key,
3. no `fork_command` or optional `progress_ack` in this slice,
4. no durable obligation kind or inbox projection for `control_ack`,
5. no public CLI or toolbox widening,
6. no active-ephemeral inspect/cancel widening,
7. no router/daemon execution redesign.

## Implementation Order

### Packet 1: Event Contract Admission And Exact-Causation Guard

Goal:

1. admit `control_ack` as an in-scope retained-worker event class,
2. remove it from deferred-worker rejection coverage,
3. require typed host `control_directive` causation before `control_ack` is accepted.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. targeted contract and classifier tests

Why first:

1. no live outcome or docs work should proceed until the runtime contract says exactly when `control_ack` is valid,
2. the key safety property is fail-closed causation, not merely adding one more event label,
3. later packets should build on one frozen acknowledgement boundary.

Verification checkpoint:

1. `control_ack` becomes an admitted event class,
2. `control_ack` is non-attention-driving by default,
3. `control_ack` fails closed for generic continue, typed `approval_response`, and typed `clarification_response`,
4. other deferred worker labels remain deferred.

### Packet 2: Live Outcome Semantics And No-Durable-Side-Effect Behaviour

Goal:

1. surface accepted `control_ack` through the live `continue_world_worker` outcome,
2. keep summaries honest about acknowledgement versus completion,
3. keep `control_ack` outside the durable obligation path.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. targeted shell integration and outcome tests

Why second:

1. once the contract is frozen, the next question is what the host actually sees on a successful acknowledged control-directive turn,
2. the slice must prove `control_ack` is immediate-only and does not silently mutate the durable ledger,
3. outcome wording is the main behavioural surface users and future slices will rely on.

Verification checkpoint:

1. a typed host `control_directive` may surface `control_ack` on the same live stream,
2. successful acknowledgement does not imply task completion,
3. no durable obligation is written for `control_ack`,
4. prior approval/clarification/control-directive behaviour remains green.

### Packet 3: Fail-Closed Regression Proof

Goal:

1. prove invalid `control_ack` contexts fail closed,
2. prove the runtime still cancels or rejects unsupported worker events correctly,
3. keep exact identity and world-binding checks stable.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. targeted shell regression tests adjacent to the touched implementation

Why third:

1. the slice is only safe if the new event class cannot leak into non-control-directive flows,
2. regression proof must cover both immediate-stream semantics and the existing deferred-worker safety posture,
3. this packet keeps the slice bounded before docs call it real.

Verification checkpoint:

1. invalid `control_ack` contexts fail with stable explanation-ready errors,
2. exact participant/backend/session/world drift still fails closed,
3. `fork_command`, host `control_directive`, and `attention_required` remain deferred worker labels,
4. the live stream still cancels correctly when unsupported labels appear.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `44` scope,
2. keep `fork_command`, optional `progress_ack`, active-ephemeral identity, and Family-2 work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-44-internal-retained-worker-control-ack-bootstrap.md`
3. `llm-last-mile/PLAN-44.md`
4. `llm-last-mile/TASKS-44.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `44` as immediate worker `control_ack` on the existing `continue_world_worker` seam, not durable control review, generalized control transport, or `fork_command`,
2. docs keep `fork_command`, optional `progress_ack`, and active-ephemeral identity widening explicitly deferred,
3. docs keep Family-2 router execution downstream and do not imply inbox- or obligation-level treatment for `control_ack`.

Verification checkpoint:

1. docs and config truth remain honest about scope,
2. validation is green,
3. the next follow-on slice can sequence `fork_command` without reopening Slice `44`.

## Risks And Mitigations

1. Risk: `control_ack` is admitted too broadly and leaks into generic or non-control-directive `continue_world_worker` flows.  
   Mitigation: tie acceptance to typed host `control_directive` causation only.
2. Risk: `control_ack` is treated like durable review work and silently widens the obligation ledger.  
   Mitigation: keep it non-attention-driving and outside the obligation persistence path.
3. Risk: acknowledgement wording implies task completion or broader execution success.  
   Mitigation: freeze outcome summaries around acknowledgement-only truth.
4. Risk: the slice quietly becomes `fork_command` or broader control transport work.  
   Mitigation: keep the only new admitted worker event class `control_ack` and leave request-payload widening out of scope.
5. Risk: a separate policy key is added prematurely and creates unnecessary matrix complexity.  
   Mitigation: reuse the already-landed `control_directives_allowed` gate unless repo evidence later proves a separate gate is necessary.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because live outcome behaviour depends on a frozen acknowledgement contract and causation rule.
2. Packet 2 before Packet 3 because regression proof should validate the final accepted semantics, not an intermediate contract.
3. Packet 3 before Packet 4 because docs and validation should reflect the final fail-closed posture.

### Can be parallelized later

1. typed host `fork_command` after `control_ack` semantics are frozen if still needed,
2. optional typed host `progress_ack` if still needed after the control acknowledgement slice lands,
3. active-ephemeral exact task identity work as its own later slice,
4. Family-2 router/attach execution after Family-1 control and acknowledgement semantics are sufficiently frozen.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `44`

If Slice `44` lands cleanly as narrow worker `control_ack` bootstrap first, the next follow-on work should remain:

1. typed host `fork_command` only, if still needed after the control-directive/ack pair is frozen,
2. optional typed host `progress_ack` only if still needed after the operational steering loop lands,
3. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
4. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `44` implementation scope.
