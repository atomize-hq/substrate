# PLAN-31: Lazy Host Attach For Host-Rooted World Start

Source SOW: [31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Source spec: [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)  
Source lifecycle design: [DESIGN-world-worker-lifecycle-model.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md)  
Adjacent landed slices: [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md), [30-public-world-scoped-agent-start-and-capability-flags.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)  
Proposed branch: `feat/lazy-host-attach-host-rooted-world-start`  
Base branch: `main`  
Plan type: specialized detached-host orchestration flow with durable obligation-ledger and router-owned attach recovery  
Status: draft for review on 2026-05-29

## Objective

Ship the specialized slice where a host-rooted orchestration session may be born without an attached host execution client, while preserving truthful durable authority and restoring sanctioned host ownership through obligation-driven attach.

This slice is complete only when all of the following are true:

1. Slice 30's normal world-backed public start remains the host-attached happy path and is not reopened.
2. Specialized born-unattached or later-detached host-rooted sessions still persist authoritative attach truth through the existing `HostAttachContract`.
3. Detached host-side work persists canonically as obligations rather than as synthetic prompts or a second canonical queue.
4. Operator-visible posture distinguishes never-attached-yet, detached-resumable, and detached-awaiting-attention states.
5. Router-owned automatic attach evaluates eligible unresolved obligations and coalesces per orchestration session.
6. Attach execution is continuity-first, fresh-fallback, and bounded by the persisted attach contract.
7. Manual `reattach` and automatic attach share one coherent authority model.
8. Detached world follow-up stays fail-closed until sanctioned host ownership returns.

## Plan Summary

The repo already contains most of the authority and persistence floor needed for this slice:

1. public `reattach`, `turn`, `stop`, and `status` surfaces already exist in [`agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs),
2. persisted host attach truth already exists in [`HostAttachContract`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:79),
3. detached posture normalization and public status projection already exist in [`orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs) and [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs),
4. current durable inbox persistence already provides a compatibility persistence seam in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:383),
5. world-member identity, lineage, and invalidation are already explicit in [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs),
6. public control tests already pin no-hidden-prompt reattach behavior, detached-world fail-closed behavior, and readable-versus-fail-closed operator surfaces in the two shell suites.

The exact remaining gap is architectural rather than parser-oriented:

1. the repo still treats detached attention and inbox semantics as the main durable pending-work surface rather than an obligation-ledger source of truth,
2. automatic attach from durable unresolved work is specified but not implemented,
3. the detached host-session taxonomy still needs to distinguish never-attached-yet from later-detached states without collapsing them into one bucket,
4. the runtime still needs a concrete continuity-first versus fresh-fallback attach executor that interoperates with manual `reattach`,
5. the lifecycle design needs to be reflected in runtime state transitions so worker attention and host posture remain separate.

## Locked Starting State

### What already exists

| Sub-problem | Existing code | Decision |
| --- | --- | --- |
| Persisted attach baseline | [`HostAttachContract`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:79) | Reuse exactly. Slice 31 may not invent a second durable attach-truth shape. |
| Persisted attach resolution | [`resolve_persisted_host_attach_contract(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs:316) | Reuse exactly. Automatic attach must resolve through the same narrowing-only path as manual `reattach`. |
| Detached session truth | [`OrchestrationSessionPosture`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs:68) | Extend truthfully. Do not collapse detached states or repurpose slice-30 happy-path semantics. |
| Durable inbox persistence | [`persist_inbox_item(...)`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:1209) | Treat as compatibility surface. Do not let it remain the final canonical pending-work ledger. |
| Detached world fail-closed routing | checks in [`state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:945) | Preserve. Specialized auto-attach must not reopen world follow-up before host ownership returns. |
| Public reattach contract | tests in [`agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs) | Preserve. No synthetic bootstrap prompt, same durable session, exact lineage. |
| Worker lineage and invalidation | [`session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs) | Reuse exactly. Worker fork and invalidation must stay explicit and fail-closed. |

### Scope decision

Proceed as four bounded implementation packets:

