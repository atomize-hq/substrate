---
slice_id: S1
seam_id: SEAM-3
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
  - THR-05
contracts_produced:
  - C-05
contracts_consumed:
  - C-01
  - C-02
  - C-03
open_remediations: []
candidate_subslices: []
---
### S1 - Contract definition: C-05 disabled health summary contract

- **User/system value**: downstream seams and docs stop inferring disabled health semantics from legacy errors; `substrate health` publishes one canonical disabled-aware summary contract.
- **Scope (in/out)**:
  - In: define the disabled summary null/omission rules, exact text lines, guidance suppression rules, and docs-alignment requirements for `substrate health`.
  - Out: shim-doctor runtime behavior and cross-platform smoke orchestration.
- **Acceptance criteria**:
  - `C-05` defines disabled summary JSON as `summary.world_ok = null`, omitted summary error fields, and empty world-deps arrays.
  - `C-05` defines the exact disabled-mode `substrate health` text lines and the suppression of enabled-world world-deps guidance.
  - Enabled-mode failure visibility remains explicit and is not collapsed into disabled wording.
  - The verification checklist names concrete `shim_health` tests and docs surfaces.
- **Dependencies**:
  - Published upstream contracts: `C-01`, `C-02`, `C-03`
  - Existing health surfaces: `crates/shell/src/builtins/health.rs`, `crates/shell/tests/shim_health.rs`
  - Docs surface: `docs/USAGE.md`
- **Verification**:
  - The checklist below must be executable as tests and doc assertions, preferably in `crates/shell/tests/shim_health.rs`.
- **Rollout/safety**: disabled mode remains non-error; enabled mode remains fail-visible.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r3---touch-surface-map`

#### C-05 contract rules (summary JSON)

1. When `.shim.world.status = "disabled"`, `summary.world_ok` must be `null`.
2. Disabled mode must omit `summary.world_error` and `summary.world_deps_error`.
3. Disabled mode must emit `summary.world_deps_missing = []` and `summary.world_deps_blocked = []`.
4. Disabled mode must not add world/world-deps probe failures to `summary.failures` solely because probes were skipped.

#### C-05 contract rules (exact text + guidance suppression)

1. Disabled `substrate health` text must use these exact lines:
   - `World backend: disabled`
   - `  Next: run \`substrate world enable\` to provision`
   - `World deps: skipped (world disabled)`
2. Disabled mode must not print enabled-world world-deps remediation guidance, including lines containing `substrate world deps current`.
3. Enabled-mode failure text remains fail-visible and may still report attention required.

#### Verification checklist

- Disabled `substrate health --json` emits `summary.world_ok = null`.
- Disabled summary omits `summary.world_error` and `summary.world_deps_error`.
- Disabled summary emits empty `world_deps_missing` / `world_deps_blocked` arrays.
- Disabled text prints the exact `C-05` lines and omits enabled-world world-deps remediation guidance.
- Enabled-mode fixture paths still emit fail-visible health output and guidance when the world is enabled and broken.
- `docs/USAGE.md` describes `.world.status` / `.world_deps.status` as the canonical machine-readable contract and matches landed disabled behavior.

#### S1.T1 - Publish the owned health-summary contract before implementation

- **Outcome**: `C-05` is concrete enough that `S2` can implement it without guesswork.
- **Inputs/outputs**: `threading.md`, `governance/seam-2-closeout.md`, `health.rs`, `shim_health.rs`, `docs/USAGE.md`
- **Thread/contract refs**: `THR-05`, `C-05`
- **Acceptance criteria**: the checklist above has named implementation/test/doc surfaces in `S2`.

Checklist:
- Implement: N/A (contract-definition slice)
- Test: N/A (but must name concrete `shim_health` and docs targets in `S2`)
- Validate: cross-check against `threading.md` contract ownership and the published `SEAM-2` handoff
- Cleanup: none
