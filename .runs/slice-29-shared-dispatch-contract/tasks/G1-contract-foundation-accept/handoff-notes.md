Parent acceptance notes:

- Accepted authoritative commit: `50a450a05f374e46c736f79f9b5c7fec5c0e54d9`
- Recorded in `branch-map.json` as `accepted_tip_after_G1`
- Opened `L1` branch `codex/feat-gateway-mediated-llm-fulfillment-s29-a3-durable-attach-freeze`
- `L1` worktree path: `/home/azureuser/__Active_Code/atomize-hq/.worktrees/substrate-s29-shared-dispatch-contract/a3-durable-attach-freeze`

Important carry-forward constraints:

- `dispatch_contract.rs`, `agent_inventory.rs`, and `validator.rs` are now frozen after `G1`.
- `L1` may touch only `orchestration_session.rs`, `state_store.rs`, and directly related tests.
