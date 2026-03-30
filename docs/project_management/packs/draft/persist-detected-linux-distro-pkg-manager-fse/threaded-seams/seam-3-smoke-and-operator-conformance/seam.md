---
seam_id: SEAM-3
seam_slug: smoke-and-operator-conformance
status: exec-ready
execution_horizon: active
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-smoke-and-operator-conformance.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-02
    - THR-03
  stale_triggers:
    - any change to the published `C-01` or `C-02` field naming path wording or additive-compatibility rules after THR-03 revalidation
    - any change to the published `C-03` or `C-04` branch matrix temp-file semantics or warning-only posture after THR-02 revalidation
    - smoke harness docs or checkpoint evidence surfaces move before `SEAM-3` lands
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
# SEAM-3 - Smoke And Operator Conformance

## Seam Brief (Restated)

- **Goal / value**: convert the landed schema, path, branch-matrix, and reliability truth into smoke coverage, operator wording, and checkpoint evidence that downstream reviewers can trust without reconstructing runtime behavior from code history.
- **Type**: conformance
- **Scope**
  - In:
    - Linux smoke assertions for the successful-Linux producer matrix and the explicit no-write boundaries
    - persisted field assertions for `host_state.platform.os_release.id`, `host_state.platform.os_release.id_like`, `host_state.platform.pkg_manager.selected`, and `host_state.platform.pkg_manager.source`
    - missing-os-release degradation and additive-compatibility evidence
    - `docs/INSTALLATION.md` wording for canonical path, producer scope, and `schema_version = 1`
    - checkpoint-ready evidence surfaces in the source planning pack
  - Out:
    - changing runtime writer mechanics or payload ownership
    - expanding non-Linux runtime behavior
    - inventing a manual validation playbook separate from the smoke harness and checkpoint artifacts
    - resolving `REM-003` or unrelated shared-file work outside the conformance surfaces
- **Touch surface**:
  - `tests/installers/install_state_smoke.sh`
  - `docs/INSTALLATION.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/plan.md`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/tasks.json`
  - `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/session_log.md`
  - `../../governance/pack-closeout.md`
- **Verification**:
  - execution starts only after seam-local artifacts make the smoke assertions, documentation wording, and checkpoint evidence concrete enough that pack closeout can rely on landed proof instead of source-pack intent
  - verification must prove the smoke harness consumes `SEAM-1` and `SEAM-2` closeout truth rather than reopening schema, path, branch-matrix, or warning-only behavior decisions
  - the pre-exec revalidation evidence for this plan is `../../governance/seam-1-closeout.md`, `../../governance/seam-2-closeout.md`, the current smoke harness surface in `tests/installers/install_state_smoke.sh`, and the current operator wording in `docs/INSTALLATION.md`
  - accepted or published owned-contract artifacts are reserved for seam-exit evidence and closeout, not pre-exec readiness for the producing seam
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
  - Required threads:
    - `THR-02`
    - `THR-03`
  - Stale triggers:
    - schema-path authority changes in `SEAM-1`
    - writer-matrix or reliability changes in `SEAM-2`
    - smoke/docs/checkpoint surface movement before landing
- **Threading constraints**
  - Upstream blockers:
    - `THR-02` must stay revalidated against the published `SEAM-2` closeout
    - `THR-03` must stay revalidated against the published `SEAM-1` closeout
    - no new blocking remediation may reopen canonical-path field-naming or writer-semantics ambiguity
  - Downstream blocked seams: none inside this pack
  - Contracts produced:
    - `C-05`
    - `C-06`
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S3` / `slice-3-seam-exit-gate.md`
- **Why this seam needs an explicit exit gate**: pack closeout and downstream maintainers need one accepted evidence record for smoke coverage, docs wording, and checkpoint posture instead of piecing it together from multiple source-pack artifacts.
- **Expected contracts to publish**:
  - `C-05`
  - `C-06`
- **Expected threads to publish / advance**:
  - `THR-02` toward `closed`
  - `THR-03` toward `closed`
- **Likely downstream stale triggers**:
  - any later change to the canonical path wording or field names
  - any later change to the Linux write/no-write matrix or warning-only posture
  - any later change to the evidence commands or checkpoint surfaces relied on by pack closeout
- **Expected closeout evidence**:
  - landed smoke assertions that match the published writer behavior
  - landed documentation wording that matches the accepted contract and runtime truth
  - checkpoint and pack-closeout evidence that points at one coherent validation story

## Slice index

- `S1` -> `slice-1-linux-smoke-conformance-and-drift-guards.md`
- `S2` -> `slice-2-operator-doc-and-checkpoint-evidence-alignment.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Upstream closeouts:
  - `../../governance/seam-1-closeout.md`
  - `../../governance/seam-2-closeout.md`
