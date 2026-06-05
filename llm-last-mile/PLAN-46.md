# PLAN-46: Internal Retained Host Progress Ack Bootstrap

Source spec: [SPEC-46-internal-retained-host-progress-ack-bootstrap.md](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-45 optional host acknowledgement slice  
Status: drafted on `2026-06-05`

## Objective

Land the narrow optional host acknowledgement slice by allowing host orchestrators to send typed `progress_ack` messages through the existing `continue_world_worker` seam, bound to exact retained-worker identity and deny-by-default policy truth, with deterministic implementation-owned prompt rendering and truthful delivery-only outcomes, without widening into durable progress obligations, control directives, fork commands, active-ephemeral task identity, Family-2 router execution, or a new public control surface.

This slice is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed host `progress_ack` payload under exact session/world/participant truth,
2. a deny-by-default policy surface explicitly gates typed host progress acknowledgements,
3. typed host progress acknowledgements render deterministically onto the existing retained member-turn seam,
4. successful typed host progress acknowledgements keep delivery semantics explicit and non-durable,
5. `progress_update` remains non-attention-driving and existing approval/control/fork flows remain bounded,
6. active-ephemeral inspect/cancel widening and Family-2 router execution remain deferred.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-46-internal-retained-host-progress-ack-bootstrap.md](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md) has been reviewed and is the source of truth for scope before implementation planning advances.

## Major Components And Dependencies

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - owns the typed `continue_world_worker` payload contract and deterministic prompt rendering
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - owns payload-policy enforcement, live delivery routing, and truthful outcome summaries
3. `crates/shell/src/execution/policy_model.rs`
   - owns shell-side deny-by-default parsing and diagnostics for any new host progress-ack gate
4. `crates/broker/src/policy.rs` and `crates/broker/src/effective_policy.rs`
   - must stay aligned with shell-local policy truth so YAML policy evaluation and diagnostics match runtime enforcement
5. `docs/CONFIGURATION.md`
   - must document the new gate and keep delivery-only scope explicit once runtime truth lands

## Plan Summary

After Slice `45`, the repo already has the retained control and fork loop on the existing continue seam:

1. typed host `approval_response` is landed,
2. typed host `clarification_response` is landed,
3. typed host `control_directive` is landed,
4. same-stream worker `control_ack` is landed,
5. worker `progress_update` is already a live non-attention-driving event class,
6. typed host `fork_command` is landed.

What remains is the last still-deferred host-to-worker message class in the retained messaging design that can land without reopening identity or router work: `progress_ack`.

That is narrower than active-ephemeral exact task identity because:

1. it stays on the already-landed retained continue seam,
2. it does not reopen `inspect_world_worker` or `cancel_world_work`,
3. it does not require a typed `task_run_id` surface.

That is narrower than Family-2 router or attach work because:

1. it does not add any new ingress surface,
2. it does not redesign review or inbox ownership,
3. it reuses the existing member-turn prompt seam and exact retained-worker routing.

The narrowest honest implementation order is:

1. freeze the typed host progress-ack contract and policy surface first,
2. freeze deterministic rendering and delivery-only summary semantics second,
3. prove non-durable and non-widening regression behavior third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. add a typed internal `progress_ack` host payload to the `continue_world_worker` contract,
2. add the minimum deny-by-default policy gate needed for typed host progress acknowledgements,
3. render typed host progress acknowledgements deterministically onto the existing retained member-turn seam,
4. keep successful delivery summaries explicit that acknowledgement is delivery-only and non-durable,
5. preserve existing `progress_update` and broader steering semantics unchanged.

### What does not change

1. no new dispatch verb,
2. no public CLI or toolbox progress-ack surface,
3. no durable progress obligation or inbox projection,
4. no typed control-directive, fork-command, or worker-event widening in this slice,
5. no transport-api schema widening in this slice,
6. no active-ephemeral inspect/cancel widening,
7. no router/daemon execution redesign.

## Implementation Order

### Packet 1: Typed Progress Ack Contract And Policy Surface

Goal:

1. freeze the typed host-side `progress_ack` payload shape,
2. add minimal deny-by-default policy parsing and denial buckets,
3. keep the slice delivery-only and non-durable.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. targeted contract and policy tests

Why first:

1. live delivery should sit on a frozen typed contract,
2. the runtime needs explicit deny-by-default policy truth before typed host progress acknowledgements can be accepted safely,
3. later packets should not infer progress-ack semantics from generic free-form continue.

Verification checkpoint:

1. typed host `progress_ack` payloads validate,
2. the new policy key is deny-by-default,
3. explanation-ready denials exist for disallowed typed host `progress_ack`,
4. the slice remains delivery-only and non-durable.

### Packet 2: Deterministic Rendering And Delivery-Only Outcome Semantics

Goal:

1. render typed host progress acknowledgements onto the existing live member-turn prompt seam deterministically,
2. keep exact retained-worker targeting mandatory,
3. keep summaries honest that the message only acknowledges seen progress.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. targeted contract and routing tests

Why second:

1. the slice needs one frozen typed rendering path before live delivery is treated as real,
2. progress acknowledgement semantics live mostly in the rendered prompt and outcome wording,
3. this keeps the slice smaller than any broader host-message envelope redesign.

Verification checkpoint:

1. accepted typed host `progress_ack` messages render deterministically,
2. exact retained-worker targeting remains mandatory,
3. successful summaries stay explicit that acknowledgement is delivery-only.

### Packet 3: Regression Coverage And No-Durable-Side-Effect Proof

Goal:

1. prove typed host `progress_ack` does not create durable obligation side effects,
2. prove it does not imply completion, pause, or fork handling,
3. keep existing `progress_update`, approval, control, and fork behavior green and bounded.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. targeted shell integration and regression tests

Why third:

1. once the contract, gate, and delivery wording are frozen, the main remaining risk is semantic widening,
2. this packet proves `progress_ack` is a real but narrow typed class rather than a hidden workflow redesign,
3. the key safety property is non-durable acknowledgement, not new lifecycle machinery.

Verification checkpoint:

1. allowed typed host `progress_ack` delivery leaves no durable obligation side effects,
2. `progress_update` remains non-attention-driving by default,
3. delivery or policy failure returns explanation-ready errors,
4. control, fork, and active-ephemeral behavior remains green and bounded.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `46` scope,
2. keep active-ephemeral identity and Family-2 work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md`
3. `llm-last-mile/PLAN-46.md`
4. `llm-last-mile/TASKS-46.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `46` as typed host `progress_ack` bootstrap over the existing `continue_world_worker` seam, not generalized control transport or progress-review workflow,
2. docs keep durable progress obligations, active-ephemeral identity widening, and Family-2 router execution explicitly deferred,
3. docs do not imply public progress UX or broader typed host-message transport.

Verification checkpoint:

1. docs and config truth remain honest about scope,
2. validation is green,
3. the next follow-on slice can sequence active-ephemeral identity work or later router work without reopening Slice `46`.

## Risks And Mitigations

1. Risk: typed host `progress_ack` quietly becomes a durable progress-review or inbox feature.
   Mitigation: keep the payload delivery-only, keep durable side effects out of scope, and keep summaries explicit that acknowledgement only confirms receipt.
2. Risk: the slice overclaims exact causation that the live normalized worker-event surface does not expose.
   Mitigation: bind to exact retained-worker identity and optional thread continuity only; defer richer progress-causation surfaces to a later transport or identity slice.
3. Risk: the slice is too weak to justify its own typed class and should stay free-form only.
   Mitigation: keep the payload minimal and explicit so the repo can decide with real usage whether the optional class is worth retaining.
4. Risk: the gate lands in the wrong policy namespace.
   Mitigation: keep `progress_ack` in the control namespace, parallel to `control_directive`, because it is a host operational acknowledgement rather than an obligation consumer or fork action.
5. Risk: the slice is misclassified as active-ephemeral or Family-2 work.
   Mitigation: keep it exact retained-worker, continue-seam only, and explicitly out of router/attach scope.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because rendering and outcome semantics depend on a frozen typed host progress-ack contract and policy surface.
2. Packet 2 before Packet 3 because regression proof should consume one deterministic renderer and one delivery-only summary contract.
3. Packet 3 before Packet 4 because docs and validation should reflect final landed behavior.

### Can be parallelized later

1. active-ephemeral exact task identity plus inspect/cancel widening as its own later identity-model slice,
2. any richer progress-review or causation model only if a later typed host-message transport design is needed,
3. Family-2 router/attach execution only after the Family-1 producer and consumer semantics are frozen enough for downstream attach behavior.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell dispatch_contract -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `46`

If Slice `46` lands cleanly as narrow typed host `progress_ack` bootstrap first, the next follow-on work should remain:

1. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
2. any richer progress-causation or typed host-message envelope work only if later repo truth proves the narrow acknowledgement slice insufficient,
3. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `46` implementation scope.
