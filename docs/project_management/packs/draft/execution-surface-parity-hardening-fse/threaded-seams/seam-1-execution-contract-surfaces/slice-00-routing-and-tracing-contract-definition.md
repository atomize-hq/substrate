---
slice_id: S00
seam_id: SEAM-1
slice_kind: contract_definition
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
---
### S00 - Routing and tracing contract definition

- **User/system value**: execution work proceeds from one owned contract bundle instead of ad-hoc replay heuristics and ambiguous tracing-validation language.
- **Scope (in/out)**:
  - In: define `C-01` and `C-02` rules, boundaries, and verification checklists tightly enough that the producer seam can later pass its contract gate.
  - Out: final publication evidence and closeout accounting, which belong to `S99`.
- **Acceptance criteria**:
  - `C-01` names the canonical four replay-routing outcomes, their inputs, and the producer-side test matrix.
  - `C-02` names the execution-mode behavior matrix, the safe trace omission rule, and what WPEP Case B must assert.
  - Both contracts include explicit target files/tests and pass-fail conditions.
- **Dependencies**:
  - `threading.md` contract registry for `C-01`, `C-02`, and `THR-01`
  - `crates/shell/src/execution/policy_snapshot.rs`
  - `crates/replay/src/replay/executor.rs`
  - `crates/shell/src/execution/manager.rs`
  - `crates/shell/src/scripts/bash_preexec.rs`
  - `docs/project_management/packs/active/world_process_exec_tracing_parity/manual_testing_playbook.md`
- **Verification**:
  - Contract rules below must map directly to executable tests or deterministic validation probes in later slices.
- **Rollout/safety**: no new user-facing flags or config keys; preserve canonical trace safety while removing routing drift.
- **Review surface refs**: `../../review_surfaces.md` R2 and R3

#### C-01 contract rules

1. **Authority**: replay must consume the same canonical world-network routing rules already owned by `crates/shell/src/execution/policy_snapshot.rs`.
2. **Four-case matrix**:
   - gate off plus restrictive `net_allowed` => no requested isolation
   - gate on plus allow-all `["*"]` => no requested isolation
   - gate on plus deny-all `[]` => requested isolation with an empty allowlist
   - gate on plus restrictive allowlist => requested isolation with canonical domains
3. **Parity requirement**: both local replay and agent-backed replay derive `policy_snapshot.net_allowed` and `world_network` from the same contract, not replay-local heuristics.
4. **Verification checklist**:
   - unit tests prove canonicalization of restrictive allowlists and empty-allowlist handling
   - replay tests prove local and agent-backed request construction preserve the same routing outcome
   - docs name the four cases without introducing a fifth replay-specific branch

#### C-02 contract rules

1. **Authority**: one behavior matrix must answer, by execution mode and platform, when `world_process_*` telemetry is expected, when `builtin_command` is expected, and whether `SUBSTRATE_ENABLE_PREEXEC` is honored, ignored, or best-effort.
2. **Safe trace posture**: canonical trace must continue omitting raw builtin or preexec command bodies; `builtin_command` keeps `command_omitted: true`.
3. **Case B requirement**: the active WPEP pack's Case B must assert the chosen matrix directly, not a proxy signal that can drift from runtime behavior.
4. **Verification checklist**:
   - runtime or harness tests show the chosen matrix outcome for the relevant modes/platforms
   - manual playbook and smoke guidance cite the same assertions
   - `docs/TRACE.md` and `docs/internals/env/inventory.md` preserve the safe-by-default omission rule

#### S00.T1 - Record the concrete routing contract for `C-01`

- **Outcome**: later implementation slices can wire replay onto the canonical helper without re-deciding empty-allowlist or allow-all behavior.
- **Inputs/outputs**:
  - Inputs: `threading.md`, `crates/shell/src/execution/policy_snapshot.rs`, `docs/REPLAY.md`
  - Outputs: locked four-case rules, helper expectations, and named test locations for replay parity
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - prefer a minimal shared helper or shared request-construction path over duplicating shell logic inside replay
  - keep local and agent-backed replay on the same contract surface
- **Acceptance criteria**:
  - every routing case has one expected `isolate_network` / `allowed_domains` outcome and one concrete verification surface
- **Test notes**:
  - `crates/shell/src/execution/policy_snapshot.rs` unit coverage remains authoritative for the matrix
  - replay-focused tests should live near `crates/replay/src/replay/executor.rs`
- **Risk/rollback notes**:
  - if helper extraction broadens surface area too far, preserve the contract and reduce the extraction boundary, rather than reintroducing replay-local heuristics

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: cross-check all four cases against the existing shell helper tests
- Cleanup: none

#### S00.T2 - Record the concrete tracing behavior matrix for `C-02`

- **Outcome**: later slices can change runtime and validation assets in one publication unit without guessing what Case B or operator docs mean.
- **Inputs/outputs**:
  - Inputs: `threading.md`, `crates/shell/src/execution/manager.rs`, `crates/shell/src/scripts/bash_preexec.rs`, WPEP playbook/smoke assets, `docs/TRACE.md`
  - Outputs: explicit mode-by-platform matrix, WPEP Case B assertion target, and named verification surfaces
- **Thread/contract refs**: `THR-01`, `C-02`
- **Implementation notes**:
  - treat `builtin_command` omission as non-negotiable
  - keep the matrix narrow to the execution surfaces in scope for this pack
- **Acceptance criteria**:
  - every mode/platform cell answers the three contract questions (`world_process_*`, `builtin_command`, `SUBSTRATE_ENABLE_PREEXEC`)
  - Case B expectation is stated plainly enough to test and document
- **Test notes**:
  - manual playbook + smoke assets are authoritative validation surfaces
  - any runtime assertions should be added where manager/dispatch behavior is already tested
- **Risk/rollback notes**:
  - if the matrix reveals a broader preexec product decision, keep the matrix explicit here and defer new public controls to a later seam

Checklist:
- Implement: N/A in this slice
- Test: N/A in this slice
- Validate: cross-check against current WPEP playbook and trace omission rules
- Cleanup: none
