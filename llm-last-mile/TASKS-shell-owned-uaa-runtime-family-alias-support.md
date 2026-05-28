# TASKS: Shell-Owned UAA Runtime Family Alias Support

Source spec: [SPEC-shell-owned-uaa-runtime-family-alias-support.md](./SPEC-shell-owned-uaa-runtime-family-alias-support.md)  
Source plan: [PLAN-shell-owned-uaa-runtime-family-alias-support.md](./PLAN-shell-owned-uaa-runtime-family-alias-support.md)  
Source handoff: [2026-05-28-151655-runtime-family-alias-support-architecture.md](../.codex/handoffs/2026-05-28-151655-runtime-family-alias-support-architecture.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: draft for review

## Execution Packets

This fix should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 implements typed inventory schema and projection only.
- Packet 2 implements runtime-family resolution and fail-closed runtime-realizability checks.
- Packet 3 aligns persisted-session and REPL readers with the corrected runtime-family contract.
- Packet 4 converts fixtures/helpers, locks regression coverage, updates docs, and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Typed Inventory Runtime-Family Contract

Session goal:

1. add explicit inventory truth at `config.cli.runtime_family`,
2. keep the field owned by typed agent inventory rather than policy or top-level config,
3. project that truth forward without changing runtime behavior yet.

### Tasks

- [ ] Task 1.1: Add typed `config.cli.runtime_family` schema and projected inventory carriage
  - Acceptance: `AgentCliConfigV1` accepts `runtime_family` as a typed `snake_case` enum with supported values `codex` and `claude_code`; `ProjectedInventoryEntryV1` carries the projected CLI runtime-family truth; `deny_unknown_fields` behavior remains intact; no policy or top-level config types are widened.
  - Verify:
    - `cargo test -p shell agents_validate -- --nocapture`
    - `cargo test -p shell agent_inventory -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_inventory.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)
    - [`crates/shell/tests/agents_validate.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. inventory can parse valid `runtime_family` values,
2. projected inventory can carry runtime-family truth for selected rows,
3. invalid values still fail through the normal typed-schema path,
4. no policy or top-level config surfaces have changed.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Explicit Runtime-Family Resolution And Fail-Closed Validation

Session goal:

1. remove literal-`agent_id` runtime-family inference from production selection-to-launch code,
2. resolve runtime family from projected inventory truth,
3. require `config.cli.runtime_family` only for the shell-owned UAA runtime-realizability path.

### Tasks

- [ ] Task 2.1: Replace literal-`agent_id` runtime-family inference in runtime contract construction
  - Acceptance: launch-contract construction no longer derives runtime family from literal inventory `agent_id`; `mapping.rs` exposes an explicitly named resolver that consumes projected runtime-family truth; exact backend selection remains keyed on `backend_id`; aliased exact backends such as `cli:codex_world` resolve to the canonical `Codex` runtime family when inventory says `runtime_family: codex`.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/mapping.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/agent_inventory.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs)

