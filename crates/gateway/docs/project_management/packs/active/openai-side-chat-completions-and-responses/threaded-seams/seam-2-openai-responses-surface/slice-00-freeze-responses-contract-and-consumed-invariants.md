---
slice_id: S00
seam_id: SEAM-2
slice_kind: contract_definition
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR 0008 changes the supported Responses input items, tool-loop rules, or minimum streaming event set before execution starts
    - "`THR-10` publishes `C-12` invariant deltas that require updating the Responses adapter boundary"
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-10
  - THR-12
contracts_produced:
  - C-11
contracts_consumed:
  - C-12
open_remediations: []
---
### S00 - Freeze Responses Contract And Consumed Adapter Invariants

- **User/system value**: implementation starts from one concrete owned Responses contract (`C-11`) plus one concrete consumed invariant set (`C-12`) instead of inventing item mapping, tool-loop, or streaming event rules during coding.
- **Scope (in/out)**:
  - In: freeze the owned `C-11` Responses subset contract, explicitly restate how `C-12` constrains the public adapter (pure transform over the normalized core), name canonical landing artifacts, and define the verification checklist that later proves sync and stream behavior while rejecting built-in tools deterministically.
  - Out: landing runtime behavior, conformance ownership for the whole pack, and seam-exit publication accounting.
- **Acceptance criteria**:
  - one canonical landing artifact path is named for `C-11`
  - `C-11` makes explicit:
    - supported input item types (`message`, `function_call_output`) and the string shorthand behavior
    - function-tools-only rules, `tool_choice`, `parallel_tool_calls`, and the tool-loop continuation rule set
    - known-but-unsupported field and built-in tool rejection posture (and unknown-field ignore posture)
    - sync Response object mapping rules (output items, tool calls, finish semantics, usage)
    - streaming event subset, required payload fields, `data.type` naming conventions, and per-shape event ordering guarantees
  - the slice restates the consumed `C-12` constraints: `/v1/responses` must parse into `GatewayRequest` and emit from the normalized response/stream model without provider-specific public streaming logic
  - the verification checklist names exact code/test anchors, edge cases, and pass/fail conditions needed for `gates.pre_exec.contract`
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-2-openai-responses-surface.md`, `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`, `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- **Verification**:
  - a reviewer can answer which fields are ignored versus rejected, how `call_id` threading works across tool-loop turns, and which streaming events are required without reading implementation diffs
  - pass condition: `SEAM-2` satisfies `gates.pre_exec.contract` now that `THR-10` is published and revalidated, without waiting for closeout-backed publication of `C-11` to exist already
- **Rollout/safety**: keep `C-11` narrow and compatibility-scoped; do not encode provider framing, internal routing policy, or schema-output expansion into the owned contract.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`)

#### Frozen canonical artifacts (this slice output)

- `C-11` (Responses subset contract, canonical landing artifact path): `docs/foundation/openai-side-responses-c11-contract.md`
- `C-12` (consumed adapter invariants): `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- Normative policy baseline: `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`

#### Execution-grade freeze for `C-11`

- **Supported top-level request fields**:
  - `model`
  - `input`
  - `tools`
  - `tool_choice`
  - `parallel_tool_calls`
  - `text`
  - `max_output_tokens`
  - `temperature`
  - `top_p`
  - `stop`
  - `stream`
  - `stream_options`
- **Supported input items**:
  - `message` with `role: system|developer|user|assistant`
  - `function_call_output` with `{ "type": "function_call_output", "call_id": string, "output": string }`
- **Supported message content parts**:
  - `input_text`
  - `input_image`
- **Reject posture**:
  - non-function tool definitions reject with `400`
  - built-in tools reject with `400`
  - non-function tool call types reject with `400`
  - JSON schema outputs remain out of scope until a later ADR
- **Ignore posture**:
  - unknown top-level request fields are ignored for forward compatibility
- **Sync response subset**:
  - `object: "response"`
  - `status` stays within `completed|in_progress|failed|incomplete`
  - `output` supports only `message` and `function_call`
  - `function_call.arguments` remains a JSON string
  - `usage` is emitted when provider token accounting exists
- **Streaming event subset**:
  - shared lifecycle/output-item events:
    - `response.created`
    - `response.output_item.added`
    - `response.output_item.done`
    - `response.completed`
  - text-stream events, when text is streamed:
    - `response.content_part.added`
    - `response.output_text.delta`
    - `response.output_text.done`
    - `response.content_part.done`
  - function-call argument events, when tool calls are streamed:
    - `response.function_call_arguments.delta`
    - `response.function_call_arguments.done`
- **Streaming payload rule**:
  - every SSE `data:` payload is JSON whose `type` exactly matches the event name
