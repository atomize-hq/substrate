# CLOSEOUT-31 Packet 4: Lazy Host Attach For Host-Rooted World Start

Source spec: [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)  
Source plan: [PLAN-31.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-31.md)  
Source tasks: [TASKS-31.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-31.md)  
Packet: `4`  
Date: `2026-05-29`

## Status

Automated Packet 4 implementation and validation are complete on 2026-05-29.

Packet 4 acceptance is still missing one explicit evidence item from the task list:

1. Linux manual smoke evidence was not captured in this session.

## Landed Scope

1. Hidden owner helper launch now flows through one shared path in [`control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs) for manual `reattach` and router-owned automatic attach.
2. Router-owned automatic attach now builds its launch plan from persisted orchestration-session truth in [`auto_attach.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs), preserving narrowing-only attach resolution and continuity-first behavior.
3. Automatic attach falls back to fresh attach planning only when persisted truth allows it and continuity is unavailable; missing required continuity fails closed before launch.
4. Fresh attach preparation no longer forces resume-only startup extensions in [`async_repl.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs), which keeps manual and automatic attach bootstrap behavior aligned.
5. Prompt-fulfillment control plumbing now exposes a fresh-attach control path in [`prompt_fulfillment.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/prompt_fulfillment.rs), even though the current Codex wrapper still rejects an empty prompt at runtime.

## Validation

The following commands passed on 2026-05-29:

1. `cargo test -p shell --lib hidden_owner_helper_attach_startup_allows_fresh_attach_without_continuity -- --nocapture`
2. `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
3. `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
4. `cargo fmt --all -- --check`
5. `cargo clippy --workspace --all-targets -- -D warnings`
6. `cargo test --workspace -- --nocapture`

## Notable Constraint

The current Codex control wrapper still rejects a prompt-free fresh control attach with `prompt must not be empty`.

That means Packet 4 now proves:

1. continuity-first automatic attach planning,
2. fresh-fallback planning/bootstrap seam behavior when continuity is unavailable,
3. fail-closed behavior when persisted truth is missing or drifted,
4. shared authority and interop across manual and automatic attach.

It does not yet prove a fully verified end-to-end prompt-free Codex fresh attach runtime path.
