# TASKS: Placement-Aware Agent Inventory And Selector Contract

Source spec: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Source plan: [PLAN-58-placement-aware-agent-inventory-and-selector-contract.md](./PLAN-58-placement-aware-agent-inventory-and-selector-contract.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: draft for review

## Execution Packets

This slice should be implemented as four sequential packets:

1. schema and projection,
2. exact selector derivation,
3. migration of inventory/docs/tests/policies,
4. final validation wall.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Placement-Aware Schema And Projection

Session goal:

1. add the `version: 2` placement-aware logical-agent inventory schema,
2. project each enabled placement into a realized row,
3. keep migration-compatible parse behavior explicit.

### Tasks

- [ ] Task 1.1: Add typed `config.placements` schema for logical agents
  - Acceptance: inventory can parse `version: 2` logical-agent files with `placements.host` and/or `placements.world`; placement-local `cli.runtime_family` remains typed; unsupported placement names fail through normal schema validation.
  - Verify:
    - `cargo test -p shell agents_validate -- --nocapture`
    - `cargo test -p shell agent_inventory -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_inventory.rs`](../crates/shell/src/execution/agent_inventory.rs)
    - [`crates/shell/tests/agents_validate.rs`](../crates/shell/tests/agents_validate.rs)

- [ ] Task 1.2: Project placement-aware logical agents into realized placement rows
  - Acceptance: projected inventory carries distinct logical-agent id, placement, realized-agent id, backend id, and display label; disabled placements do not realize rows; the projection shape is explicit enough to preserve landed Slice `59` world-runtime truth when `codex_world` migrates under `placements.world`.
  - Verify:
    - `cargo test -p shell agent_inventory -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_inventory.rs`](../crates/shell/src/execution/agent_inventory.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. placement-aware inventory parses,
2. projection emits exact realized rows,
3. no selector cutover has happened yet,
4. the distinction between logical id and realized id is explicit in the projection layer.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Placement-Qualified Exact Selector Derivation

Session goal:

1. derive exact backend ids from placement-qualified realized identities,
2. keep exact-selection semantics fail-closed,
3. derive human-facing display labels without making them selectors.

### Tasks

- [ ] Task 2.1: Derive placement-qualified realized agent ids and exact backend ids
  - Acceptance: placement-aware `codex` inventory realizes `codex-host` and `codex-world`; exact backend ids derive as `cli:codex-host` and `cli:codex-world`; backend-id grammar remains unchanged.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_inventory.rs`](../crates/shell/src/execution/agent_inventory.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](../crates/shell/src/execution/agent_runtime/validator.rs)

- [ ] Task 2.2: Keep multi-placement selection fail-closed and labels read-only
  - Acceptance: exact backend-targeted paths still require exact backend ids; multi-placement omission does not silently choose host or world; human-facing labels are derived for read-only/status surfaces only; exact-id migration does not blur the landed Slice `59` distinction between host-scoped and world-scoped runtime truth.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](../crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](../crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. placement-qualified exact ids are live for placement-aware entries,
2. labels are clearly separated from selectors,
3. host/world cannot silently cross-match,
4. backend-id grammar and policy model remain unchanged.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Inventory, Policy, Fixture, And Doc Migration

Session goal:

1. replace split `*_world` logical-agent files with placement-aware logical-agent files,
2. migrate exact ids in examples/tests/docs/runtime-control fixtures/smoke helpers,
3. leave no contradictory product truth behind.

### Tasks

- [ ] Task 3.1: Convert split inventory to placement-aware logical-agent files
  - Acceptance: multi-placement agents are represented as one logical-agent file; duplicated `codex`/`codex_world` and `claude_code`/`claude_code_world` forward product truth is removed from authoritative config examples unless the slice is explicitly re-scoped before implementation.
  - Verify:
    - manual diff review
    - `cargo test -p shell agent_inventory -- --nocapture`
  - Expected files touched:
    - [`config/agents/claude_code.yaml`](../config/agents/claude_code.yaml)
    - [`config/agents/claude_code_world.yaml`](../config/agents/claude_code_world.yaml)
    - [`config/agents/codex.yaml`](../config/agents/codex.yaml)
    - [`config/agents/codex_world.yaml`](../config/agents/codex_world.yaml)
    - adjacent agent inventory examples under [`config/agents/`](../config/agents)

- [ ] Task 3.2: Migrate exact backend ids in docs, examples, policy samples, and fixtures
  - Acceptance: authoritative docs/examples/fixtures stop treating legacy `cli:*_world` ids as forward truth when the placement-aware schema is enabled; migrated examples use placement-qualified exact ids and keep the landed Slice `59` world-runtime/operator semantics intact.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - scoped grep/manual review of authoritative forward surfaces that currently pin `cli:*_world` ids
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/shell/src/repl/async_repl.rs`](../crates/shell/src/repl/async_repl.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](../crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`scripts/substrate/dev-fresh-install-gateway-smoke.sh`](../scripts/substrate/dev-fresh-install-gateway-smoke.sh)
    - [`scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh`](../scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh)
    - relevant policy/example fixtures

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. the repo has one forward logical-agent file per multi-placement agent,
2. docs and fixtures use the current exact ids consistently,
3. old and new exact ids are not both presented as equivalent forward truth,
4. any temporary compatibility posture is explicit rather than implied.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Final Validation Wall

Session goal:

1. prove the placement-aware selector contract is internally coherent,
2. prove the placement-aware cutover preserves the landed Slice `59` runtime-realizability / remediation / installer contract.

### Tasks

- [ ] Task 4.1: Run the final validation wall
  - Acceptance: format, lint, inventory, selector, validator, and control-surface suites are green; the repo clearly distinguishes logical-agent identity, realized placement identity, and exact backend id.
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
  - Expected files touched:
    - no planned source edits; this is the validation gate after the implementation packets above.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. placement-aware inventory is the forward contract,
2. exact selectors remain fail-closed and placement-qualified,
3. labels remain human-facing only,
4. landed Slice `59` runtime truth, remediation, and installer semantics remain intact after the exact-id cutover.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After each packet:

1. confirm its verification commands are green,
2. confirm the checkpoint is satisfied,
3. confirm no packet forced a backend-id grammar change,
4. confirm no packet silently reopens or weakens the landed Slice `59` runtime/bootstrap contract.

Reopen spec/plan/tasks only if:

1. implementation cannot separate logical id from realized id cleanly,
2. compatibility posture creates a contradiction between old and new exact ids,
3. policy semantics would need to move away from exact backend ids,
4. or the landed Slice `59` runtime/bootstrap contract would have to change to make the placement-aware migration coherent.

## Packet Session Final Message Requirements

Every packet implementation session should end by stating:

1. which verification commands passed or failed,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether spec/plan/tasks need reopening,
5. the GitNexus impact-analysis results for each production symbol edited in that packet,
6. and whether the landed Slice `59` runtime/bootstrap contract remains intact after that packet.
