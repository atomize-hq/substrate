# Remaining Family-1 Scope After Slice 32 Bootstrap

Date: `2026-05-31`  
Validated against:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)
- live runtime code in:
  - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
  - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
  - [`docs/USAGE.md`](../docs/USAGE.md)
  - [`docs/TRACE.md`](../docs/TRACE.md)
  - [`docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md`](../docs/adr/draft/ADR-0045-orchestration-toolbox-internal-mcp-identity-trace-contract.md)
  - [NOTE-33-family-1-ordering-after-dispatch-bootstrap.md](./NOTE-33-family-1-ordering-after-dispatch-bootstrap.md)
  - [SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md](./SPEC-33-internal-retained-world-worker-continue-and-event-bootstrap.md)
  - [PLAN-33.md](./PLAN-33.md)
  - [TASKS-33.md](./TASKS-33.md)

## Objective

Record the repo-truth answer to one question:

1. after Slice `32` and its post-landing truth-sync patches, what remains for the separate host-orchestrator to world control-plane family,
2. and what is the narrowest honest next slice to spec before code work begins?

This note is a validation artifact, not an implementation spec.

## Scope Definition

For this note, "family 1" means the host-orchestrator to world control-plane design stack:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
2. [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
3. [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)

This note does not reopen the durable deferred-work family except for dependency boundaries:

1. [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)
2. [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)

## Current Repo Floor

The current tree already has several runtime primitives that future family-1 work must consume rather than replace.

### 1. World-member execution and exact follow-up are real

The repo already supports:

1. exact retained world-member launch through typed `member_dispatch`,
2. exact retained follow-up through typed `member_turn_submit`,
3. fail-closed authoritative world-binding checks,
4. exact public `(orchestration_session_id, backend_id)` turn targeting.

Repo-truth implication:

1. family 1 is not blocked on basic world-member runtime capability,
2. the missing seam is the host-orchestrator control plane that would call into those primitives.

### 1.5. Slice `32` bootstrap is now landed

The repo now also supports:

1. internal `run_world_task` dispatch over the real world-member execute seam,
2. internal `spawn_world_worker` retained bootstrap with authoritative worker identity and launch receipt,
3. orchestrator-only internal caller validation plus exact same-session and same-world-binding checks.

Repo-truth implication:

1. family 1 is no longer design-only end to end,
2. the remaining work starts after dispatch/bootstrap rather than before it.

### 2. The narrow public control surface is not the family-1 control plane

The repo already ships public `substrate agent start|turn|reattach|fork|stop` and REPL targeted turns.

That is not the same thing as the missing family-1 architecture.

Repo-truth implication:

1. current public and REPL surfaces are human/operator caller paths,
2. they do not freeze an internal host-orchestrator verb contract,
3. they do not provide the typed request/outcome envelope family described in the design docs.

### 3. Toolbox remains introspection-only in shipped code

`docs/USAGE.md` still describes:

1. `substrate agent toolbox status` as pre-runtime introspection,
2. `substrate agent toolbox env` as endpoint projection only,
3. no live world-execution tool surface for the orchestrator.

`ADR-0045` still marks the internal toolbox as queued architecture input, not landed implementation truth.

Repo-truth implication:

1. there is no shipped orchestrator-callable internal MCP/toolbox surface yet,
2. family 1 cannot be treated as partially landed through toolbox.

## What Is Still Missing

Family 1 remains incomplete as a distinct control plane even though its first bootstrap slice is now landed.

### 1. No landed retained-worker continue verb surface

The family-1 verb names:

1. `continue_world_worker`
2. `inspect_world_worker`
3. `fork_world_worker`
4. `cancel_world_work`
5. `stop_world_worker`

do not yet exist as landed internal follow-up/runtime control-plane actions in the current tree.

### 2. No landed retained-worker continue/event protocol

The next required messaging/event layer described in the design docs is still missing as runtime truth. In particular, the current tree does not yet have a landed internal continue contract that classifies the first retained-worker event subset:

1. `reply`
2. `progress_update`
3. `follow_up_question`
4. `blocked`
5. `result`
6. `failure`
7. explicit `attention_required` semantics for the attention-driving classes

Repo-truth implication:

1. the next missing seam is internal retained-worker continue plus minimal typed event bootstrap,
2. fuller approval/fork/control-directive classes still come later.

### 3. No landed steering-policy layer for host-orchestrator world control

The deny-by-default policy matrix described in the design docs is also still design-only.

The current tree does not show a dedicated host-to-world steering layer that separately gates:

1. actions,
2. modes,
3. exact target backends,
4. same-session boundary,
5. same-world-binding boundary,
6. worker fork autonomy.

### 4. No landed later verb expansion after continue

Even after the next continue/event slice, the later family-1 verbs still remain:

1. `inspect_world_worker`
2. `cancel_world_work`
3. `stop_world_worker`
4. `fork_world_worker`
5. approval/fork event classes and worker-requested fork autonomy

## Why Dispatch Comes First

The honest dependency order is:

1. dispatch/allocation first,
2. retained-worker messaging/steering second,
3. fuller steering-policy hardening third.

Reason:

1. the dispatch design explicitly defines itself as the first implementation-shaping contract,
2. the messaging design assumes retained worker identity already exists,
3. the policy matrix is safest when frozen against a concrete dispatch entry seam instead of a hypothetical future one.

The current repo evidence supports that order:

1. runtime already knows how to launch and target exact world members,
2. what is missing is the internal host-orchestrator call path and its typed request/outcome contract,
3. messaging and fork/approval/event semantics naturally sit on top of that first allocation seam.

## Recommended Next Slice

The narrowest honest next slice is:

1. **Slice `33`: internal retained-world-worker continue and minimal typed event bootstrap**

This slice should:

1. stay internal-only,
2. introduce only `continue_world_worker` as the next family-1 verb,
3. reuse the already-landed retained member-turn seam rather than inventing a second execution plane,
4. classify only the first typed worker-event subset:
   - `reply`
   - `progress_update`
   - `follow_up_question`
   - `blocked`
   - `result`
   - `failure`
5. keep `attention_required` explicit for the in-scope attention-driving classes,
6. defer fuller steering-policy hardening, approval/fork event classes, and the later inspect/cancel/stop/fork verbs.

## Boundary Against Family 2

This slice should come before any further family-2 expansion because:

1. family 2 already has landed local obligation-ledger and auto-attach mechanics,
2. family 1 still lacks its first live retained-worker continue and typed event seam,
3. future family-2 producers such as worker follow-up, approval, blocked, and fork-request obligations depend on the later messaging contract that now sits on top of the landed dispatch bootstrap.

What stays out of scope here:

1. router/daemon execution model changes,
2. obligation-ledger schema broadening,
3. host-global inbox or cross-host ingress,
4. detached host auto-attach policy widening.

## Bottom Line

Family 1 is no longer design-only in the current repo:

1. world-member runtime primitives are landed,
2. the first internal dispatch/bootstrap slice is landed,
3. the next missing slice is internal retained-worker continue plus minimal typed event bootstrap,
4. the remaining follow-on work after that is fuller steering-policy hardening and only then later verb expansion.
