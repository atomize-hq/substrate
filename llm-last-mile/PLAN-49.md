# PLAN-49: Internal Family-2 Host-Targeted Obligation Envelope And Wrong-Host Fail-Closed Boundary

Source spec: [SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md](./SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md)  
Source prior slice: [PLAN-48.md](./PLAN-48.md)  
Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Plan type: first post-`48` Family-2 envelope slice  
Status: implemented and validation-aligned on `2026-06-07`

## Objective

Land the first post-`48` Family-2 envelope slice by widening the local obligation ledger and router boundary only enough to carry host-targeting truth and fail closed when an obligation is targeted at the wrong host, without widening into host-global inbox, remote ingress materialization, or broader federation machinery.

This slice is complete only when all of the following are true:

1. the obligation record carries the bounded host-targeting fields needed for local Family-2 evaluation,
2. local obligation producers and persistence round-trips preserve those fields when known,
3. router-owned attach evaluation checks explicit host targeting before claim or launch,
4. explicit wrong-host obligations fail closed with explanation-ready reasons and no attach launch,
5. same-host and untargeted obligations preserve the landed Slice `48` behavior,
6. ingress metadata, `host_inbox`, and remote delivery remain deferred.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md](./SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md) has been reviewed and is the source of truth for scope before implementation planning advances.

## Major Components And Dependencies

1. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
   - owns the canonical obligation schema and persistence-boundary validation
   - must widen only for bounded host-targeting truth in this slice
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - owns durable obligation persistence and round-trips
   - must remain the authoritative write/read boundary for the widened record
3. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - owns live obligation production from retained-worker/control-plane events
   - should propagate bounded local host-targeting truth when obligations are created locally
4. `crates/shell/src/execution/agent_runtime/auto_attach.rs`
   - owns router-owned attach evaluation, claim, and fail-closed execution
   - must gain the exact local-host targeting check
5. one bounded internal host-identity helper under `crates/shell/src/execution/` or `crates/common/`
   - should provide one explanation-ready local host identity seam for this slice
   - must stay internal-only and not widen into public config or federation contracts
6. `docs/CONFIGURATION.md` and `docs/TRACE.md` if needed
   - must stay honest about what this slice does and does not land

## Plan Summary

After Slice `48`, the next remaining Family-2 seam was narrower:

1. the obligation ledger needed bounded host-targeting truth,
2. the router needed an exact local-host wrong-target check,
3. the current architecture needed an explicit answer for what to do when a foreign-targeted obligation appears locally before `host_inbox` exists.

The design stack is already clear enough to freeze that seam now:

1. the obligation ledger should preserve room for `origin_host_id` and `target_host_id`,
2. the router must act on local obligations only,
3. explicit wrong-host obligations must fail closed rather than silently reroute,
4. ingress metadata and host-global inbox layering remain later work.

The landed Slice `49` therefore:

1. freeze the bounded host-targeting envelope first,
2. widen local persistence and producers second,
3. add wrong-host router evaluation and fail-closed settlement third,
4. finished with docs and validation fourth.

## Locked Decisions

### What changes

1. Slice `49` widens the local obligation envelope for bounded host targeting.
2. Slice `49` introduces an internal local-host identity seam for exact same-host versus foreign-host evaluation.
3. Slice `49` makes wrong-host obligations fail closed in the local router-owned attach path.
4. Slice `49` keeps review-state semantics unchanged while widening attach-boundary evaluation.

### What does not change

1. no `SUBSTRATE_HOME/host_inbox/`,
2. no remote ingress materialization or remote federation protocol,
3. no new public CLI or policy surface for host targeting,
4. no broader ingress metadata widening (`ingress_source_*` remains deferred),
5. no broader `causation_*` envelope project,
6. no workflow-engine or router lifecycle productization work.

## Implementation Order

### Packet 1: Host-Targeting Envelope Contract Freeze

Goal:

1. freeze the exact bounded schema widening for host targeting,
2. keep ingress metadata and broader federation envelope out of scope,
3. define the wrong-host fail-closed semantics for the current local-only architecture.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. targeted obligation round-trip tests

Why first:

1. producer and router logic need one frozen record contract,
2. this slice must not accidentally balloon into the full ingress-ready identity envelope,
3. wrong-host behavior should consume one explicit contract rather than ad hoc optional fields.

Verification checkpoint:

1. only the bounded host-targeting fields are added,
2. persistence remains backward compatible for obligations with no host-targeting metadata,
3. wrong-host semantics are explicit in tests and/or spec-aligned helper behavior.

### Packet 2: Local Producer And Persistence Widening

Goal:

1. propagate bounded local host-targeting truth into locally created obligations where appropriate,
2. keep the state-store round-trip authoritative,
3. avoid reopening producer semantics beyond the bounded envelope.

Primary touch surface:

1. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
4. targeted producer and persistence tests

Why second:

1. router evaluation is only useful if local obligations can actually carry the widened truth,
2. producer widening is still local-only and does not require the router to change yet,
3. this keeps the later wrong-host attach behavior grounded in real persisted data rather than test-only synthetic records.

Verification checkpoint:

1. local obligations round-trip with bounded host-targeting truth when known,
2. current local-only paths without host-targeting metadata keep working,
3. no ingress metadata or host-global state path is introduced.

### Packet 3: Router Wrong-Host Fail-Closed Evaluation

Goal:

1. evaluate `target_host_id` against exact local host identity in the router-owned attach path,
2. keep same-host and untargeted obligations compatible with Slice `48`,
3. make foreign-targeted local obligations fail closed without attach launch.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/auto_attach.rs`
2. host-side router wiring under `crates/shell/src/execution/`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. targeted router/attach tests

Why third:

1. the router should consume the widened record after the persistence contract is frozen,
2. wrong-host fail-closed behavior is the main semantic risk in this slice,
3. attach behavior should stay bounded and explanation-ready before docs are updated.

Verification checkpoint:

1. foreign-targeted obligations do not launch attach,
2. wrong-host reasons are explanation-ready,
3. review state is unchanged by wrong-host fail-closed outcomes,
4. untargeted or same-host obligations preserve Slice `48` behavior.

### Packet 4: Docs Alignment And Validation

Goal:

1. align docs with the landed bounded host-targeting behavior,
2. keep ingress and host-global inbox work explicitly deferred,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `docs/TRACE.md` if touched
3. `llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md`
4. `llm-last-mile/PLAN-49.md`
5. `llm-last-mile/TASKS-49.md`

What this packet must enforce:

1. docs describe Slice `49` as bounded host-targeting envelope and wrong-host fail-closed work only,
2. docs do not imply `host_inbox`, remote ingress, or cross-host delivery have landed,
3. docs preserve the follow-on seam for ingress-ready identity fields and host-global inbox materialization.

Verification checkpoint:

1. docs and runtime truth are honest,
2. validation is green,
3. the next Family-2 follow-on can stay focused on ingress-ready identity fields and `host_inbox` layering rather than reopening wrong-host posture.

## Risks And Mitigations

1. Risk: Slice `49` balloons into the full ingress-ready identity envelope.
   Mitigation: widen only `origin_host_id` and `target_host_id`; keep `ingress_source_*` and broader `causation_*` work explicitly deferred.
2. Risk: local host identity becomes an accidental public contract.
   Mitigation: keep the host-identity seam internal-only and explanation-ready, not user-configurable in this slice.
3. Risk: wrong-host handling silently reroutes or drops obligations.
   Mitigation: require fail-closed outcomes with explicit reasons and no review-state mutation.
4. Risk: backward compatibility breaks for already-persisted local obligations with no host-targeting metadata.
   Mitigation: preserve compatibility for missing host-targeting fields and treat them as current local-only behavior.
5. Risk: producer widening leaks into host-global ingress architecture.
   Mitigation: keep all writes local-session scoped and forbid `host_inbox` or remote materialization work in this slice.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because producers need one frozen bounded envelope.
2. Packet 2 before Packet 3 because router behavior should consume real persisted host-targeting truth.
3. Packet 3 before Packet 4 because docs should describe the landed wrong-host posture honestly.

### Can be deferred

1. `ingress_source_kind`, `ingress_source_id`, and `ingress_received_at`,
2. broader `causation_event_id`, `causation_message_id`, and `causation_request_id` widening,
3. `SUBSTRATE_HOME/host_inbox/`,
4. remote federation, sync, lease, or delivery protocols,
5. public router lifecycle or workflow-engine productization.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell obligation -- --nocapture
cargo test -p shell orchestrator_world_dispatch -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `49`

If Slice `49` lands cleanly as the bounded host-targeting and wrong-host fail-closed slice, the next Family-2 follow-on should remain:

1. ingress-ready identity fields needed for future `host_inbox` or remote federation materialization,
2. host-global inbox layering under `SUBSTRATE_HOME/host_inbox/`,
3. only after those, any broader cross-host delivery or router lifecycle productization.

The Slice `49` wrong-host fail-closed posture is transitional, not permanent architecture: it should remain only until a later Family-2 slice lands a valid ingress/materialization path that makes foreign-targeted local obligations legitimate delivery artifacts instead of invalid local-only materializations.
