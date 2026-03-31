---
slice_id: S3
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - helper candidate order changes
    - helper-missing guidance changes
    - `substrate world enable` flag-surface changes
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
  - THR-02
contracts_produced:
  - C-01
contracts_consumed:
  - C-02
open_remediations:
  - REM-001
candidate_subslices: []
---
### S3 - Lock helper discovery order and fail-closed validation

- **User/system value**: `substrate world enable` keeps one stable discovery order and one stable failure posture, so developers can rely on the prefix-staged helper after install and downstream conformance work has one deterministic branch structure to verify.
- **Scope (in/out)**:
  - In: helper candidate ordering in `paths.rs`, fail-closed missing-helper behavior, and `world_enable.rs` coverage for prefix precedence and CLI flag posture
  - Out: staging implementation details, uninstall cleanup, and any broader world provisioning logic
- **Acceptance criteria**:
  - helper lookup order is exactly `SUBSTRATE_WORLD_ENABLE_SCRIPT` -> prefix helper -> inferred version-dir helper
  - when env override is absent and both bundled candidates exist, the prefix helper wins
  - missing-helper behavior remains fail-closed
  - `substrate world enable --home` stays valid and `--prefix` stays invalid
  - test coverage locks the intended order and failure posture tightly enough that later cleanup or conformance seams do not need to rediscover it
- **Dependencies**:
  - `slice-1-freeze-durable-bundle-contracts.md`
  - `slice-2-dev-install-durable-bundle-staging.md`
  - `review.md`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/tests/world_enable.rs`
- **Verification**:
  - pass condition: `paths.rs` and `world_enable.rs` encode the same helper-order and flag contract that `SEAM-3` will later consume as landed truth
- **Rollout/safety**:
  - preserve fail-closed posture rather than introducing best-effort fallback
  - keep operator guidance aligned with a dev-installed prefix helper and do not imply full production-bundle or full macOS provisioning semantics
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` R1
  - `../../review_surfaces.md` R2

#### S3.T1 - Update helper candidate resolution in `paths.rs`

- **Outcome**: the runtime helper lookup matches the durable bundle contract instead of depending on the inferred version-dir path first.
- **Thread/contract refs**:
  - `THR-02`
  - `C-01`
  - `C-02`
- **Acceptance criteria**:
  - env override remains highest priority
  - prefix helper is checked before the inferred version-dir helper
  - missing candidates remain fail-closed

#### S3.T2 - Freeze flag and failure coverage in `world_enable.rs`

- **Outcome**: regression coverage protects helper precedence, fail-closed posture, and the `--home`/invalid `--prefix` CLI contract.
- **Thread/contract refs**:
  - `THR-02`
  - `C-01`
- **Acceptance criteria**:
  - tests assert prefix-helper precedence when both staged and inferred candidates exist
  - tests keep `--home` valid and `--prefix` invalid
  - tests or assertions capture helper-missing wording closely enough to resolve `REM-001` during implementation or closeout

## Remediation focus

- `REM-001` stays attached to this slice until implementation confirms the helper-missing remediation text matches staged-prefix reality.
- ADR-0035 overlap no longer blocks promotion after the 2026-03-30 revalidation, but any later shared-script drift should still be recorded as a downstream stale trigger for `THR-02`.
- If implementation changes helper-order or user-facing failure wording beyond the frozen contract, record that as a downstream stale trigger for `THR-02`.
