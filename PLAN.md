# PLAN: Authoritative Host Attach Truth And REPL Cold-Start Parity

Source SOW: [29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md)  
Primary code anchors: [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs), [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs), [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs), [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs), [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs), [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)  
Primary test anchors: [crates/shell/tests/agent_public_control_surface_v1.rs](crates/shell/tests/agent_public_control_surface_v1.rs), [crates/shell/tests/repl_world_first_routing_v1.rs](crates/shell/tests/repl_world_first_routing_v1.rs), [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](crates/shell/tests/agent_successor_contract_ahcsitc0.rs), unit tests in [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs), unit tests in [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)  
Downstream dependency docs: [29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md), [30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md), [31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Execution branch: `feat/gateway-mediated-llm-fulfillment`  
Base branch: `main`  
Plan type: implementation-ready residual contract-authority hardening slice, no UI scope, strong shell/runtime DX scope  
Status: implementation-ready

## Objective

Finish the remaining contract-authority gaps so every host-rooted session birth path persists the same attach-relevant truth and every later attach path consumes that truth without regaining manifest-era defaults or permissive fallbacks.

This plan is complete only when all of the following are true:

1. REPL host cold start resolves through the same shared dispatch contract family already used by public start.
2. The first persisted host-rooted orchestration session stores `HostAttachContract` derived from resolved launch truth, not `HostAttachContract::from_manifest(...)`.
3. Persisted attach resolution treats `HostAttachContract` as the only durable attach baseline and applies only a bounded attach-time overlay.
4. Missing durable truth fails closed in modern steady-state paths instead of silently recovering through `Policy::default()` or manifest reconstruction.
5. Slices 30 and 31 can build on 29.75 without reopening attach-baseline semantics.

## Executive Summary

This is a hardening slice, not a redesign. The repo already has the shared vocabulary, the durable attach structure, and most of the public-start authority flow. The remaining problem is that equivalent host-rooted sessions can still persist different attach truth depending on how they were born, and later attach code can still regain broader semantics than the durable baseline intended.

The implementation direction is therefore locked:

1. `ResolvedLaunchContract` is the only birth-time authority source.
2. `HostAttachContract` is the only durable attach baseline.
3. Reattach-style attach callers may only honor that baseline, narrow it where permitted, or select among explicitly baseline-permitted modes when the durable baseline already encodes multiple allowed realizations. They may never replace or broaden it.
4. Fork is not a caller overlay on the source session. It consumes source durable truth and derives a new successor baseline for a new successor session.
5. 29.75 is greenfield for durable attach truth: no legacy compatibility helper or manifest-backfill path is part of this slice.

## Locked Decisions

These are frozen for implementation. Do not keep multiple interpretations open in code or docs.

| Topic | Locked decision | Why |
| --- | --- | --- |
| Durable attach object | `HostAttachContract` remains the only persisted attach object | A second durable model would recreate the split 29.75 exists to close |
| Birth-time authority | Host-rooted session birth persists attach truth from `ResolvedLaunchContract`, not from manifest defaults | Manifest defaults are runtime materialization detail, not contract authority |
| REPL cold start posture | REPL host cold start must join the shared dispatch contract family before the first session write | Public start and REPL cold start must stop producing different durable truth for the same backend |
| Modern missing truth | Missing or invalid durable attach truth fails closed in modern paths | Silent repair through permissive defaults broadens authority and hides corruption |
| Supported persisted-state posture | Supported persisted sessions must contain a present and valid `host_attach_contract` plus valid policy snapshot; absent, null, malformed, or incomplete durable truth fails closed | Greenfield 29.75 does not carry a compatibility classifier or repair surface |
| Attach overlay semantics | Overlay may honor baseline, narrow it where permitted, or select among explicitly baseline-permitted modes when the durable baseline already encodes multiple allowed realizations; it may not silently broaden or replace baseline semantics | Later callers cannot reinterpret the same durable session ad hoc |
| `requested_execution_scope` | For persisted attach, execution scope stays baseline-owned and immutable | 29.75 is not reopening scope selection |
| `host_execution_client_start` | Overlay may keep baseline, narrow `StartNow -> Defer`, or select among explicitly baseline-permitted modes. Because 29.75 does not add multi-mode durable encoding, current 29.75 implementations only exercise honor-or-narrow behavior. | This matches the SOW contract without letting this slice silently grow a second encoding model |
| `attach_mode_preference` | Overlay may keep baseline, narrow toward stricter continuity (`FreshAllowed -> ContinuityPreferred -> ContinuityRequired`), or select among explicitly baseline-permitted modes. Because 29.75 does not add multi-mode durable encoding, current 29.75 implementations only exercise honor-or-narrow behavior. | Continuity policy is durable baseline truth, and selection is only legal when the durable baseline explicitly says multiple realizations are allowed |
| Fork semantics | Fork may not broaden the source session by pretending to be an attach overlay. It must consume source durable truth, allocate a new successor session, and persist a new successor `HostAttachContract` through explicit successor-derivation rules. | This keeps the no-broadening overlay rule intact and prevents fork from becoming the loophole that collapses the model |
| Successor-only behavior scope | 29.75 freezes that fork uses explicit successor-baseline derivation rather than overlay broadening. It does not freeze the final successor attach-behavior policy for continuity-versus-fresh or lazy-attach defaults; that remains downstream slice 31 work unless the SOW is amended. | This keeps 29.75 focused on contract authority rather than absorbing 31's product-policy decisions |
| Retained member follow-up parity | Existing `MemberDispatchParitySubset` stays the retained-turn contract; 29.75 must not invent a second retained-member dialect | 29.5 already landed this architecture far enough to build on |
| Multi-mode baseline encoding | 29.75 does not extend `HostAttachContract` or persisted session schema to encode a set of allowed attach modes or client-start modes | This slice freezes authority semantics, not a new durable representation; “selection” remains a contract-level allowance that is dormant unless an explicit baseline encoding already exists |

## Scope

### In scope

1. Carry resolved-contract truth through REPL host cold start before first orchestration-session persistence.
2. Stop seeding modern host attach truth from `HostAttachContract::from_manifest(...)`.
3. Freeze attach-time overlay rules for persisted attach resolution and enforce them in code, diagnostics, and tests.
4. Remove steady-state fallback to `Policy::default()` and silent manifest reconstruction in modern paths.
5. Update downstream SOWs and this root plan so 30 and 31 depend on 29.75 correctly.

### NOT in scope

1. Public `substrate agent start --scope world`.
2. Attach-worker behavior and lazy attach trigger policy from slice 31.
3. Widening capability override families beyond the already-landed bounded narrowing set.
4. Inventing a second durable attach structure or third baseline domain.
5. Changing retained world-member transport format.
6. Adding new crates, binaries, or packaging flows.
7. Extending persisted attach schema to represent multi-mode allowed-mode sets for `host_execution_client_start` or `attach_mode_preference`.

## Step 0: Scope Challenge

### 0A. What already exists

The repo already contains the pieces this slice should reuse.

| Sub-problem | Existing code | Reuse decision |
| --- | --- | --- |
| Shared launch resolution vocabulary | `DispatchRequestEnvelope`, `ResolvedLaunchContract`, `DispatchBaselineKind` in [dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs) | Reuse. Do not invent a REPL-only launch dialect. |
| Public start durable attach persistence | `persist_resolved_start_attach_contract(...)` in [agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs) | Reuse semantics. Pull REPL cold start onto this same authority model. |
| Durable attach storage | `HostAttachContract`, `HostAttachLaunchKnobs`, `OrchestrationSessionRecord` in [orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs) | Reuse. Tighten constructors and recovery rules. |
| Persisted attach resolution | `resolve_persisted_host_attach_contract(...)` in [dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs) | Reuse entrypoint. Tighten overlay semantics and fail-closed behavior. |
| REPL cold start bootstrap | `resolve_host_orchestrator_bootstrap(...)` and `prepare_host_orchestrator_runtime_from_resolved(...)` in [async_repl.rs](crates/shell/src/repl/async_repl.rs) | Reuse flow. Change it to carry `ResolvedLaunchContract` instead of only `RuntimeSelectionDescriptor`. |
| Runtime materialization | `validate_runtime_realizability(...)` and `RuntimeSelectionDescriptor` | Reuse. Descriptor materialization stays downstream of contract resolution. |
| Retained member parity | `MemberDispatchParitySubset`, `retained_member_dispatch_parity_subset(...)`, `build_member_dispatch_transport_request(...)` in [async_repl.rs](crates/shell/src/repl/async_repl.rs) | Reuse. Keep parity subset intact while making parent host session birth truthful. |
| Public attach gating | `resolve_public_control_target(...)` in [state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs) | Reuse. It already enforces capability gates using durable attach truth. |

### 0B. Exact gaps this plan closes

The live code still leaves four blocking gaps:

1. `OrchestrationSessionRecord::new(...)` seeds `host_attach_contract` via `HostAttachContract::from_manifest(...)`, and REPL cold start persists that first write unchanged.
2. `sync_host_attach_contract(...)` repopulates a missing contract from manifest truth instead of treating missing modern durable truth as an error.
3. `resolve_persisted_host_attach_contract(...)` deserializes `effective_policy` and then falls back to `unwrap_or_default()`, which silently regains `Policy::default()` in modern paths.
4. Persisted attach resolution still lets request-time callers replace parts of `host_execution_client_start` and `attach_mode_preference` too freely.

### 0C. Minimum honest change

The minimum honest change set is:

1. Extend REPL bootstrap to compute and retain a `ResolvedLaunchContract`.
2. Add an explicit host-orchestrator session seed path that accepts durable attach truth up front.
3. Quarantine `from_manifest(...)` out of modern persistence and steady-state attach paths entirely.
4. Centralize persisted attach overlay validation in one helper owned by `resolve_persisted_host_attach_contract(...)`.
5. Extend unit and integration tests around the exact authority seams above.
6. Update downstream docs so the dependency floor is truthful.

Anything smaller leaves 29.75 half done and pushes the same ambiguity into 30 or 31.

### 0D. Complexity check

This slice touches more than eight files and crosses REPL, runtime, state store, CLI, tests, and docs. That would normally be a smell. Here it is justified because:

1. The work is narrow in concept even if it spans multiple seams.
2. The change is contract-hardening, not new product behavior.
3. Each touched file participates in the same authority chain.
4. There is still no need for a new crate or a new persisted state model.

### 0E. Search, completeness, and distribution

This slice should do the complete version, not the doc-only shortcut.

The shortcut is to narrow wording and accept current behavior. That saves little implementation time and guarantees later slices have to rediscover attach-authority bugs under more product pressure.

The complete version is:

1. Fix REPL birth-time persistence.
2. Fix persisted attach overlay semantics.
3. Fix modern fail-closed behavior.
4. Prove parity and failure cases in tests.
5. Align downstream docs.

No new binary, package, image, or release channel is introduced, so distribution work is unchanged.

## Contract Model

### Baseline domains

After 29.75 there are still only two baseline domains:

| Baseline domain | Used by | Authority source |
| --- | --- | --- |
| Inventory launch | public start, REPL host cold start, future world-scoped public start | `ResolvedLaunchContract` |
| Persisted host attach | `reattach`, detached-turn recovery, later lazy attach, plus source and successor sessions involved in `fork` | `HostAttachContract` |

There is no third domain for REPL-only launch semantics and no manifest-only fallback domain.

### Data flow

```text
HOST COLD START
===============
effective config + policy + inventory
        |
        v
shared dispatch resolver
        |
        +--> ResolvedLaunchContract
        |       |
        |       +--> RuntimeSelectionDescriptor (runtime materialization only)
        |       |
        |       `--> HostAttachContract::from_resolved_contract(...)
        |
        `--> OrchestrationSessionRecord persisted with authoritative attach truth


PERSISTED ATTACH
================
DispatchRequestEnvelope + HostAttachContract
        |
        v
resolve_persisted_host_attach_contract(...)
        |
        +--> validate immutable baseline fields
        +--> deserialize persisted policy snapshot
        +--> apply bounded attach overlay
        `--> ResolvedLaunchContract for attach/recovery


FORK SUCCESSOR DERIVATION
=========================
source HostAttachContract
        |
        v
explicit successor-derivation helper
        |
        +--> carry forward generalized durable truth
        +--> clear continuity-specific state
        +--> set successor attach defaults
        `--> persist new successor HostAttachContract


RETAINED MEMBER TURN
====================
existing member runtime
    or pending replacement
    or exact backend descriptor fallback
        |
        v
MemberDispatchParitySubset
        |
        v
build_member_dispatch_transport_request(...)
```

### Field ownership

#### `ResolvedLaunchContract` owns birth-time truth

For host-rooted session birth, the following fields are authoritative only once they have passed through shared resolution:

1. Backend identity and backend kind.
2. Protocol.
3. Execution scope.
4. Runtime launch descriptor.
5. Attach-relevant capabilities.
6. Effective policy snapshot.
7. Attach launch defaults.

#### `HostAttachContract` owns durable attach truth

`HostAttachContract` is the persisted attach baseline for:

1. Backend identity.
2. Protocol.
3. Execution scope.
4. Resolved runtime descriptor.
5. Attach-relevant capabilities.
6. Effective policy snapshot.
7. Continuity selector.
8. Attach policy defaults:
   - `requested_execution_scope`
   - `host_execution_client_start`
   - `attach_mode_preference`

#### Request envelope owns only bounded overlay inputs

For persisted attach, request-time input is not replacement truth. It is only:

1. A no-op if it matches the durable baseline.
2. A narrowing request if the baseline semantics allow narrowing.
3. A selection among explicitly baseline-permitted modes when the durable baseline encodes more than one acceptable realization.
4. A hard error if it broadens or semantically replaces the durable baseline.

### Fork is not an attach overlay

Fork is a separate contract operation and must not be implemented as a broadening exception to persisted attach overlay rules.

1. `reattach`, detached-turn recovery, and later lazy attach consume an existing durable baseline through the bounded overlay model above.
2. `fork` consumes the source session's durable truth as input, but it does not reinterpret that source session through caller overlay.
3. `fork` allocates a new successor session and persists a new successor `HostAttachContract`.
4. Successor derivation may transform continuity-dependent fields because it is creating a new baseline for a new session, not broadening the old one.
5. 29.75 freezes the structural rule, not the final product-policy defaults:
   - the successor path must be explicit and separate from persisted attach overlay;
   - continuity-dependent fields must be handled by successor-only derivation logic, not by broadening the source session;
   - the exact successor attach-behavior defaults remain downstream slice 31 work unless the SOW is explicitly amended.
6. Any implementation that tries to make `fork` legal by broadening the source session through `resolve_persisted_host_attach_contract(...)` is out of contract for this plan.

### Encoding rule for “selection among baseline-permitted modes”

This plan makes the encoding rule explicit so 29.75 stays narrow.

1. 29.75 does not introduce new persisted schema for “allowed mode sets” on `host_execution_client_start` or `attach_mode_preference`.
2. The current durable baseline in `HostAttachContract` remains single-valued for those fields unless the existing code already has an explicit multi-mode representation available in the same durable contract family.
3. Therefore the SOW’s “selection among baseline-permitted modes” rule remains part of the contract model, but it is intentionally dormant in 29.75 unless an explicit pre-existing durable encoding is already present.
4. In practical 29.75 implementation terms: if the durable baseline is single-valued, overlay behavior is honor-or-narrow only; any request that would require selecting among multiple baseline-permitted modes must fail closed rather than inventing new encoding semantics on the fly.
5. Fork does not use this dormant selection allowance. It gets its own explicit successor-baseline derivation rule above.
6. Any future slice that wants active selection behavior must first introduce and document an explicit durable encoding for multiple permitted realizations. That is not part of 29.75.

### Durable-state validity rule

This plan freezes one simple greenfield rule so missing truth cannot drift back into repair semantics.

1. Supported persisted sessions must carry a present and valid `host_attach_contract` and a valid serialized policy snapshot.
2. Therefore:
   - absent `host_attach_contract` fails closed;
   - present-but-null `host_attach_contract` fails closed;
   - present-but-malformed `host_attach_contract` fails closed;
   - present `host_attach_contract` plus missing `effective_policy`, missing required attach fields, or invalid serialized policy fails closed.
3. 29.75 does not introduce or preserve a migration, repair, or compatibility branch for these cases.

## Engineering Guardrails

### DRY and boundary rules

1. Do not duplicate public-start persistence logic and REPL persistence logic; the REPL should reuse the same resolved-contract-to-durable-contract semantics as `persist_resolved_start_attach_contract(...)`.
2. Do not spread attach overlay rules across `dispatch_contract.rs`, `agents_cmd.rs`, and `state_store.rs`; one helper should own the monotonicity rules.
3. Do not let `RuntimeSelectionDescriptor` become a shadow contract object; it stays a materialization descriptor only.
4. Keep manifest-derived construction logic obviously out of modern steady-state logic and attach resolution.
5. Do not introduce new crates or a second durable persisted model for this slice.

### Performance constraints

This is not a throughput-driven slice, but there are still performance constraints:

1. Do not add repeated inventory reload or policy recomputation during steady-state attach; the durable snapshot should make attach cheaper, not more expensive.
2. Do not add raw-payload classification or compatibility-only branching to the hot path.
3. Avoid repeated serde round-trips beyond the single persisted policy deserialize already required by attach resolution.
4. Do not add REPL-path logic that re-runs full contract resolution for retained member follow-up turns.

## Execution Plan

The implementation order is intentionally strict. Phase A freezes the contract semantics first. Every later phase consumes that decision rather than reinterpreting it.

### Phase A: Freeze persisted attach baseline and overlay semantics

Primary files:

1. [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
2. [crates/shell/src/execution/agents_cmd.rs](crates/shell/src/execution/agents_cmd.rs)
3. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)

Required changes:

1. Add one explicit helper that computes persisted attach launch knobs from durable baseline plus request overlay, and keep that helper scoped to reattach-style persisted attach consumers rather than fork successor creation.
2. Change `resolve_persisted_host_attach_contract(...)` so:
   - `requested_execution_scope` always comes from the contract baseline;
   - `host_execution_client_start` can honor baseline, narrow `StartNow -> Defer`, or select among explicitly baseline-permitted modes if and only if the durable baseline already encodes that allowance; otherwise it must fail closed rather than infer new multi-mode semantics;
   - `attach_mode_preference` can honor baseline, narrow toward stricter continuity, or select among explicitly baseline-permitted modes if and only if the durable baseline already encodes that allowance; otherwise it must fail closed rather than infer new multi-mode semantics;
   - invalid overlay attempts fail with field-specific baseline-truth diagnostics.
3. Replace `effective_policy.unwrap_or_default()` with explicit modern-path validation:
   - valid snapshot -> use it;
   - absent `host_attach_contract` at the session-record level -> fail closed;
   - present `host_attach_contract` plus missing or invalid snapshot -> fail closed as modern durable-state corruption.
4. Preserve `field_provenance` truth so diagnostics still explain which fields came from persisted baseline versus accepted narrowing.
5. Add an explicit successor-derivation path for fork so `agents_cmd` and public control flows do not rely on caller-side broadening of the source session baseline.
6. Verify `agents_cmd` and public control flows consume the stricter resolved contract without reopening caller-side semantic replacement.

Outputs:

1. One shared persisted-attach overlay helper.
2. One explicit baseline-vs-overlay diagnostic posture.
3. One fail-closed policy-snapshot rule used by all modern persisted-attach paths.
4. One explicit fork successor-baseline derivation rule that is separate from persisted attach overlay.

Definition of done:

1. Persisted attach never broadens or replaces durable baseline semantics.
2. Modern missing policy or contract truth fails closed.
3. Error messages explain baseline corruption or unsupported broadening.
4. Fork successor creation is legal without weakening the no-broadening overlay rule.

### Phase B: Thread resolved-contract truth into REPL host cold start

Primary files:

1. [crates/shell/src/repl/async_repl.rs](crates/shell/src/repl/async_repl.rs)
2. [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
3. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)

Required changes:

1. Change `ResolvedHostOrchestratorBootstrap` so it carries the resolved contract, not just the runtime descriptor.
2. Make `resolve_host_orchestrator_bootstrap(...)` compute a host-scoped `ResolvedLaunchContract` first, then derive `RuntimeSelectionDescriptor` from it.
3. Add an explicit session-construction path for modern host-orchestrator birth, for example `OrchestrationSessionRecord::new_host_orchestrator(...)` or a `HostAttachContractSeed`, so the first persisted session write already contains `HostAttachContract::from_resolved_contract(...)`.
4. Stop relying on `OrchestrationSessionRecord::new(...)` plus manifest seeding for REPL cold start.
5. Keep `RuntimeSelectionDescriptor` as runtime materialization input only.

Outputs:

1. REPL cold start and public start share the same birth-time authority story.
2. The first disk write for a modern REPL-born host session already contains authoritative attach truth.

Definition of done:

1. Equivalent public start and REPL cold start persist equivalent host attach truth for the same backend.
2. No modern REPL cold-start write hits disk with a manifest-derived attach contract.

### Phase C: Remove manifest-era reconstruction from steady-state logic

Primary files:

1. [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
2. [crates/shell/src/execution/agent_runtime/state_store.rs](crates/shell/src/execution/agent_runtime/state_store.rs)
3. [crates/shell/src/execution/agent_runtime/control.rs](crates/shell/src/execution/agent_runtime/control.rs)

Implementation note:

Current runtime/tests still tolerate some partial persisted `HostAttachContract` rows because `HostAttachContract`, `HostAttachCapabilities`, and `HostAttachLaunchKnobs` deserialize with defaults. Phase C must tighten that behavior so incomplete attach-contract rows fail validation/load instead of silently inheriting defaults, and any tests that currently expect partial-contract backfill on load must be updated to expect fail-closed rejection instead.

Required changes:

1. Keep `HostAttachContract::from_manifest(...)` out of steady-state modern birth paths.
2. Change `sync_host_attach_contract(...)` so it only refreshes continuity selector state on an existing contract for modern sessions.
3. Treat absent, null, malformed, incomplete, or otherwise invalid `host_attach_contract` as unsupported/corrupt durable state that fails closed.
4. Ensure persistence helpers in control and parking paths do not silently repair missing durable truth during ordinary runtime snapshots.

Outputs:

1. Zero steady-state codepaths that recreate authority from manifest defaults.
2. One clear fail-closed posture for unsupported or corrupt persisted attach truth.

Definition of done:

1. Modern missing durable attach truth is observable and rejected.
2. Any missing or invalid durable attach truth is observable and rejected.

### Phase D: Expand unit and integration coverage around the authority seams

Primary files:

1. [crates/shell/tests/repl_world_first_routing_v1.rs](crates/shell/tests/repl_world_first_routing_v1.rs)
2. [crates/shell/tests/agent_public_control_surface_v1.rs](crates/shell/tests/agent_public_control_surface_v1.rs)
3. [crates/shell/tests/agent_successor_contract_ahcsitc0.rs](crates/shell/tests/agent_successor_contract_ahcsitc0.rs)
4. Unit tests in [crates/shell/src/execution/agent_runtime/orchestration_session.rs](crates/shell/src/execution/agent_runtime/orchestration_session.rs)
5. Unit tests in [crates/shell/src/execution/agent_runtime/dispatch_contract.rs](crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

Required changes:

1. Add birth-time parity tests proving REPL cold start and public start persist the same attach truth shape for the same backend.
2. Add modern fail-closed tests for missing and corrupt persisted policy snapshots.
3. Add monotonicity tests proving `host_execution_client_start` and `attach_mode_preference` can honor baseline, narrow where allowed, and only select when the durable baseline explicitly encodes multiple permitted modes. They must never broaden or infer multi-mode semantics that are not durably encoded.
4. Add fork-successor tests proving successor creation derives a new baseline instead of broadening the source session through persisted attach overlay.
5. Add regression tests proving runtime snapshot and parked/resumed flows do not silently recreate missing modern contracts.
6. Keep retained member parity coverage intact while ensuring the parent host session baseline is now truthful.

Outputs:

1. Unit tests for authoritative persistence and fail-closed behavior.
2. Integration tests for REPL/public parity and recovery semantics.
3. A targeted test matrix that future slices 30 and 31 can trust.

Definition of done:

1. All critical authority seams have direct regression coverage.
2. Tests distinguish supported persisted-state behavior from unsupported or corrupt durable state.

### Phase E: Keep downstream docs truthful

Primary files:

1. [llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md](llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md)
2. [llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md](llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md)
3. [llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md](llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md)
4. [llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md](llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)
5. [PLAN.md](PLAN.md)

Required changes:

1. Make 29, 29.75, and this root plan say the same thing about the 29.75 contract floor.
2. Ensure 30 states it inherits durable attach truth and does not repair it.
3. Ensure 31 states attach-mode behavior consumes the 29.75 two-layer model and does not reopen baseline semantics.
4. Update lingering 29.5 language that still implies contract-authority gaps are already closed.

Outputs:

1. One consistent narrative from 29 -> 29.75 -> 30 -> 31.
2. No downstream doc that implies 30 or 31 will repair baseline contract truth left open by 29.x.

Definition of done:

1. A reader can move from 29 -> 29.75 -> 30 -> 31 without encountering contradictory contract ownership.

## Test and Validation Plan

### Code-path coverage diagram

```text
AUTHORITATIVE HOST BIRTH
[+] async_repl.rs
  ├── resolve_host_orchestrator_bootstrap()
  │   ├── [GAP] resolve shared contract before descriptor materialization
  │   └── [GAP] persist resolved contract on first orchestration-session write
  └── prepare_host_orchestrator_runtime_from_resolved()
      └── [GAP] carry resolved contract through REPL cold start

[+] orchestration_session.rs
  ├── OrchestrationSessionRecord::new(...)
  │   └── [GAP] modern host birth still seeds host_attach_contract from manifest
  ├── sync_host_attach_contract(...)
  │   ├── [GAP] missing contract silently recreated from manifest
  │   └── [ADD] continuity refresh only on existing modern contract
  └── HostAttachContract::from_resolved_contract(...)
      └── [EXISTING ★★★] preserves runtime/capability/policy truth

[+] dispatch_contract.rs
  ├── resolve_persisted_host_attach_contract(...)
  │   ├── [EXISTING ★★] persisted capabilities reused
  │   ├── [GAP] missing policy snapshot falls back to Policy::default()
  │   ├── [GAP] attach_mode_preference currently caller-replaced
  │   ├── [GAP] host_execution_client_start currently caller-replaced
  │   └── [ADD] bounded overlay helper rejects broadening
  └── field provenance
      └── [ADD] persisted baseline vs accepted narrowing provenance stays truthful

[+] retained member parity
  ├── retained_member_dispatch_parity_subset(...)
  │   └── [EXISTING ★★★] prefers live runtime / pending replacement / descriptor fallback
  └── build_member_dispatch_transport_request(...)
      └── [EXISTING ★★★] uses shared-contract-derived parity subset

COVERAGE TARGET
  Current explicit coverage is strong around persisted capability reuse and retained member parity.
  This slice must add missing parity tests for REPL cold-start birth truth, fail-closed policy snapshot handling,
  and attach overlay monotonicity.
```

Legend: `★★★` behavior + edge + error coverage already present, `★★` partial existing coverage, `[GAP]` new test required, `[ADD]` new assertion family to add.

### Required unit tests

1. `orchestration_session.rs`
   - host-orchestrator session birth from resolved contract persists authoritative attach truth on the first write;
   - modern `sync_host_attach_contract(...)` does not recreate a missing contract from manifest;
   - absent, null, malformed, or incomplete persisted contracts fail closed.
2. `dispatch_contract.rs`
   - missing modern `effective_policy` fails closed;
   - invalid serialized policy fails closed with clear diagnostics;
   - `host_execution_client_start` accepts honor, narrowing, and selection only when explicitly encoded by the durable baseline; with the current 29.75 single-valued baseline it therefore behaves as honor-or-narrow only;
   - `attach_mode_preference` accepts honor, narrowing, and selection only when explicitly encoded by the durable baseline; with the current 29.75 single-valued baseline it therefore behaves as honor-or-narrow only;
   - overlay broadening attempts fail with the correct `field`.
3. `agents_cmd.rs` and successor-focused tests
   - fork must not call persisted attach resolution in a way that broadens the source baseline;
   - successor allocation derives and persists a new successor `HostAttachContract`;
   - successor sessions use explicit successor-only handling for continuity-dependent fields rather than inheriting behavior through overlay broadening;
   - all non-successor durable truth carries forward unless an explicit successor-only rule says otherwise.
4. Durable-state validity tests
   - rows with no `host_attach_contract` fail closed as unsupported/corrupt durable state;
   - rows with present-but-null `host_attach_contract` fail closed as corrupted durable state;
   - rows with malformed `host_attach_contract` fail closed as corrupted durable state;
   - rows with a present but incomplete or otherwise invalid `host_attach_contract` fail closed as corrupted durable state.

### Required integration tests

1. `repl_world_first_routing_v1.rs`
   - REPL host cold start persists the same attach-relevant truth shape as public start for the same backend;
   - a REPL-born session that gets parked and later resumed still uses the persisted continuity selector and stored policy snapshot.
2. `agent_public_control_surface_v1.rs`
   - reattach and turn recovery reject sessions missing modern durable policy truth;
   - attach-time requests that attempt to broaden attach mode or client-start posture fail closed;
   - fork succeeds through explicit successor derivation rather than overlay broadening of the source session.
3. `agent_successor_contract_ahcsitc0.rs`
   - successor sessions preserve generalized attach truth and handle continuity-dependent fields through explicit successor derivation after the 29.75 changes;
   - successor sessions do not require overlay broadening to achieve their successor-only behavior.

### Test commands

At minimum:

```bash
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture
cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture
cargo test -p shell host_attach_contract_from_resolved_contract_preserves_truth -- --nocapture
cargo test -p shell persisted_attach_contract_is_explicit_baseline_domain -- --nocapture
```

If helper APIs change substantially, also run:

```bash
cargo test -p shell --lib execution::agent_runtime::dispatch_contract -- --nocapture
cargo test -p shell --lib execution::agent_runtime::orchestration_session -- --nocapture
```

### Manual validation

Manual validation must prove:

1. A REPL-born host session persists the same attach-relevant truth shape as a public-start-born host session for the same backend.
2. Later attach planning reuses the full durable baseline from `HostAttachContract`, including attach policy defaults, without regaining `Policy::default()` or manifest-default capabilities.
3. Missing or invalid persisted durable truth fails closed with no repair or compatibility branch.
4. Bounded overlay behavior is explicit and auditable, with no caller-side semantic replacement of persisted attach policy.
5. 30 and 31 can be read straight through after 29.75 without contradicting the actual runtime contract floor.

## Failure Modes Registry

| Code path | Realistic production failure | Test required | Error handling required | User-visible outcome |
| --- | --- | --- | --- | --- |
| REPL host cold start | session is persisted with manifest-derived defaults instead of resolved truth | Yes | Yes | Later attach behaves differently from public start |
| Persisted attach resolution | stored `effective_policy` is missing or corrupt on a session that already has `host_attach_contract` | Yes | Yes | Clear corruption error, not silent permissive attach or accidental legacy fallback |
| Durable-state validation | a row with missing or incomplete `host_attach_contract` is mistakenly accepted or silently repaired | Yes | Yes | Corrupt or unsupported sessions slip past the contract floor |
| Attach overlay | caller broadens `attach_mode_preference` or `host_execution_client_start` | Yes | Yes | Clear "baseline truth rejected field" error |
| Fork successor allocation | fork is implemented as a broadening overlay on the source session instead of successor derivation | Yes | Yes | Fork fails unexpectedly or weakens the overlay contract for every other caller |
| Runtime snapshot sync | missing contract silently recreated from manifest during parking or snapshot write | Yes | Yes | Hidden authority drift that only appears on later resume |
| Retained member parity | REPL retains old member parity while parent host session baseline changed | Existing coverage plus regression check | Yes | Follow-up turn routes with stale runtime truth |

### Critical gaps

These are the critical gaps this plan must close before implementation can be called done:

1. First-write REPL cold-start parity is currently unproven and likely false.
2. Modern missing policy snapshot currently broadens through defaults.
3. Attach-time caller input currently replaces durable attach knobs too freely.
4. Current fork allocation broadens `ContinuityRequired` source sessions by asking for `FreshAllowed` through persisted attach overlay.

## Worktree Parallelization Strategy

This plan has real parallelization opportunities, but only after the contract rules are frozen. Before that point, parallel work would just create rebase churn and semantic drift.

### Dependency table

| Step | Modules touched | Depends on |
| --- | --- | --- |
| A. Freeze attach baseline and overlay rules | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/execution/` | — |
| B. Thread resolved contract into REPL cold start | `crates/shell/src/repl/`, `crates/shell/src/execution/agent_runtime/` | A plus the shared `orchestration_session.rs` session-seed prerequisite landed from Lane C |
| C. Remove manifest fallback from steady-state paths | `crates/shell/src/execution/agent_runtime/`, `crates/shell/src/execution/agent_runtime/control.rs` | A |
| D. Expand unit and integration coverage | `crates/shell/src/execution/agent_runtime/`, `crates/shell/tests/`, `crates/shell/src/repl/` | A for resolver semantics, B and C for parity and recovery cases |
| E. Align downstream docs | `llm-last-mile/`, repo root planning docs | A complete enough to freeze wording, final pass after B and C |

### Parallel lanes

Lane A: Step A  
Reason: this freezes the contract and helper shapes every other lane consumes.

Lane B: Step B REPL bootstrap wiring plus Step D subset for REPL parity  
Reason: REPL cold-start work can proceed once the baseline and overlay helper shape is frozen, but it consumes a shared session-seed prerequisite owned outside Lane B.

Lane C: Step C plus the shared `orchestration_session.rs` session-seed prerequisite, then Step D subset for fail-closed persistence validation  
Reason: steady-state manifest-reconstruction removal already owns `agent_runtime/` and `orchestration_session.rs`, so it is the right lane to land the shared constructor or seed primitive that Lane B then consumes.

Lane D: Step E  
Reason: doc rewrites can begin once Step A lands conceptually, but final wording waits for B and C behavior to settle.

### Execution order

1. Land Lane A first. Nothing else starts until the baseline and overlay semantics are frozen.
2. After Lane A stabilizes, Lane C lands the shared `orchestration_session.rs` session-seed prerequisite plus the steady-state reconstruction removal in the same files.
3. Once that prerequisite is in place, launch the remaining Lane B and Lane C work in parallel worktrees if ownership is still clean.
4. Merge B and C, then run the full targeted test matrix.
5. Finish with Lane D once runtime behavior is stable enough that docs can be written once and stay true.

### Conflict flags

1. Lanes B and C both touch `crates/shell/src/execution/agent_runtime/`, and Phase B requires a new session-construction path in `orchestration_session.rs`. That prerequisite belongs to Lane C, not Lane B.
2. If two workers are used, assign ownership explicitly:
   - Worker B owns `async_repl.rs` plus REPL-facing tests and consumes the already-landed session-seed API;
   - Worker C owns `dispatch_contract.rs`, `orchestration_session.rs`, `state_store.rs`, `control.rs`, and attach-control tests, including the session-seed constructor or helper Phase B depends on.
3. If Lane C cannot land that prerequisite first, run B and C sequentially.
4. Lane D should not finalize any dependency wording until B and C land, or the docs will drift again.

## Acceptance Criteria

This slice is complete only when all of the following are true:

1. REPL host cold start and public start persist equivalent `HostAttachContract` truth for equivalent host-backed launches.
2. No steady-state host session birth path relies on `HostAttachContract::from_manifest(...)` as its primary truth source.
3. Persisted attach resolution reuses the full durable attach baseline from `HostAttachContract`, including attach policy defaults for `requested_execution_scope`, `host_execution_client_start`, and `attach_mode_preference`.
4. Any attach-time request variation is modeled explicitly as a bounded overlay constrained by that baseline, with no silent caller-side replacement of persisted attach policy.
5. Missing durable policy or attach truth fails closed in modern steady-state paths instead of silently broadening through defaults.
6. Code, tests, and docs all describe the same overlay rules: honor baseline, narrow where permitted, or select among baseline-permitted modes, but never broaden or replace the durable baseline silently.
7. Fork is implemented as successor-baseline derivation, not as a broadening overlay on the source session.
8. Missing, null, malformed, or incomplete persisted durable attach truth fails closed with no compatibility helper or repair branch in this slice.
9. 30 and 31 can both move forward after 29.75 without reopening baseline attach semantics.

## Implementation Tasks

Synthesized from the scope challenge, contract model, and execution plan above. Each task is buildable and directly tied to one of the contract gaps this slice owns.

- [ ] **T1 (P1, human: ~2h / CC: ~20min)** — `agent_runtime/` — Add an authoritative host-orchestrator session seed path that persists `HostAttachContract` from `ResolvedLaunchContract` on the first write.
  - Surfaced by: Step 0 / Phase B — REPL cold start currently persists manifest-derived attach truth.
  - Files: `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agents_cmd.rs`
  - Verify: unit tests for first-write host attach truth plus public-start parity checks.

- [ ] **T2 (P1, human: ~2h / CC: ~20min)** — `repl/` — Make `resolve_host_orchestrator_bootstrap(...)` carry resolved-contract truth and make REPL cold start persist it before runtime launch.
  - Surfaced by: Phase B — REPL bootstrap still materializes runtime without durable contract authority.
  - Files: `crates/shell/src/repl/async_repl.rs`
  - Verify: `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`

- [ ] **T3 (P1, human: ~2h / CC: ~15min)** — `dispatch_contract` — Add one bounded overlay helper for persisted attach launch knobs, scope it to reattach-style consumers, and reject broadening attempts.
  - Surfaced by: Step 0 / Phase A — attach-mode and client-start semantics are too caller-owned today, and fork must not become a broadening loophole.
  - Files: `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - Verify: targeted persisted-attach resolver unit tests.

- [ ] **T4 (P1, human: ~1h / CC: ~10min)** — `dispatch_contract` — Remove modern `Policy::default()` fallback from persisted attach resolution and fail closed on missing or invalid policy snapshots.
  - Surfaced by: Step 0 / Failure modes — missing durable truth currently broadens silently.
  - Files: `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
  - Verify: new unit tests plus `agent_public_control_surface_v1`.

- [ ] **T5 (P1, human: ~90min / CC: ~15min)** — `orchestration_session/control` — Remove steady-state manifest reconstruction, stop `sync_host_attach_contract(...)` from silently recreating missing modern contracts, and add an explicit successor-baseline derivation path for fork.
  - Surfaced by: Step 0 / Phase C — steady-state semantics and successor derivation both need clear fail-closed boundaries.
  - Files: `crates/shell/src/execution/agent_runtime/orchestration_session.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, `crates/shell/src/execution/agents_cmd.rs`
  - Verify: modern fail-closed tests and successor-contract tests.

- [ ] **T6 (P2, human: ~2h / CC: ~20min)** — `shell tests` — Add REPL/public-start parity coverage, attach overlay monotonicity tests, and regression coverage for parked and resumed host sessions.
  - Surfaced by: Test and validation plan — core authority paths still have coverage holes.
  - Files: `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/agent_public_control_surface_v1.rs`, `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`
  - Verify: full targeted test matrix in this plan.

- [ ] **T7 (P2, human: ~45min / CC: ~10min)** — `docs` — Update 29, 29.75, 30, and 31 wording so every slice describes the same contract floor and dependency order.
  - Surfaced by: Phase E — downstream docs already point at 29.75 and must stay truthful.
  - Files: `llm-last-mile/29-shared-agent-dispatch-envelope-and-capability-override-contract.md`, `llm-last-mile/29.75-authoritative-host-attach-truth-and-repl-cold-start-parity.md`, `llm-last-mile/30-public-world-scoped-agent-start-and-capability-flags.md`, `llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`, `PLAN.md`
  - Verify: manual read-through of 29 -> 29.75 -> 30 -> 31.
