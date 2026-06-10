# PLAN-53: Inventory-Selected First Runtime-Family Host-Orchestrator Tool Surface Landing

Source spec: [SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md)  
Source tracker note: [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)  
Prior slice: [PLAN-52.md](./PLAN-52.md)  
Plan type: first live runtime-family landing above the frozen Slice `52` adapter contract  
Status: landed runtime truth reviewed and Packet 4 closeout verified on `2026-06-10`
Validation note: Packet 4 finished by locking the operator/reporting truth surfaces around the first validated Codex-backed floor, adding focused non-Codex posture coverage, updating the tracker so remaining work stays bounded to later parity/smoke follow-through rather than reopening selection, transport, or Codex-first semantics, and absorbing one bounded async-repl validation closeout in [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs) so the packet-wide validation wall reruns green.

## Objective

Land the first real host-session tool surface while preserving the existing dynamic orchestrator selection model.

This slice is complete only when:

1. selected host orchestrators still come from effective config plus effective inventory,
2. exact backend/policy selection remains unchanged,
3. the Codex-backed path receives toolbox endpoint/version env injection plus startup/system-prompt tool-contract disclosure as the first real smoke/validation floor,
4. tool calls bridge through the frozen Slice `52` contract into the landed internal toolbox transport,
5. non-Codex runtimes keep existing session behavior and reporting reflects truthful validation/support posture instead of doc-driven blocking,
6. the slice lands without public toolbox verbs, MCP-first redesign, or `claude_code` parity.

## Phase Gate

This plan assumes the `SPECIFY` artifact in [SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md](./SPEC-53-inventory-selected-first-runtime-family-host-orchestrator-tool-surface-landing.md) has been reviewed and accepted before implementation starts.

## Tracker Update Rule

The canonical running ledger for new drift, deferrals, and circle-back items in this seam remains:

- [REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md](./REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md)

During Packet `1`-`4` execution or review:

1. add new drift or sequencing discoveries to `## Newly Surfaced During Execution`,
2. move intentional non-fixes into `## Deferred / Circle-Back Items`,
3. move reconciled items into `## Resolved Since Last Update`,
4. update `## Recommended Slice Order` if this slice changes the remaining order.

## Repo-Truth Framing

What is already landed:

1. dynamic orchestrator selection from effective config and inventory,
2. dynamic inventory discovery from `$SUBSTRATE_HOME/agents` plus workspace `.substrate/agents`,
3. exact backend allowlisting and runtime-family realization from inventory truth,
4. the frozen seven-tool Slice `52` contract,
5. the landed internal toolbox transport and world-dispatch runtime.

What is still missing:

1. a live runtime-family-specific way for a host session to actually see/call those seven tools,
2. one explicit validation/support-posture model for runtimes beyond the first proven Codex-backed path,
3. one first-family end-to-end proof that tool calls preserve Slice `52` semantics.

## Locked Decisions

### What this slice changes

1. It lands the first live host-session tool surface with the Codex-backed path as the first real smoke/validation floor.
2. It keeps orchestrator selection dynamic and inventory-driven.
3. It makes validation/support posture explicit instead of silently falling back or overclaiming parity.
4. It reuses the Slice `52` contract without reopening it.

### What this slice does not change

1. no hard-coded `codex` orchestrator id,
2. no `claude_code` tool-surface parity yet,
3. no public human toolbox execution CLI,
4. no MCP-first transport redesign,
5. no Family-2 inbox/router reopening,
6. no silent runtime-family fallback.

## Major Components And Dependencies

1. **dynamic selection and validation/support posture**
   - `crates/shell/src/execution/agent_inventory.rs`
   - `crates/shell/src/execution/agent_runtime/validator.rs`
   - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - keeps existing selection truth intact and adds explicit validated/not-yet-validated posture by resolved runtime family

