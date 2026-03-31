---
seam_id: SEAM-3
seam_slug: cross-platform-validation-drift-guards
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-cross-platform-validation-drift-guards.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
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
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---
# SEAM-3 - Cross-platform validation + drift guards

## Seam Brief (Restated)

- **Goal / value**: Keep the landed feature contract provable and non-drifting by aligning Linux smoke, installer smoke, manual cases, compile parity, and checkpoint evidence to the actual behavior that landed from `SEAM-1` and `SEAM-2`.
- **Type**: conformance
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
  - Consumes closeout-backed `THR-01`, `THR-02`, and `THR-03` from `SEAM-1` / `SEAM-2`.
  - Evidence must prove Linux smoke, installer smoke, manual cases, and checkpoint narratives all match the landed closeout-backed contracts without widening platform claims.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - Required threads (inbound): `THR-01`, `THR-02`, `THR-03` (revalidated against current closeout truth during promotion)
  - Stale triggers: listed in frontmatter
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-1` and `SEAM-2` now provide closeout-backed truth for every consumed contract/thread this seam needs
  - Downstream blocked seams:
    - none inside this extracted pack; this seam now feeds pack closeout and future follow-on work
  - Contracts produced:
    - none beyond finalized evidence mapping and stale-trigger capture
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: the pack cannot treat checkpoint evidence, smoke alignment, and platform-claim boundaries as trustworthy until this seam records them as closeout-backed proof.
- **Expected contracts to publish**:
  - none beyond finalized evidence mapping and stale-trigger records
- **Expected threads to publish / advance**:
  - `THR-01`, `THR-02`, and `THR-03` toward evidence-backed `revalidated` / `closed` posture
- **Likely downstream stale triggers**:
  - any change to the accepted path rule, remediation wording, selected-profile staging, or refresh semantics
  - any widened macOS provisioning promise or Windows support claim
  - any change to smoke commands, checkpoint wording, or manual playbook expectations
- **Expected closeout evidence**:
  - Linux smoke, installer smoke, manual playbook, and checkpoint artifacts all bound to the closeout-backed contracts from `SEAM-1` / `SEAM-2`
  - compile parity evidence for `linux`, `macos`, and `windows`
  - recorded stale triggers for future drift on evidence or platform claims

## Slice index

- `S1` -> `slice-1-consumed-contract-revalidation-and-evidence-boundary.md`
- `S2` -> `slice-2-linux-proof-checkpoint-and-drift-guards.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Upstream closeouts:
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
