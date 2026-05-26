# UAA Promptless Resume/Fork Synthesis For Substrate

Status: post-28.5 runtime synthesis, not a normative UAA spec
Date: 2026-05-24 UTC
Scope: record what the current Substrate runtime actually does after the control-only attach and honest successor-allocation split landed

Unified Agent API repo referenced in this synthesis:

- repo path: `/home/azureuser/__Active_Code/atomize-hq/unified-agent-api`
- branch observed during synthesis: `feat/promptless-resume`
- previously inspected commit during this line of work: `3b7a4ef`

That UAA branch still matters because it contains promptless resume/fork behavior Substrate
evaluated during this execution slice. But the current Substrate runtime no longer depends on blank
prompt semantics to satisfy its public `reattach` and `fork` contracts.

## Purpose

This document captures the current answer to a narrower question than earlier drafts:

Does the landed Substrate runtime still need Unified Agent API promptless resume or promptless fork
as part of its live architecture?

Current answer:

- not for public `reattach`,
- not for public `fork`,
- and not as the architectural meaning of hidden owner-helper attach either.

Future UAA contract work may still choose to publish an explicit control-only resume surface if
that proves broadly useful. But blank prompt is no longer the mechanism Substrate needs in order to
honor its current durable-session model.

## Current Recommendation

1. Treat promptless UAA resume/fork as implementation behavior under evaluation, not as required
   Substrate architecture.
2. Keep Substrate public semantics frozen:
   - `start` is prompt-taking root start,
   - `turn` is prompt-taking follow-up,
   - `reattach` is control-only attached-owner recovery for the same durable session,
   - `fork` is control-only successor durable-session allocation,
   - detached-world follow-up stays fail-closed until host ownership returns.
3. Do not revive `prompt: ""` or `InitialExecPromptPlan::NoPromptRecovery` as the meaning of
   public `reattach` or `fork`.
4. If UAA eventually needs a published control-only contract, prefer an explicit surface over
   empty-prompt signaling.

## Executive Summary

Earlier drafts in this area were driven by a real mismatch:

- Substrate needed control-only host reattachment and honest successor allocation,
- the old hidden owner-helper convergence reused prompt-bearing run shapes,
- and the local UAA branch had landed promptless resume/fork behavior.

That mismatch is no longer the live runtime story.

The current Substrate runtime now separates the three concerns explicitly:

1. prompt-bearing launch and prompt-bearing resumed turn submission still go through prompt-bearing
   run control;
2. control-only host attach uses a dedicated attach path that consumes persisted continuity without
   inventing a blank prompt;
3. public `fork` allocates durable successor state directly inside Substrate and does not launch a
   backend process at allocation time.

As a result:

- promptless UAA resume is no longer required to implement current public `reattach`,
- promptless UAA fork is not required for current public `fork`,
- and the strongest remaining argument for a future explicit UAA control-only resume surface is
  architectural cleanliness, not an immediate Substrate blocker.

## Relevant Substrate Truth

### Durable authority is the orchestration session

The durable authority is the Substrate-owned orchestration session, not one currently attached
backend process.

Primary truth anchors:

- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [ADR-0047](docs/project_management/adrs/draft/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md)
- [24-fix-host-bootstrap-readiness-and-clean-detach-parking.md](llm-last-mile/24-fix-host-bootstrap-readiness-and-clean-detach-parking.md)
- [25-host-durable-session-closeout-and-qa-hardening.md](llm-last-mile/25-host-durable-session-closeout-and-qa-hardening.md)

Important consequences:

- a clean prompt-driven backend exit must not automatically invalidate the durable host session;
- `parked_resumable` is valid session truth, not an owner-loss error;
- world-originated work may outlive the foreground host attachment;
- later control actions must reconstruct exact launch truth from durable state rather than from
  whichever participant happened to be most recent.

### Public semantics remain intentionally narrow

Canonical public meaning:

- `substrate agent start --backend ... --prompt ...` is root prompt-taking start;
- `substrate agent turn --session ... --backend ... --prompt ...` is prompt-taking follow-up on
  the same durable session;
- `substrate agent reattach --session ...` is attached-owner recovery only and does not submit a
  prompt;
- `substrate agent fork --session ...` allocates a successor durable session and returns it parked,
  unattached, and truthfully non-active;
- detached-world follow-up stays fail-closed until host ownership is attached again.

Primary anchors:

- [HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md](HOST_ORCHESTRATOR_INTENDED_BEHAVIOR_TRUTH.md)
- [docs/USAGE.md](docs/USAGE.md)
- [PLAN.md](PLAN.md)
- [ORCH_PLAN.md](ORCH_PLAN.md)

## Where Substrate Persists Attach Truth

