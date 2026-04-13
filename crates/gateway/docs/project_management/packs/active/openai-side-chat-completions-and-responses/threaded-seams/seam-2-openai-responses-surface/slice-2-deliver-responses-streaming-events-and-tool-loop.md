---
slice_id: S2
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-11` changes the streaming event subset, payload conventions, or tool-call delta behavior after this slice starts"
    - the normalized stream model changes such that Responses event streaming can no longer be implemented as a pure transform
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
### S2 - Deliver Responses Streaming Events And Tool Loop

- **User/system value**: OpenAI clients can use `stream=true` on `/v1/responses` and receive the contracted semantic event subset with tool-call delta/done semantics, while the route remains a thin adapter over the normalized core.
- **Scope (in/out)**:
  - In: implement streaming support for `/v1/responses`, including SSE headers, event payload conventions, the contracted semantic event subset, output text delta assembly, tool-call argument delta/done, completion semantics, and the runtime glue needed to keep tool loops inside the contracted Responses shape.
  - Out: built-in tools, schema outputs, and provider-specific stream parsing inside the public endpoint.
- **Acceptance criteria**:
  - streamed responses emit the required `response.*` events for the stream shape being produced, with stable `data.type` naming and payload shapes per `C-11`
  - streamed tool calls and finalization behavior align with the sync contract semantics (arguments are JSON strings, tool-loop threading is stable)
  - stream handling preserves `X-Provider` override behavior, model echo, and chain-of-thought suppression
  - the public adapter consumes the normalized stream model rather than parsing provider-specific streaming bytes
  - the supported event subset is fixed to `response.created`, `response.output_item.added`, `response.output_item.done`, `response.completed`, plus text events when text is streamed and function-call argument events when tool calls are streamed
- **Dependencies**: `S00`, `S1`, `gateway/src/server/mod.rs`, new Responses adapter module(s), `gateway/src/core.rs`, `gateway/src/providers/openai.rs`, `THR-10`, `THR-12`
- **Verification**:
  - golden streaming tests cover event ordering, per-shape required event presence, payload minimum fields, tool-call delta/done, and completion semantics
  - regression checks confirm the public adapter transforms normalized output rather than parsing provider stream framing
  - pass condition: a reviewer can explain the stream conversion boundary and contracted event subset without referencing raw provider bytes
- **Rollout/safety**: if normalized output is insufficient for a required event, update the owned contract and/or upstream normalized model intentionally; do not sneak provider-specific behavior into the public adapter.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`Likely mismatch hotspots`)

#### S2.T1 - Add The Responses Event Stream Adapter

- **Outcome**: the route emits contracted Responses SSE events from the normalized stream model.
- **Inputs/outputs**: inputs are `C-11`, `C-12`, current normalized stream handling, and `/v1/responses` route wiring; outputs are event-adapter code and handler wiring updates.
- **Thread/contract refs**: `THR-10`, `THR-12`, `C-11`, `C-12`
- **Implementation notes**: emit required `response.*` events, enforce `data.type` naming, and keep the adapter boundary pure.
- **Acceptance criteria**: streamed text-only paths work end-to-end and the handler mirrors the existing normalized stream patterns rather than inventing a second engine.
- **Test notes**: add stream fixtures that assert headers, event ordering, required fields, and completion semantics.
- **Risk/rollback notes**: the highest risk is coupling to upstream OpenAI SSE framing instead of the normalized model.

Checklist:
- Implement: wire `/v1/responses` streaming through a dedicated event adapter
- Test: assert headers, event ordering, and minimum payload fields
- Validate: confirm the stream path consumes normalized output
- Cleanup: avoid provider-specific branches in the public route

#### S2.T2 - Add Tool-Call Delta And Finalization Semantics

- **Outcome**: streamed tool calls and finalization behavior match the owned Responses contract.
- **Inputs/outputs**: inputs are normalized tool-use/final semantics, `C-11`, and the event adapter from `S2.T1`; outputs are tool-delta assembly, completion semantics, and any helper code needed to keep sync and stream semantics aligned.
- **Thread/contract refs**: `THR-10`, `THR-12`, `C-11`, `C-12`
- **Implementation notes**: keep tool-call delta assembly deterministic, preserve JSON-string arguments, validate `call_id` threading, and ensure reasoning content never becomes user-visible text.
- **Acceptance criteria**: streamed tool-call sequences can be assembled by clients into the same logical call data as sync outputs; done events are emitted consistently and tested.
- **Test notes**: add tool-delta fixtures, mixed text-plus-tool streaming cases, and completion assertions.
- **Risk/rollback notes**: if tool deltas diverge from sync contract rules, downstream conformance will freeze incompatible behavior.

Checklist:
- Implement: add tool-call delta assembly and completion semantics
- Test: cover streamed tool calls, mixed output, and done events
- Validate: confirm reasoning content stays suppressed in event mode
- Cleanup: keep mapping helpers shared between sync and stream where possible
