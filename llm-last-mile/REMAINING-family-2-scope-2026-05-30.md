# Remaining Family-2 Scope After Slices 31 And 31.25

Date: `2026-05-30`  
Validated against:
- [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)
- [PLAN-31.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-31.md)
- [TASKS-31.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-31.md)
- [SPEC-31_25.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31_25.md)
- [PLAN-31_25.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-31_25.md)
- [TASKS-31_25.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-31_25.md)
- [CLOSEOUT-31_25-remediation-2026-05-30.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/CLOSEOUT-31_25-remediation-2026-05-30.md)

## Objective

Record the repo-truth answer to one question:

1. after slices `31` and `31.25`, is the durable deferred-work / obligation-ledger / auto-attach family complete,
2. or is there still implementation-bearing scope left in the broader family-2 design stack?

This note is a validation artifact, not a new implementation spec.

## Scope Definition

For this note, "family 2" means the durable deferred-work and attach-recovery design stack:

1. [DESIGN-durable-orchestration-obligation-ledger.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-durable-orchestration-obligation-ledger.md)
2. [DESIGN-auto-attach-trigger-and-work-queue-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-auto-attach-trigger-and-work-queue-contract.md)
3. [DESIGN-router-daemon-attach-trigger-integration.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md)

This note does not evaluate the separate control-plane dispatch family:

1. [DESIGN-host-orchestrator-world-dispatch-contract.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-host-orchestrator-world-dispatch-contract.md)
2. [DESIGN-host-to-world-steering-policy-matrix.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-host-to-world-steering-policy-matrix.md)

## Validation Method

The validation pass checked:

1. design docs,
2. slice `31` and `31.25` planning/closeout docs,
3. live runtime code in:
   - [obligation_ledger.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
   - [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
   - [auto_attach.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
   - [orchestration_session.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
   - [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
4. live tests and targeted reruns:
   - `cargo test -p shell auto_attach -- --nocapture`
   - `cargo test -p shell obligation -- --nocapture`

## Landed Scope

The following family-2 work is landed in the current tree.

### 1. Canonical local obligation artifact exists

The repo now has a dedicated session-local obligation artifact in [obligation_ledger.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs).

What is real:

1. explicit obligation kinds,
2. explicit review and attach substates,
3. validation at the persistence boundary,
4. exact orchestration-session ownership,
5. world-binding support for local world-bound obligations.

### 2. Obligation persistence is authoritative for detached attention projection

[state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) now persists obligations, projects them back into compatibility inbox rows, and derives `pending_inbox_count` / detached posture from obligation truth.

What is real:

1. pending-attention projection from obligations,
2. compatibility inbox item persistence remains as a projection layer,
3. detached posture normalization continues to work against the current status surface.

### 3. Session-scoped attach claiming and dedupe exist

[state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs) implements:

1. one claimed attach obligation per orchestration session,
2. deterministic candidate selection,
3. attach-claim metadata persistence,
4. sibling-settlement after attach restoration.

### 4. Manual `reattach` and automatic attach share one launch authority path

The runtime now routes both manual `reattach` and automatic attach through the same hidden-owner-helper seam:

1. [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
2. [auto_attach.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
3. [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

This is the core runtime contract that slice `31` was supposed to land.

### 5. `31.25` correctly restored fail-closed fresh-attach truth

The remediation is real, not just documented:

1. continuity-backed attach remains allowed,
2. fresh-needed automatic attach now fails closed before backend launch,
3. prompt-free fresh control attach is not treated as shipped behavior.

That is reflected in [CLOSEOUT-31_25-remediation-2026-05-30.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/CLOSEOUT-31_25-remediation-2026-05-30.md).

## Remaining Implementation Seams

The family is not fully complete. The remaining scope is narrower than the original design stack, but it is still real implementation work.

### 1. The router/daemon execution boundary is still unresolved in production

The repo contains a real attach executor in [auto_attach.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs), but the current tree does not show a live production watcher/work-loop invoking it.

Repo-truth consequence:

1. the attach claim / execution machinery exists,
2. but the dedicated router/daemon layer described in [DESIGN-router-daemon-attach-trigger-integration.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-router-daemon-attach-trigger-integration.md) is not yet a clearly wired shipped component.

This matches the still-open question recorded in [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md): whether router-owned automatic attach is a synchronous runtime evaluation or a dedicated background daemon/work-loop artifact.

### 2. The shipped obligation record is narrower than the forward design contract

The forward design preserves room for:

1. `origin_host_id`
2. `target_host_id`
3. `ingress_source_kind`
4. `ingress_source_id`
5. `ingress_received_at`
6. `causation_event_id`
7. `causation_message_id`
8. `causation_request_id`

The landed record in [obligation_ledger.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs) does not yet carry most of that identity envelope.

Repo-truth consequence:

1. the local session-only slice is implemented,
2. the broader host-targeted / remote-ingress-ready obligation envelope is still design-only.

### 3. Auto-attach is not yet wired through an explicit deny-by-default policy layer

The broader family-2 design stack expects auto-attach to remain policy-gated by:

1. obligation kind,
2. exact boundaries,
3. host targeting,
4. later steering policy.

The shipped claim path currently checks:

1. session existence,
2. terminal state,
3. attached-host presence,
4. attach-contract presence,
5. obligation eligibility,
6. duplicate claimed rows.

It does not yet consult a dedicated policy layer matching the broader design intent.

### 4. Wrong-host targeting and host-global ingress layering are still future scope

The forward design intentionally preserves room for:

1. `SUBSTRATE_HOME/host_inbox/`,
2. wrong-host fail-closed behavior,
3. local materialization from broader ingress layers.

The current landed slice is still local-session scoped. There is no evidence in the current runtime of shipped host-global inbox or wrong-host claim handling.

## Invalidated Scope

The following previously broad interpretation is invalid:

1. "family 2 is still mostly design-only after `31` and `31.25`."

That is no longer true.

The current tree proves that the local Linux specialized slice is real:

1. obligations exist as a canonical local artifact,
2. detached attention projection derives from obligations,
3. attach claiming/coalescing is implemented,
4. manual and automatic attach share one durable authority path,
5. fresh-needed automatic attach fails closed after `31.25`.

## Recommended Classification

This should be treated as **a fresh implementation slice**, not a cleanup-only slice.

Reason:

1. the remaining work is not just wording or doc drift,
2. it still includes at least one production-runtime boundary decision,
3. it likely needs new code to wire or define the router/daemon execution model,
4. it likely needs explicit scoping around policy gating and host-targeting.

What it is not:

1. not a rewrite of slices `31` or `31.25`,
2. not a reason to reopen the already-landed local obligation / fail-closed attach mechanics,
3. not a giant multi-family orchestration rewrite.

## Recommended Next Slice Shape

If this becomes a new `SPEC/PLAN/TASKS` family, the narrowest honest objective is:

1. freeze the production execution boundary for router-owned automatic attach,
2. decide whether attach watching/execution is synchronous runtime logic or a dedicated daemon/work-loop,
3. add only the minimum policy/host-targeting envelope needed for that boundary,
4. leave broader cross-host ingress and remote federation out of scope unless explicitly promoted.

## Bottom Line

Family 2 is **partially complete**:

1. the local Linux obligation-ledger + attach-claim + fail-closed slice is landed,
2. the broader router/daemon and policy/host-targeting layer is still remaining implementation scope,
3. therefore the remaining work deserves a fresh narrow slice, not just cleanup.
