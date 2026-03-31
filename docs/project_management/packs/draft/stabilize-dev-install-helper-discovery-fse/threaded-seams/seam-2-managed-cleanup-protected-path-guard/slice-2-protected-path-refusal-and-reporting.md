---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - refusal messaging changes
    - exit-code mapping changes
    - directory-pruning behavior changes
    - manifest schema or location changes
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
  - THR-03
contracts_produced:
  - C-04
contracts_consumed:
  - C-04
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S2 - Implement protected-path refusal and reporting

- **User/system value**: operators get deterministic refusal behavior when cleanup encounters user-managed paths, and later conformance work can trust the exit class and preserved-path reporting.
- **Scope (in/out)**:
  - In: preserved-path refusal, exit-5 classification, refusal messaging, and deterministic reporting for non-managed paths
  - Out: broadening the deletion surface, changing staging semantics, or turning cleanup into recursive tree deletion
- **Acceptance criteria**:
  - user-managed regular files at managed paths are preserved
  - non-repo-managed symlinks at managed paths are preserved
  - refusal output names the refused path deterministically
  - the cleanup path does not depend on recursive deletion as a shortcut
- **Dependencies**:
  - `slice-1-managed-only-cleanup-contract.md`
  - `review.md`
  - `scripts/substrate/dev-uninstall-substrate.sh`
- **Verification**:
  - pass condition: the cleanup path can be reviewed against one refusal-class contract without reopening path authority or reporting semantics
- **Rollout/safety**:
  - fail safe on protected paths
  - keep unmanaged paths in place even if cleanup cannot complete fully
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
