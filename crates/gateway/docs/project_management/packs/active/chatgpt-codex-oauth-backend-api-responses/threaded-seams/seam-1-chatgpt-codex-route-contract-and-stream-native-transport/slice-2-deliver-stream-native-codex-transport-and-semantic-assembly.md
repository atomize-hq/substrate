---
slice_id: S2
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the route contract changes semantic event authority, continuation legality, or sync-drain failure posture after this slice starts
    - the normalized stream model changes such that Codex semantic assembly can no longer remain a pure transform over normalized output
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-14
contracts_produced:
  - C-14
contracts_consumed: []
open_remediations: []
---
### S2 - Deliver Stream-Native Codex Transport And Semantic Assembly

- **User/system value**: sync and streaming callers observe one consistent Codex route truth, with semantic output assembled from the stream event family and deterministic failures on transport drift instead of partial success.
- **Scope (in/out)**:
  - In: consume the upstream semantic event family for both sync and streaming, assemble text and tool-call arguments from semantic events rather than the completed envelope, enforce provenance-based continuation synthesis and ordering, and make malformed or truncated sync drains fail with `502 transport_drift`.
  - Out: auth-handoff ownership, pack-wide regression ownership, and post-exec publication accounting.
- **Acceptance criteria**:
  - sync and streaming share one semantic event source and one continuation rule set
  - sync success requires terminal `response.completed`
  - malformed or truncated sync drains fail with the normal gateway error envelope using `class = "transport_drift"` and status `502`
  - orphaned tool-result continuations reject before the upstream call unless authoritative normalized provenance exists
  - encrypted reasoning items remain internal transport state and never become public OpenAI-visible output on this route
- **Dependencies**: `S00`, `S1`, `crates/gateway/src/providers/openai.rs`, `crates/gateway/src/providers/streaming.rs`, `crates/gateway/src/server/openai_responses.rs`, `THR-14`
- **Verification**:
  - deterministic tests prove semantic event assembly for text-only, tool-call-only, and mixed streams
  - negative tests prove truncated sync drains and orphaned tool-result continuations fail deterministically
  - pass condition: a reviewer can explain sync and stream behavior without relying on `response.completed.response.output` or raw provider framing as the answer source
- **Rollout/safety**: keep continuation repair minimal and provenance-based; do not synthesize placeholder tool metadata or surface internal reasoning content.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`Likely mismatch hotspots`)

#### S2.T1 - Assemble Output From The Semantic Event Family

- **Outcome**: the provider treats semantic events as the route source of truth for sync and streaming output assembly.
- **Inputs/outputs**: inputs are `C-14`, current SSE parsing, and normalized stream behavior; outputs are event-assembly updates for sync drain and streaming transform logic.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: make `response.output_item.*`, `response.content_part.*`, `response.output_text.*`, and `response.function_call_arguments.*` authoritative; treat `response.completed` as lifecycle and usage truth only.
- **Acceptance criteria**: text, tool-call, and mixed streams assemble deterministically from semantic events and stay consistent between sync drain and streaming transforms.
- **Test notes**: add fixtures and assertions for text-only, tool-only, mixed, and reasoning-bearing upstream streams.
- **Risk/rollback notes**: if semantic assembly stays envelope-driven, sync behavior will continue to return apparently valid but partial output.

Checklist:
- Implement: update sync and streaming assembly to consume the semantic event family
- Test: add deterministic text, tool, mixed, and reasoning-bearing fixtures
- Validate: confirm `response.completed` is no longer treated as the assembled-answer source
- Cleanup: remove stale envelope-trusting assumptions

#### S2.T2 - Enforce Continuation Provenance And Sync-Drain Failure Rules

- **Outcome**: continuation threading and sync failure posture are explicit, deterministic, and shared across route paths.
- **Inputs/outputs**: inputs are normalized continuation history, current provider/tool state, and `C-14`; outputs are continuation-repair logic and sync failure-classification updates.
- **Thread/contract refs**: `THR-14`, `C-14`
- **Implementation notes**: synthesize missing `function_call` items only from authoritative normalized provenance, insert them immediately before matching `function_call_output`, preserve normalized-history order, and classify malformed sync drains as `transport_drift`.
- **Acceptance criteria**: orphaned tool results reject cleanly, synthesized continuations never invent placeholder metadata, and malformed sync drains cannot return partial success.
- **Test notes**: add coverage for synthesized continuation order, duplicate or orphaned tool results, truncated streams, and missing terminal completion.
- **Risk/rollback notes**: weak continuation rules will make downstream auth and conformance planning consume ambiguous route behavior.

Checklist:
- Implement: enforce provenance-based continuation repair and sync failure classification
- Test: cover orphaned/duplicate tool results, synthesis order, truncated drains, and missing completion
- Validate: confirm encrypted reasoning stays non-public even when present in the upstream stream
- Cleanup: remove placeholder synthesized tool metadata and partial-success fallbacks
