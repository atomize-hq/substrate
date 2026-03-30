---
seam_id: SEAM-2
seam_slug: install-state-writer-reliability
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-2-install-state-writer-reliability.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - any change to the published `C-01` or `C-02` field-path path-alias or upstream-vocabulary boundary after THR-01 revalidation
    - adjacent packs refactor `scripts/substrate/install-substrate.sh` or `scripts/substrate/dev-install-substrate.sh` before SEAM-2 lands
    - dry-run no-write invalid-file fallback or warning-only semantics change before `SEAM-2` closeout publishes `C-03` and `C-04`
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
  - REM-003
---
# SEAM-2 - Install-State Writer Reliability

## Seam Brief (Restated)

- **Goal / value**: make successful Linux installs reliably create or update the canonical `install_state.json` file so downstream smoke and docs can validate concrete writer behavior instead of compensating for event-only metadata writes.
- **Type**: platform
- **Scope**
  - In:
    - successful-Linux write coverage for hosted install, hosted `--no-world`, dev install, and dev `--no-world`
    - hosted `--dry-run` and non-Linux no-write boundaries
    - canonical-file create and update behavior when the file is absent or present
    - same-directory temp-file render plus a single replace step
    - warning-only fallback for unreadable JSON invalid JSON or non-`1` schema content
    - preservation of prior canonical content when temp-file write or replace fails
  - Out:
    - redefining the `host_state.platform.*` payload or package-manager vocabulary
    - smoke-harness and operator-doc rewrites
    - uninstaller cleanup-path alignment beyond carrying `REM-003`
- **Touch surface**:
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/dev-install-substrate.sh`
  - `tests/installers/install_state_smoke.sh`
  - downstream doc surface that must later match landed writer truth:
    - `docs/INSTALLATION.md`
- **Verification**:
  - execution starts only after seam-local artifacts make the successful-Linux write matrix and the reliability contract concrete enough that `SEAM-3` can validate landed behavior without reopening file-creation branch rules or failure posture
  - verification must prove the current hosted and dev installers still skip metadata writes when no host-state events occur, already converge on one canonical file path, and already use same-directory temp-file replacement plus warning-only parsing/write behavior as the execution baseline
  - the pre-exec revalidation evidence for this plan is `../../governance/seam-1-closeout.md`, the current installer surfaces in `scripts/substrate/install-substrate.sh` and `scripts/substrate/dev-install-substrate.sh`, and the current smoke harness shape in `tests/installers/install_state_smoke.sh`
  - accepted or published owned-contract artifacts are reserved for seam-exit evidence and closeout, not pre-exec readiness for the producing seam
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
  - Required threads:
    - `THR-01`
  - Stale triggers:
    - published `C-01` or `C-02` contract truth changes after THR-01 revalidation
    - shared installer-script sequencing changes
    - dry-run no-write invalid-file fallback or warning-only semantics drift
- **Threading constraints**
  - Upstream blockers:
    - `THR-01` must stay revalidated against the published SEAM-1 closeout
    - no new blocking remediation may reopen canonical-path or payload-contract ambiguity
  - Downstream blocked seams:
    - `SEAM-3`
  - Contracts produced:
    - `C-03`
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-02`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` / `slice-3-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: downstream conformance work must consume one landed writer matrix and one landed reliability story instead of reconstructing branch behavior from two installer scripts.
- **Expected contracts to publish**:
  - `C-03`
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-02` to `published`
- **Likely downstream stale triggers**:
  - any change to successful-Linux versus no-write branch coverage
  - any change to temp-file placement replace semantics or warning-only degradation
  - any change to canonical path handling that stops matching published `C-02`
- **Expected closeout evidence**:
  - landed writer evidence in both installer scripts
  - publication accounting for `C-03`, `C-04`, and `THR-02`
  - review-surface delta against the planned write-matrix and reliability posture

## Slice index

- `S1` -> `slice-1-successful-linux-write-matrix-and-no-write-boundaries.md`
- `S2` -> `slice-2-atomic-replace-and-warning-only-degradation.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Upstream closeout: `../../governance/seam-1-closeout.md`
