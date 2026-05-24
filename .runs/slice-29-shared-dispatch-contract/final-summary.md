# Final Summary

Accepted implementation/docs/test tip: `0d15fb2fe8902a9201c891eccd3d7a20325f9d72`

- Landed one shared internal dispatch contract in `dispatch_contract.rs`, with explicit inventory-backed and persisted-attach-backed baseline domains.
- Kept `HostAttachContract` as the only durable host-attach truth and generalized it to persist resolved launch semantics without inventing a second durable attach object.
- Brought the human caller plane and orchestrator-controlled dispatch back onto the same contract semantics, including same-session parked-turn reattach/stop continuity.
- Aligned the allowed docs and llm-last-mile truth surfaces to the merged runtime behavior.

Validation:

- The locked shell selectors passed, including `resolve_public_control_target`, `public_turn_prompt_requests_require_exact_session_and_backend_contract`, `new_session_starts_active_attached`, and `detached_postures_enforce_pending_inbox_truth`.
- `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture` and `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` passed on the accepted tree.
- `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace -- --nocapture` passed.
- `npx gitnexus detect-changes --repo substrate --scope staged` reported `7 files`, `37 symbols`, `0 affected processes`, `risk level: low`.
