# Validation Wall

Locked merged-tree commands from `ORCH_PLAN.md`:

```bash
cargo test -p shell resolve_public_control_target -- --nocapture
cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture
cargo test -p shell new_session_starts_active_attached -- --nocapture
cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
```

Additional parent proof obligations:

- final merged-tree GitNexus `detect-changes` transcription
- contract/domain parity audit against `PLAN.md`
- docs and `llm-last-mile/` truth-sync audit against merged runtime behavior

## Result

Accepted authoritative implementation/docs/test tip:

- `0d15fb2fe8902a9201c891eccd3d7a20325f9d72`

Locked command results on that accepted tree:

- `cargo test -p shell resolve_public_control_target -- --nocapture` passed
- `cargo test -p shell public_turn_prompt_requests_require_exact_session_and_backend_contract -- --nocapture` passed
- `cargo test -p shell new_session_starts_active_attached -- --nocapture` passed
- `cargo test -p shell detached_postures_enforce_pending_inbox_truth -- --nocapture` passed
- `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture` passed
- `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture` passed
- `cargo fmt --all -- --check` passed
- `cargo clippy --workspace --all-targets -- -D warnings` passed
- `cargo test --workspace -- --nocapture` passed

Additional proof obligations:

- GitNexus `detect-changes` on the staged implementation tree reported `7 files`, `37 symbols`, `0 affected processes`, `risk level: low`
- Contract/domain parity audit passed: the merged runtime keeps `dispatch_contract.rs` as the sole internal contract owner, makes both baseline domains explicit (`InventoryLaunch`, `PersistedHostAttach`), preserves generalized launch truth in `HostAttachContract`, and keeps human/public + orchestrator-controlled dispatch on the same semantics
- Docs truth-sync audit passed for the allowed surfaces in `docs/project_management/adrs/draft/ADR-0027-llm-and-agent-config-policy-surface.md` and `llm-last-mile/{29,30,31,README}.md`
