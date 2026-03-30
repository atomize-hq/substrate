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
    - published `C-03` or `C-04` changes branch coverage or warning-only posture before landing
    - published `C-01` or `C-02` changes field naming or canonical path wording before landing
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-03
contracts_produced:
  - C-05
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
candidate_subslices: []
---
### S1 - Linux smoke conformance and drift guards

- **User/system value**: one explicit smoke-evidence contract keeps future changes from drifting away from the landed writer and schema truth while preserving Linux-only behavior scope.
- **Scope (in/out)**:
  - In: Linux smoke assertions for the successful-Linux producer matrix, dry-run and non-Linux no-write boundaries, exact persisted platform fields, missing-os-release degradation, additive compatibility, and checkpoint command alignment
  - Out: runtime writer behavior changes, operator-doc rewrites, and non-Linux runtime expansion
- **Acceptance criteria**:
  - the smoke harness proves the successful-Linux producer matrix published in `C-03`
  - the smoke harness proves the persisted platform fields and additive-compatibility rules consumed from `C-01` and `C-02`
  - the smoke harness keeps reliability assertions aligned to the warning-only and preservation posture published in `C-04`
  - checkpoint commands or evidence references point at the same smoke truth the seam intends to close out
- **Dependencies**:
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `review.md`
  - `tests/installers/install_state_smoke.sh`
  - `../../../persist-detected-linux-distro-pkg-manager/plan.md`
  - `../../../persist-detected-linux-distro-pkg-manager/tasks.json`
  - `../../../persist-detected-linux-distro-pkg-manager/session_log.md`
- **Verification**:
  - pass condition: the smoke harness becomes the authoritative Linux conformance surface for the landed schema, path, branch-matrix, and warning-only behavior
  - planned evidence cross-checks the smoke harness against the published seam closeouts and source checkpoint surfaces
- **Rollout/safety**:
  - keeps drift guards attached to the same contract that the runtime seam actually published
  - preserves Linux-only behavior scope while still naming cross-platform parity as evidence rather than runtime behavior
- **Review surface refs**:
  - `review.md` R1
  - `../../review_surfaces.md` R1
  - `../../review_surfaces.md` R2
  - `../../review_surfaces.md` R4

#### S1.T1 - Freeze the smoke branch matrix for `C-05`

- **Outcome**: the smoke harness names one exact successful-Linux matrix plus the matching no-write boundaries.
- **Thread/contract refs**:
  - `THR-02`
  - `THR-03`
  - `C-05`
- **Acceptance criteria**:
  - hosted install, hosted `--no-world`, dev install, and dev `--no-world` are covered as explicit Linux producer branches
  - hosted `--dry-run` and non-Linux behavior remain explicit no-write coverage rather than implied gaps

#### S1.T2 - Freeze field and degradation evidence

- **Outcome**: the smoke harness proves the exact persisted platform fields, missing-os-release degradation, and additive compatibility expected by downstream reviewers.
- **Thread/contract refs**:
  - `THR-02`
  - `THR-03`
  - `C-05`
- **Acceptance criteria**:
  - smoke assertions check the four `host_state.platform.*` fields against the published schema/path contract
  - missing-os-release and additive-compatibility cases are covered without reopening runtime ownership decisions

## Contract freeze for `C-05`

- Linux smoke is the authoritative conformance surface for the successful-Linux producer matrix and the explicit no-write boundaries.
- Linux smoke asserts the canonical persisted platform fields and additive-compatibility rules consumed from `SEAM-1`.
- Linux smoke keeps the warning-only and prior-state-preservation posture aligned to the landed `SEAM-2` reliability contract.
- Checkpoint evidence references the same commands and artifacts the smoke harness actually proves.

## Verification checklist for `C-05` readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Successful-Linux producer matrix | `tests/installers/install_state_smoke.sh` | smoke coverage explicitly proves the four Linux producer branches and their matching no-write boundaries. |
| Persisted field assertions | `tests/installers/install_state_smoke.sh` | the four `host_state.platform.*` fields match the published schema/path contract. |
| Degradation and compatibility | `tests/installers/install_state_smoke.sh` | missing-os-release and additive-compatibility cases stay aligned to the accepted contract and writer closeout. |
| Checkpoint command alignment | source pack `plan.md`, `tasks.json`, `session_log.md` | cited commands and evidence refer to the same smoke behavior that lands in this seam. |

Contract-readiness for this slice is documentary plus executable: the smoke contract is concrete enough to implement and later close out without reopening runtime writer scope.
