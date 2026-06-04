# PLAN-43: Internal Retained Host Control Directive Bootstrap

Source spec: [SPEC-43-internal-retained-host-control-directive-bootstrap.md](./SPEC-43-internal-retained-host-control-directive-bootstrap.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-42 host operational steering slice  
Status: draft for review on `2026-06-04`

## Objective

Land the narrow host operational steering slice by allowing host orchestrators to send typed `control_directive` messages through the existing `continue_world_worker` seam, bound to exact retained-worker identity and deny-by-default policy truth, without widening into `control_ack`, `fork_command`, `progress_ack`, active-ephemeral task identity, Family-2 router execution, or a new public control surface.

This slice is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed `control_directive` payload under exact world/session/participant truth,
2. a deny-by-default policy surface explicitly gates typed host control directives,
3. the initial directive-kind surface is narrow and explicit,
4. deterministic prompt rendering over the existing member-turn seam is implementation-owned,
5. delivery outcomes remain honest about the lack of typed `control_ack`,
6. `control_ack`, `fork_command`, `progress_ack`, active-ephemeral inspect/cancel widening, and Family-2 router execution remain deferred.

## Plan Summary

After Slice `42`, the repo can now handle the conversational producer-consumer loop for retained follow-up: `follow_up_question` persists durably and typed host `clarification_response` consumes it over the existing `continue_world_worker` seam.

The remaining host-side typed classes are now `progress_ack`, `control_directive`, `control_ack`, and `fork_command`. The narrowest honest next widening is `control_directive` because:

1. it is a canonical host-to-worker operational class in the design stack,
2. it can stand alone without inventing a new durable obligation consumer model,
3. it is more strategically meaningful than `progress_ack`, which is explicitly optional and still lacks an exact durable causation anchor in live repo truth,
4. it can stay smaller than a transport redesign by compiling onto the already-landed member-turn prompt submit seam.

What the repo already has:

1. the seven-verb Family-1 internal dispatch surface,
2. exact retained-worker targeting and world-binding validation through `continue_world_worker`,
3. a live member-turn submit seam that already carries free-form continue prompts plus narrow typed `approval_response` and `clarification_response`,
4. a clear design vocabulary for operational directive kinds such as pause, reduce scope, summarize, checkpoint, and prepare handoff.

What the repo still lacks is:

1. a typed host-side control-directive contract at the dispatch boundary,
2. explicit deny-by-default policy truth for typed host operational steering,
3. deterministic implementation-owned rendering for typed control directives,
4. regression coverage proving delivery stays honest without implying `control_ack`.

The narrowest honest implementation order is:

1. freeze the typed control-directive contract and policy surface first,
2. freeze deterministic rendering and bounded directive taxonomy second,
3. wire live delivery and no-ack implication regressions third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. add a typed internal `control_directive` host payload to the `continue_world_worker` contract,
2. add the minimum deny-by-default policy gate needed for typed host control directives,
3. freeze a small initial directive-kind taxonomy,
4. render typed control directives deterministically onto the existing retained member-turn seam,
5. keep outcome semantics honest about successful delivery without implying later acknowledgement.

### What does not change

1. no new dispatch verb,
2. no public CLI or toolbox control-directive surface,
3. no `control_ack`, `fork_command`, or `progress_ack` in this slice,
4. no transport-api schema widening in this slice,
5. no active-ephemeral inspect/cancel widening,
6. no router/daemon execution redesign,
7. no host-global inbox or remote ingress layering.

## Implementation Order

### Packet 1: Typed Control Directive Contract And Policy Surface

Goal:

1. freeze the typed host-side control-directive payload shape,
2. add minimal deny-by-default policy parsing and denial buckets,
3. keep broader host control/ack/fork classes deferred.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. targeted contract and policy tests

Why first:

1. live delivery should sit on a frozen typed contract,
2. the runtime needs explicit deny-by-default policy truth before typed control directives can be accepted safely,
3. later packets should not infer host control semantics from free-form prompts.

Verification checkpoint:

1. typed `control_directive` payloads validate,
2. the new policy key is deny-by-default,
3. explanation-ready denials exist for disallowed typed control directives,
4. broader host control/ack/fork classes remain deferred.

### Packet 2: Deterministic Rendering And Directive Taxonomy

Goal:

1. freeze the narrow initial directive-kind set,
2. compile typed control directives onto the existing live member-turn prompt seam deterministically,
3. keep rendering implementation-owned and reviewable.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. targeted contract and routing tests

Why second:

1. the slice needs a frozen bounded vocabulary before live delivery is treated as real operational steering,
2. deterministic rendering should be explicit before outcome semantics are considered complete,
3. this keeps the slice smaller than a broader typed transport redesign.

Verification checkpoint:

1. accepted typed control directives render deterministically,
2. directive kinds outside the bounded set fail closed,
3. normal prompt-based continue remains valid and unchanged.

### Packet 3: Outcome Honesty And No-Ack-Implication Regression Proof

Goal:

1. prove the already-landed live `continue_world_worker` control-directive delivery reports truthful operational outcomes,
2. prove that successful delivery reports truthful operational outcomes,
3. prove that the slice does not imply or require typed `control_ack`.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. targeted shell integration and regression tests

Why third:

1. the live runtime path should consume the frozen contract, policy surface, and deterministic renderer,
2. this packet proves the already-landed delivery path stays honest about runtime outcomes rather than implying more transport than exists,
3. the key safety property is outcome honesty: delivery is real, but worker acknowledgement is still deferred.

Verification checkpoint:

1. typed control directives submit through the existing retained member-turn seam,
2. successful delivery reports a truthful summary without implying `control_ack`,
3. delivery failure returns explanation-ready errors,
4. existing approval/clarification behavior remains green.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `43` scope,
2. keep `control_ack`, `fork_command`, `progress_ack`, active-ephemeral identity, and Family-2 work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-43-internal-retained-host-control-directive-bootstrap.md`
3. `llm-last-mile/PLAN-43.md`
4. `llm-last-mile/TASKS-43.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `43` as typed host control-directive bootstrap over the existing `continue_world_worker` seam, not generalized control transport or Family-2 router execution,
2. docs keep `control_ack`, `fork_command`, and active-ephemeral identity widening explicitly deferred,
3. docs keep Family-2 router execution downstream.

Verification checkpoint:

1. docs remain honest about scope,
2. validation is green,
3. the next follow-on slice can sequence `control_ack` or `fork_command` without reopening Slice `43`.

## Risks And Mitigations

1. Risk: the slice quietly widens into a generic operational control language.
   Mitigation: freeze a small initial directive-kind set and keep rendering implementation-owned.
2. Risk: the slice implies that typed `control_ack` already exists.
   Mitigation: keep outcome summaries explicit about delivery only; defer acknowledgement semantics to a later slice.
3. Risk: typed control directive drags in `fork_command` or other higher-consequence control classes.
   Mitigation: limit the accepted host-side typed class to `control_directive` only.
4. Risk: `progress_ack` would have been the smaller next step and the plan overreaches.
   Mitigation: keep the slice strictly limited to self-contained host steering; do not invent new durable causation or reply semantics.
5. Risk: the slice is misclassified as public control-surface work.
   Mitigation: keep it internal-only and reuse the retained `continue_world_worker` seam exactly.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because rendering and live delivery depend on a frozen typed control-directive contract and policy surface.
2. Packet 2 before Packet 3 because the live path should consume one bounded directive taxonomy and one deterministic renderer.
3. Packet 3 before Packet 4 because docs and validation should reflect final live behavior.

### Can be parallelized later

1. typed worker `control_ack` after Slice `43` is stable,
2. typed host `fork_command` after control-directive semantics are frozen if still needed,
3. optional typed `progress_ack` if still needed after the operational steering slice lands,
4. active-ephemeral exact task identity work as its own later slice,
5. Family-2 router execution after the Family-1 producer and consumer semantics are frozen enough for downstream attach behavior.

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

## Expected Follow-On Order After Slice `43`

If Slice `43` lands cleanly as narrow typed control-directive bootstrap first, the next follow-on work should remain:

1. typed worker `control_ack` only, bound to exact retained-worker control-directive causation if still needed,
2. typed host `fork_command` only if still needed after control-directive semantics are frozen,
3. optional typed `progress_ack` only if still needed after the operational steering slice lands,
4. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
5. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `43` implementation scope.
