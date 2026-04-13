# OpenAI-Side Conformance Suite `C-13` Contract

## Purpose

This note is the canonical landing artifact for `C-13`.
It defines the SEAM-3 / `THR-13` conformance and drift-guard contract for the gateway's OpenAI-side surfaces.

`C-13` exists to keep the suite narrow, offline, and evidence-based:

- it freezes what the conformance suite must assert
- it maps those assertions to the already-contracted public subsets in `C-10`, `C-11`, and `C-12`
- it fixes fixture layout, replay boundaries, and deterministic tolerance rules

`C-13` does not define new API behavior. It only describes the regression coverage that guards the existing public contracts.

## Canonical Sources Of Truth

- SEAM-3 planning: `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/project_management/packs/active/openai-side-chat-completions-and-responses/threaded-seams/seam-3-openai-side-conformance-and-drift-guards/seam.md`
- SEAM-3 pre-exec review: `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/project_management/packs/active/openai-side-chat-completions-and-responses/threaded-seams/seam-3-openai-side-conformance-and-drift-guards/review.md`
- Chat Completions contract: `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/openai-side-chat-completions-c10-contract.md`
- Responses contract: `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/openai-side-responses-c11-contract.md`
- Shared adapter invariants: `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- ADR 0008 baseline: `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`

Evidence anchors in the repo, used to ground the contract but not treated as the contract itself:

- `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/server/openai_compat.rs`
- `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/server/openai_responses.rs`
- `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/src/server/mod.rs`

## Contract Scope

`C-13` covers the suite contract for the OpenAI-side compatibility subset only.

### In scope

- deterministic conformance coverage for `POST /v1/chat/completions` per `C-10`
- deterministic conformance coverage for `POST /v1/responses` per `C-11`
- cross-endpoint shared-behavior coverage per `C-12`
- offline fixture replay, including streaming replay, using local test assets only
- positive and negative cases for the contracted subset

### Out of scope

- expanding the public contract surface
- broad fuzzing, load testing, or perf work
- live upstream calls in CI
- snapshotting provider-specific framing or incidental implementation details

## Clause To Test-Category Map

`C-13` requires the suite to cover the following clause groups.

| Contract | Required test categories | What the suite must prove |
| --- | --- | --- |
| `C-10` Chat Completions | Sync positive cases, sync negative cases, streaming positive cases, streaming negative cases, tool-loop positive cases, tool-loop negative cases, reject/ignore checks, error-envelope checks | Request allowlist/reject list behavior, `tool` role continuation, `tool_calls` assembly, `delta.tool_calls` streaming, `[DONE]` termination, and model echo |
| `C-11` Responses | Sync positive cases, sync negative cases, streaming positive cases, streaming negative cases, tool-loop positive cases, tool-loop negative cases, reject/ignore checks, error-envelope checks | `input` parsing, `function_call_output` threading, the contracted `response.*` semantic event subset and per-shape ordering rules, `call_id` continuity, and model echo |
| `C-12` Shared invariants | Positive parity cases, negative parity cases, model echo, `X-Provider` forcing, chain-of-thought suppression, error-envelope/status mapping, thin-adapter boundary | Both endpoints must remain thin adapters over the shared normalized core and must not leak reasoning content or provider-specific stream framing |

## Determinism Rules

The suite MUST be deterministic.

- no live upstream network calls
- no timing-based assertions or sleeps
- no dependency on provider-specific framing in public handlers
- stable ordering must only be asserted where the relevant contract promises it
- optional fields may be tolerated only when the underlying contract allows them
- fixture replay must be reproducible from repository files alone

When a contract does not promise an ordering or field presence, the suite MUST assert the minimum contract shape instead of snapshotting incidental payload details.

## Fixture Schema And Namespaces

All fixtures live under:

- `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/gateway/tests/fixtures/`

The fixture namespace policy is:

- `gateway/tests/fixtures/openai_responses/` is the Responses fixture namespace
- `gateway/tests/fixtures/openai_chat_completions/` is the reserved Chat Completions fixture namespace for the later slice set
- shared fixture helpers, if any, MUST remain test-only and MUST not move into production code

### Minimal Canonical Fixture Set

`C-13` freezes the minimum planned fixture set that later slices are expected to add.

#### `gateway/tests/fixtures/openai_chat_completions/`

Planned canonical categories/files:

- `sync-text.json` - happy-path sync text-only completion
- `sync-tool-call.json` - happy-path sync completion with a tool call
- `sync-mixed.json` - happy-path sync completion with text plus tool call ordering
- `sync-tool-choice-function-selection.json` - explicit function `tool_choice` selection narrows the forwarded tool set
- `negative-unsupported-field.json` - request rejection for a known-but-unsupported top-level field
- `negative-non-function-tool.json` - request rejection for a non-function tool definition
- `negative-tool-call-id-mismatch.json` - request rejection for an invalid tool-loop continuation
- `stream-text.json` - happy-path streaming text deltas plus `[DONE]`
- `stream-tool-call.json` - happy-path streaming tool-call deltas plus `[DONE]`
- `stream-mixed.json` - happy-path mixed streaming text/tool-call sequence plus `[DONE]`
- `stream-include-usage.json` - streaming case that includes the final usage chunk

#### `gateway/tests/fixtures/openai_responses/`

Planned canonical categories/files:

- `sync-text.json` - happy-path sync Response object with text output only
- `sync-tool-call.json` - happy-path sync Response object with a function call output item
- `sync-mixed.json` - happy-path sync Response object with mixed text and function call ordering
- `request-tool-loop-function-call-output.json` - tool-loop continuation request fixture that preserves `call_id`
- `negative-built-in-tool.json` - request rejection for built-in tools
- `negative-unsupported-text-format.json` - request rejection for unsupported `text.format.type`
- `negative-non-function-tool.json` - request rejection for a non-function tool definition
- `negative-invalid-call-id.json` - request rejection for a malformed `function_call_output.call_id`
- `stream-mixed.json` - happy-path streaming Response event sequence with text and tool-call events
- `stream-tool-call.json` - happy-path streaming tool-call event sequence
- `stream-text.json` - happy-path streaming text-only event sequence
- `stream-with-usage.json` - streaming case that includes the final completed Response payload and usage-bearing terminal output

Fixture formats are intentionally small and explicit:

- JSON fixtures for request/response cases and expected assertions
- line-based SSE payload fixtures for streaming replay
- stub-provider inputs that replay normalized chunks, not live provider calls

Fixture files MUST describe only contract-relevant data:

- request payloads
- normalized provider response/stream payloads used for replay
- expected public-output fragments or ordering constraints
- forbidden fragments for drift detection

Fixture files MUST NOT encode:

- live network dependencies
- provider-only transport framing that the public handler is forbidden to parse
- incidental ids, timestamps, or implementation-private serialization details unless a contract explicitly requires them

## Stream-Replay Boundary

`C-13` fixes the stream-replay boundary at the normalized provider stream.

- the suite MAY inject in-process stub providers that emit deterministic normalized chunks
- the suite MUST exercise the public OpenAI handlers through the same public route boundary the gateway serves in production
- the suite MUST NOT replay provider-specific public framing into the contract as a source of truth
- the suite MUST treat normalized stream output as the seam between provider behavior and OpenAI-side public output

If an OpenAI-facing transform requires more detail than the normalized stream currently provides, the fix is a deliberate contract or model adjustment, not ad hoc fixture widening.

## Canonical Landing Artifact

`C-13` lands here:

- `/Users/spensermcconnell/__Active_Code/kimi-claude-adapter/docs/foundation/openai-side-conformance-suite-c13-contract.md`

This file is the durable contract anchor for `THR-13`.

## Planned Test Entrypoints And Contract Checks

`C-13` freezes the intended test surfaces that later slices must land under `gateway/tests/*`.

### Planned entrypoint files

- `gateway/tests/openai_chat_completions_conformance.rs`
- `gateway/tests/openai_responses_conformance.rs`
- `gateway/tests/openai_shared_parity.rs`

### Planned runner form

- `cargo test --manifest-path gateway/Cargo.toml --test openai_chat_completions_conformance`
- `cargo test --manifest-path gateway/Cargo.toml --test openai_responses_conformance`
- `cargo test --manifest-path gateway/Cargo.toml --test openai_shared_parity`

### Contract pass/fail conditions

- `openai_chat_completions_conformance` passes only if the suite proves:
  - sync positive cases return the contracted `chat.completion` shape
  - explicit function `tool_choice` selection narrows the forwarded tool set to the named function
  - sync negative cases reject known unsupported fields and non-function tools with the gateway error envelope
  - streaming positive cases emit `chat.completion.chunk` objects, optional usage when requested, and `[DONE]`
  - streaming negative cases fail on missing termination, wrong chunk semantics, or forbidden framing drift
  - tool-loop positive cases preserve `tool_call_id` across the continuation
  - tool-loop negative cases reject malformed or mismatched tool-loop continuations deterministically

- `openai_responses_conformance` passes only if the suite proves:
  - sync positive cases return the contracted `response` object shape
  - sync negative cases reject built-in tools, non-function tools, unsupported `text.format.type`, and malformed `call_id` values with the gateway error envelope
  - streaming positive cases emit the contracted `response.*` event subset for the stream shape being exercised, with correct `data.type` values and terminal `response.completed`
  - streaming negative cases fail on missing events, wrong event ordering, wrong `data.type` values, or provider-framing drift
  - tool-loop positive cases preserve `function_call_output.call_id` round-trip continuity
  - tool-loop negative cases reject malformed continuation input deterministically

- `openai_shared_parity` passes only if the suite proves:
  - both endpoints echo the public request `model`
  - `X-Provider` forcing behaves consistently across both endpoints
  - chain-of-thought / reasoning content never appears in public output
  - error-envelope and status mapping remain consistent across both endpoints
  - parity failures identify endpoint divergence in the shared invariants rather than hiding behind endpoint-specific snapshots

## Verification Checklist

`C-13` is complete only if a reviewer can answer yes to all of the following without reading test implementation details:

- does the suite explicitly cover `C-10`, `C-11`, and `C-12` with clause-to-test-category mapping
- does the suite name the planned fixture categories/files for both endpoint fixture namespaces
- does the suite name the planned `gateway/tests/*` entrypoints and the exact `cargo test --manifest-path gateway/Cargo.toml --test ...` forms later slices must land
- are there both positive and negative cases for the contracted rejection and tool-loop boundaries where applicable
- does the suite run offline with deterministic fixtures only
- are stream tests replayed from local normalized chunks rather than live upstream calls
- are stable ordering assertions limited to contract-promised ordering only
- are optional fields tolerated only where the contract allows variance
- do the suite boundaries avoid snapshotting provider-specific framing or incidental implementation artifacts

## Change Control

Any later expansion of the OpenAI-side subset, any change to the normalized stream model, or any change to the reject/ignore posture MUST revalidate `C-13` instead of silently widening the drift-guard surface.
