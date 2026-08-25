**Kind:** slice and task record
**Stable ID:** `B1-B2.1-family`
**Canonical for:** B0, B1, B2.1, R0, B3.2a/B3.2a-WA prerequisite, B1/B2.1-0, and joint-closeout slice/task records only
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** composite of the five preserved root compatibility spans listed in the extraction ledger
**Supersedes:** canonical ownership of the extracted source bodies; source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# B1/B2.1 family slice and task record

### B1 receipt core and activated-store allowlist

The B1 request-carrier change is high-breadth typed plumbing with a closed file boundary. Its
canonical ownership and implementation files are exactly:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`
- `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store_tests.rs` (focused
  activated-store, lock, crash-reconciliation, and legacy-writer-exclusion proof only)
- `crates/common/src/authority_commitment.rs` (new file authorized only for the shared
  equality-only commitment and correlation types)
- `crates/common/src/lib.rs`
- `crates/transport-api-types/src/lib.rs`
- `crates/world-service/src/service.rs`
- `crates/world-service/src/member_runtime.rs`

The complete additional allowlist for existing production request constructors and their focused
tests/fixtures is exactly:

- `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- `crates/shell/src/execution/agent_runtime/control.rs`
- `crates/shell/src/repl/async_repl.rs`
- `crates/replay/src/replay/executor.rs`
- `crates/shell/src/execution/platform/linux.rs`
- `crates/world-mac-lima/src/lib.rs`
- `crates/world-windows-wsl/src/backend.rs`
- `crates/world-service/tests/fs_mode.rs`
- `crates/world-service/tests/full_isolation_nonpty.rs`
- `crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`
- `crates/world-service/tests/member_runtime_world_placement_v1.rs`
- `crates/world-service/tests/overlayfs_enumeration.rs`
- `crates/world-service/tests/streamed_execute_cancel_v1.rs`
- `crates/world-service/tests/wfgad3_wildcard_deny_symlink_handling.rs`
- `crates/world-service/tests/wfgad5_strict_deny_lockdown.rs`

Only the canonical `run_world_task` path, through
`routing/dispatch/world_ops.rs::build_execute_request`, and the canonical
`continue_world_worker` path, through
`orchestrator_world_dispatch.rs::build_continue_world_worker_submit_request`, may construct a
present `WorldWorkAcceptanceContextV1`. The typed transport decoder may validate and propagate a
supplied context, and world-service may retain and acknowledge it, but neither may originate one.
Every other enumerated constructor or fixture may only initialize the additive optional field as
absent. Those mechanical initializations must not alter control flow, policy or world enforcement,
replay semantics, doctor behavior, lifecycle, terminal handling, or existing serialization
compatibility.

No environment variable, prompt, header, global/process-local side table, or parallel resolver may
substitute for the typed request field. `ExecuteStreamFrame` remains unchanged and foreground
behavior remains blocking. B2.1 supervision/replay, B3.1 retained-event semantics, C1 obligations,
B2.2 early return, and A1.2 transition authority remain non-goals. A required semantic edit outside
this exact list is `CrossDocumentChangeRequired`, not permission to widen B1.

The three HostSessionAuthority-store files above are authorized only for B1's physical persistence
substrate. `WorldWorkReceiptRegistry` owns the receipt schemas, semantic validation, proposal and
acceptance transitions, exact-retry joins, conflict rejection, and inspection. The store layer may
only open and retain the already-trusted physical root, acquire the existing cross-process root
lock, run canonical activated-store preflight, expose the fixed receipt-registry namespace, publish
opaque canonical bytes crash-safely, and reconcile interrupted receipt publications. Its capability
is bound to one exact activated authority-store identity, has no arbitrary path/collection API, and
cannot read or mutate either strict `StateRootV1` or strict `StateRootV2`, session-authority
objects, transition intents, journals, keys, or typed authority objects. The existing legacy writer remains rejected after the initialization
marker or activated root exists. No dual write, fallback write, side table, or fresh-root-only
production mode is permitted; physical serialization does not transfer receipt semantics to
HostSessionAuthority.

B1 implementation resumes in this exact order:

1. **B1-3a — activated-authority-safe receipt transaction substrate:** add only the restricted
   physical capability and genuinely activated-store proof while preserving legacy-writer
   exclusion.
2. **B1-3b — authority-bound `WorldWorkReceiptRegistry` adoption:** replace the draft generic
   `BoundAgentRuntimeStateStore` receipt writer with the restricted capability; retain all receipt
   decisions in the registry owner.
3. **B1 receipt-core review point:** prove proposal, exact retry, task/retained acknowledgement,
   activated-store persistence, and immutable inspection. This point may authorize B2.1 work but
   does not claim B1 production completion.

B2.1 then decomposes into three independently reviewable subpackets:

1. **B2.1-1 — durable observation claim and no-gap handoff:** at exact B1 acceptance, create or
   exact-join one durable receipt-scoped claim before reading any subsequent frame. Bind the claim
   to the acceptance record/revision, authority store, stream, session, work identity, world, and
   observer epoch. Replace the ephemeral accepted path's legacy active-task registration, and
   transfer the retained accepted path from foreground observation ownership to the same durable
   claim; conflict or stale identity fails closed.
2. **B2.1-2 — durable frame/event journal and compatibility waiter:** persist exact B0 identity and
   canonical bytes before foreground delivery; make an exact duplicate a no-op; reject gaps,
   reorder, conflicting identity reuse, stale observers, and post-terminal frames. Preserve the
   blocking foreground UX as a waiter over supervisor truth. Dropping that waiter cannot drop the
   claim, journal, acceptance record, or work.
