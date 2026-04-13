---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-02` changes tool/action/final semantics or `source_origin` guarantees in a way that changes the owned public block mapping"
    - public contract work starts encoding planner/executor or provider-specific distinctions instead of a capability-oriented client surface
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-03
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze Anthropic Messages Surface Contract

- **User/system value**: the first public client contract becomes concrete enough that execution can implement Claude Code-compatible behavior without inventing surface semantics or re-parsing provider quirks.
- **Scope (in/out)**:
  - In: define the owned `C-03` Anthropic Messages contract, including request/response mapping, streaming content-block semantics, tool-use and tool-result loop rules, session continuation rules, and the explicit boundary that keeps future Responses work a thin outer adapter.
  - Out: Azure provider parsing, raw Kimi normalization, planner/executor routing policy, public backend identity lock-in, and post-exec closeout accounting.
- **Acceptance criteria**:
  - one canonical `C-03` contract artifact path is named for landing, such as a `docs/foundation/` note or equivalent public-surface source of truth
  - the contract maps landed normalized `tool_intent`, `action`, and `final` semantics into Anthropic `tool_use`, `thinking`, `text`, and stop/continue behavior without exposing raw Azure payload structure
  - the contract states session/tool-result continuation rules and which route/model metadata remain public versus internal-only
  - the contract keeps future OpenAI Responses support on the same normalized core instead of introducing a second execution engine
  - the contract is concrete enough that `SEAM-5` can later consume `C-03` without reverse-engineering runtime code
- **Dependencies**: `../../threading.md` (`C-01`, `C-02`, `C-03`, `THR-01`, `THR-02`, `THR-03`), `docs/foundation/claude-code-mux-extension-boundary.md`, `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, `docs/adr/0004-prioritize-anthropic-messages-while-keeping-openai-responses-easy-later.md`, and `docs/adr/0007-integrate-via-normalized-structured-events-not-raw-provider-streams.md`
- **Verification**:
  - a reviewer can explain how normalized internal events become Anthropic content blocks and stop semantics without reading provider-specific code
  - the public contract makes the Responses-later boundary explicit rather than leaving it as an unstated intent
  - pass condition: execution can proceed without inventing new public semantics during implementation
- **Rollout/safety**: keep the contract capability-oriented; do not let public surface semantics expose provider identity, raw provider framing, or internal model-role selection.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`) and `review.md` (`R1`, `R2`, `Likely mismatch hotspots`)

#### S1.T1 - Freeze The Public Block Mapping

- **Outcome**: the seam owns one concrete mapping from normalized `C-02` events to Anthropic `tool_use`, `thinking`, `text`, and stop semantics.
- **Inputs/outputs**: inputs are `C-01`, `C-02`, and the current server/provider anchors; output is the landed `C-03` contract source that names block mapping and stop/continue rules.
- **Thread/contract refs**: `THR-03`, `C-03`, `C-02`
- **Implementation notes**: keep provider-specific provenance out of the public contract and use normalized event meaning as the only source for block mapping.

#### S1.T2 - Freeze Session And Tool-Result Loop Rules

- **Outcome**: Claude Code continuation behavior is concrete before execution starts.
- **Inputs/outputs**: inputs are the block mapping, current `/v1/messages` flow, and tool-use expectations; output is a written rule set for session carry-forward, tool-result follow-up, and final response completion.
- **Thread/contract refs**: `THR-03`, `C-03`
- **Implementation notes**: keep session semantics surface-level and do not move provider parsing or internal routing policy into the owned public contract.

#### S1.T3 - Freeze The Thin-Adapter Boundary

- **Outcome**: later OpenAI Responses work remains an adapter seam instead of a second engine.
- **Inputs/outputs**: inputs are ADR 0004, `C-01`, `C-02`, and the owned `C-03` plan; output is an explicit boundary note inside the `C-03` contract that preserves a client-agnostic core.
- **Thread/contract refs**: `THR-03`, `C-03`, `C-01`, `C-02`
- **Implementation notes**: phrase the boundary in terms of normalized-core reuse and public adapter responsibilities, not in terms of provider internals.
