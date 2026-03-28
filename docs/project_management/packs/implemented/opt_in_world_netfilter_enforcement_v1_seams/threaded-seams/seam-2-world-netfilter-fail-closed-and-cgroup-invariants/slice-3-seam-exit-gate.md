---
slice_id: S3
seam_id: SEAM-2
slice_kind: seam_exit_gate
execution_horizon: active
status: landed
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
    landing: passed
    closeout: passed
threads:
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S3 - seam-exit-gate

- **Purpose**: convert the landed runtime hardening into a downstream-consumable closeout and promotion handoff.
- **Scope (in/out)**:
  - In:
    - capture landed evidence for fail-closed errors, deny-all DNS behavior, and attach-or-fail execution paths
    - record `THR-04` publication and any downstream stale triggers for `SEAM-4` and `SEAM-5`
    - record remediation disposition and promotion readiness
  - Out:
    - net-new runtime delivery work
- **Acceptance criteria**:
  - `../../governance/seam-2-closeout.md` can be completed without ambiguity once runtime changes land.
  - `THR-04` publication is explicit and tied to concrete failure semantics around missing env guard, nft install/runtime errors, and cgroup attach failures.
  - downstream stale triggers clearly identify what would force `SEAM-4` and `SEAM-5` revalidation.
  - promotion readiness can be stated as `ready` or `blocked` from recorded evidence only.
- **Dependencies**:
  - landed work from `S1` and `S2`
  - post-exec evidence from code/tests and privileged verification
- **Verification**:
  - closeout review against `../../governance/seam-2-closeout.md`
- **Review surface refs**:
  - `../../review_surfaces.md`
  - `review.md`
- **Implementation disposition**:
  - Landed via the updated seam closeout and threading records that now cite `crates/world/src/session.rs`,
    `crates/world/src/netfilter.rs`, `crates/world/src/exec.rs`, and `cargo test -p world --lib -- --nocapture`
    as the evidence chain for fail-closed setup errors, deny-all DNS behavior, direct-exec rejection, and
    helper-path attach-or-fail semantics.
