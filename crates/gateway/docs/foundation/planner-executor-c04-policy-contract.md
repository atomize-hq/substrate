# Planner/Executor `C-04` Policy Contract

## Purpose

This note is the canonical landing artifact for `C-04`.
It defines the internal planner/executor routing policy and state-handoff rules for the gateway while keeping model-role selection internal-only and replaceable.

This contract is intentionally narrow:

- it defines how route selection is ordered and bounded
- it defines how planning-to-execution handoff is understood over normalized `C-02` semantics and the landed `C-03` public surface
- it defines what diagnostics and configuration may reveal internally
- it does not define provider parsing, public API semantics, or external backend identity

## Canonical Source Of Truth

The contract is grounded in the landed normalized and public-surface notes:

- `docs/foundation/azure-kimi-c02-normalized-event-contract.md`
- `docs/foundation/anthropic-messages-c03-contract.md`

The current implementation anchors are:

- `gateway/src/router/mod.rs`
- `gateway/src/server/mod.rs`

If this note and those anchors ever disagree, the code or the upstream seam notes must be revalidated before downstream use.

## Current Router Anchors

`gateway/src/router/mod.rs` is the current policy anchor for this seam.
The file already expresses a concrete route-selection order and should be treated as the reference behavior for this contract:

1. auto-map the incoming model name when it matches the router's auto-map rule
2. route to web search when a `web_search*` tool is present
3. route background tasks from the original model name before auto-mapping
4. route subagent requests when the system prompt carries the subagent model tag
5. apply prompt rules against the turn-starting user message
6. hand off tool-result-only continuation turns to execution/default before plan mode
7. route plan mode when thinking is enabled
8. fall back to the transformed model name or the original model name

The router also already establishes these policy behaviors:

- prompt rules are first-match-wins
- `strip_match` may remove the matched text from the selected turn-starting prompt
- prompt matching persists through tool-call turns by looking at the turn-starting user message, not just the last message
- subagent extraction removes the model tag from the system prompt after use
- route decisions remain internal policy data, not public backend identity

## Route-Selection Invariants

The following invariants are part of `C-04`:

- route selection is an internal policy layer above provider normalization and below public API semantics
- provider parsing must not decide planner/executor behavior
- public clients must not select separate planner and executor backends
- the policy may use `Kimi-K2-Thinking` and `Kimi-K2.5` internally, but those roles remain replaceable implementation details
- `RouteType` and similar routing signals are diagnostic or internal control-plane values, not public contract values
- prompt-driven routing, think-mode routing, background routing, and web-search routing remain ordered internal concerns
- any future change to the router order must preserve the ability to explain selection without reading provider parsing code

## Session-Handoff Guarantees

Planning-to-execution handoff is defined in terms of normalized `C-02` semantics and the landed `C-03` public surface, not raw provider payloads.

Required guarantees:

- a planning turn can be carried into an execution turn without exposing internal role selection to the public contract
- handoff state is owned by the gateway session, not by raw provider framing
- the gateway may preserve or inject continuation hints internally, but those hints are not public contract
- tool-result-only follow-up remains a valid continuation signal for the next assistant turn
- when a plan-mode turn resumes on a tool-result-only follow-up, the router hands off to execution/default instead of keeping the turn on the think model
- the handoff path must remain explainable using normalized `tool_intent`, `action`, and `final` semantics together with `C-03` session continuation rules

Current behavior this note treats as evidence, not as a future expansion mandate:

- the router already treats prompt selection as turn-scoped across tool-call turns
- the public surface already defines session continuation and tool-result follow-up without requiring provider-specific framing knowledge
- internal policy may depend on normalized session state, but it must not require raw Azure response inspection to stay coherent

## Diagnostics And Public Boundary Rules

Diagnostics may expose internal policy decisions to operators, but the public surface must stay capability-oriented.

Allowed internally:

- route decisions
- matched prompt evidence
- internal continuation or handoff diagnostics
- routing and policy debug logs

Not allowed publicly:

- separate public planner/executor backend identities
- provider parsing details
- raw hidden-marker syntax as a public concept
- config or docs that require operators or clients to pick planner versus executor roles

The public boundary must remain consistent with `C-03`:

- public behavior is still expressed as Anthropic-compatible capability behavior
- planner/executor identity stays hidden behind the gateway
- internal routing changes must not force a public contract rename or backend split

## Non-Goals

This note does not:

- change provider parsing or normalization logic
- define new public `/v1/messages` behavior
- add new runtime execution paths for planner/executor selection
- expose internal model-role names as stable public ids
- define downstream structured-event contracts or Substrate-facing identity policy
- require `gateway/src/server/mod.rs` to own routing policy

## Stale Triggers And Revalidation

Revalidate this note if any of the following becomes true:

- `docs/foundation/azure-kimi-c02-normalized-event-contract.md` changes normalized `tool_intent`, `action`, or `final` semantics in a way that affects policy handoff
- `docs/foundation/anthropic-messages-c03-contract.md` changes public session continuation or tool-result loop rules in a way that affects handoff
- `gateway/src/router/mod.rs` changes its routing order, turn-scoping rules, or role-selection assumptions materially
- `gateway/src/server/mod.rs` starts depending on planner/executor identity as public behavior
- provider-aware branching or raw provider parsing starts determining route selection
- planner/executor identity becomes visible in public docs, public config, or public backend naming

The default posture is `current` until one of those triggers fires.
If a trigger fires, this contract should be revalidated before downstream seams rely on it.

## Verification Checklist

`C-04` is complete only if a reviewer can answer yes to all of the following without reading provider parsing code:

- can the route-selection order be explained from the current router anchors
- can planner/executor routing be described as internal policy rather than public backend identity
- can the handoff path be explained using normalized `C-02` semantics and the landed `C-03` session rules
- do diagnostics remain internal-only while public docs and config stay capability-oriented
- do the non-goals keep provider parsing and external identity out of scope
- are the stale triggers clear enough that later seams know when to revalidate

## Compatibility Notes

- This note is compatible with the landed `C-02` normalization boundary and the landed `C-03` public surface.
- This note does not require a runtime rewrite to become useful; it freezes the policy language around current router behavior.
- This note is intentionally replaceable. Future runtime changes may refine the policy, but they must keep planner/executor identity internal-only.
