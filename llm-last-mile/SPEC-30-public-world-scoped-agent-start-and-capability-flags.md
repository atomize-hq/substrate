# Spec: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Decomposition basis: feature-slice breakdown produced on 2026-05-27  
Phase: `SPECIFY`  
Status: draft realigned to host-first product intent on 2026-05-27

## Assumptions

These are the assumptions I am making so the spec stays concrete. Correct any of them before moving to planning.

1. Omitting `--scope` should resolve requested execution substrate through workspace-local config/profile/policy first, then global config/policy, rather than hardcoding host.
2. Public capability controls use `--disable-capability <capability>` as the primary flag and `--disable-cap <capability>` as the alias, with values restricted to the already-landed narrowing family from slice 29.75:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`.
3. Public world-scoped root start is explicitly Linux-first for this slice; non-Linux behavior must fail closed with explicit guidance.
4. `--scope host` is the explicit bypass-world path: orchestration starts on the host and later dispatch stays host-scoped unless a later slice reopens that behavior.
5. The thin slice should treat world scope as the default execution substrate behind a host session, not as “run the first visible prompt directly in a world agent before the host session is attached.”
6. This slice may change CLI parsing, runtime session state, world binding/session setup, and docs, but it must not change the durable authority model validated by slices 28.5, 29, and 29.75.

## Objective

Build a public `substrate agent start` surface that launches a host-rooted durable orchestration session first, persists authoritative host attach truth at session birth, and treats world scope as the default execution substrate the host agent later uses for dispatched world work.

Primary operator story:

1. The operator may omit `--scope`, or select `--scope host` / `--scope world` explicitly.
2. Omitting `--scope` resolves requested execution substrate through workspace-local config/profile/policy first, then global config/policy.
3. `--scope host` explicitly bypasses world.
4. `--scope world` explicitly requests the default world-backed path while still starting a host-rooted attached orchestration session.
5. The inaugural operator prompt is fulfilled by the host orchestration agent; later world work is dispatched by that host agent under the established world session/binding.

## Tech Stack

- Language: Rust 2021, MSRV 1.89+
- CLI parsing: `clap` in [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
- Public agent control surface: [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- Shared dispatch contract: [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- Durable session truth: [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- World member follow-up and dispatch plumbing: [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), [`crates/shell/src/execution/routing/dispatch/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch)
- Operator docs: [`docs/USAGE.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md), [`llm-last-mile/README.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)

## Commands

Build:

```bash
cargo build --workspace
```

Format:

```bash
cargo fmt --all -- --check
```

Lint:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```

Targeted tests:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
```

Full test wall:

```bash
cargo test --workspace -- --nocapture
```

Manual operator validation targets:

```bash
substrate agent start --backend <backend_id> --prompt "hello" --json
substrate agent start --backend <backend_id> --scope world --prompt "hello" --json
substrate agent start --backend <backend_id> --scope world --disable-capability event_stream --json
substrate agent status --json
substrate agent turn --session <orchestration_session_id> --backend <backend_id> --prompt "next" --json
substrate agent doctor --json
```

## Project Structure

This feature is expected to touch these areas:

- `crates/shell/src/execution/cli.rs`
  - Public CLI argument definitions for `substrate agent ...`
- `crates/shell/src/execution/agents_cmd.rs`
  - Public `start`, `turn`, `reattach`, `fork`, `stop`, status rendering, and dispatch-envelope construction
- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - Shared resolver, override validation, provenance, and persisted attach resolution
- `crates/shell/src/execution/agent_runtime/orchestration_session.rs`
  - Durable orchestration session posture/state invariants and persisted attach contract truth
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - Public session posture classification and live-session control/status selection
- `crates/shell/src/execution/agent_runtime/control.rs`
  - Helper launch plans, prompt streaming envelopes, world binding/session setup, and Linux world-member dispatch behavior
- `crates/shell/tests/agent_public_control_surface_v1.rs`
  - End-to-end public CLI control-plane regression coverage
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - Status / doctor / contract regression coverage
- `docs/USAGE.md`
  - Operator-facing contract source for public CLI behavior
- `llm-last-mile/`
  - Planning and scope documents that must stay aligned with shipped behavior

## Code Style

Follow the existing `shell` crate style: narrow helpers, explicit fail-closed user errors, `Result<T, anyhow::Error>`, and exact contract wording in tests.

Example style:

```rust
fn start_dispatch_classifier(
    err: &crate::execution::agent_runtime::DispatchResolutionError,
) -> &'static str {
    use crate::execution::agent_runtime::DispatchResolutionErrorKind;

    match err.kind {
        DispatchResolutionErrorKind::AmbiguousBaselineSelection => "ambiguous_backend",
        DispatchResolutionErrorKind::OverrideDeniedByPolicy => "policy_disallow",
        DispatchResolutionErrorKind::BaselineNotFound => "unknown_backend",
        _ => "runtime_start_failed",
    }
}
```

