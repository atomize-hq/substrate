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

Recommended first landed runtime-family exposure:

1. `codex`

Objective:

1. keep host orchestrator selection dynamic from effective config + effective inventory + exact backend policy,
2. expose the frozen tool set when the selected orchestrator resolves to the first landed runtime family,
3. route tool calls through the landed internal toolbox transport,
4. prove runtime-owned injection of session/caller/world identity without hard-coding the orchestrator id.

Why second:

1. it creates the first end-to-end live host-agent delegation path,
2. it proves whether the adapter model is honest before parity work begins,
3. it keeps `codex` as the first landed exposure mechanism without corrupting the repo's dynamic selection truth.

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
2. dynamic inventory-selected landing with `codex` as the first supported live tool-exposure family,
3. `claude_code` parity second.

Why:

1. the repo already treats `codex` and `claude_code` as runtime-family truth, but not as identical exposure mechanisms,
2. one-family-first lowers blast radius,
3. parity can then be measured against a real working contract instead of a speculative dual landing,
4. this preserves the current config/inventory/policy-driven orchestrator selection model instead of turning `codex` into a hard-coded orchestrator identity.

This note does not claim `codex` is philosophically preferred; only that it is the narrower first implementation seam and should be interpreted as the first landed exposure mechanism, not the only selectable orchestrator.

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

### 2026-06-09 — pre-Slice 53 spec pass

1. **Sequencing clarification: codex-first must not mean hard-coded orchestrator identity**
   - Live repo truth in [`validate_orchestrator_selection`](../crates/shell/src/execution/agent_runtime/validator.rs) and [`load_effective_agent_inventory`](../crates/shell/src/execution/agent_inventory.rs) shows the selected host orchestrator remains dictated by `agents.hub.orchestrator_agent_id` against the effective inventory loaded from `$SUBSTRATE_HOME/agents/` plus workspace `.substrate/agents/`.
   - Current Slice `53` resolution: treat `codex` as the first landed runtime-family exposure mechanism only. Do not hard-code `codex` as the selected orchestrator id or bypass exact backend/policy selection to get the first live tool surface.
2. **Dependency-surface drift: current generic host run request does not yet carry a first-class tool catalog, and the design inputs point to runtime-owned endpoint injection instead**
   - Repo/delivery review surfaced that current host prompt submission flows through the pinned `unified-agent-api` `AgentWrapperRunRequest`, which carries `prompt`, `working_dir`, `timeout`, `env`, and `extensions`, but no first-class generic tool-definition payload. The relevant design docs also freeze that the toolbox already exists as a session-scoped internal endpoint projected through `SUBSTRATE_AGENT_TOOLBOX_ENDPOINT` plus `SUBSTRATE_AGENT_TOOLBOX_VERSION`, so the missing work is a runtime-family adapter/injection seam above that transport rather than inventing a second tool-definition source.
   - Current Slice `53` resolution: prefer runtime-owned toolbox env injection through UAA for the first Codex landing, and use bounded Codex-specific config/home shaping only as needed to consume that injected endpoint honestly.
3. **Adapter-shape clarification: non-MCP first landing also needs prompt-side tool disclosure**
   - Reading [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md) and [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md) clarified that env projection alone is not the full integration layer: because the first landing is not MCP-server-based tool registration, the selected host runtime still needs startup/system-prompt disclosure of the seven-tool contract above the injected endpoint.
   - Current Slice `53` resolution: treat the first-family adapter as a two-part path — runtime-owned endpoint/version env injection plus startup/system-prompt injection of tool names, argument ownership, and receipt/follow-up semantics — while still routing actual calls through the frozen Slice `52` contract.

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

### 2026-06-09 — Slice 52 Packet 2 review/fix pass

1. **Resolved live-truth drift: retained follow-up admissibility must stay tool-specific**
   - Review surfaced that the first Packet `2` adapter translation pass overconstrained retained follow-up handles by requiring every retained target to remain authoritative-live before translation in [`tool_invocation_contract.rs`](../crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs).
   - That drift contradicted existing runtime truth in [`state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs), where exact retained `inspect_world_worker` can target non-live retained workers and exact retained `stop_world_worker` can target non-authoritative-live workers.
   - Packet `2` resolution: retain adapter translation in the bounded contract module, but make retained follow-up admissibility tool-specific so `inspect_world_worker` stays linkage-only, `stop_world_worker` stays non-terminal, and `continue` / `fork` / retained `cancel` stay authoritative-live.
2. **Resolved doc-truth alignment gap: Slice 52 closeout artifacts must stop reading as planning-only**
   - Packet `4` review surfaced that [`SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md`](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md), [`PLAN-52.md`](./PLAN-52.md), and [`TASKS-52.md`](./TASKS-52.md) still carried planning-era status text after the Packet `1`-`3` contract work had already landed.
   - Packet `4` resolution: update those repo-local slice artifacts to record the green validation wall and keep the Slice `52` closeout language explicit that runtime-family registration, MCP exposure, and public CLI landing remain follow-on work rather than implied runtime truth.
3. **Resolved validation follow-up: Packet 4 needed one bounded clippy-driven contract-module refactor**
   - Packet `4` validation surfaced that [`tool_invocation_contract.rs`](../crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs) still needed a bounded internal refactor so the Slice `52` adapter-contract module would satisfy `cargo clippy --workspace --all-targets -- -D warnings`.
   - Packet `4` resolution: keep the refactor local to the contract module, preserve the frozen request/receipt semantics, and record the closeout docs truthfully so the green validation wall is not described as docs-only when one bounded code follow-up was actually required.

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
