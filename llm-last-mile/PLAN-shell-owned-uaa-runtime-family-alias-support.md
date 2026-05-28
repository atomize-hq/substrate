# Plan: Shell-Owned UAA Runtime Family Alias Support

Source spec: [SPEC-shell-owned-uaa-runtime-family-alias-support.md](./SPEC-shell-owned-uaa-runtime-family-alias-support.md)  
Plan type: architecture-corrective follow-on for shell-owned UAA runtime realization  
Status: draft for review  
Implementation posture: greenfield contract, no compatibility fallback

## Objective

Implement the approved architecture from the spec:

1. exact backend selection remains keyed on derived `backend_id`,
2. shell-owned UAA runtime family becomes an explicit inventory property at `config.cli.runtime_family`,
3. runtime family is required only for the shell-owned UAA runtime-realizability path,
4. policy and top-level config remain unchanged,
5. canonical persisted/wire runtime-family state remains `codex` / `claude_code`,
6. no literal-`agent_id` fallback remains anywhere in production runtime realization.

## Plan Summary

The current bug is produced by one architectural mistake repeated in a few places:

1. exact backend identity is selected correctly,
2. runtime family is then inferred from literal inventory `agent_id`,
3. persisted readers also assume runtime family can be reconstructed from hard-coded strings,
4. tests and docs encode the same shortcut.

The correct fix is to separate these concerns cleanly:

1. inventory owns exact backend identity plus explicit runtime-family declaration,
2. dispatch/validator resolve runtime family from projected inventory,
3. runtime launch and world transport continue to operate on canonical runtime-family enums,
4. persisted state continues to serialize canonical runtime-family names only,
5. tests and docs explicitly declare runtime family where runtime realization is expected.

Because the contract is greenfield, the plan should not preserve any implicit legacy shape. Instead, it should make missing `config.cli.runtime_family` a fail-closed error for shell-owned UAA runtime candidates and update all runtime-realizable fixtures/examples accordingly.

## Locked Decisions

### What changes

1. Add `config.cli.runtime_family` to agent inventory schema under `AgentCliConfigV1`.
2. Add a typed inventory enum for supported runtime families.
3. Thread projected runtime-family truth through inventory projection into runtime contract construction.
4. Replace `orchestrator_backend_kind(agent_id)` with an explicitly named resolver that consumes runtime-family truth rather than literal `agent_id`.
5. Update runtime-realizable tests and docs so shell-owned UAA entries declare runtime family explicitly.

### What does not change

1. No policy key changes.
2. No top-level config changes.
3. No backend-id grammar changes.
4. No host-rooted authority or world-binding contract changes.
5. No canonical persisted/wire runtime-family spelling changes.
6. No heuristic inference from binary, alias naming, or tuple axes.

## Implementation Order

### Phase 1: Add Typed Inventory Runtime-Family Schema

Goal:

1. introduce a typed inventory field at `config.cli.runtime_family`,
2. keep schema ownership in agent inventory rather than policy or top-level config,
3. make the field available to projected inventory entries without yet changing runtime behavior.

Primary touch surface:

1. `crates/shell/src/execution/agent_inventory.rs`
2. `docs/CONFIGURATION.md`
3. agent inventory test fixtures that assert parse/validation behavior

Recommended shape:

1. add `runtime_family: Option<AgentCliRuntimeFamilyV1>` to `AgentCliConfigV1`,
2. define `AgentCliRuntimeFamilyV1` as a `snake_case` enum with:
   - `Codex`
   - `ClaudeCode`
3. add projected inventory carriage for `cli_runtime_family`,
4. keep `deny_unknown_fields` behavior intact so the new field is typed and validated automatically.

Why first:

1. it sets the contract truth in the right typed layer,
2. it avoids inventing ad hoc string parsing later in runtime code,
3. it lets later phases fail closed with better errors because the schema exists.

Verification checkpoint:

1. inventory parsing accepts valid `runtime_family` values,
2. invalid values fail with the normal schema/serde error posture,
3. policy validation remains untouched,
4. projected inventory can carry `cli_runtime_family` for selected rows.

Risks:

1. putting the enum in the wrong module can create awkward layering or duplicate types,
2. adding the field too high in the stack could pollute top-level config ergonomics.

Mitigation:

1. keep the schema enum in or adjacent to `agent_inventory.rs`,
2. convert from inventory enum into runtime enum at the runtime-resolution seam rather than the other way around.

### Phase 2: Replace Agent-Id Runtime Inference With Explicit Runtime-Family Resolution

Goal:

1. remove literal-`agent_id` inference from production launch-contract construction,
2. require explicit runtime-family truth for shell-owned UAA runtime-realizable candidates,
3. preserve exact backend-id routing and fail-closed behavior.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/mapping.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/agent_runtime/validator.rs`

Required changes:

1. replace `orchestrator_backend_kind(agent_id)` with a better-named helper such as:
   - `resolve_shell_owned_runtime_backend_kind(...)`
   - or equivalent
2. feed that helper explicit projected/inventory runtime-family truth,
3. keep orchestrator-only validation semantics separate from general runtime-family resolution,
4. add a fail-closed error when a shell-owned UAA runtime candidate lacks `config.cli.runtime_family`.

Why second:

1. this is the real bug fix,
2. it restores architectural honesty at the selection-to-realization seam,
3. later persistence/test updates should build on the corrected production contract.

Verification checkpoint:

1. `validate_runtime_realizability(...)` succeeds for explicit host/runtime entries with declared runtime family,
2. `validate_member_selection(...)` succeeds for `cli:codex_world` with `runtime_family: codex`,
3. exact backend selection still rejects wrong scope/protocol/capability rows fail-closed,
4. missing `runtime_family` on a shell-owned UAA candidate fails with explicit contract wording.

Risks:

1. conflating orchestrator validation with general runtime-family resolution again,
2. accidentally requiring `runtime_family` for unrelated CLI rows.

Mitigation:

1. scope the new requirement inside the shell-owned UAA runtime-realizability checks only,
2. keep the existing `kind/protocol/cli.mode` gates as the entry criteria for that requirement.

### Phase 3: Align Persistence, REPL Parity, and Runtime Readers

Goal:

1. ensure the corrected runtime-family contract is consistent with persisted session truth,
2. preserve canonical `resolved_agent_kind` serialization,
3. eliminate any remaining implicit coupling between alias identity and runtime family in runtime readers.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
2. `crates/shell/src/repl/async_repl.rs`
3. `crates/shell/src/execution/agent_runtime/control.rs`
4. possibly `crates/shell/src/execution/agent_runtime/session.rs`

Required changes:

1. keep canonical writes of runtime family as `codex` / `claude_code`,
2. ensure readers continue to interpret those values as runtime family, not agent alias,
3. update any reconstructors that still conceptually blur runtime family and inventory identity,
4. avoid changing wire enums or persisted spelling unless a concrete blocker forces it.

Why third:

1. production selection must be correct before persistence parity can be audited meaningfully,
2. this phase is about consistency and retained-session safety, not initial resolution.

Verification checkpoint:

1. persisted manifests for `codex_world` keep `agent_id = codex_world`, `backend_id = cli:codex_world`, `resolved_agent_kind = codex`,
2. retained-member reconstruction continues to work for aliased exact backends,
3. host attach truth and world member truth stay architecturally distinct.

Risks:

1. accidentally changing canonical persisted spellings,
2. widening the fix into a wire compatibility migration.

Mitigation:

1. preserve canonical runtime-family enum strings,
2. keep wire/persistence changes minimal and local to readers if possible.

### Phase 4: Test Fixture Conversion, Regression Coverage, and Docs

Goal:

1. convert runtime-realizable fixtures to the explicit runtime-family contract,
2. add alias-specific regression coverage,
3. align docs with the new greenfield contract.

Primary touch surface:

1. `crates/shell/tests/agent_public_control_surface_v1.rs`
2. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
3. `crates/shell/tests/repl_world_first_routing_v1.rs`
4. nearby unit tests in `validator.rs`, `async_repl.rs`, `state_store.rs`
5. `docs/CONFIGURATION.md`
6. slice docs in `llm-last-mile/` as needed

Required coverage:

1. host orchestrator `codex` plus world alias `codex_world`,
2. doctor/runtime-realizability success for explicit alias entries,
3. exact targeted world turns for `::cli:codex_world`,
4. retained-member reuse/relaunch preserving exact backend identity,
5. fail-closed missing-field behavior.

Why last:

1. tests should lock the final contract, not a half-finished intermediate state,
2. docs should describe the exact shipped contract.

Verification checkpoint:

1. all runtime-realizable test helpers explicitly declare `runtime_family`,
2. zero remaining production reliance on literal `agent_id` for runtime-family resolution,
3. docs explain that supported shell-owned runtime families are explicit inventory declarations, while policy still keys on exact backend ids.

Risks:

1. because no compatibility path exists, the test-fixture churn may be broad,
2. doc language may accidentally drift into policy/config overreach.

Mitigation:

1. treat fixture conversion as an explicit phase, not incidental cleanup,
2. keep doc wording disciplined: inventory field for realization, policy for exact backend selectors.

## Sequencing and Parallelism

### Must stay sequential

1. Phase 1 before Phase 2 because runtime resolution needs typed inventory truth first.
2. Phase 2 before Phase 3 because persistence/parity should reflect the corrected runtime contract.
3. Phase 2 before most integration test updates because the intended behavior has to exist before alias regressions can pass.

### Can be parallelized later

1. unit-test updates in `validator.rs` and `agent_inventory.rs` can proceed in parallel once the schema shape is stable,
2. integration fixture conversion can be split by suite,
3. doc updates can proceed in parallel with late-stage testing once wording is frozen.

## Verification Wall

Minimum verification before calling the work complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell agent_runtime::validator -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
```

