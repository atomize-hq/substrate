# Continue World Worker Blocking Deviation Debug

Date: 2026-07-05  
Workspace: `/home/spenser/__Active_code/substrate`

## Purpose

This document tracks a newly surfaced implementation deviation between the authoritative design stack and the current retained-world-worker behavior.

The specific seam is:

1. the design says retained world-worker continuation should be receipt-oriented and compatible with parked/resumable host orchestration,
2. the current implementation appears to hold `continue_world_worker` open as a blocking streamed call until the continued turn exits,
3. that collapses the intended handoff window where the host/orchestrator should be able to move on, park, and later resume through obligations, inbox, and reattach.

This document exists to keep that gap explicit while we investigate and patch it.

## Executive Summary

The current failure is not just that one cancel smoke returned `stale_linkage`.

The deeper issue is:

1. retained-worker design authority says long-lived or resumable work should return receipts and durable handles rather than forcing one blocking tool call,
2. retained workers are meant to remain routable authoritative orchestration state while the host/orchestrator may keep reasoning or go parked-resumable,
3. later worker events, obligations, and auto/manual attach paths are supposed to restore host engagement,
4. but current `continue_world_worker` implementation waits on the streamed member turn until terminal exit,
5. and clean exit then releases authoritative runtime ownership and transitions the worker back to `Ready`,
6. so same-turn follow-on control such as retained `cancel_world_work` misses the active authoritative-live window entirely.

That appears to be a real implementation deviation from the design, not merely a bad wording choice in the smoke prompt.

## Primary Design Authority

### 1. Agent-visible tool surface is supposed to be receipt-oriented, not one long blocking call

`llm-last-mile/DESIGN-host-orchestrator-tool-invocation-surface.md` freezes:

1. the host should be able to delegate world work, keep going, and later resume or inspect results,
2. long-lived or resumable work should return receipts and durable handles instead of forcing one blocking tool call,
3. the host may park and later re-engage through durable state rather than pretending the original tool call stayed open,
4. the tool model is meant to be receipt-oriented and resumable.

This is the clearest design authority for the seam.

### 2. Retained workers are meant to survive clean turn exit and remain resumable

`llm-last-mile/DESIGN-world-worker-lifecycle-model.md` freezes:

1. retained workers may transition `running -> parked`,
2. later `continue` against parked retained workers is part of the contract,
3. clean bootstrap exit after authoritative retained identity plus resumable session identity must not destroy retained continuity,
4. retained workers are distinct from ephemeral one-shot tasks.

### 3. Host orchestration is meant to re-engage through obligations, inbox, and attach

The following docs all reinforce that the host/orchestrator should not need one long blocking tool call to remain coherent:

1. `llm-last-mile/DESIGN-durable-orchestration-obligation-ledger.md`
2. `llm-last-mile/DESIGN-durable-orchestration-notification-inbox-contract.md`
3. `llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md`
4. `llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md`

Together they freeze that:

1. durable obligations are canonical downstream of worker/runtime events,
2. inbox/review is a projection over obligations,
3. auto-attach is a projection over obligations,
4. manual `reattach` remains canonical,
5. host-side re-engagement should occur through durable session truth rather than one continuously open tool invocation.

## Important Implementation Distinction

There is one design detail that must not be misread:

`llm-last-mile/DESIGN-internal-toolbox-transport-and-session-binding.md` says the internal transport stream is not complete until a terminal `result` frame is emitted.

That is a wire-level transport rule.

It is not itself authority that the agent-visible `continue_world_worker` tool must behave like one blocking synchronous call for long-lived retained work.

The higher-level design stack above the transport says the opposite:

1. retained work should be receipt-oriented,
2. the host may move on or park,
3. later engagement happens through durable handles and obligations.

So the likely deviation is not the raw transport framing rule. The likely deviation is how the current tool implementation is binding retained continuation semantics to that transport too literally.

## Current Implementation Truth

### 1. `continue_world_worker` currently awaits full streamed turn completion

Current source:

- [crates/shell/src/execution/orchestrator_world_dispatch.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs:1337)
- [crates/shell/src/execution/orchestrator_world_dispatch.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs:3397)

The current flow:

1. builds the retained member-turn submit request,
2. calls `execute_continue_world_worker_stream_for_turn_kind(...)`,
3. awaits the stream,
4. loops until a terminal `ExecuteStreamFrame::Exit` is received,
5. only then returns the `ContinueWorldWorkerOutcomeV1`.

That means the tool currently behaves like a blocking call from the caller's point of view.

### 2. Clean exit releases runtime ownership and returns the retained worker to `Ready`

Current source:

