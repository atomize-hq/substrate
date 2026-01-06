# Slice Closeout Gate Report — env_var_taxonomy_and_override_split / EV0

Date (UTC): 2026-01-05T14:36:34Z

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/env_var_taxonomy_and_override_split/`

Slice spec:
- `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior: config-shaped legacy `SUBSTRATE_*` values could be treated as operator override inputs in effective config resolution.
- New behavior: effective config resolution consults only `SUBSTRATE_OVERRIDE_*` for env override inputs; config-shaped `SUBSTRATE_*` values are exported state only.
- Why: prevent “stale exports” from bypassing the resolver and make env inputs vs exported state unambiguous across platforms.
- Links:
  - `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`
  - `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none during execution)

## Checks Run (Evidence)

- `make integ-checks`: pass
  - Includes: `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets`
  - Worktree: `wt/ev0-override-split-integ`
  - Final merge commit: `0da831b0bc9a6ec71ed0f8d8476cf18e82098b63`
- `make feature-smoke FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split" PLATFORM=all RUNNER_KIND=self-hosted WORKFLOW_REF="feat/env_var_taxonomy_and_override_split" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`: pass (run id `20718597240`)

## Repo-Wide Grep/Audit (Required Evidence)

This slice requires an explicit audit to ensure no commands bypass effective config resolution by treating config-shaped legacy `SUBSTRATE_*` values as behavior-changing inputs.

Commands run (verbatim):
- `rg -n "SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)" -S crates src scripts`
- `rg -n "env::var(_os)?\\(\"SUBSTRATE_(WORLD(_ENABLED)?|ANCHOR_MODE|ANCHOR_PATH|CAGED|POLICY_MODE|SYNC_AUTO_SYNC|SYNC_DIRECTION|SYNC_CONFLICT_POLICY|SYNC_EXCLUDE)\"\\)" -S crates`

Summary:
- Broad scan: `471` matching lines across `69` files (pattern also matches non-scope `SUBSTRATE_WORLD_*` like `SUBSTRATE_WORLD_SOCKET`, `SUBSTRATE_WORLD_ID`, `SUBSTRATE_WORLD_FS_MODE`, and `SUBSTRATE_WORLD_DEPS_*`)
- Direct Rust `env::var` scan: `48` matching lines across `11` files (includes test modules outside `crates/*/tests/`)

Findings (must be exhaustive; list each hit and disposition):
- Fixed (rewired to effective config / `SUBSTRATE_OVERRIDE_*`):
  - None in final integ pass (all behavior-affecting inputs already consult effective config / `SUBSTRATE_OVERRIDE_*` per spec).
