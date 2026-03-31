---
slice_id: S3
seam_id: SEAM-2
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - downstream seams consume provisional cleanup facts instead of closeout-backed evidence
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
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S3 - Seam-exit gate

This slice plans the deterministic handoff that downstream seam promotion may consume once `SEAM-2` has landed and closeout is recorded.

#### Required closeout records

- Update `../../governance/seam-2-closeout.md` with:
  - landed managed-symlink cleanup evidence
  - landed manifest-tracked copied-binary cleanup evidence
  - landed protected-path refusal and preserved-path evidence
  - thread state updates:
    - `THR-03` -> `published`
  - contracts published or changed:
    - `C-04`
  - any review-surface delta discovered during landing, especially:
    - refusal messaging differences
    - manifest-schema differences
    - bundle-path or directory-pruning differences
  - remediation disposition:
    - record whether any cleanup-specific blocker was resolved, accepted, or carried forward

#### Promotion readiness criteria

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- `THR-03` is explicitly recorded as `published`
- the closeout records one landed managed-cleanup truth that `SEAM-3` can consume
- any downstream-relevant stale triggers are recorded before promotion:
  - refusal messaging drift
  - manifest drift
  - managed-target classification drift
  - directory-pruning drift

#### Evidence checklist

- One closeout-backed record of managed-symlink cleanup evidence
- One closeout-backed record of manifest-tracked copied-binary cleanup evidence
- One closeout-backed record of protected-path refusal behavior
- One recorded test or smoke evidence set for preserved-path refusal and reporting
- One planned-vs-landed delta note covering any review-surface movement that downstream seams must revalidate against
