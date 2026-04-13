---
pack_id: openai-side-chat-completions-and-responses
pack_version: v1
pack_status: extracted
source_ref: docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md plus current gateway OpenAI-compat adapter surfaces in gateway/src/server/
execution_horizon:
  active_seam: null
  next_seam: null
---

# Scope Brief - OpenAI-Side Chat Completions and Responses

- **Goal**: expand the gateway’s OpenAI-shaped public ingress so OpenAI SDKs and integrations can use `/v1/chat/completions` for compatibility and `/v1/responses` for modern tool+streaming semantics, while keeping both endpoints thin adapters over the same normalized internal core.
- **Why now**: the repo already exposes a limited `/v1/chat/completions` route and the provider layer already speaks upstream OpenAI-family protocols (including `/v1/responses` for some upstream models), but public OpenAI-side ingress is currently incomplete and does not match the contract in ADR 0008 (notably: streaming, function-tool loop, and Responses semantics).
- **Primary user(s) + JTBD**:
  - OpenAI SDK / integration users want a drop-in path that “just works” against this gateway without adopting the Anthropic `/v1/messages` surface.
  - Gateway operators want clear, testable compatibility boundaries (supported fields, rejected fields, and error envelope) and a single core execution model (no forked engine per public API family).
- **In-scope**:
  - Expand public `POST /v1/chat/completions` to match the ADR 0008 contract subset:
    - request parsing (roles, string-or-parts content, user/assistant image URLs, text-only system/developer, tool messages)
    - function tools only (`tools`, `tool_choice`) and the tool loop using `tool` role messages
    - non-streaming response shape (single choice, tool call mapping, finish reasons, usage)
    - streaming response shape (`text/event-stream`, chunk semantics, tool call deltas, `[DONE]`, optional usage chunk)
    - shared behavior: model echo, `X-Provider` override, error envelope, chain-of-thought suppression
  - Add public `POST /v1/responses` to match the ADR 0008 contract subset:
    - input item parsing (`message` items + `function_call_output` items)
    - function tools only and the tool loop using `function_call_output` items
    - non-streaming response object shape (`output` items)
    - streaming event set and SSE payload conventions
    - shared behavior: model echo, `X-Provider` override, error envelope, chain-of-thought suppression
  - Conformance hardening to lock the above contracts into drift guards (tests and negative-case coverage).
- **Out-of-scope**:
  - OpenAI built-in tools (web search, file search, code interpreter, MCP, etc.) and non-function tool call types
  - JSON schema / structured outputs, audio modalities, logprobs, `n > 1`, and other known-but-unsupported fields (must be rejected per ADR)
  - Replacing `/v1/messages` as the primary public contract for the current Claude Code path
  - Introducing a second execution engine or provider-normalization fork for OpenAI-facing endpoints
- **Success criteria**:
  - `/v1/chat/completions` matches the contracted request/response semantics (including streaming + tool loop) as a thin adapter over the existing normalized core
  - `/v1/responses` exists and matches the contracted request/response semantics (including streaming + tool loop) as a thin adapter over the existing normalized core
  - shared behavior (model echo, provider forcing, error envelope, and chain-of-thought suppression) is consistent across both endpoints
  - a minimal conformance suite exists that prevents drift on the compatibility subset, including negative-case tests for rejected fields and non-function tools
- **Constraints**:
  - ADR 0008 is the normative contract for OpenAI-side public ingress
  - `IMPORTANT_SUBSTRATE_ALIGNMENT.md` still constrains boundary discipline and identity posture
  - public `/v1/chat/completions` and public `/v1/responses` must parse into the same internal `GatewayRequest` shape and convert back from the same internal `GatewayResponse`/stream model (no forked semantics)
  - streaming adapters must remain pure transforms over provider-normalized streams (no provider-specific streaming logic in the public adapter)
- **External systems / dependencies**:
  - OpenAI client SDKs and OpenAI-shaped integrations targeting Chat Completions and/or Responses
  - upstream provider behavior via existing provider adapters (especially `gateway/src/providers/openai.rs`)
  - the current gateway normalized core, routing, and tool-use representation
- **Known unknowns / risks**:
  - matching OpenAI streaming chunk/event details closely enough for existing SDK parsers without turning the adapter into a brittle spec mirror
  - representing tool calls and tool results as function-only objects while preserving the internal normalized tool model (and preventing built-in tool leakage)
  - ensuring chain-of-thought / reasoning content is never surfaced as user-visible text even when upstream providers include it
  - keeping unknown-field ignore behavior (forward-compat) while still rejecting known-but-unsupported fields deterministically
- **Assumptions**:
  - the existing normalized-core architecture is the single execution model and is sufficient to support both OpenAI-shaped ingress surfaces without semantic forks
  - the current `/v1/chat/completions` implementation in `gateway/src/server/mod.rs` and `gateway/src/server/openai_compat.rs` is a starter adapter that can be expanded rather than replaced wholesale
  - the repo’s existing provider support for upstream `/v1/responses` (e.g., Codex models) can be reused behind a new public `/v1/responses` adapter
