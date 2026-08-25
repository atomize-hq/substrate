# Phase and Slice Map

## Sequencing rules

Each slice is independently reviewable. A0 is a diagnostic inventory; every later implementation slice must move one authority boundary plus proof. A slice may read sibling context without editing sibling ownership.

### Bounded review sequence

Every new slice follows the development-review contract in `04`. Freeze the selected integrated
outcome, scope, proof gates, subject-fingerprint method, and review budget before implementation.
Run one discovery review or same-subject burst, consolidate valid `P1`/`P2` remediation, and use one
different-fresh delta-focused closure review. Up to two supplemental causal cycles are permitted
only for `P1`/`P2` findings directly caused or unmasked by the immediately preceding remediation
inside unchanged authority and risk. `CLEAN` ends the loop; unrelated blockers, expansion, or budget
exhaustion stop non-completed. Unfixed `P3`/`P4` findings go to `06` and do not create remediation or
review cycles.

The parent validates the review-cycle record after each returned cycle and runs the `--next-cycle`
preflight before launching any closure or supplemental review. Historical packet-specific reviewer
counts remain evidence of those packets, not an automatic requirement inherited by later slices.

Hard dependency spine:

```text
A0 -> A1.1e -> B0 -> B1-3a/B1-3b receipt core -> B2.1-1/2/3 ------------------+
              \-> A1.2a current-authority protocol -> A1.2a-WB correction     |
                  -> A1.2a-S bounded Start                                    |
                  -> B1/B2.1-R0 -> B3.2a -> B3.2a-WA ------------------------+
                                                                               -> B1/B2.1-0
                                                                               -> joint closeout
                                                                               -> B3.1 -> C1 -> A1.2b
                                                                               -> A1.3-P1 -> A1.4 -> A1
A1.1d integrated Linux/native-macOS closeout -> A1
A1 -> A2/A3 -> E2 -> B2.2 -> remaining B3.2 -> B4 -> C2 -> C3
B1/B2.1 joint production closeout + E1 -> E2
D1 -> D2 -> D3
E1 -> E2 -> E3 -> E4
B4 + C1 + D2 + E2 + E3 -> D3 `RG-OBS-01` integration closure
D2 and E1 must agree on PolicySnapshotV3, but may land in either order behind fail-closed gates.
```

This graph is acyclic. `B1-3a/B1-3b receipt core` is a review state, not a claim that B1 is
production-complete: it supplies the exact proposal, acknowledgement, activated-store acceptance
record, and immutable anchor that B2.1 consumes. B2.1 may begin only after that core is review-
clean. B1 and B2.1 then receive one joint production closeout, and B3.1 has no edge from either
packet separately.

Cross-packet integration hold: a lower-level A1 persistence packet may expose a preexisting
production bypass that only a later canonical owner packet can close. The production-ingress audit
found that A1.1e is a necessary reader but not a sufficient joint-closeout foundation: it cannot
create the current authority its reader requires, does not return the applied caller descriptor,
and does not represent the shared dispatcher's live-retained count. The corrected bounded corridor
keeps the already-preserved B0 → B1 receipt core → B2.1 branch independent while
A1.1e → A1.2a current-authority protocol → A1.2a-WB Host/world-binding correction →
A1.2a-S bounded internal Start adoption →
B1/B2.1-R0 → B3.2a → B3.2a-WA builds the authority/retained branch. Those branches first join at
B1/B2.1-0 action-scoped preparation, then proceed through the joint
closeout → B3.1 → C1 → A1.2b even while A1.1d
integrated Linux and native-macOS closeout remain open. The corridor may not issue or apply host
transitions except the exact greenfield Start subset assigned to A1.2a, demote episodes, extract
general persistence, enable foreground early return, or perform retained-worker lifecycle beyond
B3.2a's exact creation/admission state and fail-closed live count. A1.2b resumes only after C1 can return the complete semantic cut from B2.1's durable exact
event truth and B3.1's typed retained-event semantics. This
hold does not permit A1.3 consumer adoption. The failing regression remains explicit, stale
lifecycle/world-binding behavior remains rejected, the feature branch remains non-landable, and
A1.2b/A1.3 must close their assigned gates before A1 can complete. This exception creates no
A1.1d-6, waives no gate, bypasses no A2/A3 ownership, and does not apply to another blocker.

The corridor is minimal for these reasons:

1. B0 owns runtime-generated stream/frame/event/terminal identity and ordering; B2.1 may not invent
   those identities after observation.
2. B1-3a/B1-3b precede B2.1 because a durable observer claim and journal require one accepted task
   or active-run identity. The review-clean core preallocates the proposed acceptance ID and typed
   request context that world-service retains before B3.1 can emit an exact accepted-run envelope;
   the proposal is not accepted truth until the B0 acknowledgement is durably recorded. B1 itself
   does not close at that review point because the production path must still hand the accepted
   stream to B2.1 without attempting a legacy writer.