3. **B2.1-3 — restart and terminal reconciliation:** discover every nonterminal claim and exact
   durable cursor after restart; resume only through exact producer replay/reconciliation; make
   terminal closeout immutable and idempotent. A surviving process-memory world-service replay
   registry supports host/shell restart. A world-service restart or other producer unavailability
   leaves the durable claim nonterminal and unresolved. EOF, timeout, PID, helper, socket,
   readiness, process death, endpoint absence, caller presence, or local error never fabricates
   terminal truth.


#### B2.1-3 exact replay/startup allowlist correction

The existing B2.1-authorized supervisor/journal module, `orchestrator_world_dispatch.rs`, bounded
StateStore and HostSessionAuthority physical-capability wiring, transport API/client replay types,
world-service `service.rs`/`member_runtime.rs` producer plumbing, and focused tests remain unchanged.
The only additional B2.1-3 source authorization is:

- new bounded module `crates/world-service/src/runtime_replay.rs`;
- in `crates/world-service/src/handlers.rs`, only
  `handlers.rs::execute_stream_replay`;
- in `crates/world-service/src/lib.rs`, only the `runtime_replay` module declaration and the exact
  replay-route registration in `build_router`;
- in `crates/shell/src/repl/async_repl.rs`, only the import required for the canonical supervisor
  recovery entry point and `run_async_repl`, limited to calling that one entry point and retaining
  its returned observation tasks; focused colocated tests are allowed only if this is the existing
  test location for that startup hook.

Those file names are not wildcards. Handlers, router construction, and REPL startup may contain no
supervisor semantics, durable-record enumeration, journal interpretation, lifecycle decision, or
terminal inference. Producer replay must use exact acceptance-record/stream/cursor lookup, strict
frame identity/order/byte preservation, and explicit stream/frame/byte bounds. Stream enumeration,
fuzzy or session-only replay lookup, payload/request/prompt persistence, replay-generated
acceptance/claim/terminal/lifecycle/obligation/success truth, replay-across-world-service-restart
claims without durable producer proof, PID/liveness-based resolution, and early foreground return
are forbidden. B3.1 semantics, C1 obligation materialization, B4 outcomes, A1.2b transitions, and
new policy or world-capability semantics remain outside B2.1.

After all three B2.1 subpackets plus review-clean A1.2a, A1.2a-WB, A1.2a-S, B1/B2.1-R0, B3.2a, B3.2a-WA, and B1/B2.1-0,
**B1/B2.1 joint production closeout** proves both accepted families' no-gap handoff,
legacy-writer exclusion after acceptance, restart survival, exact terminal behavior, blocking
compatibility, the differential baseline, and supported doctor plus installed-product smoke. Actual
caller/waiter/guard drop is proven on the RunWorldTask and ephemeral accepted-task production
routes, while retained foreground-waiter drop is separately proved at the accepted-stream boundary.
B3.1 remains blocked until this joint closeout passes.

The exact B3.2a source allowlist in the Track B row below is extended by only these three
integration-test files:

- `crates/world-service/tests/member_runtime_world_placement_v1.rs`
- `crates/world-service/tests/streamed_execute_cancel_v1.rs`
- `crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`

Their authorization is limited to mechanical initialization of the new optional
`MemberDispatchRequestV1` launch-authority-proof field and compatibility regression proof.
Existing explicitly pre-activation or legacy request literals must initialize that field as
`None` while preserving every other fixture input, test name, assertion, expected outcome, and
expected error. A test that claims to exercise the authority-managed B3.2a route must instead use
the exact valid proof through the canonical production path; it cannot be made to pass with
`None`. No helper default that hides a missing Rust literal field, fixture-only authority,
alternate proof, alternate transport, side table, production-path change, or weakened assertion is
authorized. The row's prohibition on a wildcard or other integration site applies after this exact
three-file addition. That mechanical allowance did not itself complete B3.2a; the recorded
B3.2a/B3.2a-WA result in `05` now supplies the complete closeout.

The same Rust-literal rule requires one further exact mechanical extension and no semantic
extension. In `crates/shell/src/execution/orchestrator_world_dispatch.rs`, only
`build_run_world_task_transport_request` and `build_fork_world_worker_transport_request` may add
the new field as explicit `None`. In `crates/world-mac-lima/src/lib.rs`, only
`convert_member_dispatch` may add the field as explicit `None`. `#[serde(default)]` supplies the
absent optional field only while deserializing wire input; it does not initialize a Rust struct
literal. These three `None` initializers preserve every existing task, fork, and macOS-conversion
field and behavior and grant none of those paths retained-worker launch authority. They authorize
no task/fork policy, identity, lineage, lifecycle, transport-selection, world-placement, or macOS
policy change; no macOS authority-managed Spawn adoption; no helper default, side table,
environment carrier, alternate route, or hidden proof synthesis; and no edit to
`crates/world-api/src/lib.rs`, `Service::execute`, or `convert_member_dispatch_request`.
Authority-managed B3.2a Spawn still requires `Some(exact proof)` and fails closed before process
creation when that proof is absent, malformed, or mismatched. That mechanical allowance did not
itself complete B3.2a; the recorded result in `05` now does.