- Derived/exported-state consumption only (value set earlier in-process from effective config):
  - `crates/broker/src/mode.rs`
  - `crates/replay/src/replay/executor.rs`
  - `crates/replay/src/replay/helpers.rs`
  - `crates/replay/src/state.rs`
  - `crates/shell/src/builtins/world_deps/guest.rs`
  - `crates/shell/src/builtins/world_deps/runner.rs`
  - `crates/shell/src/builtins/world_deps/state.rs`
  - `crates/shell/src/builtins/world_enable/runner/helper_script.rs`
  - `crates/shell/src/builtins/world_enable/runner/manager_env.rs`
  - `crates/shell/src/builtins/world_enable/runner/paths.rs`
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/src/builtins/world_enable/runner/verify.rs`
  - `crates/shell/src/execution/env_scripts.rs`
  - `crates/shell/src/execution/invocation/plan.rs`
  - `crates/shell/src/execution/platform/mod.rs`
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
  - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
  - `crates/shell/src/execution/routing/path_env.rs`
  - `crates/shell/src/execution/routing/replay.rs`
  - `crates/shell/src/execution/routing/world.rs`
  - `crates/shell/src/execution/settings/mod.rs`
  - `crates/shell/src/execution/settings/runtime.rs`
  - `crates/shell/src/execution/socket_activation.rs`
  - `crates/shim/src/context.rs`
  - `crates/shim/src/exec/logging.rs`
  - `crates/telemetry-lib/src/correlation.rs`
  - `crates/trace/src/context.rs`
  - `crates/trace/src/span.rs`
  - `crates/world-agent/src/internal_exec.rs`
  - `crates/world-agent/src/service.rs`
  - `crates/world-mac-lima/src/lib.rs`
  - `crates/world/src/exec.rs`
  - `crates/world/src/guard.rs`
  - `crates/world-windows-wsl/src/backend.rs`
  - `scripts/linux/world-provision.sh`
  - `scripts/mac/lima-warm.sh`
  - `scripts/substrate/install-substrate.sh`
  - `scripts/substrate/world-deps.yaml`
  - `scripts/wsl/provision.sh`
- Test-only:
  - `crates/replay/tests/integration.rs`
  - `crates/replay/tests/planner_executor.rs`
  - `crates/shell/src/execution/invocation/tests.rs`
  - `crates/shell/src/execution/platform/macos.rs` (unit tests)
  - `crates/shell/src/execution/platform_world/windows.rs` (unit tests)
  - `crates/shell/src/execution/routing/builtin/tests.rs`
  - `crates/shell/src/execution/routing/dispatch/tests/host_replay.rs`
  - `crates/shell/src/execution/routing/dispatch/tests/linux_world.rs`
  - `crates/shell/src/execution/settings/tests.rs`
  - `crates/shell/tests/common.rs`
  - `crates/shell/tests/config_set.rs`
  - `crates/shell/tests/config_show.rs`
  - `crates/shell/tests/ev0_override_split.rs`
  - `crates/shell/tests/fail_closed_semantics.rs`
  - `crates/shell/tests/logging.rs`
  - `crates/shell/tests/policy_routing_semantics.rs`
  - `crates/shell/tests/replay_world.rs`
  - `crates/shell/tests/shell_behavior.rs`
  - `crates/shell/tests/shell_env.rs`
  - `crates/shell/tests/shim_health.rs`
  - `crates/shell/tests/socket_activation.rs`
  - `crates/shell/tests/support/mod.rs`
  - `crates/shell/tests/world_deps_layering.rs`
  - `crates/shell/tests/world_deps.rs`
  - `crates/shell/tests/world_enable.rs`
  - `crates/shell/tests/world_verify.rs`
  - `crates/shim/tests/integration.rs`
  - `crates/world-agent/tests/fs_mode.rs`
  - `crates/world-agent/tests/full_isolation_nonpty.rs`
  - `crates/world-agent/tests/full_isolation_pty.rs`

## Cross-Platform Smoke

Record run ids/URLs for required platforms:
- Linux: `https://github.com/atomize-hq/substrate/actions/runs/20718597240` (run id `20718597240`)
- macOS: `https://github.com/atomize-hq/substrate/actions/runs/20718597240` (run id `20718597240`)
- Windows: `https://github.com/atomize-hq/substrate/actions/runs/20718597240` (run id `20718597240`)

Key coverage (must be validated by smoke):
- `policy.mode` (via `SUBSTRATE_POLICY_MODE`)
- `world.caged` (via `SUBSTRATE_CAGED`)
- `world.anchor_mode` (via `SUBSTRATE_ANCHOR_MODE`)

If any platform-fix work was required:
- What failed: cross-platform smoke required follow-up fixes across runner/OS variants (Windows manager detection, deterministic env hashing for trace, CI smoke stability).
- What was changed: merged EV0 core + platform-fix integration branches into `ev-ev0-override-split-integ`, then FF merged into `feat/env_var_taxonomy_and_override_split` (final head `0da831b0bc9a6ec71ed0f8d8476cf18e82098b63`).
- Why the change is safe (guards, cfg, feature flags): fixes are platform-scoped or behavior-preserving (cfg-gated where applicable) and are covered by all-platform feature smoke run `20718597240`.

## Smoke ↔ Manual Parity

- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output

Notes:
- Smoke validates the required minimum key set (`policy.mode`, `world.caged`, `world.anchor_mode`) and the “workspace wins over overrides” precedence rule on all platforms.
