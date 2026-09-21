# Substrate → Collider B01–B05 contract handoff draft

**Status:** Proposal for review, 2026-09-20. Confirmed product intent is recorded; this document's new contract rules are not yet owner-approved or wire-final. No runtime implementation, existing-owner rewrite, control-plane cutover, or Collider adoption is authorized.
**Start with:** [B03 Lane/World binding proposal](lane-world-binding.md).
**Objective:** Let Collider plan against coherent Substrate-owned operations without mistaking proposed semantics, existing source, supported exposure, and runtime proof for one status.

## 1. Scope and source binding

The [core architecture dossier](../../../SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md) is the source of confirmed intent; it is not replaced here. The [historical runtime dossier](../../../RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md) remains unchanged, including R1–R34.

Working assumption: retain the existing Collider planning scenario—one exact Linux ephemeral World task in Control Plane/Lane views plus Workspace dependency requirement read/add/remove—as the bounded handoff target. That selection comes from the consumer plan, not new user ratification of an implementation dispatch. It does not define the complete Substrate architecture or defer V1 per-World overrides.

This is a semantic proof target, not a selected executable runtime profile. B01's readiness gate below must be satisfied before treating that scenario as runnable or safe.

| Input | Bound identity / qualification |
|---|---|
| Core dossier | SHA-256 `07d956b9936569dd8f7fcb12b8b6500b49b91187d91167cb42453413b9fdbe15`; current 335-line revision includes SCA-12. The initial 314-line dossier was read in full; the research follow-up delta was reviewed in place. |
| Historical dossier | SHA-256 `d3538ad11d215ac801ee3ba6c21605d98d4c376ef6033c174fd485765e1dcfca`; historical findings, not new runtime proof. |
| Local authoring checkout | `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`, branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`, HEAD `a8b1a44ed2f9c33f6963eb552f9da05c198bce6a`, tree `766709683f93fa5d151da1db74d297160324cba7`. Initially only the two dossiers were untracked; no tracked/staged changes. |
| Authoritative document inspection | SSH host `spenser-linux-codex`, `/home/spenser/__Active_code/substrate`, same branch, HEAD `088f62d81c182b92cac404983b8bf512becc829c`, tree `b40da914c1e5d73fc79f13c0946f6af9c97455d1`, clean when inspected. Eleven referenced owner/reference documents matched their local SHA-256 values. No source execution, tests, installed-runtime audit, W-worktree audit, or separate authority-branch refresh. |
| Collider consumer | `/Users/spensermcconnell/__Active_Code/atomize-hq/collider`, branch `spike/agentkit-controller-correction`, HEAD `75631d432099fa18f1a320fc93863207ac7bcd2b`; only its pre-existing bundle ZIP was untracked. Consumer documents were read, not edited; fixture/native results were not rerun. |

Consumer inputs: [boundary plan](../../../../collider/docs/agent-native/controller-ownership-plan/substrate-accepted-work-boundary-plan.md), [lifecycle mapping](../../../../collider/docs/agent-native/controller-ownership-plan/controller-owner-lifecycle-contract.md), [glossary](../../../../collider/docs/agent-native/canonical/00-collider-v1-glossary.md), and [foundations §11](../../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md#11-context-and-continuity-model). Cross-repository links assume sibling checkouts; pin the named Collider revision when handing this off elsewhere.

Initial authoring fence was this draft and the companion B03 draft. The 2026-09-20 user-authorized research follow-up additionally updates the core dossier, adds one research brief, and links it from the backlog. Existing canonical runtime owners, the historical dossier, source/tests, and Collider files remain unchanged. New operation names and result categories below describe meanings, not existing exports, Rust structs, HTTP routes, or a selected transport. No generic facade subsystem is required merely to provide these operations.

## 2. Shared boundary rules proposed for owner review

- Authenticate the real human/local caller or approved delegated caller. Workspace/Lane/World/profile fields supplied by a client are references, not authority. Read, reconciliation, and replay require current authorization too.
- Preserve the [existing authority map](../../../llm-last-mile/runtime-refactor/architecture/authority-map.md): HSA for session authority; dispatch for control orchestration; registry for acceptance; Supervisor for observation/terminal truth; policy owners for permissions; World/config owners for environment state. A UI, transport, or lane orchestrator is not a substitute durable writer.
- Expose capability support, current authorization, and availability/readiness separately. Unknown/unsupported is not false success or a frontend simulation of owner behavior.
- Maintain separate client request, accepted task/turn, logical worker, orchestration session, Lane, World realization, cancellation request, cursor, and UI projection identities.
- Retain strict schema/reference validation and version explicitly. Do not silently mutate existing V1 records or create compatibility-only execution paths to bypass missing contracts.
- No observer closure, timeout, missing tail, cancellation ACK, or process liveness manufactures acceptance or terminality. No retry after uncertainty creates new work automatically.
- Return structured outcomes and source references; separate human diagnostics from machine replies. Redact secrets and do not expose internal capabilities or provider credentials.
- New proposals below are contract-review candidates. Existing safety contracts remain controlling until reconciled; the draft does not promote any seam to `ContractCorrectAndProven`.

## 3. B01 — Authenticated submission and exact reconciliation

**Existing owners:** dispatch + HSA + Receipt Registry + E2 commitment + B2.2 receipt construction. [Receipt/supervision family](../../../llm-last-mile/runtime-refactor/seams/runtime-event-receipt-supervision-and-retained-runtime.md), [ephemeral receipt](../../../llm-last-mile/runtime-refactor/contracts/active-ephemeral-task-receipt-v1.md), [B2.2 owner](../../../llm-last-mile/runtime-refactor/slices/b2-2-foreground-receipt-return.md).

**Runtime-profile readiness gate:** before running the consumer proof, name its supported ingress, runtime family/backend and version, action/tool capabilities, credential posture, and exact World execution route. Bind these to current source and applicable security-owner requirements, then prove the selected path on installed Linux. No such profile is selected or certified by this draft. Existing B1/B2.1/B2.2 receipt behavior is real bounded functionality; a receipt test or in-World process placement is not proof of credential isolation or complete side-effect mediation.

The [runtime dossier](../../../RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md) records copied Codex configuration/auth debt (R16), incomplete E3 producer/custody/admission/retained execution and proof (R23–R28), and absent/partial D1 envelope, D2 brokerage and D3 joined proof (R29–R30). At the reviewed Linux `088f62d…` source binding, [world-service](../../../crates/world-service/src/service.rs) rejects V2 member launch as unimplemented. Credential-bearing E3/V2/retained routes therefore remain design-only here; the inherited copy bridge is not a substitute for their security contract. Any selected UAA shell/edit/write/tool/process/network/native side-effect channel must have applicable broker/enforcement evidence or be disabled. If that cannot be established for a proposed profile, keep it design-only rather than count it as the narrow proof.

This gate does not declare all ephemeral execution absent or require completion of every E3/D1/D2/D3 feature before any consumer experiment. A narrower supported profile may be proposed with its own explicit limits and evidence; it must not silently reduce required security or imply proof of excluded routes.

### Proposed operation meanings

| Operation | Request meaning | Result meaning |
|---|---|---|
| Submit ephemeral intent | Versioned caller request key; authenticated caller/delegation; Workspace; B03 concrete destination; typed action/prompt and B04 selected-reference inputs; permitted execution requirements. | Pending with a recoverable request reference and explicit admission/composition phase; conclusive pre-submission refusal; or exact accepted work/receipt and binding. Transport uncertainty is an unknown outcome, not an owner terminal result. |
| Reconcile request | Same authenticated request namespace/key, optionally its owner-issued correlation reference. | Exact known request disposition and original acceptance material if accepted; not-recorded/expired/unavailable distinctions where supported. Never resubmit from a read. |

Namespace a request key by authenticated principal, Workspace, and operation. Bind it to canonical semantic inputs before uncertain side effects. Include destination and selected-reference versions in equality; exclude incidental UI display/transport values. Canonicalization must be defined by the owner, not by hashing raw frontend messages or exposing secret-derived request material.

Same key and same immutable inputs joins the same request; reuse with changed inputs conflicts. Different keys with identical prompt text are distinct intents. A pending request may progress; retry does not mean freezing its first observed pending response forever. Once accepted, original immutable acceptance/policy/binding material is returned unchanged, alongside separately labeled current observations.

Do not promise exactly-once execution. An ACK-loss path must reconcile B1/runtime evidence and the original request, including a partially published receipt. Do not mark it refused merely because foreground reporting failed. Current read authorization and historical policy evidence are distinct; neither recomputes the other.

**Recommended retention rule:** publish an explicit retry/reconciliation horizon and do not silently recycle expired request keys. An expired/unknown-history result must not launch new work; a caller needs an explicit new intent. Numeric retention, durable tombstones, namespace reuse prevention, and crash-stable storage placement remain owner decisions before this is wire-final.

**Hierarchy:** a direct Workspace-level redirection is a separate authorized control operation, not a changed-payload retry of the original request. Preserve causal links and inform the Lane coordinator. This draft does not define the redirection API or permit an AgentKit message to become that authority.

### 3.1 Request admission and consumer acceptance are separate

The following is a proposed result algebra, not exported enum names or a new runtime lifecycle. Existing owners remain authoritative; an admission correlation does not create a task, and a projection failure does not undo B1 acceptance.

| Result meaning | Required evidence / client behavior |
|---|---|
| `Pending(Preparing)` | Durable request correlation exists; submission has not crossed the guarded runtime boundary. Not accepted work. Only the admitted owner path may advance it after current authorization/preconditions pass. |
| `Pending(SubmissionUnresolved)` | Submission may have occurred; exact acknowledgement/acceptance is not yet resolved. Preserve the same request and proposed work identity. Never release it for a fresh launch because of timeout. |
| `Pending(AcceptedMaterialIncomplete)` | Exact B1 acceptance is known, but B2.1/E2/B2.2 or B03 consumer joins are not yet complete. Report that distinction without exposing private records; do not call it a pre-launch failure or return a fabricated receipt. |
| `Accepted` | Exact authenticated original receipt material and B03 binding are complete and durably recoverable. Return that immutable composition; current observations are separate. Work may already be terminal. |
| `RefusedBeforeSubmission` | Conclusive owner evidence that this recorded attempt was rejected before submission; no runtime acceptance is possible for it. Reason is privacy-filtered. Changed authorization or destination requires a new explicit intent, not mutation of this result. |
| `KeyConflict` | This key is already bound to different canonical inputs. The original request is unchanged and may still be running; conflict says nothing about its terminality. |
| `NotRecorded` / `ExpiredHistory` / `Unavailable` | Read coverage is distinguished from absence. None authorizes resubmission or implies no work ran. `NotRecorded` is only a coherent lookup observation, not a reservation against a concurrently arriving submit. |

Authentication/authorization failure, invalid envelope, and unsupported version are access/validation failures, not dispositions of an inaccessible original request. Transport loss is caller uncertainty. Neither may be collapsed into `RefusedBeforeSubmission`. After submission, only exact owner evidence may establish work state; do not force runtime launch failure into a pre-submission category. If recovery cannot establish an outcome, keep it unresolved.

`ReconcileRequest` is read-only: it may report newer owner recovery evidence but never resubmit, repair indexes, publish missing joins, or reconstruct historical policy/binding. Recovery/publication belongs to an explicitly authorized existing-owner path, outside the read. The E2-RM projection remains non-reconciling and consumes historical bytes, not current policy or supervisor revisions.

### 3.2 Immutable request identity and the admission boundary

Proposed responsibility-level inputs, not wire types:

```text
RequestKey = (authenticated_principal_ref, workspace_ref, operation_kind, client_key)
Submit = (schema_version, client_key, workspace_ref, destination_expectation,
          action_and_payload, selected_context_versions, execution_requirements)