- [ ] Task 2.2: Enforce fail-closed runtime-realizability checks for missing or unsupported runtime-family truth
  - Acceptance: runtime-realizability validation requires `config.cli.runtime_family` only when `config.kind=cli`, `protocol=substrate.agent.session`, and effective `cli.mode=persistent`; missing `runtime_family` fails closed with explicit contract wording; unrelated CLI entries outside that path do not become newly invalid; policy allowlisting remains keyed only on exact `backend_id`.
  - Verify:
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)
    - [`crates/shell/src/execution/agent_runtime/mapping.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mapping.rs)
    - [`crates/shell/tests/agents_validate.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. production runtime resolution no longer depends on literal `agent_id`,
2. `cli:codex_world` and similar aliases remain distinct exact backends while resolving the correct canonical runtime family,
3. missing `runtime_family` fails closed only for the intended shell-owned UAA path,
4. policy and backend-selection semantics remain unchanged.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Persistence And REPL Parity

Session goal:

1. preserve canonical persisted runtime-family truth after the production resolver changes,
2. keep retained-session and retained-member readers aligned with canonical `resolved_agent_kind`,
3. prevent alias identity from leaking into persisted runtime-family fields.

### Tasks

- [ ] Task 3.1: Preserve canonical persisted session truth for aliased exact backends
  - Acceptance: persisted manifests for aliased backends keep `agent_id` and `backend_id` exact, but continue to serialize canonical `resolved_agent_kind` values as `codex` or `claude_code`; manifest readers continue interpreting `resolved_agent_kind` as runtime family rather than alias identity; no wire or persistence field rename is introduced.
  - Verify:
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

- [ ] Task 3.2: Keep REPL retained-member reconstruction and targeted follow-up routing exact-backend aware
  - Acceptance: retained-member reconstruction continues to derive the runtime implementation from canonical persisted runtime-family truth while preserving exact backend identity such as `cli:codex_world`; targeted follow-up flows do not collapse aliased world backends back to `cli:codex`; REPL parity remains host/world truthful.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/repl/async_repl.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. canonical persisted runtime-family spelling remains unchanged,
2. exact backend identity still survives through retained-session and retained-member paths,
3. alias identity is not serialized into `resolved_agent_kind`,
4. REPL follow-up routing preserves `cli:codex_world` versus `cli:codex`.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Fixture Conversion, Regression Locking, Docs, And Validation

Session goal:

1. treat fixture/helper churn as a first-class implementation stream,
2. convert runtime-realizable inventories and persisted-manifest writers to the explicit runtime-family contract,
3. update docs to reflect the shipped contract,
4. run the final validation wall.

### Tasks

- [ ] Task 4.1: Convert runtime-realizable fixture and helper writers to explicit runtime-family truth
  - Acceptance: any shared helper or inline fixture that emits `kind=cli`, `protocol=substrate.agent.session`, and effective `cli.mode=persistent` now declares `config.cli.runtime_family`; persisted-manifest fixture writers stop mirroring alias `agent_id` into `resolved_agent_kind`; missing-field fixtures are kept only where the test is explicitly asserting fail-closed rejection.
  - Verify:
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/tests/common.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/common.rs)
    - [`crates/shell/tests/support/mod.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/support/mod.rs)

- [ ] Task 4.2: Lock alias-topology regressions across doctor, start, turn, retained-member, and fail-closed missing-field behavior
  - Acceptance: regression coverage explicitly exercises host orchestrator `codex` plus world alias `codex_world`; `substrate agent doctor --json` succeeds for the explicit alias topology; `agent start --backend cli:codex_world --scope world` and targeted follow-up/retained-member paths preserve exact backend identity; dedicated failures cover missing `runtime_family` on shell-owned UAA candidates.
  - Verify:
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/src/execution/agent_runtime/validator.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs)

- [ ] Task 4.3: Update public config docs to describe explicit inventory runtime-family support without widening policy or top-level config
  - Acceptance: docs stop framing runtime-realizable support as a hard-coded list of literal realized backends; `docs/CONFIGURATION.md` explains that shell-owned UAA runtime realization depends on exact backend selection plus explicit `config.cli.runtime_family`; supported values are documented as `codex` and `claude_code`; policy wording remains exact-`backend_id` only.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md)

- [ ] Task 4.4: Run the final validation wall and focused operator smoke
  - Acceptance: formatting, clippy, validator coverage, the three targeted shell integration suites, and the focused alias-topology smoke commands all pass; final results show no production reliance on literal `agent_id` for runtime-family resolution and no regression in exact backend selection or policy semantics.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell agent_runtime::validator -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `substrate agent doctor --json`
    - `substrate agent start --backend cli:codex --scope host --prompt "host ok" --json`
    - `substrate agent start --backend cli:codex_world --scope world --prompt "world ok" --json`
    - `substrate agent turn --session <session_id> --backend cli:codex_world --prompt "next" --json`
  - Expected files touched:
    - No planned source edits; this is the final validation gate after the implementation tasks above.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. helper and fixture conversion is complete and explicit,
2. alias-topology regressions are pinned across doctor, start, turn, and retained-member flows,
3. docs describe `config.cli.runtime_family` as inventory-only runtime-realization truth,
4. the full validation wall and focused smoke commands pass.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh spec-driven-development restart.

Proceed directly to the next packet only when:

1. the current packet's verification steps are green,
2. the current packet checkpoint is satisfied,
3. no approved architectural constraint was violated,
4. the next packet is still consistent with the source spec, plan, and this TASKS document.

Reopen spec, plan, or TASKS only if one of these is true:

1. implementation discovers a real contradiction in the approved architecture,
2. a packet forces a scope change,
3. verification shows the planned dependency order is wrong,
4. a new requirement appears.

If none of those conditions are met, continue packet-to-packet without re-specifying or re-planning.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether any condition to reopen spec, plan, or TASKS was discovered,
5. the GitNexus impact-analysis results for each production symbol edited in that packet, including any `HIGH` or `CRITICAL` warnings that had to be reviewed before editing,
6. any remaining risks, deferred follow-ups, or assumptions that the next packet needs to know.

If a packet is not fully green, the final message must say explicitly that the next packet should not begin yet.

## Notes For Implementation

- Packet 1 is intentionally schema-first. Do not widen it into runtime behavior or doc rewrites beyond what is strictly needed to land typed inventory truth.
- Packet 2 is the architectural correction packet. Run GitNexus impact analysis on each concrete symbol before editing production runtime functions, and stop to review blast radius if risk comes back `HIGH` or `CRITICAL`.
- Packet 3 should preserve canonical persisted runtime-family spelling. If implementation pressure suggests renaming `resolved_agent_kind` or changing wire enums, stop and ask first.
- Packet 4 treats fixture/helper conversion as first-class work, not cleanup. Update shared writers before chasing individual failing tests so the greenfield contract is applied consistently.
- For Packet 4 and later follow-ons, prefer the globally installed `gitnexus` CLI rather than `npx gitnexus ...` in this repo. If the index may be stale, refresh it with `gitnexus analyze --repo /home/azureuser/__Active_Code/atomize-hq/substrate`, then run repo-qualified impact and changed-scope checks such as `gitnexus impact --repo /home/azureuser/__Active_Code/atomize-hq/substrate --target <symbol> --direction upstream` and `gitnexus detect-changes --repo /home/azureuser/__Active_Code/atomize-hq/substrate --scope unstaged`.
