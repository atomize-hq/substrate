---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: future
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR 0008 changes the supported Chat Completions field set, tool-loop rules, or streaming semantics before execution starts
    - the normalized request, tool, or stream model changes such that the public adapter cannot remain a pure transform
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
### S00 - Freeze Chat Completions Contract And Shared Adapter Invariants

- **User/system value**: execution starts from one concrete public contract and one concrete adapter-boundary contract instead of discovering request, tool-loop, or streaming rules during implementation.
- **Scope (in/out)**:
  - In: freeze the owned `C-10` Chat Completions subset, freeze the owned `C-12` shared adapter invariants, name canonical landing artifacts, and define the verification checklist that later proves sync and stream behavior without provider-specific branching.
  - Out: landing runtime behavior, `/v1/responses`, conformance ownership for the whole pack, and seam-exit publication accounting.
- **Acceptance criteria**:
  - one canonical landing artifact path is named for `C-10`
  - one canonical landing artifact path or equivalent source-of-truth location is named for `C-12`
  - the contract names the supported roles, content-part mapping, function-tool-only rules, tool-result follow-up shape, reject/ignore matrix, sync response shape, stream chunk shape, usage handling, and `[DONE]` semantics
  - the adapter invariants state that both public OpenAI endpoints must parse into `GatewayRequest` and emit from the same normalized response/stream model without provider-specific public streaming logic
  - the verification checklist names exact code/test anchors, edge cases, and pass/fail conditions needed for `gates.pre_exec.contract`
- **Dependencies**: `../../threading.md`, `../../scope_brief.md`, `../../seam-1-openai-chat-completions-surface.md`, `gateway/src/server/mod.rs`, `gateway/src/server/openai_compat.rs`, `gateway/src/core.rs`, `gateway/src/models/mod.rs`, `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`
- **Verification**:
  - a reviewer can answer which fields are ignored versus rejected, how tool messages and function tools map into normalized structures, and how sync versus stream outputs are shaped without reading implementation diffs
  - the slice names a concrete publication target for both owned contracts and a narrow verification plan for later implementation slices
  - pass condition: `SEAM-1` can later satisfy `gates.pre_exec.contract` without waiting for closeout-backed publication to exist already
- **Rollout/safety**: keep the contract capability-oriented and thin; do not encode provider framing, internal routing policy, or `/v1/messages`-specific assumptions into the owned OpenAI-side contracts.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`R1`, `R2`, `Likely mismatch hotspots`)

#### Frozen canonical artifacts (this slice output)

- `C-10` (Chat Completions subset contract): `docs/foundation/openai-side-chat-completions-c10-contract.md`
- `C-12` (shared adapter invariants): `docs/foundation/openai-side-adapter-invariants-c12-contract.md`
- Normative policy baseline: `docs/adr/0008-expand-openai-side-support-via-chat-completions-and-responses.md`

#### S00.T1 - Freeze The Request And Tool-Loop Contract

- **Outcome**: `C-10` names one concrete request subset and tool-loop rule set for `POST /v1/chat/completions`.
- **Inputs/outputs**: inputs are ADR 0008, `threading.md`, current `openai_compat` parsing, and current core message/tool shapes; outputs are the `C-10` contract source path plus a completed request/tool-loop matrix.
- **Thread/contract refs**: `THR-11`, `C-10`
- **Implementation notes**: make explicit how `system`, `user`, `assistant`, and `tool` roles map into the normalized request, how string-or-parts content behaves, and which known-but-unsupported fields are rejected instead of ignored.
- **Acceptance criteria**: function tools only, `tool_choice`, tool-result follow-up, image URLs, and unknown-field ignore behavior are all named with exact pass/fail rules.
- **Test notes**: identify positive and negative fixtures for role parsing, content-part parsing, built-in tool rejection, and unsupported field rejection.
- **Risk/rollback notes**: if this matrix stays implicit, S1 and S2 will encode behavior that `SEAM-3` later has to reverse-engineer.

Checklist:
- Implement: freeze the request/tool-loop matrix and name the canonical `C-10` artifact location
- Test: enumerate positive and negative verification cases tied to the matrix
- Validate: confirm every supported or rejected field in ADR 0008 has an explicit disposition
- Cleanup: remove any ambiguity about skipped roles or TODO tool handling

#### S00.T2 - Freeze The Sync And Stream Output Contract

- **Outcome**: `C-10` names one concrete OpenAI completion and chunk contract over the normalized output model.
- **Inputs/outputs**: inputs are ADR 0008, current sync response shaping, current normalized stream assumptions, and `threading.md`; outputs are the sync/stream output section of `C-10`.
- **Thread/contract refs**: `THR-11`, `C-10`
- **Implementation notes**: include tool-call object mapping, `arguments` string rules, finish reasons, optional usage chunk behavior, and `[DONE]` termination.
- **Acceptance criteria**: one reviewer can explain sync choices, stream delta assembly, chunk ordering, finish reasons, usage behavior, and `[DONE]` without consulting runtime code.
- **Test notes**: name golden fixtures for text-only, tool-call-only, mixed sync output, streamed text deltas, streamed tool-call deltas, and final usage chunk placement.
- **Risk/rollback notes**: stream semantics are the highest compat risk; vague wording here will leak into endpoint-specific stream code.

Checklist:
- Implement: freeze one sync response object mapping and one stream chunk mapping
- Test: identify the exact golden fixtures and ordering assertions needed
- Validate: confirm the contract stays compatible with the normalized output model rather than raw provider SSE
- Cleanup: remove ambiguity about usage presence, finish reasons, and final termination

#### S00.T3 - Freeze The Shared Adapter Invariants

- **Outcome**: `C-12` makes the thin-adapter boundary concrete enough that `SEAM-2` and `SEAM-3` can later consume it without inferring architecture from code.
- **Inputs/outputs**: inputs are `threading.md`, `review_surfaces.md`, `gateway/src/core.rs`, `gateway/src/models/mod.rs`, and the planned `C-10` mapping; outputs are the `C-12` invariants artifact or equivalent source-of-truth section.
- **Thread/contract refs**: `THR-10`, `C-12`, `C-10`
- **Implementation notes**: state that OpenAI-facing endpoints parse into `GatewayRequest`, emit from the same normalized response/stream model, suppress chain-of-thought, and forbid provider-specific public streaming logic.
- **Acceptance criteria**: the invariants explicitly identify the conversion boundary for sync and stream paths and name which code/test anchors prove thin-adapter compliance.
- **Test notes**: identify the contract tests or regression checks that must fail if endpoint-specific engines or provider-specific public stream logic appear.
- **Risk/rollback notes**: without `C-12`, `SEAM-2` can drift into a sibling engine instead of a sibling adapter.

Checklist:
- Implement: freeze one explicit adapter-boundary rule set and name its landing location
- Test: define drift-guard assertions that detect endpoint-specific engine behavior
- Validate: confirm the invariants are strong enough for downstream seams to consume
- Cleanup: remove architecture ambiguity around stream conversion and chain-of-thought suppression
