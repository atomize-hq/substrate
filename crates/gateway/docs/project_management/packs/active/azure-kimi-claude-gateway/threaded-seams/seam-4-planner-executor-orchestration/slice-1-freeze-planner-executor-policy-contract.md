---
slice_id: S1
seam_id: SEAM-4
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "`C-03` changes public session/tool-loop guarantees or thin-adapter rules in a way that changes policy assumptions"
    - "`C-02` changes normalized tool/action/final semantics used by planner-to-executor handoff"
    - internal policy work starts exposing planner/executor identity in public contract surfaces or config examples
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
  - THR-04
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze Planner Executor Policy Contract

- **User/system value**: internal orchestration rules become concrete enough that execution can implement planner/executor policy without inventing route-selection or handoff semantics during coding.
- **Scope (in/out)**:
  - In: define the owned `C-04` internal policy contract, including route-selection rules, state handoff guarantees, diagnostics boundaries, and explicit public/internal separation.
  - Out: public Anthropic surface semantics, provider parsing, external backend identity, and post-exec closeout accounting.
- **Acceptance criteria**:
  - one canonical `C-04` contract artifact path is named for landing, such as a `docs/foundation/` policy note or equivalent internal source of truth
  - the contract names how normalized `tool_intent`, `action`, and `final` semantics feed planner/executor handoff without requiring provider-specific parsing
  - the contract states which diagnostics and config surfaces remain internal-only versus what stays public capability-oriented behavior
  - the contract keeps future downstream conformance work from reverse-engineering runtime policy code
- **Dependencies**: `../../threading.md`, `../../governance/seam-3-closeout.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/azure-kimi-c02-normalized-event-contract.md`, and `gateway/src/router/mod.rs`
- **Verification**:
  - a reviewer can explain planner/executor handoff from normalized events without reading provider parsing code
  - the policy contract makes the public/internal boundary explicit instead of leaving it as unwritten intent
  - pass condition: execution can proceed without inventing route-selection semantics during implementation
- **Rollout/safety**: keep policy internal and replaceable; do not let planner/executor naming become a public backend identity.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md`

#### S1.T1 - Freeze Route Selection Rules

- **Outcome**: one concrete rule set defines how internal policy selects planning versus execution behavior without changing the public client contract.
- **Inputs/outputs**: inputs are landed `C-03`, `C-02`, and current router anchors; output is the landed `C-04` contract source that names route-selection invariants and exclusions.
- **Thread/contract refs**: `THR-04`, `C-04`, `C-02`
- **Implementation notes**: keep provider-specific provenance out of the policy contract and keep route selection downstream of normalized event meaning.

#### S1.T2 - Freeze Session Handoff Guarantees

- **Outcome**: planning-to-execution state transfer is concrete before runtime changes begin.
- **Inputs/outputs**: inputs are current session boundaries and normalized event semantics; output is a written rule set for handoff ownership, carry-forward state, and failure boundaries.
- **Thread/contract refs**: `THR-04`, `C-04`
- **Implementation notes**: keep handoff semantics internal and avoid making public clients reason about internal role selection.

#### S1.T3 - Freeze Diagnostics And Public Boundary Rules

- **Outcome**: diagnostics and config surfaces can help operators without leaking internal role truth into the public gateway identity.
- **Inputs/outputs**: inputs are the owned `C-04` plan and landed `C-03` boundary; output is an explicit boundary note inside the `C-04` contract.
- **Thread/contract refs**: `THR-04`, `C-04`, `C-03`
- **Implementation notes**: diagnostics may expose internal decisions for debugging, but public docs and examples must stay capability-oriented.