1. detached taxonomy and obligation-ledger scaffolding,
2. obligation persistence plus status/review projection,
3. router-owned auto-attach trigger and coalescing,
4. attach execution, manual/automatic interop, fail-closed follow-up, docs, and validation.

Do not reopen:

1. slice 30's host-attached public world-start happy path,
2. public capability or scope caller-surface expansion,
3. general world-worker introspection CLI design,
4. non-Linux parity.

## Frozen Execution Contract

If implementation needs to deviate, update the spec and this plan first.

### Authority contract

1. The orchestration session remains the durable authority root whether or not a host execution client is attached.
2. Persisted `HostAttachContract` truth is written at birth and copied forward through sanctioned successor flows only.
3. Later attach attempts may narrow or honor persisted truth, but may not silently broaden it or re-derive it from live snapshots.

### Detached taxonomy contract

The runtime must keep these detached states operationally distinct:

1. never attached yet,
2. previously attached and resumable,
3. detached with unresolved attention-driving obligations.

The implementation may rename item 1, but it may not merge the three states into one detached bucket.

### Obligation contract

1. Canonical detached pending-work truth is the obligation ledger.
2. Review/inbox state is a projection over obligations.
3. Automatic attach eligibility is a projection over obligations.
4. Attention-driving obligation classes must include, at minimum:
   - `follow_up_required`
   - `approval_required`
   - `blocked`
   - `fork_request`

### Attach-trigger contract

1. Router-owned evaluation inspects unresolved eligible obligations rather than raw trace events or synthetic prompt reconstruction.
2. Automatic attach is session-scoped and must coalesce to at most one claimed attach episode per orchestration session at a time.
3. Automatic attach may be satisfied or superseded by manual `reattach`, but the durable authority model remains one-session, one-owner.

### Attach execution contract

1. Valid continuity selector present:
   - prefer continuity attach
2. Valid continuity selector absent, but attach truth still valid:
   - permit fresh attach
3. Persisted attach truth missing or invalid:
   - fail closed before backend launch

### Follow-up contract

1. World follow-up remains fail-closed while the orchestration session is born-unattached or otherwise detached.
2. Operator-visible errors must continue to point callers back to sanctioned attach recovery.
3. No world-to-world continuation or hidden host bootstrap prompt may bypass attach recovery.

### Lifecycle contract

1. Worker lifecycle state remains separate from host-session posture.
2. `attention_pending` belongs to retained workers.
3. `awaiting_attention` belongs to host-session posture.
4. Ephemeral work that needs later follow-up must escalate explicitly rather than silently entering retained behavior.

## Implementation Order

### Packet 1: Freeze detached taxonomy and add obligation-ledger scaffolding

Goal:

1. formalize the specialized detached host-session taxonomy,
2. add canonical obligation-ledger record shapes and validators,
3. keep current inbox artifacts as compatibility projection only.

Why first:

1. automatic attach and review projection both depend on a canonical pending-work record,
2. status semantics cannot be made truthful until the detached taxonomy is explicit,
3. this isolates the data-model and state-transition risk before background behavior is introduced.

Primary touch surface:

1. `orchestration_session.rs`
2. `state_store.rs`
3. new `obligation_ledger.rs`
4. `agent_successor_contract_ahcsitc0.rs`

Verification checkpoint:

1. posture invariants pin never-attached-yet versus parked versus awaiting-attention,
2. obligation records validate required fields and terminal resolution rules,
3. existing slice-30 attached-path tests stay green.

### Packet 2: Persist obligations from detached host-side work and derive review/status projection

Goal:

1. persist eligible detached host-side work as obligations,
2. derive review/inbox projection from obligation truth,
3. derive host detached posture from unresolved attention-driving obligations.

Why second:

1. router-driven automatic attach should not exist before durable obligation truth exists,
2. operator-visible status/read surfaces need to reflect the new canonical truth before background automation is added.

Primary touch surface:

1. `state_store.rs`
2. `control.rs`
3. `agents_cmd.rs`
4. `agent_public_control_surface_v1.rs`
5. `agent_successor_contract_ahcsitc0.rs`

