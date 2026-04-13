---
slice_id: S3
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - implementation emits different normalized event semantics for explicit versus hidden tool-intent paths
    - parser code starts encoding planner/executor routing or Anthropic surface policy
    - raw Azure payload fields leak into downstream consumer contracts instead of staying debug-only
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-02
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S3 - Implement The Provider Normalization Boundary

- **User/system value**: Azure-specific parsing becomes a contained provider seam, and later seams can build on normalized events without inheriting provider quirks.
- **Scope (in/out)**:
  - In: wire the provider adapter or provider-mode boundary, implement normalization from explicit and hidden tool-intent paths into `C-02`, preserve raw payload inspection only as debug evidence, and add regression coverage against the fixture corpus.
  - Out: public Anthropic gateway delivery, planner/executor policy, or external structured-event publication.
- **Acceptance criteria**:
  - both explicit `tool_calls` and hidden `reasoning_content` markers emit the same normalized event shapes under `C-02`
  - the provider boundary records what upstream transform logic is reused and what Azure-specific logic is bypassed or extended
  - regression coverage asserts explicit-only, hidden-only, mixed, malformed, and no-tool behaviors according to the `S1` checklist and `S2` fixtures
  - raw Azure payload inspection remains available for debugging and closeout evidence without becoming a downstream dependency
- **Landed outputs**:
  - provider boundary implementation: `gateway/src/providers/openai.rs`
  - regression surface: `gateway/src/providers/openai.rs` unit tests plus `gateway/tests/fixtures/azure_kimi/*.json`
- **Dependencies**: `S1`, `S2`, `../../threading.md` (`C-01`, `C-02`, `THR-01`, `THR-02`), `docs/foundation/claude-code-mux-extension-boundary.md`, and ADR 0003
- **Verification**:
  - a reviewer can point to one provider-boundary path where Azure normalization happens and one downstream event model consumed by later seams
  - pass condition: downstream seams can plan against normalized tool/action/final events only, with regression evidence showing that explicit and hidden Azure paths converge there
  - failure conditions are explicit: parser logic that chooses models or routing roles, event outputs that differ by source path, or missing regression coverage for observed Azure variants
- **Rollout/safety**: keep any fallback or debug mode internal; do not publish a downstream contract that depends on provisional Azure quirks.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md` (`R1`, `Planned seam-exit gate focus`)

#### REM-002 Execution Checklist - Streaming Hidden-Marker Closure

- **Why this blocker exists**: the landed non-stream path in `gateway/src/providers/openai.rs` already parses hidden tool markers from `message.reasoning_content`, but the streaming transform still keeps Kimi `delta.reasoning_content` internal and therefore drops hidden marker-only streamed tool intent instead of normalizing it into the same `C-02` event path.
- **Doc/code surfaces that must change**:
  - `gateway/src/providers/openai.rs` streaming normalization around `transform_chunk` and the Kimi-specific `reasoning_content` handling
  - `gateway/src/providers/openai.rs` unit coverage for streamed hidden-marker reconstruction and stop-reason behavior
  - `gateway/tests/fixtures/azure_kimi/` with one streamed hidden-marker fixture that maps raw streaming evidence to the expected normalized output
  - `../../governance/seam-2-closeout.md` after implementation lands so `THR-02` publication can be retried against landed evidence
- **Verification plan**:
  - add one regression that feeds streamed `reasoning_content` deltas containing `<|tool_calls_section_begin|>` markers and asserts the Anthropic-facing stream emits the same `tool_use` shape as the existing non-stream hidden-marker path
  - add one regression that proves a streamed hidden-marker-only response terminates with `stop_reason = tool_use` even when the provider never emits explicit `delta.tool_calls`
  - keep a control assertion that non-marker `reasoning_content` deltas for Kimi remain internal and do not leak `thinking_delta` blocks downstream
- **Pass/fail conditions**:
  - pass when streamed hidden-marker-only Azure responses normalize into the same `C-02` `tool_intent`/`action` posture already frozen for non-stream hidden markers and explicit tool calls
  - fail when streamed hidden-marker-only responses still require downstream seams to inspect raw Azure `reasoning_content` deltas or when the streamed path invents a fourth consumer-visible event kind
- **Downstream update required after landing**: rerun `SEAM-2` seam-exit accounting in `../../governance/seam-2-closeout.md`; only retry `THR-02` publication and `SEAM-3` horizon elevation after the streamed hidden-marker evidence is recorded there

#### S3.T1 - Wire The Azure Provider Boundary

- **Outcome**: one provider-boundary implementation path owns Azure Kimi parsing and normalization.
- **Inputs/outputs**: inputs are `C-01`, the frozen `C-02` contract, and the adopted gateway baseline; output is the chosen adapter or provider-mode boundary plus a clear reuse-versus-bypass statement for inherited upstream behavior.
- **Thread/contract refs**: `THR-01`, `THR-02`, `C-01`, `C-02`
- **Implementation notes**: keep the boundary below planner/executor policy and above raw Azure transport details.

#### S3.T2 - Normalize Explicit And Hidden Tool Intent

- **Outcome**: both Azure signal paths converge into one internal event model.
- **Inputs/outputs**: inputs are the fixture corpus and normalized contract rules; output is parser behavior for explicit `tool_calls`, hidden `reasoning_content`, mixed signals, malformed markers, and no-tool responses.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: when explicit and hidden signals conflict, follow the collision rule frozen in `S1` rather than adding ad hoc precedence rules later.

#### S3.T3 - Add Regression And Drift Guards

- **Outcome**: the seam prevents hidden Azure parsing behavior from silently regressing.
- **Inputs/outputs**: inputs are the fixture corpus, expected normalized outputs, and provider implementation; output is regression coverage and drift guards tied directly to `C-02` invariants.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: the guards should fail on normalized-event drift, not only on raw payload text differences, so downstream seams stay protected even if Azure formatting changes.
