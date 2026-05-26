---
slice_id: S1
seam_id: SEAM-4
slice_kind: conformance
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
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
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
open_remediations: []
---
### S1 - Manual validation and owner-surface audit

- **User/system value**: manual verification becomes trustworthy because every operator-visible surface maps back to one landed contract owner.
- **Scope (in/out)**:
  - In:
    - `manual_testing_playbook.md`
    - one-owner-per-surface assertions
    - explicit mapping from playbook checks to upstream contracts
  - Out:
    - new command semantics
    - new schema or policy wording
    - runtime transport changes
- **Acceptance criteria**:
  - playbook assertions map cleanly to `C-01` through `C-04`
  - no operator-visible surface claims a second owner
  - stale archived references are removed or explicitly tracked
- **Dependencies**:
  - `review.md`
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
  - `../../governance/seam-3-closeout.md`
  - `docs/contracts/gateway/operator-contract.md`
  - `docs/contracts/gateway/status-schema.md`
  - `docs/contracts/gateway/policy-evaluation.md`
  - `docs/contracts/gateway/runtime-parity.md`
- **Verification**:
  - pass condition: the manual playbook can be read as a contract-consumption checklist without inventing new ownership language
- **Rollout/safety**:
  - treat stale operator wording as a conformance bug even when runtime behavior is correct
- **Review surface refs**: `review.md` R1
