# PLAN-52: Internal Runtime-Owned Host-Orchestrator Tool Adapter Contract Freeze

Source spec: [SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)  
Source tracker note: [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)  
Companion design inputs:
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
Plan type: first post-toolbox-design adapter-contract freeze slice  
Status: landed runtime truth reviewed and Packet 4 closeout verified on `2026-06-09`
Validation note: Packet 4's validation wall is green. Final closeout required one bounded validation-driven refactor in [`tool_invocation_contract.rs`](../crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs), but Slice `52` still remains bounded to the Substrate-native adapter-contract freeze and still does not imply live `codex`/`claude_code` tool registration, MCP exposure, or public human toolbox execution.

## Objective

Freeze one runtime-owned adapter contract for the host orchestrator’s future agent-visible tool surface, above the already-landed internal toolbox transport and below any later runtime-family-specific registration.

This slice is complete only when:

1. the seven tool names are frozen against live dispatch-action truth,
2. runtime-owned versus model-supplied arguments are frozen,
3. receipt and follow-up handle semantics are frozen,
4. the contract is explicitly Substrate-native in v1,
5. the slice lands without tool registration inside `codex` or `claude_code`, without a public CLI expansion, and without transport redesign.

## Phase Gate

This plan assumes the `SPECIFY` artifact in [SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md) has been reviewed and accepted as the bounded source of truth before implementation starts.

## Tracker Update Rule

The canonical running ledger for new drift, deferrals, and circle-back items in this seam is:

- [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)

During any Packet `1`-`4` execution or review pass:

1. add newly surfaced items to the tracker’s `## Newly Surfaced During Execution`,
2. move intentional non-fixes or later-slice work into `## Deferred / Circle-Back Items`,
3. move reconciled items into `## Resolved Since Last Update`,
4. update the tracker’s `## Recommended Slice Order` and `## Bottom Line` if the new item changes sequencing.

Do not rely on the packet closeout message alone as the durable record for those notes.

## Repo-Truth Framing

The current repo is past the transport-invention phase.

What is already landed:

1. session-scoped internal toolbox transport,
2. exact typed internal dispatch actions,
3. retained-worker and active-ephemeral follow-up routing,
4. exact active task and retained participant identity checks,
5. operator-visible toolbox introspection.

What is still missing:

1. a frozen model-visible contract the runtime can present later,
2. one explicit split between model intent and runtime-owned injected truth,
3. one exact follow-up handle model that the first runtime-family landing can reuse.

## Live-Truth Corrections This Plan Must Respect

The design inputs are directionally right, but this slice must follow live code where they differ.

1. The internal request envelope in live `dispatch_contract.rs` is narrower than the conceptual design envelope.
   - Do not reintroduce unlanded request fields such as `caller_backend_id`, `caller_role`, `capability_overrides`, `requested_policy_narrowing`, or `created_at` as if they are already transport truth.
2. Live validation currently requires `idempotency_key` on every internal dispatch request.
   - The adapter contract should therefore freeze runtime-owned id generation against current validation, not against a narrower create-only interpretation.
3. Active-ephemeral inspect/cancel identity is exact on input (`task_run_id`), but current typed internal outcomes still reuse `target_participant_id` for that identity.
   - The adapter contract should normalize that asymmetry without rewriting the underlying internal transport in this slice.

## Drift-Resolution Decisions Locked By This Plan

These are not open implementation choices for Slice `52`; they are the plan-level resolution of the live-truth drift.

1. **Request-envelope drift**
   - Freeze the adapter contract against the live `WorldDispatchRequestV1` field set.
   - Do not widen the internal transport in Slice `52`.
   - If the repo later wants the broader conceptual envelope, land that as a separate transport-widening slice.
2. **`idempotency_key` drift**
   - Treat `idempotency_key` as a universal runtime-injected field for all adapter calls that become internal dispatch requests.
   - Do not ask the model to supply it.
   - Do not narrow validator behavior inside Slice `52`.
3. **Active-ephemeral outcome-identity drift**
   - Normalize the adapter-visible contract so active-ephemeral receipts and follow-up outcomes use canonical `task_run_id`.
   - Leave the current internal typed-outcome asymmetry in place for now.
   - If internal field harmonization is desired later, treat it as follow-on receipt/resume hardening work.

