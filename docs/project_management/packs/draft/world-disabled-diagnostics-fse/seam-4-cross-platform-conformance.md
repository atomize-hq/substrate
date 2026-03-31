---
seam_id: SEAM-4
seam_slug: cross-platform-conformance
type: conformance
status: proposed
execution_horizon: future
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - any landed delta in SEAM-1 through SEAM-3
    - platform-specific socket/pipe/path assumptions change on Linux/macOS/Windows
    - CP1 checkpoint scope changes or smoke environment assumptions change
    - future packs touch shared diagnostics files without revalidating platform evidence
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

# SEAM-4 - Cross-platform conformance

- **Goal / value**:
  Preserve the source pack's strongest property: Linux/macOS/Windows parity backed by explicit manual and smoke evidence for disabled mode, enabled-but-broken mode, and config-resolution failure mode.
- **Scope**
  - In:
    - Manual testing playbook coverage for disabled, enabled-but-broken, and invalid-config cases
    - Platform-specific smoke scripts for Linux, macOS, and Windows
    - CP1-style checkpoint evidence that combines compile parity, feature smoke, and full CI testing expectations from the source pack
    - Revalidation of shared-file overlap with adjacent diagnostics packs before closeout
  - Out:
    - Net-new CLI runtime behavior
    - Changes to the underlying config resolver or reporting logic except as needed to consume landed evidence
    - Authoritative task graph regeneration from the source pack
- **Primary interfaces**
  - Inputs:
    - landed outputs from `SEAM-1`, `SEAM-2`, and `SEAM-3`
    - platform-specific environment setup and fixture expectations
    - smoke/playbook assertions and checkpoint boundaries from the source pack basis
  - Outputs:
    - pack-level evidence that threads are revalidated across Linux/macOS/Windows
    - closeout-ready proof that disabled/skipped semantics did not regress on non-primary platforms
- **Key invariants / rules**:
  - Same disabled/skipped meaning across Linux/macOS/Windows
  - Informational report generation still exits `0`
  - Config-resolution failures still exit `2`
  - Conformance work must consume landed truth and must not invent unpublished behavior or backfill missing contracts
- **Dependencies**
  - Direct blockers:
    - `SEAM-1`
    - `SEAM-2`
    - `SEAM-3`
  - Transitive blockers:
    - CI/runtime availability on all required platforms
    - shared-file churn from adjacent diagnostics-related packs
  - Direct consumers:
    - pack closeout and future downstream seam promotion
  - Derived consumers:
    - any future pack that assumes disabled/skipped behavior is already stable cross-platform
- **Touch surface**:
  `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`, and checkpoint evidence that mirrors the source pack's CP1 expectations.
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - For this seam specifically: seam-local review should prove the same assertions pass on Linux/macOS/Windows and that adjacent-pack overlap is revalidated before closeout.
- **Risks / unknowns**:
  - Risk: non-primary platforms can pass compile parity but still diverge in pipe/socket/path semantics.
  - De-risk plan: keep platform-native smoke scripts and explicit expectations for both disabled and enabled-but-broken modes.
  - Risk: future packs touching `health.rs` or `shim_doctor/report.rs` can invalidate evidence without an obvious contract break.
  - De-risk plan: require downstream revalidation triggers in closeout and keep stale triggers explicit.
- **Rollout / safety**:
  This seam is conformance-only. Its value is preventing drift and making later promotions depend on recorded, cross-platform truth.
- **Downstream decomposition context**:
  - Why this seam is `active`, `next`, or `future`: it has no value until upstream behavior actually lands and therefore must remain future at extraction time.
  - Which threads matter most: `THR-04`, `THR-05`.
  - What the first seam-local review should focus on: platform parity, smoke determinism, and cross-pack revalidation triggers.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: none or minimal; this seam primarily publishes evidence and thread advancement rather than new runtime contracts
  - Threads likely to advance: `THR-04`, `THR-05`
  - Review-surface areas likely to shift after landing: evidence capture, docs/examples parity notes, and downstream stale-trigger annotations
  - Downstream seams most likely to require revalidation: any future diagnostics-output, attribution, json-envelope, or provisioning seam that touches the same files
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
