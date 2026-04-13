---
seam_id: SEAM-1
seam_slug: openai-chat-completions-surface
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-1-openai-chat-completions-surface.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md
    - docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
  required_threads:
    - THR-10
    - THR-11
  stale_triggers:
    - ADR 0008 changes the supported `POST /v1/chat/completions` subset, streaming semantics, or known-but-unsupported field posture
    - "`gateway/src/core.rs` or shared tool/content models change in a way that forces endpoint-specific request or streaming logic"
    - provider-normalized output changes in a way that invalidates the planned OpenAI chunk transform, tool-call delta assembly, or chain-of-thought suppression rules
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
# SEAM-1 - OpenAI Chat Completions Surface

## Seam Brief (Restated)

- **Goal / value**: deliver a real `POST /v1/chat/completions` compatibility surface so OpenAI-shaped SDKs and integrations can target the gateway without adopting `/v1/messages`, while keeping the endpoint a thin adapter over the shared normalized core.
- **Type**: `capability`
- **Scope**
  - **In**:
    - freeze the owned `C-10` compatibility contract and owned `C-12` shared adapter invariants tightly enough that implementation can proceed without inventing request, tool-loop, or streaming behavior during coding
    - expand request parsing to cover the ADR 0008 subset: roles, string-or-parts content, image URLs, tool messages, function tools only, and `tool_choice`
    - deliver non-stream and stream output shaping for OpenAI Chat Completions, including single-choice behavior, tool-call mapping, finish reasons, usage handling, SSE chunk sequencing, and `[DONE]`
    - preserve shared behavior across the route: request `model` echo, `X-Provider` override, contracted error envelope, and chain-of-thought suppression
  - **Out**:
    - public `/v1/responses` work
    - built-in tools, non-function tool types, structured outputs, audio, `logprobs`, `n > 1`, and other ADR-scoped unsupported fields
    - provider-specific streaming logic in public adapters
    - conformance ownership that belongs to `SEAM-3`
- **Touch surface**:
  - `gateway/src/server/mod.rs` route wiring, sync/stream handler flow, and error-envelope behavior for `/v1/chat/completions`
  - `gateway/src/server/openai_compat.rs` request parsing plus sync/stream response transforms
  - shared core anchors in `gateway/src/core.rs` and `gateway/src/models/mod.rs` where the OpenAI surface must stay aligned with normalized request, tool, and content semantics
  - verification surfaces in `gateway/src/server/mod.rs`, `gateway/src/providers/openai.rs`, and `gateway/tests/` or equivalent fixture locations
- **Verification**:
  - the seam is ready to execute only when the owned `C-10` and `C-12` rules are concrete enough that implementation does not need to guess about rejected fields, tool message mapping, tool-call argument encoding, or streaming chunk sequencing
  - execution must prove `chat/completions` remains a pure transform over the normalized request/response model instead of becoming a second execution engine or a provider-shaped stream path
  - verification must cover sync text, tool-call-only and mixed paths, streaming text and tool-call deltas, usage-chunk handling, `[DONE]`, model echo, and the contracted error envelope
  - publication of the final contract artifacts is seam-exit work; pre-exec readiness depends on seam-local concreteness, not on closeout-backed publication existing already
- **Basis posture**:
  - **Currentness**: `current`
  - **Upstream closeouts assumed**: `docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-2-closeout.md`, `docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md`
  - **Required threads**: `THR-10`, `THR-11`
  - **Stale triggers**:
    - ADR 0008 changes the supported Chat Completions subset or the reject/ignore posture
    - the normalized core stops supporting a thin transform for tool-use and final-response semantics
    - provider-normalized stream shapes change enough that OpenAI chunk emission, tool-call deltas, or usage-chunk ordering must be re-planned
- **Threading constraints**
  - **Upstream blockers**: no in-pack direct blocker; the planning basis depends on the published external normalized-core and Anthropic-surface closeouts remaining current
  - **Downstream blocked seams**: `SEAM-2`, `SEAM-3`
  - **Contracts produced**: `C-10`, `C-12`
  - **Contracts consumed**: none in-pack; this seam relies on upstream closeout-backed normalized-core behavior from the `azure-kimi-claude-gateway` pack

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream work cannot safely assume `chat/completions` compatibility exists just because the route is present; promotion must consume closeout-backed evidence that the endpoint now publishes `C-10`, freezes `C-12`, and keeps streaming plus tool-loop behavior thin over the normalized core.
- **Expected contracts to publish**: `C-10`, `C-12`
- **Expected threads to publish / advance**: `THR-10` from `defined` to `published`; `THR-11` from `defined` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-2` if the shared adapter invariants move, especially around normalized tool representation, stream conversion boundaries, or chain-of-thought suppression
  - `SEAM-3` if chunk ordering, tool-call delta assembly, finish reasons, or rejected-field behavior differs from the planned contract
- **Expected closeout evidence**:
  - one canonical landing artifact for the `C-10` Chat Completions contract
  - one canonical landing artifact or equivalent source of truth for `C-12` shared adapter invariants
  - landed handler/adapter code plus regression coverage for sync and stream behavior
  - explicit publication accounting for `THR-10` and `THR-11`

## Slice index

- `S00` -> `slice-00-freeze-chat-completions-contract-and-shared-adapter-invariants.md`
- `S1` -> `slice-1-deliver-chat-completions-request-and-sync-response-mapping.md`
- `S2` -> `slice-2-deliver-chat-completions-streaming-and-tool-loop.md`
- `S3` -> `slice-3-lock-chat-completions-fixtures-and-drift-guards.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
