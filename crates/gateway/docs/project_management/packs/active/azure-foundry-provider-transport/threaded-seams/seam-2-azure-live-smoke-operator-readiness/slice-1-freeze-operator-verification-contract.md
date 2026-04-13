---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - `C-07` changes the live smoke assumptions for auth, base URL, deployment mapping, or request-body invariance
    - the operator contract leaves redacted evidence or troubleshooting categories ambiguous enough that later slices would invent them during implementation
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-06
  - THR-07
contracts_produced:
  - C-08
contracts_consumed:
  - C-07
  - C-03
  - C-04
  - C-05
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze The Operator Verification Contract

- **User/system value**: future operators inherit one concrete Azure smoke-and-troubleshooting contract instead of rediscovering live verification rules from code or trial-and-error.
- **Scope (in/out)**:
  - In: define the owned `C-08` contract for redacted smoke steps, success signals, required evidence, and troubleshooting taxonomy.
  - Out: running live credentials, publishing closeout, or broad observability work beyond the bounded operator workflow.
- **Acceptance criteria**:
  - one canonical `C-08` artifact path is named for landing: `docs/foundation/azure-foundry-c08-operator-verification-contract.md`
  - the contract names one real `/v1/messages` smoke path for both `Kimi-K2-Thinking` and `Kimi-K2.5`
  - the contract explicitly names redacted evidence expectations and troubleshooting categories for auth, URL, deployment, and route failures
  - the contract preserves `C-05` capability-oriented boundary rules and does not leak planner/executor or deployment internals as public identity
- **Dependencies**: `../../threading.md`, `../../governance/seam-1-closeout.md`, `docs/foundation/azure-foundry-c07-runtime-transport-contract.md`, `docs/foundation/anthropic-messages-c03-contract.md`, `docs/foundation/planner-executor-c04-policy-contract.md`, `docs/foundation/substrate-boundary-c05-contract.md`
- **Verification**:
  - a reviewer can explain the live smoke path, redacted evidence expectations, and troubleshooting taxonomy by reading the contract alone
  - pass condition: later implementation can land operator docs and diagnostics without guessing what must be captured or how failures are categorized
- **Rollout/safety**: keep secrets redacted and keep the operator contract capability-oriented.

#### S1.T1 - Freeze The Redacted Smoke Procedure

- **Outcome**: `C-08` names the canonical live smoke path through `/v1/messages` for both Azure Kimi routes.
- **Thread/contract refs**: `THR-07`, `C-08`, `THR-06`, `C-07`

#### S1.T2 - Freeze Success Signals And Failure Taxonomy

- **Outcome**: the operator contract names the success signals and the troubleshooting categories for auth, URL, deployment, and route failures.
- **Thread/contract refs**: `THR-07`, `C-08`, `C-07`

#### S1.T3 - Freeze Evidence And Redaction Rules

- **Outcome**: the contract says what evidence must be captured and what must remain redacted.
- **Thread/contract refs**: `THR-07`, `C-08`, `C-05`
