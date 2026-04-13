---
slice_id: S2
seam_id: SEAM-5
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-02` changes normalized event semantics or stable field guarantees in a way that alters downstream structured-event assumptions"
    - "`C-03` changes public session continuation or tool-result loop rules in a way that alters event rendering expectations"
    - downstream docs or schema work starts depending on raw provider stream chunks or internal policy roles
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-04
  - THR-05
contracts_produced:
  - C-06
contracts_consumed:
  - C-02
  - C-03
  - C-05
open_remediations: []
candidate_subslices: []
---
### S2 - Deliver Structured Events And Drift Guards

- **User/system value**: later integration can consume normalized, durable events rather than raw provider chunks or one-off surface behavior.
- **Scope (in/out)**:
  - In: define the owned `C-06` downstream structured-event contract, including the event boundary, stable structure, and drift guards that keep provider transport internal.
  - Out: direct Substrate implementation, raw provider payload exposure, and public identity changes already owned by `S1`.
- **Acceptance criteria**:
  - one canonical `C-06` contract artifact path is named for landing: `docs/foundation/substrate-structured-events-c06-contract.md`
  - the contract states downstream consumers rely on normalized structured events, not raw provider transport
  - drift guards identify the public/config/doc changes that require revalidation
  - the contract keeps event-shape choices explainable from normalized gateway behavior
  - the contract names the runtime emission and verification anchors that must stay stable for downstream consumers
- **Landed outputs**:
  - contract note target: `docs/foundation/substrate-structured-events-c06-contract.md`
  - runtime anchors: `gateway/src/providers/openai.rs`, `gateway/src/providers/streaming.rs`, and `gateway/src/server/mod.rs`
  - verification anchors: `gateway/tests/fixtures/azure_kimi/explicit-tool-calls-k2-thinking-stream.json`, `gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-stream.json`, `gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-nonstream.json`, `gateway/tests/fixtures/azure_kimi/mixed-reasoning-and-tool-calls-k2-thinking.json`, and `gateway/tests/fixtures/azure_kimi/no-tool-control-k2-5-stream.json`
- **Dependencies**: `S1`, `../../threading.md`, `../../governance/seam-3-closeout.md`, `../../governance/seam-4-closeout.md`, `docs/foundation/claude-code-mux-extension-boundary.md`, `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`, `gateway/src/providers/openai.rs`, `gateway/src/providers/streaming.rs`, `gateway/src/server/mod.rs`, and the `gateway/tests/fixtures/azure_kimi/` contract fixtures
- **Verification**:
  - a reviewer can explain the downstream event boundary without reading provider parsing code
  - downstream consumers do not need raw provider chunks to understand the contract
  - drift triggers are explicit enough to force revalidation when the public boundary changes
  - pass condition: execution can land `C-06` with one stable downstream event contract over the existing normalized event surfaces and fixture-backed verification anchors
  - failure conditions are explicit: downstream behavior depends on raw SSE framing, provider-specific hidden markers, or event-shape rules that are only discoverable from implementation code
- **Rollout/safety**: keep the event boundary normalized and durable; do not let temporary adapter convenience become the downstream contract.
- **Review surface refs**: `../../review_surfaces.md` (`R2`, `R3`) and `review.md`

#### S2.T1 - Freeze The Structured-Event Contract

- **Outcome**: one owned `C-06` artifact defines the downstream event boundary in terms of normalized semantics rather than provider transport.
- **Inputs/outputs**: inputs are `C-02`, `C-03`, ADR 0007, and the extension-boundary invariants; output is `docs/foundation/substrate-structured-events-c06-contract.md` with explicit event-shape guarantees and exclusions.
- **Thread/contract refs**: `THR-05`, `C-06`, `C-02`, `C-03`
- **Implementation notes**: keep provider provenance and raw chunk sequencing debug-only, not consumer-facing contract detail.

#### S2.T2 - Anchor Runtime Emission And Adapter Surfaces

- **Outcome**: the seam names where the normalized boundary is emitted today and where future downstream adapters are allowed to attach.
- **Inputs/outputs**: inputs are the current provider streaming path, Anthropic-compatible server surface, and structured-event expectations; output is an execution checklist covering `gateway/src/providers/openai.rs`, `gateway/src/providers/streaming.rs`, and `gateway/src/server/mod.rs`.
- **Thread/contract refs**: `THR-05`, `C-06`, `C-02`
- **Implementation notes**: describe stable event meaning and attachment points without turning Anthropic SSE framing into the downstream contract.

#### S2.T3 - Add Drift Guards And Verification Anchors

- **Outcome**: later closeout can prove the event boundary stayed normalized and durable as upstream seams evolve.
- **Inputs/outputs**: inputs are the fixture corpus, the planned `C-06` note, and the seam-local stale triggers; output is explicit drift-guard language plus a verification checklist tied to the landed Azure fixture set.
- **Thread/contract refs**: `THR-05`, `C-06`, `C-02`, `C-03`
- **Implementation notes**: guards should fail on provider-framing leakage, raw marker dependence, or event-shape drift that would force Substrate-side reverse engineering.
