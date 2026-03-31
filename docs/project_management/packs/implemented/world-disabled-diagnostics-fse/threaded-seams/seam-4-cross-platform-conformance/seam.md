---
seam_id: SEAM-4
seam_slug: cross-platform-conformance
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-cross-platform-conformance.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - any landed delta in SEAM-1 through SEAM-3 that changes disabled-mode status, omission, or exact-copy contracts
    - platform-specific socket, pipe, or path assumptions change on Linux/macOS/Windows before conformance evidence is captured
    - scripts/mac/smoke.sh, scripts/windows/wsl-smoke.ps1, or the Linux world/health doctor workflow drift without synchronized revalidation
    - future packs touch health.rs, shim_doctor/report.rs, shim_doctor/output.rs, or docs/USAGE.md before closeout
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
# SEAM-4 - Cross-platform conformance

## Seam Brief (Restated)

- **Goal / value**: prove that the landed disabled-aware diagnostics contracts hold on Linux, macOS, and Windows so pack closeout and future downstream work can rely on one cross-platform truth instead of Linux-only evidence.
- **Type**: conformance
- **Scope**
  - In:
    - record Linux, macOS, and Windows evidence for disabled mode, enabled-but-broken mode, and invalid-config fail-fast behavior
    - align repo-native smoke and doctor workflows with the landed `C-02` through `C-05` contracts
    - revalidate shared-file overlap before closeout so future packs inherit explicit stale triggers instead of implicit risk
  - Out:
    - net-new CLI runtime behavior
    - changes to the underlying config resolver or report builders beyond consuming landed evidence
    - speculative task-graph expansion beyond the seam-local slices below
- **Touch surface**: `crates/shell/tests/shim_doctor.rs`, `crates/shell/tests/shim_health.rs`, `scripts/mac/smoke.sh`, `scripts/windows/wsl-smoke.ps1`, Linux `substrate world doctor` / `substrate shim doctor` / `substrate health` proof commands, and the closeout evidence surfaces that summarize those runs.
- **Verification**:
  - preserve the targeted Linux regression proof in `shim_doctor.rs` and `shim_health.rs`
  - run platform-native smoke or doctor workflows on Linux, macOS, and Windows with explicit disabled, enabled-but-broken, and invalid-config assertions
  - record enough evidence that downstream reviewers can trace each platform result back to the landed `C-02` through `C-05` contracts
- **Basis posture**:
  - Currentness: current (revalidated against `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`, and `../../governance/seam-3-closeout.md`)
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`, `../../governance/seam-3-closeout.md`
  - Required threads: `THR-02`, `THR-03`, `THR-04`, `THR-05`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers resolved: `THR-02`, `THR-03`, `THR-04`, and `THR-05` are published and revalidated for this seam
  - Downstream blocked seams: pack closeout and any future diagnostics packs that assume cross-platform evidence exists
  - Contracts produced: none; this seam advances evidence and thread confidence rather than introducing new runtime contracts
  - Contracts consumed: `C-02`, `C-03`, `C-04`, `C-05`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: it must convert repo-native Linux/macOS/Windows evidence into a downstream-consumable closeout record for `THR-04` and `THR-05`.
- **Expected contracts to publish**: none; this seam closes the loop on already published contracts
- **Expected threads to publish / advance**: `THR-04`, `THR-05`
- **Likely downstream stale triggers**: smoke-script drift, doctor-workflow drift, or shared diagnostics-file churn that invalidates platform evidence without changing the contract text itself
- **Expected closeout evidence**: targeted `shim_doctor` / `shim_health` regression runs, platform-native smoke or doctor commands for Linux/macOS/Windows, and an explicit revalidation statement for shared-file overlap

## Slice index

- `S1` -> `slice-1-platform-matrix-and-playbook.md`
- `S2` -> `slice-2-smoke-checkpoint-and-revalidation.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
