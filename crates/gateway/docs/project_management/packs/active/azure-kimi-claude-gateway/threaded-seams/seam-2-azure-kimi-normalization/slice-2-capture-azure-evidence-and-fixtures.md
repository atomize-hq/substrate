---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - Azure Foundry probes produce hidden-tool variants not represented in the fixture corpus
    - "`Kimi-K2.5` semantics diverge in a way that changes normalized output expectations for `C-02`"
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
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Capture Azure Evidence And Fixtures

- **User/system value**: the seam stops arguing from folklore and gets a durable evidence set that proves what Azure Kimi actually returns in the cases downstream seams care about.
- **Scope (in/out)**:
  - In: reproduce or refresh Azure Foundry probe cases, capture raw request/response artifacts, classify explicit and hidden tool-intent cases, and build a fixture corpus with expected normalized outputs.
  - Out: shipping public gateway behavior changes or embedding planner/executor policy into the probe process.
- **Acceptance criteria**:
  - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/threaded-seams/seam-2-azure-kimi-normalization/evidence/manifest.json` inventories one explicit `tool_calls` case, one hidden `reasoning_content` case, one mixed case, and one no-tool control case.
  - `gateway/tests/fixtures/azure_kimi/explicit-tool-calls-k2-thinking-stream.json`, `gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-nonstream.json`, `gateway/tests/fixtures/azure_kimi/mixed-reasoning-and-tool-calls-k2-thinking.json`, and `gateway/tests/fixtures/azure_kimi/no-tool-control-k2-5-stream.json` each map raw Azure evidence to expected `C-02` normalized output.
  - `crates/gateway/docs/project_management/packs/active/azure-kimi-claude-gateway/threaded-seams/seam-2-azure-kimi-normalization/evidence/variant-notes.md` records the current `Kimi-K2.5` hidden-marker observation and the streaming hidden-marker variant note for later revalidation.
  - any divergence from the assumptions in `docs/foundation/claude-code-mux-5a372fb-validation.md` is recorded as a stale-trigger candidate for seam exit
- **Landed outputs**:
  - evidence manifest: `evidence/manifest.json`
  - variant note: `evidence/variant-notes.md`
  - case evidence packs: `evidence/cases/explicit-tool-calls-k2-thinking-stream/`, `evidence/cases/hidden-markers-k2-thinking-nonstream/`, `evidence/cases/mixed-reasoning-and-tool-calls-k2-thinking/`, `evidence/cases/no-tool-control-k2-5-stream/`
  - normalized fixtures: `gateway/tests/fixtures/azure_kimi/explicit-tool-calls-k2-thinking-stream.json`, `gateway/tests/fixtures/azure_kimi/hidden-markers-k2-thinking-nonstream.json`, `gateway/tests/fixtures/azure_kimi/mixed-reasoning-and-tool-calls-k2-thinking.json`, `gateway/tests/fixtures/azure_kimi/no-tool-control-k2-5-stream.json`
- **Dependencies**: `S1`, `../../README.md`, ADR 0002, `docs/foundation/claude-code-mux-5a372fb-validation.md`, `/Users/spensermcconnell/__Active_Code/openClaw/.codex/handoffs/2026-03-27-144003-azure-kimi-claude-adapter.md`, and `/Users/spensermcconnell/__Active_Code/openClaw/.codex/handoffs/2026-03-27-141151-ccr-kimi-routing-debug.md`
- **Verification**:
  - a reviewer can trace every expected normalized result back to one raw Azure artifact
  - the fixture set makes unsupported cases explicit rather than silently omitting them
  - pass condition: the seam has enough evidence to decide whether `THR-02` can ever be published without later re-parsing Azure payloads downstream
- **Rollout/safety**: sanitize probe artifacts as needed, but do not strip away the evidence necessary to prove hidden-tool behavior.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`R2`, `Likely mismatch hotspots`)

#### S2.T1 - Reproduce And Capture Azure Response Cases

- **Outcome**: the seam has a current raw evidence set for the Azure behaviors that matter to normalization.
- **Inputs/outputs**: inputs are the handoff chain, ADR 0002, and the current Azure provider path; output is a categorized set of raw request/response artifacts covering explicit, hidden, mixed, and no-tool cases where available.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: keep native Moonshot or upstream Kimi behavior separate from Azure Foundry evidence so the seam does not overclaim what `5a372fb` proves.

#### S2.T2 - Build The Fixture Corpus And Expected Outputs

- **Outcome**: every captured case has an expected normalized result under `C-02`.
- **Inputs/outputs**: inputs are the raw Azure artifacts and the frozen contract from `S1`; output is a fixture corpus plus expected normalized outputs used by parser and regression tests.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: keep fixtures narrow and reproducible; mixed or ambiguous cases should preserve the ambiguity label if the contract intentionally treats them conservatively.

#### S2.T3 - Record Variant And Drift Notes

- **Outcome**: the seam names where future revalidation is required instead of letting variant behavior surprise downstream seams later.
- **Inputs/outputs**: inputs are the refreshed fixture set and current `5a372fb` note; output is a concise record of `Kimi-K2-Thinking` versus `Kimi-K2.5` behavior and any stale-trigger candidates for seam exit.
- **Thread/contract refs**: `THR-02`, `C-02`
- **Implementation notes**: if new variants appear, record the exact downstream surfaces affected rather than turning them into generic parser uncertainty.
