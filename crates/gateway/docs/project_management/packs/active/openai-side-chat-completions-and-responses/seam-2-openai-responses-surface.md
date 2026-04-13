---
seam_id: SEAM-2
seam_slug: openai-responses-surface
type: capability
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
  required_threads:
    - THR-10
    - THR-12
  stale_triggers:
    - ADR 0008 contract changes for `/v1/responses` supported item types, streaming event minimum set, or tool loop requirements
    - `SEAM-1` lands shared adapter invariant changes that require revalidation before this seam can become exec-ready
    - upstream provider `/v1/responses` behavior shifts enough to invalidate adapter assumptions about tool streaming or output item mapping
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S99
  status: passed
open_remediations: []
---

# SEAM-2 - OpenAI Responses Public Surface

- **Goal / value**: add `POST /v1/responses` as the preferred modern OpenAI-shaped ingress for tool use and richer streaming semantics while remaining a thin adapter over the same normalized internal core.
- **Scope**
  - In:
    - request parsing per ADR 0008:
      - `input` as string shorthand and as item arrays (`message`, `function_call_output`)
    - function tools only (`tools`, `tool_choice`, `parallel_tool_calls`)
    - non-streaming Response object mapping (`object: response`, `output` items with `message` and `function_call`)
    - streaming SSE events per ADR 0008 minimum set, including function call argument deltas/done
    - shared behavior aligned with `SEAM-1`: model echo, `X-Provider` override, error envelope, chain-of-thought suppression
  - Out:
    - built-in tools and non-function tool call types
    - JSON schema outputs and additional item types beyond the contracted subset
- **Primary interfaces**
  - Inputs:
    - `POST /v1/responses` request subset from ADR 0008
    - `function_call_output` tool continuation items with `call_id`
  - Outputs:
    - non-streaming Response object (`object: response`, `output: [...]`)
    - streaming semantic-event subset: `response.created`, `response.output_item.added`, `response.output_item.done`, `response.completed`, plus text events (`response.content_part.added`, `response.output_text.delta`, `response.output_text.done`, `response.content_part.done`) when text is streamed and function-call argument events (`response.function_call_arguments.delta`, `response.function_call_arguments.done`) when tool calls are streamed; `data.type` matches the event name
    - error envelope compatible with ADR 0008
- **Key invariants / rules**:
  - remain a thin adapter over the shared core and `THR-10` invariants (no forked execution model)
  - function tools only; tool outputs must be strings (caller serializes structured data)
  - must not surface provider chain-of-thought / reasoning content as user-visible text
  - unknown top-level fields are ignored; known-but-unsupported built-in tools and non-function tool call types reject with `400`
- **Dependencies**
  - Direct blockers:
    - none; `THR-10` is already published by `SEAM-1` and revalidated here against `governance/seam-1-closeout.md` plus `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
  - Transitive blockers:
    - provider-stream normalization must remain available as a reusable input to streaming transforms
  - Direct consumers:
    - `SEAM-3` (conformance)
  - Derived consumers:
    - future expansions of the Responses subset once explicitly added by later ADRs
- **Touch surface**:
  - `gateway/src/server/mod.rs` (route wiring and handler)
  - new adapter module(s) under `gateway/src/server/` for Responses request/response/stream transforms
  - shared core shapes and stream model in `gateway/src/core.rs` and related modules
  - tests: `gateway/tests/*` (Response object fixtures and SSE golden tests)
- **Verification**:
  - golden tests for non-streaming response objects:
    - text-only response
    - tool-call response (`function_call` output items with JSON-string arguments)
  - golden tests for streaming events:
    - event order and `data.type` matching for the contracted semantic event subset, with assertions conditioned on whether the stream is text output, function-call output, or mixed output
    - output text delta assembly and done events
    - function call argument delta/done events
  - tool-loop fixture: follow-up request appending `function_call_output` items continues the conversation correctly
- **Risks / unknowns**:
  - Risk: Responses streaming event compatibility is more structured than Chat Completions chunks; SDK event parsing may be sensitive to missing events or inconsistent payload fields.
  - De-risk plan: implement the contracted semantic event subset first and use fixture-driven tests to enforce event ordering and required fields for the events each stream shape actually emits before adding any optional events.
- **Rollout / safety**:
  - keep `/v1/messages` as the primary contract; `responses` is additive
  - prefer minimal compatibility subset first; expand only via later ADRs
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: ADR 0008 makes Responses the preferred modern surface, but it should build on the already-proven shared adapter invariants established by `SEAM-1`.
  - Which threads matter most: `THR-10` is already revalidated as the consumed basis, and `THR-12` publishes the Responses contract for downstream conformance once implementation lands.
  - What the seam-local pre-exec review locked in: the ADR 0008-compatible semantic event subset, `call_id` threading, reject-vs-ignore posture, and thin-adapter constraints that execution must preserve.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-11`
  - Threads likely to advance: `THR-12` (to `published`)
  - Review-surface areas likely to shift after landing: R2/R3 should be updated with concrete module names for Responses transforms and any shared transform extraction done during landing.
  - Downstream seams most likely to require revalidation: `SEAM-3` if the event set or tool streaming semantics drift during implementation.
