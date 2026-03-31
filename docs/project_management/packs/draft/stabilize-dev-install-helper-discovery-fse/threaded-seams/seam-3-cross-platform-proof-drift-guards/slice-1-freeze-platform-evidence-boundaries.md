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
    - platform support wording drift
    - checkpoint wording drift
    - upstream contract wording drift
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
### S1 - Freeze platform evidence claim boundaries

- **User/system value**: smoke, playbook, and checkpoint work can execute against one explicit claim boundary instead of rediscovering how much Linux, macOS, and Windows behavior may be asserted.
- **Scope (in/out)**:
  - In: claim-boundary wording in `manual_testing_playbook.md`, `platform-parity-spec.md`, and checkpoint-facing evidence summaries
  - Out: new runtime behavior, new helper bundle assets, or changing the upstream cleanup/discovery contracts themselves
- **Acceptance criteria**:
  - macOS wording is explicit that scope is helper discovery, validation, and managed cleanup only
  - Windows wording remains compile parity only
  - checkpoint and playbook wording names the landed upstream contract surfaces `C-01`..`C-04`
- **Dependencies**:
  - `review.md`
  - `../../threading.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- **Verification**:
  - pass condition: evidence surfaces can be reviewed against one stable claim boundary without reopening platform support scope
- **Rollout/safety**:
  - refuse overclaimed platform support
  - treat wording drift as a blocker, not a documentation cleanup later
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2

#### S1.T1 - Freeze the macOS claim boundary for `REM-002`

- **Outcome**: macOS evidence surfaces stop implying release-root parity that the landed upstream seams did not publish.
- **Files**:
  - `manual_testing_playbook.md`
  - `platform-parity-spec.md`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `THR-03`
  - `REM-002`
- **Acceptance criteria**:
  - macOS wording is limited to helper discovery, validation, and managed cleanup
  - no playbook or parity text implies that all release-root assets are staged
  - any future release-root expansion is described as out-of-scope follow-on work rather than silently folded into this seam

#### S1.T2 - Freeze Windows compile-parity wording

- **Outcome**: Windows evidence remains an explicit compile-parity lane and does not drift into implied behavior support.
- **Files**:
  - `platform-parity-spec.md`
  - checkpoint-facing evidence summaries
- **Thread/contract refs**:
  - `THR-02`
  - `THR-03`
- **Acceptance criteria**:
  - Windows wording remains compile parity only
  - checkpoint-facing summaries do not imply supported `substrate world enable` behavior on Windows

## Claim-boundary freeze

- `manual_testing_playbook.md` and `platform-parity-spec.md` are the authoritative planning targets for the operator-facing macOS scope boundary.
- macOS evidence may prove helper discovery, validation, and managed cleanup only unless additional release-root assets are intentionally staged by later work.
- Windows evidence remains compile parity only.
- `REM-002` stays open until landed evidence records whether the wording drift was resolved, accepted, or carried forward, but it no longer acts as an implicit readiness blocker.

## Verification checklist for S1 readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| macOS scope wording | `manual_testing_playbook.md`, `platform-parity-spec.md` | wording is limited to helper discovery, validation, and managed cleanup. |
| release-root overclaim refusal | `manual_testing_playbook.md`, checkpoint-facing evidence summaries | no operator-facing proof surface implies that unstaged release-root assets are already covered. |
| Windows compile-parity wording | `platform-parity-spec.md`, checkpoint-facing evidence summaries | Windows remains explicit compile parity only. |
