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
  stale_triggers:
    - "Any installer or world-agent service configuration change affecting WORLD_NETFILTER_ENABLE"
    - "Any change to nftables ruleset shape or DNS handling for deny-all"
    - "Any new world execution path or weaker attach-or-fail behavior under isolate_network"
    - "Any new enforcement failure class or diagnostic wording change"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: passed
    closeout: failed
threads:
  - THR-04
contracts_produced: []
contracts_consumed:
  - C-02
  - C-03
open_remediations:
  - REM-005
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
  - Landed as a blocked closeout: the runtime hardening is present in code and unit coverage, but `THR-04` is not yet publishable because no privileged Linux verification artifact is recorded in `../../governance/seam-2-closeout.md`.
