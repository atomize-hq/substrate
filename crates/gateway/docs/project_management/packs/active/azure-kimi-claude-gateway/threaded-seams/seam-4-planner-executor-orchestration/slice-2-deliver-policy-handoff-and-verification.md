---
slice_id: S2
seam_id: SEAM-4
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - routing implementation starts depending on raw provider parsing or public surface quirks instead of normalized events
    - session handoff or diagnostics begin leaking planner/executor role selection into public-facing behavior
    - verification cannot prove planner/executor handoff over normalized events without relying on raw provider inspection
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
### S2 - Deliver Policy Handoff And Verification

- **User/system value**: the gateway gets a usable internal planner/executor orchestration layer that respects the landed public surface and normalized core instead of turning route selection into a hidden architectural fork.
- **Scope (in/out)**:
  - In: implement or tighten route-selection policy, planning-to-execution handoff, internal diagnostics/config surfaces, and the verification surface that proves policy stays above normalization and below the public client contract.
  - Out: provider parsing, public Anthropic contract changes, external identity lock-in, and seam-exit closeout accounting.
- **Acceptance criteria**:
  - internal policy consumes landed normalized events and does not require raw Azure response inspection to decide planner/executor behavior
  - planning-to-execution handoff works through normalized session state rather than public-client-specific hacks
  - verification covers route selection, handoff, and policy diagnostics without exposing internal role truth on the public surface
  - code or docs preserve the boundary that keeps public Anthropic behavior owned by `SEAM-3` and external lock-in owned by `SEAM-5`
- **Dependencies**: `S1`, `../../threading.md`, `../../governance/seam-3-closeout.md`, `gateway/src/router/mod.rs`, `gateway/src/server/mod.rs`, and `docs/foundation/anthropic-messages-c03-contract.md`
- **Verification**:
  - a reviewer can trace one planning-to-execution handoff through normalized events and router policy without crossing into raw provider payload logic
  - pass condition: route selection and handoff are explainable in terms of `C-04` over `C-02`
  - failure conditions are explicit: provider-aware policy branches, public exposure of internal roles, or a handoff path that depends on public-client-specific state
- **Rollout/safety**: keep internal policy replaceable and avoid turning temporary diagnostics or config convenience into the architectural contract.
- **Review surface refs**: `../../review_surfaces.md` (`R1`, `R2`, `R3`) and `review.md`

#### S2.T1 - Implement Route Selection Behind Normalized Events

- **Outcome**: planner/executor routing is driven by normalized events and internal policy rules rather than provider-specific behavior.
- **Inputs/outputs**: inputs are `C-02`, landed `C-03`, and the owned `C-04` contract; outputs are router and policy changes plus any code-local documentation needed to keep the boundary explicit.
- **Thread/contract refs**: `THR-04`, `C-04`, `C-02`
- **Implementation notes**: treat provider provenance as debug-only and keep route selection above normalization.

#### S2.T2 - Prove The Planning-To-Execution Handoff

- **Outcome**: the seam has executable verification for state handoff, route decisions, and internal-only diagnostics boundaries.
- **Inputs/outputs**: inputs are the policy contract and current router/session behavior; outputs are tests, fixtures, or scripted verification paths that cover handoff success and containment of internal role truth.
- **Thread/contract refs**: `THR-04`, `C-04`
- **Implementation notes**: verification must prove the handoff without requiring reviewers to inspect raw Azure payload frames.

#### S2.T3 - Preserve Public Identity Containment

- **Outcome**: internal policy work does not create separate public backend identities or public planner/executor controls.
- **Inputs/outputs**: inputs are the landed `C-04` contract and implementation anchors; output is explicit code or doc evidence that public-facing behavior remains capability-oriented.
- **Thread/contract refs**: `THR-04`, `C-04`, `C-03`
- **Implementation notes**: diagnostics may describe internal decisions, but public-facing examples and config must stay singular and capability-oriented.
