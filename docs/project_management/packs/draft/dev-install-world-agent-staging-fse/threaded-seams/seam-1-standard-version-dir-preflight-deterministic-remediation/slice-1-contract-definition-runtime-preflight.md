---
slice_id: S1
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
  - C-01
  - C-02
  - C-03
contracts_consumed: []
open_remediations:
  - REM-001
candidate_subslices: []
---
### S1 - Contract definition: standard version-dir preflight + ordering

- **User/system value**: Downstream seams can implement and validate staging + conformance work against a single, concrete runtime contract surface (paths, ordering, exit codes, and remediation visibility).
- **Scope (in/out)**:
  - In: make `C-01`/`C-02`/`C-03` concrete enough for execution and downstream revalidation.
  - Out: landing the code changes themselves (handled in S2/S3).
- **Acceptance criteria**:
  - The contracts are stated as executable rules (exact paths, ordering, sufficiency, exit code, and “no writes” invariants).
  - Verification plan names concrete test locations and asserts operator-visible stderr content (not helper logs).
  - Override carve-out is explicit and is not accidentally covered by standard version-dir guarantees.
- **Dependencies**: none (producer seam contract-definition slice).
- **Verification**:
  - `crates/shell/tests/world_enable.rs` covers standard version-dir behavior (no override), including dry-run parity.
  - Contract assertions do not rely on helper output (ties to `REM-001`).
- **Rollout/safety**:
  - Fail closed on missing artifacts.
  - Keep `--dry-run` side-effect-free.
- **Review surface refs**:
  - `../../review_surfaces.md` (R1/R2)
  - `../../threading.md` (C-01..C-03, THR-01/02)
  - `../../governance/remediation-log.md` (REM-001)

#### Contract rules (concrete)

These are the binding contract statements this seam must land and later publish in closeout.

- **C-01 (paths + derivation + sufficiency)**:
  - In the standard version-dir flow (override unset), resolve `<home>/bin/substrate`, canonicalize it, and derive `version_dir = parent(parent(canonical_substrate))`.
  - Check accepted staged executable paths in fixed order:
    1) `<version_dir>/bin/world-agent`
    2) `<version_dir>/bin/linux/world-agent`
  - Either accepted path is sufficient (do not require both).
  - When both exist, the root `bin/world-agent` path is the selected/primary path.
- **C-02 (deterministic missing-artifact failure)**:
  - If neither accepted path exists, exit `3` **before** helper launch, provisioning, health verification, config writes, or manager-env writes.
  - Render exactly one operator-visible remediation block on stderr that includes:
    - both accepted staged paths (literal path strings)
    - `scripts/substrate/dev-install-substrate.sh --no-world`
    - `cargo build -p world-agent`
  - `--dry-run` shares the same preflight and remediation block; it exits `0` only when an accepted artifact exists.
- **C-03 (state root + ordering + override carve-out)**:
  - Resolve state root as `--home` → `SUBSTRATE_HOME` → `~/.substrate`.
  - `--dry-run` writes no config, helper logs, manager-env exports, or systemd state.
  - Missing-artifact paths write nothing.
  - Non-dry-run success keeps `world.enabled=false` until helper execution and health verification both succeed.
  - `SUBSTRATE_WORLD_ENABLE_SCRIPT` remains an explicit carve-out and is outside the standard version-dir preflight guarantee.

#### S1.T1 - Map contract rules to implementation loci

- **Outcome**: Each contract rule is mapped to an implementation point + a test assertion.
- **Inputs/outputs**:
  - Inputs: `threading.md` C-01..C-03, `runner.rs`, `runner/paths.rs`, and existing tests.
  - Outputs: a checklist used by S2/S3 to ensure no contract drift.
- **Thread/contract refs**: `THR-01`, `THR-02`; `C-01`, `C-02`, `C-03`.
- **Implementation notes**:
  - Keep preflight in `runner.rs` (not helper), per `REM-001`.
  - Use `runner/paths.rs::resolve_version_dir()` as the canonical derivation anchor unless contract explicitly changes it.
- **Acceptance criteria**: The mapping is unambiguous: every contract bullet has an explicit “where” and “how tested”.
- **Test notes**: Prefer integration tests in `crates/shell/tests/world_enable.rs` that assert both exit status and stderr text.
- **Risk/rollback notes**: If contract text is ambiguous, downstream revalidation becomes non-deterministic.

Checklist:
- Implement: N/A (contract slice)
- Test: N/A (contract slice)
- Validate: confirm `REM-001` is explicitly addressed in S3
- Cleanup: none
