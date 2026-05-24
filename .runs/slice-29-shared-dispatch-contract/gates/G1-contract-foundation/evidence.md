# G1 Evidence

`G1` passes because the integrated authoritative tree satisfies the contract-foundation acceptance criteria in `ORCH_PLAN.md`.

1. `dispatch_contract.rs` exists and is exported from `agent_runtime/mod.rs`.
   - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:1)
   - [`crates/shell/src/execution/agent_runtime/mod.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs:2)
2. Inventory-backed and persisted-attach-backed baseline domains are explicit, with per-field provenance tracked inside the shared contract owner.
   - Inventory projection/origin helpers: [`agent_inventory.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:105), [`agent_inventory.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs:196)
   - Inventory and persisted-attach resolvers: [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:210), [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:313), [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:389), [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:470)
3. Policy remains narrowing-only and fail-closed, and the contract produces runtime materialization inputs instead of a second merge owner.
   - Override/policy rejection layers and provenance: [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:156), [`dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:442)
   - Runtime materialization from `ResolvedLaunchContract`: [`validator.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:124), [`validator.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:173), [`validator.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs:231)
4. `control.rs` consumes downstream descriptor/state-store results rather than owning top-level merge semantics, so no `L0` control edit was required.
   - Public prompt dispatch resolves through exact participant/session targeting only: [`control.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:1601), [`control.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:2272)
   - The focused contract-semantic control test passes: [`control.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:2733)
5. The accepted authoritative diff stayed inside the authorized `L0` hotspot set.
   - `git diff --name-only HEAD^ HEAD` => `agent_inventory.rs`, `dispatch_contract.rs`, `mod.rs`, `validator.rs`
   - No durable-state, public CLI, REPL, docs, or `.runs/**` worker edits were merged from the lane.
6. Focused authoritative validation passed.
   - `cargo fmt --all -- --check`
   - `cargo test -p shell dispatch_contract -- --nocapture`
   - `cargo test -p shell validate_member_selection_returns_descriptor_for_unique_world_member -- --nocapture`
   - `cargo test -p shell validate_exact_backend_selection_bypasses_world_ambiguity_when_backend_matches_exactly -- --nocapture`
   - `cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture`
7. The accepted authoritative tip is recorded and `L1` branches from it exactly.
   - `accepted_tip_after_G1 = 50a450a05f374e46c736f79f9b5c7fec5c0e54d9`
   - `L1` worktree was created from that SHA at `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a3-durable-attach-freeze`

Ancillary evidence:

- A broader `cargo test -p shell validate_ -- --nocapture` attempt earlier hit unrelated linker bus errors from integration targets matched by the broad filter; the focused selectors above are the authoritative `G1` evidence.
- An authoritative `cargo clippy -p shell --lib --tests -- -D warnings` attempt was blocked by disk exhaustion before cleanup. This is an environment incident, not a code regression, and is recorded in the task artifacts.
- `npx gitnexus detect-changes --repo substrate --scope compare --base-ref HEAD^` now returns `Changes: 6 files, 16 symbols`, `Affected processes: 0`, `Risk level: low`, but the stale index still misclassifies some symbols and includes unrelated `AGENTS.md`/`CLAUDE.md` drift.
