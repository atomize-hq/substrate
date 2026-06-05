# PLAN-45: Internal Retained Host Fork Command Bootstrap

Source spec: [SPEC-45-internal-retained-host-fork-command-bootstrap.md](./SPEC-45-internal-retained-host-fork-command-bootstrap.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Plan type: first post-Slice-44 retained fork-command slice  
Status: drafted on `2026-06-05`

## Objective

Land the narrow host fork-orchestration slice by allowing host orchestrators to send typed `fork_command` messages through the existing `continue_world_worker` seam, bound to exact retained-worker identity and deny-by-default policy truth, while reusing the already-landed `fork_world_worker` bootstrap and lineage path, without widening into worker auto-fork, optional `progress_ack`, active-ephemeral task identity, Family-2 router execution, or a new public control surface.

This slice is complete only when all of the following are true:

1. the internal `continue_world_worker` contract accepts a typed host `fork_command` payload under exact session/world/participant truth,
2. a deny-by-default policy surface explicitly gates typed host fork commands,
3. typed host fork commands render deterministically onto the existing retained member-turn seam,
4. successful typed host fork commands reuse the already-landed fork bootstrap and keep source-to-child lineage explicit,
5. delivery and allocation outcomes remain honest about what happened,
6. worker auto-fork, optional `progress_ack`, active-ephemeral inspect/cancel widening, and Family-2 router execution remain deferred.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-45-internal-retained-host-fork-command-bootstrap.md](./SPEC-45-internal-retained-host-fork-command-bootstrap.md) has been reviewed and is the source of truth for scope before implementation planning advances.

## Major Components And Dependencies

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - owns the typed `continue_world_worker` payload contract and deterministic prompt rendering
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - owns payload-policy enforcement, exact-source routing, outcome summaries, and fork-bootstrap reuse
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - owns authoritative retained-worker identity, boundary truth, and lineage-preserving source resolution
4. `crates/shell/src/execution/policy_model.rs`
   - owns shell-side deny-by-default parsing and diagnostics for any new host fork-command gate
5. `crates/broker/src/policy.rs` and `crates/broker/src/effective_policy.rs`
   - must stay aligned with shell-local policy truth so YAML policy evaluation and diagnostics match runtime enforcement
6. `docs/CONFIGURATION.md`
   - must document the new gate and keep deferred scope explicit once runtime truth lands

## Plan Summary

After Slice `44`, the repo already has the host operational steering loop on the retained continue seam:

1. typed host `control_directive` is landed,
2. same-stream worker `control_ack` is landed,
3. worker `fork_request` and `fork_recommendation` obligations are already landed and remain host-mediated,
4. exact-source `fork_world_worker` allocation with explicit lineage is already landed.

What remains is the next distinct typed host class still deferred on that same seam: `fork_command`.

That is narrower than optional `progress_ack` because:

1. `progress_ack` remains explicitly optional in the design stack,
2. `progress_ack` still lacks a stronger exact live anchor than the already-frozen control loop,
3. `fork_command` can reuse already-landed exact-source fork bootstrap and lineage truth instead of inventing brand-new semantics.

That is narrower than Family-2 router or attach work because:

1. the fork allocator already exists in Family 1,
2. the missing gap is message-class bootstrap and policy truth on the retained continue seam,
3. no host-global inbox, router work-loop, or cross-host ingress widening is required to land it.

The narrowest honest implementation order is:

1. freeze the typed host fork-command contract and policy surface first,
2. freeze deterministic rendering and exact-source bootstrap reuse second,
3. prove outcome honesty and fail-closed rollback behaviour third,
4. finish with docs and the validation wall.

## Locked Decisions

### What changes

1. add a typed internal `fork_command` host payload to the `continue_world_worker` contract,
2. add the minimum deny-by-default policy gate needed for typed host fork commands,
3. render typed host fork commands deterministically onto the existing retained member-turn seam,
4. reuse the already-landed exact-source fork bootstrap and lineage path when the typed command is accepted,
5. keep delivery and allocation summaries explicit and truthful.

### What does not change

1. no new dispatch verb,
2. no public CLI or toolbox fork-command surface,
3. no worker auto-fork, worker-issued `fork_command`, or broader autonomy rubric in this slice,
4. no optional `progress_ack` in this slice,
5. no transport-api schema widening in this slice,
6. no active-ephemeral inspect/cancel widening,
7. no router/daemon execution redesign.

## Implementation Order

### Packet 1: Typed Fork Command Contract And Policy Surface

Goal:

