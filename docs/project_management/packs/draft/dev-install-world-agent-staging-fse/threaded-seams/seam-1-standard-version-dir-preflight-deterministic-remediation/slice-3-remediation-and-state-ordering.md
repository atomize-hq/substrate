---
slice_id: S3
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - helper-output suppression or visible remediation path changes
    - world.enabled ordering or --home precedence changes
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
  - C-02
  - C-03
contracts_consumed: []
open_remediations:
  - REM-001
candidate_subslices: []
---
### S3 - Land deterministic remediation + enforce no-write ordering

- **User/system value**: Operators receive one visible remediation and deterministic exit classification *before* any privileged/provisioning/helper behavior; success ordering remains safe and reviewable.
- **Scope (in/out)**:
  - In:
    - Missing-artifact remediation block: minimum required content, stderr visibility, stable formatting
    - Exit `3` classification for missing-artifact in both dry-run and non-dry-run
    - Enforce ordering: no helper launch, provisioning, verification, config writes, manager-env writes, or helper-log initialization on missing-artifact paths
    - Ensure `world.enabled` remains `false` until helper execution + health verification succeed
  - Out:
    - Changing helper behavior itself (keep helper boundary intact)
    - Adding new env vars or flags
- **Acceptance criteria**:
  - Missing-artifact in standard version-dir flow emits remediation block on stderr and exits `3`.
  - The remediation block contains:
    - both accepted staged paths
    - `scripts/substrate/dev-install-substrate.sh --no-world`
    - `cargo build -p world-agent`
  - On missing-artifact (dry-run or non-dry-run), helper does not run and no state is written.
  - On success, `world.enabled` flips to `true` only after helper and verification succeed.
  - The behavior is implemented consistently for both code paths (`run_enable` and `run_enable_with_provision_deps`).
- **Dependencies**: S1 (contract statements), S2 (preflight wiring).
- **Verification**:
  - `crates/shell/tests/world_enable.rs` asserts:
    - stderr includes the remediation block strings
    - helper log is absent (helper did not run)
    - config and env exports are absent on missing-artifact and dry-run flows
    - config remains disabled when helper fails (existing behavior) and is preserved
- **Rollout/safety**: Fail closed; preserve override carve-out; do not broaden macOS/Windows guarantees.
- **Review surface refs**:
  - `../../threading.md` (`C-02`, `C-03`)
  - `../../governance/remediation-log.md` (`REM-001`)

#### S3.T1 - Add stderr remediation renderer in runner boundary

- **Outcome**: A single function produces the remediation block with stable minimum content.
- **Inputs/outputs**:
  - Inputs: derived `version_dir` + accepted staged path strings
  - Outputs: stderr text (no helper logs required)
- **Thread/contract refs**: `THR-02` (`C-02`)
- **Implementation notes**:
  - Do not rely on helper output; this is a runner-owned contract surface (`REM-001`).
  - Keep message content minimal but exact; tests should assert on required substrings, not full formatting.
- **Acceptance criteria**: Contains both path strings plus the two required command hints.
- **Test notes**: Assert stderr contains the four required substrings.
- **Risk/rollback notes**: If remediation text becomes invisible, conformance cannot trust the contract.

Checklist:
- Implement: renderer + callsites
- Test: missing-artifact stderr assertions
- Validate: confirm no helper output is required for operator remediation
- Cleanup: none

#### S3.T2 - Enforce strict no-write ordering on missing-artifact paths

- **Outcome**: Missing-artifact failures are “pure”: no helper, no logs, no config/env writes.
- **Inputs/outputs**:
  - Inputs: runner control flow around preflight, log init, helper run, verify, and config/env writes
  - Outputs: early return / `std::process::exit(3)` (or equivalent) prior to any mutation
- **Thread/contract refs**: `THR-02` (`C-02`), `THR-01` (`C-03`)
- **Implementation notes**:
  - Ensure the missing-artifact branch is taken before `initialize_log_file()` and `append_log_line()`.
  - Keep `--dry-run` missing-artifact behavior aligned (exit `3` + remediation; no state writes).
- **Acceptance criteria**: Tests can prove “no state writes” using:
  - no config file created under `--home`
  - no env export file written under `--home`
  - no helper log file created/modified
  - helper invocation log remains absent
- **Test notes**: Use the existing fixture’s `config_exists()`, `env_sh_exists()`, and `log_contents()` helpers.
- **Risk/rollback notes**: If order drifts, the entire “enable later” workflow becomes non-deterministic and violates the pack’s critical path.

Checklist:
- Implement: move/guard log init and helper run behind preflight success
- Test: missing-artifact path asserts no writes
- Validate: ensure existing “helper fails -> world remains disabled” behavior still holds
- Cleanup: avoid introducing new state-write surfaces
