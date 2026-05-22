---
slice_id: S3
seam_id: SEAM-2
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - any change to selected-profile mapping or staged-link paths
    - any change to `ln -sfn` refresh semantics or disabled-world posture
    - any change that broadens production-installer scope beyond regression-only status
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-04
contracts_consumed:
  - C-01
  - C-03
open_remediations: []
candidate_subslices: []
---
### S3 - Seam-exit gate

This slice plans the closeout handoff that `SEAM-3` will consume once `SEAM-2` lands.

#### Required closeout records

- Update `../../governance/seam-2-closeout.md` with:
  - landed evidence for selected-profile staging and both accepted link locations
  - proof that reruns refresh links with `ln -sfn`
  - disabled-world and no-provisioning evidence for the `--no-world` path
  - installer-smoke disposition and the production-installer regression-only boundary
  - thread state update: `THR-03` -> `published`
  - `seam_exit_gate.status` and `seam_exit_gate.promotion_readiness`

#### Promotion readiness criteria

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- `THR-03` is explicitly recorded as `published`
- downstream stale triggers are recorded for path, profile, refresh, and scope changes

#### Evidence checklist

- staged-link evidence for both accepted paths
- selected-profile mapping proof for debug and release
- disabled-world / no-provisioning evidence
- installer-smoke evidence with production-installer scope still reference-only
