---
seam_id: SEAM-3
seam_slug: cross-platform-validation-drift-guards
type: conformance
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - Linux-only behavior scope changes
    - macOS or Windows parity-only posture changes
    - smoke commands, manual playbook cases, or checkpoint evidence requirements change
    - upstream helper-discovery or provisioning work changes shared runner or installer surfaces after evidence is drafted
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  source_ref: threaded-seams/seam-3-cross-platform-validation-drift-guards/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
open_remediations: []
---

# SEAM-3 - Cross-platform validation + drift guards

This seam is closed. Its authoritative exit-gate record lives in `governance/seam-3-closeout.md`.

- **Goal / value**: Keep the landed feature contract provable and non-drifting by aligning Linux smoke, installer smoke, manual cases, compile parity, and checkpoint evidence to the actual behavior that lands from the first two seams.
- **Scope**
  - In:
    - `platform-parity-spec.md` as the authoritative platform-claim boundary
    - `manual_testing_playbook.md` cases for staging, missing-artifact failure, and success-path ordering
    - `smoke/linux-smoke.sh` and installer smoke as authoritative Linux behavior evidence
    - compile parity on `linux`, `macos`, and `windows`
    - checkpoint-boundary proof, run-id capture, and session-log / quality-gate alignment
    - stale-trigger capture when upstream contracts or platform claims drift
  - Out:
    - new runtime behavior not already owned by `SEAM-1` or `SEAM-2`
    - supported `substrate world enable` behavior on Windows
    - widened macOS provisioning guarantees
    - changing the accepted path rule, remediation wording, or staging behavior directly
- **Primary interfaces**
  - Inputs:
    - published runtime contracts from `SEAM-1`
    - published staging contracts from `SEAM-2`
    - `platform-parity-spec.md`
    - `manual_testing_playbook.md`
    - `smoke/linux-smoke.sh`
    - `tests/installers/install_smoke.sh`
    - `pre-planning/ci_checkpoint_plan.md`, `tasks.json`, `session_log.md`, and `quality_gate_report.md`
  - Outputs:
    - Linux behavior evidence that matches the landed runtime and staging contracts
    - bounded platform claims for macOS and Windows parity surfaces
    - checkpoint evidence and stale-trigger records that future follow-on work can trust
- **Key invariants / rules**:
  - Linux is the only behavior-delta platform in this feature.
  - macOS and Windows remain parity surfaces; Windows keeps exit `4` unsupported posture.
  - Checkpoint proof runs only after the full two-seam behavior contract is present.
  - Evidence must not overclaim `cargo clean` robustness, widened macOS provisioning, or any behavior outside the source contract.
  - Pack-level quality gates and checkpoint notes must stay aligned to the same acceptance surfaces as the smoke and manual playbook.
- **Dependencies**
  - Direct blockers:
    - `THR-01`, `THR-02`, and `THR-03` must all have closeout-backed truth
  - Transitive blockers:
    - checkpoint wiring drift
    - platform-scope drift in adjacent packs or ADRs
    - source-pack human-review expectations remaining unresolved at seam-local review time
  - Direct consumers:
    - pack closeout and any future follow-on work that needs a trusted evidence baseline
  - Derived consumers:
    - future helper-discovery, provisioning, macOS-parity, or Windows-enable packs that rely on stable proof surfaces and stale-trigger history
- **Touch surface**:
  - `platform-parity-spec.md`
  - `manual_testing_playbook.md`
  - `smoke/linux-smoke.sh`
  - `tests/installers/install_smoke.sh`
  - `pre-planning/ci_checkpoint_plan.md`
  - `tasks.json`
  - `session_log.md`
  - `quality_gate_report.md`
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Expected proof points:
    - Linux smoke proves the staged-path rule, missing-artifact failure, remediation content, and `world.enabled` ordering invariant
    - installer smoke remains aligned with the dev-install staging boundary and shows no unintended production-installer drift
    - manual cases 1-5 still match the landed contracts and capture exit codes, `readlink` results, and config-state expectations
    - compile parity remains green for `linux`, `macos`, and `windows`
    - checkpoint evidence records compile parity, feature smoke, and any follow-up platform-fix tasks without widening platform claims
- **Risks / unknowns**:
  - Risk: evidence can drift from actual landed contracts when upstream seams, adjacent helper-discovery work, or shared runner surfaces change.
  - De-risk plan: require revalidation against `THR-01`, `THR-02`, and `THR-03`, and treat stale evidence as a basis problem rather than silently carrying it forward.
  - Risk: parity documentation can overstate support on macOS or Windows.
  - De-risk plan: keep platform wording explicit and narrow in smoke, checkpoint, and closeout artifacts.
- **Rollout / safety**:
  - Use the checkpoint only after both behavior seams land.
  - Preserve deterministic skip / unsupported posture on non-Linux platforms.
  - Record stale triggers instead of silently reusing evidence when upstream reality changes.
- **Downstream decomposition context**:
  - Why this seam is `active`: `SEAM-1` and `SEAM-2` now provide closeout-backed truth for `THR-01`, `THR-02`, and `THR-03`, so the conformance seam can execute against landed behavior instead of provisional planning assumptions.
  - Which threads matter most: `THR-01`, `THR-02`, and `THR-03`.
  - What the seam-local review focused on: evidence-to-contract alignment, platform claim boundaries, checkpoint wiring, and stale-trigger capture.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: minimal or none beyond finalized evidence mapping and stale-trigger records.
  - Threads likely to advance: `THR-01`, `THR-02`, and `THR-03` toward `revalidated` or `closed`.
  - Review-surface areas likely to shift after landing: smoke assertions, checkpoint wording, session-log evidence summaries, and platform-scope notes.
  - Downstream seams most likely to require revalidation: any future macOS parity expansion, Windows enablement, helper-discovery hardening, or provisioning-surface change.
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
