Parent acceptance notes:

- Accepted authoritative commit: `42636f8eb68fe2e817d973a4896f5c35cfc5b12b`
- Recorded in `branch-map.json` as `accepted_tip_after_G2`
- Opened `L2` branch `codex/feat-gateway-mediated-llm-fulfillment-s29-a4-human-caller-adoption`
- Opened `L3` branch `codex/feat-gateway-mediated-llm-fulfillment-s29-a5-repl-dispatch-adoption`
- `L2` worktree path: `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a4-human-caller-adoption`
- `L3` worktree path: `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a5-repl-dispatch-adoption`

Important carry-forward constraints:

- `dispatch_contract.rs`, `agent_inventory.rs`, `validator.rs`, `orchestration_session.rs`, and `state_store.rs` are now frozen after `G2`.
- `L2` may touch only `agents_cmd.rs`, `control.rs`, `prompt_fulfillment.rs`, `crates/shell/tests/agent_public_control_surface_v1.rs`, and minimal supporting tests in its owned surface.
- `L3` may touch only `repl/async_repl.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, and `world_ops.rs` only if a concrete additive transport field is proven necessary.
