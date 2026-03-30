---
seam_id: SEAM-6
seam_slug: validation-evidence-contract-reconciliation
type: conformance
status: proposed
execution_horizon: next
plan_version: v2
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
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
open_remediations:
- REM-001
- REM-002
---

# SEAM-6 - Validation evidence and contract reconciliation

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
    - `SEAM-1`
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
    - `SEAM-5`
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
  - Because this seam **consumes** upstream contracts, verification may depend on accepted upstream evidence from `SEAM-1` through `SEAM-5`.
  - The first seam-local review should try to falsify:
    - whether any named reconciliation target can still present APT-only or mutation-at-runtime truth
    - whether smoke/manual evidence leaves a missing platform lane or an unrecorded fixture assumption
    - whether parity docs or smoke scripts accidentally redefine behavior instead of proving it
  - A passing pre-exec posture should leave pack closeout able to trust one final validation story.
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
  - This seam is now `next` and terminal. It should remain seam-brief only until the earlier behavior seams publish enough truth to validate and reconcile against.
  - The most important threads are `THR-01`, `THR-02`, `THR-03`, `THR-04`, and `THR-05`.
  - The first seam-local review should focus on parity completeness, smoke/manual fixture assumptions, and every named second-truth reconciliation target.
  - Source-plan lineage: `NASP-PWS-docs_validation`, `NASP4`, `platform-parity-spec.md`, `manual_testing_playbook.md`, and the three smoke scripts.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - none beyond terminal evidence and reconciliation truth
  - Threads likely to advance:
    - `THR-01` through `THR-05` from `published` to `revalidated` or `closed`
  - Review-surface areas likely to shift after landing:
    - the platform posture diagram once automated vs manual evidence is fully recorded
    - the touch-surface map around reconciliation targets
  - Downstream seams most likely to require revalidation:
    - none inside this pack; this seam should feed pack closeout instead
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