Substrate now persists attach-relevant truth under the orchestration session itself.

Key places:

- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)
- [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)

Important internal rules:

- `host_attach_contract` is the durable host attach truth carried by the orchestration session;
- `host_attach_contract.continuity_uaa_session_id` is private continuity state, not a public
  selector;
- successor allocation copies attach-contract shape but clears inherited
  `continuity_uaa_session_id`;
- public selectors still reject `internal.uaa_session_id`.

## What The Current Substrate Runtime Actually Does

### Prompt-bearing follow-up still uses prompt-bearing resume

Normal public follow-up `turn` remains prompt-bearing and still fits ordinary resume semantics.

Host path:

- [submit_host_prompt_turn(...)](crates/shell/src/execution/agent_runtime/control.rs)

World-member follow-up path:

- [submit_turn(...)](crates/world-service/src/member_runtime.rs)
- [MemberTurnSubmitRequestV1 transport use](crates/shell/src/repl/async_repl.rs)

Those paths carry a real prompt. They are not the problem this synthesis is about.

### Control-only attach is now a dedicated runtime path

The hidden owner-helper retained-owner flow now has an explicit attach mode instead of reusing
blank-prompt run shaping.

Important flow:

- attach planning uses the persisted `host_attach_contract` from the orchestration session;
- `build_attach_launch_plan(...)` carries forward the exact durable session id plus private
  continuity selector when present;
- `start_host_orchestrator_runtime_with_prepared_prompt(...)` calls
  `PromptFulfillmentBridge::run_attach_control(...)` when the startup mode is control-only attach;
- `run_attach_control(...)` maps backend-native attach/resume events into the existing control
  stream and synthesizes canonical session-handle facets so readiness/persistence code can observe
  attachment honestly.

Primary anchors:

- [build_attach_launch_plan(...)](crates/shell/src/execution/agents_cmd.rs)
- [start_host_orchestrator_runtime_with_prepared_prompt(...)](crates/shell/src/repl/async_repl.rs)
- [PromptFulfillmentBridge::run_attach_control(...)](crates/shell/src/execution/prompt_fulfillment.rs)

This means current control-only reattach is implemented as explicit attach behavior, not as
`prompt: ""`.

### Public `fork` is now durable successor allocation, not backend launch

Public `fork` no longer needs promptless backend fork semantics in order to preserve honest
Substrate truth.

Current behavior:

- `allocate_fork_successor(...)` allocates the successor orchestration session and successor
  participant directly in Substrate;
- the successor copies attach-contract shape from the source session;
- the successor clears inherited `continuity_uaa_session_id`;
- the successor is persisted as `parked_resumable` with `attached_participant_id = null`;
- public `run_fork(...)` returns the parked successor immediately and does not attach a live owner
  loop.

Primary anchors:

