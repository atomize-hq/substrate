**Kind:** crosswalk
**Stable ID:** `B1-B2.1-family`
**Canonical for:** B1/B2.1 family seam ownership, activated-store, replay/startup, and dispatch-authority crosswalks
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Source span:** composite of the four preserved root compatibility spans listed in the extraction ledger
**Supersedes:** canonical ownership of the extracted source bodies; source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# B1/B2.1 family seam crosswalk

### B3.2a-WA bounded seam-ownership addendum

The following rows amend only the named action for B3.2a-WA. They do not replace the preserved root
[`Host authority and ingress`](../02-seam-crosswalk.md#a-host-authority-and-ingress) or
[`Dispatch, policy, receipts, and retained runtime`](../02-seam-crosswalk.md#b-dispatch-policy-receipts-and-retained-runtime)
seam rows and do not change any current semantic classification, authority/enforcement verdict, or
proof status.

| Seam | B3.2a-WA bounded action | Must not imply | Classification after B3.2a-WA |
|---|---|---|---|
| HostSessionAuthority | Supplies the exact durable session/world ID/generation/policy proof consumed as an immutable adoption precondition. | Backend metadata may not create, repair, replace, or override HSA binding; adoption returns no HSA authority. | `MislandedWrongModel` |
| RetainedWorkerRuntime | Supplies already-durable admission, R0-registration join, and sole `TransportClaimedNonterminal` launch ownership; its exact proof is equality-validated before adoption. | Adoption changes no admission/R0/claim/routability/terminal record and an exact joined claim still cannot resend. | `MislandedWrongModel` |
| WorldDispatchControl | Carries the validated authority-managed `Some(exact proof)` to the existing service boundary and consumes the resulting launch/error outcome. | It cannot select a replacement world, author physical ownership metadata, or treat adoption as Registered, routable, terminal, or successful launch truth. | `MislandedWrongModel` |
| WorldRuntimeAdapterExecutionEnvelope | Names equality-only session/world/generation/policy/project/spec evidence required before authority-managed realization. | B3.2a-WA adds no persisted envelope, wire field, schema version, authority source, credential behavior, or launch proof. | `DefensiveScaffoldingOnly` |
| RuntimeFamilyRealizationAdapter | World-service/Linux backend durably exact-adopts `GenericExactBoundWorld` as `SharedSessionOwnerExactBoundWorld` under trusted locking and publication conventions, preserving world ID/generation; exact retry joins and conflicts fail closed. | No alternate world creation, HSA/admission mutation, PID/liveness authority, prompt/request persistence, compatibility-path change, provider behavior change, platform promotion, or claim that adoption proves member creation. | `UsefulFootholdButWrongBoundary` |

**B3.2a/B3.2a-WA closeout:** the retained creation/admission bridge, typed launch proof, both
production Spawn adapters, exact Registered/terminal projection, and Linux exact-bound-world
adoption are review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The bounded live
toolbox proof retained one admission and created no alternate world: HSA, the launch receipt, and
backend ownership metadata all named world
`wld_019f6a96-bdf1-7db1-a5cc-58cbd0f370c9`, generation `0`, for session
`aos_27f14606e443f73927d2f892beabc704`. This evidence changes none of the classifications above,
makes no non-Linux claim, and leaves durable abandoned-admission resolution with remaining B3.2
and its user/tool-facing inspect/cancel surface with B4.


### B1/B2.1 activated-store ownership and production-handoff correction

The `WorldWorkReceiptRegistry`, `WorldWorkExecutionSupervisor`, `WorldDispatchControl`,
`HostSessionAuthority`, and `StateStore` rows remain distinct. `WorldWorkReceiptRegistry` is the sole
semantic owner of proposal and immutable acceptance schemas, validation, exact-retry joins,
conflict rejection, and exact acceptance inspection. `WorldWorkExecutionSupervisor` is the sole
semantic owner of post-acceptance active observation claims, frame/event journal state, terminal
reconciliation, restart recovery, and caller-drop survival. `WorldDispatchControl` may project those
owners' truth into existing blocking compatibility behavior only where those owners supply the
complete decision inputs; it cannot create, replace, or delete that truth. The early bridge covers
ephemeral accepted-task Inspect/Cancel/Wait only. Retained Inspect/Cancel/Stop still require later
RetainedWorkerRuntime/B4 lifecycle and closeout truth and remain compatibility projections.

The HostSessionAuthority store supplies only separately scoped receipt-registry and supervisor-
journal physical transactions over the retained trusted root: exact root/store binding, the existing
cross-process lock, activated-store preflight, no-follow handles, crash-safe publication, and
interrupted-publication reconciliation. StateStore may carry or construct each semantic owner at
the production integration boundary, but neither `AgentRuntimeStateStore` nor
`BoundAgentRuntimeStateStore` becomes a generic activated-store writer.

Each physical capability is usable only for its fixed semantic owner's namespace and cannot read or
mutate either strict `StateRootV1` or strict `StateRootV2`, session-authority records, transition
intents, application journals, keys, or typed authority objects. Legacy session/participant writers remain rejected after authority
activation; neither packet whitelists files inside those legacy collections. No dual write,
fallback write, fixed-path active-task side table, fresh-root compatibility mode, process-local
durable truth, or physical-store interpretation of receipt/supervisor bytes is allowed.
B1-3a/B1-3b may become review-clean, after which B2.1 replaces the accepted production path's
legacy active-task registration with the durable supervisor claim and journal. Foreground
guard/waiter drop cannot erase accepted or supervised work. Physical serialization does not
transfer semantic authority to HostSessionAuthority, change either authority revision domain, or
promote a seam. B1 and B2.1 close only through their joint production integration gate; B3.1
remains blocked until it passes.


### B2.1-3 producer replay and startup activation correction

The restart owner and the replay producer are deliberately different seams.
`WorldWorkExecutionSupervisor` alone enumerates durable nonterminal claims, performs the immutable
receipt anti-join, interprets journal/cursor state, validates the exact
acceptance-record/stream/cursor binding, starts observers, records unresolved replay state, and
commits terminal truth. `RuntimeEventTransport` may retain exact B0 frames in a bounded
world-service process-memory registry and replay them only for one supplied exact acceptance-record
ID, stream ID, and frame cursor. The replay surface cannot enumerate streams, accept fuzzy or
session-only lookup, change frame identity/order/bytes, or create any receipt, claim, obligation,
lifecycle, terminal, cancellation, deletion, or success fact.

If the shell restarts while the world-service registry survives, canonical supervisor recovery may
replay the missing suffix and resume the live stream without duplicate journal transitions. If
world-service restarts or replay is otherwise missing, mismatched, expired, unavailable, corrupt,
reordered, or conflicting, the durable claim stays nonterminal and unresolved. No PID, helper,
socket, process, readiness, timeout, caller, endpoint, EOF, or stream-exhaustion observation may
advance or resolve it.

The current production startup integration is only one call from `run_async_repl` to the canonical
supervisor recovery operation plus retention of the returned observation tasks. The REPL cannot
read supervisor records or reproduce their logic. Recovery-store corruption, invalid claim
identity, or impossible durable state is fatal and fails shell startup closed. A valid nonterminal
claim whose producer replay is currently unavailable is not corruption: it remains durably
unresolved and may be reported exactly without pretending successful resumption or making ordinary
startup depend on fabricated completion. This bounded hook does not establish full surface
neutrality and promotes no seam.


### B1/B2.1 dispatch-authority ownership audit

The production-ingress audit selected **Case B — canonical production input is missing** because
A1.1e's exact reader could not resolve a session until a production authority already existed and,
at that audit point, the source branch contained no production authority creator/adopter. A1.2a
and A1.2a-S now boundedly satisfy the ordinary-internal-host creator/adopter gap. The current
prepared dispatcher still requires canonical retained caller/session/target record shapes and a
`live_retained_worker_count` that A1.1e and A1.2a-S do not represent. `retained_worker_refs` are
immutable object refs without live or terminal lifecycle state, so counting them would silently change the existing
`max_live_retained_workers` steering contract. The legacy `authoritative_live` composite cannot
fill that gap.

The smallest acyclic correction has seven prerequisites before the joint closeout:

The B1/B2.1-0 production-route audit additionally found one bounded read-only adapter on the real
host tool path. `tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1` may change
only its active-task branch. That branch consumes the exact current HostSessionAuthority, immutable
`WorldWorkAcceptanceRecordV1`, and `WorldWorkExecutionSupervisor` claim/cursor/terminal state and
requires exact equality across store, orchestration session, caller participant/backend, world ID/
generation, task or active-run identity, acceptance identity, and stream identity where applicable.
It may project that joined truth into its existing result but owns and mutates no durable state,
cannot synthesize a legacy active-task record, and cannot infer lifecycle from any episode or
transport observation. Unknown task, stale linkage, caller/backend mismatch, world mismatch,
nonterminal work, exact terminal work, and unresolved producer observation remain distinct
outcomes. Receipt existence without the exact supervisor claim is insufficient.

The retained-worker branch is frozen on its compatibility path, including target selection and
error categories. To make the branch boundary real, the existing shared pre-`match` legacy caller/
world resolution may be mechanically relocated into the retained-worker branch with byte-equivalent
inputs, order, errors, and results. This grants no retained behavior change and no second resolver;
only the active-task branch receives the canonical HSA/receipt/supervisor read. Function signature
and callers are unchanged. The approved GitNexus HIGH impact
is limited to the known 12 direct callers, four execution processes, and 14 impacted symbols for
this exact branch correction. A new production symbol, signature change, retained-branch change,
or new HIGH/CRITICAL process family is `CrossDocumentChangeRequired`.

1. **A1.2a — current-authority establishment/read prerequisite** first owns the strict
   greenfield-only V1-to-V2 root upgrade, then owns only production Start
   reservation/issuance/claim/application, initial authority birth, crash-safe exact retry, and a
   typed read result that joins the applied Start descriptor to exact
   store/session/active-participant/lineage/workspace/world/revision/current-policy truth and yields
   the accepted-home bound capability. Strict V1 remains readable but cannot receive Start or later
   semantic mutation; a nonempty, unsafe, or otherwise non-greenfield V1 root is not converted.
   Tests establish state through this production protocol, never a test-only writer. It does not own Attach, ResumeOneTurn, post-turn handling, obligation
   consumption, transition-correlation supply to world work, or public CLI/REPL adoption.
2. **A1.2a-WB — Host/world-binding validation correction** preserves descriptor-scope equality
   with the requested launch scope while freezing the four valid/invalid combinations: `Host +
   None`, `Host + Some(exact)`, and `World + Some(exact)` are accepted; `World + None` is rejected.
   Start issuance, application/persistence, and exact current-authority resolution enforce that
   same matrix; an accepted persisted authority cannot be rejected later by an obsolete `Host +
   Some` reader rule. An empty or malformed binding is rejected without mutation. Exact binding stays on
   `DurableSessionAuthorityV1`; host participant manifests remain host-scoped and gain no world
   placement fields. This correction is limited to `transition.rs`, colocated
   `transition_tests.rs`, and `facade.rs::HostSessionAuthority::resolve_current_exact`; all other
   facade behavior remains outside scope. It has no schema, canonical-byte, golden-vector,
   migration, compatibility, or persisted-object rewrite.
   This matches existing host-rooted world-backed source truth:
   `build_world_start_host_attach_dispatch_envelope` keeps the attach execution scope `Host`, and
   `agent_status_selected_host_row_stays_unchanged_when_parent_session_has_world_binding` proves the
   host participant projection stays host-scoped while the parent session carries world ID and
   generation. The correction does not introduce a new product feature.
3. **A1.2a-S — bounded internal Start adoption prerequisite** makes
   `prepare_host_orchestrator_runtime_from_resolved` produce only a non-authoritative
   `GreenfieldHostStartProposalV1` carrying the accepted home/store, normalized workspace,
   descriptor, policy, and shell observation but no authoritative session/participant/run identity;
   it performs no persistence or transport and is never represented as a pending
   `PreparedAgentRuntime`. On the real dormant-host launch path,
   `dispatch_targeted_follow_up_turn` obtains the exact optional initial world binding and calls
   `apply_greenfield_host_start_from_authority`, which derives the domain-separated issuer key from
   the greenfield store identity plus the complete canonical Start plan and, before any transport
   or legacy write, invokes A1.2a. The exact applied result constructs the existing fully
   materialized `PreparedAgentRuntime` and in-memory compatibility projection and carries the bound
   authority capability through `RuntimeOrchestrationContext` to the internal toolbox. Fork/member
   preparation and remote-member consumption remain unchanged. On this path the large runtime
   start function skips activated-store legacy session/participant/snapshot writes; events and
   process state remain observations and startup ownership remains Pending. A1.2a-S does not adopt
   hidden-owner plans, public CLI Start, Attach, ResumeOneTurn, auto-attach, startup result
   reconciliation, or post-turn behavior; the active A1.3-P1 packet retains those real public/successor gates while the older A1.3 fence remains held only as history.
4. **B1/B2.1-R0 — canonical retained-target prerequisite** is a bounded two-owner handshake.
   The existing validated retained-creation ingress request/idempotency ID and caller-fixed
   participant plan first exact-join or create an HSA-owned reservation that validates that
   participant and fixes registration and object identities,
   commitments, expected authority, policy, world, and timestamp before object publication.
   RetainedWorkerRuntime then creates the immutable descriptor, participant-specific resume handle,
   and retained-worker object graph, with the worker binding the current policy ref. HostSessionAuthority
   validates exact kind/commitment/session/world/backend/protocol scope and atomically appends only
   that retained participant ID to authoritative lineage, adds only its retained-worker ref,
   advances the authority revision, and records an immutable non-transition registration proof
   consumed by `resolve_exact`, and atomically marks the reservation Applied. Crash retry and a
   lost response rejoin by the same issuer ID; changed bytes or a stale/conflicting retry fail
   closed. Registration cannot change active caller, posture, workspace, world, policy, origin, or
   any unrelated ref. If startup ownership is Pending, the proof forms one contiguous ancestry
   link without rewriting the original expected revision or accepting startup. R0 then
   exact-resolves the target for retained-turn proof, but deliberately has no production ingress
   caller and cannot by itself satisfy full-dispatcher proof. It may not use a legacy participant
   writer or test fixture, and it does not pull forward messaging, accepted-turn observation,
   park/cancel/stop/fork, live-retained admission counting, or broader B3.2 behavior.
   R0 is independently review-clean through `bb3eefba`; this is component evidence only and did
   not itself complete B3.2a. B3.2a plus B3.2a-WA are now independently review-clean through
   `d0a70727c2bec2b2d6fe0754ea469c4682684dda`, and no seam is promoted.
5. **B3.2a — retained creation/admission bridge prerequisite** is the smallest early
   RetainedWorkerRuntime-owned subset needed to invoke R0 from the real `SpawnWorldWorker` path
   without weakening `max_live_retained_workers`. Before R0, one durable cross-process admission
   CAS exact-joins a RetainedWorkerRuntime-owned domain-separated keyed fingerprint of the complete validated Spawn request,
   authority observation, descriptor/runtime plan, policy/cap, and participant; counts all
   nonterminal slots; enforces the cap; allocates/collision-checks stable participant/bootstrap-run
   IDs; and persists `SlotReserved` without any request, prompt, or payload preimage. A registry
   with queued slots and no head is valid. Only exact re-presentation of the complete canonical
   request for the lowest-sequence queued slot may acquire the durable per-session
   `AuthorityRegistrationHead`, fix an expected authority revision, and enter R0; digest equality
   alone is insufficient, changed bytes conflict without mutation, and later slots cannot
   overtake the earliest slot. Current-head R0 reconciliation releases that head in a separate
   durable transaction and never automatically promotes another record. Queued slots cannot
   reserve the same revision. The
   bridge covers both production adapters: direct `dispatch_orchestrator_world_request` transport
   and `handle_internal_toolbox_world_dispatch_request`, which must use an authority-bound runtime
   preparer instead of its retry-local participant allocator. Both carry the same typed proof
   through transport-api `MemberDispatchRequestV1`; the real `Service::execute_stream` member branch
   first completes the existing strict validation, performs B3.2a-WA exact same-world ownership
   adoption, and only then passes the unchanged typed request to exact validation at
   `MemberRuntimeManager::launch`.
   The live adapter also makes `start_remote_member_runtime_with_prepared` validate the same
   session/request/participant/backend/protocol/world registration instead of invoking any legacy
   session/participant writer on an activated store. Its fixed durable admission record is
   nonterminal before transport, becomes routable only on exact matching Registered truth, and
   becomes terminal only on exact B0 terminal truth. EOF, error, drop, PID/helper/socket state, or
   observer loss remains interrupted-nonterminal, counted live, and non-authoritative. Thus the
   live count can narrow spawn admission without using HSA refs or compatibility liveness. It adds
   no retained turn, message, park/cancel/stop/fork, abandonment resolution, or final lifecycle
   policy. PID, caller, helper, socket, endpoint, timeout, EOF, process-liveness, and observer-loss
   observations cannot acquire, renew, replace, or steal the head; an abandoned earliest slot
   remains live and blocks later slots until its exact request retries. Remaining B3.2 owns the
   exact durable admission-state resolution and restart reconciliation; B4 owns the user/tool-facing
   inspect/cancel verb and distinct outcomes. B3.2a implements neither later protocol.
6. **B3.2a-WA — exact bound-world ownership adoption prerequisite** consumes only the already-
   validated authority-managed `Some(exact proof)` plus exact HSA session/world/generation/policy,
   project/world-spec identity, and current generic-world metadata. Before member creation, the
   runtime-family/Linux world backend durably changes only physical ownership metadata from
   `GenericExactBoundWorld` to `SharedSessionOwnerExactBoundWorld`, preserving world ID and
   generation. Exact retry joins without rewrite; any foreign owner, changed session/policy/project/
   spec/generation, missing/corrupt/ambiguous metadata, or partial publication fails closed without
   creating an alternate world. The operation changes no HSA, admission, R0, transport-claim,
   routability, or terminal record and does not prove launch. Compatibility `None`, ordinary world
   execution, filesystem/network/caging/policy/discovery/write-sync behavior, and untested platform
   posture remain unchanged. It adds no `world-api` field, persisted wire-schema version, side
   table, prompt/request preimage, shell rebinding, PID/liveness authority, or recovery semantics.
7. **B1/B2.1-0 — action-scoped dispatch preparation** consumes those read results plus the
   recovered B1 acceptance and B2.1 supervisor truth, and replaces the
   missing legacy session/caller projection inputs for RunWorldTask, ordinary retained
   ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait. It separates those paths
   from `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, and fork compatibility
   admission/lifecycle accounting. It consumes B3.2a routability for ordinary retained Continue,
   while the B3.2a Spawn bridge alone consumes the canonical live count.
   Authority, receipt-registry, and supervisor access uses one explicitly authorized trusted
   open-and-bind conversion or a caller-supplied bound capability. The already-landed B3.2a
   creation/admission bridge is the sole spawn change; spawn steering/admission/outcome and fork
   behavior remain unchanged and unpromoted. Retained Inspect/Cancel/Stop also remains unchanged for B3.2/B4. This
   prerequisite neither fabricates a canonical live count nor
   changes its steering semantics.

After those prerequisites, `WorldWorkReceiptRegistry` supplies immutable accepted-work identity,
`WorldWorkExecutionSupervisor` supplies active observation and terminal truth, and
`WorldDispatchControl` only validates and joins those sources. Process IDs, helpers, sockets,
prompts, foreground guards, process-local maps, and `authoritative_live` remain episode or
compatibility observations and cannot become B-owned durable authority.

The recovered ownership join is review-clean through `83101dcb`. The active-task branch of
`resolve_follow_up_dispatch_authority_v1` is a read-only exact join over current HSA, immutable
acceptance, and supervisor claim/cursor/terminal state; its retained branch and all retained
compatibility behavior are unchanged. `WorldDispatchControl` uses the joined B-owned view only for
RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task
Inspect/Cancel/Wait. Spawn remains the sole consumer of B3.2a canonical live-count/admission truth.
This result promotes no row and does not start the joint production closeout.

A1.2b remains after B3.1/C1 and is the sole owner of successor Attach/Resume issuance/application,
startup/post-turn reconciliation, obligation-cut consumption, release, and optional production
transition correlation. A1.2a does not move that work earlier. Full dispatcher entry remains
mandatory proof; calling an exact lower-level receipt/supervisor resolver directly is not an
acceptable substitute. The preserved broad A1.2 checkpoint is not restored or modified.
