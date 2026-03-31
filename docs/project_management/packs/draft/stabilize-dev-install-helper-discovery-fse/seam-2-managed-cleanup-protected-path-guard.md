---
seam_id: SEAM-2
seam_slug: managed-cleanup-protected-path-guard
type: capability
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
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

- **Goal / value**: Make dev-uninstall remove only dev-managed staged assets while preserving user-managed files and non-repo-managed symlinks, so repeated install and uninstall cycles stay safe and auditable.
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
- **Primary interfaces**
  - Inputs:
    - `scripts/substrate/dev-uninstall-substrate.sh --prefix --profile`
    - the staged bundle-path contract from `SEAM-1`
    - managed-asset markers and manifest schema from `SEAM-1`
    - shared exit taxonomy for protected-path refusal
  - Outputs:
    - bounded managed-only cleanup behavior
    - deterministic preserved-path reporting and refusal classification
    - cleanup evidence that later smoke and manual validation can assert
- **Key invariants / rules**:
  - Delete only repo-managed symlinks or manifest-tracked copied Linux guest binaries.
  - Never treat fixed path names alone as authority to delete.
  - Never widen cleanup to a recursive delete of the whole scripts tree.
  - Preserve user-managed regular files and non-repo-managed symlinks in place.
  - Report protected paths deterministically when cleanup stops.
- **Dependencies**
  - Direct blockers:
    - `THR-01` must publish the landed bundle surface and managed-marker rules from `SEAM-1`
  - Transitive blockers:
    - shared exit-code taxonomy
    - any ADR-0035 drift that changes the staged bundle surface before cleanup planning promotes
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future repeated install and uninstall workflows that depend on safe prefix reuse
- **Touch surface**:
  - `scripts/substrate/dev-uninstall-substrate.sh`
  - the dev-install-owned manifest under the selected prefix
  - fixed bundle paths under `$SUBSTRATE_HOME/scripts/...` and `bin/linux/...`
  - validation surfaces that pin refusal-class behavior
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Expected proof points:
    - repo-managed staged symlinks are removed cleanly
    - manifest-tracked copied Linux guest binaries are removed while unmanaged copies remain untouched
    - a user-managed regular file at a managed path is preserved
    - a non-repo-managed symlink at a managed path is preserved
    - preserved paths are reported with the protected-path refusal class defined in `contract.md`
- **Risks / unknowns**:
  - Risk: manifest drift can cause false deletion or missed cleanup.
  - De-risk plan: bind cleanup eligibility to the published manifest shape from `SEAM-1`, and capture negative cases explicitly in the first seam-local review.
  - Risk: ownership classification for symlinks can misclassify a path in multi-checkout or moved-checkout scenarios.
  - De-risk plan: keep deletion bounded to repo-managed targets only and treat ambiguity as refusal rather than deletion.
- **Rollout / safety**:
  - Fail safe on protected paths.
  - Keep unmanaged paths in place even when cleanup cannot complete fully.
  - Do not broaden deletion authority beyond the exact managed contract published upstream.
- **Downstream decomposition context**:
  - Why this seam is active: it is bounded and decomposable, and its basis is now revalidated against the landed bundle and manifest shape from `SEAM-1`.
  - Which threads matter most: `THR-01` and `THR-03`.
  - What the first seam-local review should focus on: ownership classification, manifest schema and location, refusal messaging, negative cases, and directory-pruning behavior.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-04`.
  - Threads likely to advance: `THR-03`, plus consumption-side revalidation against `THR-01`.
  - Review-surface areas likely to shift after landing: preserved-path messaging, cleanup disposition summaries, and any empty-directory pruning behavior.
  - Downstream seams most likely to require revalidation: `SEAM-3`.
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