2. **family-specific host prompt/env injection seam**
   - `crates/shell/src/execution/agent_runtime/control.rs` (`submit_host_prompt_turn(...)` / `AgentWrapperRunRequest.env`)
   - `crates/shell/src/execution/prompt_fulfillment.rs`
   - `crates/shell/src/execution/agents_cmd.rs` (`HiddenOwnerHelperStartupPromptPlan.prompt_text`)
   - likely the narrowest first-family boundary for runtime-owned toolbox endpoint/version env injection plus startup/system-prompt tool-contract composition, with bounded Codex-specific shaping layered on top only if needed

3. **frozen tool invocation semantics**
   - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
   - remains the authority for tool names, runtime-owned injection, and receipt/result normalization

4. **internal toolbox transport/runtime**
   - `crates/shell/src/repl/async_repl.rs`
   - already landed; Slice `53` must consume it rather than redesign it

5. **operator-truth surfaces**
   - `crates/shell/src/execution/agents_cmd.rs`
   - bounded reporting for supported vs unsupported runtime-family tool exposure

6. **dependency seam (only if needed)**
   - `crates/gateway/src/adapter_runtime.rs`
   - pinned `unified-agent-api` surface via `crates/shell/Cargo.toml`
   - only touch this if the first validated Codex-backed landing cannot be achieved through an existing bounded path

## Plan Summary

The first live landing should not change **which** orchestrator is selected. It should change only whether the selected orchestrator's runtime family can receive the frozen seven-tool surface.

The narrowest honest Slice `53` is:

1. preserve dynamic selection truth first,
2. add explicit tool-surface validation/support posture second,
3. land one Codex-backed runtime-owned env-injected plus prompt-augmented first validated tool exposure path third,
4. wire tool calls/results through Slice `52` semantics fourth,
5. finish with tests and truthful reporting last.

## Implementation Order

### Packet 1: Preserve Dynamic Selection And Add Explicit Tool-Surface Validation/Support Truth

Goal:

1. confirm the live tool surface keys off resolved runtime family rather than hard-coded orchestrator ids,
2. add one explicit posture model where the Codex-backed path is first smoke-validated and other runtimes are reported truthfully as not-yet-validated / not-yet-guaranteed where that is the truth,
3. keep ordinary host-session behavior unchanged for non-Codex runtimes unless implementation evidence proves a real incompatibility.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/validator.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/agents_cmd.rs` only where truthful reporting is needed

Why first:

1. it prevents the codex-first landing from quietly becoming codex-hard-coded selection,
2. it defines truthful validation/support posture before family-specific injection work starts,
3. later packets should consume this posture truth instead of inventing ad hoc gates.

Verification checkpoint:

1. selection remains `agents.hub.orchestrator_agent_id` + effective inventory + exact backend policy,
2. resolved runtime family is the only new family-specific input to validation/reporting posture,
3. non-Codex runtimes keep existing host session behavior unless code proves a real incompatibility.

### Packet 2: Land The First Validated Codex-Backed Toolbox-Env Plus Prompt-Contract Injection Path

Goal:

1. make the Codex-backed path actually see the seven frozen tools in the first validated landing,
2. inject the authoritative toolbox endpoint/version into that live host runtime at the prompt-submission boundary,
3. inject the seven-tool contract details into the startup/system prompt because the first landing is not MCP-server-based tool registration,
4. keep the semantic tool contract family-independent even if Codex still needs bounded config shaping on top of that env injection,
5. avoid turning “Codex is first smoke floor” into a hard runtime gate if generic adapter behavior is already honest.

Primary touch surface:

1. `crates/shell/src/execution/prompt_fulfillment.rs`
2. `crates/shell/src/execution/agent_runtime/control.rs` (`submit_host_prompt_turn(...)` env injection seam)
3. `crates/shell/src/execution/agents_cmd.rs` (`HiddenOwnerHelperStartupPromptPlan.prompt_text` composition seam)
4. `crates/gateway/src/adapter_runtime.rs` and/or bounded UAA dependency bump surfaces only if required

Why second:

1. this is the first genuinely new runtime-family behavior,
2. Packet 1 should already have frozen when the new path is allowed to exist,
3. dependency widening, if any, should stay bounded to the first-family env/config injection seam.

Verification checkpoint:

1. the Codex-backed path receives a real tool surface through runtime-owned toolbox env injection plus startup/system-prompt tool-contract disclosure rather than fake direct tool-list injection,
2. no hard-coded orchestrator id is required,
3. the implementation does not introduce a documentation-only hard block for other selected runtimes.

### Packet 3: Bridge Live Tool Calls Through Slice 52 Contract Into Internal Toolbox Runtime

Goal:

1. route live tool calls from the first validated Codex-backed path through `tool_invocation_contract.rs`,
2. preserve runtime-owned request/session/caller/world injection,
3. preserve receipt/result semantics for active-task and retained-worker flows.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
2. `crates/shell/src/execution/prompt_fulfillment.rs`
3. `crates/shell/src/repl/async_repl.rs` only where existing transport hookup is required

Why third:

1. Packet 2 must first make the tool surface exist,
2. the live call loop should reuse the frozen Slice `52` contract instead of inventing family-specific result shapes,
3. this is the packet that proves the first end-to-end host delegation path.

Verification checkpoint:

1. a live codex host tool call reaches internal world dispatch,
2. runtime-owned fields are injected by the shell rather than supplied by the model,
3. returned results preserve frozen receipt semantics.

### Packet 4: Validation Wall, Truthful Reporting, And Tracker Closeout

Goal:

1. lock the validation wall for the first-family landing,
2. make reporting surfaces truthful about first-validated vs not-yet-validated runtimes,
3. record any bounded deferrals for later Slice `54+` work.

Primary touch surface:

1. `crates/shell/src/execution/agents_cmd.rs`
2. focused shell tests under `crates/shell/tests/`
3. `llm-last-mile/REMAINING-host-orchestrator-tool-invocation-surface-2026-06-08.md`

Why last:

1. truth surfaces should describe what actually landed,
2. Packet 4 is where any dependency or family-parity deferrals can be written down cleanly,
3. it keeps the first three packets focused on runtime truth.

Verification checkpoint:

1. targeted tests are green,
2. `doctor` / `toolbox status` do not overclaim validation/support posture beyond what is actually proven,
3. remaining-scope notes truthfully capture what Slice `53` did and did not land.

## Risks And Mitigations

1. **Risk: codex-first implementation accidentally hard-codes orchestrator identity**
   - Mitigation: gate by resolved runtime family only and keep explicit tests with alias backends / non-`codex` agent ids.
2. **Risk: current dependency surface may still need bounded widening even after env injection is chosen**
   - Mitigation: keep any dependency widening explicit, bounded, and isolated to the family-specific submission/env seam; prefer existing runtime env injection plus prompt augmentation plus existing UAA Codex config surfaces first.
3. **Risk: “Codex-first validation” drifts into “Codex-only eligibility”**
   - Mitigation: separate validated smoke coverage from runtime allowance, keep non-Codex behavior truthful rather than implicitly blocked, and require explicit evidence before adding any hard runtime disablement.
4. **Risk: Slice 53 reopens Slice 52 semantics**
   - Mitigation: route all live tool calls through `tool_invocation_contract.rs` and reject any family-specific receipt drift.

## Parallelism Guidance

Mostly sequential:

1. Packet `1` must land before Packet `2`.
2. Packet `2` must establish the injection seam before Packet `3` can wire tool calls through it.
3. Packet `4` can begin once Packet `3` is stable.

Possible bounded parallelism:

1. test-fixture prep for non-Codex validation/support-posture truth can happen while Packet `2` is being implemented,
2. tracker/doc updates can be drafted in parallel with final validation, but only merged after runtime truth is known.
