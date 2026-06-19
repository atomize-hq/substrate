# TASKS: Post-Placement-Aware Compatibility Retirement

Source spec: [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md)  
Source plan: [PLAN-60-post-placement-aware-compatibility-retirement.md](./PLAN-60-post-placement-aware-compatibility-retirement.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions plus final validation  
Status: draft for review

## Phase Gate

These tasks assume:

1. Slice `59` remains the green runtime-truth floor,
2. Slice `58` cutover is reflected in forward repo truth,
3. Slice `60` is bounded to retirement of placement-aware compatibility posture rather than new runtime or storage redesign.

## Execution Packets

This slice should be implemented as five sequential packets:

1. freeze the retirement boundary,
2. retire the effective-inventory compatibility bridge,
3. retire split-entry exact selector compatibility,
4. migrate forward docs/examples/fixtures/scripts,
5. run the final validation wall.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Freeze The Retirement Boundary

Session goal:

1. define which surfaces are forward truth,
2. define which legacy-name hits are allowed as history or negative tests,
3. create the grep wall that later packets will use as a contract.

### Tasks

- [ ] Task 1.1: Freeze the forward-surface grep wall and historical allowlist
  - Acceptance: the slice names the directories/files that must become legacy-name-free current truth and separately names the historical or negative-test allowlist; implementation no longer has to guess whether a hit is in scope.
  - Boundary contract:
    - `docs/`, `config/`, and `scripts/` are forward-truth surfaces and must not keep legacy split-entry ids or unqualified pre-placement exact selectors such as `cli:codex` / `cli:claude_code` as acceptable history.
    - `llm-last-mile/` planning records may retain old ids only as explicitly historical provenance.
    - temporary Packet `1` allowlist seams are `crates/shell/src/execution/agent_inventory.rs`, `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`, `crates/shell/src/execution/agent_runtime/validator.rs`, `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/builtins/world_gateway.rs`, `crates/shell/src/execution/agents_cmd.rs`, `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/host_inbox_materialization.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/agent_runtime/auto_attach.rs`, `crates/shell/src/execution/agent_runtime/host_inbox.rs`, `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/session.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and `crates/shell/src/repl/async_repl.rs`.
    - active forward docs/scripts may still hit the widened Packet `1` wall only in `docs/TRACE.md`, `docs/internals/world/gateway_auth_handoff.md`, `docs/reference/world/verification/gateway_auth_handoff.md`, `scripts/linux/world-provision.sh`, and `scripts/mac/smoke.sh` until Packet `4` retires those current-truth examples.
    - `crates/shell/tests/**` and inline `#[cfg(test)]` coverage may retain legacy ids only for explicit bridge-removal coverage, persisted-state continuity, or fail-closed retirement assertions; Packet `4` must narrow the remaining hits to those intentional cases.
    - if a live old-id dependency appears outside that bounded allowlist, if any old-id hit must remain in `docs/`, `config/`, or `scripts/`, or if removal would reopen Slice `59` runtime semantics, reopen Slice `60` planning before starting Packet `2`.
  - Verify:
    - manual review of [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md), [PLAN-60-post-placement-aware-compatibility-retirement.md](./PLAN-60-post-placement-aware-compatibility-retirement.md), and [TASKS-60.md](./TASKS-60.md)
    - `rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" docs config crates/shell scripts -g '!target'`
    - `rg -nP "\bcli:(codex|claude_code)\b(?!-)" docs config crates/shell scripts -g '!target'`
  - Files:
    - [`llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md`](../llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md)
    - [`llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md`](../llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md)
    - [`llm-last-mile/TASKS-60.md`](../llm-last-mile/TASKS-60.md)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. forward truth versus historical evidence is explicit, including the difference between current forward-surface pre-placement-selector debt and the bounded historical/compatibility allowlist,
2. the grep wall is defined,
3. reopen conditions are stated if hidden live dependencies appear.

Do not start Packet 2 until Packet 1 is reviewed and green.

## Packet 2: Retire The Effective-Inventory Compatibility Bridge

Session goal:

1. remove the Slice `58` compatibility bridge from forward live behavior,
2. keep effective inventory authoritative on placement-aware realized identities,
3. preserve fail-closed behavior if a hidden dependency survives.

### Tasks

- [ ] Task 2.1: Remove v2-to-split-entry effective-inventory materialization from forward live behavior
  - Acceptance: placement-aware manifests no longer materialize split-entry-shaped effective rows as normal live behavior; effective inventory remains usable through realized placement identities only.
  - Verify:
    - `cargo test -p shell agent_inventory -- --nocapture`
    - `cargo test -p shell agents_validate -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_inventory.rs`](../crates/shell/src/execution/agent_inventory.rs)
    - [`crates/shell/tests/agents_validate.rs`](../crates/shell/tests/agents_validate.rs)

- [ ] Task 2.2: Prove public control surfaces no longer rely on the bridge
  - Acceptance: status/doctor/start-adjacent consumers continue to work with placement-aware realized identities and do not expect compatibility-generated split-entry rows.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](../crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. placement-aware effective inventory no longer depends on split-entry materialization,
2. live control surfaces still function with realized placement identities,
3. no new compatibility branch was introduced.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Retire Split-Entry Exact Selector Compatibility

Session goal:

1. stop accepting split-entry exact backend ids as live forward selectors,
2. keep placement-qualified exact ids green,
3. fail closed with explicit migration guidance.

### Tasks

- [ ] Task 3.1: Reject retired split-entry exact backend ids on runtime selection paths
  - Acceptance: exact backend selectors like `cli:codex_world` and `cli:claude_code_world` fail closed with explicit guidance; placement-qualified exact ids continue to work.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs)

- [ ] Task 3.2: Retire forward policy/example/REPL dependence on pre-placement exact ids
  - Acceptance: active policy fixtures and REPL/runtime examples use only placement-qualified exact ids unless they are explicit negative tests; forward-happy-path coverage no longer treats split-entry ids or unqualified `cli:codex` / `cli:claude_code` selectors as current valid examples.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
    - relevant forward-facing test fixtures

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. split-entry exact ids are no longer forward-valid selectors,
2. placement-qualified exact ids are the only green path,
3. runtime truth from Slice `59` remains untouched.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Migrate Forward Docs, Scripts, And Fixtures

Session goal:

1. remove contradictory current-truth examples,
2. keep remaining legacy-name hits only as history or explicit retirement tests,
3. align active scripts and docs with the final identity model.

### Tasks

- [ ] Task 4.1: Normalize authoritative docs and active scripts to placement-qualified ids
  - Acceptance: forward operator docs and active smoke helpers no longer present split-entry ids or unqualified `cli:codex` / `cli:claude_code` selectors as current valid choices.
  - Verify:
    - manual review of changed docs/scripts
    - `rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" docs scripts -g '!target'`
    - `rg -nP "\bcli:(codex|claude_code)\b(?!-)" docs scripts -g '!target'`
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`scripts/substrate/dev-fresh-install-gateway-smoke.sh`](../scripts/substrate/dev-fresh-install-gateway-smoke.sh)
    - [`scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh`](../scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh)
    - any other forward docs/scripts discovered by the grep wall

- [ ] Task 4.2: Normalize forward-facing test fixtures while preserving explicit negative retirement tests
  - Acceptance: remaining old-id references in tests are either historical-fixture provenance or explicit retirement/fail-closed assertions; forward-happy-path fixtures use placement-qualified ids only.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - relevant files under [`crates/shell/tests/`](../crates/shell/tests/)

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. forward docs/scripts/examples are placement-aware only,
2. remaining legacy-name hits are explicit history or explicit negative tests,
3. old and new ids, including unqualified pre-placement selectors, are no longer presented as coequal current truth.

Do not start Packet 5 until Packet 4 verification is green.

## Packet 5: Final Validation Wall

Session goal:

1. prove the retirement slice is coherent end to end,
2. prove Slice `59` runtime truth survived unchanged,
3. prove the repo now has one forward identity model.

### Tasks

- [ ] Task 5.1: Run the final validation wall and forward-truth grep proof
  - Acceptance: format, lint, targeted shell tests, and the forward-truth grep wall are green; any remaining split-entry-name or unqualified pre-placement selector hits are intentional history or negative tests only.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell agents_validate -- --nocapture`
    - `cargo test -p shell agent_inventory -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" docs config crates/shell scripts -g '!target'`
    - `rg -nP "\bcli:(codex|claude_code)\b(?!-)" docs config crates/shell scripts -g '!target'`
  - Files:
    - no planned source edits; validation only

### Packet 5 Checkpoint

Packet 5 is complete only when:

1. the Slice `58` compatibility bridge is retired from forward live behavior,
2. placement-qualified exact ids are the only forward selectors,
3. authoritative forward truth is placement-aware only,
4. Slice `59` runtime semantics remain green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.
4. Packet 4 blocks Packet 5.

## Inter-Packet Review Rules

After each packet:

1. confirm its verification commands are green,
2. confirm the packet checkpoint is satisfied,
3. confirm no packet reopened Slice `59` runtime semantics,
4. confirm remaining legacy-name hits are either intentional history or explicit negative tests.

Implementation-gate rules for packet sessions:

1. run GitNexus impact analysis before editing any production symbol,
2. warn before proceeding if impact is `HIGH` or `CRITICAL`,
3. run `gitnexus_detect_changes()` before committing.

Reopen spec/plan/tasks only if:

1. a still-supported live control surface depends on the retired compatibility bridge,
2. a supported persisted runtime/session path truly requires split-entry alias reads after the forward retirement,
3. selector retirement would force a Slice `59` runtime-semantic change,
4. or the grep wall reveals that forward and historical surfaces cannot be separated cleanly with the current scope.

## Packet Session Final Message Requirements

Every implementation session should end by stating:

1. which verification commands passed or failed,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether spec/plan/tasks need reopening,
5. the GitNexus impact-analysis results for each production symbol edited in that packet.
