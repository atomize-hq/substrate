---
slice_id: S3
seam_id: SEAM-3
slice_kind: seam_exit_gate
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - any upstream contract or staging rule change after SEAM-3 evidence lands
    - any widened platform claim, checkpoint requirement, or manual proof expectation
    - any shared-surface overlap that invalidates Linux smoke or installer smoke evidence
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
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
open_remediations: []
candidate_subslices: []
---
### S3 - Seam-exit gate

This slice plans the final evidence lock-in that pack closeout and future follow-on work will consume once `SEAM-3` lands.

#### Required closeout records

- Update `../../governance/seam-3-closeout.md` with:
  - Linux smoke and installer smoke disposition tied back to `THR-01`, `THR-02`, and `THR-03`
  - manual playbook and checkpoint evidence references
  - compile-parity disposition for `linux`, `macos`, and `windows`
  - stale-trigger records for future drift on runner, dev-install, platform-claim, or checkpoint surfaces
  - explicit thread-state updates for the consumed upstream threads as the evidence lock-in closes

#### Promotion readiness criteria

- `gates.post_exec.landing = passed` and `gates.post_exec.closeout = passed`
- `seam_exit_gate.status = passed`
- `seam_exit_gate.promotion_readiness = ready`
- Linux proof surfaces and checkpoint artifacts all reference closeout-backed upstream truth
- platform-claim boundaries remain explicit and narrow

#### Evidence checklist

- Linux smoke evidence bound to `C-01` / `C-02` / `C-03` / `C-04`
- installer smoke evidence with production-installer scope still bounded
- manual playbook and checkpoint evidence references
- compile-parity evidence for `linux`, `macos`, and `windows`
