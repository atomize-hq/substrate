# Spec: World Codex Direct-Member Bootstrap Compatibility Bridge Completion

Source authorities:
- [handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md](../handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md)
- [handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md](../handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md)
- [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md)
- [DESIGN-codex-world-home-auth-and-config-mapping.md](./DESIGN-codex-world-home-auth-and-config-mapping.md)
- [DESIGN-workspace-scoped-adapter-overlay-model.md](./DESIGN-workspace-scoped-adapter-overlay-model.md)
- [docs/contracts/gateway/runtime-parity.md](../docs/contracts/gateway/runtime-parity.md)
- [docs/internals/world/gateway_auth_handoff.md](../docs/internals/world/gateway_auth_handoff.md)
- [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)
- [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
- [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
- [`docs/USAGE.md`](../docs/USAGE.md)
Phase: `SPECIFY`  
Status: draft for review

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The active June 21, 2026 world-Codex failure is no longer best explained by a world-binding gap, a stale deployment gap, or a missing guest-runtime-binary gap; the linked handoffs instead narrow it to the direct member bootstrap seam.
2. The current direct `cli:codex-world` member-dispatch path will remain in place for at least one bounded transitional slice, so the next honest work is to make that path truthful enough to run without pretending it is the long-term gateway-front-door architecture.
3. The best current diagnosis from the June 21, 2026 handoffs is machine-profile-specific rather than universal Codex truth: on the diagnosed file-backed-auth machine profile, isolated `CODEX_HOME` plus seeded `auth.json` was enough for `codex login status`, while isolated `CODEX_HOME` plus seeded `auth.json` and user-level `config.toml` made `codex exec` succeed.
4. The repo does not yet have a fully Substrate-owned Codex model/provider inventory surface ready to replace host Codex config as the source for the direct-path compatibility bootstrap, so this slice may temporarily derive a **narrow non-secret user-level bootstrap subset** from host Codex config only as a compatibility bridge for the current file-backed direct-member bootstrap path.
5. This slice must not widen into general Codex config projection. In particular, it must not project or reconcile MCP servers, app runtimes, apps/connectors, hooks, rules, skills/plugins, custom agents, workspace `.codex` overlays, profile overlays, managed requirements/allowlists, logs, sessions, rollout state, caches, daemon files, or workspace-shared plugin state.
6. Auth authority remains separate from ordinary config projection. This slice may preserve the existing direct-member auth compatibility bridge, but it must not redefine the target architecture away from Substrate-owned auth delivery through the in-world gateway seam.
7. Slice `59` guest-runtime delivery and exact-backend allowlist behavior remain the already-landed floor; this slice must preserve those truths rather than reopen them.

If any of these are wrong, correct them before implementation.

## Objective

Complete the **bounded transitional compatibility bridge** for direct `cli:codex-world` member launch so the current world-member path stops failing on the diagnosed June 21, 2026 file-backed-auth machine profile where:

1. isolated `CODEX_HOME` with seeded `auth.json` is enough for `codex login status`,
2. but the same isolated home still fails `codex exec` because the default fallback model is unsupported,
3. and the best current diagnosis is that the missing input is narrow non-secret bootstrap config rather than additional auth material.

This slice must answer:

1. what narrow non-secret **user-level** Codex bootstrap config is allowed to cross the host-to-world compatibility boundary for the current file-backed direct member path,
2. how that config is materialized into isolated `CODEX_HOME` without promoting host `~/.codex` to architectural authority,
3. how the direct bridge stays exact-backend-gated and compatibility-only,
4. what fail-closed behavior applies when the bridge cannot derive a truthful bootstrap config,
5. and how the repo documents that this bridge is transitional and must later retire behind the gateway-front-door realization.

This slice does **not** land the generic projection framework, the Codex steady-state gateway cutover, workspace-overlay support, or future MCP/app-runtime/skills capability work.

## Tech Stack

- Rust workspace (`cargo`)
- `crates/world-service` direct member runtime bootstrap/materialization
- `crates/shell` exact-backend-gated member-dispatch env injection
- existing `unified-agent-api-codex` integration already used by the direct member path
- existing world-scoped runtime floor from Slice `59`
- `llm-last-mile/` spec/plan/tasks authority

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted tests:

```bash
cargo test -p world-service prepare_codex_runtime_env -- --nocapture
cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture
cargo test -p shell c3_internal_toolbox_run_world_task_fast_completion_still_streams_registered_task_run_id_before_terminal_result -- --nocapture
```

Focused repo-truth checks:

```bash
rg -n "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME|CODEX_HOME|config\\.toml|auth\\.json|credentials\\.json" \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service \
  /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs
```

Manual smoke floor:

```bash
~/.substrate/bin/substrate agent start --backend cli:codex-world --scope world --prompt 'Reply with WORLD READY only.' --json
```

Manual closeout smoke:

```bash
# Re-run the exact public bootstrap smoke from the June 21, 2026 user transcript
# and confirm that from_the_world_worker.md is created.
```

## Project Structure

```text
crates/shell/src/execution/routing/dispatch/world_ops.rs
  Shell-side exact-backend-gated member-dispatch request builder. Owns whether the
  runtime-internal host seed-home hint is injected for cli:codex-world.

crates/world-service/src/member_runtime.rs
  Direct world-member runtime launcher. Owns isolated CODEX_HOME creation, bounded
  compatibility artifact materialization, and stripping internal hints before child spawn.

crates/world-service/Cargo.toml
  May need only minimal dependency adjustment if bounded config parsing/rendering needs a
  crate that is not already available.

docs/USAGE.md
  Public operator note for the direct-member compatibility bridge and its bounded scope.

handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md
handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md
  The live diagnosis and proof floor this slice must preserve.

llm-last-mile/
  Spec/plan/tasks authority for this bounded transitional slice.
```

## Code Style

Follow the repo’s Rust posture:

- `Result<T, anyhow::Error>` with `Context(...)`
- exact-backend fail-closed branching
- small helper functions for bootstrap artifact materialization
- no ambient fallthrough from “missing narrow compatibility input” to “let Codex guess from defaults”

Illustrative style:

```rust
fn render_codex_bootstrap_config(
    seed_home: &Path,
    target_home: &Path,
) -> Result<(), anyhow::Error> {
    let bootstrap = read_bootstrap_startup_subset(seed_home)
        .context("read bounded Codex bootstrap config from host seed home")?;

    write_bootstrap_config_toml(target_home, &bootstrap)
        .context("write bounded compatibility config.toml into isolated CODEX_HOME")?;

    Ok(())
}
```

## Testing Strategy

Frameworks and test levels:

1. unit tests in `crates/world-service` for isolated `CODEX_HOME` preparation,
2. focused shell tests for exact-backend allowlist gating,
3. targeted regression coverage for direct `run_world_task` dispatch behavior,
4. manual smoke proof with rebuilt installed binaries.

Coverage expectations for this slice:

1. prove the direct member path on the diagnosed profile no longer behaves as “auth only,”
2. prove the bridge stays narrow and internal,
3. prove exact-backend gating remains intact,
4. prove the live public bootstrap smoke succeeds after rebuild/redeploy.

If an automated test cannot faithfully execute external Codex behavior hermetically, at minimum the slice must still:

1. unit-test bounded config rendering/materialization semantics,
2. preserve the documented isolated-home proof in manual verification,
3. and avoid claiming broader automation coverage than actually exists.

## Boundaries

- Always:
  - keep the slice bounded to the direct `cli:codex-world` member bootstrap bridge
  - keep auth and non-secret bootstrap config conceptually separate even if both materialize into isolated `CODEX_HOME`
  - keep exact backend allowlist truth authoritative for host credential/config reads
  - materialize only the minimum non-secret **user-level** bootstrap config required to preserve truthful startup on the diagnosed profile, at minimum model plus directly coupled provider/base-URL settings if required
  - document the bridge as transitional and attach explicit retirement intent
- Ask first:
  - adding any compatibility artifact beyond `auth.json`, optional `.credentials.json`, and a bounded rendered `config.toml`
  - replaying Codex profile overlays such as `~/.codex/*.config.toml`
  - replaying project config such as repo `.codex/config.toml`
  - changing the gateway auth-handoff architecture
  - widening into generic projection framework code, workspace overlays, or cross-adapter abstractions
  - persisting retained writable Codex state beyond the current direct-launch need
- Never:
  - treat host `~/.codex` as the new steady-state authority model
  - copy whole Codex home trees, logs, sessions, caches, rollout state, daemon state, profile overlays, or repo workspace `.codex`
  - silently project MCP/app-runtime/apps-connectors/hooks/rules/skills/plugin/custom-agent config or state through this bridge
  - replay plugin-bundled MCP servers, plugin-bundled hooks, managed requirements/allowlists, or workspace-shared plugin state through this bridge
  - expose `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` as a public supported contract
  - weaken fail-closed exact-backend policy gating to “world scope implies host-read permission”

## Success Criteria

This slice is complete only when all of the following are true:

1. direct `cli:codex-world` member launch no longer falls back to an unsupported default model solely because isolated `CODEX_HOME` lacks non-secret bootstrap config,
2. the isolated home contains only the bounded compatibility artifacts required by this slice:
   - seeded auth artifacts already allowed by current bridge policy,
   - and a **bounded rendered compatibility `config.toml`** containing at minimum the model setting plus any directly coupled user-level provider/base-URL settings required for truthful startup on the diagnosed profile,
3. the implementation does **not** replay `~/.codex/*.config.toml` profile overlays or repo `.codex/config.toml` project config through the direct member bridge,
4. the implementation does **not** copy or project MCP/app-runtime/apps-connectors/hooks/rules/skills/custom-agent/plugin/workspace-overlay state through the direct member bridge,
5. exact-backend allowlist behavior remains unchanged: no `cli:codex-world` host-read permission means no bridge materialization,
6. when the bridge cannot derive truthful bootstrap config, the path fails closed with a direct explanation rather than silently letting Codex choose an unsupported default,
7. targeted tests are green and the rebuilt installed runtime passes the live June 21, 2026 public bootstrap smoke, including creation of `from_the_world_worker.md`,
8. docs/comments/spec text explicitly mark this as a compatibility bridge that later retires behind the gateway-front-door realization.

## Open Questions

1. Beyond top-level `model`, which directly related non-secret **user-level** provider/base-URL keys must be preserved for truthful startup on the current supported Codex line and the diagnosed profile?
   - Default for this spec: preserve only the smallest non-secret subset required for the direct member path to honor the same startup model/provider choice on the diagnosed profile; do not assume the universal exact minimal subset is already known, and do not widen into profile overlays or project config replay in this slice.
2. Can a hermetic automated test prove the external unsupported-default-model behavior, or should the slice stop at unit-tested materialization plus manual smoke proof?
   - Default for this spec: do both if practical, but do not block the slice on a fully hermetic external-process proof if the unit/materialization tests and live smoke close the bug honestly.