Verification checkpoint:

1. creating or resolving obligations updates detached posture truth correctly,
2. compatibility inbox/read surfaces remain coherent,
3. detached world follow-up still fails closed.

### Packet 3: Implement router-owned auto-attach trigger and session coalescing

Goal:

1. evaluate unresolved eligible obligations for attach,
2. coalesce by orchestration session,
3. persist attach-episode claim state and outcomes.

Why third:

1. automatic attach depends on canonical obligation truth,
2. it introduces the highest concurrency risk and should be isolated before attach execution changes.

Primary touch surface:

1. new `auto_attach.rs`
2. `state_store.rs`
3. `control.rs`
4. `agent_inventory.rs`
5. tests in `agent_public_control_surface_v1.rs`

Verification checkpoint:

1. multiple eligible obligations in one session produce one attach episode,
2. different sessions may progress independently,
3. manual `reattach` can satisfy an outstanding automatic attach claim without duplicate backend launches.

### Packet 4: Continuity-first/fresh-fallback attach execution, interop, docs, and full validation

Goal:

1. execute continuity-first or fresh-fallback attach through persisted truth,
2. preserve one coherent manual/automatic authority model,
3. keep detached world follow-up fail-closed until ownership is restored,
4. align docs and run the validation wall.

Why last:

1. attach execution depends on both obligation truth and trigger coalescing,
2. it carries the highest end-to-end operator contract risk and should land behind the fullest test wall.

Primary touch surface:

1. `control.rs`
2. `dispatch_contract.rs`
3. `agents_cmd.rs`
4. `agent_public_control_surface_v1.rs`
5. `agent_successor_contract_ahcsitc0.rs`

Verification checkpoint:

1. continuity-first attach is exercised when valid,
2. fresh attach is exercised only when continuity is unavailable but persisted truth still allows it,
3. stale or invalid attach truth fails closed,
4. docs and final validation wall are green.

Implementation status on 2026-05-29:

1. hidden-owner-helper launch now routes through one shared `control.rs` path for manual `reattach` and router-owned automatic attach.
2. router-owned automatic attach resolves persisted attach truth through the same narrowing-only contract resolver used by manual `reattach`, requiring continuity when persisted truth still requires it and otherwise permitting a continuity-preferred fresh fallback.
3. attach startup extension wiring now distinguishes continuity-backed attach from fresh attach preparation so the router path can fail closed before launch or omit resume-only startup state when continuity is absent.
4. the current Codex control wrapper still rejects a prompt-free fresh control attach at the backend boundary, so Packet 4 currently proves fresh-fallback planning/bootstrap seams and fail-closed behavior without claiming a fully verified prompt-free Codex runtime attach.

## Risks And Mitigations

1. Risk: detached taxonomy drifts and reopens slice-30 happy-path semantics.
   Mitigation: pin slice-30 attached-path tests unchanged and add explicit slice-31 specialized-path assertions.
2. Risk: inbox compatibility and obligation truth diverge.
   Mitigation: make one canonical projection path and test both pending and resolved transitions from the same obligation record.
3. Risk: automatic attach duplicates manual `reattach` or launches twice for sibling obligations.
   Mitigation: persist session-scoped attach-episode claim state and assert one-launch semantics in tests.
4. Risk: attach execution silently broadens persisted truth.
   Mitigation: route every attach through `resolve_persisted_host_attach_contract(...)` and assert fail-closed behavior on drift.
5. Risk: worker lifecycle leaks into host posture and reintroduces ambiguous `awaiting_attention` semantics.
   Mitigation: keep worker-level `attention_pending` state and host-level detached posture assertions in separate tests and separate persistence records.

## Deferred Beyond This Slice

1. Non-Linux automatic attach parity
2. New public CLI for direct worker lifecycle inspection
3. Broad caller-surface expansion beyond current `reattach` / `turn` / `status` / `stop`
4. Fuzzy worker or obligation targeting
5. Any attach recovery path that bypasses persisted attach truth
