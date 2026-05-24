# UAA Promptless Resume/Fork Synthesis For Substrate

Status: working synthesis for ongoing research, not an implementation spec  
Date: 2026-05-23 UTC  
Scope: consolidate the current substrate-side architectural context and the relevant Unified Agent API contract drift so future sessions can start from one document

Unified Agent API repo being evaluated in this synthesis:

- repo path: `/home/azureuser/__Active_Code/atomize-hq/unified-agent-api`
- branch: `feat/promptless-resume`
- checked commit during this synthesis: `3b7a4ef`

That branch contains the recently landed UAA changes this document is evaluating from the substrate
side. The goal here is to decide how substrate should move forward in response to that landed branch
state, not to assume that the current UAA branch behavior is automatically the correct long-term
contract.

## Purpose

This document captures the current understanding of whether Substrate actually needs the recently
landed Unified Agent API promptless resume/fork behavior.

The short answer is:

- promptless resume may be justified for Substrate's internal hidden owner-helper recovery path,
- promptless fork is not currently considered necessary,
- and blank prompt should not be the long-term contract mechanism for either control-only mode.

This is a research synthesis. It is not the final architecture decision and it is not a request to
implement anything yet.

## Current Recommendation

The current substrate-leaning recommendation is:

1. Do not normalize promptless fork as a durable Unified Agent API v1 contract.
2. Avoid relying on blank prompt as the long-term intent signal anywhere.
3. If a control-only resume path is still required by Substrate's durable parked-session model,
   prefer an explicit contract shape over implicit empty-prompt semantics.
4. Keep Substrate public semantics frozen:
   - `start` is prompt-taking root start,
   - `turn` is prompt-taking follow-up,
   - `reattach` is non-prompt attached-owner recovery,
   - `fork` is non-prompt successor durable-session allocation,
   - detached-world follow-up stays fail-closed until `reattach`.

## Executive Summary

The UAA change that landed on `feat/promptless-resume` makes
`agent_api.session.resume.v1` and `agent_api.session.fork.v1` accept a blank `prompt` and treat
that blank prompt as a real control action:

- promptless `resume` means "resume/reattach without sending a new user turn"
- promptless `fork` means "fork and surface a session handle without starting a turn"

That behavior is real in code, committed in the UAA repo, and currently covered by tests.
However, the normative UAA specs still describe prompt-bearing resume/fork semantics and still
describe prompt validation as non-empty after trimming.

Substrate does have a real architectural pressure around control-only recovery because the durable
authority is the Substrate orchestration session, not the attached Codex process. A clean
prompt-driven backend exit can still leave a valid parked-resumable session behind, and internal
Substrate recovery may need to reconnect to the backend-native session identity without sending a
new prompt.

But that pressure is much stronger for resume than for fork.

The current conclusion is:

- promptless resume has a plausible substrate use case if the current hidden owner-helper model
  stays,
- promptless fork does not currently appear necessary because substrate can require fork only when a
  real prompted follow-up is needed, and
- even for resume, blank prompt is a poor long-term contract shape compared to an explicit
  control-only mode.

## Relevant Substrate Truth

### Durable authority is the orchestration session

The key substrate architectural truth is that the durable authority is the Substrate-owned
orchestration session, not one currently attached backend process.

Primary truth anchors:

- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Important consequences:

- a clean prompt-driven backend exit must not automatically invalidate the durable host session
- `parked_resumable` is a valid session posture, not an owner-loss error
- `awaiting_attention` is also valid durable truth, not an implicit dead session
- world-originated work must survive even when no host client is currently attached

### Public semantics are intentionally narrow

Substrate's public lifecycle contract is already intentionally split between prompt-taking and
control-only actions.

Canonical public meaning:

- `substrate agent start --backend ... --prompt ...` is root prompt-taking start
- `substrate agent turn --session ... --backend ... --prompt ...` is prompt-taking follow-up on the
  same durable session
- `substrate agent reattach --session ...` is attached-owner recovery only and does not submit a
  prompt
- `substrate agent fork --session ...` is a control action that allocates a successor durable
  session and is not supposed to be reinterpreted as a prompt-taking action

Primary anchors:

- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [docs/USAGE.md](docs/USAGE.md)
- [PLAN.md](PLAN.md)
- [ORCH_PLAN.md](ORCH_PLAN.md)

