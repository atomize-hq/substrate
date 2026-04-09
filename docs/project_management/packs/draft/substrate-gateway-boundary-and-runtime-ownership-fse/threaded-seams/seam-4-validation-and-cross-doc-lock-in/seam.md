---
seam_id: SEAM-4
seam_slug: validation-and-cross-doc-lock-in
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-4-validation-and-cross-doc-lock-in.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
  stale_triggers:
    - manual testing assertions drift from the landed operator, schema, policy, or runtime/parity contracts
    - `plan.md`, `tasks.json`, or checkpoint boundaries drift from the accepted seam and slice ordering
    - `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, or `docs/TRACE.md` restate stale ownership or schema wording
    - stale archived `packs/active/...` references or ADR cross-links reintroduce ownership ambiguity
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
  planned_location: S99
  status: passed
open_remediations: []
---
# SEAM-4 - Validation and cross-doc lock-in

## Seam Brief (Restated)

- **Goal / value**:
  - Prove that every ADR-0040 surface has one owner and that manual validation, docs, and planning artifacts all consume the same landed truth.
  - Prevent late drift where the contracts are correct in isolation but stale in operator docs, playbooks, or quality-gate evidence.
- **Type**: conformance
- **Scope**
  - In:
    - one-owner-per-surface manual validation
    - `manual_testing_playbook.md`
    - `plan.md`, `tasks.json`, `session_log.md`, and `quality_gate_report.md`
    - `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, and `docs/TRACE.md`
    - checkpoint boundary alignment and stale-link normalization posture
  - Out:
    - new command semantics
    - new `status --json` schema fields or policy rules
    - runtime transport design changes
    - provisioning or backend-warm implementation changes
- **Touch surface**:
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/manual_testing_playbook.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/plan.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/tasks.json`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/session_log.md`
  - `docs/project_management/packs/draft/substrate-gateway-boundary-and-runtime-ownership/quality_gate_report.md`
  - `docs/CONFIGURATION.md`
  - `docs/USAGE.md`
  - `docs/WORLD.md`
  - `docs/TRACE.md`
- **Verification**:
  - This seam consumes published contracts `C-01`, `C-02`, `C-03`, and `C-04`; pre-exec verification passes because the upstream closeouts are landed, their required threads are revalidated, and the seam-local slices make the conformance touch set explicit.
  - This seam does not own a new durable contract. Execution should therefore prove alignment and drift detection, not invent new contract wording.
  - Later seam-local verification should prove:
    - every ADR-0040 surface has one and only one owner
    - manual playbook assertions match the landed contracts
    - docs and planning artifacts reflect the same truth
    - stale archived references are either normalized or explicitly tracked as follow-up evidence
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-2-closeout.md`
    - `../../governance/seam-3-closeout.md`
  - Required threads:
    - `THR-01`
    - `THR-02`
    - `THR-03`
    - `THR-04`
  - Stale triggers: see frontmatter `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers:
    - none after the seam-1, seam-2, and seam-3 closeouts published the contracts this seam consumes
  - Downstream blocked seams:
    - none inside this pack
  - Contracts produced:
    - none
  - Contracts consumed:
    - `C-01`
    - `C-02`
    - `C-03`
    - `C-04`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**:
  - the pack cannot close safely unless the manual playbook, docs, and quality-gate artifacts all cite one consistent ownership boundary and one consistent validation posture
- **Expected contracts to publish**:
  - none; this seam locks conformance around already published contracts
- **Expected threads to publish / advance**:
  - inbound `THR-02`, `THR-03`, and `THR-04` remain revalidated through execution closeout evidence
- **Likely downstream stale triggers**:
  - manual validation assertions change without corresponding contract changes
  - docs or quality-gate artifacts restate stale ownership or status-schema wording
  - checkpoint wiring drifts from the accepted seam and slice ordering
- **Expected closeout evidence**:
  - landed manual playbook and one-owner-per-surface updates
  - landed docs alignment across `docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`, and `docs/TRACE.md`
  - landed `plan.md`, `tasks.json`, `session_log.md`, and `quality_gate_report.md` alignment

## Slice index

- `S1` -> `slice-1-manual-validation-and-owner-surface-audit.md`
- `S2` -> `slice-2-operator-docs-and-trace-alignment.md`
- `S3` -> `slice-3-plan-task-and-quality-gate-lock-in.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-4-closeout.md`
