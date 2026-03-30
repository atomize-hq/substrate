---
seam_id: SEAM-1
seam_slug: persisted-platform-metadata-contract
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_seam_brief: ../../seam-1-persisted-platform-metadata-contract.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - upstream detection contract changes selected-manager or pkg_manager.source vocabulary after pre-exec revalidation
    - os_release sentinel or field-path rules change before SEAM-1 closeout publishes C-01 and C-02
    - ADR-0032 or related docs recreate competing feature-directory authority before closeout
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
# SEAM-1 - Persisted Platform Metadata Contract

## Seam Brief (Restated)

- **Goal / value**: freeze one exact persisted `install_state.json` payload and path contract so downstream writer and conformance seams can implement against a single schema, a single canonical file rule, and a single upstream-authority boundary.
- **Type**: integration
- **Scope**
  - In:
    - exact `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source` field paths
    - `schema_version = 1` additive-compatibility and merge rules for preserving `host_state.group`, `host_state.linger`, and unknown keys
    - canonical on-disk path semantics for `<effective_prefix>/install_state.json` plus the `$SUBSTRATE_HOME/install_state.json` default-prefix alias
    - verbatim-copy authority boundaries to the upstream distro and package-manager detection contract
    - seam-local verification checklists that make `C-01` and `C-02` concrete enough for later execution
  - Out:
    - implementing successful-Linux write triggers, dry-run or non-Linux no-write branches, and warning-only failure behavior
    - same-directory temp-file replace mechanics beyond naming the contract requirement
    - smoke assertions, checkpoint evidence, and operator-doc rewrites
    - uninstaller cleanup-path alignment
- **Touch surface**:
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/install-state-schema-spec.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/decision_register.md`
  - `docs/project_management/adrs/draft/ADR-0032-stashing-ferret.md`
  - downstream consumer surfaces that must later match the published contract:
    - `scripts/substrate/install-substrate.sh`
    - `scripts/substrate/dev-install-substrate.sh`
    - `tests/installers/install_state_smoke.sh`
    - `docs/INSTALLATION.md`
- **Verification**:
  - execution starts only after seam-local artifacts make `C-01` and `C-02` concrete enough that `SEAM-2` can implement one exact payload/path contract without reopening field ownership or canonical-path questions
  - verification must prove the platform block remains additive on `schema_version = 1`, preserves `host_state.group`, `host_state.linger`, and unknown keys, and copies upstream detection outputs verbatim instead of re-deriving them locally
  - the pre-exec revalidation evidence for this plan is the accepted source-pack `contract.md` plus `DR-0005`, the latest `best-effort-distro-package-manager` contract, and the current effective-prefix installer path surfaces in `install-substrate.sh` and `dev-install-substrate.sh`
  - accepted or published contract artifacts are reserved for seam-exit evidence and closeout, not pre-exec readiness for the producing seam
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: none
  - Required threads: none; `SEAM-1` is the producer for `THR-01` and `THR-03`
  - Stale triggers:
    - upstream detection contract changes selected-manager or source vocabulary
    - os_release sentinel or field-path rules change
    - ADR-0032 or related docs recreate dual-authority pack-path inputs
- **Threading constraints**
  - Upstream blockers:
    - no blocking upstream remediations remain open after the accepted contract override and `DR-0005` established the canonical feature-directory authority
    - upstream detection vocabulary and `<unknown>` sentinel semantics must remain unchanged through closeout-backed downstream promotion
  - Downstream blocked seams:
    - `SEAM-2`
    - `SEAM-3`
  - Contracts produced:
    - `C-01`
    - `C-02`
  - Contracts consumed: none

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` / `slice-3-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: both downstream seams consume this seam's contract truth, so promotion cannot rely on inferred field naming, path semantics, or unpublished authority-boundary decisions.
- **Expected contracts to publish**:
  - `C-01`
  - `C-02`
- **Expected threads to publish / advance**:
  - `THR-01` to `published`
  - `THR-03` to `published`
- **Likely downstream stale triggers**:
  - field-path or nesting changes under `host_state.platform.*`
  - `<effective_prefix>` versus `$SUBSTRATE_HOME` wording changes
  - any change to verbatim-copy ownership for `pkg_manager.selected` or `pkg_manager.source`
- **Expected closeout evidence**:
  - landed contract and schema evidence that names one exact payload shape and one exact canonical-path rule
  - publication accounting for `C-01`, `C-02`, `THR-01`, and `THR-03`
  - review-surface delta against the planned authority boundary, additive-compatibility story, and downstream stale-trigger set

## Slice index

- `S1` -> `slice-1-persisted-schema-and-merge-contract.md`
- `S2` -> `slice-2-canonical-path-and-authority-boundary.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-1-closeout.md`