Conventions:

- Use four-space indentation and Rust 2021 idioms.
- Add small helper functions when a contract split needs a name; do not bury public CLI policy in long inline branches.
- Keep error taxonomy explicit and stable. If a denial is expected by design, tests should assert the classifier and the human-readable reason.
- Reuse shared dispatch and persisted-attach structures instead of inventing start-only copies.
- Keep docs and tests written in the same vocabulary as the runtime.
- Keep public flag spelling explicit: `--disable-capability` is canonical, `--disable-cap` is the only alias, and there is no single-letter short flag.

## Testing Strategy

Frameworks:

- Rust unit tests and integration tests via `cargo test`
- Existing shell integration harnesses under `crates/shell/tests/`

Test levels for this feature:

1. Unit tests in `dispatch_contract.rs`
   - Validate `--scope` mapping, supported capability override families, narrowing-only behavior, policy denial, and persisted-attach constraints.
2. Integration tests in `agent_public_control_surface_v1.rs`
   - Validate omitted-scope resolution order, host-scoped bypass behavior, host-first world-backed start success, world binding/session setup, and public follow-up behavior.
3. Integration tests in `agent_successor_contract_ahcsitc0.rs`
   - Validate host lifecycle/status truth for the new default path and preserve current parked / awaiting-attention projection contracts.
4. Manual smoke checks
   - Validate the exact operator story for `start`, `status`, `reattach`, and `turn`.

Coverage expectations:

- Every new public flag must have at least one positive parser/behavior test and one negative fail-closed test.
- Every new public resolution rule must have command-level assertions.
- Existing host lifecycle semantics (`active_attached`, `parked_resumable`, `awaiting_attention`) must keep regression coverage so this slice cannot silently break them.
- World-backed start must prove host-first prompt handling plus world binding/session setup without depending on a born-unattached default posture.

## Boundaries

- Always:
  - Reuse the shared dispatch contract from slice 29 instead of adding a CLI-only launch dialect.
  - Keep durable authority host-rooted for all `--scope world` starts.
  - Persist authoritative `HostAttachContract` truth at session birth.
  - Treat the inaugural operator prompt as a host-orchestrator concern, even when scope resolves to world.
  - Fail closed on unsupported scope/backend combinations and unsupported capability overrides.
  - Update docs and tests together with runtime behavior.
- Ask first:
  - Introducing any new dependency.
  - Changing the public JSON shape beyond what this feature requires for truthful host lifecycle or scope-resolution reporting.
  - Broadening capability overrides beyond the five narrowing-only fields.
  - Relaxing the current Linux-first world-member behavior or changing cross-platform rollout promises.
- Never:
  - Reopen standalone world-root continuity.
  - Reintroduce backfill/repair logic for missing persisted attach truth.
  - Treat the thin slice as a two-prompt “host plus world bootstrap” product.
  - Route the inaugural public prompt directly into a first world agent while describing the feature as host-first orchestration.
  - Add a second public follow-up dialect for world workers.

## Success Criteria

The feature is done only when all of the following are true:

1. `substrate agent start` accepts explicit scope selection through one shared dispatch-envelope flow.
2. Omitting `--scope` resolves requested execution substrate through workspace-local config/profile/policy first, then global config/policy.
3. `substrate agent start --scope host` explicitly bypasses world and preserves host-rooted root-start behavior.
4. `substrate agent start --scope world` creates a host-rooted durable orchestration session, persists authoritative host attach truth at birth, and establishes world binding/session truth for later host-dispatched world work.
5. The inaugural operator prompt is handled by the host orchestration agent rather than being sent directly to a first world worker/member.
6. Public capability flags, if present, only affect the already-supported narrowing family:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`, exposed as `--disable-capability <capability>` with `--disable-cap <capability>` as the alias.
7. Unsupported capability fields such as `session_start`, `llm`, and `mcp_client` remain fail closed.
8. The default public world-backed path uses the normal host-attached lifecycle and does not require `born_unattached` as the operator-facing happy-path posture.
9. Public world-scoped root start is supported only on Linux for this slice; non-Linux platforms fail closed with explicit posture guidance.
10. `docs/USAGE.md` should not be rewritten ahead of runtime changes, but the slice docs and integration expectations must describe the same intended contract.

## Resolved Decisions

These review decisions are now frozen for this spec:

1. Public capability narrowing uses `--disable-capability <capability>` with `--disable-cap <capability>` as the alias.
2. There is no single-letter short flag for capability narrowing.
3. Public world-scoped root start is Linux-first for this slice.
4. Omitting `--scope` resolves through workspace-local config/profile/policy first, then global config/policy.
5. `--scope host` is the explicit bypass-world path.
6. `born_unattached` is not the default thin-slice happy-path posture.

## Review Gate

Do not advance to `PLAN` until a human has reviewed this spec and either:

1. accepted the assumptions as written, or
2. corrected the assumptions or frozen decisions that should drive the plan.