Reconcile = (schema_version, workspace_ref, operation_kind, client_key)
```

The principal is derived by ingress, never trusted from a request field. The owner resolves any execution-session/caller/delegation references and validates them against [B03's HSA bridge](lane-world-binding.md#45-proposed-receipt-composition-and-the-hsa-bridge). Neither UI connection ID nor orchestrator episode ID namespaces the user request. Delegated principal/effective-actor equivalence, permitted operation kinds, and exact reference encodings must be specified before this is callable; rotating credentials must not silently change request ownership or confer access.

Canonical equality includes the exact destination and association revision, action/payload, selected context revisions and required/optional semantics, caller-supplied execution constraints, and any caller-specified delegation selector. Owner-generated proposal/work IDs are allocated once and reused, not new caller equality inputs. Derived policy/configuration/context evidence is bound at admission with its owning rules; it is not recomputed to redefine an accepted request on retry. No prompt/payload preimage or raw secret-derived digest is added to durable correlation records. The owner must specify a protected equality commitment and privacy-safe diagnostics; this draft does not invent a hash algorithm or persistence format.

Recommended admission ordering:

1. Authenticate ingress and authorize access to the request namespace. Resolve a recorded key before considering a new launch. Same-key equality and current read permission govern return of an existing result; a changed current World association or parent policy must not rewrite its accepted history.
2. For an unrecorded, admissible key, serialize key binding and durable correlation to the existing proposed B1 identity before uncertain effects. Concurrent identical submissions join it; conflicting inputs cannot allocate another proposal. This extends the owning admission path, not a second acceptance registry. Exact placement/atomicity remain a design gate.
3. Before submission, validate B03 destination and HSA authority, current policy/capabilities, and required context/configuration preconditions. Bind the admission to guarded owner revisions so a change cannot occur between checking and launch unnoticed. Preparation is not a lasting permission grant. If not submitted, stale preconditions produce the conclusive refusal; never retarget to latest automatically.
4. Keep request and association custody through ambiguous submission. Runtime acknowledgement establishes B1 truth; B2.1 observation, E2 immutable policy material, B2.2 receipt return, and B03 historical binding must exact-join before reporting the complete consumer `Accepted` result. This is a partial ordering of owner obligations, not a replacement for their call protocols.
5. Retry/restart joins the same evidence. Accepted retries use authenticated original material despite policy/association drift, subject to current read authorization. Pre-submission continuation needs current execution authority; ambiguous submission needs exact owner recovery, not another launch. If the owner cannot safely distinguish those cases, report unresolved.

Caller-request identity remains separate from B1 `request_id`, proposed `acceptance_record_id`, and any E2 idempotency key. Their mapping must be explicit, durable, and validated; equal-looking strings do not prove the join. A response-lost caller can reconcile by the original key without needing a response-issued correlation token. Numeric retention/expiry and support for key lookup after recovery remain prerequisites, not current guarantees.

**Proof required:** duplicate concurrent requests; distinct identical-content intents; same-key changed-payload conflicts; response loss before/after runtime acceptance and receipt publication; restart; current authorization denied on reconciliation; expired-key behavior; forged actor/delegation and stale B03 destination. Add A→B→A association races, current-parent drift after acceptance, incomplete accepted-material publication, read-only reconciliation byte checks, and a submit racing `NotRecorded`. Each success must join existing owners; none is a fixture-only readiness claim.

## 4. B02 — Exact-work read, observation, and cancellation

**Existing owners:** registry, Supervisor, runtime event transport, WorldDispatchControl; RetainedWorkerRuntime only for actual worker lifecycle. [Cancel categories](../../../llm-last-mile/runtime-refactor/contracts/cancel-outcome-categories.md), [B4 owner](../../../llm-last-mile/runtime-refactor/slices/b4-receipt-targeted-cancel-inspect-stop.md).

### Proposed operation meanings

| Operation | Input | Result |
|---|---|---|
| Read accepted work | Authenticated caller and exact owner receipt/work reference. | Coherent exact-work observation cut: acceptance reference, immutable B03 binding, owner state/revision, optional exact terminal result, observation completeness/availability, and source provenance. |
| Observe accepted work | Exact work plus owner cursor/observation epoch or an explicit initial-read request. | Authorized snapshot/replay/live continuation with a declared cut, duplicate handling and retention limits, or an explicit stale/gap/unavailable result. |
| Request cancellation | Exact accepted work; caller cancel-request key; current authorization. | Original owner cancellation identity when recorded, explicit refusal/invalid target/unreachable/unknown, pending closeout, or already-terminal evidence. |
| Reconcile cancellation | Exact work and cancellation correlation. | Current request disposition plus independently authoritative work outcome. No second cancellation intent merely because the ACK was lost. |

Bind receipt revision, stream/frame cursor, semantic-event identity, and observer epoch separately. Read-cut/replay-to-live handoff must avoid unreported gaps; a cursor is not a universal Workspace sequence or an AgentKit sequence. Exact retention counts, source epochs, durable-prefix boundaries, concurrent subscription limits, and transport binding remain owner-defined, not inherited from old planning constants.

Producer disappearance may leave an unresolved nonterminal claim. Preserve available history and expose an unavailable tail; do not replay by rerunning the prompt. UI unmount/disconnect only detaches observation. Owner recovery guarantees after shell/service restart must be stated and tested separately, not assumed from Lane durability.

Preserve the existing accepted-work cancellation distinctions: pending closeout, already terminal, no active cancelable work, unreachable owner, invalid target, World mismatch, ambiguous target, and policy denial. Only exact work-terminal evidence yields a cancelled task/turn. Completion can win before or after cancellation acknowledgement. Worker stop, turn/task cancel, and pending-admission cancel remain different operations.

The [current dispatch outcomes](../../../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) also include `CancelledBeforeTransport` and `AlreadyRoutable` for pending-admission control. B4 further requires distinct cancelled-before-registration, registered-before-transport, transport-ambiguous/cancel-pending, already-routable and terminal dispositions. These describe admission progress, not interchangeable accepted-task terminal states. The bounded operations above target accepted work only: pending-admission cancellation is explicitly unsupported by this proposed surface until separately specified using exact admission identity. If exposed later, preserve these outcomes and their owner projections losslessly; never substitute an accepted-work receipt, `NoActiveCancelableWork`, or a fabricated task terminal result.

Expose `Completed`, `Failed`, `Cancelled`, `NeedsRetainedFollowup`, and `Invalidated` losslessly where the selected owner supports them. `NeedsRetainedFollowup` ends ephemeral work; it creates no worker or continuation route. Do not invent a terminal wall-clock timestamp: absent source time stays absent, even if the UI uses a labeled presentation timestamp.

**Hierarchy extension:** authorized redirection must produce coordinator-visible causation/state updates without a hidden competing instruction chain. Ordering, stale-plan rejection, notification recovery, and escalation acknowledgement need their own messaging/control contract. They are not implied by observing a receipt and need not be implemented for the one-task proof.

**Proof required:** acceptance without running; immediate terminal; two authorized views of one work item; disconnect/reconnect; duplicate/conflicting/out-of-order frames; missing tail and producer restart; cancel/completion races; late cancellation ACK; exact terminal readback; nonstandard terminal mapping; unauthorized read/cancel; explicit rejection of unsupported pending-admission targets without substituting task cancellation. Any later admission-control extension must prove lossless B4 outcome mapping. Unsupported observations remain visible as such.

## 5. B03 — Lane and World binding

Use the [companion B03 proposal](lane-world-binding.md). It supplies the proposed identity split, concrete selection, admission/association race boundary, historical/current association distinction, and Lane facts versus derived state.

The selected task needs one validated existing Lane/World destination and exact accepted binding. It does not force one World per Lane, a new Lane per task, a continuously running Lane orchestrator, or a complete promotion API. Readiness for the narrow proof still requires a genuine owner-backed association—not a fixture Lane ID taped onto a receipt.

## 6. B04 — Typed selection, semantic resolution, and delivered context

**Owners:** Handbook owns semantic Scope/Resolution; Substrate owns authorized assembly and actual execution delivery; Collider owns selection/presentation, not semantic resolution. [Consumer foundations](../../../../collider/docs/agent-native/canonical/01-collider-v1-foundations.md#11-context-and-continuity-model) establish the consumer requirement. Handbook's callable owner contract was not inspected in this pass; its exact integration remains a named prerequisite, not a guessed API.

### Proposed operation meaning

Inputs identify purpose/action, permitted typed source/artifact/work-item references with their owner and revision, current Scope/Resolution posture, and the validated Workspace/Lane/World destination where applicable. A selector is not a grant to read its target. Define allowed reference kinds against the actual Handbook contract before writing wire types.

The handoff is: caller selects → authorized Handbook resolution → Substrate verifies execution visibility/policy and assembles → exact execution receives the admitted material. Resolution or assembly can fail without accepting a World task. Any preparation after acceptance must use explicit owner work-state semantics, not erase accepted work on error.

Recommended first-path handling: required missing, stale, malformed, or unauthorized references refuse preparation; do not silently substitute latest content. Optional omission is permissible only if explicitly declared and reported with provenance. Revalidate effective authority/binding before delivery; stale authority cannot be rescued by reusing a previous context pack.

Return/link distinct resolved and injected records bound to the actual work. Preserve acquired and produced lineage as execution supplies it; no full context inspector is required. A claim of delivery needs runtime-path evidence, not merely a successful resolver call. Transcript history can be selected material, never implicitly the complete injected context.

Fresh reconstruction is the default interaction posture; long-lived logical workers and durable orchestration identities remain allowed. Native/provider continuation is explicit and separately qualified, not inferred from a Lane view or adjacent message. The selected ephemeral proof does not require Qualified Session continuation.

**Proof required:** valid selected references reach the real execution; stale revisions, denied visibility and wrong Lane fail; required missing input is not silently dropped; a same-UI-thread invocation does not silently resume native context; resolved and injected provenance are distinguishable; permission/binding drift between selection and delivery is handled safely.

## 7. B05 — Revision-aware dependency requirement mutation

**Existing owner references:** [World deps](../../reference/world/deps/README.md) distinguishes inventory, enabled declarations, and applied status; [internals](../../internals/world/deps.md) describes owner implementation concerns. Source/test call paths and all-writer coordination were not audited here.

### Proposed bounded operation meanings

| Operation | Input | Result |
|---|---|---|
| Read Workspace requirements | Authenticated caller and Workspace reference; optional page cursor. | Authorized known items, provenance, Workspace-local declaration versus inherited/effective enablement, editability, and opaque revision covering relevant preconditions. Applied status, if included, identifies the exact World and observation freshness. |
| Set Workspace requirement | Canonical item reference, desired Workspace-local enablement, expected revision, mutation key, current caller authorization. | Exact committed revision and authoritative result readback; no-op disposition; conflict; denied/invalid/unsupported; or unknown outcome requiring reconciliation. |
| Reconcile mutation | Caller-scoped mutation identity. | That operation's committed result/revision or explicit unresolved/refused/expired state—not an inference from the latest value alone. |

Removing a Workspace declaration does not necessarily disable an inherited dependency and does not uninstall anything. The boolean edit means the local declaration, not an arbitrary override of all effective enablement. Inherited/unknown/uneditable entries remain read-only in the bounded proof.

The expected revision must cover the actual Workspace patch and relevant inventory/parent/config inputs used to validate the edit. Under the owner's serialization boundary: validate caller, compare relevant preconditions, normalize the operation, persist the declaration and recoverable mutation result, and return that committed revision/readback. Competing stale writers receive conflict without partial mutation. Identical retries join the same operation; changed-payload reuse conflicts.

All supported writers of the covered patch must participate in this protocol or be explicitly excluded by an accepted contract. A UI-only mutex or atomic rename does not provide compare-and-set across other supported CLI/config writers. Unsupported external edits are outside that guarantee: detect changed preimages where possible and refuse/reconcile; do not claim protection against arbitrary concurrent host writers without the required filesystem enforcement.

If another authorized mutation follows an ACK-lost success, the latest value may differ; preserve the earlier operation's committed result rather than reporting it never happened. A mutation that has not been durably correlated cannot be called safely retryable.

### Full-architecture compatibility, without expanding the proof

SCA-10 requires shared Lane baselines and explicit per-World overrides in V1. The eventual owner contract must distinguish inherited baseline, explicit overrides, effective desired configuration, and applied World state. Dependency/configuration merge rules must not masquerade as policy widening. Destination-baseline reconciliation on reassociation remains explicit; stable files do not excuse ignoring security revocation.

The current proof edits only Workspace declarations. It neither exposes Lane/World override editing nor certifies their V1 implementation. A declaration edit does not install/provision, refresh a World, merge code, or silently mutate active working state. Existing docs separately prohibit runtime system-package mutation and host-native provisioning of those packages; do not promise arbitrary OS dependency variants from an override alone. Unsupported environment realizations must be reported, not satisfied by granting host mutation or silently adding a deferred remote/container backend.

**Proof required:** authorized read/add/remove/readback; inherited versus local enablement; stale and concurrent mutations through every supported writer; inventory/parent drift; invalid item and denial; external-edit detection limits; result durability through crash/ACK loss and later writes; no install/provision or active-World mutation caused by the declaration operation.

## 8. Consumer mapping requirements

| Owner fact | Collider interpretation / prohibited shortcut |
|---|---|
| Pending/unknown request, no accepted reference | Request UI only; not running, failed work, or a fabricated queue state. |
| Exact accepted work | Accepted-first projection; running only when owner evidence supports it. |
| Receipt + historical Lane/World binding | Same work can appear in Control Plane and Lane projections; opening two views does not create two behavioral owners. |
| Cancel-request acknowledgement | Pending cancellation; continue observing until exact terminal evidence. Client `cancelRequest.generation` is not owner cancellation identity. |
| Pending-admission control target/outcome | Explicit unsupported-path result in the bounded accepted-work surface. Any later supported extension preserves `CancelledBeforeTransport`, `AlreadyRoutable`, and B4 admission progress separately from task terminality. |
| Owner terminal / unusual result class | Lossless mapping or explicit unsupported contract path; no success/cancel fabrication. Terminal time remains unavailable when absent. |
| Owner cursor and read cut | Adapter checkpoint/projection sequence is separate; filtering cannot introduce unexplained sequence holes. |
| Declared dependency change | Configuration result, not installed/applied confirmation. Applied status must name the observed World. |

Collider's checked-in correction is a patched local client candidate, not a demand that Substrate adopt AgentKit protocol names. The existing consumer lifecycle mapping must be reviewed against the owner contracts once approved; it was not rewritten or approved here. C-OQ-027, native/deployment proof, licensing/distribution, and adoption are not closed by a contract draft.

## 9. Reconciliation and readiness register

This is a proposal-review checklist, not a new canonical status ledger. Link to actual owners for status changes.

| Item | Freshly read document evidence | Remaining boundary before implementation-ready claim |
|---|---|---|
| B01 | B2.2 owner records landed/review-clean receipt return and E2-RM use, with separate closure unlocated. The older consumer plan must not call these wholly absent. | Satisfy the runtime-profile readiness gate; review §3.1–3.2 result algebra/admission ordering; select supported human/delegated ingress and durable correlation placement, canonicalization, retention/expiry, exact wire types and call chain. |
| B02 | Existing Supervisor and cancellation contracts define exact ownership and outcomes. B4 owner records landed source but unverified exact terminal proof/closure. | Public authenticated read/observe/cancel, cut/cursor/retention/restart contract, result mapping and exact-path proof. |
| B03 | HSA exact binding and physical adoption exist as named owner contracts, not the newly confirmed multi-World Lane authority. | Review companion §3.1/4.5 revision scopes and linked receipt carrier; choose authority placement/HSA bridge, exact wire schemas, transition/admission serialization, supported association reads. |
| B04 | Collider canonical context/continuity requirements were refreshed. | Handbook owner contract, concrete typed references/callable integration, preparation/acceptance boundary and delivered-context evidence. |
| B05 | World deps docs distinguish declaration and application. | Complete revision dependency set, supported writer inventory, transactional mutation/result persistence, wire errors and real read/edit/readback proof. |

**Cross-cutting successor obligations:** (1) session-bound World authority versus Workspace multi-World Lanes; (2) reconcile existing filesystem-sync/history and guest-image plans with stable project state; (3) immutable worker cap versus authorized same-worker expansion; (4) ownership-only auto-attach versus separately authorized bounded reasoning; (5) global/Workspace dependency configuration versus Lane baselines and World overrides. These are recorded design differences, not permission to remove current safeguards. The [auto-attach invariant](../../../llm-last-mile/runtime-refactor/architecture/invariants/07-auto-attach-restores-ownership-only.md) stays unchanged; a future trigger owns its own action authorization.

**Required research dependency:** [World state and environment research/experiments](../../ideas/world-state-environment-research.md) must compare alternatives and produce evidence before selecting the new filesystem/environment mechanisms or asserting their guarantees. It references ADR-0009/0010 and recoverable historical plans, including the older image/Workspace overlay keying. Guest OS isolation and project baseline isolation are separate questions. The older guest-rootfs initiative is not automatically adopted, made V1, or deferred with alternative-Lane backing. Research execution and experiments remain unstarted.

Continue semantic decisions for B01–B05; keep mechanism-dependent B03 state operations and B05 environment realization explicit dependencies. The narrow B05 declaration mutation is not per-World realization proof. A protocol-level test or unchanged declaration revision cannot demonstrate actual dependency isolation or stable project files.

Some obligations are outside the selected proof but not outside the product. B03 needs real binding; B05 can prove Workspace mutation separately. Stable filesystem behavior and V1 overrides must be satisfied before claiming complete V1, even if a smaller experiment does not exercise them. Worker fork, lane-orchestrator delivery, active-work transition, broad scheduling, and same-World editing mechanisms remain separately scoped. No optional non-E3 lease or E3 exclusion removal is selected here.

## 10. Review, validation, and next owner work

The useful next review unit is B03 authority/binding and B01 acceptance integration, followed by B02 observation/control. B04 and B05 can be specified independently using the same identity/authority conventions; this ordering is not a request to dispatch parallel agents or implement anything.

For each contract, owner review must choose exact schemas and supported entrypoints, resolve the named open semantics, identify production callers/enforcement, and attach its negative/race/recovery test plan. Only then is a consumer mapping implementable. Runtime acceptance additionally requires the actual installed Linux path; fixture tests or documentation approval are insufficient.

Document-review checks for these two drafts:

```bash
git diff --check
git diff --no-index --check /dev/null docs/contracts/draft/lane-world-binding.md
git diff --no-index --check /dev/null docs/contracts/draft/collider-b01-b05-handoff.md
shasum -a 256 SUBSTRATE_CORE_ARCHITECTURE_SESSION_DECISIONS_2026-09-20.md RUNTIME_REFACTOR_SESSION_DECISIONS_2026-09-18.md
git status --short
```

For new files, no-index checks can report differences; inspect whitespace diagnostics separately from a normal difference exit status. Also validate Markdown link targets/fragments, source pins, the active authoring fence, and absence of accidental status promotion. No Rust source changed, so no Cargo test result or runtime proof is claimed. Implementation test commands must be bound to the later admitted source slice rather than guessed in this draft.

**Initial authoring validation (before research follow-up):** root-agent consistency review and mechanical checks covered both drafts, 27 Markdown links/fragments, unchanged dossier hashes, and the two-new-document fence. Those results apply to the earlier draft bytes.

**Research follow-up validation:** root-agent delta review and mechanical checks passed for 53 local Markdown link occurrences (all draft/brief links and newly added dossier/backlog targets), seven historical Git paths, whitespace, the updated dossier source pin, and the five-document scope. HEAD and index remained unchanged; the historical runtime dossier remained byte-identical. No independent reviewer, owner approval, commit, integration, or runtime test is claimed.

**B03/B01 semantic follow-up evidence (2026-09-20):** reread the focused authority map, HSA binding invariant, receipt/supervision seam, ephemeral receipt, B2.2 owner, and relevant E2 commitment/E2-RM sections. All six files were byte-compared with the freshly rebound clean Linux checkout at the same `088f62d81c182b92cac404983b8bf512becc829c` revision. This pass edits only these two proposal drafts; the dossier, research brief/backlog, historical dossier, canonical owners, source, and Collider files are unchanged. Proposed revision scopes, receipt composition and result semantics are not owner approval or runtime proof. Earlier validation counts describe earlier bytes.

Root review and mechanical checks for this semantic follow-up passed: 32 draft link targets/fragments, whitespace, dossier source pin, unchanged HEAD/index and protected documents, and the two-file edit fence. Root review also distinguishes stale-destination rejection for a new request from immutable original-binding return for an accepted retry. These are documentation checks, not execution tests or independent approval.

**Drift rule:** before owner acceptance or implementation admission, recheck the dossier hashes, relevant Substrate owner documents and shared dependencies, and consumer mapping at the recorded bases. A moving branch tip alone is not grounds for a complete reread; a changed relevant authority or dependency is. Never treat a file hash alone as proof that its transitive semantics stayed unchanged.

**2026-09-21 review correction:** the independent source/document review requested changes for the runtime-profile gate and pending-admission outcome coverage, and qualified the HSA redesign inference. These draft corrections address that scope without approving a runtime profile or adding implementation authority. Existing bytes were first checkpointed at local `f461ac3f0` on `docs/substrate-collider-contracts`; the earlier authoring/source bindings above remain historical evidence, not that branch's current HEAD. This correction does not update the dossiers, research scope, canonical owners, or runtime source.
