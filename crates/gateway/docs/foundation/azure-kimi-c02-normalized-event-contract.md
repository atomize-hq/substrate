# Azure Kimi `C-02` Normalized Event Contract

## Purpose

This note is the canonical landed source for `C-02`.
It defines the internal normalized event contract for Azure Kimi provider behavior so downstream seams can cite one stable artifact instead of reverse-engineering parser code or raw Azure payloads.

The contract is intentionally narrow:

- it covers normalization of Azure Kimi `tool_calls`, hidden `reasoning_content` markers, mixed cases, malformed markers, and empty-content fallback
- it does not define planner/executor policy
- it does not define public surface behavior
- it does not define transport, deployment, or client-facing API semantics

## Canonical Vocabulary

`C-02` uses one normalized event vocabulary with three contract-level event kinds:

1. `tool_intent`
2. `action`
3. `final`

### `tool_intent`

Represents a normalized tool invocation request that downstream consumers may execute or route.

Required contract meaning:

- the model has expressed tool intent
- the intent may originate from explicit Azure `tool_calls` or hidden `reasoning_content`
- downstream consumers must not need to inspect raw Azure sentinel syntax to understand the intent

Stable fields:

- `event_kind`
- `tool_name`
- `tool_id` when Azure provides one
- `tool_arguments` when Azure provides parseable arguments
- `source_origin`

### `action`

Represents a normalized intermediate step associated with tool use or assistant progress that is still part of the same internal response flow.

Required contract meaning:

- the response is not yet final
- the event may carry assistant progress text or tool-related intermediate state
- the event stays within the normalized internal model and does not expose raw provider chunk shape

Stable fields:

- `event_kind`
- `summary_text` when present
- `source_origin`

### `final`

Represents the terminal assistant completion for the normalized response.

Required contract meaning:

- no further tool intent is required for the current response
- downstream seams can treat the response as complete

Stable fields:

- `event_kind`
- `final_text` when present
- `source_origin`

## Field Rules

`C-02` distinguishes stable contract fields from debug-only provenance.

Stable contract fields:

- `event_kind`
- `tool_name`
- `tool_id`
- `tool_arguments`
- `summary_text`
- `final_text`
- `source_origin`

Debug-only provenance:

- `provenance_ref`
- raw Azure sentinel strings
- raw Azure response fragments
- provider chunk ordering details

Downstream seams may rely on stable contract fields.
They must not depend on debug-only provenance as part of the consumer contract.

`source_origin` is stable only as a normalized internal label.
It must not expose raw Azure provider path names, raw chunk shapes, or any requirement that downstream consumers distinguish explicit `tool_calls` from hidden `reasoning_content`.

## Collision Rules

Azure may present explicit `tool_calls` and hidden `reasoning_content` in the same response.

`C-02` resolves that collision as follows:

- explicit `tool_calls` take precedence for tool identity and arguments when they are present and parseable
- hidden `reasoning_content` may still contribute provenance and fallback intent classification
- if both paths indicate the same tool intent, the normalized event remains a single `tool_intent`
- if the explicit and hidden paths disagree, the normalized result must preserve the conflict as a conservative `tool_intent` classification with provenance retained for debugging, not as two competing consumer-visible contracts

The consumer-facing rule is simple:

- downstream seams see one normalized contract path
- they do not re-parse Azure sentinel syntax to resolve the collision themselves

## Malformed Marker And Empty-Content Semantics

Malformed markers:

- if hidden `reasoning_content` markers are incomplete, out of order, or otherwise malformed, the contract does not invent a new provider-specific shape
- the parser may record provenance for the malformed input, but the consumer-facing contract remains conservative
- malformed hidden markers never become a new downstream parsing obligation

Empty-content fallback:

- if Azure returns empty content alongside otherwise meaningful tool intent, the normalized event may still be `tool_intent` or `final` depending on the parsed provider signals
- empty content alone is not sufficient to create a new event kind
- empty content must never force downstream consumers to inspect raw Azure response framing to determine meaning

## Reuse Versus Bypass

`C-02` is designed to reuse the published `C-01` provider-boundary shape while bypassing any upstream transform behavior that would blur Azure-specific normalization into generic provider logic.

Required boundary statement:

- reuse the `C-01` provider extension boundary as the attachment point
- bypass any inherited behavior that depends on raw provider chunk shape or assumes explicit `tool_calls` are the only source of tool intent
- keep Azure normalization below planner/executor policy and above public surface rendering

## Verification Checklist

`C-02` is ready for downstream consumption only if the following can be answered yes:

- explicit-only Azure responses normalize into one `tool_intent` or `final` path without raw sentinel exposure
- hidden-only Azure responses normalize into one `tool_intent` or `final` path without downstream re-parsing
- mixed explicit-plus-hidden Azure responses resolve through the collision rule above
- malformed-marker Azure responses remain conservative and do not create new consumer-facing parsing obligations
- empty-content Azure responses still normalize into a stable event without leaking provider framing

Pass/fail framing:

- pass when downstream seams can cite this note directly and rely on `C-02` stable fields only
- fail when downstream seams need raw Azure payload semantics, routing policy, or public-surface behavior to interpret the contract