Recommended focused smoke after tests:

```bash
substrate agent doctor --json
substrate agent start --backend cli:codex --scope host --prompt "host ok" --json
substrate agent start --backend cli:codex_world --scope world --prompt "world ok" --json
substrate agent turn --session <session_id> --backend cli:codex_world --prompt "next" --json
```

## Major Risks and Mitigations

### Risk 1: Field placed in the wrong schema layer

If `runtime_family` lands in top-level config or policy, the contract will fight the repo's typed ergonomics.

Mitigation:

1. keep it only in `config.cli`,
2. keep policy unchanged,
3. document the separation explicitly.

### Risk 2: Runtime-family requirement leaks too broadly

If the field becomes mandatory for all CLI rows, unrelated inventory becomes noisy and semantically misleading.

Mitigation:

1. gate the requirement inside the shell-owned UAA runtime-realizability path only,
2. preserve ordinary inventory parse success for non-UAA CLI rows.

### Risk 3: Persistence contract accidentally changes

If the fix changes canonical `resolved_agent_kind` spellings, retained-session and wire behavior could drift.

Mitigation:

1. preserve canonical runtime-family serialization,
2. change only how runtime family is sourced before launch, not how it is stored after launch.

### Risk 4: Greenfield stance causes underestimated fixture churn

Because there is no compatibility fallback, every runtime-realizable fixture that currently relies on implicit `codex` / `claude_code` behavior must be updated.

Mitigation:

1. inventory-search the suites first,
2. convert shared helper writers before chasing individual failing tests,
3. keep fixture conversion as a named workstream.

## Review Questions

1. Is the inventory-only field placement acceptable as the final contract shape?
2. Is the planned requirement scope narrow enough: only `kind=cli` + `protocol=substrate.agent.session` + effective `cli.mode=persistent`?
3. Is the greenfield stance acceptable even though it broadens test/doc churn by removing any fallback?