- **Tool loop rule**:
  - the model emits `function_call` output items
  - client follow-up sends one `function_call_output` per tool call
  - `function_call_output.call_id` must match the emitted tool id exactly
  - `function_call_output.output` must be a string
- **Boundary anchors consumed from `C-12`**:
  - request parsing terminates in `gateway/src/core.rs::GatewayRequest`
  - response mapping originates from `gateway/src/core.rs::GatewayResponse` plus the normalized stream model
  - tool requests/results round-trip through `gateway/src/models/mod.rs` `tool_use` and `tool_result` content blocks
  - public handlers do not parse provider-specific stream framing

#### S00.T1 - Freeze The Request And Tool-Loop Contract

- **Outcome**: `C-11` names one concrete request subset and tool-loop rule set for `POST /v1/responses`.
- **Inputs/outputs**: inputs are ADR 0008, `threading.md`, any existing Responses provider behavior, and normalized request/tool shapes; outputs are the `C-11` contract source path plus a completed request/tool-loop matrix.
- **Thread/contract refs**: `THR-12`, `C-11` (and consumption of `THR-10` / `C-12`)
- **Implementation notes**: make explicit how `input` shorthand expands, how `message` and `function_call_output` items map into the normalized model, how `call_id` is validated and threaded, and which known-but-unsupported fields are rejected instead of ignored.
- **Acceptance criteria**: function tools only, `tool_choice`, `parallel_tool_calls`, tool-loop continuation, unknown-field ignore behavior, and built-in tool rejection are all named with exact pass/fail rules.
- **Test notes**: identify positive and negative fixtures for item parsing, call-id threading, built-in tool rejection, and unsupported field rejection.
- **Risk/rollback notes**: if this matrix stays implicit, streaming and tool-loop code will encode behavior that downstream conformance later has to reverse-engineer.

Checklist:
- Implement: freeze the request/tool-loop matrix and name the canonical `C-11` artifact location
- Test: enumerate positive and negative verification cases tied to the matrix
- Validate: confirm every supported or rejected field in ADR 0008 has an explicit disposition
- Cleanup: remove ambiguity about `call_id` threading and item normalization

#### S00.T2 - Freeze The Sync Response Object Contract

- **Outcome**: `C-11` names one concrete Responses object mapping over the normalized output model.
- **Inputs/outputs**: inputs are ADR 0008, normalized gateway response semantics, and `threading.md`; outputs are the Response object section of `C-11`.
- **Thread/contract refs**: `THR-12`, `C-11`, `C-12`
- **Implementation notes**: define output item shapes, tool call object mapping, finish semantics, usage handling, and chain-of-thought suppression rules.
- **Acceptance criteria**: text-only, tool-call-only, and mixed outputs map deterministically; tool arguments remain JSON strings; model echo uses the request model.
- **Test notes**: name golden fixtures for text-only, tool-call-only, mixed output, and usage present/absent cases.
- **Risk/rollback notes**: vague Response object rules will cause stream and sync adapters to diverge.

Checklist:
- Implement: freeze one sync Response object mapping and required fields
- Test: identify the exact golden fixtures needed
- Validate: confirm the contract is shaped over normalized outputs, not raw provider payloads
- Cleanup: remove ambiguity about finish and usage semantics

#### S00.T3 - Freeze The Streaming Event Subset Contract

- **Outcome**: `C-11` names one concrete streaming event subset and payload convention for `/v1/responses`.
- **Inputs/outputs**: inputs are ADR 0008 and normalized stream semantics; outputs are the streaming event section of `C-11`.
- **Thread/contract refs**: `THR-12`, `C-11`, `C-12`
- **Implementation notes**: define the shared lifecycle/output-item events, the text-only events, the function-call-only events, required `data.type` naming, payload minimum fields, ordering, and termination behavior.
- **Acceptance criteria**: one reviewer can explain the event adapter without consulting runtime code, and the contract makes it impossible to “mostly work” while omitting required events for a given stream shape.
- **Test notes**: name golden event-stream fixtures and ordering assertions.
- **Risk/rollback notes**: streaming parity is the highest compat risk; ambiguity here leads to endpoint-specific hacks and conformance drift.

Checklist:
- Implement: freeze the streaming event subset and payload conventions
- Test: identify golden streams and ordering assertions
- Validate: confirm chain-of-thought stays suppressed in event mode
- Cleanup: remove any ambiguity about `data.type` and done events

#### Verification anchors frozen by this slice

- route wiring and request/response entrypoint: `gateway/src/server/mod.rs`
- shared boundary types: `gateway/src/core.rs`, `gateway/src/models/mod.rs`
- canonical owned contract landing artifact: `docs/foundation/openai-side-responses-c11-contract.md`
- canonical consumed invariant artifact: `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- fixture and regression ownership: `gateway/tests/` plus adapter-local unit coverage under `gateway/src/server/`
