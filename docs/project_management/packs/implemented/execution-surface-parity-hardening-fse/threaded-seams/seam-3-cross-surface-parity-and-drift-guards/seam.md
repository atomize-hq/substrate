---
seam_id: SEAM-3
seam_slug: cross-surface-parity-and-drift-guards
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-3-cross-surface-parity-and-drift-guards.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-2-closeout.md
  required_threads:
    - THR-01
    - THR-02
  stale_triggers:
    - `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, or the WPEP playbook/smoke assets drift away from the published `SEAM-1` or `SEAM-2` closeouts
    - replay-routing, tracing-validation, or abnormal-terminal-loss regression surfaces stop proving the same contracts described in the operator docs
    - `SEAM-1` or `SEAM-2` closeout stale-trigger subjects change after this seam decomposes
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
# SEAM-3 - Cross-surface parity and drift guards

## Seam Brief (Restated)

- **Goal / value**: turn the published contracts from `SEAM-1` and `SEAM-2` into durable docs, smoke guidance, and regression guards so maintainers do not have to rediscover the same execution-surface truth.
- **Type**: conformance
- **Scope**
  - In:
    - align `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, and related contract docs to the published `THR-01` and `THR-02` closeouts
    - update the active WPEP pack's playbook and smoke assertions so they describe landed behavior rather than provisional assumptions
    - add or adjust regression surfaces that keep replay-routing and abnormal-terminal-loss behavior aligned with the docs
    - capture downstream stale triggers when a publication or regression surface drifts from the landed runtime truth
  - Out:
    - new runtime behavior beyond what `SEAM-1` and `SEAM-2` have already landed
    - opportunistic cleanup outside replay, tracing, and shell-resilience conformance
- **Touch surface**: `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`, `docs/project_management/packs/active/world_process_exec_tracing_parity/smoke/_core.sh`, `docs/REPLAY.md`, `docs/TRACE.md`, `docs/USAGE.md`, and the regression suites that pin replay-routing and REPL abnormal-exit behavior.
- **Verification**:
  - every changed doc or smoke assertion must map back to a published upstream contract
  - replay-routing and abnormal-terminal-loss regression surfaces must pin the same truth the docs publish
  - any remaining drift must be recorded explicitly as a stale trigger or remediation, not left implicit

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`

## Seam-exit gate plan

- **Planned location**: `S99` (`slice-99-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: `SEAM-3` is the consumer seam that converts the published upstream contracts into durable cross-surface guidance and drift guards. Downstream work should not rely on inferred conformance.
- **Expected contracts to publish**: none beyond conformance evidence and stale-trigger accounting
- **Expected threads to publish / advance**: `THR-01` and `THR-02` remain `revalidated` on this seam and should terminate in explicit closeout evidence
- **Expected closeout evidence**: landed docs/playbook/smoke/regression updates plus explicit stale-trigger accounting for any future drift

## Slice index

- `S1` -> `slice-1-cross-surface-doc-lock-in.md`
- `S2` -> `slice-2-wpep-playbook-and-smoke-alignment.md`
- `S3` -> `slice-3-regression-and-drift-guards.md`
- `S99` -> `slice-99-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-3-closeout.md`
