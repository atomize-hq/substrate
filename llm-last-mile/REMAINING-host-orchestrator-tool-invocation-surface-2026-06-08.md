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

Slice `52` plus Slice `53` now cover the first three slots in the original sequence:

1. the shared runtime-owned adapter contract freeze,
2. the first Codex-backed live runtime-family landing,
3. the initial receipt/resume hardening plus Packet `4` operator/tracker truth closeout.

The remaining honest order is therefore:

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

### Slice E: Broader smoke and docs follow-through after parity or surface widening

Objective:

1. align runtime docs with any later multi-family live surface that actually lands,
2. capture bounded smoke expectations beyond the current first-family validation wall,
3. keep queued MCP language clearly separate from the landed Substrate-native adapter path.

Why last:

1. Slice `53` already closed the first-family operator/tracker truth surfaces,
2. any broader doc/smoke pass should wait until later parity or surface widening changes real repo truth again.

## Parity Strategy Recommendation

The safer default is:

1. keep the landed shared semantic contract and Codex-backed first validated floor as the baseline,
2. land `claude_code` parity next only when the live surface can be proven without overclaiming support,
3. run any broader docs/smoke refresh only after that later parity truth exists.

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

### 2026-06-10 — Slice 53 Packet 4 closeout deferrals

1. **Second runtime-family live host-tool parity remains deferred**
   - Slice `53` closed the first-family validation wall and truthful operator/reporting surfaces, but it did not land a smoke-validated `claude_code` host-tool surface.
   - Follow-up slice family: later Slice `54+` parity work should consume the already-landed contract, Codex-backed floor, and Packet `4` reporting truth without reopening selection or transport semantics.
2. **Broader multi-family smoke/doc refresh remains deferred until parity truth changes**
   - Packet `4` already aligned the first-family truth surfaces in repo-local slice artifacts and operator outputs.
   - Follow-up slice family: only reopen a broader docs/smoke pass after later runtime-family parity or other live surface widening materially changes repo truth.

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

### 2026-06-10 — Slice 53 Packet 4 closeout

1. **Resolved operator-truth gap: first-family validation/support posture is now explicit on doctor/toolbox surfaces**
   - Packet `4` closed the remaining reporting gap by keeping exact selected backend ids visible while publishing the landed live-tool posture on `substrate agent doctor --json` and `substrate agent toolbox status --json`.
   - The first validated Codex-backed floor remains explicit, and selected `claude_code` orchestrators are reported as `not_yet_smoke_validated` / `not_yet_guaranteed` instead of being silently treated as parity or blocked by documentation wording alone.
2. **Resolved non-Codex coverage gap: selected claude host launch truth is now tested without claiming live parity**
   - Packet `4` added focused coverage in [`agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs) and [`agent_successor_contract_ahcsitc0.rs`](../crates/shell/tests/agent_successor_contract_ahcsitc0.rs) that the selected non-Codex host path stays on the exact `cli:claude_code` runtime-family truth and does not stage the first validated Codex-only toolbox contract.
   - Resolution stays bounded to truthful posture/launch-plan/operator coverage only; Slice `53` still does not claim a smoke-validated `claude_code` live host-tool surface.
3. **Resolved slice-artifact drift: Slice 53 repo-local spec/plan/tasks now describe the landed Packet 4 closeout truth**
   - Packet `4` updated the Slice `53` repo-local artifacts so they stop reading as proposed-only work after the operator/reporting closeout and validation wall landed.
   - The remaining ledger now points forward to later runtime-family parity and broader follow-through rather than restating already-landed first-family work as still pending.
4. **Resolved validation follow-up: Packet 4 needed one bounded async-repl closeout**
   - Packet `4` reran the required workspace validation wall and surfaced two local issues in [`async_repl.rs`](../crates/shell/src/repl/async_repl.rs): one `clippy -D warnings` needless-borrow fix at the internal toolbox transport registration seam, and a small expectation drift set in toolbox-routing tests whose assertions no longer matched the frozen live contract/routing truth.
   - Packet `4` resolution: keep the follow-up inside `async_repl.rs`, applying the one-token clippy fix plus test expectation realignment only, so the green validation wall remains truthful without widening Slice `53` beyond operator/reporting/tracker closeout scope.

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

1. the internal toolbox transport, shared adapter contract, first Codex-backed live landing, and Packet `4` operator/tracker truth surfaces are now landed,
2. the Codex-backed path is the first smoke-validated host-tool floor while selected non-Codex host paths remain truthfully reported as not-yet-validated / not-yet-guaranteed rather than silently forced through a Codex-only gate,
3. the main remaining execution seam is second runtime-family parity for `claude_code`,
4. any broader docs/smoke refresh should wait until that later parity or other live surface widening changes repo truth again.
