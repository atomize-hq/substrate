---
slice_id: S2
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - published bootstrap, smoke, or runtime anchors drift from the support guide before implementation begins
    - the troubleshooting boundary lands with different artifact or evidence boundaries than the delivery work assumes
    - operator-visible failure signatures change enough that the planned escalation flow would be wrong
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-08
  - THR-09
  - THR-10
contracts_produced:
  - C-11
contracts_consumed:
  - C-09
  - C-10
open_remediations: []
candidate_subslices: []
---
### S2 - Deliver Support Flow And Evidence Review Surfaces

- **User/system value**: a real operator can follow one reproducible troubleshooting flow, review the right evidence in order, and escalate failures by owner without reverse-engineering runtime code or unpublished seam history.
- **Scope (in/out)**:
  - In: land the operator-facing troubleshooting guide, bounded README/support-surface updates, evidence review checklist, and escalation guidance required to make `C-11` executable in practice.
  - Out: publishing final closeout, changing runtime code, or broad platform support work outside this pack.
- **Acceptance criteria**:
  - operator-facing surfaces make the ownership matrix explicit and tie each branch to the expected evidence review order
  - README or equivalent support surfaces align with the frozen `C-11` contract and the published `C-09` and `C-10` truth
  - the support path names which evidence is required, which evidence is optional, and how it stays redacted
  - no support step requires provider-only bypasses or teaches internal identities as public truth
- **Dependencies**: `S1`, `gateway/README.md`, `gateway/src/router/mod.rs`, `gateway/src/server/mod.rs`
- **Verification**:
  - pass condition: an operator can classify representative failures on paper from the support surfaces alone and identify the expected evidence for each branch
  - pass condition: a reviewer can map each documented ownership branch and evidence surface to an existing file or runtime anchor without consulting provider code
- **Rollout/safety**: prefer redacted examples and bounded escalation checklists over ad hoc shell history or unstructured transcripts.
- **Review surface refs**: `review.md#r1---failure-ownership-path-that-should-land`, `review.md#r2---evidence-review-order-the-seam-must-make-explicit`

#### S2.T1 - Align The Operator Troubleshooting Flow

- **Outcome**: one operator-facing guide tells the support story from published bootstrap and smoke evidence through owner classification and escalation.
- **Inputs/outputs**: inputs from `C-11`, `C-09`, `C-10`, and `gateway/README.md`; output is the aligned troubleshooting/support surface
- **Thread/contract refs**: `THR-08`, `THR-09`, `THR-10`, `C-11`
- **Implementation notes**: keep the flow capability-oriented and bounded to the current ownership matrix
- **Acceptance criteria**: operators no longer need to infer evidence order or owner boundaries from runtime code

#### S2.T2 - Surface The Evidence Review Checklist

- **Outcome**: the support assets tell operators exactly which bootstrap, smoke, statusline, routing-history, and optional trace artifacts to inspect before escalation.
- **Inputs/outputs**: inputs from `gateway/src/router/mod.rs`, `gateway/src/server/mod.rs`, `C-09`, and `C-10`; output is an aligned evidence review checklist
- **Thread/contract refs**: `THR-09`, `THR-10`, `C-11`
- **Implementation notes**: keep the evidence minimal, redacted, and sufficient for safe ownership classification
- **Acceptance criteria**: operators can tell what is required, what is optional, and how each artifact feeds the ownership matrix

#### S2.T3 - Capture Closeout Input Checklist For Support Truth

- **Outcome**: the seam records exactly which landed artifacts and support-proof points `S3` must later publish in closeout.
- **Inputs/outputs**: inputs from `C-11`, the aligned troubleshooting guide, and the evidence review checklist; output is a closeout-ready checklist for support-boundary evidence
- **Thread/contract refs**: `THR-10`, `C-11`
- **Implementation notes**: this is still delivery work, not closeout; it prepares the evidence chain without claiming publication already happened
- **Acceptance criteria**: `S3` can enumerate landed support-boundary surfaces without reopening planning questions
