# PLAN-48: Internal Family-2 Router-Owned Session Auto-Attach Execution Boundary

Source spec: [SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md)  
Source remaining-scope note: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Plan type: first Family-2 execution-boundary slice  
Status: implemented on `2026-06-07`
Landed posture note: the first internal host-side router-owned local-session auto-attach execution boundary is landed repo-wide; it remains Linux-only in v1, local-session-only, deny-by-default through `workflow.router.enabled` plus the relevant existing retained-obligation gate, continuity-first on persisted attach truth, and bounded away from cross-host ingress, public daemon UX, worker continuation, and review-state resolution.
Validation note: Packet 4's validation wall is green. Final validation did not require any in-scope Slice `48` stabilization follow-up, and no broader host-targeting, ingress, or workflow-engine work was reopened.

## Objective

Land the first Family-2 execution-boundary slice by turning the already-landed local obligation-ledger and attach helper mechanics into a real host-side router-owned automatic attach path for detached local orchestration sessions, with exact session coalescing, continuity-first restoration, fail-closed policy gating, and honest attach-state settlement, without widening into cross-host ingress, workflow-engine breadth, public router daemon UX, or obligation review resolution.

This slice is complete only when all of the following are true:

1. a non-test host-side router-owned execution path exists for local session auto-attach,
2. router-owned auto-attach is deny-by-default and consults `workflow.router.enabled` plus the relevant existing world-dispatch obligation/fork gates,
3. router-owned execution evaluates only local exact session truth, local obligations, and persisted attach-contract truth,
4. at most one obligation leads an attach episode per orchestration session at a time,
5. continuity attach is preferred and fresh attach stays fail-closed fallback only,
6. successful attach restores host ownership truth and settles attach-state substates without resolving review state,
7. manual `reattach` and router-owned attach converge without duplicate launch,
8. public CLI daemon control, cross-host ingress, and worker continuation remain deferred.

## Phase Gate

This plan assumes the `SPECIFY` phase artifact in [SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md) has been reviewed and is the source of truth for scope before implementation planning advances.

## Major Components And Dependencies

1. `crates/shell/src/execution/agent_runtime/auto_attach.rs`
   - already owns local candidate selection, attach-mode selection, hidden-owner launch, and fail-closed settle helpers
   - must remain the semantic center of router-owned local session auto-attach
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
   - already owns exact local session load, obligation persistence, claim coalescing, and settlement after attach restoration
   - must remain the authoritative source of "one claim per orchestration session"
3. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
   - owns current local obligation and attach-state model
   - should stay semantically stable in this slice even if helper methods widen slightly
4. host-side shell execution wiring under `crates/shell/src/execution/`
   - needs the first real non-test router-owned call site or internal work-loop entrypoint
   - must stay host-side and internal-only
5. `crates/shell/src/execution/policy_model.rs`, `crates/broker/src/policy.rs`, and `crates/broker/src/effective_policy.rs`
   - already expose `workflow.router.*` and the existing world-dispatch obligation/fork gates
   - must provide the deny-by-default contract this slice actually enforces
6. `docs/CONFIGURATION.md`, `docs/reference/policy/contract.md`, and `docs/TRACE.md`
   - must stay honest once router-owned auto-attach becomes live runtime truth

## Plan Summary

After Slice `47`, the Family-1 control plane is no longer the narrowest next seam. The repo already has the local Family-2 primitives:

1. a canonical local obligation artifact,
2. attach claim and sibling-settlement helpers,
3. continuity-first hidden-owner attach execution,
4. fail-closed fresh-attach truth,
5. manual exact-session `reattach`.

What the repo still does not have is the actual execution boundary that owns those primitives in production. `execute_session_auto_attach(...)` exists, but there is no non-test call site or host-side router-owned loop that uses it.

The design stack is consistent enough to freeze the missing boundary now:

1. the router is host-side, not world-service-owned,
2. auto-attach consumes durable local obligations, not raw stream events,
3. attach launch is session-scoped even though eligibility starts obligation-scoped,
4. successful attach restores host ownership only and does not resolve review state,
5. cross-host inbox and remote ingress stay later work.

The narrowest honest Slice `48` is therefore:

1. freeze policy and semantic alignment first,
2. add the first real host-side router-owned execution path second,
3. prove settlement, manual-reattach convergence, and observability third,
4. finish with docs and validation fourth.

## Locked Decisions

### What changes

1. Slice `48` chooses a host-side router-owned work-loop or equivalent internal entrypoint, not synchronous inline evaluation inside ordinary retained-worker control flows.
2. Slice `48` reuses the current local obligation artifact and attach-state enum semantics rather than renaming them first.
3. Router-owned attach is deny-by-default and is gated by:
   - `workflow.router.enabled`
   - the existing relevant world-dispatch obligation/fork gates for the claimed obligation kind
4. Router-owned attach remains local-only, exact-session only, and Linux-first.
5. Successful router-owned attach restores a sanctioned host execution client and settles attach-state substates only.

### What does not change

1. no public router/daemon CLI management surface,
2. no host-global inbox or cross-host routing,
3. no obligation review resolution automation,
4. no worker continuation, approval, clarification, or fork execution from the router path,
5. no general workflow-engine implementation,
6. no world-service-owned attach logic.

## Implementation Order

### Packet 1: Policy And Attach-Semantic Contract Freeze

Goal:

1. freeze the actual allow/deny contract for router-owned local auto-attach,
2. map current local attach-state names onto the forward design semantics without renaming the artifact first,
3. make the router path fail closed when policy or local boundary truth is missing.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/auto_attach.rs`
2. `crates/shell/src/execution/agent_runtime/obligation_ledger.rs`
3. `crates/shell/src/execution/policy_model.rs`
4. `crates/broker/src/policy.rs`
5. `crates/broker/src/effective_policy.rs`
6. targeted policy and auto-attach tests

Why first:

1. the repo already has working local attach helpers, but they do not yet consult a dedicated router policy layer,
2. execution wiring should consume one frozen policy contract rather than deciding allow/deny ad hoc,
3. the current attach-state enum mismatch with the forward design should be handled as semantic mapping, not as surprise rename churn during later packets.

Verification checkpoint:

1. router-owned auto-attach is deny-by-default,
2. missing or disabled `workflow.router.enabled` fails closed,
3. per-kind eligibility remains deny-by-default unless the relevant existing gate allows it,
4. current local attach-state semantics are explicit and stable in tests.

### Packet 2: Production-Owned Router Execution Path

Goal:

1. introduce the first real non-test host-side router-owned execution path,
2. enumerate local candidate sessions and obligations under exact local truth,
3. call the existing local attach executor through one internal work-loop or equivalent internal entrypoint.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/auto_attach.rs`
2. host-side shell execution wiring under `crates/shell/src/execution/`
3. `crates/shell/src/execution/agent_runtime/state_store.rs`
4. targeted shell tests

Why second:

1. once policy and local semantics are frozen, the next real gap is the missing production call site,
2. the slice needs a host-side owner for evaluation -> claim -> attach -> settle,
3. adding the real execution path before broad observability work keeps the slice focused on the core boundary.

Verification checkpoint:

1. a non-test host-side call site exists,
2. the router evaluates only local session and obligation truth,
3. continuity attach is preferred and fresh attach stays fail-closed fallback only,
4. no world-service-owned or public CLI-owned attach execution path is introduced.

### Packet 3: Settlement, Manual Reattach Convergence, And Observability

Goal:

1. prove that router-owned attach and manual `reattach` converge without duplicate launch,
2. keep success, fail-closed, and sibling-supersession outcomes distinct,
3. emit explanation-ready router-owned decisions without widening into a general workflow engine.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/auto_attach.rs`
2. `crates/shell/src/execution/agent_runtime/state_store.rs`
3. any narrow host-side trace/event helper used by the chosen router entrypoint
4. `docs/TRACE.md` only if a real trace family is added
5. targeted shell tests

Why third:

1. the main semantic risk after wiring the call site is duplicate or misleading attach behavior,
2. the router must stay explanation-ready without taking on worker business logic,
3. observability should reflect real settled behavior rather than provisional design intent.

Verification checkpoint:

1. successful manual `reattach` cancels or converges router-owned queued/claimed work without duplicate attach launch,
2. successful router-owned attach settles attach-state substates but does not resolve review state,
3. router-owned decisions are explanation-ready and preserve exact session/obligation/backend joins.

### Packet 4: Docs Alignment And Final Validation

Goal:

1. align policy and runtime docs with the real Slice `48` boundary,
2. keep local-only, Linux-first, router-owned scope explicit,
3. run the validation wall.

Primary touch surface:

1. `docs/CONFIGURATION.md`
2. `docs/reference/policy/contract.md`
3. `docs/TRACE.md` if touched
4. `llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md`
5. `llm-last-mile/PLAN-48.md`
6. `llm-last-mile/TASKS-48.md`

What this packet must enforce:

1. docs describe Slice `48` as the first host-side router-owned Family-2 execution boundary, not as cross-host routing or a public daemon feature,
2. docs state the deny-by-default gate honestly,
3. docs keep broader host-global inbox, richer obligation identity envelope, and workflow-engine breadth explicitly deferred.

Verification checkpoint:

1. docs and policy truth are honest,
2. validation is green,
3. the next Family-2 follow-on can stay focused on broader host-targeting or ingress envelope work instead of reopening the local execution boundary.

## Risks And Mitigations

1. Risk: Slice `48` accidentally becomes a general router-daemon or workflow-engine project.
   Mitigation: keep the slice local-only, attach-only, internal-only, and exact-session scoped.
2. Risk: policy ownership gets split between unrelated namespaces.
   Mitigation: reuse `workflow.router.enabled` for router indirect execution and existing world-dispatch obligation/fork gates for per-kind eligibility unless implementation proves that is insufficient.
3. Risk: the current attach-state enum name mismatch causes needless churn.
   Mitigation: preserve the current runtime names and document the semantic mapping instead of renaming the artifact in this slice.
4. Risk: success semantics drift into obligation resolution or silent worker continuation.
   Mitigation: keep success strictly limited to restoring attached host truth and settling attach substates.
5. Risk: no real production call site appears, leaving the slice as another local-helper refinement only.
   Mitigation: Packet 2 explicitly requires a non-test host-side execution path as the core completion criterion.

## Sequencing And Parallelism

### Must stay sequential

1. Packet 1 before Packet 2 because the production call site should consume one frozen policy and attach-state contract.
2. Packet 2 before Packet 3 because convergence and observability should validate the real execution path, not a placeholder helper.
3. Packet 3 before Packet 4 because docs and validation should describe the landed boundary honestly.

### Can be deferred

1. host-global inbox or remote ingress layering,
2. broader host-targeted obligation identity envelope,
3. public router daemon lifecycle UX,
4. broader workflow-engine or toolbox routing work,
5. cross-host claim ownership and wrong-host delivery beyond local fail-closed posture.

## Verification Wall

Minimum validation before calling the slice complete:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p shell auto_attach -- --nocapture
cargo test -p shell state_store -- --nocapture
cargo test -p shell policy_model -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
cargo test -p substrate-broker -- --nocapture
cargo test --workspace -- --nocapture
```

## Expected Follow-On Order After Slice `48`

If Slice `48` lands cleanly as the local router-owned execution boundary, the next Family-2 follow-on should remain:

1. any broader host-targeted or wrong-host fail-closed obligation envelope widening,
2. any ingress-ready identity fields needed for host-global inbox or remote federation,
3. any broader router lifecycle or workflow-engine productization only after the local execution boundary is no longer open.