This is important because UAA's original v1 semantics matched that separation better than the new
blank-prompt sentinel does.

## Where Substrate Currently Depends On UAA Session Identity

Substrate persists the surfaced backend-native session identity as internal state and intentionally
keeps it separate from the public orchestration session id.

Key places:

- [crates/shell/src/execution/agent_runtime/session.rs](crates/shell/src/execution/agent_runtime/session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
- [crates/world-service/src/member_runtime.rs](crates/world-service/src/member_runtime.rs)

Important internal rule:

- `internal.uaa_session_id` is used internally for exact resume/fork against the backend-native
  session
- `internal.uaa_session_id` is never a public selector
- public control and public turn reject selectors that match `internal.uaa_session_id`

Relevant checks:

- [state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)

## What The Current Substrate Runtime Actually Does

### Prompt-taking follow-up already uses prompt-bearing resume

Normal public follow-up `turn` is prompt-bearing and already fits the original UAA contract.

Host path:

- [submit_host_prompt_turn(...)](crates/shell/src/execution/agent_runtime/control.rs)

World-member follow-up path:

- [submit_turn(...)](crates/world-service/src/member_runtime.rs)
- [MemberTurnSubmitRequestV1 transport use](crates/shell/src/repl/async_repl.rs)

Both of these pass a real prompt and use `agent_api.session.resume.v1` with the persisted internal
UAA session id. That part does not need promptless behavior.

### Hidden owner-helper recovery currently creates the pressure

The real architectural pressure comes from the hidden owner-helper retained-owner path.

Important flow:

- owner-helper modes include `Resume`, `ResumeOneTurn`, and `Fork`
- those modes currently require `internal.uaa_session_id`
- startup can choose `InitialExecPromptPlan::NoPromptRecovery`
- `NoPromptRecovery` becomes `prompt: ""` when creating the UAA run request

Relevant files:

- [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)

This is the strongest substrate-side argument for a control-only resume capability in some form.

## Important Live-Code Mismatch Inside Substrate

There is still a mismatch between older substrate planning docs and current runtime shaping.

The planning intent in older lifecycle/control slices was:

- public `resume` should launch with `agent_api.session.resume.v1`
- public `fork` should launch with `agent_api.session.fork.v1`

Good examples:

- [PLAN-19.md](llm-last-mile/PLAN-19.md)
- [19-public-agent-control-surfaces.md](llm-last-mile/19-public-agent-control-surfaces.md)

But the current hidden owner-helper request shaping still routes:

- `OwnerHelperMode::Resume`
- `OwnerHelperMode::ResumeOneTurn`
- `OwnerHelperMode::Fork`

through the same resume-extension shaping path in:

- [async_repl.rs](crates/shell/src/repl/async_repl.rs)

That means:

- substrate does not yet prove, in live code, that promptless fork is inherently required
- some of the perceived pressure for promptless fork comes from unfinished convergence between plan
  and implementation

## Relevant Unified Agent API Change

### What changed

In the local UAA checkout at `/home/azureuser/__Active_Code/atomize-hq/unified-agent-api`, the
branch `feat/promptless-resume` now allows blank prompts for:

- `agent_api.session.resume.v1`
- `agent_api.session.fork.v1`

Key implementation files:

- [../unified-agent-api/crates/agent_api/src/backend_harness/normalize.rs](../unified-agent-api/crates/agent_api/src/backend_harness/normalize.rs)
- [../unified-agent-api/crates/agent_api/src/backends/session_selectors.rs](../unified-agent-api/crates/agent_api/src/backends/session_selectors.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/exec.rs](../unified-agent-api/crates/agent_api/src/backends/codex/exec.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/fork.rs](../unified-agent-api/crates/agent_api/src/backends/codex/fork.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/harness.rs](../unified-agent-api/crates/agent_api/src/backends/codex/harness.rs)
- [../unified-agent-api/crates/agent_api/src/backends/claude_code/util.rs](../unified-agent-api/crates/agent_api/src/backends/claude_code/util.rs)

Behavioral result:

- promptless resume becomes a real control-only resume
- promptless fork becomes a real control-only fork
- Codex promptless fork still performs `thread/fork` and returns a handle, but skips `turn/start`

### Why this is contract drift

The normative UAA docs still say prompt is non-empty and still describe resume/fork as
prompt-bearing operations.

Primary spec anchors:

- [../unified-agent-api/docs/specs/unified-agent-api/run-protocol-spec.md](../unified-agent-api/docs/specs/unified-agent-api/run-protocol-spec.md)
- [../unified-agent-api/docs/specs/unified-agent-api/extensions-spec.md](../unified-agent-api/docs/specs/unified-agent-api/extensions-spec.md)
- [../unified-agent-api/docs/specs/claude-code-session-mapping-contract.md](../unified-agent-api/docs/specs/claude-code-session-mapping-contract.md)
- [../unified-agent-api/docs/specs/codex-wrapper-coverage-scenarios-v1.md](../unified-agent-api/docs/specs/codex-wrapper-coverage-scenarios-v1.md)
- [../unified-agent-api/docs/specs/codex-app-server-jsonrpc-contract.md](../unified-agent-api/docs/specs/codex-app-server-jsonrpc-contract.md)

The main contract mismatch is:

- spec says prompt must be non-empty
- implementation says prompt may be blank if resume/fork is present

## Why Promptless Resume Still Has A Real Case

Given ADR-0047 and follow-on slices 23/24/25, Substrate may still need an internal way to:

- reconnect a hidden owner-helper to the persisted backend-native session
- re-establish attached ownership or resumable continuity
- surface the handle and session continuity
- do all of that without sending a new user turn

That is not the same as public `turn`.
That is much closer to the public meaning of `reattach`.

If Substrate keeps the current hidden owner-helper architecture, a control-only resume path appears
architecturally legitimate.

What is not yet settled is where that control-only resume should live:

- inside UAA v1 via empty prompt sentinel
- inside a new explicit UAA surface
- or inside a more substrate-private control layer above prompt-bearing UAA runs

## Why Promptless Fork Is Not Currently Recommended

The current substrate position is:

- promptless fork is not needed
- fork should only happen when there is a real prompted follow-up need
- the wrapped backend is `codex exec` style, not a persistent TUI model

That position fits the current evidence well.

Reasons promptless fork is not recommended:

1. Substrate's real architectural pressure is about reattachment and parked-session recovery, which
   is resume-shaped, not fork-shaped.
2. The current live substrate runtime does not even cleanly use `session.fork.v1` in the
   owner-helper path yet; it still routes `Fork` mode through resume-extension shaping.
3. UAA promptless fork is a larger semantic shift than promptless resume because it turns a
   prompt-bearing "fork and send follow-up turn" API into a "fork only" control API.
4. Codex promptless fork required new synthetic-status behavior just to preserve session-handle
   event surfacing without any turn output, which is further evidence that this is a new operation,
   not a small validation tweak.

If Substrate only needs fork when it is actually going to send a prompt, then the original UAA fork
contract is already closer to the right abstraction.

## Recommended Direction

### Direction A: the "do it right" path

This is the preferred direction.

- Do not keep blank prompt as the durable intent signal.
- Do not adopt promptless fork as stable UAA v1 semantics.
- If Substrate still needs control-only resume, introduce an explicit contract for it rather than
  reusing "empty prompt".

Reason:

- explicit intent preserves the old invariant that empty prompt normally means caller error
- explicit intent avoids ambiguity around whitespace-only prompts
- explicit intent is easier to document, publish, test, and reason about across backends

### Direction B: acceptable temporary unblocker

Only if needed for near-term substrate progress:

- tolerate promptless `session.resume.v1` temporarily
- treat it as implementation-available but contractually unsettled
- do not extend that tolerance to promptless fork unless substrate's architecture genuinely demands
  fork-without-turn later

## Architectural Options To Explore Next

These are the likely viable paths.

### Option 1: explicit UAA control-only resume surface

Examples:

- `agent_api.session.resume.v2` with an explicit `start_turn: false`
- a sibling extension key that means "control-only session attach/resume"

Pros:

- keeps intent explicit
- preserves v1 prompt-bearing semantics
- aligns well with substrate `reattach`

Cons:

- requires UAA contract work and doc publication

### Option 2: explicit UAA control-only fork surface

This is only worth considering if Substrate later decides it truly needs `fork` without an
immediate prompt.

At the moment this is not recommended.

### Option 3: keep control-only behavior in Substrate/gateway-private orchestration logic

This would keep UAA focused on prompt-bearing run semantics and push attach/fork-only continuity
into a more control-plane-specific substrate layer.

Pros:

- preserves UAA conceptual cleanliness
- fits the idea that parked-session continuity is substrate-owned

Cons:

- may require more substrate-specific glue
- may duplicate some backend-session handling already present in UAA

## Important Document Pointers

### Substrate truth anchors

- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)
- [PLAN.md](PLAN.md)
- [ORCH_PLAN.md](ORCH_PLAN.md)

