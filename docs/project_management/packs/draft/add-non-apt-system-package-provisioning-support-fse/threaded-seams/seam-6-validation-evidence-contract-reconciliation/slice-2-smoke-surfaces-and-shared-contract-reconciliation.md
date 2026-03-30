---
slice_id: S2
seam_id: SEAM-6
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
  - shared ADR or contract docs drift away from the published manager-aware behavior
  - smoke surfaces stop matching the supported, unsupported, or manual-only platform lanes
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
  - THR-03
  - THR-04
  - THR-05
contracts_produced: []
contracts_consumed:
  - C-01
  - C-02
  - C-03
  - C-04
  - C-05
open_remediations:
  - REM-001
candidate_subslices: []
---
### S2 - Smoke surfaces and shared contract reconciliation

- **User/system value**:
  - the smoke scripts and shared docs stop acting as parallel truth sources and instead reinforce the already-published manager-aware contracts.
- **Scope (in/out)**:
  - In:
    - refresh the Linux/macOS/Windows smoke surfaces so their assertions match the published support matrix and runtime/provisioning contracts
    - reconcile ADR-0033, the APT-pack contract, the bundles contract, and shared world/deps docs with the pack-root contracts
    - resolve the second-truth posture tracked in `REM-001`
  - Out:
    - new runtime or provisioning behavior
    - terminal closeout accounting beyond the evidence and reconciliation surfaces landed here
- **Acceptance criteria**:
  - smoke scripts reinforce supported, unsupported, and manual-only platform lanes without redefining behavior
  - every named reconciliation target now defers to or restates the accepted manager-aware contracts in one consistent voice
  - `REM-001` has concrete resolution evidence or an explicit terminal disposition ready for closeout
- **Dependencies**:
  - landed closeouts `../../governance/seam-1-closeout.md` through `../../governance/seam-5-closeout.md`
- **Verification**:
  - review of the three smoke scripts
  - review of each named reconciliation target
  - closeout citation readiness for `../../governance/seam-6-closeout.md`
- **Rollout/safety**:
  - no new behavior ownership; this slice only proves and reconciles published truth
- **Review surface refs**:
  - `review.md`

#### S2.T1 - Align the smoke surfaces with the published platform posture

- **Outcome**:
  - the smoke scripts prove the same manager-aware support matrix the docs and contracts describe.
- **Files**:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/linux-smoke.sh`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/macos-smoke.sh`
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/smoke/windows-smoke.ps1`
- **Thread/contract refs**:
  - `THR-02`
  - `THR-04`
  - `THR-05`
- **Acceptance criteria**:
  - smoke assertions distinguish supported, unsupported, and manual-only lanes
  - smoke guidance does not imply runtime package-manager mutation

#### S2.T2 - Reconcile the named second-truth contract surfaces

- **Outcome**:
  - overlapping ADR and contract docs stop presenting a second truth for manager-aware world-deps behavior.
- **Files**:
  - `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - `docs/reference/world/deps/README.md`
  - `docs/internals/world/deps.md`
- **Thread/contract refs**:
  - `THR-01`
  - `THR-05`
  - `REM-001`
- **Acceptance criteria**:
  - each surface now defers to or restates the accepted manager-aware contracts in one consistent voice
  - no reconciled surface reintroduces APT-only or mutation-at-runtime behavior
