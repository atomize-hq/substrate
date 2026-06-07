# Spec: Internal Family-2 Host-Targeted Obligation Envelope And Wrong-Host Fail-Closed Boundary

Source remaining-scope note: [REMAINING-family-2-scope-2026-05-30.md](./REMAINING-family-2-scope-2026-05-30.md)  
Prior slice:
- [SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md)
- [PLAN-48.md](./PLAN-48.md)
- [TASKS-48.md](./TASKS-48.md)
Related design stack:
- [DESIGN-durable-orchestration-obligation-ledger.md](./DESIGN-durable-orchestration-obligation-ledger.md)
- [DESIGN-auto-attach-trigger-and-work-queue-contract.md](./DESIGN-auto-attach-trigger-and-work-queue-contract.md)
- [DESIGN-router-daemon-attach-trigger-integration.md](./DESIGN-router-daemon-attach-trigger-integration.md)
- [DESIGN-durable-orchestration-notification-inbox-contract.md](./DESIGN-durable-orchestration-notification-inbox-contract.md)
- [DESIGN-host-to-world-steering-policy-matrix.md](./DESIGN-host-to-world-steering-policy-matrix.md)
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)  
Phase: `SPECIFY`  
Status: implemented and validation-aligned on `2026-06-07`

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `48` closed the local router-owned execution boundary, so the next honest Family-2 seam is data-model and boundary widening rather than more router-loop productization.
2. This slice should land only the minimum obligation-envelope widening needed for host targeting and wrong-host fail-closed behavior:
   - `origin_host_id`
   - `target_host_id`
3. This slice should not yet land the broader ingress-ready envelope:
   - no `SUBSTRATE_HOME/host_inbox/`
   - no remote ingress materialization
   - no `ingress_source_kind`
   - no `ingress_source_id`
   - no `ingress_received_at`
   - no broader `causation_*` widening beyond already-landed local truth
4. The local router must remain the consumer of local obligations only. If a locally persisted obligation carries an explicit foreign `target_host_id`, that is currently an invalid local-materialization condition, not a signal to attempt cross-host delivery.
5. The repo does not yet expose a dedicated authoritative host-identity contract in the Family-2 runtime surface. This slice may introduce or freeze one bounded internal local-host identity helper without widening into public config or federation design.
6. Existing local obligations produced on this host may safely record local host-targeting truth when it is known, as long as the ledger remains local-session authoritative and review state stays separate from attach state.
7. Wrong-host fail-closed behavior for this slice should affect attach-processing and router claim behavior only. It must not silently resolve review state or delete obligations.

If any of these are wrong, correct them before implementation.

## Objective

Land the first post-`48` Family-2 envelope slice by widening the local orchestration obligation record and router evaluation rules just enough for exact host targeting and wrong-host fail-closed behavior, without widening into host-global inbox materialization, remote federation, or broader workflow-engine work.

Primary runtime story:

1. a local unresolved obligation is created with exact local session truth and bounded host-targeting metadata,
2. the local router evaluates that obligation using local session truth plus exact local host identity,
3. if the obligation is untargeted or explicitly targeted at this host, normal local attach eligibility continues,
4. if the obligation is explicitly targeted at a different host, the local router does not launch attach work and records explanation-ready fail-closed posture,
5. review state remains unresolved until explicit host action,
6. no new host-global ingress or remote delivery machinery lands in this slice.

## Observed Repo Floor

The current repo already has most of the local Family-2 floor that this slice should reuse:

1. the canonical local obligation artifact exists as [`OrchestrationObligationRecord`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs),
2. local obligation persistence and round-trip validation already live in [`state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs),
3. retained-worker and control-plane producers already persist obligation records through [`orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs),
4. router-owned attach execution already consumes local obligations through [`auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs),
5. Slice `48` already closed the local router-owned execution boundary, deny-by-default policy gate, and manual-reattach convergence.

What this slice now lands in the live tree is the bounded host-targeting envelope itself:

1. the obligation record now carries `origin_host_id` and `target_host_id`,
2. the router path now has an exact local-host boundary check for obligation targeting,
3. the runtime now treats an explicit foreign-targeted local obligation as an explanation-ready fail-closed local-materialization condition until a future `host_inbox` or remote ingress layer exists.

That keeps Slice `49` honest: the local-only boundary is widened just enough for exact host targeting, but no broader ingress or federation machinery is implied.

## Tech Stack

- Language: Rust `2021`, MSRV `1.89+`
- Expected runtime files:
  - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
  - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
  - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
  - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
- Possible bounded helper seam for local host identity:
  - one new or existing internal file under `crates/shell/src/execution/` or `crates/common/`
- Expected docs:
  - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
  - [`docs/TRACE.md`](../docs/TRACE.md) only if a real wrong-host derived trace shape lands

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

Targeted validation floor:

```bash
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell obligation -- --nocapture
```

Targeted router/dispatch validation:

```bash
cargo test -p shell orchestrator_world_dispatch -- --nocapture
```

Full validation wall:

```bash
cargo test --workspace -- --nocapture
```

## Project Structure

The current repo structure relevant to this slice is:

- `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
  - canonical obligation schema and persistence-boundary validation
  - Slice `49` should widen the record only for bounded host-targeting truth, not the full ingress envelope
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - durable obligation persistence, round-trips, and attach-state settlement
  - Slice `49` should keep the authoritative local write/read boundary here
- `crates/shell/src/execution/agent_runtime/auto_attach.rs`
  - router-owned attach eligibility, claim, and fail-closed execution
  - Slice `49` should add exact wrong-host evaluation here or in a narrow helper it calls
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - current retained-worker and control-plane obligation producers
  - Slice `49` should update producer paths so local obligations can carry bounded host-targeting truth when available
- `docs/`
  - docs must stay honest that this slice lands local host targeting and wrong-host fail-closed behavior only, not host-global ingress
- `llm-last-mile/`
  - repo-local planning authority for the Family-2 follow-on slices

## Code Style

Follow the existing runtime style: exact identity, fail-closed boundary checks, explanation-ready denial reasons, and no silent widening into product surfaces that are not landed.

Preferred style:

```rust
if let Some(target_host_id) = obligation.target_host_id.as_deref() {
    if target_host_id != local_host_id {
        anyhow::bail!(
            "wrong_target_host: obligation {} targets host {} not local host {}",
            obligation.obligation_id,
            target_host_id,
            local_host_id,
        );
    }
}
```

Conventions:

1. use exact local identity rather than fuzzy routing,
2. keep wrong-host posture explanation-ready,
3. keep review-state and attach-state semantics separate,
4. do not add new public config or CLI surfaces for host targeting in this slice,
5. keep ingress-ready fields and host-global inbox machinery deferred,
6. keep no-host-target legacy/local obligations compatible when `target_host_id` is absent.

## Testing Strategy

Frameworks:

- Rust unit tests
- Rust integration tests
- shell runtime regression suites

Test levels for this slice:

1. obligation-schema and persistence tests:
   - `origin_host_id` and `target_host_id` round-trip when present,
   - missing host-targeting fields remain backward compatible,
   - invalid empty/whitespace host ids fail at the persistence boundary if the slice chooses to validate them strictly
2. local producer tests:
   - local obligation creation preserves exact session/backend/world truth and adds bounded local host-targeting metadata when expected
3. router wrong-host tests:
   - obligations targeted at the local host stay eligible,
   - obligations with no `target_host_id` preserve current local-only behavior,
   - obligations targeted at a different host fail closed without launching attach
4. settlement and observability tests:
   - wrong-host fail-closed posture records explanation-ready reason,
   - wrong-host handling does not resolve review state or fabricate continuation

## Boundaries

- Always:
  - keep the obligation ledger as the canonical local durable truth
  - keep router evaluation local-only and exact-host scoped
  - preserve backward compatibility for obligations with no host-targeting metadata
  - keep wrong-host outcomes fail-closed and explanation-ready
- Ask first:
  - adding a new public policy key, CLI surface, or host-global state path
  - widening into ingress metadata or remote federation mechanics
  - introducing a cross-repo or cross-process host identity contract outside the bounded Family-2 runtime seam
- Never:
  - add `SUBSTRATE_HOME/host_inbox/` in this slice
  - let the router consume remote/global ingress records directly
  - silently reroute a foreign-targeted obligation onto the local host
  - resolve review state just because host targeting fails

## Success Criteria

This slice is complete only when all of the following are true:

1. the local obligation record can carry bounded host-targeting truth needed for Family-2 local evaluation,
2. local producers and persistence round-trips preserve that truth without reopening the broader ingress envelope,
3. router-owned auto-attach evaluates `target_host_id` against exact local host identity before attach launch,
4. explicit wrong-host obligations fail closed with explanation-ready reasons and no attach launch,
5. untargeted or same-host obligations preserve the landed Slice `48` behavior,
6. no host-global inbox, remote ingress materialization, or broader federation workflow lands.

## Open Questions

1. Should the bounded local host identity for this slice be a dedicated persisted machine identifier or a narrower internal helper over already-available host-visible identity? Default assumption for this spec: either is acceptable if it is internal-only, stable enough for same-host checks, and does not widen into public config or federation design.
2. When an explicit foreign-targeted obligation is present locally before `host_inbox` exists, should the router leave it untouched or settle it failed closed? Default assumption for this spec: fail closed with an explanation-ready reason because local materialization of a foreign-targeted obligation is invalid in the current architecture.
