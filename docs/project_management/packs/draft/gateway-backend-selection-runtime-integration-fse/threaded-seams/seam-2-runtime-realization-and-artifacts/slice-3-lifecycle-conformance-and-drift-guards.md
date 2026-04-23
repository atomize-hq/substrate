---
slice_id: S3
seam_id: SEAM-2
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - runtime launch, readiness, or restart semantics change after S2
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
contracts_produced: []
contracts_consumed:
  - C-03
  - C-04
open_remediations: []
---
### S3 - Lock lifecycle conformance and drift guards

- **User/system value**:
  - Ensures sync/status/restart keep honoring the selected backend and the new runtime artifact contract once S1 and S2 land.
- **Scope (in/out)**:
  - In: lifecycle restart semantics, readiness behavior, regression tests, and implementation notes required for closeout
  - Out: new runtime protocol or schema invention, parity proof, and rollout governance
- **Acceptance criteria**:
  - sync/status/restart preserve the selected backend contract after S1 and S2 land
  - readiness and restart outcomes stay explicit for supported, unavailable, and unsupported-backend cases
  - drift-guard tests cover the intended runtime behavior without widening unrelated CLI/status surfaces
- **Dependencies**:
  - `S1`
  - `S2`
  - `THR-02`
  - `C-03`
  - `C-04`
- **Verification**:
  - runtime and shell regression tests covering sync/status/restart, explicit unsupported-backend handling, and managed artifact/readiness behavior
- **Rollout/safety**:
  - do not let restart or readiness silently reintroduce Codex-only assumptions after earlier slices land
- **Review surface refs**:
  - `../review.md`
  - `../../review_surfaces.md`

#### S3.T1 - Prove lifecycle actions preserve the selected backend

- **Outcome**:
  - sync, status, and restart all operate against the same selected backend and runtime artifacts after the earlier slices generalize the path.
- **Inputs/outputs**:
  - Inputs: landed S1/S2 behavior, shell tests, runtime tests
  - Outputs: regression coverage and any required runtime lifecycle fixes
- **Thread/contract refs**:
  - `THR-02`
  - `C-03`
  - `C-04`
- **Implementation notes**:
  - preserve the selected backend id across restart rather than re-deriving behavior from local defaults
  - keep unsupported-backend and dependency-unavailable behavior explicit
- **Acceptance criteria**:
  - lifecycle actions cannot silently fall back to the old Codex-specific path
  - restart semantics remain deterministic with managed artifacts in place
- **Test notes**:
  - add or refresh sync/status/restart coverage that exercises the selected backend contract end to end
- **Risk/rollback notes**:
  - weak lifecycle coverage will leave closeout unable to prove what `SEAM-3` is supposed to consume

Checklist:
- Implement:
  - lock lifecycle actions to the selected-backend runtime path
- Test:
  - cover sync, status, and restart on the generalized path
- Validate:
  - confirm selected-backend continuity after restart

#### S3.T2 - Capture runtime drift guards for closeout

- **Outcome**:
  - the seam lands the tests and implementation notes that closeout needs to publish `THR-02`.
- **Inputs/outputs**:
  - Inputs: landed runtime behavior, tests, supporting ADR-0046 implementation notes
  - Outputs: drift guards and closeout-ready evidence targets
- **Thread/contract refs**:
  - `THR-02`
- **Implementation notes**:
  - prefer deterministic runtime and shell tests over broad prose
  - supporting ADR-0046 docs remain subordinate implementation notes
- **Acceptance criteria**:
  - closeout can cite stable evidence targets for lookup, auth, artifact, and lifecycle behavior
  - downstream parity planning no longer needs to infer runtime semantics from code archaeology
- **Test notes**:
  - verify named evidence targets match landed files and commands
- **Risk/rollback notes**:
  - weak drift guards will cause `SEAM-3` promotion to depend on inference again

Checklist:
- Implement:
  - record the evidence surfaces `SEAM-2` closeout will need
- Test:
  - confirm evidence targets line up with landed runtime behavior
- Validate:
  - ensure `SEAM-3` can consume one explicit runtime handoff