3. B2.1 owns durable observation, duplicate/reorder rejection, terminal ordering, and restart
   reconciliation. The existing foreground may block as a receipt waiter until B2.2.
4. B3.1 is required because C1 needs exact retained target/run/thread/class/attention/causation
   semantics that neither a generic transport carrier nor the receipt registry owns. B3.1 moves
   the existing provider-payload classification to a fail-closed producer-side normalizer before
   `AgentEvent` construction; the emitted typed member, not host parsing of `AgentEvent.data`, is
   then the semantic source.
5. C1 alone classifies and materializes obligations and declares the complete cut. A1.2b consumes
   that result unchanged.

The typed host-transition correlation does not add a reverse dependency on A1.2b. B1 places both
the carrier and its equality-only opaque commitment representation in `substrate-common`, adds it
optionally to the typed request context, and keeps it absent on production paths until
HostSessionAuthority supplies it; no new A1.1e hash domain or verifier is required. B3.1 copies it,
B2.1 journals the bytes without interpreting them, B3.1 validates equality with B1, and C1 is fully functional for exact inputs while
refusing a transition-scoped Complete result when the correlation is absent or mismatched. A1.2b
later becomes the only production source, validates its own intent/revision/payload commitment,
supplies its exact intent/transition-run correlation through the HostSessionAuthority-owned call
boundary, and consumes the C1 result unchanged. Its real-path adoption is therefore the downstream
integration gate, not a C1 prerequisite.

No A2 prerequisite is pulled forward because process/PID/socket demotion is unnecessary to build
the producer-to-ledger truth path. No A3 prerequisite is pulled forward because StateStore may
provide bounded atomic persistence without deciding receipt, observation, messaging, or obligation
semantics.

