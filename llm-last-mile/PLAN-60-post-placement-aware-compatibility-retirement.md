# Plan: Post-Placement-Aware Compatibility Retirement

Source spec: [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md)  
Related landed slices:
- [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)
- [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Plan type: bounded compatibility retirement after placement-aware cutover  
Status: draft for review  
Implementation posture: direct retirement of forward compatibility posture, preserve Slice `59` runtime truth, do not widen into unrelated historical cleanup

## Objective

Implement the post-cutover retirement slice so the repo stops carrying the Slice `58` compatibility bridge and stops presenting split-entry exact backend ids as forward truth.

This plan must land:

1. one explicit boundary between historical references and forward product truth,
2. retirement of the placement-aware effective-inventory compatibility bridge,
3. fail-closed retirement of split-entry exact selectors,
4. migration of forward docs/examples/tests/policies/scripts to the final placement-aware identity model,
5. and a final proof that Slice `59` runtime truth still holds after the retirement.

## Plan Summary

The repo has already crossed the meaningful semantic boundary:

1. placement-aware `version: 2` manifests are the forward inventory model,
2. placement-qualified exact backend ids are already present in forward docs/config,
3. Slice `59` has already made world-runtime truth honest,
4. but the tree still carries compatibility posture from the cutover and still contains forward-facing references to split-entry ids.

So the correct next move is:

1. freeze a precise retirement boundary,
2. remove the live compatibility bridge,
3. make old split-entry selectors fail closed instead of silently coexisting,
4. migrate remaining forward surfaces to the final identity model,
5. prove historical references are either intentional history or explicit negative tests.

Do **not** widen this slice into:

1. new runtime/artifact delivery work,
2. session-store/flat-record compatibility retirement unrelated to placement-aware identity,
3. or a broad rewrite of historical planning artifacts that are only preserved as provenance.

## Locked Decisions

### What changes

1. Placement-aware inventory remains the only forward model.
2. Placement-qualified exact backend ids become the only forward selectors.
3. The `version: 2` to split-entry effective-inventory compatibility bridge is retired from forward live behavior.
4. Legacy split-entry exact ids such as `cli:codex_world` fail closed with explicit guidance if encountered on live control surfaces.
5. Forward docs/examples/tests/policies/scripts are normalized to the final placement-aware identity model.

### What does not change

1. Slice `59` runtime-realizability, world-deps, and installer semantics remain frozen floor.
2. Backend-id grammar stays `<kind>:<name>`.
3. Policy remains keyed on exact backend ids.
4. Historical `llm-last-mile/` records may remain historical evidence.
5. This slice does not become a general session-store compatibility retirement.

## Implementation Order

### Phase 1: Freeze The Retirement Boundary

Goal:

1. distinguish forward product truth from historical evidence,
2. define the grep wall and historical allowlist,
3. prevent this slice from turning into an unbounded rename hunt.

Primary touch surface:

1. `llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md`
2. `llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md`
3. `llm-last-mile/TASKS-60.md`
4. narrow test/docs comments only if they must explicitly mark a retained historical reference as historical

Required changes:

1. define which surfaces count as forward truth,
2. define which surfaces may keep old ids as history or negative tests,
3. encode the grep wall the slice will use for verification,
4. name the temporary Packet `1` allowlist seams so later packets can remove them deliberately instead of treating every grep hit as equally in-scope.

Verification checkpoint:

1. the slice has an explicit historical allowlist,
2. implementation work can focus on forward surfaces without rewriting provenance,
3. reopen criteria are explicit if hidden live dependencies appear.

Phase `1` boundary contract:

1. The forward-truth grep wall remains `docs/`, `config/`, `crates/shell/`, and `scripts/` so the final slice proof stays honest.
2. `docs/`, `config/`, and `scripts/` are zero-tolerance forward-truth surfaces for legacy split-entry ids.
3. `llm-last-mile/` remains outside that wall as historical provenance, not active operator truth.
4. Temporary allowlist seams are limited to:
   - `crates/shell/src/execution/agent_inventory.rs`
   - `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
   - `crates/shell/src/execution/agent_runtime/validator.rs`
   - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - `crates/shell/src/builtins/world_gateway.rs`
   - `crates/shell/src/execution/agent_runtime/control.rs`
   - `crates/shell/src/execution/agent_runtime/host_inbox.rs`
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
5. `crates/shell/tests/**` and inline `#[cfg(test)]` coverage may retain legacy names only when asserting bridge-removal coverage, persisted-state continuity, or fail-closed retirement behavior; Packet `4` must narrow the remaining hits to those intentional cases.
6. If implementation discovers a live dependency outside that bounded allowlist, or any unavoidable old-id hit in `docs/`, `config/`, or `scripts/`, stop and reopen spec/plan/tasks before widening scope.
7. If removing an allowlisted hit would reopen Slice `59` runtime semantics, treat that as a reopen condition rather than routine Slice `60` work.

### Phase 2: Retire The Effective-Inventory Compatibility Bridge

Goal:

1. stop materializing placement-aware manifests back into split-entry-shaped effective rows as forward live behavior,
2. keep live control surfaces aligned to realized placement identities,
3. avoid reopening runtime truth while removing the bridge.

Primary touch surface:

1. `crates/shell/src/execution/agent_inventory.rs`
2. `crates/shell/tests/agents_validate.rs`
3. `crates/shell/tests/agent_public_control_surface_v1.rs`
4. `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`

Required changes:

1. retire the Packet `1.5` compatibility materialization path from normal live behavior,
2. keep placement-aware realized identities authoritative in effective inventory,
3. make any unsupported legacy path fail closed explicitly instead of silently backfilling split-entry rows.

Verification checkpoint:

1. effective inventory is placement-aware only,
2. no live control surface still depends on split-entry materialization,
3. old ids are no longer synthesized as normal output.

### Phase 3: Retire Split-Entry Exact Selector Compatibility

Goal:

1. stop treating split-entry exact backend ids as forward-valid selectors,
2. keep exact backend selection fail-closed,
3. preserve Slice `59` runtime truth and exact placement semantics.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/validator.rs`
2. `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
3. `crates/shell/src/execution/policy_model.rs`
4. `crates/shell/src/repl/async_repl.rs`
5. adjacent runtime/control tests

Required changes:

1. reject retired exact ids with explicit migration guidance,
2. keep placement-qualified exact ids working,
3. ensure runtime/control/policy surfaces do not silently reinterpret old ids as new ids,
4. preserve exact host/world placement truth after retirement.

Verification checkpoint:

1. `cli:codex-host` / `cli:codex-world` remain green,
2. `cli:codex_world` / `cli:claude_code_world` fail closed with stable guidance,
3. no runtime surface silently cross-maps old and new ids.

### Phase 4: Migrate Forward Docs, Fixtures, Policies, And Smoke Helpers

Goal:

1. remove contradictory product truth from active forward surfaces,
2. keep historical references only where they are explicitly historical,
3. align examples and smoke helpers with the final identity model.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `scripts/substrate/dev-fresh-install-gateway-smoke.sh`
3. `scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh`
4. forward-facing test fixtures under `crates/shell/tests/`
5. any active policy/example surfaces still pinning split-entry ids

Required changes:

1. update active docs/examples/policies/scripts to placement-qualified exact ids only,
2. keep negative tests that intentionally mention retired ids clearly negative,
3. avoid scrubbing historical `llm-last-mile/` records unless clarification is required.

Verification checkpoint:

1. forward docs/scripts/examples no longer present old ids as live current truth,
2. remaining old-id hits are either historical evidence or explicit retirement tests,
3. operator guidance is consistent end to end.

### Phase 5: Final Validation Wall

Goal:

1. prove the compatibility retirement is coherent end to end,
2. prove the retirement did not reopen Slice `59` runtime semantics,
3. prove the repo has one forward identity model.

Verification wall:

1. `cargo fmt --all -- --check`
2. `cargo clippy --workspace --all-targets -- -D warnings`
3. `cargo test -p shell agents_validate -- --nocapture`
4. `cargo test -p shell agent_inventory -- --nocapture`
5. `cargo test -p shell dispatch_contract -- --nocapture`
6. `cargo test -p shell agent_runtime::validator -- --nocapture`
7. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
8. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
9. `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
10. forward-truth negative grep on `docs`, `config`, `crates/shell`, and `scripts`

Exit criteria:

1. the effective-inventory compatibility bridge is gone from forward live behavior,
2. placement-qualified exact ids are the only forward selectors,
3. retired split-entry ids fail closed with explicit guidance,
4. forward product truth is placement-aware only,
5. Slice `59` runtime truth remains intact.

## Sequencing And Parallelism

1. Phase 1 must land before code retirement begins.
2. Phase 2 must land before selector retirement can be honest.
3. Phase 3 must land before doc/policy/script migration can be called complete.
4. Phase 4 must land before the final grep wall means anything.
5. Phase 5 closes the slice only after both runtime tests and grep proof are green.

Limited parallelism that is acceptable:

1. doc/example inventory of old-id references can happen while Phase 2 implementation is underway,
2. policy fixture updates can be drafted while validator/dispatch retirement is being finalized,
3. but code landing remains sequential because the bridge removal and selector retirement define the contract the docs/tests must lock.

## Risks

### Risk 1: Hidden live dependency on the compatibility bridge

If a live control surface still depends on v2-to-v1 materialization, bridge removal could appear to “break placement-aware inventory” even though the real bug is incomplete consumer cutover.

Mitigation:

1. freeze the boundary first,
2. use control-surface tests as the checkpoint,
3. reopen the slice only if a genuine live dependency survives outside the Packet `1` retirement inventory.

### Risk 2: Historical docs get mistaken for forward truth

If the grep wall is too broad or too vague, the slice may either rewrite provenance unnecessarily or leave contradictory active surfaces behind.

Mitigation:

1. maintain an explicit historical allowlist,
2. keep forward-surface grep and historical provenance review separate,
3. prefer comment framing over provenance destruction.

### Risk 3: Selector retirement accidentally reopens Slice 59 runtime semantics

If old-id retirement changes runtime-family or world-binary semantics instead of just identity, the slice will spill into runtime redesign.

Mitigation:

1. keep Slice `59` as frozen floor,
2. run validator/dispatch/runtime regression tests after each packet,
3. treat runtime-semantic changes as reopen conditions.

### Risk 4: Old and new exact ids remain coequal in tests or policies

If some fixtures still treat `cli:codex_world` as valid while others require `cli:codex-world`, operator truth will stay contradictory.

Mitigation:

1. migrate forward fixtures and policy examples in the same slice,
2. keep any old-id references only in negative tests or clearly historical surfaces,
3. include a grep proof in the final wall.

## Non-Goals

1. Reopening Slice `59` runtime/artifact/install semantics.
2. Redesigning backend-id grammar.
3. Retiring unrelated session-store compatibility bridges unless a direct Slice `60` dependency is discovered.
4. Rewriting historical planning artifacts merely to erase old names from provenance.
5. Adding a new alias window or another compatibility bridge after the cutover.
