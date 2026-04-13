---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: landed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - docs/foundation/claude-code-c09-operator-bootstrap-contract.md changes the bootstrap sequence, evidence hooks, or Claude Code attachment rules that the live smoke contract depends on
    - gateway/src/router/mod.rs or gateway/src/server/mod.rs change the observable normal, think, or continuation behavior before implementation begins
    - route-evidence or redaction expectations drift enough that the contract would encode the wrong live-proof posture
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
  - C-03
  - C-04
  - C-08
  - C-09
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze The Live Session Smoke Contract And Coverage

- **User/system value**: later execution and downstream troubleshooting inherit one concrete live smoke contract instead of rediscovering scenario coverage, evidence posture, or redaction rules from scattered runtime anchors.
- **Scope (in/out)**:
  - In: define the owned `C-10` contract artifact, the three required live smoke branches, the route-evidence posture, and the minimum redacted proof set.
  - Out: landing the smoke procedure assets themselves, running live sessions, or writing the later troubleshooting ownership matrix.
- **Acceptance criteria**:
  - one canonical `C-10` landing path is named: `docs/foundation/claude-code-c10-live-session-smoke-verification-contract.md`
  - the contract states the three required live branches: normal execution, think/planner, and tool-loop continuation
  - the contract names the minimum redacted evidence posture, including required and optional evidence surfaces, without exposing provider or deployment identity as public truth
  - the contract keeps the smoke proof grounded in real Claude Code sessions above the published `C-09` bootstrap path
- **Dependencies**: `../../threading.md`, `../../governance/seam-1-closeout.md`, `docs/foundation/claude-code-c09-operator-bootstrap-contract.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/planner-executor-c04-policy-contract.md`, `docs/foundation/azure-foundry-c08-operator-verification-contract.md`
- **Verification**:
  - a reviewer can explain the three live branches and minimum redacted evidence by reading the contract alone
  - pass condition: `S2` can implement operator-facing smoke procedures without inventing new scenario or evidence semantics
- **Rollout/safety**: keep proof client-real, keep evidence redacted, and avoid teaching provider or planner/executor identity as public setup truth.
- **Review surface refs**: `review.md#r1---live-smoke-branch-coverage-that-should-land`, `review.md#r2---evidence-chain-the-live-smoke-seam-must-make-explicit`

#### S1.T1 - Freeze The Scenario Matrix

- **Outcome**: `C-10` names the required normal, think/planner, and tool-result continuation scenarios without collapsing them into one generic smoke step.
- **Inputs/outputs**: inputs from `C-09`, router/server anchors, and review surfaces; output is the scenario matrix section of `C-10`
- **Thread/contract refs**: `THR-08`, `THR-09`, `C-10`, `C-09`
- **Implementation notes**: keep the proof grounded in real Claude Code behavior rather than gateway-only probes
- **Acceptance criteria**: an operator can tell which branch they are proving and why each branch matters

#### S1.T2 - Freeze Evidence And Redaction Rules

- **Outcome**: `C-10` states which route, statusline, and optional tracing artifacts count as minimum redacted live-smoke proof.
- **Inputs/outputs**: inputs from `C-08`, `gateway/README.md`, `gateway/src/server/mod.rs`, and `gateway/src/router/mod.rs`; output is the evidence and redaction section of `C-10`
- **Thread/contract refs**: `THR-09`, `C-10`, `C-08`
- **Implementation notes**: keep statusline evidence required, tracing optional, and route labels internal/support-facing rather than public identity
- **Acceptance criteria**: a reviewer can identify the minimum proof set and its redaction rules without reading runtime code

#### S1.T3 - Freeze The Contract Artifact Path And Verification Checklist

- **Outcome**: the seam has one canonical `C-10` artifact path and a reviewer checklist that can gate execution work.
- **Inputs/outputs**: inputs from threading, review questions, and prior sections; output is the artifact-location and verification section of `C-10`
- **Thread/contract refs**: `THR-09`, `C-10`
- **Implementation notes**: the checklist should confirm scenario coverage, evidence posture, and redaction rules, not post-exec publication
- **Acceptance criteria**: `SEAM-2` can execute without reopening planning questions about what counts as live-proof truth