The widened internal structs and helper parameter require exactly seven further mechanical
production-symbol exceptions. In `crates/shell/src/execution/orchestrator_world_dispatch.rs`,
`fork_world_worker` and `continue_world_worker_fork_command_bootstrap_after_delivery` may only pass
explicit `None` for the optional retained-worker launch-authority context when calling the widened
stream helper. In `crates/shell/src/repl/async_repl.rs`,
`apply_greenfield_host_start_from_authority`, `prepare_hidden_owner_helper_runtime`,
`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
`prepare_fork_child_runtime_startup_for_descriptor`, and
`prepare_member_runtime_startup_for_descriptor` may only initialize, destructure, or preserve the
new optional retained-worker launch-authority proof/admission fields as `None`. Rust requires these
explicit field/parameter sites; the exceptions confer no retained-worker launch authority and
change no packet ownership, policy, lifecycle, transport, outcome, host-runtime, hidden-helper,
legacy-member, or fork behavior. Fork remains outside B3.2a authority adoption. Host Start retains
only its already-established host-session authority, and legacy member preparation remains the
pre-activation compatibility path distinct from
`prepare_member_runtime_startup_from_authority_registration`. The authority-registration preparer
is the only B3.2a authority-managed retained construction path and must supply
`Some(exact proof/admission context)`; it cannot fall back to legacy preparation or reinterpret a
generic `None` as compatibility. No other semantic or structural change in these seven symbols is
authorized. That mechanical allowance did not itself complete B3.2a; the recorded result in `05`
now does.

After A1.2a, A1.2a-WB, A1.2a-S, B1/B2.1-R0, B3.2a, and B3.2a-WA are independently review-clean, the exact future implementation allowlist for
B1/B2.1-0 action-scoped dispatch preparation and its colocated real-dispatcher proof is:

- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs`, limited to
  `resolve_follow_up_dispatch_authority_v1`'s active-task branch, the mechanical imports required
  by that branch, mechanical relocation of the existing shared pre-`match` legacy caller/world
  resolution unchanged into the retained-worker branch so the active branch no longer executes
  it, the existing colocated historical production-path test
  `dispatch_contract_adapter_follow_up_resolution_uses_authoritative_active_task_state`, and
  focused negative tests required to prove the same bounded contract

This docs-only correction authorizes those edits only after it is independently review-clean. The
three-file boundary is not a wildcard and does not authorize semantic
changes elsewhere in the already-reviewed B1/B2.1 implementation. A required edit outside it is
`CrossDocumentChangeRequired`. In particular, the adapter may consume but not modify the A1.1e
facade, authority schemas, receipt schema, supervisor schema, transport API, world-service,
transition protocol, obligations, policy enforcement, UAA, retained event taxonomy, spawn/fork
admission semantics, or unrelated fork/stop lifecycle behavior.

The `tool_invocation_contract.rs` exception is narrower than its file name. The function signature
and all callers remain unchanged. Its retained-worker branch, retained target selection, retained
error categories, and retained Inspect/Cancel/Stop, fork, continue-fork, and Spawn behavior are
frozen. The existing shared pre-`match` legacy caller/world resolution may only be moved into that
retained branch with identical inputs, order, errors, and results; this is structural partitioning,
not retained semantic authorization. The active-task branch alone may read the exact current HostSessionAuthority, immutable
acceptance record, and supervisor claim/cursor/terminal state through the existing bound capability
or authorized trusted open-and-bind boundary, exact-join them to the complete store/session/caller/
backend/world/task-or-active-run/acceptance/stream identity, and project that truth into the existing
tool-invocation result. It owns no durable state and performs no mutation or legacy active-task
synthesis. Missing, stale, ambiguous, cross-session, backend/world mismatched, task-reused,
incomplete, or conflicting truth fails closed without collapsing the existing distinct outcomes.
The approved upstream GitNexus impact is HIGH: 12 direct callers, four execution processes, and 14
impacted symbols. That approval applies only to this active-task branch; any signature, caller,
retained-branch, new HIGH/CRITICAL process-family, or additional production-symbol change is a stop.

For that closeout, "the differential baseline" means the monotonic transition gate in `04`,
recorded as `RG-DIFF-01` in `05`, not raw pass/fail totals. In particular, a historical failure
becoming a pass is accepted only with exact causal-resolution proof; it is neither automatic
success nor automatic regression. Any unresolved transition is `BaselineRegressionAmbiguous`,
keeps the joint closeout open, and leaves B3.1 blocked.

| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B0 — Durable runtime event identity and ordering carrier** | Make the runtime producer assign stable stream/frame/event/terminal identity and monotonic ordering before any host observer sees a frame. | Runtime event transport. | `01` invariants 3–6; `02` RuntimeEventTransport row; `04` runtime carrier contract; `05` `RG-EVENT-01`; internal toolbox transport framing/terminality. | ReceiptRegistry; Supervisor; MessagingProtocol; world-service execution producers. | Bounded `AgentEvent` fields in `crates/common/src/agent_events.rs`; bounded `ExecuteStreamFrame` fields in `crates/transport-api-types/src/lib.rs`; frame/event producers in `crates/world-service/src/{service,member_runtime}.rs`; carrier-only decode/match compatibility in exactly `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and `crates/shell/src/repl/async_repl.rs`; focused tests adjacent to those areas. | No receipt persistence, supervisor/journal or consumer-side duplicate rejection, semantic changes in the named shell files, retained-message classification, obligation work, unrelated transport redesign, or model-facing UX change. | Every accepted stream has one stable `stream_id`; every frame has a strictly monotonic producer sequence; every semantic event has a stable event ID/sequence; the terminal frame names the exact terminal event ID/sequence and is last. The producer never originates a gap, reorder, conflicting identity reuse, or post-terminal frame. Exact replay preserves identity and bytes; B2.1, not B0, accepts that replay as a no-op or rejects a conflict. | Producer clauses of `RG-EVENT-01`; prerequisite clauses of `RG-SUP-01` and `RG-MSG-01`; B0 carrier clause of `RG-OBS-01`. | Landed A1.1e exact authority facade; no A2/A3 prerequisite. | Consumer replay/deduplication/rejection, receipt acceptance, durable observation/restart, retained semantics, obligation materialization/cut, foreground early return, and every seam promotion. |
| **B1-3a/B1-3b — Receipt registry core (B1 not closed)** | Persist `WorldWorkAcceptanceRecordV1` for both ephemeral tasks and retained turns from an exact B0 pre-terminal acknowledgement; keep the immutable accepted anchor inspectable independently of foreground scope. | WorldWorkReceiptRegistry. | `02` ReceiptRegistry row; `04` acceptance context/record + active receipts + receipt acceptance; `05` receipt rows; tool-invocation async model. | HostSessionAuthority; RuntimeEventTransport; EffectivePolicyResolver; B2.1 supervisor handoff; RetainedWorkerRuntime; E2 policy commitments. | Exactly the canonical ownership, activated-store substrate, and carrier-constructor/test files enumerated in **B1 receipt core and activated-store allowlist** above; no implicit wildcard or additional integration site is authorized. | No host transition issuance/authentication/interpretation, generic activated-store writer, weakened legacy-writer exclusion, `ExecuteStreamFrame` change, accepted record before acknowledgement, active observation/journal ownership, foreground early return, retained lifecycle, obligation materialization, or UAA mediation. | Proposal, exact retry, task and retained acknowledgement, activated-store persistence, immutable accepted inspection, and legacy-writer rejection are review-clean. The proposal alone is never accepted truth. This exit authorizes B2.1 consumption but explicitly does not claim B1 production completion. | Acceptance clauses of `RG-RECEIPT-03` and `RG-BASE-02`; B1 acceptance clause of `RG-OBS-01`. | B0 and landed A1.1e; optional correlation does not require A1.2 and cannot authenticate, mint, verify, or interpret it. | Joint B1/B2.1 production closeout; E2 final receipt/cap commitments; A1.2 correlation supply; foreground early return; retained lifecycle; obligations; seam promotion. |
| **B2.1-1/B2.1-2/B2.1-3 — Supervisor handoff and durable observation** | Transfer sole post-acceptance stream ownership for both accepted families into a receipt-scoped durable claim and journal with no-gap handoff, restart-safe observation, exact replay handling, and terminal reconciliation while preserving blocking foreground behavior. | WorldWorkExecutionSupervisor. | `01` invariants 4–6; `02` supervisor row; `04` runtime carrier, receipt acceptance, and supervisor rules; `05` `RG-SUP-01`/`RG-SUP-02`/`RG-OBS-01`. | RuntimeEventTransport; ReceiptRegistry; WorldDispatchControl; SurfaceAdapter; MessagingProtocol; ObligationLedger; RetainedWorkerRuntime. | The existing bounded supervisor/journal module under `crates/shell/src/execution/`; `execution/orchestrator_world_dispatch.rs`; narrow construction/integration wiring in `agent_runtime/state_store.rs` to the separately scoped opaque supervisor physical capability in `agent_runtime/host_session_authority/store.rs`, `store/platform/transaction.rs`, and focused `store_tests.rs`; bounded transport API/client replay types; `crates/world-service/src/{service,member_runtime}.rs` producer plumbing; and focused claim/no-gap/dedupe/waiter/caller-drop/restart/terminal tests. The only additions are new bounded `crates/world-service/src/runtime_replay.rs`; only `handlers.rs::execute_stream_replay`; only the `runtime_replay` module declaration and exact replay route in `lib.rs::build_router`; and only the canonical-recovery import plus `run_async_repl` call/task retention and focused colocated tests in `crates/shell/src/repl/async_repl.rs`, exactly as frozen in **B2.1-3 exact replay/startup allowlist correction**. | No semantic supervisor logic in handlers, router, or REPL; no stream enumeration or fuzzy/session-only lookup; no payload/request/prompt persistence; no replay-generated acceptance, claim, terminal, lifecycle, obligation, or success truth; no replay-across-world-service-restart claim without durable producer proof; no PID/liveness resolution; no foreground early return, fixed-path active-task side table, generic activated-store writer, weakened legacy-writer rejection, runtime identity generation, retained-message taxonomy, obligation materialization, inbox/router behavior, B3.1 semantics, C1 materialization, final B4 outcomes, A1.2b transitions, policy/UAA work, new world-capability semantics, or terminal inference from local/process/transport state. | B2.1-1 durably exact-joins or creates the claim at acceptance, excludes ephemeral legacy registration, and transfers retained observation from the foreground; B2.1-2 journals exact canonical B0 frames/events before waiter delivery and rejects all non-exact order/replay; B2.1-3 recovers every nonterminal claim/cursor plus any exact accepted-but-unclaimed interrupted handoff and reconciles only from exact producer truth. Host/shell restart resumes only while exact producer replay survives; world-service restart or producer unavailability leaves valid claims durably unresolved and nonterminal. Fatal store/claim corruption fails startup closed, while an individually unavailable valid producer does not fabricate success or require ordinary startup failure. Waiter drop preserves work. Only exact B0 terminal identity closes immutable observation. | Consumer clauses of `RG-EVENT-01`; `RG-SUP-01` observation/receipt clauses; `RG-SUP-02`; `RG-RECEIPT-03`; `RG-RECEIPT-04`; B2.1 clauses of `RG-OBS-01`. | Review-clean B1-3a/B1-3b receipt core, B0, and landed A1.1e; B1 need not yet be production-closed. | Joint B1/B2.1 production closeout; model-facing early return (`B2.2`); retained semantic envelope (`B3.1`); C1 cut; final cancel semantics; seam promotion. |
| **B1/B2.1-R0 — Canonical retained-target protocol prerequisite** | Add the smallest durable registration protocol needed by later retained full-dispatcher proof: HostSessionAuthority exact-joins or reserves the retained-creation issuer request ID, validates the caller-fixed participant plan, and fixes every registration/object identity, commitment, expected-authority value, policy, world, and timestamp before object publication; RetainedWorkerRuntime publishes the exact reserved descriptor/resume-handle/retained-worker graph; HostSessionAuthority atomically appends exactly that participant to lineage, adds its worker ref, advances authority revision, records `RetainedWorkerAuthorityRegistrationV1`, and marks the request Applied; RetainedWorkerRuntime exact-resolves the bound target. R0 intentionally has no production ingress caller. | RetainedWorkerRuntime owns object construction and exact target read; HostSessionAuthority alone owns the replay-stable request reservation/index, revisioned lineage/ref mutation, request completion, and proof. | `02` Case B audit and RetainedWorkerRuntime row; `04` field-source table, registration proof, and bounded adapter; `05` `RegressionMasked` stop. | A1.2a-S production-created bound authority; B3.2a production creation/admission bridge; later remaining B3.2 lifecycle. | Exactly `crates/shell/src/execution/agent_runtime/retained_worker_runtime.rs`, `crates/shell/src/execution/agent_runtime/mod.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/mod.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store_schema.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store_tests.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/object_persistence.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/reachability.rs`, and `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`; focused tests remain in the two named test-bearing files. The landed eight-file source subset and proof are recorded in `05`; unused allowed files remain unchanged. | No production ingress integration, legacy participant writer, or test-only fixture; no messaging, accepted-turn observation, active-turn lifecycle, park/cancel/stop/fork, live-count/admission semantics, B3.2 expansion, Start/Attach/Resume transition, active-caller/posture/workspace/world/policy/origin/unrelated-ref change, Pending-start expected-revision rewrite, or second authority writer. | The reservation CAS precedes every object write and exact-joins only the same issuer request and bytes. It validates that the participant equals the caller-supplied plan; it does not allocate a retry-local participant. The final atomic root CAS validates the reserved complete object graph, appends only the new lineage member/ref, persists the immutable non-transition proof that `resolve_exact` accepts, and marks the request Applied. Crash before reservation has no mutation; crash after reservation/object publication and lost response after final CAS rejoin the fixed request/proof/result. If Start startup ownership is Pending, the proof is one unique contiguous ancestry link from the original application revision; it is not startup acceptance. Stale/conflicting revision or duplicate/substituted/absent/cross-scope object fails closed. The resulting exact target binds store/session/lineage/world/backend/role without compatibility fallback. This packet is component proof only; B3.2a now invokes it from a durable per-session registration head and supplies the production evidence recorded in `05`. | Prerequisite retained-target clauses of `RG-RECEIPT-03`, `RG-OBS-01`, and `RG-DIFF-01`. | Review-clean A1.2a-S. It consumes no B1 receipt or B2.1 supervisor datum. | B1/B2.1-0, joint closeout, complete retained admission/lifecycle, messaging, B3.1/remaining B3.2, and every seam promotion. |
| **B3.2a — Retained creation/admission bridge prerequisite** | Connect R0 to both real `SpawnWorldWorker` production adapters and add only canonical creation/admission truth. A locked durable CAS fingerprints the complete validated Spawn request and authority/runtime/policy plan with a RetainedWorkerRuntime-owned admission key, counts all nonterminal slots, enforces the cap, and fixes participant/bootstrap-run identities. One durable per-session registration head serializes R0 revision selection. Queued slots with no head are valid; only exact re-presentation of the complete lowest-sequence request may acquire it, with no persisted request/prompt/payload preimage, no digest-only promotion, no overtaking, and no inferred stealing. Both the direct dispatcher and live internal-toolbox retained-runtime path carry the same exact R0/admission proof through the real transport-api `Service::execute_stream` member-dispatch route before transport launch. | RetainedWorkerRuntime owns the admission key/commitment, admission/routability/terminal state, and the live count. HostSessionAuthority owns only R0 registration and none of the admission-key lifecycle. WorldDispatchControl and the live SurfaceAdapter consume both without owning them; world-service equality-validates the typed launch proof but does not mint authority or admission truth. | `02` Case B audit and RetainedWorkerRuntime/WorldDispatchControl rows; `04` R0 proof, admission key/fingerprint, admission state machine, proof carrier, and two production orders; `05` `RegressionMasked` stop and `RG-ADMISSION-01`. | R0; A1.2a-S bound authority; B0 Registered/terminal truth; remaining B3.2 lifecycle. | Exactly `crates/shell/src/execution/agent_runtime/retained_worker_runtime.rs`, `crates/shell/src/execution/agent_runtime/mod.rs`, and opaque fixed-registry/key physical-capability wiring only in `crates/shell/src/execution/agent_runtime/host_session_authority/store.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`, `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`, and `crates/shell/src/execution/agent_runtime/host_session_authority/store_tests.rs`; in `crates/shell/src/execution/orchestrator_world_dispatch.rs`, only `dispatch_orchestrator_world_request`, new `WorldDispatchSteeringInput` and `WorldDispatchConcurrencyInput`, new `prepare_authority_bound_spawn_world_worker` and `spawn_prepared_world_worker`, `PreparedSpawnWorldWorkerBootstrap`, `prepare_spawn_world_worker_bootstrap`, `enforce_world_dispatch_steering_policy`, `validate_authoritative_session_boundary`, `validate_authoritative_world_binding_for_steering`, `validate_authoritative_world_binding`, `acquire_world_dispatch_concurrency_guard`, `spawn_world_worker`, `build_spawn_world_worker_transport_request`, `execute_spawn_world_worker_stream`, mechanical explicit-`None` initialization only in `build_run_world_task_transport_request` and `build_fork_world_worker_transport_request`, mechanical explicit-`None` helper-argument propagation only in `fork_world_worker` and `continue_world_worker_fork_command_bootstrap_after_delivery`, and colocated tests; in `crates/shell/src/repl/async_repl.rs`, only `PreparedAgentRuntime`, `handle_internal_toolbox_world_dispatch_request`, new `prepare_member_runtime_startup_from_authority_registration`, `build_member_dispatch_transport_request`, `start_remote_member_runtime_with_prepared`, new `validate_remote_retained_start_authority_proof`, mechanical `None` initialization/preservation only in `apply_greenfield_host_start_from_authority`, `prepare_hidden_owner_helper_runtime`, `start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`, `prepare_fork_child_runtime_startup_for_descriptor`, and `prepare_member_runtime_startup_for_descriptor`, and colocated tests; in `crates/shell/src/execution/routing/dispatch/world_ops.rs`, only `MemberDispatchTransportRequest`, `build_member_dispatch_payload`, and colocated tests; in `crates/transport-api-types/src/lib.rs`, only `RetainedWorkerAdmissionCommitmentCarrierV1`, `RetainedWorkerLaunchAuthorityProofV1`, their optional `MemberDispatchRequestV1` carrier field/strict validation, and colocated tests; in `crates/world-service/src/service.rs`, only the `Service::execute_stream` member-dispatch branch and colocated direct carrier pass-through tests; in `crates/world-service/src/member_runtime.rs`, only `MemberRuntimeManager::launch`, new `validate_retained_worker_launch_authority_proof`, and colocated tests; and in `crates/world-mac-lima/src/lib.rs`, only mechanical explicit-`None` initialization in `convert_member_dispatch`. `crates/world-api/src/lib.rs`, `Service::execute`, and `convert_member_dispatch_request` are explicitly outside this route and allowlist. No wildcard or other integration site is authorized. | No generic prepared-dispatch rewrite, test-only target, compatibility/HSA-ref live count, HostSessionAuthority admission key/domain, permissive proof or legacy-writer fallback, action/backend/session/world policy change, Spawn outcome/event change, accepted-turn receipt/supervision, messaging, provider semantics, park/cancel/stop/fork, admission cancellation or abandonment resolution, final lifecycle reconciliation, macOS authority-managed Spawn adoption, macOS world-placement/policy change, world-api change, B3.1, obligation, early-return, or A1.2b work. | Direct `dispatch_orchestrator_world_request` routes only Spawn through the authority-bound bridge. Live `handle_internal_toolbox_world_dispatch_request` uses the same bridge and authority-bound runtime preparer, never its retry-local participant allocator, while retaining its runtime handle/map. The named steering functions consume narrow inputs with identical denials. Every fingerprint-dependent join and state advance receives the complete canonical request input, and the separate admission key remains verification-capable for every retained record. Only one per-session head fixes current authority; reconciling its applied proof advances only that record and releases the head. A separate exact retry of the earliest queued request alone may acquire the next head after complete fingerprint and R0-only ancestry verification. Both authority-managed Spawn transport builders require `Some(exact proof)` on the A1.2a-S path; RunWorldTask, ForkWorldWorker, fork continuation, Host Start, hidden-helper construction, legacy member preparation, and the macOS world-api conversion explicitly carry `None` and gain no retained-worker launch authority. The authority-registration member preparer remains the only B3.2a managed construction and cannot fall back to legacy preparation. `Service::execute_stream` first completes strict proof validation and B3.2a-WA durable exact same-world ownership adoption, then passes the unchanged transport-api member dispatch to `MemberRuntimeManager::launch` for launch-boundary revalidation; missing/mismatched authority-managed proof or failed adoption rejects before member creation. Real-path tests independently drive both Spawn adapters through that exact carrier and launch validator. Exact B0 Registered makes Routable; the remaining body is observer-owned. Only exact B0 terminal truth terminalizes; EOF, Error, drop, observer loss, PID/helper/socket/process posture, or restart uncertainty remains live/nonroutable. The live activated path performs no legacy participant/snapshot write. Pre-activation compatibility is unchanged. | Prerequisite retained-creation/admission clauses of `RG-BASE-04`, `RG-RECEIPT-03`, `RG-OBS-01`, `RG-DIFF-01`, and the exact-retry-only B3.2a clauses of `RG-ADMISSION-01`. | Review-clean A1.2a-S, review-clean R0, and B0. It consumes no B1/B2.1-core datum. | Durable abandoned-admission resolution and restart reconciliation in remaining B3.2; the user/tool-facing exact inspect/cancel surface and distinct outcomes in B4; B1/B2.1-0, joint closeout, accepted-turn lifecycle, messaging, B3.1, and every seam promotion. |
| **B3.2a-WA — Exact bound-world ownership adoption prerequisite** | Authority-managed retained Spawn realizes the exact HSA-bound generic world as the shared-session owner without changing world ID/generation or creating an alternate world. `ExactBoundWorldOwnershipAdoptionV1` is an internal operation contract, not a world-api field or persisted wire schema. | HostSessionAuthority owns the exact durable session binding; RetainedWorkerRuntime owns admission/R0/transport-claim truth; runtime-family/world backend owns physical realization and ownership metadata only. | `01` exact bound-world invariant; `02` B3.2a-WA seam addendum; `04` `ExactBoundWorldOwnershipAdoptionV1`; `05` `RG-WORLD-ADOPT-01`. | B3.2a typed proof and transport claim; world-service member placement; Linux shared-world metadata publication; compatibility `None` and ordinary world execution. | Exactly `crates/world-service/src/service.rs`, `crates/world/src/lib.rs`, and `crates/world/src/session.rs`, plus focused colocated or existing integration tests needed for this contract. | No shell/HSA rebinding, world-api or schema-version change, side table, alternate world, policy/project/spec relaxation, request/prompt persistence, PID/helper/liveness authority, compatibility-path semantic change, filesystem/network/caging/discovery/write-sync change, platform promotion, member-lifecycle recovery, resend, or claim that adoption proves launch. | After the authority-managed proof is validated and before member process creation, the backend exact-joins session/world/generation/participant/policy/project/spec and current generic metadata, then atomically/durably publishes `GenericExactBoundWorld -> SharedSessionOwnerExactBoundWorld` under trusted locking/temp-file/fsync/rename/directory durability. Exact retry joins without rewrite; conflicting, missing, corrupt, ambiguous, partially published, or unsupported evidence fails closed with no alternate world or mutation. Adoption preserves HSA and RetainedWorkerRuntime bytes, does not prove transport/Registered/routability/terminal success, and compatibility `None` remains unchanged. Linux doctor, ordinary world execution, and bounded authority-managed Spawn smoke pass; the result is review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. | `RG-WORLD-ADOPT-01`; applicable world-placement clauses of `RG-BASE-02`, `RG-BASE-04`, `RG-OBS-01`, and `RG-DIFF-01`. | Review-clean A1.2a-S, R0, B0, and the B3.2a authority-managed proof/claim path. | B1/B2.1-0, non-Linux adoption, abandoned-claim recovery, and every seam promotion. |
| **B1/B2.1-0 — Action-scoped dispatch preparation prerequisite** | Consume the A1.2a-S production-bound current-authority capability, the B1/B2.1-R0 exact retained-target read, B3.2a routability truth, B3.2a-WA exact physical ownership truth, and the recovered B1 receipt/B2.1 supervisor truth; replace missing legacy session/caller/target inputs for RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait; make the existing active-task tool adapter read the same joined truth; and remove `live_retained_worker_count` from only that B-owned prepared view. This is the first branch join. | WorldDispatchControl consumes HostSessionAuthority, RetainedWorkerRuntime, ReceiptRegistry, and Supervisor truth without owning any of them; the tool adapter is a read-only projection. | `02` Case B audit; `04` field-source table and bounded adapter; `05` `RegressionMasked` stop. | A1.2a-S; B1/B2.1-R0; B3.2a; B3.2a-WA; ReceiptRegistry; Supervisor; legacy continue-fork/retained-control/fork compatibility. | After independent docs review, exactly `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/execution/agent_runtime/state_store.rs`, and only `crates/shell/src/execution/agent_runtime/tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1`'s active-task branch, its mechanical imports, mechanical branch partitioning that moves the existing shared legacy caller/world lookup unchanged into the retained branch, its existing historical production-route test, and focused negative tests. | No signature or caller change; no retained behavior/error/order/result change; no test-only authority writer; no authority creation/transition in B code; no compatibility synthesis of caller or target lifecycle; no `retained_worker_refs`-as-live substitution; no `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, further spawn/fork steering, or retained-lifecycle change; no receipt/supervisor schema change. | Full dispatcher and real host tool-invocation route reach only the named B-owned paths from production-created current authority and R0+B3.2a canonical retained-target/routability truth after exact bound-world ownership adoption; authority/receipt/supervisor access uses the bound capability; missing, stale, ambiguous, reused, incomplete, or conflicting scope fails closed with distinct outcomes preserved. Continue-fork, retained Inspect/Cancel/Stop, fork behavior/errors, and the reviewed B3.2a/B3.2a-WA Spawn bridge are unchanged. The authorized HIGH impact remains bounded to 12 direct callers, four known execution processes, and 14 impacted symbols. This prerequisite is review-clean through `83101dcb`. | Prerequisite clauses of `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`, `RG-OBS-01`, and `RG-DIFF-01`. | Review-clean A1.2a-S, R0, B3.2a, B3.2a-WA, and B1/B2.1 cores. | Joint closeout, B3.1, remaining retained lifecycle, A1.2b, and every seam promotion. |
| **B1/B2.1 — Joint production integration closeout** | Prove the real ephemeral and retained accepted paths proceed from immutable B1 acceptance into B2.1 supervision without any legacy-writer attempt or observation gap. Use B1/B2.1-0 only for RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait. | HostSessionAuthority/A1.2a own exact current authority, A1.2a-WB owns only its Host/world-binding write/read correction, and A1.2a-S owns only bounded Start adoption; RetainedWorkerRuntime owns R0 object construction/read plus B3.2a admission/routability while HostSessionAuthority owns only R0's atomic lineage/ref mutation and proof; the runtime-family/Linux backend owns only B3.2a-WA physical realization and ownership metadata; ReceiptRegistry and Supervisor retain distinct accepted-work and observation ownership; WorldDispatchControl only validates and joins them. | All B1/B2.1 sections in `01`–`05`; production Spawn setup through both adapters, world-service proof validation, B3.2a-WA exact same-world ownership adoption, live acceptance, full-dispatcher named-path inspect/wait/cancel, drop, restart, and reconciliation. Retained control cases remain full-dispatcher differential inputs but not failure-to-pass closeout proof. | RuntimeEventTransport; A1.2a-S bound authority; B1/B2.1-R0 exact retained-target read; B3.2a creation/admission truth; B3.2a-WA physical ownership truth; compatibility callers. | After prerequisite review, new joint-closeout implementation edits are limited to `crates/shell/src/execution/orchestrator_world_dispatch.rs` and `crates/shell/src/execution/agent_runtime/state_store.rs`, including colocated focused tests; no wildcard. The already-reviewed B1/B2.1-0 tool adapter remains within its prerequisite allowlist and receives no additional joint-closeout authorization. Already-reviewed B1/B2.1, A1.2a-WB, A1.2a-S, R0, B3.2a, and B3.2a-WA code remains within its prior packet allowlists. Closeout rows in this six-file pack only after proof. | No authority creation in closeout code; no transition issuance/application/correlation; no live-retained truth fabrication; no `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, additional spawn/fork, or retained-lifecycle change; no B3.1, C1, A1.2b, B2.2, B4 final outcomes, policy, UAA, retained semantic, obligation, or early-return work. | A1.2a creates exact greenfield authority, A1.2a-WB freezes the exact placement/binding matrix without changing placement, and A1.2a-S adopts it on the internal host/toolbox path with zero activated legacy writes. R0 creates/proves/exact-resolves the retained target, B3.2a creates it through both real Spawn adapters with typed world-service proof validation and exact routability/live admission, and B3.2a-WA durably adopts the exact HSA-bound world before member creation without changing its ID/generation or creating an alternate world. The named action-scoped view excludes `live_retained_worker_count`; both accepted families prove restart survival and exact terminal behavior, while actual caller/waiter/guard drop is proven on the RunWorldTask and ephemeral accepted-task production routes and retained foreground-waiter drop remains separately proved at the accepted-stream boundary. No accepted work is followed by legacy active-task registration or activated-store rejection; ephemeral accepted-task Inspect/Wait/Cancel consumes exact receipt/supervisor truth; terminality is exact. Retained Inspect/Cancel/Stop stays on the unchanged full dispatcher and may only remain `FailToSameFailure` until remaining B3.2/B4; a direct resolver substitution is still `RegressionMasked`. Complete differential, doctor/smoke, and fresh review gates pass. | Applicable B1 and B2.1 gates, production legacy-writer exclusion, blocking compatibility, caller-drop/restart, `RG-OBS-01` acceptance/journal joins, `RG-WORLD-ADOPT-01`, and `RG-DIFF-01`. | Review-clean B1/B2.1 cores, A1.2a, A1.2a-WB, A1.2a-S, R0, B3.2a, B3.2a-WA, and B1/B2.1-0. | B3.1 and every later packet. |

