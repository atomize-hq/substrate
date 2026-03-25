---
seam_id: SEAM-4
seam_slug: world-doctor-netfilter-status-observability
type: integration
status: proposed
execution_horizon: future
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-05
  stale_triggers:
    - "Any change to doctor endpoint schema or transport"
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-4 - Observability: doctor output makes enforcement status obvious

- **Goal / value**: Provide operator-visible, machine-readable diagnostics that explain whether netfilter was requested, whether it is enabled, whether the safety gate is present, and the last failure reason.
- **Scope**
  - In:
    - Extend world doctor JSON to include a small netfilter status block:
      - requested vs enabled
      - `WORLD_NETFILTER_ENABLE` present
      - last failure reason (if any)
  - Out:
    - Implementing the enforcement itself (`SEAM-2`).
    - Config lever details (`SEAM-3`) beyond potentially surfacing the requested state.
- **Primary interfaces**
  - Inputs:
    - World-agent internal enforcement request flags and error state.
  - Outputs:
    - doctor JSON netfilter status block.
- **Key invariants / rules**:
  - Diagnostics must be sufficient to debug “policy says deny-all but ping works” confusion.
- **Dependencies**
  - Direct blockers:
    - `SEAM-2` to define failure taxonomy and to record last failure reason.
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-5` smoke playbooks and regression tests
- **Touch surface**:
  - world-agent doctor endpoint + `substrate world doctor --json` plumbing
- **Verification**:
  - JSON shape tests (additive fields) and manual verification during smoke.
- **Risks / unknowns**:
  - Risk: missing or ambiguous failure taxonomy could lead to low-signal diagnostics.
  - De-risk plan: define a minimal set of failure classes aligned to operational actions (missing env guard, nft unavailable, resolution failure, cgroup attach failure).
- **Rollout / safety**:
  - Diagnostics are safe additive changes; should land before broad opt-in to reduce support load.
- **Downstream decomposition context**:
  - Why this seam is `future`: it depends on enforcement implementation details and benefits from discovered failure modes.
  - Which threads matter most: `THR-05`.
  - What the first seam-local review should focus on: schema stability and actionability.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-07`
  - Threads likely to advance:
    - `THR-05` to `defined`
  - Review-surface areas likely to shift after landing:
    - diagrams and operational runbooks
  - Downstream seams most likely to require revalidation:
    - `SEAM-5`

