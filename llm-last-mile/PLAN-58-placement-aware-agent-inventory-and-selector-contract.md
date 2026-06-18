# Plan: Placement-Aware Agent Inventory And Selector Contract

Source spec: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Related landed slice: [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Plan type: inventory/selector contract redesign after world-runtime contract landing  
Status: draft for review  
Implementation posture: spec-first, bounded migration, preserve landed Slice `59` runtime/bootstrap behavior while migrating config and selector shape

## Objective

Implement the approved placement-aware logical-agent contract so the repo can stop modeling host/world as permanently duplicated logical agents while preserving:

1. exact backend-id selection,
2. fail-closed host/world separation,
3. explicit runtime-family truth,
4. and the already-landed Slice `59` runtime-realizability / world-deps / installer contract during the placement-aware cutover.

## Plan Summary

The current tree already has both the runtime-family alias correction and the landed Slice `59` world-runtime truth floor, but it still packages host and world as duplicate logical agents such as `codex` / `codex_world` and `claude_code` / `claude_code_world`.

The correct next move is:

1. add a new placement-aware inventory schema version,
2. project each enabled placement into a concrete realized backend row,
3. close the validation-versus-live-control-surface gap with an interim compatibility bridge,
4. derive exact backend ids and display labels from that projection,
5. migrate policies/docs/tests from old split exact ids to placement-qualified exact ids,
6. and stop there.

Do **not** widen this slice into new guest-runtime provisioning, world-deps authoring, or member bootstrap remediation. Those seams are already landed in Slice `59` and should only be touched here when the placement-aware cutover requires mechanical exact-id preservation.

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
4. The concrete world-runtime/bootstrap contract remains the Slice `59` authority; this slice only relocates how that truth is represented after placement-aware migration.
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

### Phase 1.5: Bridge Single-Placement V2 Into Live Control Surfaces

Goal:

1. eliminate the false-green state where `version: 2` inventory validates but disappears from live inventory consumers,
2. keep single-placement `version: 2` inventory materially usable before Packet `2`,
3. fail closed explicitly for multi-enabled `version: 2` inventory until placement-qualified exact ids are live.

Primary touch surface:

1. `crates/shell/src/execution/agent_inventory.rs`
2. `crates/shell/tests/agents_validate.rs`
3. `crates/shell/tests/agent_public_control_surface_v1.rs`
4. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Required changes:

1. materialize unambiguous single-placement `version: 2` inventory into the current effective-inventory/control-surface bridge,
2. preserve legacy host-vs-world exact-id truth inside that bridge so host-only and world-only placements do not blur together,
3. replace silent omission for multi-enabled `version: 2` inventory with an explicit fail-closed diagnostic until Packet `2` lands,
4. keep the forward placement-qualified selector contract deferred to Packet `2`.

Verification checkpoint:

1. single-placement `version: 2` inventory is visible to current list/doctor/start-adjacent control surfaces,
2. workspace-local `version: 2` inventory no longer yields stale or missing effective-inventory truth for single-placement entries,
3. multi-enabled `version: 2` inventory cannot silently validate green and then disappear from live surfaces,
4. no `cli:codex-host` / `cli:codex-world` cutover has happened yet.

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
2. apply the same forward contract to any other split multi-placement CLI inventory still modeled as `*_world` pairs,
3. move exact allowlists/docs/tests to placement-qualified ids,
4. remove the duplicated-file product truth.

Primary touch surface:

1. `config/agents/`
2. `docs/CONFIGURATION.md`
3. `crates/shell/src/execution/policy_model.rs`
4. `crates/shell/src/repl/async_repl.rs`
5. `crates/shell/tests/agent_public_control_surface_v1.rs`
6. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
7. `crates/shell/tests/repl_world_first_routing_v1.rs`
8. `scripts/substrate/dev-fresh-install-gateway-smoke*.sh`
9. nearby validator/dispatch/runtime-control tests that still pin legacy exact ids

Required changes:

1. replace split inventory files with placement-aware equivalents,
2. update policy examples, REPL/runtime-control fixtures, smoke helpers, and docs from legacy `*_world` exact ids to placement-qualified exact ids,
3. update follow-up syntax expectations and read-only/status examples,
4. make the migration boundary explicit in docs/release notes if old ids remain temporarily supported.

Verification checkpoint:

1. no authoritative docs still describe split `*_world` files as the forward model,
2. tests pin placement-qualified exact backend ids,
3. policy examples are exact and current,
4. the repo has one logical-agent file per multi-placement agent.

### Phase 4: Final Validation Wall

Goal:

1. prove the placement-aware selector contract is coherent end-to-end,
2. prove the cutover did not weaken or blur the landed Slice `59` runtime/bootstrap behavior.

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
4. landed Slice `59` runtime-truth, remediation, and installer semantics remain intact after the placement-aware migration.

## Sequencing And Parallelism

1. Phase 1 must land before any compatibility bridge or selector cutover.
2. Phase 1.5 must land before Packet `2`; otherwise `version: 2` inventory remains misleadingly non-live.
3. Phase 2 must land before policy/doc/test migration can be honest.
4. Phase 3 should happen after selector derivation is stable so the docs/tests lock the real contract.
5. Any runtime-contract reopen work should happen only if implementation uncovers a contradiction with landed Slice `59`; otherwise runtime/bootstrap behavior stays out of scope.

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

### Risk 3: Placement migration accidentally weakens the landed Slice 59 runtime contract

If exact-id or inventory migration drops, hides, or reinterprets the world-runtime fields that Slice `59` already made truthful, the repo could clean up names while regressing real launchability semantics.

Mitigation:

1. keep the placement-local ownership rule explicit in the spec,
2. treat Slice `59` as the preserved runtime authority during every migration packet,
3. reopen runtime docs/code only if a concrete contradiction appears during cutover.

### Risk 4: Validation and live control surfaces drift apart during the migration window

If `agents validate` says a `version: 2` inventory is good while `agent list`, `agent doctor`, or exact-backend launch surfaces silently omit it, the packet boundary becomes misleading and later packets build on disputed truth.

Mitigation:

1. insert an explicit Phase `1.5` compatibility bridge,
2. make multi-enabled pre-cutover states fail closed diagnostically instead of disappearing,
3. add control-surface coverage alongside parse/projection coverage before beginning Packet `2`.

## Non-Goals

1. Re-fixing or redesigning the landed Slice `59` world-runtime realizability, world-deps delivery, or installer contract except where the placement-aware cutover requires mechanical exact-id migration.
2. Defining world-deps package contents for Codex.
3. Changing backend-id grammar to support `cli:codex:world`.
4. Widening public selector ergonomics to logical-agent shorthand.
5. Reopening runtime-family alias support.
