---
seam_id: SEAM-3
seam_slug: cross-platform-proof-drift-guards
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-cross-platform-proof-drift-guards.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
  stale_triggers:
    - Linux or macOS behavior scope changes
    - Windows compile-parity-only posture changes
    - smoke command set or manual playbook cases change
    - checkpoint boundary or checkpoint evidence requirements change
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
open_remediations:
  - REM-002
---
# SEAM-3 - Cross-platform proof + drift guards

## Seam Brief (Restated)

- **Goal / value**:
  - Keep the landed feature contract provable and non-drifting across Linux and macOS behavior validation, Windows compile parity, and checkpoint evidence after the first two seams land.
- **Type**: conformance
- **Scope**
  - In:
    - manual playbook coverage for bundle staging, helper discovery, and managed cleanup
    - Linux and macOS smoke wrappers as authoritative behavior evidence
    - Windows smoke as compile-parity evidence only
    - checkpoint-boundary proof after the cleanup seam lands
  - Out:
    - new runtime behavior not already owned by `SEAM-1` or `SEAM-2`
    - enabling supported `substrate world enable` behavior on Windows
    - broadening the staged bundle beyond the upstream seams
- **Touch surface**:
  - `manual_testing_playbook.md`
  - `platform-parity-spec.md`
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`
  - checkpoint and closeout evidence surfaces
- **Verification**:
  - The seam-local basis now consumes closeout-backed `THR-01`, `THR-02`, and `THR-03`.
  - Pre-exec verification must make the proof boundary concrete enough that smoke, playbook, and checkpoint surfaces stay aligned to landed upstream contracts without overclaiming platform support.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
  - Stale triggers:
    - Linux or macOS behavior scope changes
    - Windows compile-parity-only posture changes
    - smoke command set or manual playbook cases change
    - checkpoint boundary or checkpoint evidence requirements change
- **Threading constraints**
  - Upstream blockers:
    - no upstream closeout blocker remains
    - `SEAM-1` and `SEAM-2` now publish the contract truth this seam consumes
  - Downstream blocked seams:
    - none
  - Contracts produced:
    - none beyond landed evidence mapping
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Pre-exec readiness posture

- `REM-002` remains open, but it is seam-local and non-blocking because `S1`, `S2`, and `S3` now own the exact macOS claim-boundary, Windows compile-parity, and seam-exit accounting work needed before landing.
- `SEAM-3` owns evidence and wording alignment only; it does not reopen the upstream helper-bundle or cleanup contracts already published by `SEAM-1` and `SEAM-2`.
- Execution work may begin because the remaining ambiguity is bounded to seam-local proof surfaces and closeout disposition, not to an upstream handoff or an unresolved producer contract.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - the pack cannot close until smoke evidence, playbook wording, and checkpoint truth are all recorded against the landed upstream contracts
- **Expected contracts to publish**:
  - none beyond final evidence alignment
- **Expected threads to publish / advance**:
  - `THR-01`, `THR-02`, and `THR-03` toward `revalidated` or `closed`
- **Likely downstream stale triggers**:
  - platform support wording drift
  - smoke assertion drift
  - checkpoint boundary or evidence requirement drift
- **Expected closeout evidence**:
  - landed smoke proof across Linux, macOS, and Windows compile parity
  - landed manual playbook and checkpoint alignment
  - explicit stale-trigger record for future platform or evidence drift

## Slice index

- `S1` -> `slice-1-freeze-platform-evidence-boundaries.md`
- `S2` -> `slice-2-refresh-cross-platform-proof-surfaces.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