1. freeze the typed host-side `fork_command` payload shape,
2. add minimal deny-by-default policy parsing and denial buckets,
3. keep optional `progress_ack` and broader autonomy deferred.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/policy_model.rs`
3. `crates/broker/src/policy.rs`
4. `crates/broker/src/effective_policy.rs`
5. targeted contract and policy tests

Why first:

1. routed fork reuse should sit on a frozen typed contract,
2. the runtime needs explicit deny-by-default policy truth before typed host fork commands can be accepted safely,
3. later packets should not infer fork-command semantics from generic continue or from direct `fork_world_worker` alone.

Verification checkpoint:

1. typed host `fork_command` payloads validate,
2. the new policy key is deny-by-default,
3. explanation-ready denials exist for disallowed typed host fork commands,
4. optional `progress_ack` and broader autonomy remain deferred.

### Packet 2: Deterministic Rendering And Exact-Source Bootstrap Reuse

Goal:

1. render typed host fork commands onto the existing live member-turn prompt seam deterministically,
2. resolve and preserve exact-source retained-worker truth before reuse,
3. prepare reuse of the landed fork bootstrap path without side effects yet.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. targeted contract and routing tests

Why second:

1. the slice needs one frozen typed rendering path before child-allocation behaviour is treated as real,
2. exact-source identity and lineage reuse must stay explicit before routing does side effects,
3. this keeps the slice smaller than a broader host-message transport redesign.

Verification checkpoint:

1. accepted typed host fork commands render deterministically,
2. exact retained source-worker targeting remains mandatory,
3. invalidated, terminal, or boundary-drifted sources fail closed before child allocation.

### Packet 3: Routed Allocation And Outcome Honesty

Goal:

1. route accepted typed host fork commands through the existing fork bootstrap path,
2. keep source-to-child lineage explicit,
3. keep summaries honest about delivery versus allocation and preserve fail-closed rollback behaviour.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. targeted shell integration and regression tests

Why third:

1. routed allocation should consume a frozen contract, policy surface, and exact-source resolver,
2. this packet proves typed host fork command is a real retained messaging-class widening rather than a doc-only placeholder,
3. the key safety property is one authoritative child-allocation path, not parallel fork machinery.

Verification checkpoint:

1. allowed typed host fork commands allocate one retained child and return explicit lineage,
2. successful summaries stay truthful about delivery and allocation,
3. failure returns explanation-ready errors and leaves no partial child side effects,
4. worker `fork_request`, worker `fork_recommendation`, `control_directive`, and optional `progress_ack` behaviour remains green and bounded.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align repo-local docs with the frozen Slice `45` scope,
2. keep worker auto-fork, optional `progress_ack`, active-ephemeral identity, and Family-2 work explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md`
3. `llm-last-mile/PLAN-45.md`
4. `llm-last-mile/TASKS-45.md`
5. targeted test suites

What this packet must enforce:

1. docs describe Slice `45` as typed host `fork_command` bootstrap over the existing `continue_world_worker` seam plus reuse of the landed fork bootstrap path, not generalized control transport or Family-2 router execution,
2. docs keep worker auto-fork, optional `progress_ack`, and active-ephemeral identity widening explicitly deferred,
3. docs keep Family-2 router execution downstream and do not imply public fork-command UX or generalized typed host transport.

Verification checkpoint:

1. docs and config truth remain honest about scope,
2. validation is green,
3. the next follow-on slice can sequence optional `progress_ack` or later identity/router work without reopening Slice `45`.

## Risks And Mitigations

1. Risk: typed host `fork_command` quietly becomes worker auto-fork or broader autonomy.
   Mitigation: keep Substrate as the allocator, keep worker-side autonomy deferred, and preserve separate deny-by-default worker-request gates.
2. Risk: the slice invents a second child-allocation path instead of reusing the landed fork bootstrap.
   Mitigation: route accepted typed host fork commands through the existing exact-source fork path and preserve explicit lineage truth.
3. Risk: summaries imply successful child allocation when only typed host command delivery succeeded.
   Mitigation: freeze outcome wording around delivery versus allocation truth explicitly.
4. Risk: optional `progress_ack` would have been the smaller next step and the plan overreaches.
   Mitigation: rely on the already-landed fork allocator and explicit planning stack order; keep `progress_ack` deferred unless repo evidence later proves it should come first.
5. Risk: the slice is misclassified as public fork UX or Family-2 work.
   Mitigation: keep it internal-only, exact-source, and tied to the existing retained continue seam plus landed fork bootstrap only.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because rendering and exact-source reuse depend on a frozen typed host fork-command contract and policy surface.
2. Packet 2 before Packet 3 because routed child allocation should consume one exact-source preparation path and one deterministic renderer.
3. Packet 3 before Packet 4 because docs and validation should reflect final routed behaviour.

### Can be parallelized later

1. optional typed `progress_ack` after typed host fork-command semantics are frozen if still needed,
2. active-ephemeral exact task identity work as its own later identity-model slice,
3. Family-2 router/attach execution after the Family-1 producer and consumer semantics are frozen enough for downstream attach behaviour.

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

## Expected Follow-On Order After Slice `45`

If Slice `45` lands cleanly as narrow typed host `fork_command` bootstrap first, the next follow-on work should remain:

1. optional typed `progress_ack` only, if still needed after the retained control and fork loop is frozen,
2. active-ephemeral exact task identity plus inspect/cancel widening only as its own later identity-model slice if still needed,
3. Family-2 router/attach execution only as its own downstream implementation track.

That follow-on order is part of this plan discipline. It is not part of Slice `45` implementation scope.
