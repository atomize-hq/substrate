---
seam_id: SEAM-2
seam_slug: openai-responses-surface
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-2-openai-responses-surface.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-1-closeout.md
    - crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md
  required_threads:
    - THR-10
    - THR-12
  stale_triggers:
    - ADR 0008 contract changes for `/v1/responses` supported item types, streaming event minimum set, or tool loop requirements
    - "`SEAM-1` publishes `THR-10` with invariant deltas that require revalidation before this seam can become exec-ready"
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
# SEAM-2 - OpenAI Responses Surface

## Seam Brief (Restated)

- **Goal / value**: add `POST /v1/responses` as the preferred modern OpenAI-shaped ingress for function-tool loops and richer streaming semantics while remaining a thin adapter over the shared normalized core.
- **Type**: `capability`
- **Scope**
  - **In**:
    - implement the ADR 0008 subset request parser for `input` string shorthand plus item arrays (`message`, `function_call_output`)
    - function tools only (`tools`, `tool_choice`, `parallel_tool_calls`) and the tool loop using `function_call_output` with `call_id`
    - non-streaming Response object mapping (`object: response`, `output` item array with `message` and `function_call` items)
    - streaming SSE event mapping per the ADR 0008 minimum set, including function call argument delta/done and completion semantics
    - shared behavior aligned with `THR-10` invariants: model echo, `X-Provider` override, error envelope posture, chain-of-thought suppression
  - **Out**:
    - built-in tools and non-function tool call types
    - schema outputs and item types beyond the contracted subset
    - provider-specific streaming logic inside the public adapter
    - pack-wide conformance ownership that belongs to `SEAM-3`
- **Touch surface**:
  - `gateway/src/server/mod.rs` (route wiring + handler)
  - new adapter module(s) under `gateway/src/server/` for Responses request/response/stream transforms
  - shared core shapes and stream model in `gateway/src/core.rs` and related modules
  - tests: `gateway/tests/*` (Response object fixtures and SSE golden tests)
- **Verification**:
  - the seam is ready to execute only when the owned `C-11` rules are concrete enough that implementation does not need to guess about:
    - supported input item types and tool-loop continuation rules
    - response `output` item mapping rules (text vs function calls)
    - minimum streaming event set, required payload fields, and event ordering guarantees
    - known-but-unsupported field and built-in tool rejection posture
  - pre-exec readiness does **not** require a final accepted `C-11` artifact to already be landed; publication happens at seam-exit and closeout
  - tool streaming and event emission must consume the normalized internal stream model (per `C-12`) and must never parse provider-specific stream framing inside the public endpoint
- **Basis posture**:
  - **Currentness**: `current` (revalidated against `SEAM-1` closeout-backed `THR-10` publication)
  - **Upstream closeouts assumed**:
    - `crates/gateway/docs/project_management/packs/active/openai-side-chat-completions-and-responses/governance/seam-1-closeout.md`
    - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/governance/seam-3-closeout.md`
  - **Required threads**: `THR-10`, `THR-12`
  - **Stale triggers**:
    - ADR 0008 changes the Responses subset or the minimum streaming event set
    - `THR-10` publishes a shared invariant delta that changes tool representation or stream conversion boundaries
    - upstream `/v1/responses` behavior shifts enough that mapping assumptions must be re-planned
- **Threading constraints**
  - **Upstream blockers**: none at pre-exec; `THR-10` is already published by `SEAM-1` and revalidated here, but any later `C-12` delta would stale this basis
  - **Downstream blocked seams**: `SEAM-3`
  - **Contracts produced**: `C-11` (Responses subset contract)
  - **Contracts consumed**: `C-12` (shared adapter invariants) from `SEAM-1`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99`
- **Why this seam needs an explicit exit gate**: downstream conformance cannot assume `/v1/responses` compatibility exists just because the route is present; promotion must consume closeout-backed evidence that the Response object, tool-loop behavior, and streaming event set match the owned `C-11` contract while remaining thin over `C-12`.
- **Expected contracts to publish**: `C-11`
- **Expected threads to publish / advance**: `THR-12` from `identified` to `published`
- **Likely downstream stale triggers**:
  - `SEAM-3` if event ordering, payload fields, tool-loop semantics, or built-in tool rejection posture drift during landing
  - future Responses expansions if item types or schema-output behavior is added without a new ADR and contract update
- **Expected closeout evidence**:
  - one canonical landing artifact for the `C-11` Responses subset contract (doc or code-backed source-of-truth + tests)
  - landed handler/adapter code for `/v1/responses` plus regression coverage for sync Response objects and streaming events
  - explicit publication accounting for `THR-12`

## Slice index

- `S00` -> `slice-00-freeze-responses-contract-and-consumed-invariants.md`
- `S1` -> `slice-1-deliver-responses-request-and-sync-response-object-mapping.md`
- `S2` -> `slice-2-deliver-responses-streaming-events-and-tool-loop.md`
- `S3` -> `slice-3-lock-responses-fixtures-and-contract-publication.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
