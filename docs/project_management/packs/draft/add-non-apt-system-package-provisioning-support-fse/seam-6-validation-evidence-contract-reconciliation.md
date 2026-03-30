---
seam_id: SEAM-6
seam_slug: validation-evidence-contract-reconciliation
type: conformance
status: landed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
  - governance/seam-1-closeout.md
  - governance/seam-2-closeout.md
  - governance/seam-3-closeout.md
  - governance/seam-4-closeout.md
  - governance/seam-5-closeout.md
  required_threads:
  - THR-01
  - THR-02
  - THR-03
  - THR-04
  - THR-05
  stale_triggers:
  - C-01 through C-05 change after parity/manual/smoke surfaces are written
  - repo-default macOS Lima fixture or manual Arch fixture assumptions change
  - overlapping ADR or docs continue to present a second truth for manager-aware behavior
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
  planned_location: S3
  status: passed
open_remediations: []
---

# SEAM-6 - Validation evidence and contract reconciliation

This seam is landed. Its authoritative planning and exit-gate record live in `threaded-seams/seam-6-validation-evidence-contract-reconciliation/` and `governance/seam-6-closeout.md`.

- **Goal / value**:
  - Lock the cross-platform evidence story and remove second-truth drift from overlapping ADR and documentation surfaces once the manager-aware behavior seams land.
- **Scope**
  - In:
    - one accepted Linux/macOS/Windows support matrix
    - one manual testing playbook with the required operator evidence cases
    - three smoke scripts that validate the platform-specific assertion sets
    - one exact reconciliation target set for ADR-0033, the APT pack contract, the bundles contract, and world/deps docs
    - final parity and reconciliation evidence needed for pack closeout
  - Out:
    - new probe, schema, provisioning, or runtime behavior
    - widening pacman support beyond the contracts already owned by earlier seams
    - checkpoint/task automation as a feature seam
- **Primary interfaces**
  - Inputs:
    - `C-01` from `SEAM-1`
    - `C-02` from `SEAM-2`
    - `C-03` from `SEAM-3`
    - `C-04` from `SEAM-4`
    - `C-05` from `SEAM-5`
    - repo-default Lima guest assumptions and manual Arch fixture assumptions
  - Outputs:
    - authoritative parity/manual/smoke evidence package for pack closeout
    - explicit reconciliation target list for shared manager-aware docs
    - terminal drift-guard basis for future maintainers
- **Key invariants / rules**:
  - pack-level validation surfaces must not invent a second behavior contract; they execute the behavior already fixed by earlier seams
  - Linux host-native and Windows provisioning remain unsupported evidence lanes
  - macOS default smoke proves the supported guest-backend path on the repo-default Ubuntu-based Lima guest
  - Arch-family pacman-success remains a manual-only evidence lane in this pack
  - reconciliation is not complete until the named overlapping docs stop presenting a second truth for the shared manager-aware contract
- **Dependencies**
  - Direct blockers:
    - none; `SEAM-1` through `SEAM-5` now publish the contracts and closeouts this terminal seam consumes.
  - Transitive blockers:
    - checkpoint/task automation may need to be refreshed if validation scope changes, but that governance work is not a seam here
  - Direct consumers:
    - none inside this pack; this is the terminal conformance seam
  - Derived consumers:
    - pack closeout
    - operators, support, and future maintainers
- **Touch surface**:
  - Primary planning surfaces:
    - `slices/NASP4/NASP4-spec.md`
    - `platform-parity-spec.md`
    - `manual_testing_playbook.md`
    - `smoke/linux-smoke.sh`
    - `smoke/macos-smoke.sh`
    - `smoke/windows-smoke.ps1`
  - Likely downstream reconciliation surfaces once seam-local planning begins:
    - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
    - `docs/reference/world/deps/README.md`
    - `docs/internals/world/deps.md`
- **Verification**:
  - Because this seam **consumes** upstream contracts, verification depended on accepted upstream evidence from `SEAM-1` through `SEAM-5`.
  - Seam-local review focused on falsifying:
    - whether any named reconciliation target can still present APT-only or mutation-at-runtime truth
    - whether smoke/manual evidence leaves a missing platform lane or an unrecorded fixture assumption
    - whether parity docs or smoke scripts accidentally redefine behavior instead of proving it
  - The landed pre-exec and post-exec posture leaves pack closeout able to trust one final validation story.
- **Risks / unknowns**:
  - Risk:
    - shared-contract reconciliation targets can remain out of sync and continue to present a second truth after earlier seams land.
  - De-risk plan:
    - keep that visible as `REM-001` and force seam-local review to inspect every named reconciliation target directly.
  - Risk:
    - Arch-family pacman-success evidence on macOS depends on a non-default manual Lima fixture and can drift if the fixture assumptions are not kept explicit.
  - De-risk plan:
    - keep that visible as `REM-002` and require the manual playbook and closeout evidence to name the fixture assumptions explicitly.
- **Rollout / safety**:
  - This seam should not create new behavior. Its job is to prove and lock earlier behavior so the feature does not drift after landing.
  - Because it is a conformance seam, it must not become a cleanup bucket for net-new probe or provisioning work that belongs upstream.
- **Downstream decomposition context**:
  - This seam was the terminal active seam after `SEAM-5` closed with a passed seam-exit gate, published `THR-05`, and left the terminal validation basis current.
  - The most important threads are `THR-01`, `THR-02`, `THR-03`, `THR-04`, and `THR-05`.
  - The landed seam-local review focused on parity completeness, smoke/manual fixture assumptions, and every named second-truth reconciliation target.
  - Source-plan lineage: `NASP-PWS-docs_validation`, `NASP4`, `platform-parity-spec.md`, `manual_testing_playbook.md`, and the three smoke scripts.
- **Realized seam-exit record**:
  - Contracts published:
    - none beyond terminal evidence and reconciliation truth
  - Threads consumed:
    - `THR-01` through `THR-05` remain `revalidated` and are recorded with explicit terminal no-downstream-carry accounting
  - Review-surface areas that shifted at landing:
    - the platform posture now records the manual-only Arch-family evidence lane explicitly
    - the reconciliation targets now defer to or restate the accepted manager-aware contracts in one voice
  - Downstream seams requiring revalidation:
    - none inside this pack; the seam now feeds pack closeout instead
