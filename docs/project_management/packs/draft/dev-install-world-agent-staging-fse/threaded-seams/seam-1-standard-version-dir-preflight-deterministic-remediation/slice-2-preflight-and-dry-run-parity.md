---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - accepted staged path set or sufficiency rule changes
    - standard version-dir derivation changes
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
  - C-01
  - C-03
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S2 - Implement accepted staged path preflight + dry-run parity

- **User/system value**: `substrate world enable` becomes deterministic in the standard version-dir flow: it can only proceed when an accepted staged `world-agent` exists, and `--dry-run` truthfully reports readiness without mutating state.
- **Scope (in/out)**:
  - In: compute version dir (override unset), check accepted staged paths in order, select the primary path, and bind `--dry-run` success to that preflight.
  - Out: remediation wording and strict “no helper before remediation” enforcement details (handled in S3).
- **Acceptance criteria**:
  - With override unset, preflight checks `<version_dir>/bin/world-agent` then `<version_dir>/bin/linux/world-agent`.
  - Either path is sufficient.
  - `--dry-run` exits `0` only when an accepted path exists, otherwise exits `3`.
  - `--dry-run` does not write config/env/log/systemd state and does not invoke the helper.
  - The behavior is implemented consistently for both code paths (`run_enable` and `run_enable_with_provision_deps`).
- **Dependencies**: S1 contract-definition rules.
- **Verification**:
  - Add/extend `crates/shell/tests/world_enable.rs` cases to assert:
    - missing artifact + no override + `--dry-run` -> exit `3`, no helper log, no config/env writes
    - present artifact + no override + `--dry-run` -> exit `0`, no helper log, no config/env writes
    - override set still behaves as carve-out (no standard version-dir preflight guarantees)
- **Rollout/safety**: Fail closed on missing artifacts; preserve Windows unsupported posture.
- **Review surface refs**:
  - `../../review_surfaces.md` (R1/R2)
  - `../../threading.md` (`C-01`, `C-03`)

#### S2.T1 - Add accepted staged artifact discovery helper

- **Outcome**: A single function answers: “is there an accepted staged `world-agent`, and which path wins?”
- **Inputs/outputs**:
  - Inputs: `runner/paths.rs::resolve_version_dir()` and `threading.md` C-01 ordering
  - Outputs: `Option<PathBuf>` (or equivalent) for the selected accepted path
- **Thread/contract refs**: `THR-01` (`C-01`, `C-03`)
- **Implementation notes**:
  - Keep the search order fixed and explicit; avoid globbing.
  - Return the selected primary path for future logging/trace/diagnostics if needed (without implying execution happens here).
- **Acceptance criteria**: When both exist, selects `<version_dir>/bin/world-agent`.
- **Test notes**: Unit-testable helper in `runner/paths.rs` plus integration assertions in `crates/shell/tests/world_enable.rs`.
- **Risk/rollback notes**: If search order drifts, downstream staging work can satisfy the wrong path set.

Checklist:
- Implement: add discovery helper + plumb into runner preflight
- Test: add unit/integration coverage for order + sufficiency
- Validate: ensure override carve-out is preserved
- Cleanup: keep helper APIs private to world-enable runner unless reuse is proven

#### S2.T2 - Wire preflight into `--dry-run` path

- **Outcome**: `--dry-run` uses the same preflight as non-dry-run and returns truthfully.
- **Inputs/outputs**:
  - Inputs: `WorldEnableArgs` + derived `version_dir`
  - Outputs: exit status + printed plan
- **Thread/contract refs**: `THR-01`, `THR-02`
- **Implementation notes**:
  - Ensure dry-run checks happen before any log initialization or helper invocation.
  - Keep plan output stable and not dependent on helper output.
- **Acceptance criteria**: dry-run is side-effect-free and exit code matches contract.
- **Test notes**: Assert helper log remains absent (`fixture.log_contents().is_none()`).
- **Risk/rollback notes**: If dry-run becomes side-effectful, it breaks operator trust and conformance evidence.

Checklist:
- Implement: gate dry-run success on preflight
- Test: new dry-run test cases
- Validate: confirm no new files under `--home` or default home
- Cleanup: none
