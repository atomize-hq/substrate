---
pack_id: stabilize-dev-install-helper-discovery-seam-pack
pack_version: v1
pack_status: extracted
source_ref: stabilize-dev-install-helper-discovery.zip
execution_horizon:
  active_seam: SEAM-2
  next_seam: SEAM-3
---

# Scope Brief - stabilize-dev-install-helper-discovery

- **Goal**: Stabilize dev-install and dev-uninstall so a selected `$SUBSTRATE_HOME` contains a durable helper bundle, `substrate world enable` can reliably discover that bundle before falling back to the inferred version-directory helper, and dev-uninstall removes only repo-managed staged assets.
- **Why now**: The source pack isolates a brittle inner-loop failure: `cargo clean` can remove `<repo>/target/scripts/...`, which breaks helper discovery unless the runtime helper bundle is staged under `$SUBSTRATE_HOME`. Cleanup safety must land in the same feature boundary so repeated install/uninstall cycles remain auditable and non-destructive.
- **Primary user(s) + JTBD**: Operators and developers running dev installs across Linux and macOS who need `substrate world enable` to keep working after rebuild and clean cycles, plus maintainers who need deterministic cleanup and cross-platform evidence before merging.
- **In-scope**:
  - Stage a fixed runtime helper bundle under `$SUBSTRATE_HOME`.
  - Preserve the exact helper discovery order for `substrate world enable`: override env var, then prefix bundle helper, then inferred version-dir helper.
  - Keep `--home` as the valid flag for `substrate world enable` and keep `--prefix` invalid there.
  - Remove only repo-managed staged symlinks and manifest-tracked copied Linux guest binaries during dev-uninstall.
  - Preserve user-managed files and non-repo-managed symlinks with deterministic protected-path refusal behavior.
  - Keep Linux and macOS behavior validation explicit, while Windows remains compile-parity only.
- **Out-of-scope**:
  - Expanding Windows behavior beyond compile parity.
  - Introducing new config keys, config files, config precedence, or environment-precedence rules.
  - Changing production install layout under `$SUBSTRATE_HOME/versions/...`.
  - Broadening cleanup outside the fixed helper-bundle surface.
  - Expanding dev-install into ADR-0035-style world-agent staging or full macOS bundle parity.
- **Success criteria**:
  - After dev-install, the documented helper/runtime bundle exists under `$SUBSTRATE_HOME`, including helper scripts, `world-deps.yaml`, macOS Lima support files, and best-effort Linux guest binaries.
  - `substrate world enable` resolves helpers in the exact order `SUBSTRATE_WORLD_ENABLE_SCRIPT` → `$SUBSTRATE_HOME/scripts/substrate/world-enable.sh` → `<inferred version dir>/scripts/substrate/world-enable.sh`.
  - `cargo clean` no longer breaks helper discovery when the prefix helper still exists.
  - Dev-uninstall removes only repo-managed staged symlinks and manifest-tracked copied Linux guest binaries.
  - User-managed destinations remain in place and trigger the protected-path refusal class (`exit 5`) rather than destructive cleanup.
  - Linux/macOS smoke evidence and Windows compile-parity evidence stay explicit and aligned to the feature contract.
- **Constraints**:
  - The code touch set is intentionally narrow: `scripts/substrate/dev-install-substrate.sh`, `scripts/substrate/dev-uninstall-substrate.sh`, `crates/shell/src/builtins/world_enable/runner/paths.rs`, and `crates/shell/tests/world_enable.rs`.
  - The host launcher contract stays unchanged: `$SUBSTRATE_HOME/bin/substrate` continues to point at the live host build output under `<repo>/target/<profile>/substrate`.
  - The helper-missing posture stays fail-closed.
  - Cleanup must never use recursive deletion as a shortcut for managed-bundle cleanup.
  - macOS expectations remain limited to helper discovery and validation surfaces unless the bundle scope expands to stage additional release-root assets.
- **External systems / dependencies**:
  - `scripts/substrate/dev-install-substrate.sh`
  - `scripts/substrate/dev-uninstall-substrate.sh`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/tests/world_enable.rs`
  - `manual_testing_playbook.md`, `smoke/linux-smoke.sh`, `smoke/macos-smoke.sh`, `smoke/windows-smoke.ps1`
  - Shared exit-code taxonomy and related ADR guidance referenced by the source pack
  - Adjacent ADR-0035 work on overlapping install-script and helper-script surfaces
- **Known unknowns / risks**:
  - Multi-checkout installs targeting the same `$SUBSTRATE_HOME` can compete for the same staged helper paths.
  - Repo-managed symlink staging can become dangling if a checkout is moved or removed.
  - The helper-missing remediation text in `paths.rs` may remain production-oriented unless it is explicitly narrowed during seam-local review.
  - macOS helper discovery can be correct even when full provisioning would still need additional release-root assets; evidence must not overclaim support.
  - ADR-0035 overlap can stale the basis if it changes install-script or helper-script surfaces first.
- **Assumptions**:
  - Seam extraction is workflow-first rather than entity-first because the source pack already converged on a two-stage behavior change: land helper staging/discovery first, then cleanup safety.
  - `SEAM-2` is the active horizon seam after the SEAM-1 closeout; `SEAM-3` is the next seam and remains future depth until the cleanup closeout lands.
  - The source planning pack remains the authoritative input for this extraction; SEAM-2 now has seam-local planning, but no post-exec evidence exists yet.
  - Seam-exit concerns are inferred from the documented contracts, checkpoint plan, and known cross-queue overlaps rather than from landed code.