The B1/B2.1 production-ingress ownership audit selects **Case B**. A1.2a now provides the
current-authority protocol and exact read; A1.2a-WB has corrected its Host/world-binding validation;
and A1.2a-S has adopted greenfield Start on the ordinary internal host bootstrap and carries the
bound capability into the live toolbox. B1/B2.1-R0 now provides the review-clean minimal
RetainedWorkerRuntime object/read plus HostSessionAuthority-owned lineage/ref registration proof for a canonical retained target;
B3.2a has now supplied the real Spawn integration plus canonical admission/routability truth that
R0 deliberately lacks, and B3.2a-WA has exact-adopted the HSA-bound world for shared-session
physical ownership without changing that binding; both are review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. B1/B2.1-0 now partitions
`prepare_orchestrator_world_dispatch` by action and payload so RunWorldTask, ordinary retained
ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait do not consume the missing
legacy `live_retained_worker_count`; `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop,
and fork admission/lifecycle behaviors remain unchanged, unpromoted, and outside the bridge. Spawn
keeps its existing steering/outcome but obtains its live count and registration from B3.2a. This
makes the prerequisite review-clean through `83101dcbcc750e6e8fb8979bea19f1f777792188` and
moves no Attach/Resume, post-turn, obligation, correlation, public-adoption, accepted-turn, or
broader retained-lifecycle work early. A1.2b remains after C1.

All code areas below are allowlists for planning, not permission to edit every listed file. Future implementation must run live impact analysis before symbol changes.

## Track A — Authority and surface neutrality

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A0 — Authority leak inventory** | Produce a repo-grounded inventory of every process/socket/helper/owner/cwd/env value currently used in durable decisions. | `00`; `01` invariants 1–3; `02` A0 inventory contract plus HostSessionAuthority, StateStore, and SurfaceAdapter rows; helper walkthrough/debug docs | HostSessionAuthority; HostExecutionEpisode; StateStore; CompatibilityReadModel | `agent_runtime/control.rs`; `agent_runtime/state_store.rs`; `execution/agents_cmd.rs`; `execution/orchestrator_world_dispatch.rs`; `repl/async_repl.rs`; UAA launch/member-runtime paths | No behavior changes except diagnostics/tests. No authority facade. No seventh control-pack file. | A committed inventory table in `02-seam-crosswalk.md` classifies every usage as `SignalOnly`, `FastPathTransport`, `AuthorityDecision`, or `CompatibilityRead`, and records proposed owner seam, first migration target, and proof gate. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01` |
| **A1 — HostSessionAuthority facade** | Establish the only API that resolves exact durable session/caller/binding truth, owns revision-checked posture transitions, preserves strict V1 intent reads, uses strict `HostSessionTransitionIntentV2` only for production Start, and later uses an independently reviewed strict V3 successor intent for real `Attach` and `ResumeOneTurn` paths. | `01` Authority map + invariants 1–3; `02` HostSessionAuthority and helper-plan/mode/parent-control/private-stop/auto-park rows; `04` `DurableSessionAuthorityV1` + strict V1/V2 Start and later V3 successor-intent contract; `DESIGN-host-orchestrator-world-dispatch-contract.md` exact identity rules | StateStore; CompatibilityReadModel; HostExecutionEpisode; WorldDispatchControl; AutoAttachProjection | Bounded modules under `crates/shell/src/execution/agent_runtime/host_session_authority/`; bounded facade types/re-exports and integration only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only as the bounded persistence/integration surface; bounded explicit-bootstrap-home entry points only in `crates/common/src/paths.rs` and `crates/shell/src/execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; in `agent_runtime/control.rs`, only `HiddenOwnerHelperLaunchPlan` and its write/load/remove/launch integration plus the separately bounded A1.3-P1 atomic public-adoption packet; in `execution/agents_cmd.rs`, only `Start`/`Attach`/`ResumeOneTurn` intent issuance and `run_owner_helper` consumption; in `repl/async_repl.rs`, only owner-helper validation/application and real REPL adoption; in `agent_runtime/auto_attach.rs`, only `build_auto_attach_launch_plan` and directly required plan integration; focused tests | No general `HostExecutionEpisode` demotion; no endpoint/path redesign; no helper removal; no auto-attach policy, eligibility, or settlement redesign; no receipt/supervisor work; no config/policy/inventory precedence, merge, or interpretation changes; no unrelated lifecycle, transport, or schema cleanup; no unrelated StateStore cleanup or general persistence extraction. | All real CLI and REPL transitions route through `HostSessionAuthority`: Start uses a durable revision-bound V2 intent, while Attach/Resume use the later V3 successor intent; exact applied retry joins without reapplying; helper loss/restart reconciles; stale, substituted, mismatched, expired, or conflicting replay fails closed; stale observations cannot overwrite newer authority; `RG-BASE-01` and `RG-BASE-02` remain green. Do not claim A1 before A1.1–A1.4, A1.3-P1, and the final proof wall complete. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02` |
| **A2 — HostExecutionEpisode demotion** | Represent REPL/helper/toolbox/recovered processes as episodes whose PID/socket/readiness data are observations only. | `01` invariants 1–2; `02` SurfaceAdapter row; `04` HostExecutionEpisodeV1; `DESIGN-internal-toolbox-transport-and-session-binding.md` | HostSessionAuthority; InternalToolboxTransport; RouterAttachTrigger | `agent_runtime/control.rs`; `execution/agents_cmd.rs`; `repl/async_repl.rs`; episode-focused tests | No deletion of `__owner-helper`; no transport protocol rewrite; no auto-attach redesign. | Killing/orphaning an episode leaves durable session truth intact; stale episode updates are revision-rejected; transport classification is explicit. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01` |
| **A3 — Persistence and compatibility split** | Reduce StateStore to atomic persistence/schema evolution and isolate diagnostic compatibility reads from new authority writes. | `02` StateStore + CompatibilityReadModel rows; `01` authority map; `04` durable revision rules | HostSessionAuthority; InboxProjection; receipt persistence | `agent_runtime/state_store.rs`; a bounded new facade/module under `agent_runtime/`; persistence/read-model tests | No wholesale database rewrite; no conversion of pre-A1 authority artifacts into A1 authority; no behavior changes outside moved ownership. | New authority writes bypass compatibility projection; torn-root/unsupported-state diagnostics remain read-only; newer revisions always win. | `RG-AUTH-02`, `RG-BASE-01`, `RG-BASE-03` |

### A1 bounded packet decomposition

A1 is one independently reviewable slice implemented in the following order. Each packet must remain green and reviewable before the next begins; prerequisites established by an early packet do not satisfy later production-adoption or slice-closeout gates.

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.1 — authority-store core** | [Canonical A1.2/earlier-histories family row](a1-2-earlier-histories/slice-and-task.md#a11-aggregate-packet-row). | — | — | — | — | — |
| **A1.2 — intent issuance/claim/application** | [Canonical A1.2/earlier-histories family row](a1-2-earlier-histories/slice-and-task.md#a12-aggregate-packet-row). | — | — | — | — | — |
| **A1.3-P0 — held preparatory split** | Canonical content: [`linux-first-runtime-resumption/slice-and-task.md#a1-bounded-packet-decomposition--a13-family`](linux-first-runtime-resumption/slice-and-task.md#a1-bounded-packet-decomposition--a13-family). | — | — | — | — | — |
| **A1.3 — held historical public/successor fence** | Canonical content: [`linux-first-runtime-resumption/slice-and-task.md#a1-bounded-packet-decomposition--a13-family`](linux-first-runtime-resumption/slice-and-task.md#a1-bounded-packet-decomposition--a13-family). | — | — | — | — | — |
| **A1.3-P1 — atomic public adoption and startup/post-turn completion** | Canonical content: [`linux-first-runtime-resumption/slice-and-task.md#a1-bounded-packet-decomposition--a13-family`](linux-first-runtime-resumption/slice-and-task.md#a1-bounded-packet-decomposition--a13-family). | — | — | — | — | — |
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, then close A1 without changing projection policy or settlement ownership. | Only `build_auto_attach_launch_plan` and directly required plan integration in `agent_runtime/auto_attach.rs`; focused auto-attach producer/consumer tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, and idempotent applied result. | No auto-attach policy, eligibility, claim, or settlement redesign; no router responsibility expansion; no endpoint/path redesign; no A2 demotion; no subsequent-slice work. | Prove manual and auto-attach cannot substitute or double-apply an intent, retry/restart converges, every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` pass on real CLI and REPL paths; only then may A1 close. | No A1 gate remains claimable until this packet's final wall passes; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows remain unresolved for A2/A3, and later sibling-seam gates remain out of scope. |

#### Corrected A1.2 internal dependency split

Canonical content: [`a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split`](a1-2-earlier-histories/slice-and-task.md#corrected-a12-internal-dependency-split).

#### A1.1 internal green subpacket decomposition

Canonical content: [`a1-2-earlier-histories/slice-and-task.md#a11-internal-green-subpacket-decomposition`](a1-2-earlier-histories/slice-and-task.md#a11-internal-green-subpacket-decomposition).

### A1.1d internal review checkpoints

Canonical content: [`a1-2-earlier-histories/slice-and-task.md#a11d-internal-review-checkpoints`](a1-2-earlier-histories/slice-and-task.md#a11d-internal-review-checkpoints).

### Historical A1 sequencing checkpoint

Canonical content: [`a1-2-earlier-histories/slice-and-task.md#historical-a1-sequencing-checkpoint`](a1-2-earlier-histories/slice-and-task.md#historical-a1-sequencing-checkpoint).

## Track B — World-dispatch receipts, supervision, and cancel

### B1 receipt core and activated-store allowlist

Canonical content: [`b1-b2-1/slice-and-task.md#b1-receipt-core-and-activated-store-allowlist`](b1-b2-1/slice-and-task.md#b1-receipt-core-and-activated-store-allowlist).

#### B2.1-3 exact replay/startup allowlist correction

Canonical content: [`b1-b2-1/slice-and-task.md#b21-3-exact-replaystartup-allowlist-correction`](b1-b2-1/slice-and-task.md#b21-3-exact-replaystartup-allowlist-correction).


| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B0 through B1/B2.1 joint closeout — canonical family rows moved** | [Canonical B1/B2.1 packet rows](b1-b2-1/slice-and-task.md#b1-receipt-core-and-activated-store-allowlist) | — | — | — | — | — | — | — | — | — |
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
| **B3.1 — canonical family row moved** | [Canonical B3.1 packet row](b3-1-c1/slice-and-task.md#b31-packet-row) | — | — | — | — | — | — | — | — | — |
| **B3.2 — Remaining retained receipt, messaging, and lifecycle completion** | Complete model-facing retained receipts, host-to-worker envelopes, delivery semantics, accepted-turn lifecycle/park behavior, durable resolution of abandoned `SlotReserved` and `AuthorityRegistrationHead` admissions, reconciliation of post-R0/pre-transport and ambiguous transport-claim states, and full creation-observer restart reconciliation without changing B3.1 event truth or B3.2a's already-landed registration/admission identities. | WorldWorkerMessagingProtocol owns message semantics; RetainedWorkerRuntime owns worker lifecycle and every durable admission-state resolution. | `02` MessagingProtocol + RetainedWorkerRuntime rows; `04` retained receipt/manifest, B3.2a admission state, and deferred admission-recovery contract; full messaging and lifecycle designs; `05` `RG-ADMISSION-01`. | Supervisor; SteeringPolicyEngine; ObligationLedger; B4 cancel. | `orchestrator_world_dispatch.rs`; `agent_runtime/{dispatch_contract,session,state_store}.rs`; `agent_runtime/retained_worker_runtime.rs` only for deferred durable admission resolution and reconciliation; `world-service/member_runtime.rs`; focused retained lifecycle tests. | No auto-attach, no rewrite of B3.2a registration identity or exact terminal truth, no obligation ownership transfer, no liveness-inferred admission resolution, and no dispatch narrowing beyond carrying E2 snapshot/cap refs. | Continue returns the B1 accepted active-run receipt completed by E2 through B2.2; host-to-worker messages preserve exact identity/causation; clean turn parks without losing worker continuity. Exact retry rejoins the same admission. Authorized durable resolution exact-joins issuer request, admission record, session, authority, policy, participant, and state; reconciles partial R0 registration or transport ambiguity instead of deleting or blindly rolling back; terminalizes idempotently; and releases the live cap only after exact terminal durable proof, permitting the next eligible queued request. Crashes before/after resolution converge across restart without PID, timeout, caller, helper, socket, endpoint, observer, or process-liveness inference. Creation-observer interruption/restart also reconciles without liveness inference; failure is durable and exact. | Remaining `RG-RECEIPT-02`, `RG-MSG-01`, `RG-BASE-04`, `RG-WORKER-EXIT-01`, and the durable-state clauses of `RG-ADMISSION-01`. | B3.2a, A1, A2/A3, E2, B2.2, B3.1, and C1. | B4 user/tool-facing admission inspect/cancel and worker cancel/inspect/stop outcomes, C2/C3, and auto-attach. |
| **B4 — Receipt-targeted cancel/inspect/stop** | Resolve control against active receipts and worker manifests with distinct durable outcomes, and expose exact inspection and cancellation of pending Spawn admission through WorldDispatchControl without taking ownership of RetainedWorkerRuntime state transitions. | WorldDispatchControl owns the user/tool-facing verb and resolves exact targets; ReceiptRegistry owns immutable accepted identity; Supervisor owns active observation and terminal truth; RetainedWorkerRuntime owns worker stop lifecycle and durable admission-state resolution. | `04` cancel categories, deferred admission-recovery contract, and supervisor cancellation rules; `02` WorldDispatchControl/ReceiptRegistry/RetainedRuntime rows; `05` cancel/closeout rows and `RG-ADMISSION-01`. | HostSessionAuthority; Supervisor; private transport episode status. | `orchestrator_world_dispatch.rs`; `agent_runtime/{state_store,dispatch_contract,control}.rs`; world-service cancel seam; control tests. | No weakening exact identity, no treating no active run as stale linkage, no liveness-based admission resolution, and no stop/cancel/admission-abandonment conflation. | Same-turn continue→cancel targets active run. Pending Spawn inspection/cancellation exact-joins the admission identity and returns distinct outcomes for cancelled-before-registration, registered-before-transport, transport-ambiguous/cancel-pending, already routable/terminal, invalid target, ambiguity, and policy denial. Repeated terminal resolution is idempotent; no-active, terminal, unreachable, invalid, mismatch, ambiguity, and policy denial for active work remain distinct. | `RG-CANCEL-01`, `RG-CANCEL-02`, `RG-CLOSE-01`, `RG-BASE-04`, and the user/tool-facing clauses of `RG-ADMISSION-01`; B4 cancel clause of `RG-OBS-01`. | B2.2 and B3.2. | C2/C3 and unrelated lifecycle/policy work. |

### B1/B2.1-0 recorded prerequisite result

Canonical content: [`b1-b2-1/slice-and-task.md#b1b21-0-recorded-prerequisite-result`](b1-b2-1/slice-and-task.md#b1b21-0-recorded-prerequisite-result).

## B1/B2.1 joint production closeout recorded result

Canonical content: [`b1-b2-1/slice-and-task.md#b1b21-joint-production-closeout-recorded-result`](b1-b2-1/slice-and-task.md#b1b21-joint-production-closeout-recorded-result).

## B3.1 recorded result

Canonical content: [`b3-1-c1/slice-and-task.md#b31-recorded-result`](b3-1-c1/slice-and-task.md#b31-recorded-result).

## Track C — Obligations, inbox, auto-attach, and router attach

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C1 — canonical family row moved** | [Canonical C1 packet row](b3-1-c1/slice-and-task.md#c1-packet-row) | — | — | — | — | — | — | — | — | — |
| **C2 — Inbox and auto-attach projections** | Make inbox and attach eligibility pure projections over obligation state and effective policy. | InboxProjection and AutoAttachProjection. | `02` projection rows; notification inbox design; auto-attach trigger design. | ObligationLedger; CompatibilityReadModel; SteeringPolicyEngine. | `agent_runtime/{host_inbox,obligation_ledger,auto_attach,state_store}.rs`; `host_inbox_materialization.rs`; projection tests. | No host process launch, no worker action, and no removal of compatibility ingress without migration proof. | Deleting/rebuilding projections does not lose obligation truth; session-coalesced claim is deterministic; wrong-host and policy denial fail closed. | Projection clauses of `RG-OBL-02`, `RG-ATTACH-01`, `RG-BASE-03`. | A1, A2/A3, C1, and B4; B4 transitively supplies B2.2/B3.2/E2. | C3 router launch and unrelated worker controls. |
| **C3 — Router ownership restoration** | Consume attach-eligible claims, restore one sanctioned host episode, settle the claim, and stop. | RouterAttachTrigger for triggering; HostSessionAuthority for the attach transition. | `02` RouterAttachTrigger row; router responsibilities/non-responsibilities; `04` HostExecutionEpisodeV1. | HostSessionAuthority; SurfaceAdapter; AutoAttachProjection. | `agent_runtime/auto_attach.rs`; bounded router entrypoint; helper launch adapter; router tests. | No prompt replay, approval/answer/fork/continue, or always-running backend assumption. | Router produces one attach outcome per session claim and cannot invoke worker-control verbs; manual reattach coexists without duplicate ownership. | `RG-ATTACH-01`, `RG-ATTACH-02`, `RG-AUTH-01`. | C2 and A1/A2. | Worker-control, receipt, and unrelated policy work. |

### C1 recorded result

Canonical content: [`b3-1-c1/slice-and-task.md#c1-recorded-result`](b3-1-c1/slice-and-task.md#c1-recorded-result).

## Track D — UAA execution envelope and side-effect mediation

Staging rule: the managed gateway secure-FD carrier is already landed and must be preserved. Until D2 and direct world-UAA adoption of that carrier land, D1 fail-closed behavior applies to world-scoped side-effect-capable or credential-requiring UAA operations that claim Substrate policy mediation. Existing direct world-UAA behavior may remain only behind an explicitly named and logged compatibility mode. Copied host credentials/config must be identified as a compatibility copy bridge. Compatibility mode must be excluded from `ContractCorrectAndProven` claims and from mediation, caging, gateway-adoption, or credential-handoff acceptance evidence.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **D1 — World adapter execution envelope** | Materialize and persist `WorldRuntimeAdapterExecutionEnvelopeV1` with guest entrypoint, projection identity, world binding, policy snapshot, broker requirement, credential posture, secret-handoff ref, and explicit mediation/compatibility posture. | `02` envelope + realization rows; `04` envelope and `LaunchTimeSecretHandoffV1` contracts; existing gateway auth-handoff internals/verification docs; Codex home/auth mapping design; config projection design | AgentConfigProjectionService; EffectivePolicyResolver; RetainedWorkerManifest; in-world Substrate gateway | `agent_runtime/{validator,dispatch_contract}.rs`; `execution/agent_inventory.rs`; `world-service/member_runtime.rs`; existing managed gateway launch/handoff types; `config/agents/*`; envelope tests | No per-operation broker yet; no host runtime behavior change; no reimplementation of the existing FD carrier; no unnamed credential fallback; no compatibility evidence counted as contract proof. | Host/world envelopes are distinct; world Codex cannot use host path; sibling workers cannot alias envelope/projection identity. Every world envelope records `SecureGatewayHandoff`, `CompatibilityCopyBridge`, or `NoCredentialsRequired`; secure posture joins the existing carrier to the exact gateway receiver and non-secret evidence; Codex receives that gateway's endpoint/session contract; compatibility copy is named/logged/non-promotable; credential-requiring mediated mode fails closed without exact adoption. | `RG-UAA-01`, `RG-UAA-02`, `RG-CONFIG-01`, `RG-CONFIG-03`, `RG-CONFIG-04` |
| **D2 — WorldCommandExecutionBroker** | Mediate every world-UAA side-effect intent through the accepted `PolicySnapshotV3` and existing world-service enforcement path. | `02` broker row; `04` envelope and snapshot acceptance; `01` invariants 8–11 | EffectivePolicyResolver; runtime-family adapters; world-service enforcement | `crates/gateway`; relevant `agent-api-*`; `world-service` execute/guard/fs/network paths; bounded shell adapter glue | No parallel sandbox; no env-only enforcement; no shell-only claim while edit/MCP/write channels bypass. | Shell, apply/edit, direct write, write-capable tool/MCP, process, and network channels are brokered or disabled; snapshot mismatch/unbrokered intent fails closed. | `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-02`; D2 broker clause of `RG-OBS-01` |
| **D3 — Codex/UAA end-to-end closure** | Prove world Codex/UAA placement, adoption of the existing secure gateway credential bootstrap, and side-effect mediation, including non-zero turn closeout and diagnostics. | `05` credential carrier/adoption, UAA caging, external-sandbox, and non-zero rows; `04` secret handoff + supervisor terminal rules; Codex mapping design | Supervisor; broker; envelope; config projection; in-world gateway | targeted Codex/UAA runtime integration tests and smoke harnesses; only defects exposed within D1/D2/E3 boundaries | No broad provider feature expansion; no FD-carrier rewrite without a separately proven defect; no advisory-only success; no copied-credential compatibility evidence counted as proof; no suppression of non-zero exits. | Real worker turn points Codex at the managed gateway, joins its one-time credential consumption to the exact envelope, creates no copied secret file or inherited child FD, proves caged/uncaged policy contrast and no bypassing edit/tool channel, records durable failure on non-zero, and retains session authority; traces contain handoff state but no secret material. Final `RG-OBS-01` integration closure additionally requires the already-landed B0/B1/B2.1/B3.1/C1/B4, E2/E3, and D2 clause evidence to join in one trace proof. | `RG-UAA-03`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-WORKER-EXIT-01`, final integration clause of `RG-OBS-01` |

## Track E — Dispatch-scoped policy narrowing and config projection

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E1 — Restricted world_fs narrowing** | Accept a request-scoped restricted `PolicyPatch.world_fs`, validate path-containment monotonicity, and materialize a canonical narrowed snapshot. | `02` EffectivePolicyResolver + NarrowingPatch rows; `04` narrowing contract/rules; steering policy matrix capability section | WorldCommandExecutionBroker; receipt acceptance; agent inventory overlay logic | `crates/broker`; `execution/{policy_model,policy_snapshot,agent_inventory}.rs`; transport API policy types; resolver tests | No new filesystem policy model; no broadening dimensions; no receipt/manifests yet. | Gate=false rejects; gate=true accepts only narrowing; file-under-directory containment works; escapes/symlinks/broadening fail closed. | `RG-POLICY-01`, `RG-POLICY-02` |
| **E2 — Policy commitments on work and workers** | Build immutable active-run snapshots and retained-worker caps linked to B1 acceptance records; recompute future turns as parent ∧ cap ∧ turn patch before B2.2/B3.2 expose final receipts. | `04` acceptance/receipt/manifest and immutable snapshot rules; `02` ReceiptRegistry/RetainedRuntime rows | B1 acceptance records; B3.1 event truth; B2.2/B3.2 consumers; E1 resolver; fork lifecycle | receipt/manifest persistence; `orchestrator_world_dispatch.rs`; retained lifecycle code; policy tests | No mutation of accepted snapshots; no automatic worker broadening; no config rendering. | Final receipts/manifests reference their B1 acceptance record and record immutable hash/ref/revision/reason; parent narrowing affects future turns; parent broadening does not widen worker; fork inherits cap. | `RG-POLICY-03`, `RG-RECEIPT-02`, `RG-CANCEL-01`; E2 policy clause of `RG-OBS-01` |
| **E3 — AgentConfigProjectionService and gateway adoption** | Make logical/effective/native config projection first-class per retained worker and point world Codex at the existing in-world gateway secure-FD credential/session boundary. | `02` config projection + envelope + realization rows; `04` envelope projection, existing-carrier adoption note, and `LaunchTimeSecretHandoffV1`; existing gateway auth-handoff internals/verification docs; config projection, Codex mapping, and workspace overlay designs | Envelope; retained manifest; EffectivePolicyResolver; in-world gateway; RuntimeFamilyRealizationAdapter | `execution/agent_inventory.rs`; `crates/codex`; `world-service/member_runtime.rs`; existing in-world gateway endpoint/session integration points; bounded projection module; projection tests | No reimplementation of the landed FD carrier; no ambient runtime file as authority; no secret payload in projected files, manifests, traces, or UAA child FDs; no workspace-sync policy decision. | Sibling workers get isolated non-secret projections; `CODEX_HOME`, `config.toml`, and provider wiring can be rebuilt from Substrate truth plus accepted policy; Codex uses the exact managed gateway whose credentials arrived through the existing one-time FD carrier; copied auth/config runs only as named/logged/non-promotable compatibility; narrowed hints never substitute for enforcement. | `RG-CONFIG-01`, `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`; E3 credential-handoff clause of `RG-OBS-01` |
| **E4 — Host-visible write/sync contract** | Freeze and prove when world writes are immediately host-visible, isolated, or explicitly reconciled under narrowed policy. | `05` host-visible sync row; `04` monotonicity and snapshot rules; workspace overlay model sync boundary | E1–E3; world-service fs enforcement; broker | `world-service` fs/overlay execution; relevant `crates/world*`; workspace-sync code/tests/docs | No silent best-effort copying; no treating command success as host-visible proof; no policy broadening to make sync pass. | Matrix proof covers `host_visible=true`, full isolation, narrowed write allowlists, retained turns, denied writes, and explicit reconciliation semantics. | `RG-SYNC-01`, `RG-POLICY-02`, `RG-UAA-03` |

## Slice closeout minimum

A0 closeout records inventory coverage, classification evidence, proposed owners, and first migration target in `02-seam-crosswalk.md`.

Every implementation slice closeout records:

1. the crosswalk row(s) changed;
2. the authority decision moved;
3. the production call path now using it;
4. the enforcement point, or `not applicable` with justification;
5. exact unit/integration/smoke evidence;
6. the permanent regression gate added or preserved; and
7. why sibling seams were not accidentally widened.

Do not promote a crosswalk row merely because its slice landed. Promote only after the seam-level four-part landing rule is satisfied.

## A1.1d-5R2-2F0-HC packet insertion and authorization

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-hc-packet-insertion-and-authorization`](a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-hc-packet-insertion-and-authorization).

## A1.1d-5R2-2F0 historical differential authority gate

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-historical-differential-authority-gate`](a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0-historical-differential-authority-gate).

## A1.1d-5R2-2F historical F3/F4 correction and allowlist

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f-historical-f3f4-correction-and-allowlist`](a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f-historical-f3f4-correction-and-allowlist).

### Mandatory F3/F4 entry and exit gates

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#mandatory-f3f4-entry-and-exit-gates`](a1.1d-5r2-2f/slice-and-task.md#mandatory-f3f4-entry-and-exit-gates).

## A1.1d-5R2-2F5-PD — Non-mutating nested doctor boundary

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f5-pd--non-mutating-nested-doctor-boundary`](a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f5-pd--non-mutating-nested-doctor-boundary).

### F5-PD required regression wall

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#f5-pd-required-regression-wall`](a1.1d-5r2-2f/slice-and-task.md#f5-pd-required-regression-wall).

### F5-PD mandatory stops

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#f5-pd-mandatory-stops`](a1.1d-5r2-2f/slice-and-task.md#f5-pd-mandatory-stops).

## A1.1d-5R2-2F — completed phase record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f--completed-phase-record`](a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f--completed-phase-record).
## A1.1d-5R2-2-B1 broad-wall provenance phase

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/slice-and-task.md#a11d-5r2-2-b1-broad-wall-provenance-phase`](a1.1d-5r2-2-renewed-closeout/slice-and-task.md#a11d-5r2-2-b1-broad-wall-provenance-phase).

## A1.1d-5R2-2 renewed closeout publication phases

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/slice-and-task.md#a11d-5r2-2-renewed-closeout-publication-phases`](a1.1d-5r2-2-renewed-closeout/slice-and-task.md#a11d-5r2-2-renewed-closeout-publication-phases).

## R2-2 remediation insertion before renewed closeout (historical RP0-RP4 plan)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/slice-and-task.md#r2-2-remediation-insertion-before-renewed-closeout-historical-rp0-rp4-plan`](a1.1d-5r2-2-renewed-closeout/slice-and-task.md#r2-2-remediation-insertion-before-renewed-closeout-historical-rp0-rp4-plan).
## A1.1d-5R3 authoritative implementation index

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-authoritative-implementation-index`](a1.1d-5r3/slice-and-task.md#a11d-5r3-authoritative-implementation-index).

### `A1.1d-5R3-HOME`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-home`](a1.1d-5r3/slice-and-task.md#a11d-5r3-home).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-manifest`](a1.1d-5r3/slice-and-task.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-linux`](a1.1d-5r3/slice-and-task.md#a11d-5r3-linux).

### `EVIDENCE:R3-LINUX-IMP-01` and `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/slice-and-task.md#evidencer3-linux-imp-01-and-a11d-5r3-linux-closeout`](a1.1d-5r3/slice-and-task.md#evidencer3-linux-imp-01-and-a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-mac`](a1.1d-5r3/slice-and-task.md#a11d-5r3-mac).

### `EVIDENCE:R3-MAC-IMP-01` and `A1.1d-5R3-MAC-CLOSEOUT`

Canonical content: [`a1.1d-5r3/slice-and-task.md#evidencer3-mac-imp-01-and-a11d-5r3-mac-closeout`](a1.1d-5r3/slice-and-task.md#evidencer3-mac-imp-01-and-a11d-5r3-mac-closeout).

### `A1.1d-5R3-WIN`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-win`](a1.1d-5r3/slice-and-task.md#a11d-5r3-win).

### `EVIDENCE:R3-WIN-IMP-01` and `A1.1d-5R3-WIN-CLOSEOUT`

Canonical content: [`a1.1d-5r3/slice-and-task.md#evidencer3-win-imp-01-and-a11d-5r3-win-closeout`](a1.1d-5r3/slice-and-task.md#evidencer3-win-imp-01-and-a11d-5r3-win-closeout).

### `A1.1d-5R3-UNIX`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-unix`](a1.1d-5r3/slice-and-task.md#a11d-5r3-unix).

### Final native evidence task sequence

Canonical content: [`a1.1d-5r3/slice-and-task.md#final-native-evidence-task-sequence`](a1.1d-5r3/slice-and-task.md#final-native-evidence-task-sequence).

### `A1.1d-5R3-CLOSEOUT`

Canonical content: [`a1.1d-5r3/slice-and-task.md#a11d-5r3-closeout`](a1.1d-5r3/slice-and-task.md#a11d-5r3-closeout).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/slice-status.md#r3-implementation-status-append`](a1.1d-5r3/slice-status.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/slice-status.md#a11d-5r3-manifest`](a1.1d-5r3/slice-status.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/slice-status.md#a11d-5r3-linux`](a1.1d-5r3/slice-status.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/slice-status.md#a11d-5r3-linux-closeout`](a1.1d-5r3/slice-status.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/slice-status.md#a11d-5r3-mac`](a1.1d-5r3/slice-status.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/slice-and-task-status.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/slice-and-task-status.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN packet status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07`](r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-evidence-recovery-plan-packet-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current evidence-command and successor rule (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-evidence-recovery-plan-current-evidence-command-and-successor-rule-2026-08-07`](r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-evidence-recovery-plan-current-evidence-command-and-successor-rule-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07).
## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

Canonical content: [`r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10`](r3-mac-evidence-recovery/slice-and-task-status.md#aux-r3-mac-system-keychain-software-signer-correction-2026-08-10).
## R3 macOS retirement/recovery serial-gate amendment (2026-08-13; docs-only)

Canonical content: [`r3-mac-evidence-recovery/slice-and-task-status.md#r3-macos-retirementrecovery-serial-gate-amendment-2026-08-13-docs-only`](r3-mac-evidence-recovery/slice-and-task-status.md#r3-macos-retirementrecovery-serial-gate-amendment-2026-08-13-docs-only).
## Current Linux-first scheduling reset (2026-08-20; controlling)

Canonical content: [`macos-dev-parity/slice-and-task.md#current-linux-first-scheduling-reset-2026-08-20-controlling`](macos-dev-parity/slice-and-task.md#current-linux-first-scheduling-reset-2026-08-20-controlling).