- [crates/shell/src/repl/async_repl.rs](/home/spenser/__Active_code/substrate/crates/shell/src/repl/async_repl.rs:7580)

On clean retained-member exit:

1. runtime ownership is released,
2. state transitions to `Ready`,
3. heartbeat is touched,
4. orchestration activity is refreshed.

That matches the parked/resumable flavor of the lifecycle, but it also means the authoritative-live execution window is gone by the time the blocking `continue_world_worker` call returns.

### 3. Retained cancel requires an active authoritative-live running target

Current source:

- [crates/shell/src/execution/agent_runtime/state_store.rs](/home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:2463)

Retained `cancel_world_work` currently requires:

1. exact session/world/backend/participant identity,
2. active authoritative orchestrator linkage,
3. target participant still `authoritative_live`,
4. owner process still live,
5. `cancel_supported=true`,
6. `latest_run_id` present,
7. target state still `Running`.

If `authoritative_live` has already been dropped, the current failure mode is:

- `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

## Live Evidence That Surfaced The Gap

Primary log artifact:

- `/tmp/cancel-retained-worker-one-turn.json`

What was attempted:

1. in one orchestration turn, tell the host orchestrator to issue `continue_world_worker`,
2. do not wait for the worker reply,
3. immediately issue `cancel_world_work` against the same retained worker,
4. return the raw cancel result.

Observed shape:

1. the orchestration agent claimed it was sending continue and then immediately canceling,
2. the runtime showed one monolithic `command_execution` span rather than two independently visible interleavable dispatches,
3. the final result was:
   - `stale_linkage: orchestration session ... retained worker ... is no longer authoritative-live`

That by itself did not initially prove design drift.

After re-reading the design authority, it now looks like the stronger issue is:

1. the smoke was trying to exercise a valid product-model behavior,
2. but the implementation likely does not currently expose the intended handoff point at all.

## Root Cause Hypothesis

Root cause hypothesis:

The retained `continue_world_worker` tool implementation has collapsed the higher-level parked/resumable receipt-oriented design into a blocking transport-shaped call, so the host/orchestrator cannot actually hand off long-lived retained work and regain control through durable receipts/obligations the way the design stack requires.

Stated more concretely:

1. the internal transport may legitimately stream until terminal result,
2. but the agent-visible retained-worker continuation seam should not require the caller to remain blocked on that stream for long-lived retained work,
3. current implementation appears to do exactly that,
4. which means same-turn follow-on control and broader parked/resumable orchestration semantics are both narrower than design authority.

## Why This Matters

If this hypothesis is correct, the impact is broader than one cancel repro.

It means the current implementation risks violating the design model for:

1. host/orchestrator parking while retained workers continue autonomously,
2. durable handle and receipt-oriented retained-worker control,
3. later inbox or obligation-driven re-engagement,
4. attach/reattach continuity semantics for long-lived retained work,
5. possible same-session follow-on tooling that assumes the host regains control before continued retained work is fully terminal.

## What This Is Not

This document is not claiming yet that:

1. `stop_world_worker` is broken again,
2. `cancel_world_work` itself is conceptually wrong,
3. the internal transport framing rule is wrong,
4. every retained-worker path is invalid.

The narrower claim is:

1. the design expects retained continuation to behave as a handoff-compatible, receipt-oriented control surface,
2. the current implementation appears to behave as a blocking streamed execution surface,
3. that is the seam to investigate and patch.

## Immediate Next Probes

The next narrow probes should focus on where the design/implementation divergence begins.

### Probe 1: accepted-handoff boundary

Determine whether there is any current code path where retained `continue_world_worker` publishes an authoritative receipt/accepted state before terminal stream completion.

### Probe 2: obligation publication timing

Determine whether long-lived retained worker events that should drive durable obligations are only observed after the full blocking stream returns, or whether they can already be durably materialized while the host/orchestrator is no longer foreground-active.

### Probe 3: adapter versus transport layering mistake

Determine whether the bug belongs in:

1. `continue_world_worker` adapter/tool semantics,
2. world-dispatch orchestration code,
3. retained member-turn stream classification/persistence,
4. or the runtime-owned attach/obligation bridge.

### Probe 4: regression test seam

Add a targeted test that encodes the intended model:

1. retained continuation returns a durable handle or accepted receipt without requiring full worker completion,
2. host/orchestrator can later inspect or react through durable state,
3. clean worker exit and obligation materialization do not require the original call to stay foreground-blocked.

## Current Status

Status: open debug seam  
Classification: design-authority implementation deviation  
Priority: high, because it changes the honesty of the retained world-worker model rather than just one smoke expectation
