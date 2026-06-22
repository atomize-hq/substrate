# TASKS-62: World Codex Direct-Member Bootstrap Compatibility Bridge Completion

Source spec: [SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md](./SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md)  
Source plan: [PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md](./PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md)  
Phase: `TASKS`  
Execution model: three sequential implementation packets  
Status: draft for review

## Phase Gate

These tasks assume:

1. Slice `59` runtime-realizability truth remains the landed floor,
2. the June 21, 2026 handoffs provide the best current diagnosis for the remaining direct-member failure on the diagnosed file-backed-auth machine profile: missing narrow non-secret user-level bootstrap config rather than missing guest runtime or missing world binding,
3. the three DESIGN docs remain architectural inputs only; this slice is still a bounded compatibility repair, not the implementation of the generic projection framework.

Do not begin implementation until the spec and plan are accepted.

## Execution Packets

This slice should be implemented as three sequential packets:

1. pin the direct compatibility-bridge boundary in tests,
2. land bounded bootstrap config materialization plus fail-closed diagnostics,
3. update docs and run the validation/live-smoke wall.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Pin The Direct Compatibility Bridge Boundary

Session goal:

1. freeze the current bug boundary in automation,
2. prove the current seam is still auth-seeding-only and does not yet replay config on the diagnosed file-backed direct-member path,
3. keep exact-backend policy gating pinned before implementation widens the seam.

### Tasks

- [ ] Task 1.1: Expand world-service bootstrap coverage for bounded config materialization
  - Acceptance: `member_runtime` coverage explicitly exercises isolated `CODEX_HOME` preparation for the current file-backed direct-member bootstrap path and proves the current contract honestly: auth seeding occurs, the internal seed-home env is removed before child spawn, optional `.credentials.json` still behaves as today, and user-level/profile/project config artifacts are not replayed or materialized yet.
  - Verify:
    - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
    - related world-service tests adjacent to the bootstrap helpers

- [ ] Task 1.2: Keep exact-backend seed-home gating pinned in shell tests
  - Acceptance: shell-side coverage still proves that only the exact allowlisted backend receives the runtime-internal host seed-home hint and that broader world scope does not imply host-read permission automatically.
  - Verify:
    - `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/routing/dispatch/world_ops.rs`](../crates/shell/src/execution/routing/dispatch/world_ops.rs)
    - nearby shell tests for the member-dispatch builder

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. world-service tests pin that the current bridge is auth-seeding-only and does not yet replay/materialize bounded config behavior,
2. shell tests still pin exact-backend allowlist truth,
3. the slice boundary is tight enough that Packet 2 cannot hide broader config projection inside the same seam.

Do not start Packet 2 until Packet 1 is reviewed and green.

## Packet 2: Land Bounded Bootstrap Config Materialization And Fail-Closed Diagnostics

Session goal:

1. preserve truthful direct Codex startup in isolated `CODEX_HOME`,
2. keep the bridge compatibility-only and narrow,
3. fail closed when the bounded startup subset cannot be derived.

### Tasks

- [ ] Task 2.1: Derive and render the narrow non-secret Codex startup subset
  - Acceptance: the direct member path reads only from the already policy-gated host seed-home source, derives the smallest non-secret **user-level** startup subset required for truthful direct launch on the diagnosed profile, and renders that subset into isolated `CODEX_HOME/config.toml` instead of copying broader Codex home/config state; at minimum this preserves model plus directly coupled provider/base-URL settings if required by the diagnosed profile, and it does not replay `~/.codex/*.config.toml` profile overlays or repo `.codex/config.toml`.
  - Verify:
    - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
    - `rg -n "config\\.toml|model|provider|base_url|profile" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
    - [`crates/world-service/Cargo.toml`](../crates/world-service/Cargo.toml) only if a minimal parsing/rendering dependency is required

- [ ] Task 2.2: Keep the bridge internal and bounded
  - Acceptance: the implementation still strips `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME` before child spawn, still materializes only the bounded direct-launch compatibility artifacts, does not replay profile/project config layers, and does not project MCP/app-runtime/apps-connectors/hooks/rules/skills/custom-agent/plugin/workspace-overlay state through the bridge, including plugin-bundled MCP servers, plugin-bundled hooks, managed requirements/allowlists, or workspace-shared plugin state.
  - Verify:
    - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
    - `rg -n "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME|mcp|skills|plugin|workspace \\.codex|app-runtime|hooks|rules|agents|requirements" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)

