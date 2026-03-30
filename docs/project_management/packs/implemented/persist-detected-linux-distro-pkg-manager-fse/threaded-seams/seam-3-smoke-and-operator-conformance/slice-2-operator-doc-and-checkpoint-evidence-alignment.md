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
    - published `C-01` or `C-02` changes canonical-path wording or field names before landing
    - published `C-03` or `C-04` changes producer-scope or reliability wording before landing
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
  - C-06
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
  - C-05
open_remediations: []
candidate_subslices: []
---
### S2 - Operator doc and checkpoint evidence alignment

- **User/system value**: one explicit wording and evidence contract keeps operators, maintainers, and pack closeout reading the same truth that the code and smoke harness prove.
- **Scope (in/out)**:
  - In: `docs/INSTALLATION.md` wording, source planning-pack checkpoint surfaces, session-log evidence alignment, and pack-closeout handoff inputs
  - Out: runtime writer mechanics, new CLI/env/log surfaces, and unrelated documentation cleanup outside the persisted-metadata scope
- **Acceptance criteria**:
  - `docs/INSTALLATION.md` names the canonical path, shared producer scope, `schema_version = 1`, and the four persisted platform fields without drift
  - checkpoint and session evidence surfaces cite the same commands and contract story the smoke harness actually proves
  - cross-platform parity is stated as compile/test evidence rather than non-Linux runtime metadata behavior
  - `REM-002` stays resolved by the landed wording and is reflected accurately in seam-exit evidence
- **Dependencies**:
  - `S1`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `../../governance/remediation-log.md`
  - `review.md`
  - `docs/INSTALLATION.md`
  - `../../../persist-detected-linux-distro-pkg-manager/plan.md`
  - `../../../persist-detected-linux-distro-pkg-manager/tasks.json`
  - `../../../persist-detected-linux-distro-pkg-manager/session_log.md`
  - `../../governance/pack-closeout.md`
- **Verification**:
  - pass condition: operator wording and checkpoint evidence consume the same landed contract and smoke truth without reopening runtime scope
  - planned evidence cross-checks the doc and checkpoint surfaces against the seam closeouts and the landed smoke contract from `S1`
- **Rollout/safety**:
  - keeps the resolved operator-wording fix visible in the final conformance evidence instead of letting it disappear into a broader bucket
  - prevents pack closeout from relying on stale commands or stale path wording
- **Review surface refs**:
  - `review.md` R2
  - `../../review_surfaces.md` R3
  - `../../review_surfaces.md` R4

#### S2.T1 - Freeze operator wording for `C-06`

- **Outcome**: installation guidance reflects the same canonical path, producer scope, schema version, and field naming that the accepted contracts and runtime seam published.
- **Thread/contract refs**:
  - `THR-02`
  - `THR-03`
  - `C-06`
- **Acceptance criteria**:
  - documentation does not drift from the published canonical path or the four platform field names
  - documentation names the shared hosted-plus-dev Linux producer scope without implying non-Linux runtime writes

#### S2.T2 - Freeze checkpoint evidence alignment

- **Outcome**: checkpoint and session evidence point at the same validation story the smoke harness and docs now encode.
- **Thread/contract refs**:
  - `THR-02`
  - `THR-03`
  - `C-06`
- **Acceptance criteria**:
  - `plan.md`, `tasks.json`, `session_log.md`, and pack-closeout inputs cite the same commands and artifacts that the landed smoke/doc work uses
  - cross-platform evidence remains parity-only and does not overstate runtime behavior

## Contract freeze for `C-06`

- `docs/INSTALLATION.md` is the authoritative operator-facing wording surface for the canonical path, shared Linux producer scope, `schema_version = 1`, and the four persisted platform fields.
- Checkpoint and session evidence surfaces cite the same commands and artifacts the smoke harness uses.
- Cross-platform parity remains compile/test evidence rather than non-Linux runtime metadata behavior.
- `REM-002` remains resolved by the landed wording and is reflected accurately in seam-exit evidence.

## Verification checklist for `C-06` readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Canonical path and field wording | `docs/INSTALLATION.md` | operator wording matches the published schema/path contract and runtime scope. |
| Producer-scope wording | `docs/INSTALLATION.md` | hosted/dev Linux producer scope is explicit and non-Linux runtime behavior is not overstated. |
| Checkpoint evidence alignment | source pack `plan.md`, `tasks.json`, `session_log.md` | commands and evidence artifacts match the smoke work that lands in `S1`. |
| Pack-closeout handoff | `../../governance/pack-closeout.md` | pack closeout receives one coherent summary of smoke, docs, and parity evidence. |

Contract-readiness for this slice is documentary: the wording and evidence contract is concrete enough to implement and later close out without reopening runtime writer or schema authority scope.
