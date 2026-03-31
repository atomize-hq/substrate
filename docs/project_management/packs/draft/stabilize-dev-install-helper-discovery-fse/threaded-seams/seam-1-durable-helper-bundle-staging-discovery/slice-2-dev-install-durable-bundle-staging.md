---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v2
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - fixed staged path list changes before landing
    - shared install-script surfaces move under ADR-0035 before promotion
    - managed-marker or manifest recording rules change
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
  - C-02
  - C-03
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S2 - Stage the durable helper bundle in dev install

- **User/system value**: developers get one durable bundle under `$SUBSTRATE_HOME` that survives `cargo clean`, and downstream cleanup work receives a clear managed surface instead of reverse-engineering whatever the installer happened to leave behind.
- **Scope (in/out)**:
  - In: staging the fixed script/YAML/macOS support surface under `$SUBSTRATE_HOME`, best-effort Linux guest binaries under `bin/linux/`, and manifest/marker recording that distinguishes managed assets
  - Out: helper lookup changes in `paths.rs`, uninstall behavior, protected-path refusal semantics, and broader release-root parity work
- **Acceptance criteria**:
  - `dev-install-substrate.sh` stages the fixed `C-02` bundle surface under the selected `$SUBSTRATE_HOME`
  - repo-managed script/YAML/macOS assets are staged as repo-managed symlinks
  - best-effort Linux guest binaries under `bin/linux/` stay within the named managed surface either as repo-managed symlinks into local build outputs or as manifest-tracked copied binaries cached from Lima
  - `$SUBSTRATE_HOME/bin/substrate` remains pointed at the live host build output
  - the durable bundle no longer depends on `<repo>/target/scripts/...` continuing to exist after install time
- **Dependencies**:
  - `slice-1-freeze-durable-bundle-contracts.md`
  - `review.md`
  - `scripts/substrate/dev-install-substrate.sh`
- **Verification**:
  - pass condition: after a dev install, the fixed durable surface exists under `$SUBSTRATE_HOME`, and the managed-vs-unmanaged distinction is explicit enough for later cleanup and closeout evidence
- **Rollout/safety**:
  - preserve user-managed destination safety and deterministic overwrite/refusal behavior
  - do not widen the staged surface beyond the contract-frozen paths
  - do not move or redefine the host launcher contract
- **Review surface refs**:
  - `review.md` R1
  - `review.md` R2
  - `../../review_surfaces.md` R1
  - `../../review_surfaces.md` R3

#### S2.T1 - Stage the fixed durable bundle tree

- **Outcome**: the helper/runtime files move from an implicit build-output dependency to an explicit durable tree under `$SUBSTRATE_HOME`.
- **Thread/contract refs**:
  - `THR-01`
  - `THR-02`
  - `C-02`
- **Acceptance criteria**:
  - each fixed script and macOS support file in `C-02` is staged under `$SUBSTRATE_HOME`
  - the prefix helper path exists independently of later `cargo clean` operations
  - staging logic does not redefine `$SUBSTRATE_HOME/bin/substrate`

#### S2.T2 - Record the managed asset boundary

- **Outcome**: the installer leaves enough provenance for downstream cleanup to distinguish repo-managed assets from user-managed ones.
- **Thread/contract refs**:
  - `THR-01`
  - `C-03`
- **Acceptance criteria**:
  - repo-managed symlink assets are recorded as such through their target provenance
  - copied Linux guest binaries are removable only when they appear in `.dev-install-managed/mac-linux-binaries.txt`
  - no other staged asset class becomes implicitly dev-managed

## Implementation notes

- Keep staging logic confined to `scripts/substrate/dev-install-substrate.sh`.
- Treat repo-managed symlink assets and manifest-tracked copied Linux guest binaries as distinct asset classes with different cleanup eligibility.
- Preserve deterministic refusal or overwrite behavior for the managed surface; do not rely on destructive cleanup shortcuts.

## Planned verification checklist

| Check | Planned location | Pass condition |
| --- | --- | --- |
| Fixed bundle staging | `scripts/substrate/dev-install-substrate.sh` | all `C-02` paths are staged under `$SUBSTRATE_HOME` |
| Durable helper survival | manual/dev-install flow plus later closeout evidence | prefix helper remains available after `cargo clean` removes repo build-output helpers |
| Managed symlink classification | staging metadata and closeout evidence | repo-managed symlink assets are explicitly identifiable |
| Managed Linux guest binaries | staging metadata and `bin/linux/` bundle output | symlinked local build outputs and manifest-tracked copied guest binaries are distinguishable from user-managed files |