- [ ] Task 2.3: Add explanation-ready fail-closed diagnostics for missing bounded startup truth
  - Acceptance: when the bridge cannot derive the required bounded startup subset for the diagnosed profile, the direct member path fails with a direct explanation rather than silently allowing Codex to fall back to an unsupported default model under isolated `CODEX_HOME`.
  - Verify:
    - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
    - targeted world-service test name(s) added for the failure branch, if separate from the existing helper test
  - Files:
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs)
    - related world-service tests for error behavior

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. isolated direct-member homes receive bounded startup config instead of depending on Codex defaults,
2. profile/project config replay and broader config/state domains are still absent from the bridge,
3. missing bounded startup truth fails closed with a specific explanation,
4. the implementation still reads as a transitional direct-member seam rather than a generic projection framework.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Docs, Validation Wall, And Live Smoke Proof

Session goal:

1. make repo truth explicit about the transitional bridge,
2. prove the rebuilt installed runtime fixes the real smoke,
3. stop the slice before future capability widening begins.

### Tasks

- [ ] Task 3.1: Update direct-member operator/developer docs with bounded bridge posture
  - Acceptance: docs explain that direct `cli:codex-world` member bootstrap currently depends on a bounded compatibility bridge for the current file-backed direct-member path that includes narrow non-secret startup config in addition to auth seeding, and they do not imply this is the final steady-state architecture or a generic future-capability foundation.
  - Verify:
    - `rg -n "cli:codex-world|CODEX_HOME|compatibility bridge|gateway" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md`
  - Files:
    - [`docs/USAGE.md`](../docs/USAGE.md)

- [ ] Task 3.2: Run the targeted validation wall
  - Acceptance: the focused bootstrap, shell gating, and direct-dispatch regression tests are green after the implementation.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
    - `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`
    - `cargo test -p shell c3_internal_toolbox_run_world_task_fast_completion_still_streams_registered_task_run_id_before_terminal_result -- --nocapture`
  - Files:
    - no planned source edits; validation only

- [ ] Task 3.3: Rebuild/redeploy and rerun the live smoke floor plus the June 21 public bootstrap smoke
  - Acceptance: the rebuilt installed runtime no longer reproduces the direct-member exit-1 failure on the diagnosed machine profile; the trivial world bootstrap probe succeeds, and the exact June 21 public bootstrap smoke creates `from_the_world_worker.md`.
  - Verify:
    - `~/.substrate/bin/substrate agent start --backend cli:codex-world --scope world --prompt 'Reply with WORLD READY only.' --json`
    - exact June 21, 2026 public bootstrap smoke command from the user transcript
  - Files:
    - no planned source edits; rebuild/redeploy and manual verification only

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. repo truth describes the direct member seam as a bounded compatibility bridge,
2. targeted validation is green,
3. the rebuilt installed runtime fixes the real smoke,
4. nothing in docs or implementation encourages future capability work to piggyback on this bridge.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.

## Inter-Packet Review Rules

After each packet:

1. confirm its checkpoint is satisfied,
2. confirm the slice has not widened into broader Codex projection work,
3. confirm the gateway-front-door target architecture is still preserved,
4. confirm future MCP/app-runtime/apps-connectors/hooks/rules/skills/plugin/custom-agent/workspace-overlay work remains out of scope.
