---
seam_id: SEAM-3
seam_slug: parity-and-contract-lock-in
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-parity-and-contract-lock-in.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - published replay fragments, telemetry fields, or omission rules drift from ../../governance/seam-2-closeout.md
    - docs examples, smoke wrappers, or manual playbook steps drift from the published runtime contract
    - required parity platforms or allowed divergences change from the source parity spec
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
  planned_location: S4
  status: pending
open_remediations: []
---
# SEAM-3 - Parity and contract lock-in (threaded decomposition)

## Seam Brief (Restated)

- **Goal / value**:
  - Lock replay tests, docs, smoke wrappers, and manual validation to the runtime truth published by `SEAM-2`.
  - Prevent post-publication drift across Linux, macOS, and Windows without introducing new runtime semantics.
- **Type**: conformance
- **Scope**
  - In:
    - regression coverage for override env, workspace config, global config, unknown-source fallback, and replay-local opt-out omission rules
    - docs alignment for `docs/REPLAY.md`, `docs/TRACE.md`, and `docs/COMMANDS.md`
    - smoke-wrapper and manual playbook alignment with the same contract and test filters
    - parity evidence for Linux, macOS, and Windows
  - Out:
    - new replay behavior or new trace event types
    - helper redesign or runtime contract expansion
    - foundation or runtime reimplementation
- **Touch surface (expected)**:
  - `crates/shell/tests/replay_world.rs`
  - `docs/REPLAY.md`
  - `docs/TRACE.md`
  - `docs/COMMANDS.md`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/world-disabled-reason-attribution/smoke/windows-smoke.ps1`
- **Verification**:
  - This seam consumes `C-02`, `C-03`, and `C-04` exactly as published in `../../governance/seam-1-closeout.md` and `../../governance/seam-2-closeout.md`.
  - Verification must prove that tests, docs, smoke wrappers, and the manual playbook all pin the same fragments, codes, tokenized displays, and omission rules.
- **Basis posture**:
  - Currentness: `current` because `../../governance/seam-2-closeout.md` now publishes the landed runtime copy and telemetry contract.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - Required threads:
    - `THR-02`
    - `THR-03`
    - `THR-04`
- **Threading constraints**
  - Upstream blockers:
    - none while `../../governance/seam-2-closeout.md` remains the published source of truth for replay copy and telemetry
  - Downstream blocked seams:
    - none inside this pack
  - Contracts consumed:
    - `C-02`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S4` (`slice-4-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - parity and lock-in work must end with one closeout-backed record showing that tests, docs, smoke wrappers, and manual validation all align to the same published runtime contract.
- **Expected threads to revalidate / close**:
  - `THR-02`
  - `THR-03`
  - `THR-04`
- **Expected closeout evidence**:
  - replay tests pinning the published fragments, codes, and omission rules
  - docs and smoke wrappers aligned to the same expectations
  - a parity-ready manual playbook that references the same filters and assertions

## Slice index

- `S1` -> `slice-1-regression-coverage-and-trace-locks.md`
- `S2` -> `slice-2-docs-and-playbook-alignment.md`
- `S3` -> `slice-3-smoke-wrapper-and-parity-evidence.md`
- `S4` -> `slice-4-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
