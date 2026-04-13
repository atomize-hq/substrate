---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: future
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-10` changes chunk semantics, usage-chunk rules, or tool-call delta behavior after this slice starts"
    - the normalized stream model changes such that Chat Completions streaming can no longer be implemented as a pure transform
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-10
  - THR-11
contracts_produced:
  - C-10
  - C-12
contracts_consumed: []
open_remediations: []
---
### S2 - Deliver Chat Completions Streaming And Tool Loop

- **User/system value**: OpenAI clients can use `stream=true` and tool-call continuation on `chat/completions` without being forced onto `/v1/messages`, and the route still stays thin over the normalized core.
- **Scope (in/out)**:
  - In: implement streaming support for `/v1/chat/completions`, including SSE headers, chunk conversion, text deltas, tool-call deltas, optional usage chunk handling, `[DONE]`, and the runtime glue needed to keep tool loops inside the contracted Chat Completions shape.
  - Out: full conformance ownership, `/v1/responses`, built-in tools, and provider-specific public stream parsing.
- **Acceptance criteria**:
  - the route no longer rejects `stream=true` for supported requests
  - streamed text and tool-call output emit OpenAI-compatible chunk objects plus `[DONE]` while consuming the normalized stream model instead of raw provider SSE
  - tool-call deltas assemble into the same logical contract as sync tool calls and preserve `arguments` as JSON strings
  - stream handling preserves `X-Provider` override behavior, model echo, and chain-of-thought suppression
- **Dependencies**: `S00`, `S1`, `gateway/src/server/mod.rs`, `gateway/src/server/openai_compat.rs`, `gateway/src/core.rs`, `gateway/src/providers/openai.rs`, `THR-10`, `THR-11`
- **Verification**:
  - golden streaming tests cover chunk sequencing, tool-call delta assembly, optional final usage chunk, and `[DONE]`
  - regression checks confirm the public adapter transforms normalized output rather than parsing provider-specific stream framing
  - pass condition: one reviewer can explain the stream conversion boundary and tool-loop behavior without referencing raw provider bytes
- **Rollout/safety**: keep the stream adapter at the public boundary; if normalized output is insufficient, change the owned contracts or upstream core deliberately instead of sneaking provider-specific logic into the handler.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`R1`, `R2`, `Likely mismatch hotspots`)

#### S2.T1 - Add The OpenAI Chunk Stream Adapter

- **Outcome**: the Chat Completions route emits contracted OpenAI chunk SSE from the normalized stream model.
- **Inputs/outputs**: inputs are `C-10`, `C-12`, current normalized stream handling, and the current `/v1/chat/completions` handler; outputs are stream-adapter code and route wiring updates.
- **Thread/contract refs**: `THR-10`, `THR-11`, `C-10`, `C-12`
- **Implementation notes**: emit `chat.completion.chunk` objects, `[DONE]`, and the right SSE headers without letting provider-specific public framing leak into the route.
- **Acceptance criteria**: streamed text-only paths work end-to-end and the handler’s control flow mirrors the existing normalized stream pattern rather than inventing a new engine.
- **Test notes**: add stream fixtures that assert headers, event ordering, chunk object shape, and `[DONE]` termination.
- **Risk/rollback notes**: the highest risk is coupling to Anthropic-style bytes or upstream OpenAI bytes instead of the normalized model.

Checklist:
- Implement: wire `/v1/chat/completions` streaming through a dedicated OpenAI chunk adapter
- Test: assert headers, chunk object shape, and `[DONE]`
- Validate: confirm the stream path still consumes normalized output
- Cleanup: remove the hard rejection of `stream=true` for supported cases

#### S2.T2 - Add Tool-Call Delta And Finalization Semantics

- **Outcome**: streamed tool calls and finalization behavior match the owned Chat Completions contract.
- **Inputs/outputs**: inputs are normalized tool-use/final semantics, `C-10`, and the stream adapter from `S2.T1`; outputs are tool-delta assembly, finish-reason handling, and optional usage-chunk logic.
- **Thread/contract refs**: `THR-10`, `THR-11`, `C-10`, `C-12`
- **Implementation notes**: keep tool-call delta assembly deterministic, preserve JSON-string arguments, and ensure reasoning content never becomes user-visible text.
- **Acceptance criteria**: streamed tool-call sequences can be assembled by clients into the same logical call data as sync responses; final usage chunk behavior is explicit and tested when requested by the contract.
- **Test notes**: add tool-delta fixtures, mixed text-plus-tool streaming cases, and final usage-chunk assertions.
- **Risk/rollback notes**: if tool deltas and finalization diverge from sync contract rules, downstream conformance will freeze incompatible behavior.

Checklist:
- Implement: add tool-call delta assembly and finalization behavior
- Test: cover streamed tool calls, mixed chunks, finish reasons, and usage-chunk placement
- Validate: confirm reasoning content stays suppressed in stream mode
- Cleanup: document or encode any shared helper needed to keep sync and stream semantics aligned