### B1/B2.1-0 recorded prerequisite result

The recovered receipt, supervisor, replay/startup, and authority-store correction commits are
`6436289f`, `c519024b`, `de727091`, and `717579b0`; the action-scoped dispatch prerequisite is
`83101dcb`. Its exact fourteen historical production-path tests retain their names and routes, the
shell differential is `981 passed / 160 failed / 0 ignored` to
`1054 passed / 149 failed / 0 ignored`, and every one of the eleven `FailToPass` transitions is
causally attributed to the real dispatcher or tool-to-dispatch correction. All forbidden
transition classes are zero and all 149 retained normalized failure signatures are unchanged.
This result closes only B1/B2.1-0 at that historical checkpoint. The later joint production
integration closeout is now recorded below; B3.1 and C1 are complete on the bound Tuesday,
August 4, 2026 candidate, A1.2b is complete on that same bound candidate as the internal durable
successor/post-turn protocol only, and no seam is promoted.


## B1/B2.1 joint production closeout recorded result

At bound source `f37943eb917285a044c5e12a05b481572c8d0a09` /
`54ae7b2a2d467a568b575665b99dcb94ef2893a2`, the joint production integration closeout recorded that
the frozen product/test source already satisfied the named accepted-family handoff without further
product or test edits. Real `RunWorldTask` and ordinary retained `ContinueWorldWorker` preserve the
immutable B1 acceptance anchor, exact-join the canonical receipt-scoped supervisor claim before
another frame is read, journal canonical B0 bytes before waiter delivery, resume only from the
exact durable cursor and producer replay after supported host/shell restart, remain durably
unresolved when replay is unavailable, and close only from exact B0 terminal truth. `RunWorldTask`
plus the named ephemeral Inspect/Wait/Cancel paths prove actual caller/waiter/guard drop survival
on the production-owned path and consume the same receipt/supervisor truth; ordinary retained
`ContinueWorldWorker` proves the full dispatcher handoff and separately preserves
foreground-waiter-drop survival at the accepted-stream boundary. No accepted production path
attempts legacy active-task registration or another legacy writer.

Broad proof remained monotonic through both `make shell-lib-wall` and `make shell-lib-wall-serial`
at the accepted `1322 discovered / 1274 passed / 48 failed / 0 ignored` baseline with failure-name
SHA-256 `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`. Focused dispatcher,
receipt/supervisor, restart/replay, active-task, retained-continue, and real tool-route tests are
green; supported Linux doctor plus installed-product smoke is preserved in
[`review-control/b1-b2-1-joint-closeout-linux-evidence.md`](../review-control/b1-b2-1-joint-closeout-linux-evidence.md);
the differential record is in
[`review-control/b1-b2-1-joint-closeout-differential-evidence.json`](../review-control/b1-b2-1-joint-closeout-differential-evidence.json).
The closeout changes only this control-pack/review-control surface, promotes no seam, and makes
B3.1 dependency-ready.
