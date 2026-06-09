# Remaining Host-Orchestrator Tool Invocation Surface Scope

Date: `2026-06-08`  
Validated against:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [REMAINING-family-1-and-2-scope-2026-06-08.md](./REMAINING-family-1-and-2-scope-2026-06-08.md)
- [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
- [`docs/USAGE.md`](../docs/USAGE.md)
- live runtime code in:
  - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
  - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
  - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
  - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

## Objective

Record the repo-truth answer to one question after the new toolbox DESIGN docs were added:

1. what is already landed for the internal host-orchestrator control plane,
2. what is still missing before a live host AI can actually call those capabilities inside its own session,
3. why that remaining work is likely a multi-slice implementation family rather than one small follow-on patch,
4. and what order the follow-on slices should take.

This note is a tracking and sequencing artifact, not a new implementation spec.

## Scope Definition

For this note:

1. "internal toolbox transport" means the landed session-scoped internal world-dispatch caller seam.
2. "tool invocation surface" means the agent-visible callable tool boundary inside the host orchestrator session.
3. "runtime-family adapter" means the shell-owned bridge that makes those tools visible and executable for a specific orchestrator runtime family such as `codex` or `claude_code`.
4. "MCP" means an optional later adapter/export shape, not the current landed internal wire truth.

## Ongoing Ledger Maintenance

Use this note as the canonical running ledger for this seam.

When new work on the host-orchestrator tool invocation surface surfaces:

1. execution drift between design docs and live code,
2. bounded follow-on items that should be deferred,
3. validation discoveries that need to be circled back around to later,
4. sequencing changes that affect which slice should come next,

record them here rather than burying them only in a spec, plan, tasks doc, or chat transcript.

### Where to add new notes

Add new items only in the three running-ledger sections near the end of this file:

1. `## Newly Surfaced During Execution`
2. `## Deferred / Circle-Back Items`
3. `## Resolved Since Last Update`

### How to write each note

For each new item:

1. start with the date and the slice or packet that surfaced it,
2. identify whether it is:
   - live-truth drift,
   - explicit deferral,
   - validation finding,
   - or sequencing change,
3. name the affected docs and code files directly,
4. say whether the item changes:
   - the current slice scope,
   - a later slice,
   - or only doc truth,
5. if follow-up work is required, say which later slice family it belongs to.

### Movement rules

1. Add new discoveries to `Newly Surfaced During Execution` first.
2. If the item is intentionally not fixed in the current slice, copy or move it into `Deferred / Circle-Back Items`.
3. Once the repo truth or the planning truth is reconciled, move the item into `Resolved Since Last Update`.
4. If any item changes the recommended slice order, update `## Recommended Slice Order` and `## Bottom Line` in this same file during the same edit.
5. Keep this note concise and execution-facing; do not turn it into a replacement design doc.

## Current Repo Truth

The tree is no longer missing dispatch runtime fundamentals.

### 1. The internal control-plane transport is already landed

The current tree already has:

1. a session-scoped toolbox endpoint derived from exact `orchestration_session_id`,
2. a typed `WorldDispatchRequestV1` contract,
3. a Unix-first newline-delimited JSON request/response framing model,
4. event-capable response behavior,
5. typed outcomes for the landed Family-1 verb set.

Repo-truth consequence:

1. the missing work is not "invent internal dispatch runtime,"
2. the missing work is above the transport, not below it.

### 2. The operator-visible toolbox surface remains introspection-only

The current operator-facing posture is still:

1. `substrate agent toolbox status`
2. `substrate agent toolbox env`

Those surfaces expose endpoint/posture truth, but they do not expose a public human CLI for the internal dispatch verbs.

Repo-truth consequence:

1. the host agent still lacks an actual callable tool boundary,
2. prompt text or environment hints alone are not the missing integration layer.

### 3. The shell-owned orchestrator families are known, but tool injection is not yet landed

The runtime-realizable orchestrator families remain:

1. `codex`
2. `claude_code`

But the current tree does not yet show a landed family-specific tool registration or runtime-owned tool injection seam for the world-dispatch verbs.

Repo-truth consequence:

1. the next work is adapter/tool-surface work,
2. that work will likely need at least one semantic slice plus one runtime-family landing slice.

## What Is Already Closed

The following questions should now be treated as closed enough for sequencing:

1. whether the internal toolbox transport exists at all,
2. whether the transport is session-scoped,
3. whether the transport is typed or ad hoc,
4. whether the internal control plane already has a real verb family,
5. whether the v1 direction should treat the transport as the source of truth,
6. whether prompt-only is sufficient for the host agent,
7. whether subprocess `substrate ...` calls should define the primary in-session model.

## What Is Still Missing

### 1. Agent-visible tool registration and discovery

The host agent still needs a real way to:

1. see what tools exist,
2. understand their structured arguments,
3. call them naturally from within the live session.

This is not solved by the current transport alone.

### 2. Runtime-owned argument injection

The adapter still needs to define exactly how it will supply:

1. `request_id`
2. `idempotency_key`
3. `orchestration_session_id`
4. `caller_participant_id`
5. authoritative `world_id`
6. authoritative `world_generation`

without forcing the model to invent them.

### 3. Receipt-oriented call semantics

The host tool surface still needs a frozen answer for:

1. how `run_world_task` returns early identity such as `task_run_id`,
2. how `spawn_world_worker` returns authoritative retained-worker receipts,
3. how later `inspect`, `continue`, `cancel`, and `stop` consume those durable identities.

### 4. Runtime-family landing order

The repo still needs one sequencing decision:

1. `codex` first with later `claude_code` parity,
2. or immediate dual-family work.

### 5. Smoke and operator-truth follow-through

After the first adapter lands, the tree will still need bounded follow-through on:

1. runtime-family smoke coverage,
2. docs alignment,
3. honest operator/runtime truth for what is now internal-tool-callable versus still deferred.

## Why This Is Probably More Than Two Slices

The new DESIGN docs likely describe at least three distinct implementation problems:

1. define the runtime-owned adapter contract and the agent-visible tool vocabulary,
2. land one real runtime-family adapter that exposes those tools in a live orchestrator session,
3. harden the receipt/resume/follow-up path so the agent can safely continue, inspect, or cancel later work,
4. optionally repeat the adapter landing for the second runtime family if parity is not included in the first landing,
5. align docs/smoke/operator truth around the new live capability.

Repo-truth consequence:

1. forcing all of that into one or two slices risks scope inflation,
2. a small ordered family is more honest and more reviewable.

## Recommended Slice Order

If the team wants the narrowest honest implementation sequence, the order should be:

### Slice A: Runtime-owned adapter contract freeze

Objective:

1. freeze the agent-visible tool vocabulary,
2. freeze which arguments are model-supplied versus runtime-injected,
3. freeze receipt-oriented result semantics for `run_world_task` and `spawn_world_worker`,
4. keep the slice bounded to contract/adapter-shape work rather than full runtime-family productization.

Why first:

1. both supported runtime families need one shared semantic contract,
2. later implementation slices should not invent their own ad hoc tool shapes.

### Slice B: First runtime-family adapter landing

Recommended first target:

1. `codex`

Objective:

1. expose the frozen tool set inside one live host orchestrator family,
2. route tool calls through the landed internal toolbox transport,
3. prove runtime-owned injection of session/caller/world identity.

Why second:

1. it creates the first end-to-end live host-agent delegation path,
2. it proves whether the adapter model is honest before parity work begins.

### Slice C: Receipt/resume/inspection hardening

Objective:

1. tighten the lived experience around `task_run_id`, retained-worker receipts, and later follow-up calls,
2. prove the host can move on or park and later resume/inspect/cancel safely,
3. keep the control-plane model receipt-oriented rather than synchronous-only.

Why third:

1. the first adapter landing may prove the visibility/mechanics seam,
2. this slice then makes the orchestration behavior operationally solid.

### Slice D: Second runtime-family parity

Recommended target:

1. `claude_code`

Objective:

1. preserve the same semantic tool contract,
2. land family-specific exposure mechanics without changing control-plane meaning,
3. prove parity on the second supported orchestrator runtime family.

Why fourth:

1. parity work should consume the proven contract and first-family lessons,
2. it should not reopen the core transport or tool semantics.

### Slice E: Docs, smoke, and truth alignment

Objective:

1. align runtime docs with the now-live agent-visible tool surface,
2. capture bounded smoke expectations,
3. keep queued MCP language clearly separate from the landed Substrate-native adapter path.

Why last:

1. docs should describe what actually landed,
2. this keeps earlier slices focused on runtime truth first.

## Parity Strategy Recommendation

The safer default is:

1. shared semantic contract first,
2. `codex` first live landing,
3. `claude_code` parity second.

Why:

1. the repo already treats `codex` and `claude_code` as runtime-family truth, but not as identical exposure mechanisms,
2. one-family-first lowers blast radius,
3. parity can then be measured against a real working contract instead of a speculative dual landing.

This note does not claim `codex` is philosophically preferred; only that it is the narrower first implementation seam.

## Key Open Questions For The First Spec

The next spec should still choose explicitly:

1. whether Slice A and Slice B stay separate or are combined into one very narrow first implementation slice,
2. whether the first agent-visible landing exposes all seven verbs immediately or only the minimum live subset,
3. whether the first slice includes any smoke/debug helper surface or keeps that fully out of scope,
4. whether the second runtime-family parity slice is mandatory before the family is considered complete.

## Newly Surfaced During Execution

### 2026-06-08 — Slice 52 contract-freeze planning pass

1. **Live-truth drift: internal request-envelope breadth**
   - The conceptual request envelope in [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md) is broader than the live `WorldDispatchRequestV1` shape in [`dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs).
   - Current Slice `52` resolution: freeze the adapter contract against the live request envelope and do not widen the internal transport in the contract-freeze slice.
2. **Live-truth drift: universal `idempotency_key` validation**
   - Live `WorldDispatchRequestV1::validate()` currently requires non-empty `idempotency_key` for every action.
   - Current Slice `52` resolution: treat `idempotency_key` as a universal runtime-injected field in the adapter contract.
3. **Live-truth drift: active-ephemeral outcome identity asymmetry**
   - Active-ephemeral inspect/cancel input uses exact `task_run_id`, but current typed internal outcomes still reuse `target_participant_id` for that identity in [`dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) and [`orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs).
   - Current Slice `52` resolution: normalize the adapter-visible contract to canonical `task_run_id` without rewriting the internal outcome plane in the contract-freeze slice.

## Deferred / Circle-Back Items

### 2026-06-08 — follow-ons explicitly deferred by Slice 52 planning

1. **Possible transport-envelope widening**
   - If later runtime-family landing or trace requirements truly need fields beyond the live `WorldDispatchRequestV1` shape, land that as a separate transport-focused slice rather than sneaking it into adapter work.
2. **Possible validator narrowing for `idempotency_key`**
   - If the repo later wants `idempotency_key` to be required only for selected create-style actions, that should be a separate validator/wire cleanup slice.
3. **Possible internal typed-outcome harmonization**
   - If the repo wants active-ephemeral inspect/cancel outcomes to expose `task_run_id` directly instead of reusing `target_participant_id`, do that in a later receipt/resume hardening or typed-outcome cleanup slice.
4. **First runtime-family landing**
   - The next execution-bearing follow-on after Slice `52` remains the first real runtime-family adapter landing, with `codex` still the narrower default first target unless live repo truth changes.

## Resolved Since Last Update

### 2026-06-08

1. The next honest seam after the new toolbox design docs is now explicitly frozen as **runtime-owned adapter contract work above the landed internal transport**, not more transport invention and not Family-2 inbox/router reopening.

## Do Not Reopen

The following should stay frozen unless live evidence forces otherwise:

1. do not reopen MCP as the required internal wire protocol for v1,
2. do not reopen whether the internal toolbox transport exists,
3. do not reopen prompt-only as a sufficient host-agent integration model,
4. do not reopen subprocess CLI as the primary in-session invocation model,
5. do not reopen Family-1 dispatch runtime invention below the existing transport,
6. do not reopen Family-2 inbox/obligation semantics unless the new adapter work directly proves a contradiction.

## Bottom Line

Current repo truth is:

1. the internal toolbox transport and typed dispatch runtime are already landed,
2. the remaining missing seam is the live host agent’s tool-invocation surface,
3. that remaining seam is likely a small multi-slice family rather than one patch,
4. the honest order is shared adapter contract first, then first-family live adapter landing, then receipt/resume hardening, then second-family parity, then docs/smoke truth alignment.