## Major Components And Dependencies

1. bounded adapter contract surface
   - likely a new module under `crates/shell/src/execution/agent_runtime/`
   - owns the model-visible tool vocabulary, input shapes, receipt shapes, and follow-up handle rules
   - must remain distinct from the internal wire contract in `dispatch_contract.rs`
2. internal dispatch contract truth
   - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - stays the transport authority and must not be replaced
3. authoritative routing and receipt builders
   - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - already owns exact receipt creation and exact follow-up validation
   - should be reused by the adapter contract rather than duplicated
4. authoritative retained and active-task state
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - may need bounded helper reuse for exact handle resolution
5. runtime-family/tooling surfaces
   - `crates/shell/src/repl/async_repl.rs`
   - `crates/shell/src/execution/agents_cmd.rs`
   - these are evidence surfaces for the current toolbox posture, but Slice `52` must not widen them into real family registration yet

## Plan Summary

After the new toolbox DESIGN docs and the 2026-06-08 remaining-scope tracker note, the next honest execution-bearing seam is not a runtime-family implementation yet.

The narrowest honest Slice `52` is:

1. freeze the shared tool contract first,
2. freeze runtime-owned injection rules second,
3. freeze exact receipt and follow-up identity semantics third,
4. finish with docs/comments/validation last,
5. then hand a stable contract to the first runtime-family landing slice.

## Locked Decisions

### What this slice changes

1. It freezes one shared adapter-visible tool vocabulary over the landed internal verbs.
2. It freezes which arguments the model may supply versus which arguments the runtime must inject.
3. It freezes receipt-oriented semantics for fresh work allocation and later follow-up.
4. It freezes that the v1 adapter path is Substrate-native rather than MCP-first.

### What this slice does not change

1. no live `codex` tool registration yet,
2. no live `claude_code` tool registration yet,
3. no MCP server landing,
4. no public human `substrate agent toolbox <verb>` CLI,
5. no redesign of the internal toolbox transport,
6. no reopening of Family-2 inbox/router semantics.

## Implementation Order

### Packet 1: Shared Adapter Contract And Handle Vocabulary Freeze

Goal:

1. define the adapter-visible tool vocabulary,
2. define the two exact follow-up handle families,
3. keep the contract grounded to the already-landed internal verb set.

Primary touch surface:

1. one new bounded adapter-contract module under `crates/shell/src/execution/agent_runtime/`
2. [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) only where cross-references or bounded helper types are required
3. unit tests proving one-to-one mapping between adapter-visible tools and internal dispatch actions

Why first:

1. runtime-family landing should not invent its own tool names,
2. handle semantics depend on a frozen shared vocabulary,
3. this packet prevents later family-specific drift.

Verification checkpoint:

1. exactly seven tool names are frozen,
2. exact `task_run_id` and `participant_id` handles are frozen,
3. the shared adapter contract depends only on the live internal request envelope,
4. no runtime-family registration work has started.

### Packet 2: Runtime-Owned Injection Rules And Fresh-Allocation Translation

Goal:

1. freeze and implement the runtime-owned injection split,
2. map `run_world_task` and `spawn_world_worker` tool calls to the landed internal request contract,
3. keep runtime-owned ids and world binding out of model-authored input.

Primary touch surface:

