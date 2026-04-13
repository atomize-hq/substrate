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
    - docs/foundation/claude-code-c09-operator-bootstrap-contract.md changes bootstrap responsibilities or evidence hooks enough that the ownership matrix would be wrong
    - docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md or docs/foundation/claude-code-c10-live-session-smoke-procedure.md changes live failure signatures or evidence posture before implementation begins
    - gateway/README.md, gateway/src/router/mod.rs, or gateway/src/server/mod.rs change the operator-visible evidence order or route semantics before implementation begins
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
  - C-05
  - C-08
  - C-09
  - C-10
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze The Troubleshooting Boundary Contract And Taxonomy

- **User/system value**: later support work inherits one concrete troubleshooting contract instead of rediscovering ownership categories, evidence order, or redaction rules from scattered smoke and runtime anchors.
- **Scope (in/out)**:
  - In: define the owned `C-11` contract artifact, the ownership matrix, the evidence review order, and the minimum redaction-safe troubleshooting posture.
  - Out: landing the operator support guide itself, closeout publication, or broad observability work outside the troubleshooting boundary.
- **Acceptance criteria**:
  - one canonical `C-11` landing path is named: `docs/foundation/claude-code-c11-troubleshooting-and-support-boundary-contract.md`
  - the contract distinguishes Claude Code setup, gateway runtime/config, Azure transport, and broader drift without collapsing those categories
  - the contract states which evidence must be reviewed first and which evidence remains optional
  - the contract preserves the `C-05` boundary and keeps provider, deployment, and planner/executor identity internal or support-facing rather than public truth
- **Dependencies**: `../../threading.md`, `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`, `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`, `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`, `docs/foundation/claude-code-c10-live-session-smoke-procedure.md`, `docs/foundation/azure-foundry-c08-operator-verification-contract.md`, `docs/foundation/substrate-boundary-c05-contract.md`
- **Verification**:
  - a reviewer can classify failures by owner from the contract alone
  - pass condition: `S2` can implement operator-facing support surfaces without inventing new ownership or evidence semantics
- **Rollout/safety**: keep the troubleshooting story capability-oriented, keep examples redacted, and avoid turning internal routing or provider labels into public support identity.
- **Review surface refs**: `review.md#r1---failure-ownership-path-that-should-land`, `review.md#r2---evidence-review-order-the-seam-must-make-explicit`

#### S1.T1 - Freeze The Ownership Matrix

- **Outcome**: `C-11` names the required ownership branches without collapsing Claude Code setup, gateway runtime/config, Azure transport, and broader drift into one generic support bucket.
- **Inputs/outputs**: inputs from `C-09`, `C-10`, `C-08`, and review surfaces; output is the ownership-matrix section of `C-11`
- **Thread/contract refs**: `THR-08`, `THR-09`, `THR-10`, `C-11`, `C-09`, `C-10`
- **Implementation notes**: keep the matrix grounded in published evidence surfaces rather than unpublished source-reading expectations
- **Acceptance criteria**: an operator can tell who owns the next troubleshooting step and why

#### S1.T2 - Freeze Evidence Review Order And Redaction Rules

- **Outcome**: `C-11` states which bootstrap, smoke, statusline, routing-history, and optional tracing artifacts are reviewed first and how they remain redacted.
- **Inputs/outputs**: inputs from `C-09`, `C-10`, `gateway/README.md`, `gateway/src/router/mod.rs`, and `gateway/src/server/mod.rs`; output is the evidence-order and redaction section of `C-11`
- **Thread/contract refs**: `THR-09`, `THR-10`, `C-11`, `C-08`, `C-05`
- **Implementation notes**: keep statusline and `last_routing.json` primary, keep `trace.jsonl` optional, and keep provider/deployment labels support-facing only
- **Acceptance criteria**: a reviewer can identify the minimum troubleshooting evidence and its redaction rules without reading runtime code

#### S1.T3 - Freeze The Contract Artifact Path And Verification Checklist

- **Outcome**: the seam has one canonical `C-11` artifact path and a reviewer checklist that can gate support-surface delivery.
- **Inputs/outputs**: inputs from threading, review questions, and prior sections; output is the artifact-location and verification section of `C-11`
- **Thread/contract refs**: `THR-10`, `C-11`
- **Implementation notes**: the checklist should confirm ownership clarity, evidence order, and boundary rules, not post-exec publication
- **Acceptance criteria**: `SEAM-3` can execute without reopening planning questions about what counts as troubleshooting-boundary truth
