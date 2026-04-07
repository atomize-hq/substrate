---
slice_id: S1
seam_id: SEAM-1
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
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
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
---
### S1 - Replay routing parity

- **User/system value**: `substrate replay` stops being a security-sensitive special case and uses the same routing contract as normal execution.
- **Scope (in/out)**:
  - In:
    - route replay request construction through the canonical world-network contract from `C-01`
    - remove replay-local divergence for `policy_snapshot.net_allowed` and `world_network`
    - add tests proving the four routing cases for local and agent-backed replay
  - Out:
    - tracing behavior matrix publication and WPEP validation changes
    - downstream docs-only lock-in work owned by `SEAM-3`
- **Acceptance criteria**:
  - replay request construction no longer hardcodes `net_allowed: []` or `world_network: None`
  - both local and agent-backed replay emit the same routing decision for the four canonical cases
  - test coverage names and protects the four-case matrix directly
- **Dependencies**:
  - `C-01`
  - `THR-01`
  - `crates/shell/src/execution/policy_snapshot.rs`
  - `crates/replay/src/replay/executor.rs`
- **Verification**:
  - targeted unit/integration tests around replay request construction and allowed-domain canonicalization
  - doc cross-check against `docs/REPLAY.md`
- **Rollout/safety**: preserve current public CLI behavior while eliminating routing drift that could weaken isolation semantics.
- **Review surface refs**: `../../review_surfaces.md` R2

#### S1.T1 - Wire replay onto the canonical routing helper

- **Outcome**: replay derives the same `policy_snapshot.net_allowed` and `world_network` as shell execution.
- **Inputs/outputs**:
  - Inputs: `C-01`, `resolve_world_network_policy*` helpers, replay request-construction call sites
  - Outputs: replay request builder uses the shared contract for both local and agent-backed paths
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - prefer reusing existing helper semantics over cloning them into replay
  - if replay cannot call the exact helper directly, extract the smallest shared layer that preserves one routing truth
- **Acceptance criteria**:
  - no remaining replay-only branch invents a fifth routing outcome
  - allow-all and deny-all are preserved distinctly
- **Test notes**:
  - exercise gate-disabled restrictive allowlist, gate-enabled allow-all, gate-enabled deny-all, and gate-enabled restrictive allowlist
- **Risk/rollback notes**:
  - keep helper extraction bounded to routing semantics; do not drag unrelated world-spec concerns into replay

Checklist:
- Implement: canonical helper wiring
- Test: add/extend replay parity tests
- Validate: confirm world-network fields match shell contract in all four cases
- Cleanup: remove redundant replay-local routing code

#### S1.T2 - Add replay parity regression coverage

- **Outcome**: future changes to replay or policy snapshot code cannot silently reintroduce routing drift.
- **Inputs/outputs**:
  - Inputs: replay executor tests, policy snapshot test fixtures
  - Outputs: named tests that lock the four-case matrix and the agent-backed/local parity requirement
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - keep cases table-driven where possible so the contract remains inspectable
- **Acceptance criteria**:
  - tests fail if replay restores `world_network: None` or empty `net_allowed` when that is not the canonical outcome
- **Test notes**:
  - run the targeted replay test set plus any nearby policy-snapshot tests
- **Risk/rollback notes**:
  - avoid coupling tests to transient verbose-output text when contract fields themselves are available

Checklist:
- Implement: replay parity tests
- Test: run targeted replay and policy-snapshot tests
- Validate: compare expectations to `C-01`
- Cleanup: none
