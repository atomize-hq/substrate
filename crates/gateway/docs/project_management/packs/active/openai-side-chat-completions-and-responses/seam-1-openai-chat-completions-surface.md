---
seam_id: SEAM-1
seam_slug: openai-chat-completions-surface
type: capability
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
  required_threads:
    - THR-10
    - THR-11
  stale_triggers:
    - ADR 0008 contract changes for `chat/completions` supported/unsupported fields, streaming semantics, or tool loop requirements
    - gateway core `GatewayRequest` or tool representation changes in a way that forces adapter behavior changes (instead of pure transforms)
    - provider-normalized streaming shape changes in a way that breaks SSE chunk transforms or tool-call deltas
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

# SEAM-1 - OpenAI Chat Completions Public Surface (Expanded)

- **Goal / value**: make `POST /v1/chat/completions` a real OpenAI compatibility surface (including streaming + function tools) so existing OpenAI SDKs/integrations can target the gateway without adopting `/v1/messages`.
- **Scope**
  - In:
    - request parsing per ADR 0008 (roles, string-or-parts content, image URLs, tool messages)
    - function tools only (`tools`, `tool_choice`) and tool loop via `tool` role messages
    - shared behavior: `model` echo, `X-Provider` override behavior, error envelope contract, and chain-of-thought suppression
    - streaming support: `text/event-stream`, chunk objects, tool-call deltas, optional final usage chunk, and `[DONE]`
  - Out:
    - built-in tools and non-function tool call types
    - JSON schema outputs, audio, `logprobs`, `n > 1`, and other known-but-unsupported fields
- **Primary interfaces**
  - Inputs:
    - `POST /v1/chat/completions` request subset from ADR 0008
    - `X-Provider: <provider_name>` header override
  - Outputs:
    - non-streaming OpenAI Chat Completion response object (`object: chat.completion`)
    - streaming OpenAI chunk stream (`object: chat.completion.chunk` + `[DONE]`)
    - error envelope `{ "error": { "type": "error", "class": "auth|url|deployment|route|transport_drift", "message": "..." } }`
- **Key invariants / rules**:
  - ignore unknown top-level fields for forward-compat; reject known-but-unsupported fields with `400`
  - `choices` contains exactly one element (`n=1` contract)
  - function tools only; tool call `arguments` must remain a JSON string
  - must not surface provider chain-of-thought / reasoning content as user-visible text
  - remain a thin adapter over the shared normalized core (no endpoint-specific engine and no provider-specific streaming logic)
- **Dependencies**
  - Direct blockers:
    - none within this pack; this seam owns publishing shared adapter invariants (`THR-10`)
  - Transitive blockers:
    - upstream core + provider seams must remain stable (basis closeouts referenced above)
  - Direct consumers:
    - `SEAM-2` (shared invariants) and `SEAM-3` (conformance)
  - Derived consumers:
    - future OpenAI-side expansions (schema outputs, richer modalities) gated by later ADRs
- **Touch surface**:
  - `gateway/src/server/mod.rs` (`handle_openai_chat_completions` and route wiring)
  - `gateway/src/server/openai_compat.rs` (request parsing + response transform; currently missing tools/streaming)
  - shared core shapes: `gateway/src/core.rs`, `gateway/src/models/*` (tool blocks + content blocks), stream response types
  - tests: `gateway/tests/*` (new fixtures and regression coverage)
- **Verification**:
  - golden tests for non-streaming responses: text-only, tool-call-only, mixed content, usage present/absent
  - golden tests for streaming: chunk sequencing, tool call delta assembly, final usage chunk (when requested), `[DONE]`
  - negative tests: known-but-unsupported fields reject with the contracted error envelope and status mapping
  - cross-check: `model` echo behavior is correct even when routing maps to a different provider model internally
- **Risks / unknowns**:
  - Risk: streaming parity with OpenAI SDK parsers is subtle (chunk shapes, tool-call deltas, usage chunk placement).
  - De-risk plan: add fixture-driven tests for both “text streaming” and “tool call streaming”, and validate chunk/event ordering against the contracted minimum set.
- **Rollout / safety**:
  - keep `/v1/messages` as the primary supported surface and treat `chat/completions` as explicitly compatibility-scoped
  - preserve current non-streaming behavior for existing callers while introducing streaming behind `stream=true`
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: ADR 0008 orders compatibility-first; this seam is the lowest-friction adoption bridge.
  - Which threads matter most: `THR-10` (shared invariants) and `THR-11` (compat contract publication) must be explicit before `SEAM-2` deepens and before conformance locks in behavior.
  - What the first seam-local review should focus on: adapter-boundary discipline, tool-loop correctness, and streaming transform correctness without provider-specific branching.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-10` and `C-12`
  - Threads likely to advance: `THR-10` (to `published`), `THR-11` (to `published`)
  - Review-surface areas likely to shift after landing: R1 tool-loop diagram may gain explicit “tool_call_id” / “call_id” mapping notes; R2/R3 may gain concrete module names for the Responses adapter.
  - Downstream seams most likely to require revalidation: `SEAM-2` and `SEAM-3` if chunk/event shapes or tool mapping invariants shift during landing.
