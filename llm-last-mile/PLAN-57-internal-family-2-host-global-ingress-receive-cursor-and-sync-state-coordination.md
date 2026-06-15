# PLAN-57: Internal Family-2 Host-Global Ingress Receive-Cursor And Sync-State Coordination

Source spec: [SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md](./SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md)  
Source tracker note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Prior Family-2 slice: [PLAN-51.md](./PLAN-51.md)  
Plan type: first post-`51` host-global ingress lifecycle-coordination slice  
Phase: `PLAN`  
Status: proposed on `2026-06-12`

## Objective

Land the next bounded Family-2 seam by introducing source-scoped host-global ingress coordination above the already-landed `host_inbox` layer, so upstream ingress can be durably applied exactly once into canonical local host-inbox state before the existing materialization and router passes consume it.

This slice is complete only when:

1. the repo has one exact durable coordination model for source-scoped ingress receive-cursor or sync-state truth,
2. ingress application durably persists canonical host-inbox records before advancing coordination state,
3. duplicate replay is idempotent rather than duplicative,
4. invalid or non-monotonic replay fails closed,
5. local materialization and router-owned auto-attach keep consuming only their already-landed lower layers,
6. the slice lands without broader federation, lease/lock coordination, or public operator UX.

## Phase Gate

This plan assumes the `SPECIFY` artifact in [SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md](./SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md) has been reviewed and accepted before implementation starts.

## Tracker Update Rule

The sequencing source remains:

- [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)

The broader product-truth tracker that should stay aligned with that note is:

- [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)

During execution:

1. record new findings in the remaining-scope note only if they materially change the recommendation that host-global ingress lifecycle coordination is the next honest Family-2 seam,
2. keep the gap matrix aligned that this slice is receive-cursor/sync-state coordination only, not cross-host delivery or public inbox UX,
3. do not widen this slice into Windows parity, later toolbox follow-through, or governance cleanup just because those remain open elsewhere.

## Repo-Truth Framing

What is already landed:

1. bounded host-global inbox artifacts under `SUBSTRATE_HOME/host_inbox/`,
2. exact local materialization from host-inbox records into canonical obligations,
3. router-owned auto-attach consuming local obligations only,
4. fail-closed wrong-host and malformed-artifact behavior at the local materialization boundary.

What remains as the next narrow Family-2 seam:

1. freeze one source-scoped coordination artifact above canonical host-inbox records,
2. make ingress receipt and cursor advancement exact and idempotent,
3. keep later cross-host delivery/federation work deferred,
4. keep materialization and router sequencing layered exactly as already designed.

## Locked Decisions

### What this slice changes

1. It freezes source-scoped ingress receive-cursor or sync-state truth.
2. It defines when cursor advancement is allowed locally.
3. It adds an internal ingress apply boundary that durably writes host-inbox records before source advancement.
4. It proves replay/idempotence at the host-global ingress layer.

### What this slice does not change

1. no cross-host delivery engine,
2. no distributed lease/lock coordination,
3. no public host-inbox review or management UX,
4. no redesign of host-inbox materialization,
5. no router-owned attach redesign,
6. no selector, toolbox, or platform-parity work.

## Major Components And Dependencies

1. **host-global ingress artifact and coordination types**
   - `crates/shell/src/execution/agent_runtime/host_inbox.rs`
   - one new bounded coordination module under `crates/shell/src/execution/agent_runtime/`
   - own source key, receive-cursor or sync-state shape, and exact validation rules

2. **authoritative persistence and idempotence**
   - `crates/shell/src/execution/agent_runtime/state_store.rs`
   - owns coordination artifact paths, read/write helpers, duplicate detection, and advancement ordering

3. **internal ingress apply boundary**
   - one bounded host-side helper under `crates/shell/src/execution/`
   - or store-owned helpers only, if that is enough
   - sequences “persist host-inbox record, then advance source coordination state”

4. **downstream coexistence proof**
   - `crates/shell/src/execution/host_inbox_materialization.rs`
   - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - prove the already-landed local materialization/router path remains unchanged in responsibility

5. **regression floor**
   - unit tests in `host_inbox.rs`
   - unit tests in `state_store.rs`
   - targeted execution tests if an ingress apply helper is introduced

## Plan Summary

The narrowest honest Slice `57` is:

1. freeze the source-scoped coordination artifact and advancement contract first,
2. persist and validate coordination state second,
3. add the bounded ingress apply sequencing and idempotence proof third,
4. finish with coexistence validation and repo-truth closeout last.

## Implementation Order

### Packet 1: Freeze The Coordination Contract

Goal:

1. define one source-scoped receive-cursor or sync-state artifact,
2. freeze the advancement rule that durable host-inbox persistence must happen first,
3. make explicit that the host-global coordination layer does not replace canonical host-inbox truth.

