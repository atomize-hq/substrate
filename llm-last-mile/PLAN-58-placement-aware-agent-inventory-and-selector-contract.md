# Plan: Placement-Aware Agent Inventory And Selector Contract

Source spec: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Plan type: inventory/selector contract redesign before world-runtime bootstrap follow-on  
Status: draft for review  
Implementation posture: spec-first, bounded migration, no guest-runtime bootstrap work in this slice

## Objective

Implement the approved placement-aware logical-agent contract so the repo can stop modeling host/world as permanently duplicated logical agents while preserving:

1. exact backend-id selection,
2. fail-closed host/world separation,
3. explicit runtime-family truth,
4. and a clean handoff to the later world-runtime realizability slice.

## Plan Summary

The current tree already has the runtime-family alias correction, but it still packages host and world as duplicate logical agents such as `codex` and `codex_world`.

The correct next move is:

1. add a new placement-aware inventory schema version,
2. project each enabled placement into a concrete realized backend row,
3. derive exact backend ids and display labels from that projection,
4. migrate policies/docs/tests from old split exact ids to placement-qualified exact ids,
5. and stop there.

Do **not** widen this slice into guest-runtime provisioning, world-deps authoring, or member bootstrap remediation. Those are follow-on work once the placement container is honest.

## Locked Decisions

### What changes

1. Add a placement-aware logical-agent inventory schema (`version: 2`).
2. Treat top-level `id` as logical-agent identity.
3. Derive placement-qualified realized identities such as `codex-host` and `codex-world`.
4. Derive exact backend ids such as `cli:codex-host` and `cli:codex-world`.
5. Derive human-facing labels such as `codex (host)` and `codex (world)`.

### What does not change

1. Backend-id grammar stays `<kind>:<name>`.
2. Exact backend selection remains fail-closed.
3. `config.cli.runtime_family` remains explicit inventory truth.
4. The world-runtime/bootstrap contract is not defined here.
5. Policy still allowlists exact backend ids, not logical-agent ids.

## Implementation Order

### Phase 1: Add Placement-Aware Inventory Schema And Projection

Goal:

1. introduce `version: 2` logical-agent inventory with `config.placements`,
2. preserve parse support for current split `version: 1` files during migration,
3. project enabled placements into concrete realized rows.

Primary touch surface:

1. `crates/shell/src/execution/agent_inventory.rs`
2. `crates/shell/tests/agents_validate.rs`
3. `docs/CONFIGURATION.md`

Required changes:

1. add typed placement schema for `host` and `world`,
2. introduce projection fields for logical-agent id, placement, realized-agent id, and display label,
3. keep `config.cli.runtime_family` placement-local in the new schema,
4. ensure disabled placements do not realize rows.

Verification checkpoint:

1. `version: 2` inventory parses,
2. projection yields correct realized rows,
3. `version: 1` fixtures still parse if migration support is intentionally retained,
4. no selection semantics change yet.

### Phase 2: Cut Exact Selector Derivation Over To Placement-Qualified Realized Backends

Goal:

1. make exact backend-id derivation placement-qualified for `version: 2` entries,
2. keep exact backend selection fail-closed,
3. prevent logical-agent shorthand from silently choosing host or world.

Primary touch surface:

1. `crates/shell/src/execution/agent_inventory.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/agent_runtime/validator.rs`
4. `crates/transport-api-types/src/lib.rs` only if helper wording needs alignment, not grammar widening

Required changes:

1. derive `backend_id = <kind>:<logical>-<placement>` for placement-aware entries,
2. thread logical-agent id versus realized-agent id distinctly through runtime selection,
3. derive human-facing display labels for read-only/status surfaces,
4. keep multi-placement omission fail-closed.

Verification checkpoint:

1. exact selection works for `cli:codex-host` and `cli:codex-world`,
2. invalid or ambiguous selectors still fail closed,
3. display labels remain derived/read-only,
4. host/world cannot silently cross-match.

### Phase 3: Migrate Inventory, Policies, Docs, And Tests To Placement-Qualified Identity

Goal:

1. replace the split `codex` + `codex_world` topology with one logical placement-aware file,
2. move exact allowlists/docs/tests to placement-qualified ids,
3. remove the duplicated-file product truth.

Primary touch surface:

1. `config/agents/`
2. `docs/CONFIGURATION.md`
3. `crates/shell/tests/agent_public_control_surface_v1.rs`
4. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
5. nearby validator/dispatch tests

Required changes:

1. replace split inventory files with placement-aware equivalents,
2. update policy examples and docs from `cli:codex_world` to `cli:codex-world`,
3. update fixtures and follow-up syntax expectations,
4. make the migration boundary explicit in docs/release notes if old ids remain temporarily supported.

Verification checkpoint:

1. no authoritative docs still describe split `*_world` files as the forward model,
2. tests pin placement-qualified exact backend ids,
3. policy examples are exact and current,
4. the repo has one logical-agent file per multi-placement agent.

### Phase 4: Final Validation Wall

Goal:

1. prove the placement-aware selector contract is coherent end-to-end,
2. stop before guest-runtime bootstrap work begins.

Verification wall:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p shell agent_inventory -- --nocapture`
4. `cargo test -p shell dispatch_contract -- --nocapture`
5. `cargo test -p shell agent_runtime::validator -- --nocapture`
6. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
7. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`

Exit criteria:

1. placement-aware inventory is the forward contract,
2. exact backend ids are placement-qualified and fail-closed,
3. human-facing labels are derived and distinct from selectors,
4. no guest-runtime/bootstrap behavior was implicitly promised or silently changed.

## Sequencing And Parallelism

1. Phase 1 must land before any selector cutover.
2. Phase 2 must land before policy/doc/test migration can be honest.
3. Phase 3 should happen after selector derivation is stable so the docs/tests lock the real contract.
4. Guest-runtime/bootstrap design should start only after this plan is green.

## Risks

### Risk 1: Identity drift between logical and realized agent ids

If implementation keeps using `agent_id` ambiguously, the repo will blur logical grouping and exact realized identity again.

Mitigation:

1. add explicit typed fields for logical id versus realized id,
2. avoid suffix heuristics in downstream code.

### Risk 2: Partial migration leaves old and new exact ids coequal

If some docs/tests/policies say `cli:codex_world` while others say `cli:codex-world`, operator truth will become contradictory.

Mitigation:

1. treat migration as a first-class phase,
2. keep docs/examples/fixtures in the same packet as the selector cutover,
3. fail closed on ambiguous compatibility if needed.

### Risk 3: Placement schema lands without a clear handoff to world-runtime truth

If the placement container lands but the next slice still has no obvious home for guest-runtime requirements, the repo will clean up names without solving the real blocker.

Mitigation:

1. keep the placement-local ownership rule explicit in the spec,
2. immediately follow this slice with the world-runtime realizability contract slice.

## Non-Goals

1. Fixing the actual `exit 127` Codex world bootstrap failure.
2. Defining world-deps package contents for Codex.
3. Changing backend-id grammar to support `cli:codex:world`.
4. Widening public selector ergonomics to logical-agent shorthand.
5. Reopening runtime-family alias support.
