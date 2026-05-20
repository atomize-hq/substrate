---
slice_id: S1
seam_id: SEAM-2
slice_kind: implementation
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - upstream `THR-01` selection truth changes after review refresh
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-03
open_remediations:
  - REM-003
---
### S1 - Land binding lookup and capability gates

- **User/system value**:
  - Makes runtime realization honor the selected backend as a fixed input instead of collapsing every supported path back to `cli:codex`.
- **Scope (in/out)**:
  - In: binding lookup, capability advertisement load, unsupported capability handling, and runtime-owned failure classification
  - Out: request/auth schema widening, config render details, artifact semantics, and downstream parity proof
- **Acceptance criteria**:
  - selected backend id maps to one runtime binding for the duration of the lifecycle action
  - missing runtime binding after shell selection succeeds surfaces as dependency unavailable rather than invalid selection
  - unsupported capability or extension failures occur before spawn and remain runtime-owned behavior
- **Dependencies**:
  - `THR-01`
  - `C-01`
  - `C-03`
- **Verification**:
  - targeted world-agent tests around backend binding lookup, unsupported capability handling, and failure-bucket separation
- **Rollout/safety**:
  - fail closed before spawn when capability requirements are not met; do not silently route unsupported backends through the current Codex-only path
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S1.T1 - Replace Codex-only binding assumptions with selected-backend lookup

- **Outcome**:
  - runtime request preparation and binding resolution consume the selected backend id directly rather than treating `cli:codex` as the only integrated binding.
- **Inputs/outputs**:
  - Inputs: `THR-01`, `docs/contracts/substrate-gateway-backend-adapter-protocol.md`, `crates/world-agent/src/service.rs`, `crates/world-agent/src/gateway_runtime.rs`
  - Outputs: binding-resolution implementation, failure-class updates, targeted tests
- **Thread/contract refs**:
  - `THR-01`
  - `C-01`
  - `C-03`
- **Implementation notes**:
  - start from prepared request handling and runtime binding resolution
  - preserve shell-owned invalid-selection behavior and make runtime-owned missing binding explicit
  - keep one selected backend id bound through sync/status/restart
- **Acceptance criteria**:
  - runtime lookup never re-derives backend selection from local defaults
  - non-selected backends cannot appear implicitly during lifecycle handling
- **Test notes**:
  - add tests for supported binding lookup and missing-binding dependency-unavailable behavior
- **Risk/rollback notes**:
  - partial binding generalization will leave the runtime with mixed Codex-only and adapter-driven branches

Checklist:
- Implement:
  - resolve runtime bindings from the selected backend id
- Test:
  - capture successful lookup and missing-binding failure classes
- Validate:
  - confirm runtime no longer treats `cli:codex` as the only integrated binding

#### S1.T2 - Land capability-gate order before spawn

- **Outcome**:
  - capability validation happens before runtime spawn and aligns to the canonical adapter protocol.
- **Inputs/outputs**:
  - Inputs: protocol/schema contracts, runtime request preparation, world-agent tests
  - Outputs: capability-gate implementation and pre-spawn failure coverage
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
- **Implementation notes**:
  - fail closed on unsupported required capabilities or extension keys before process launch
  - keep capability validation distinct from auth validation and config render
- **Acceptance criteria**:
  - unsupported capability and unsupported extension failures are deterministic and pre-spawn
  - capability outcomes remain separate from auth-source or backend-binding failures
- **Test notes**:
  - add tests for unsupported capability and extension-key rejection before spawn
- **Risk/rollback notes**:
  - weak gate ordering will make later artifact and restart work depend on undefined runtime posture

Checklist:
- Implement:
  - enforce capability and extension validation before spawn
- Test:
  - prove pre-spawn rejection for unsupported capability cases
- Validate:
  - confirm runtime-owned failure buckets stay distinct