### Substrate runtime files

- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/execution/prompt_fulfillment.rs](crates/shell/src/execution/prompt_fulfillment.rs)
- [crates/world-service/src/member_runtime.rs](crates/world-service/src/member_runtime.rs)
- [crates/world-service/src/prompt_fulfillment.rs](crates/world-service/src/prompt_fulfillment.rs)
- [docs/USAGE.md](docs/USAGE.md)

### UAA normative docs

Local UAA checkout under review:

- repo: `/home/azureuser/__Active_Code/atomize-hq/unified-agent-api`
- branch: `feat/promptless-resume`
- commit observed during synthesis: `3b7a4ef`

Why this branch matters:

- it contains the landed promptless resume/fork behavior substrate is evaluating
- it is the concrete implementation state future sessions should inspect when checking code, tests,
  and spec drift
- this synthesis treats that branch as the current implementation truth while leaving the
  long-term contract decision open

- [../unified-agent-api/docs/specs/unified-agent-api/run-protocol-spec.md](../unified-agent-api/docs/specs/unified-agent-api/run-protocol-spec.md)
- [../unified-agent-api/docs/specs/unified-agent-api/extensions-spec.md](../unified-agent-api/docs/specs/unified-agent-api/extensions-spec.md)
- [../unified-agent-api/docs/specs/claude-code-session-mapping-contract.md](../unified-agent-api/docs/specs/claude-code-session-mapping-contract.md)
- [../unified-agent-api/docs/specs/codex-wrapper-coverage-scenarios-v1.md](../unified-agent-api/docs/specs/codex-wrapper-coverage-scenarios-v1.md)
- [../unified-agent-api/docs/specs/codex-app-server-jsonrpc-contract.md](../unified-agent-api/docs/specs/codex-app-server-jsonrpc-contract.md)
- [../unified-agent-api/docs/specs/unified-agent-api/capability-matrix.md](../unified-agent-api/docs/specs/unified-agent-api/capability-matrix.md)

