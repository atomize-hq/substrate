---
seam_id: SEAM-1
seam_slug: durable-helper-bundle-staging-discovery
type: capability
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - helper candidate order changes
    - fixed bundle path list changes
    - macOS release-root scope expands beyond helper discovery and dry-run proof
    - ADR-0035 changes shared install-script or helper-script surfaces
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S4
  status: passed
open_remediations: []
---

# SEAM-1 - Durable helper-bundle staging + discovery

- **Goal / value**: Make a dev install leave behind a durable helper/runtime bundle under `$SUBSTRATE_HOME` so `substrate world enable` stays operable after `cargo clean`, while preserving the exact helper lookup order and fail-closed posture.
- **Scope**
  - In:
    - stage the fixed runtime bundle under `$SUBSTRATE_HOME`
    - preserve the helper resolution order `SUBSTRATE_WORLD_ENABLE_SCRIPT` → prefix helper → inferred version-dir helper
    - keep `--home` valid and keep `--prefix` invalid on `substrate world enable`
    - keep the host launcher contract unchanged while the helper bundle becomes durable
  - Out:
    - production layout changes under `$SUBSTRATE_HOME/versions/...`
    - new config keys, config precedence, or environment-precedence rules
    - Windows behavior enablement
    - ADR-0035 world-agent staging or full macOS bundle parity
- **Primary interfaces**
  - Inputs:
    - `scripts/substrate/dev-install-substrate.sh --prefix/--profile --no-world --no-shims`
    - `SUBSTRATE_WORLD_ENABLE_SCRIPT`
    - `$SUBSTRATE_HOME`
    - `<inferred version dir>/scripts/substrate/world-enable.sh`
    - existing build outputs under `<repo>/target/<profile>/...`
  - Outputs:
    - prefix-staged helper/runtime bundle at the documented fixed paths
    - deterministic helper-order behavior in `paths.rs`
    - fail-closed helper-missing behavior and stable CLI flag surface
- **Key invariants / rules**:
  - `SUBSTRATE_WORLD_ENABLE_SCRIPT` remains the highest-priority helper candidate.
  - The prefix helper bundle must survive `cargo clean` even when `<repo>/target/scripts/...` disappears.
  - Script, YAML, and macOS support assets stage as repo-managed symlinks.
  - Linux guest binaries under `bin/linux/` are dev-managed only as repo-managed symlinks into local build outputs or as manifest-tracked copied outputs cached from Lima.
  - `$SUBSTRATE_HOME/bin/substrate` remains unchanged and keeps pointing at the live host build output.
  - Missing helper candidates remain a fail-closed condition rather than a best-effort fallback.
- **Dependencies**
  - Direct blockers:
    - no internal seam blocker
    - external revalidation if ADR-0035 changes shared install-script or helper-script surfaces first
  - Transitive blockers:
    - shared exit-code taxonomy and no-new-config-precedence policy
    - macOS release-root coupling stays narrowed to helper discovery and validation, not full provisioning parity
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - any future pack that broadens dev-install staging or “enable later” recovery behavior
- **Touch surface**:
  - `scripts/substrate/dev-install-substrate.sh`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/tests/world_enable.rs`
  - the staged bundle tree under `$SUBSTRATE_HOME/scripts/...` and `bin/linux/...`
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Expected proof points:
    - prefix helper and companion bundle files exist after dev-install
    - helper discovery prefers the prefix helper when both prefix and inferred version-dir helpers exist
    - helper discovery still works after `cargo clean`
    - `substrate world enable --prefix` is rejected while `--home` stays valid
    - Linux and macOS smoke wrappers can consume the landed helper-order behavior later
- **Risks / unknowns**:
  - Risk: multi-checkout installs can collide on the same `$SUBSTRATE_HOME/scripts/substrate/*` paths.
  - De-risk plan: keep overwrite and refusal behavior deterministic, publish the exact managed surface at seam exit, and force downstream revalidation when shared script surfaces move.
  - Risk: the helper-missing remediation message in `paths.rs` may remain too production-oriented for dev-install failures.
  - De-risk plan: capture the message and exit-class wording explicitly during seam-local review and treat wording drift as a `THR-02` revalidation trigger.
- **Rollout / safety**:
  - Fail closed when no helper candidate exists.
  - Preserve user-managed destination safety; do not introduce destructive overwrite shortcuts.
  - Keep Windows unchanged except for compile-parity validation downstream.
- **Closeout posture**:
  - This seam has left the forward window after `governance/seam-1-closeout.md` recorded `THR-01` and `THR-02` as published with `promotion_readiness: ready`.
  - The owned contract surface is now authoritative pack truth through `contract.md`, `decision_register.md`, and the seam closeout record rather than through provisional planning only.
  - Downstream seams that consume this handoff are `SEAM-2` and `SEAM-3`.
