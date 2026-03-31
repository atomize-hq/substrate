---
seam_id: SEAM-2
seam_slug: managed-cleanup-protected-path-guard
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-2-managed-cleanup-protected-path-guard.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
  required_threads:
    - THR-01
  stale_triggers:
    - manifest location or schema changes
    - repo-managed symlink ownership rules change
    - fixed bundle path list changes
    - protected-path exit-code taxonomy changes
    - ADR-0035 changes shared install-script or helper-script surfaces
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
# SEAM-2 - Managed cleanup + protected-path guard

## Seam Brief (Restated)

- **Goal / value**:
  - Make dev-uninstall remove only dev-managed staged assets while preserving user-managed files and non-repo-managed symlinks, so repeated install and uninstall cycles stay safe and auditable.
- **Type**: capability
- **Scope**
  - In:
    - remove repo-managed staged symlinks only at the fixed bundle paths
    - remove copied Linux guest binaries only when they are recorded in the dev-install manifest
    - preserve user-managed regular files and non-repo-managed symlinks
    - report preserved paths and use the protected-path refusal class (`exit 5`)
  - Out:
    - recursive cleanup outside the fixed bundle surface
    - changing the staging mechanism itself
    - Windows world-enable behavior enablement
    - production uninstall behavior beyond the dev-install helper bundle
- **Touch surface**:
  - `scripts/substrate/dev-uninstall-substrate.sh`
  - the dev-install-owned manifest under the selected prefix
  - fixed bundle paths under `$SUBSTRATE_HOME/scripts/...` and `bin/linux/...`
  - validation surfaces that pin refusal-class behavior
- **Verification**:
  - Pre-exec verification must make the owned contract concrete enough that implementation can remove only managed targets, refuse preserved paths, and keep cleanup authority bounded without waiting for post-exec publication.
  - The seam-local basis for execution is the current uninstall script surface plus the SEAM-1 closeout-backed bundle layout and managed-marker rules.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`
  - Required threads (inbound): `THR-01`
  - Stale triggers:
    - manifest location or schema changes
    - repo-managed symlink ownership rules change
    - fixed bundle path list changes
    - protected-path exit-code taxonomy changes
    - ADR-0035 changes shared install-script or helper-script surfaces
- **Threading constraints**
  - Upstream blockers:
    - no seam-local upstream closeout blocker exists
    - `SEAM-1` closeout now publishes the bundle surface and managed-marker truth this seam consumes
  - Downstream blocked seams:
    - `SEAM-3`
  - Contracts produced:
    - `C-04`
  - Contracts consumed:
    - `C-02`
    - `C-03`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - `SEAM-3` can only consume cleanup truth once the managed-only deletion boundary, protected-path refusal class, and preserved-path reporting are recorded in closeout-backed form.
- **Expected contracts to publish**:
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-03`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - any change to refusal classification or preserved-path messaging
  - any change to manifest-backed Linux guest binary cleanup rules
  - any change to directory-pruning behavior around the fixed bundle surface
  - any cleanup expansion that starts treating fixed path names as authority rather than provenance
- **Expected closeout evidence**:
  - landed managed-symlink cleanup evidence
  - landed manifest-tracked copied-binary cleanup evidence
  - landed protected-path refusal and preserved-path evidence
  - explicit downstream stale-trigger record for `SEAM-3`

## Slice index

- `S1` -> `slice-1-managed-only-cleanup-contract.md`
- `S2` -> `slice-2-protected-path-refusal-and-reporting.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