- [allocate_fork_successor(...)](crates/shell/src/execution/agents_cmd.rs)
- [fork_successor_attach_contract(...)](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [public_reattach_and_fork_preserve_exact_session_and_lineage_contracts](crates/shell/tests/agent_public_control_surface_v1.rs)

This is a durable-state allocation action, not a hidden prompted run and not a backend-native
control-only fork.

## What No Longer Matches Earlier Drafts

The following older substrate-side assumptions are no longer live architecture:

- `InitialExecPromptPlan::NoPromptRecovery` as the meaning of public `reattach` or `fork`;
- blank prompt being converted into UAA run requests to recover attached ownership;
- public `fork` needing `agent_api.session.resume.v1` or `agent_api.session.fork.v1` in order to
  allocate a truthful successor session;
- hidden owner-helper `Fork` as a shared convergence mode that proved promptless fork pressure.

The current runtime has replaced that convergence with:

- dedicated control-only attach,
- prompt-bearing resumed-turn launch,
- and local successor allocation.

## Relevant Unified Agent API Change

The local UAA branch under evaluation still contains promptless resume/fork behavior:

- `agent_api.session.resume.v1`
- `agent_api.session.fork.v1`

Key implementation files in the local UAA checkout:

- [../unified-agent-api/crates/agent_api/src/backend_harness/normalize.rs](../unified-agent-api/crates/agent_api/src/backend_harness/normalize.rs)
- [../unified-agent-api/crates/agent_api/src/backends/session_selectors.rs](../unified-agent-api/crates/agent_api/src/backends/session_selectors.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/exec.rs](../unified-agent-api/crates/agent_api/src/backends/codex/exec.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/fork.rs](../unified-agent-api/crates/agent_api/src/backends/codex/fork.rs)
- [../unified-agent-api/crates/agent_api/src/backends/codex/harness.rs](../unified-agent-api/crates/agent_api/src/backends/codex/harness.rs)
- [../unified-agent-api/crates/agent_api/src/backends/claude_code/util.rs](../unified-agent-api/crates/agent_api/src/backends/claude_code/util.rs)

That behavior is still real in the UAA implementation tree.

What changed on the Substrate side is the conclusion:

- Substrate no longer needs blank-prompt resume/fork as the mechanism for its current public
  control surfaces;
- the UAA implementation can still be interesting evidence for future contract design;
- but it is not a live dependency for this runtime slice.

## Why Promptless Resume Is No Longer An Immediate Substrate Requirement

There is still a meaningful architectural question about whether UAA should eventually publish an
explicit control-only resume surface.

But the old immediate Substrate pressure has been relieved because:

1. Substrate can now reattach through a dedicated attach-control path;
2. continuity stays private under the persisted host attach contract;
3. prompt-bearing follow-up remains on prompt-bearing `turn`;
4. public success truth is checked against actual attachment, not inferred from synthetic prompt
   completion.

So the current question is no longer "how do we make blank prompt safe enough?" It is "would an
explicit UAA control-only attach/resume surface still be worth standardizing later?"

## Why Promptless Fork Is Not Needed For Current Substrate

Current Substrate evidence is now much stronger than earlier drafts:

1. public `fork` is satisfied by local durable successor allocation;
2. successor truth is honest without attaching a live owner loop;
3. inherited continuity is intentionally cleared on the successor;
4. no prompt-bearing follow-up is smuggled into `fork`;
5. the runtime no longer needs backend-native promptless fork to preserve public semantics.

That makes promptless UAA fork unnecessary for the current architecture.

## Recommended Direction

### Direction A: keep current Substrate architecture and decouple it from blank prompt

This is the active recommendation.

- keep control-only attach inside the dedicated attach path;
- keep prompt-bearing follow-up on prompt-bearing `turn`;
- keep successor allocation as durable-state work;
- do not describe blank prompt as the architectural substrate for public `reattach` or `fork`.

### Direction B: explore future explicit UAA control-only resume only if it buys clarity

If future cross-backend work wants one published UAA control-only contract, prefer an explicit
surface rather than empty-prompt signaling.

Examples to consider later:

- an explicit attach/resume control API;
- a versioned resume extension with a first-class "no turn" mode;
- a gateway/private adapter seam if Substrate remains the only consumer.

### Direction C: do not pursue promptless UAA fork unless requirements materially change

There is no current Substrate requirement that justifies making promptless fork part of stable
published semantics.

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
- [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
- [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
- [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
- [crates/shell/src/execution/prompt_fulfillment.rs](crates/shell/src/execution/prompt_fulfillment.rs)
- [crates/world-service/src/member_runtime.rs](crates/world-service/src/member_runtime.rs)
- [docs/USAGE.md](docs/USAGE.md)

### UAA normative docs

Local UAA checkout under review:

- repo: `/home/azureuser/__Active_Code/atomize-hq/unified-agent-api`
- branch: `feat/promptless-resume`
- commit previously observed in this line of work: `3b7a4ef`

- [../unified-agent-api/docs/specs/unified-agent-api/run-protocol-spec.md](../unified-agent-api/docs/specs/unified-agent-api/run-protocol-spec.md)
- [../unified-agent-api/docs/specs/unified-agent-api/extensions-spec.md](../unified-agent-api/docs/specs/unified-agent-api/extensions-spec.md)
- [../unified-agent-api/docs/specs/claude-code-session-mapping-contract.md](../unified-agent-api/docs/specs/claude-code-session-mapping-contract.md)
- [../unified-agent-api/docs/specs/codex-wrapper-coverage-scenarios-v1.md](../unified-agent-api/docs/specs/codex-wrapper-coverage-scenarios-v1.md)
- [../unified-agent-api/docs/specs/codex-app-server-jsonrpc-contract.md](../unified-agent-api/docs/specs/codex-app-server-jsonrpc-contract.md)
- [../unified-agent-api/docs/specs/unified-agent-api/capability-matrix.md](../unified-agent-api/docs/specs/unified-agent-api/capability-matrix.md)

## Questions To Keep Answering

1. Would a published explicit UAA control-only resume surface simplify multi-backend adapter work
   enough to justify the extra contract surface?
2. Should that future surface live in UAA proper, in the gateway adapter seam, or in a
   Substrate-private control plane?
3. Do any future world-start or lazy-attach slices create a real new requirement that changes the
   current "no promptless fork needed" conclusion?

## Bottom Line

Based on the current Substrate runtime:

- promptless UAA resume is no longer required for live public `reattach`,
- promptless UAA fork is not required for live public `fork`,
- blank prompt should not be described as current Substrate architecture,
- and any future UAA control-only contract should be explicit rather than empty-prompt-shaped.
