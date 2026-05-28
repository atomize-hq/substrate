# Spec: Public World-Scoped Agent Start And Capability Flags

Source SOW: [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Decomposition basis: feature-slice breakdown produced on 2026-05-27  
Phase: `SPECIFY`  
Status: reopened on 2026-05-28 after closeout audit found missing required Linux manual smoke evidence; honest Packet 4 closure is currently blocked on world-backed smoke runtime access

## Assumptions

These are the assumptions I am making so the spec stays concrete. Correct any of them before moving to planning.

1. Omitting `--scope` should resolve a preferred execution scope through workspace-local config/profile/policy first, then global config/policy, probe for an exact backend match in that preferred scope, and fall back once to the alternate scope only when the preferred scope has no exact match.
2. Public capability controls use `--disable-capability <capability>` as the primary flag and `--disable-cap <capability>` as the alias, with values restricted to the already-landed narrowing family from slice 29.75:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`.
3. Public world-scoped root start is explicitly Linux-first for this slice; non-Linux behavior must fail closed with explicit guidance.
4. `--scope host` is the explicit bypass-world path: orchestration starts on the host and later dispatch stays host-scoped unless a later slice reopens that behavior.
5. The thin slice should treat world scope as the default execution substrate behind a host session, not as “run the first visible prompt directly in a world agent before the host session is attached,” and the inaugural prompt should therefore remain strictly host-routed.
6. Packet 4 may still tighten status/control-surface behavior, docs, and validation coverage, but it must not change the durable authority model or runtime start floor validated by slices 28.5, 29, 29.75, and landed Packets 1-3 of slice 30.

## Observed Repo Floor

The current repo already freezes the Packet-1 through Packet-3 behavior that this spec now treats as the starting floor:

1. Omitted `--scope` resolves the effective default scope, probes for an exact backend match in that preferred scope, falls back once to the alternate scope if needed, and stamps the resolved scope into `DispatchRequestEnvelope`.
2. Public world-scoped root start now uses the host-first attached runtime model:
   a host-rooted orchestration session is launched through the hidden owner-helper path, the inaugural prompt is host-routed, authoritative world session/binding truth is established before `start` returns, and the successful session remains `active_attached` rather than `born_unattached`.
3. That same Packet-2 floor already persists top-level `world_id` and `world_generation` on the orchestration session as the durable projection of the authoritative world session/binding truth established at start.
4. Later world-member launch logic already contains fail-closed checks that require authoritative parent world binding truth and reject missing or mismatched binding before member launch.
5. The public status/control surfaces already distinguish readable degradation from fail-closed control boundaries: `agent status` may stay readable with warnings, while toolbox and doctor surfaces fail closed when authoritative parent or world-boundary proof is unavailable.
6. Packet 4 should therefore treat items 1-5 as landed floor and freeze only the remaining operator-facing truth, control-surface hardening, docs alignment, and validation wall.

## Objective

Build a public `substrate agent start` surface that launches a host-rooted durable orchestration session first, persists authoritative host attach truth at session birth, and treats world scope as the default execution substrate the host agent later uses for dispatched world work.

Primary operator story:

1. The operator may omit `--scope`, or select `--scope host` / `--scope world` explicitly.
2. Omitting `--scope` resolves a preferred execution scope through workspace-local config/profile/policy first, then global config/policy, and falls back once to the alternate scope only if the preferred scope has no exact backend match.
3. `--scope host` explicitly bypasses world.
4. `--scope world` explicitly requests the default world-backed path while still starting a host-rooted attached orchestration session.
5. The inaugural operator prompt is fulfilled by the host orchestration agent; later world work is dispatched by that host agent under the established world session/binding.

## Landed Packet 2 Floor

### Omitted `--scope` Resolution

1. Omitted `--scope` first resolves the preferred default scope from workspace-local config/profile/policy, then global config/policy.
2. The runtime then probes for an exact backend match in that preferred scope.
3. If the preferred scope has no exact backend match, the runtime probes the alternate scope exactly once.
4. If the alternate scope matches, that resolved scope becomes the authoritative scope for the request, the persisted launch truth, and operator-visible `scope` output.
5. If neither scope has an exact backend match, `start` fails closed with `unknown_backend`.
6. This alternate-scope fallback is intended slice-30 product behavior for Packet 2; changing it later requires an explicit contract update rather than a silent runtime tweak.

### What “World-Backed Host Session” Means At `agent start` Time

For `--scope world`, or omitted `--scope` that resolves to world, a successful `substrate agent start` must mean all of the following before the command returns:

1. A durable host-rooted orchestration session already exists.
2. That session is already in the normal host-attached lifecycle, not `born_unattached`.
3. Authoritative `HostAttachContract` truth is persisted at birth.
4. Authoritative world session/binding truth needed for later host-dispatched world work is already persisted.
5. The host session is the primary operator-facing thing that was launched; world is the execution substrate behind it.

### Immediate Truth Versus Lazy Truth

Truth that must exist immediately at successful start:

1. The orchestration session record.
2. An attached host owner/participant suitable for the normal public host lifecycle.
3. Persisted host attach truth.
4. Persisted authoritative world session/binding truth, including the durable world identity needed for later dispatch.

Truth that may remain lazy in this thin slice:

1. The first host-decided world dispatch after the inaugural prompt.
2. Any later world worker/member conversation created because the host chooses world work.
3. Any broader automatic dispatch or attach policy beyond the root-start path itself.

### Inaugural Prompt Routing

1. The inaugural prompt is strictly host-routed in slice 30.
2. Public `agent start` must not submit the inaugural operator prompt directly to a first world worker/member.
3. If the host later dispatches work into world, that is subsequent host behavior, not the meaning of public root start.

### Deferred Beyond Packet 2

This spec intentionally leaves the following outside Packet 2 and outside the thin-slice contract:

1. Any explicit world-first or `born_unattached` public start mode.
2. Any manual-attach or automatic-attach bridge work associated with parked `30.25` or later slice 31 follow-ons.
3. Automatic world-dispatch policy, pending-work triggers, or inbox-driven attach behavior.
4. Capability broadening beyond the already-supported narrowing-only family.
5. Non-Linux parity for public world-backed root start.

## Landed Packet 3 Floor

### Canonical World Identity Reuse

1. The `world_id` and `world_generation` persisted by Packet 2 are already the canonical durable projection of the authoritative world session/binding truth for that orchestration session.
2. Packet 3 must treat that persisted parent binding as the single source of truth for later host-decided world work.
3. Later world-member launch must reuse the same authoritative parent binding rather than minting an unrelated or detached world identity.
4. If the authoritative parent binding is missing or does not match the active world session, later world-member launch must fail closed.

### Later World Work Must Stay Lazy

1. Packet 3 must not introduce an eager first world-member conversation at public `start` return.
2. Packet 3 must not introduce automatic world dispatch from pending work, inbox activity, or other background triggers.
3. The first world worker/member conversation may still remain lazy until the host actually chooses world work.

### Lifecycle Guardrails

1. The default public world-backed path remains the normal host-attached lifecycle rather than `born_unattached`.
2. Packet 3 may preserve specialized `born_unattached` status semantics for older or specialized sessions, but it must not reintroduce that posture as the thin-slice happy path.
3. Packet 3 is runtime/readiness floor only; broader operator-facing status hardening remains Packet 4 work.

## Frozen Packet 4 Contract

### Operator-Facing Lifecycle And Status Truth

1. The default public world-backed happy path remains the normal host-attached lifecycle from the first successful `start` return.
2. `agent status` must continue to project that happy path as attached host truth rather than reviving `born_unattached` as the default slice-30 success posture.
3. Existing `active_attached`, `parked_resumable`, and `awaiting_attention` semantics remain valid and must not be repurposed by Packet 4.
4. `born_unattached` may still appear for specialized or legacy sessions that genuinely persist that posture, but Packet 4 must not describe or test it as the default world-backed start path.

### Control-Surface Hardening

1. `agent status` remains a readable projection surface: when authoritative parent/session linkage is incomplete, it may degrade with warnings rather than fail closed.
2. `agent toolbox status` remains a fail-closed control surface for active-session authorization: it must prefer authoritative live parent/session manifests over trace history and may expose `active_world_binding` only when the live parent session carries both `world_id` and `world_generation`.
3. `agent toolbox env` must continue to fail closed when no authoritative live orchestrator session is available, even if historical trace events suggest otherwise.
4. `agent doctor` must continue to fail closed at orchestrator selection, runtime realizability, policy allowlist, and required world-boundary checks instead of implying partial readiness.

### Linux-First And Non-Linux Fail-Closed Expectations

1. Linux remains the only supported public happy path for `agent start --scope world` in slice 30.
2. Non-Linux `--scope world` root start must remain explicit fail-closed behavior with `unsupported_platform_or_posture` guidance rather than a degraded “best effort” mode.
3. Packet 4 must preserve the distinction between supported Linux world-backed start and specialized/legacy postures that may still surface elsewhere in status output.

### Final Validation Wall

1. Slice 30 cannot close honestly until `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`, `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`, and `cargo test --workspace -- --nocapture` are green.
2. Required Linux manual smoke must record the actual commands and operator-visible outcomes for host-scoped start stability, successful world-backed start truth, `agent status`, `agent toolbox status`, `agent toolbox env`, `agent doctor`, omitted-scope world-default routing, and later host-mediated world dispatch.
3. If any required Linux manual smoke step cannot run because of environment or runtime limitations, Packet 4 remains open and the exact command, blocker, and unmet acceptance items must be recorded without relaxing the closeout bar.
4. Required non-Linux manual smoke must still confirm the explicit `unsupported_platform_or_posture` fail-closed posture for public world-backed root start.

## Tech Stack

- Language: Rust 2021, MSRV 1.89+
- CLI parsing: `clap` in [`crates/shell/src/execution/cli.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs)
- Public agent control surface: [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- Shared dispatch contract: [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
- Durable session truth: [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs), [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
- World member follow-up and dispatch plumbing: [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs), [`crates/shell/src/execution/routing/dispatch/`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch)
- Slice planning docs: [`llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md), [`llm-last-mile/PLAN-30.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md), [`llm-last-mile/TASKS-30.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-30.md)

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
  - Helper launch plans, prompt streaming envelopes, authoritative world binding persistence, and Linux world-member dispatch behavior
- `crates/shell/tests/agent_public_control_surface_v1.rs`
  - End-to-end public CLI control-plane regression coverage
- `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - Status / doctor / contract regression coverage
- `llm-last-mile/`
  - Planning and scope documents that become the Packet-4 source of truth in this narrow pass

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
   - Validate omitted-scope preferred-scope resolution plus alternate-scope fallback, host-scoped bypass behavior, host-first world-backed start success, authoritative world session/binding truth at start, and public follow-up behavior.
3. Integration tests in `agent_successor_contract_ahcsitc0.rs`
   - Validate authoritative world identity/status truth for later world work and preserve current parked / awaiting-attention projection contracts.
4. Manual smoke checks
   - Validate the exact operator story for `start`, `status`, `toolbox`, `doctor`, `reattach`, and `turn`.

Coverage expectations:

- Every new public flag must have at least one positive parser/behavior test and one negative fail-closed test.
- Every new public resolution rule must have command-level assertions.
- Existing host lifecycle semantics (`active_attached`, `parked_resumable`, `awaiting_attention`) must keep regression coverage so this slice cannot silently break them.
- World-backed start must prove host-first prompt handling plus authoritative world session/binding setup without depending on a born-unattached default posture.
- `agent status` readable degradation, toolbox fail-closed authorization, and doctor fail-closed readiness checks must all have explicit regression coverage because Packet 4 freezes those surfaces as distinct operator contracts.

## Boundaries

- Always:
  - Reuse the shared dispatch contract from slice 29 instead of adding a CLI-only launch dialect.
  - Treat the Packet-1 omitted-scope fallback behavior as frozen unless the spec is deliberately changed.
  - Keep durable authority host-rooted for all `--scope world` starts.
  - Persist authoritative `HostAttachContract` truth at session birth.
  - Treat the landed Packet-2 host-first world-start success shape as floor rather than reopening it.
  - Treat the inaugural operator prompt as a host-orchestrator concern, even when scope resolves to world.
  - Treat Packet-2 `world_id` and `world_generation` persistence plus landed Packet-3 reuse/fail-closed behavior as the canonical parent binding floor rather than first-time work to be rediscovered.
  - Treat `agent status` as a readable degradation surface, but toolbox and doctor as fail-closed control surfaces at authoritative parent/world-boundary seams.
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
2. Omitting `--scope` resolves the preferred default scope through workspace-local config/profile/policy first, then global config/policy, probes for an exact backend in that preferred scope, and falls back once to the alternate scope only if needed.
3. `substrate agent start --scope host` explicitly bypasses world and preserves host-rooted root-start behavior.
4. The resolved scope from step 2 is stamped into the request and is the authoritative scope reported back to the operator.
5. `substrate agent start --scope world`, or omitted `--scope` that resolves to world, creates a host-rooted durable orchestration session, persists authoritative host attach truth at birth, and establishes authoritative world session/binding truth for later host-dispatched world work before `start` returns.
6. The same successful world-backed start is already truthfully host-attached at return time and does not use a participant-less `born_unattached` success posture.
7. The `world_id` and `world_generation` persisted at start are treated as the canonical durable projection of that authoritative world session/binding truth, and later host-decided world work reuses that same authoritative parent binding with fail-closed mismatch handling.
8. The inaugural operator prompt is handled by the host orchestration agent rather than being sent directly to a first world worker/member.
9. Public capability flags, if present, only affect the already-supported narrowing family:
   `session_resume`, `session_fork`, `session_stop`, `status_snapshot`, and `event_stream`, exposed as `--disable-capability <capability>` with `--disable-cap <capability>` as the alias.
10. Unsupported capability fields such as `session_start`, `llm`, and `mcp_client` remain fail closed.
11. `agent status` remains readable when parent/session linkage is degraded, while `agent toolbox` and `agent doctor` preserve fail-closed control-surface behavior at authoritative parent/world-boundary seams.
12. `agent toolbox status` only surfaces `active_world_binding` when the authoritative live parent session carries both `world_id` and `world_generation`; missing binding proof is non-fatal for status but not a license to infer one.
13. The default public world-backed path uses the normal host-attached lifecycle and does not require `born_unattached` as the operator-facing happy-path posture.
14. Public world-scoped root start is supported only on Linux for this slice; non-Linux platforms fail closed with explicit posture guidance.
15. The llm-last-mile slice docs and validation expectations become sufficient to implement and close Packet 4 without reopening the landed Packet-1 through Packet-3 floor.

## Resolved Decisions

These review decisions are now frozen for this spec:

1. Public capability narrowing uses `--disable-capability <capability>` with `--disable-cap <capability>` as the alias.
2. There is no single-letter short flag for capability narrowing.
3. Public world-scoped root start is Linux-first for this slice.
4. Omitting `--scope` resolves the preferred default scope through workspace-local config/profile/policy first, then global config/policy, and falls back once to the alternate scope if the preferred scope has no exact backend match.
5. The resolved scope after that probe/fallback sequence is the authoritative scope for the request and operator-visible output.
6. `--scope host` is the explicit bypass-world path.
7. `born_unattached` is not the default thin-slice happy-path posture.
8. Packet 2 requires immediate host-session truth plus persisted world binding truth, but it does not require an eager first world-worker conversation at `start` return.
9. Packet 4 preserves readable `agent status` degradation while keeping toolbox and doctor fail closed at authoritative parent/world-boundary seams.

## Open Questions

1. This spec freezes resolved `scope` as operator-visible truth, but it does not require a second public field exposing the preferred-versus-fallback provenance. If Packet 4 needs that extra reporting, it should be called out explicitly during implementation rather than inferred.
2. This spec requires authoritative world binding truth before `start` returns, but it does not freeze one specific internal mechanism for proving world readiness beyond that durable contract.
3. This spec freezes the operator-facing rule that `agent status` may degrade readably while toolbox and doctor fail closed, but it does not require Packet 4 to invent new user-facing labels beyond the current warning/reason strings already exercised by the regression suites.

## Review Gate

Do not advance to `PLAN` until a human has reviewed this spec and either:

1. accepted the assumptions as written, or
2. corrected the assumptions or frozen decisions that should drive the plan.
