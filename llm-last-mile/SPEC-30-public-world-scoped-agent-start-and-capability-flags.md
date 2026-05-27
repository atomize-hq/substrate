# Spec: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Decomposition basis: feature-slice breakdown produced on 2026-05-27  
Phase: `SPECIFY`  
Status: implementation-aligned on 2026-05-27; the Linux-first public world-start contract, `born_unattached`, and public capability-narrowing behavior now match the runtime in this branch

## Assumptions

These are the assumptions I am making so the spec stays concrete. Correct any of them before moving to planning.

1. Omitting `--scope` must preserve current behavior, which means public `agent start` still defaults to host-scoped start.
2. Public capability controls use `--disable-capability <capability>` as the primary flag and `--disable-cap <capability>` as the alias, with values restricted to the already-landed narrowing family from slice 29.75:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`.
3. Public world-scoped root start is explicitly Linux-first for this slice; non-Linux behavior must fail closed with explicit guidance.
4. Born-unattached host-rooted world start uses the distinct operator-visible label `born_unattached` and must not reuse `parked_resumable` or `detached_reattachable` when that would imply prior attach history.
5. This slice may change CLI parsing, runtime session state, status projection, and docs, but it must not change the durable authority model validated by slices 28.5, 29, and 29.75.

## Objective

Build a public `substrate agent start --scope world ...` surface that launches a world-scoped worker under a host-rooted durable orchestration session, persists authoritative host attach truth at session birth, and avoids eager host execution-client startup.

Primary operator story:

1. The operator selects `--scope host` or `--scope world` explicitly.
2. `--scope host` preserves the current root-start behavior.
3. `--scope world` creates a host-rooted durable session, persists the resolved host attach contract, launches a world worker through the shared dispatch contract, and leaves host attach deferred.
4. Before any sanctioned host attach occurs, status and follow-up surfaces must tell the truth about the session without implying active host ownership or prior attach continuity.

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
  - Helper launch plans, prompt streaming envelopes, and Linux world-member submit behavior
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
   - Validate host start parity, world-scoped root start success or fail-closed posture, deferred host attach, world member launch, and public follow-up behavior.
3. Integration tests in `agent_successor_contract_ahcsitc0.rs`
   - Validate status / doctor JSON truth for born-unattached host-rooted sessions and preserve current host vs world projection contracts.
4. Manual smoke checks
   - Validate the exact operator story for `start`, `status`, `reattach`, and `turn`.

Coverage expectations:

- Every new public flag must have at least one positive parser/behavior test and one negative fail-closed test.
- Every new status/posture state must have command-level JSON assertions.
- Existing host-only public root-start behavior must keep regression coverage so this slice cannot silently break it.
- `born_unattached` must have explicit status-surface coverage distinct from detached continuity coverage.

## Boundaries

- Always:
  - Reuse the shared dispatch contract from slice 29 instead of adding a CLI-only launch dialect.
  - Keep durable authority host-rooted for all `--scope world` starts.
  - Persist authoritative `HostAttachContract` truth at session birth.
  - Fail closed on unsupported scope/backend combinations and unsupported capability overrides.
  - Update docs and tests together with runtime behavior.
- Ask first:
  - Introducing any new dependency.
  - Changing the public JSON shape beyond what this feature requires for truthful born-unattached status.
  - Broadening capability overrides beyond the five narrowing-only fields.
  - Relaxing the current Linux-first world-member behavior or changing cross-platform rollout promises.
- Never:
  - Reopen standalone world-root continuity.
  - Reintroduce backfill/repair logic for missing persisted attach truth.
  - Treat born-unattached sessions as `parked_resumable` or `detached_reattachable` if that implies prior attach history.
  - Start a host execution client eagerly just to manufacture ownership theater for `--scope world`.
  - Add a second public follow-up dialect for world workers.

## Success Criteria

The feature is done only when all of the following are true:

1. `substrate agent start` accepts explicit scope selection through one shared dispatch-envelope flow.
2. `substrate agent start --scope host` preserves the current host-rooted root-start behavior.
3. `substrate agent start --scope world` creates a host-rooted durable orchestration session, persists authoritative host attach truth at birth, launches a world worker/member under that session, and does not eagerly start a host execution client.
4. Public capability flags, if present, only affect the already-supported narrowing family:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`, exposed as `--disable-capability <capability>` with `--disable-cap <capability>` as the alias.
5. Unsupported capability fields such as `session_start`, `llm`, and `mcp_client` remain fail closed.
6. A born-unattached host-rooted world-start session has the truthful non-terminal operator-visible status `born_unattached`, which does not imply active host attachment or prior attach history.
7. Public world-scoped root start is supported only on Linux for this slice; non-Linux platforms fail closed with explicit posture guidance.
8. Detached-world follow-up remains fail closed until sanctioned host attach occurs.
9. `docs/USAGE.md`, the slice doc, and integration tests all describe the same operator contract.

## Resolved Decisions

These review decisions are now frozen for this spec:

1. Public capability narrowing uses `--disable-capability <capability>` with `--disable-cap <capability>` as the alias.
2. There is no single-letter short flag for capability narrowing.
3. The born-unattached operator-visible status label is `born_unattached`.
4. Public world-scoped root start is Linux-first for this slice.
5. Omitting `--scope` preserves the current host-scoped default behavior.

## Review Gate

Do not advance to `PLAN` until a human has reviewed this spec and either:

1. accepted the assumptions as written, or
2. corrected the assumptions or frozen decisions that should drive the plan.
