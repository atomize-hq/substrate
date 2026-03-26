---
seam_id: SEAM-4
seam_slug: world-doctor-netfilter-status-observability
type: integration
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - seam-1-closeout.md
    - seam-2-closeout.md
    - seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - "Any change to PolicySnapshotV3/world_network requested-isolation derivation requires SEAM-4 and SEAM-5 revalidation."
    - "Any change to runtime failure taxonomy or WORLD_NETFILTER_ENABLE propagation requires SEAM-4 and SEAM-5 revalidation."
    - "Any change to doctor endpoint schema, platform rendering, or JSON field naming requires SEAM-5 revalidation."
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---

# SEAM-4 - Observability: doctor output makes enforcement status obvious

- **Goal / value**: Provide operator-visible, machine-readable diagnostics that explain whether netfilter was requested, whether it is enabled, whether the safety gate is present, and the last failure reason.
- **Scope**
  - In:
    - Extend `world doctor --json` with an additive netfilter status block that records requested vs enabled state, `WORLD_NETFILTER_ENABLE` presence, and the last actionable failure reason.
    - Keep the doctor surface aligned across world-agent output and shell-side JSON rendering so operators can distinguish host-gate, service-env, and runtime failure causes.
  - Out:
    - Implementing the enforcement itself (`SEAM-2`).
    - Core host gate semantics (`SEAM-3`) or Snapshot V3 routing ownership (`SEAM-1`) beyond consuming their published handoffs.
- **Primary interfaces**
  - Inputs:
    - Published Snapshot/WorldSpec routing semantics from `SEAM-1`.
    - Published host gate semantics from `SEAM-3`.
    - Published runtime failure taxonomy and enablement guard behavior from `SEAM-2`.
  - Outputs:
    - Doctor JSON netfilter status block (`C-07`) consumed by `SEAM-5` smoke and regression work.
- **Key invariants / rules**:
  - Diagnostics must make “requested vs enabled vs failed” explicit enough to debug “policy says deny-all but ping works” without reading implementation logs.
  - The doctor shape must remain additive and stable for existing `substrate world doctor --json` consumers.
- **Dependencies**
  - Direct blockers:
    - none; `SEAM-1`, `SEAM-2`, and `SEAM-3` now publish the handoffs this seam consumes.
  - Transitive blockers:
    - none
  - Direct consumers:
    - `SEAM-5` smoke playbooks and regression tests
- **Touch surface**:
  - `crates/agent-api-types`
  - `crates/world-agent` doctor endpoint / handlers
  - `crates/shell` world doctor JSON rendering on Linux, macOS, and Windows
- **Verification**:
  - doctor JSON shape tests and focused rendering coverage
  - manual verification during downstream smoke work once the doctor block lands
- **Risks / unknowns**:
  - Risk: the doctor surface collapses distinct request/gate/runtime states into one low-signal status.
  - De-risk plan: keep the schema explicitly split across requested/enabled/guard/failure fields and pin it with focused tests.
- **Rollout / safety**:
  - Diagnostics are additive and safe to ship ahead of broader opt-in, reducing operator ambiguity before conformance work locks behavior in.
- **Downstream decomposition context**:
  - Why this seam is `active`: the prior runtime closeout is landed and promotion-ready, so the doctor contract can now plan against recorded upstream truth instead of provisional expectations.
  - Which threads matter most: inbound `THR-01`, `THR-02`, `THR-03`, `THR-04`; outbound `THR-05`.
  - What the first seam-local review should focus on: schema actionability, platform rendering parity, and preserving the upstream failure taxonomy without ambiguity.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-07`
  - Threads likely to advance:
    - `THR-05` to `published`
  - Review-surface areas likely to shift after landing:
    - doctor JSON examples, CLI diagnostics, and downstream runbooks
  - Downstream seams most likely to require revalidation:
    - `SEAM-5`
