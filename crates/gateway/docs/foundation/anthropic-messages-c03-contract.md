# Anthropic Messages `C-03` Contract

## Purpose

This note is the canonical landing artifact for `C-03`.
It defines the public Anthropic Messages gateway surface for the gateway while keeping the core client-agnostic and keeping provider normalization below the public boundary.

This contract is intentionally narrow:

- it defines the public `/v1/messages` surface semantics
- it defines how normalized `C-02` events are rendered into Anthropic-compatible public behavior
- it defines session continuation and tool-result loop rules
- it preserves a thin future OpenAI Responses adapter over the same normalized core

It does not define:

- Azure provider parsing
- raw Kimi framing or hidden marker syntax as public behavior
- planner/executor policy
- downstream deployment or identity policy

## Canonical Source Of Truth

`C-02` is the only internal source of truth for this contract.

Public `C-03` behavior must be derived from normalized `C-02` semantics, not from:

- raw Azure payload framing
- hidden `reasoning_content` marker syntax
- provider chunk ordering details
- internal role selection or routing policy

If a detail is not expressible from `C-02` normalized `tool_intent`, `action`, and `final` semantics, it is not part of the `C-03` public contract.

## Public Mapping

### `tool_intent`

When the normalized core emits `tool_intent`, the public Anthropic surface exposes that intent as Anthropic `tool_use` content blocks.

Required behavior:

- `tool_name`, `tool_id`, and `tool_arguments` from the normalized event drive the public `tool_use` block
- the public surface may emit one or more `tool_use` blocks if the normalized event set contains more than one tool intent
- the assistant turn is considered incomplete until tool results are returned or another terminal condition is reached
- the matching public stop behavior is `stop_reason: tool_use`

What this contract does not do:

- it does not require clients to inspect raw Azure `tool_calls`
- it does not expose hidden Kimi marker syntax
- it does not require the public surface to know whether the intent came from explicit provider tool calls or hidden provider markers

### `action`

When the normalized core emits `action`, the public Anthropic surface renders that progress as Anthropic `text` for the landed Kimi path.

Required behavior:

- `action` is a public intermediate state, not a new transport protocol
- Kimi hidden-marker-derived progress stays internal until normalization and then surfaces as public `text`
- `thinking` is not guaranteed by `action`; it may only appear when a normalized non-Kimi surface explicitly emits a thinking semantics from normalized state, not from raw provider reasoning markers
- `action` does not require the client to understand provider-specific chunk shape
- `action` does not create a new public contract for raw provider provenance

Current boundary evidence:

- Kimi hidden-marker-derived material stays internal to normalization
- the public surface must not re-emit hidden marker syntax as a public `thinking` contract
- Kimi-derived progress currently surfaces as public `text`, not as a public `thinking` guarantee
- any public `thinking` behavior must come from normalized surface semantics, not from raw Azure framing

### `final`

When the normalized core emits `final`, the public Anthropic surface exposes terminal assistant content as Anthropic `text` and closes the turn with Anthropic terminal stop semantics.

Required behavior:

- the response is complete from the public contract perspective
- the public surface emits terminal `text` content when present
- the matching stop behavior is a terminal Anthropic stop reason, typically `end_turn`
- truncation or stop-sequence semantics remain compatible with the normalized completion state

This contract does not over-specify implementation details beyond the normalized public outcome.

## Session And Tool-Result Continuation

Session continuation is defined in terms of public Anthropic messages, not raw provider frames.

Required behavior:

- a user turn containing `tool_result` blocks is treated as the follow-up to a prior `tool_use`
- the gateway preserves the conversation as one normalized session across these turns
- tool-result continuation is driven by normalized message content, not by inspecting raw Azure response structure
- a tool-result-only follow-up remains a valid continuation signal for the next assistant turn

Publicly visible rules:

- the client submits `tool_result` blocks when a tool completes
- the gateway continues the session using the normalized conversation state
- the client does not need provider-specific framing knowledge to continue the loop

Internal implementation detail:

- the gateway may preserve or inject continuation hints internally
- those hints are not part of the public contract

## Thin Responses-Later Boundary

`C-03` must remain a thin public surface over the same normalized core so future OpenAI Responses support stays an adapter seam, not a second execution engine.

Required boundary statement:

- Anthropic Messages is the first public ingress contract
- provider normalization remains below the public surface
- future Responses support wraps the same normalized core
- no later adapter may require a forked execution path or a second internal model of session/tool behavior

In practice, this means:

- `C-03` owns the Anthropic public contract only
- `C-02` owns the normalized event semantics only
- later client surfaces consume the same normalized core rather than re-parsing provider data

## Verification Checklist

`C-03` is complete only if a reviewer can answer yes to all of the following without reading raw provider parsing code:

- can the public `tool_use` mapping be explained directly from normalized `tool_intent` semantics
- can `action` and `final` be explained as public Anthropic behavior without exposing raw Azure framing
- do session continuation and tool-result follow-up rules stay within the public Anthropic message model
- does the contract keep Kimi hidden markers internal rather than promoting them to public surface truth
- does the note make the Responses-later path explicitly thin over the same normalized core

## Compatibility Notes

- This contract is compatible with the landed `C-02` normalization boundary and does not broaden it.
- This contract is compatible with the current `/v1/messages` surface and does not require runtime code changes in this seam note.
- This contract is intentionally capability-oriented and must not be read as a public declaration of provider identity, planner role selection, or raw transport shape.