1. the new adapter-contract module
2. [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
3. [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) only if a bounded authoritative binding helper is needed

Why second:

1. fresh allocation is where runtime-owned identity injection matters most,
2. live request validation already proves which fields are required,
3. later follow-up semantics should consume the same injected authority model.

Verification checkpoint:

1. `run_world_task` and `spawn_world_worker` map into valid internal requests,
2. model input cannot override runtime-owned session/caller/world-binding fields,
3. live `idempotency_key` requirements are satisfied for every internal request path,
4. Slice `52` has not changed validator scope or transport fields to achieve that.

### Packet 3: Outcome Normalization And Exact Follow-Up Resolution

Goal:

1. normalize internal typed outcomes into adapter-visible receipts,
2. freeze exact follow-up resolution semantics for retained versus active-ephemeral handles,
3. avoid forcing the model to compensate for internal outcome asymmetries.

Primary touch surface:

1. the new adapter-contract module
2. [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
3. [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) if bounded handle lookup helpers are needed

Why third:

1. outcome normalization depends on the Packet 1 vocabulary and Packet 2 injection rules,
2. follow-up semantics are the main operational risk in this slice,
3. this packet lets later runtime-family landing consume a stable receipt model instead of raw internal structs.

Verification checkpoint:

1. `run_world_task` yields an adapter-visible exact active-task receipt,
2. `spawn_world_worker` yields an adapter-visible exact retained-worker receipt,
3. `inspect_world_worker` and `cancel_world_work` accept exactly one handle family at a time,
4. active-ephemeral adapter-visible outcomes surface canonical `task_run_id` despite the current internal typed-outcome asymmetry,
5. `continue_world_worker` and `stop_world_worker` remain retained-only.

### Packet 4: Repo-Local Docs, Code Comments, And Validation

Goal:

1. align repo-local planning/docs/comments with the frozen contract,
2. keep later runtime-family landing explicitly deferred,
3. run the bounded validation wall.

Primary touch surface:

1. `llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md`
2. `llm-last-mile/PLAN-52.md`
3. `llm-last-mile/TASKS-52.md`
4. bounded code comments or developer-facing notes only if needed to prevent false runtime claims

Why fourth:

1. the docs should describe the contract that actually landed,
2. this packet keeps tool registration, parity, and smoke work clearly out of Slice `52`,
3. a clean closeout makes the next runtime-family slice easier to scope honestly.

Verification checkpoint:

1. docs/comments match live runtime truth,
2. the validation wall is green,
3. the slice still has not widened into runtime-family landing or MCP work.

## Risks And Mitigations

1. Risk: the slice freezes against the broader design-only envelope instead of live code.
   Mitigation: treat `dispatch_contract.rs`, `orchestrator_world_dispatch.rs`, and `state_store.rs` as source of truth and explicitly encode the live-truth corrections above.
2. Risk: the slice drifts into `codex` or `claude_code` tool registration.
   Mitigation: keep registration and family exposure mechanics out of scope; freeze only the shared contract.
3. Risk: adapter-visible handles become fuzzy or opaque enough to lose exact identity.
   Mitigation: keep exact `task_run_id` and exact `participant_id` as first-class handle truth even if wrapped in a structured receipt.
4. Risk: the model is still asked to repeat backend/world-binding details on follow-up calls even though the runtime already knows them authoritatively.
   Mitigation: freeze runtime-derived follow-up resolution wherever authoritative retained or active state already exists.
5. Risk: active-ephemeral outcome normalization reopens the internal transport contract.
   Mitigation: normalize only at the adapter layer; do not rewrite the landed internal outcome structs in this slice.

## Sequencing And Parallelism

This slice should run sequentially.

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

There is little safe parallelism because tool vocabulary, injection rules, and follow-up handles are tightly coupled. Parallel work would likely create incompatible local contract assumptions.

## Verification Checkpoints Between Packets

1. After Packet 1, confirm the frozen tool names and exact handle families are still aligned with live dispatch actions.
2. After Packet 2, confirm fresh-allocation translation injects runtime-owned truth and satisfies current request validation.
3. After Packet 3, confirm adapter-visible receipts and follow-up semantics remain exact and fail closed.
4. After Packet 4, confirm docs and validation reflect a contract freeze only, not a runtime-family landing.

## Explicit Later Slices After Slice 52

Slice `52` is intentionally not the runtime-family landing.

The likely honest follow-ons after this slice are:

1. **likely Slice 53: first runtime-family landing**
   - recommended first target: `codex`
   - register the frozen tools inside one real host runtime family
   - prove runtime-owned injection and internal toolbox routing end to end
2. **likely Slice 54: receipt/resume hardening**
   - harden the lived host experience around parked state, later inspect/cancel/continue, and receipt ergonomics
   - keep this distinct from the contract-freeze slice
3. **likely Slice 55: second runtime-family parity**
   - recommended target: `claude_code`
   - preserve the same semantics while landing family-specific exposure mechanics
4. **likely Slice 56: docs/smoke alignment**
   - align public docs, smoke coverage, and operator-truth surfaces to the newly live tool path

Those follow-ons should consume Slice `52`; they should not redefine it.

## Bottom Line

The next honest slice is a contract slice, not a productization slice.

Freeze the shared adapter contract now, then land runtime-family exposure against that contract later.