### UAA implementation files

Local checkout under review:

- repo: `/home/azureuser/__Active_Code/atomize-hq/unified-agent-api`
- branch: `feat/promptless-resume`
- commit observed during synthesis: `3b7a4ef`

These files are the most relevant implementation surfaces in that branch:

- [../unified-agent-api/crates/agent_api/src/backend_harness/normalize.rs](../unified-agent-api/crates/agent_api/src/backend_harness/normalize.rs)
- [../unified-agent-api/crates/agent_api/src/backends/session_selectors.rs](../unified-agent-api/crates/agent_api/src/backends/session_selectors.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/exec.rs](../unified-agent-api/crates/agent_api/src/backends/codex/exec.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/fork.rs](../unified-agent-api/crates/agent_api/src/backends/codex/fork.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/harness.rs](../unified-agent-api/crates/agent_api/src/backends/codex/harness.rs)
- [../unified-agent-api/crates/agent_api/src/backends/claude_code/util.rs](../unified-agent-api/crates/agent_api/src/backends/claude_code/util.rs)

## Questions To Keep Answering

1. Does Substrate still need any control-only session behavior once the hidden owner-helper design
   is fully reconciled with the durable parked-session model?
2. If yes, is control-only resume the only real requirement?
3. Can control-only resume live in a substrate-private orchestration seam instead of changing UAA
   run semantics?
4. If UAA must own it, what is the cleanest explicit contract shape that preserves non-empty prompt
   as the default invariant?
5. Is there any remaining substrate path that truly requires fork-without-turn after the current
   prompt-bearing `turn` and non-prompt `reattach` split is respected?

## Bottom Line

Based on the current substrate truth and the current UAA implementation:

- promptless resume has a plausible internal substrate use case
- promptless fork is not currently needed
- blank prompt is still the wrong long-term contract mechanism

If this work continues in future sessions, start here, then follow the document pointers above only
for the specific open question being worked next.
