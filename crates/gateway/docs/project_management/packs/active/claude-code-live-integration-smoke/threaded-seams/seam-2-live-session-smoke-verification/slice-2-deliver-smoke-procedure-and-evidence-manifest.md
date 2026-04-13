---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - the published bootstrap docs or runtime anchors drift from the smoke procedure before implementation begins
    - the route-evidence posture changes in a way that invalidates the planned operator checklist
    - C-10 lands with different artifact or evidence boundaries than the delivery work assumes
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
threads:
  - THR-08
  - THR-09
contracts_produced:
  - C-10
contracts_consumed:
  - C-09
open_remediations: []
candidate_subslices: []
---
### S2 - Deliver Smoke Procedure And Evidence Manifest

- **User/system value**: a real operator can follow one reproducible live smoke procedure, capture the right redacted evidence, and explain normal, think, and continuation outcomes without reverse-engineering runtime code.
- **Scope (in/out)**:
  - In: land the operator-facing smoke procedure, redacted evidence checklist, and any bounded README or helper-surface updates required to make `C-10` executable in practice.
  - Out: running the real live sessions, publishing final closeout, or broad troubleshooting ownership work.
- **Acceptance criteria**:
  - operator-facing surfaces make the three live smoke branches explicit and tie each one to expected route/evidence outcomes
  - README or equivalent smoke-procedure surfaces align with the frozen `C-10` contract and the published `C-09` bootstrap path
  - the live smoke path names which evidence is required, which evidence is optional, and how it stays redacted
  - no smoke step requires provider-only bypasses or teaches internal identities as public truth
- **Dependencies**: `S1`, `gateway/README.md`, `gateway/src/router/mod.rs`, `gateway/src/server/mod.rs`
- **Verification**:
  - pass condition: an operator can run the three live branches on paper from the procedure alone and identify the expected evidence for each branch
  - pass condition: a reviewer can map each documented branch and evidence surface to an existing file or runtime anchor without consulting provider code
- **Rollout/safety**: prefer redacted examples and bounded checklists over ad hoc shell history or unstructured transcripts.
- **Review surface refs**: `review.md#r1---live-smoke-branch-coverage-that-should-land`, `review.md#r2---evidence-chain-the-live-smoke-seam-must-make-explicit`

#### S2.T1 - Align The Operator Smoke Procedure

- **Outcome**: one operator-facing procedure tells the live smoke story from published bootstrap through normal, think, and continuation proof.
- **Inputs/outputs**: inputs from `C-10`, `C-09`, and `gateway/README.md`; output is the aligned smoke procedure surface
- **Thread/contract refs**: `THR-08`, `THR-09`, `C-10`
- **Implementation notes**: keep the procedure client-real and bounded to the three required branches
- **Acceptance criteria**: operators no longer need to infer scenario order or route expectations from runtime code

#### S2.T2 - Surface The Redacted Evidence Manifest

- **Outcome**: the smoke assets tell operators exactly which statusline, route, transcript, and optional trace artifacts count as proof for each branch.
- **Inputs/outputs**: inputs from `gateway/src/router/mod.rs`, `gateway/src/server/mod.rs`, and `C-08`; output is an aligned evidence manifest/checklist
- **Thread/contract refs**: `THR-09`, `C-10`, `C-08`
- **Implementation notes**: keep the evidence minimal, redacted, and sufficient for later troubleshooting work
- **Acceptance criteria**: operators can tell what is required, what is optional, and how each artifact maps to the three live branches

#### S2.T3 - Capture Closeout Input Checklist For Live Proof

- **Outcome**: the seam records exactly which landed artifacts and proof points `S3` must later publish in closeout.
- **Inputs/outputs**: inputs from `C-10`, the aligned smoke procedure, and the evidence manifest; output is a closeout-ready checklist for live-proof evidence
- **Thread/contract refs**: `THR-09`, `C-10`
- **Implementation notes**: this is still delivery work, not closeout; it prepares the evidence chain without claiming live execution already happened
- **Acceptance criteria**: `S3` can enumerate landed smoke-proof surfaces without reopening planning questions
