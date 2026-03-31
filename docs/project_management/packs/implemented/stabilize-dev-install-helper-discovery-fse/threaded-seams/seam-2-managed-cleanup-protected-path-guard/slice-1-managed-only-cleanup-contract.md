---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - manifest location or schema changes
    - repo-managed symlink ownership rules change
    - fixed bundle path list changes
    - protected-path exit-code taxonomy changes
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
contracts_produced:
  - C-04
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze the managed-only cleanup contract

- **User/system value**: downstream cleanup and conformance work can plan against one explicit managed-cleanup boundary instead of inferring deletion authority from implementation shortcuts.
- **Scope (in/out)**:
  - In: repo-managed symlink deletion, manifest-tracked copied Linux guest binary deletion, protected-path refusal semantics, and preserved-path reporting
  - Out: staging implementation changes, cross-platform evidence, and any recursive delete authority outside the fixed bundle surface
- **Acceptance criteria**:
  - `C-04` names the exact managed-only cleanup boundary and refusal class
  - the contract distinguishes provenance from location for symlinks and copied binaries
  - the contract keeps user-managed paths out of deletion authority even when they sit under a managed path prefix
- **Dependencies**:
  - `review.md`
  - `../../threading.md`
  - `../../governance/seam-1-closeout.md`
- **Verification**:
  - pass condition: `scripts/substrate/dev-uninstall-substrate.sh` can be planned against one explicit contract set without reopening ownership or refusal-class questions
- **Rollout/safety**:
  - preserve user-managed destinations
  - do not widen deletion authority beyond the published managed surface
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