Primary touch surface:

1. `llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md`
2. `llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md`
3. `llm-last-mile/TASKS-57.md`
4. bounded runtime comments/docs only if the current tree contradicts the frozen boundary

Why first:

1. later packets need one exact answer for what counts as source identity and cursor advancement,
2. it prevents “sync-state” from drifting into an implicit delivery engine,
3. it keeps the slice above the landed `host_inbox` layer rather than silently editing that contract.

Verification checkpoint:

1. the slice has one explicit source-key rule,
2. the slice has one explicit cursor-advancement rule,
3. the slice has one explicit layering rule separating coordination state from canonical host-inbox records.

### Packet 2: Persist Source Coordination State

Goal:

1. add store-owned durable coordination artifacts and helpers,
2. validate source key and cursor truth,
3. keep malformed or impossible state fail-closed.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/host_inbox.rs`
2. one new bounded coordination module under `crates/shell/src/execution/agent_runtime/`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`

Why second:

1. Packet `1` defines the artifact contract,
2. this is the first runtime packet that needs durable state truth,
3. later ingress application should depend on store-owned helpers rather than inventing its own writes.

Verification checkpoint:

1. coordination artifacts round-trip durably,
2. invalid cursor/source state fails validation,
3. no host-inbox materialization logic is widened just to persist coordination state.

### Packet 3: Add Exact Ingress Apply Sequencing And Replay Truth

Goal:

1. add one bounded internal ingress apply path,
2. persist canonical host-inbox records before advancing source coordination state,
3. make same-cursor replay idempotent and mismatched replay fail closed.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/state_store.rs`
2. one bounded host-side ingress helper under `crates/shell/src/execution/`
3. `crates/shell/src/execution/agent_runtime/host_inbox.rs`

Why third:

1. Packet `2` first gives the slice durable coordination truth,
2. this is where the slice actually proves “exact local receipt before advance,”
3. it is the highest-value runtime behavior in the slice and should stay narrowly scoped to ingress application.

Verification checkpoint:

1. applying a new cursor persists exactly one host-inbox record and advances once,
2. replaying the same cursor/record is idempotent,
3. non-monotonic or mismatched replay fails closed without advancing.

### Packet 4: Coexistence Proof, Docs Alignment, And Validation Wall

Goal:

1. prove the landed `host_inbox -> local obligation -> router` layering still holds,
2. align docs/tracker truth with the bounded coordination slice,
3. run the validation wall.

Primary touch surface:

1. `crates/shell/src/execution/host_inbox_materialization.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs` only if bounded coexistence assertions require it
3. Slice `57` spec/plan/tasks docs
4. `AGENT_ORCHESTRATION_GAP_MATRIX.md` only if bounded wording alignment is needed

Why last:

1. repo truth should describe what actually landed,
2. coexistence is easier to verify once ingress sequencing is real,
3. this packet is where the slice proves it did not widen into delivery/federation/productization.

Verification checkpoint:

1. downstream materialization still consumes host-inbox records only,
2. router-owned auto-attach still consumes local obligations only,
3. docs say clearly that lease/lock coordination and broader federation remain later seams.

## Risks And Mitigations

1. **Risk: source coordination state becomes a second owner of host-global record truth**
   - Mitigation: keep source coordination state as source progress only; canonical host-inbox records remain the receipt artifact.

2. **Risk: cursor advancement happens before durable local receipt**
   - Mitigation: centralize the apply sequence in store-owned helpers and regression-test ordering.

3. **Risk: replay handling widens into ad hoc dedup heuristics**
   - Mitigation: freeze exact repeat-vs-mismatch rules and fail closed on ambiguous replay.

4. **Risk: the slice drifts into distributed lease/lock semantics**
   - Mitigation: keep lease/lock coordination explicitly deferred and stop at single-host source coordination.

5. **Risk: downstream materialization/router behavior gets redesigned unnecessarily**
   - Mitigation: treat `host_inbox_materialization.rs` and router attach as consumers to prove unchanged boundaries, not as redesign surfaces.

## Parallelism Guidance

1. Packet `1` must happen first because it freezes the coordination vocabulary and advancement rules.
2. After Packet `1`, Packet `2` and Packet `3` can be explored in parallel, but final integration should stay centralized because both touch `state_store.rs` and the host-inbox contract.
3. Packet `4` must happen last.

## Success Markers

The slice is green only when:

1. source-scoped receive-cursor or sync-state truth is durable and exact,
2. local host-inbox receipt precedes cursor advancement,
3. replay is idempotent and explanation-ready,
4. later delivery/federation work remains clearly deferred.
