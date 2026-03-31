---
slice_id: S3
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - shim doctor disabled snapshots do not preserve source attribution
    - health summary wording diverges from the doctor text contract
    - `--no-world` loses `cli_flag` identity in disabled-mode health flows
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S3 - Health parity and disabled-path plumbing

- **User/system value**: `substrate health` exposes the same disable truth as doctor surfaces in both JSON and human mode, including the tricky `--no-world` path that currently loses structured provenance.
- **Scope (in/out)**:
  - In: preserve reason/source through shim-doctor collection, surface the top-level fields in `health --json`, and print the exact `C-01` reason string in human mode when disabled.
  - Out: downstream closeout and publication accounting (S4).
- **Acceptance criteria**:
  - `substrate health --json` emits the same top-level fields and omit rules as doctor JSON.
  - Human-mode `substrate health` prints the exact disable-attribution line before the existing disabled guidance.
  - `--no-world` preserves `cli_flag` attribution end-to-end instead of degrading to a generic disabled status.
  - Existing world-disabled guidance and summary semantics remain intact apart from the additive attribution line/fields.
- **Dependencies**:
  - `slice-1-contract-definition-json-health-disable-attribution.md`
  - structured helper from S2
  - `crates/shell/src/builtins/shim_doctor/report.rs`
  - `crates/shell/src/builtins/health.rs`
- **Verification**:
  - Extend `crates/shell/tests/shim_health.rs` for disabled JSON fields, enabled omission, and human-mode message parity.
  - Extend `crates/shell/tests/shim_doctor.rs` only if `ShimDoctorReport` or its disabled snapshots must grow new fields to preserve attribution into health.
- **Rollout/safety**:
  - Keep existing “World backend: disabled” and “World deps: skipped” guidance intact.
  - Treat health parity drift as a release blocker because operators will otherwise see conflicting disable explanations across adjacent surfaces.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### S3.T1 - Preserve disable attribution through shim-doctor and health

- **Outcome**: health has access to the same reason/source contract even when it routes through disabled shim-doctor snapshots instead of directly calling doctor entrypoints.
- **Inputs/outputs**:
  - Inputs: CLI world flags, diagnostics-world resolution, disabled shim-doctor snapshot/report state, and the shared reason/source helper.
  - Outputs: top-level health JSON fields plus one exact human-mode attribution line when disabled.
- **Thread/contract refs**: `THR-01`, `THR-02`; `C-01`, `C-02`, `C-03`.
- **Implementation notes**:
  - `resolve_diagnostics_world_enabled(...)` is currently a boolean-only boundary; add the smallest plumbing needed so health can preserve the winning source without creating a second precedence path.
  - If new report fields are introduced, keep them additive and scoped to disable attribution so `shim doctor` consumers do not pay for unrelated schema churn.
  - Human-mode health should reuse the exact reason string rather than formatting its own paraphrase.
- **Acceptance criteria**: `substrate health --no-world` proves `cli_flag` specifically, not just “disabled”.
- **Test notes**: add focused disabled-mode tests for CLI, workspace, and source-unknown paths; preserve current enabled-mode and invalid-config behavior.
- **Risk/rollback notes**: if health cannot preserve upstream truth safely, the fix belongs in shim-doctor plumbing, not in health-specific heuristics.

Checklist:
- Implement: preserve reason/source through shim-doctor disabled paths; emit top-level health fields; print exact human-mode line
- Test: `shim_health.rs` parity coverage and any needed `shim_doctor.rs` report-shape checks
- Validate: confirm disabled guidance remains and the new line appears only when disabled
- Cleanup: remove duplicated disable-message formatting if introduced during plumbing
