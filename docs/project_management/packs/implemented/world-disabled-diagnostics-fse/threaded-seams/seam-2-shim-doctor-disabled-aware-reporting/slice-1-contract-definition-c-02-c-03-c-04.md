---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-02
  - THR-03
  - THR-04
contracts_produced:
  - C-02
  - C-03
  - C-04
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S1 - Contract definition: C-02 / C-03 / C-04 disabled shim-doctor contracts

- **User/system value**: downstream seams stop inferring disabled state from legacy booleans or error strings; `shim doctor` publishes one canonical disabled-mode contract surface.
- **Scope (in/out)**:
  - In: define the disabled-mode world status enum, world-deps status enum plus omission rules, and the exact text-line contract for shim doctor.
  - Out: health summary behavior and docs alignment (owned by `SEAM-3`).
- **Acceptance criteria**:
  - `C-02` defines `.world.status` and the allowed values `healthy | needs_attention | disabled | unknown`.
  - `C-03` defines `.world_deps.status` and the allowed values `ok | error | skipped_disabled | unknown`, plus which legacy fields must be omitted in disabled mode.
  - `C-04` defines the exact disabled-mode shim-doctor lines and the rule that disabled/skipped states do not print `Error:` lines.
  - The verification checklist names concrete tests and no-probe assertions in `crates/shell/tests/shim_doctor.rs`.
- **Dependencies**:
  - Published classifier contract `C-01` from `../../governance/seam-1-closeout.md`
  - Existing report/output surfaces: `crates/shell/src/builtins/shim_doctor/report.rs`, `crates/shell/src/builtins/shim_doctor/output.rs`
  - Pack review surfaces: `../../review_surfaces.md`
- **Verification**:
  - The checklist below must be executable as tests, preferably in `crates/shell/tests/shim_doctor.rs`.
- **Rollout/safety**: JSON remains additive-only and enabled-mode failure visibility remains intact.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r3---touch-surface-map`

#### C-02 contract rules (world status)

1. Disabled mode must set `.world.status = "disabled"` and must not run world backend probes.
2. Enabled healthy mode may emit `.world.status = "healthy"`.
3. Enabled failure mode may emit `.world.status = "needs_attention"` and retain enabled-mode diagnostics.
4. Unknown is reserved for genuinely unavailable classification surfaces, not for disabled mode.

#### C-03 contract rules (world-deps status + omission)

1. Disabled mode must set `.world_deps.status = "skipped_disabled"` and must not compute applied world-deps state.
2. Disabled mode must omit legacy probe-derived fields that imply a real probe occurred, specifically disabled-path `world.error`, `world.details`, `world_deps.error`, and `world_deps.report`.
3. Enabled mode may emit `.world_deps.status = "ok"` or `"error"` according to the real applied-state result.

#### C-04 contract rules (exact text)

1. Disabled shim-doctor text must use these exact contract lines:
   - `World backend:`
   - `  Status: disabled`
   - `  Next: run \`substrate world enable\` to provision`
   - `World deps:`
   - `  Status: skipped (world disabled)`
2. Disabled or skipped states must not print `Error:` lines.
3. Enabled-mode failure text remains fail-visible and is not collapsed into disabled wording.

#### Verification checklist

- Disabled `substrate shim doctor --json` emits `.world.status = "disabled"` and `.world_deps.status = "skipped_disabled"`.
- Disabled mode omits forbidden legacy error/details/report fields.
- Disabled text path prints the exact `C-04` lines and no `Error:` lines.
- Enabled-mode fixture paths still emit probe-backed statuses and failure visibility when the world is enabled.
- A fixture that would fail probes if executed proves the disabled path does not call those probes.

#### S1.T1 - Publish the owned shim-doctor contracts before implementation

- **Outcome**: `C-02`, `C-03`, and `C-04` are concrete enough that `S2` can implement them without guesswork.
- **Inputs/outputs**: `threading.md`, `governance/seam-1-closeout.md`, `shim_doctor` report/output surfaces.
- **Thread/contract refs**: `THR-02`, `THR-03`, `THR-04`, `C-02`, `C-03`, `C-04`
- **Acceptance criteria**: the checklist above has named implementation/test surfaces in `S2`.

Checklist:
- Implement: N/A (contract-definition slice)
- Test: N/A (but must name concrete `shim_doctor` test targets in `S2`)
- Validate: cross-check against `threading.md` contract ownership and the published `C-01` handoff
- Cleanup: none
