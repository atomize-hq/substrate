---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - fixed staged path list changes
    - helper candidate order or CLI flag surface changes
    - managed-asset eligibility or manifest shape changes
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
  - C-02
  - C-03
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Freeze the durable bundle and helper-discovery contracts

- **User/system value**: downstream cleanup and conformance work can plan against one explicit durable-bundle contract instead of inferring path lists, helper precedence, or managed-asset rules from partially implemented scripts.
- **Scope (in/out)**:
  - In: the fixed staged path list, repo-managed versus manifest-tracked asset rules, helper resolution order, fail-closed posture, and `world enable` flag contract
  - Out: script implementation details, cleanup execution, smoke-wrapper landing, and broader macOS provisioning scope
- **Acceptance criteria**:
  - `C-02` names one exact fixed staged path surface under `$SUBSTRATE_HOME`
  - `C-03` makes the managed-asset boundary explicit enough for later cleanup work
  - `C-01` locks helper precedence, `--home` validity, invalid `--prefix`, and fail-closed behavior
  - the slice leaves no ambiguity about which parts of the bundle are symlinked versus best-effort copied
- **Dependencies**:
  - `review.md`
  - `../../threading.md`
  - `../../seam-1-durable-helper-bundle-staging-discovery.md`
- **Verification**:
  - pass condition: `scripts/substrate/dev-install-substrate.sh`, `paths.rs`, and `world_enable.rs` can land against one explicit contract set without reopening ownership or lookup-order questions
- **Rollout/safety**:
  - prevents downstream seams from widening cleanup ownership or helper-discovery behavior
  - keeps Windows and full macOS provisioning out of scope
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` R1
  - `../../review_surfaces.md` R2

#### S1.T1 - Freeze the durable staged bundle surface

- **Outcome**: one exact `C-02` bundle inventory is agreed before implementation.
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `C-02`
- **Acceptance criteria**:
  - the durable bundle includes:
    - `scripts/substrate/world-enable.sh`
    - `scripts/substrate/install-substrate.sh`
    - `scripts/substrate/world-deps.yaml`
    - `scripts/mac/lima-warm.sh`
    - `scripts/mac/lima/substrate.yaml`
    - `scripts/mac/lima/substrate-dev.yaml`
    - best-effort Linux guest binaries under `bin/linux/`
  - `$SUBSTRATE_HOME/bin/substrate` remains unchanged and is not pulled into the managed helper bundle contract

#### S1.T2 - Freeze helper discovery and managed-asset eligibility

- **Outcome**: `C-01` and `C-03` are concrete enough for staging and later cleanup/conformance work.
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `C-01`
  - `C-03`
- **Acceptance criteria**:
  - helper lookup order remains `SUBSTRATE_WORLD_ENABLE_SCRIPT` -> prefix helper -> inferred version-dir helper
  - missing helper candidates stay fail-closed
  - `substrate world enable --home` remains valid and `--prefix` remains invalid
  - repo-owned script, YAML, and macOS support assets remain repo-managed symlinks
  - Linux guest binaries are removable only when they remain repo-managed symlinks into local build outputs or when copied binaries are manifest-tracked

## Contract freeze

### `C-02` - Fixed durable runtime bundle surface

- The durable staged surface under `$SUBSTRATE_HOME` includes exactly:
  - `scripts/substrate/world-enable.sh`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/world-deps.yaml`
  - `scripts/mac/lima-warm.sh`
  - `scripts/mac/lima/substrate.yaml`
  - `scripts/mac/lima/substrate-dev.yaml`
  - best-effort Linux guest binaries under `bin/linux/`
- Additive or subtractive path-list changes are outside this seam and stale `THR-01` plus `THR-02`.
- The host launcher at `$SUBSTRATE_HOME/bin/substrate` is not part of the durable bundle contract and remains unchanged.

### `C-03` - Managed-asset eligibility

- Repo-owned script, YAML, and macOS support assets in the durable bundle stage as repo-managed symlinks.
- Linux guest binaries under `bin/linux/` are repo-managed only when they remain repo-managed symlinks into local build outputs or when copied from Lima and recorded in `.dev-install-managed/mac-linux-binaries.txt`.
- No other asset class is considered dev-managed by this seam.
- The contract does not authorize recursive cleanup or ownership inference beyond symlink target provenance plus manifest membership.

### `C-01` - Helper discovery and CLI contract

- `substrate world enable` resolves helpers in this exact order:
  - `SUBSTRATE_WORLD_ENABLE_SCRIPT`
  - `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh`
  - `<inferred version dir>/scripts/substrate/world-enable.sh`
- If none of those candidates exists, the command exits fail-closed.
- `--home` remains valid for `substrate world enable`.
- `--prefix` remains invalid for `substrate world enable`.
- Any helper-order, flag-surface, or helper-missing guidance change stales `THR-02`.

## Verification checklist for contract readiness

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Durable bundle inventory | `scripts/substrate/dev-install-substrate.sh` | one fixed path list is staged under `$SUBSTRATE_HOME` without widening scope |
| Managed-asset rules | `scripts/substrate/dev-install-substrate.sh` and later closeout evidence | symlinked assets and manifest-tracked copied binaries are distinguishable without ambiguity |
| Helper order | `crates/shell/src/builtins/world_enable/runner/paths.rs` | prefix helper wins over inferred version-dir when env override is absent |
| Flag and fail-closed behavior | `crates/shell/tests/world_enable.rs` | `--home` remains valid, `--prefix` remains invalid, and missing helpers still fail closed |
