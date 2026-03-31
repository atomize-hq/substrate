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
    - smoke assertion drift
    - manual playbook drift
    - checkpoint evidence drift
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
  - THR-03
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations:
  - REM-002
candidate_subslices: []
---
### S2 - Refresh cross-platform proof surfaces

- **User/system value**: operators and maintainers get one closeout-backed proof set that demonstrates Linux and macOS behavior claims plus Windows compile parity without overclaiming unsupported flows.
- **Scope (in/out)**:
  - In: `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`, and the paired manual/checkpoint evidence surfaces
  - Out: upstream cleanup or helper-discovery behavior changes, new platform enablement, or bundle-surface expansion
- **Acceptance criteria**:
  - Linux and macOS smoke assertions track the landed upstream contracts
  - Windows smoke remains compile parity only
  - manual and checkpoint evidence reference the same closeout-backed truth as smoke output
- **Dependencies**:
  - `slice-1-freeze-platform-evidence-boundaries.md`
  - `review.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- **Verification**:
  - pass condition: smoke and manual proof surfaces can be planned and reviewed against one current upstream truth set
- **Rollout/safety**:
  - preserve deterministic skip behavior on non-target hosts
  - record stale triggers instead of silently reusing outdated evidence
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2

#### S2.T1 - Refresh smoke and manual proof against landed upstream truth

- **Outcome**: Linux/macOS smoke and manual evidence consume the same closeout-backed `C-01`..`C-04` truth instead of recreating behavior claims from memory.
- **Files**:
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
  - `manual_testing_playbook.md`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
- **Acceptance criteria**:
  - Linux/macOS smoke reflects the landed helper-order and protected-path-refusal contracts
  - Windows smoke remains compile parity only
  - manual evidence cites the same closeout-backed contract truth as the smoke surfaces

#### S2.T2 - Freeze checkpoint-facing proof alignment

- **Outcome**: checkpoint and closeout-facing summaries point at the same smoke/manual truth that `SEAM-3` plans to land.
- **Files**:
  - checkpoint evidence surfaces
  - `../../governance/seam-3-closeout.md`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
  - `REM-002`
- **Acceptance criteria**:
  - checkpoint evidence preserves the narrowed macOS scope and Windows compile-parity posture
  - closeout-facing summaries cite the same proof surfaces as the smoke/manual work

## Proof-surface freeze

- `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`, and `manual_testing_playbook.md` are the authoritative proof surfaces for this seam.
- Checkpoint-facing evidence must summarize those same proof surfaces instead of inventing broader platform-support claims.
- `REM-002` remains a seam-local evidence-boundary reminder until closeout records the final disposition.

## Verification checklist for S2 readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Linux/macOS smoke truth | `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh` | smoke assertions match the landed helper-discovery and cleanup contracts without overclaiming scope. |
| Windows parity truth | `smoke/windows-smoke.ps1` | Windows remains explicit compile parity only. |
| Manual/checkpoint alignment | `manual_testing_playbook.md`, checkpoint evidence surfaces | manual and checkpoint summaries point at the same closeout-backed truth as smoke output. |
