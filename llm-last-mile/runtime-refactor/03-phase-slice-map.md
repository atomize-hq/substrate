# Phase and Slice Map

## Sequencing rules

Each slice is independently reviewable. A0 is a diagnostic inventory; every later implementation slice must move one authority boundary plus proof. A slice may read sibling context without editing sibling ownership.

Hard dependency spine:

```text
A0 -> A1.1e -> B0 -> B1-3a/B1-3b receipt core -> B2.1-1/2/3 ------------------+
              \-> A1.2a current-authority protocol -> A1.2a-WB correction     |
                  -> A1.2a-S bounded Start                                    |
                  -> B1/B2.1-R0 -> B3.2a -> B3.2a-WA ------------------------+
                                                                               -> B1/B2.1-0
                                                                               -> joint closeout
                                                                               -> B3.1 -> C1 -> A1.2b
                                                                               -> A1.3 -> A1.4 -> A1
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
| **A1 — HostSessionAuthority facade** | Establish the only API that resolves exact durable session/caller/binding truth, owns revision-checked posture transitions, preserves strict V1 intent reads, uses strict `HostSessionTransitionIntentV2` only for production Start, and later uses an independently reviewed strict V3 successor intent for real `Attach` and `ResumeOneTurn` paths. | `01` Authority map + invariants 1–3; `02` HostSessionAuthority and helper-plan/mode/parent-control/private-stop/auto-park rows; `04` `DurableSessionAuthorityV1` + strict V1/V2 Start and later V3 successor-intent contract; `DESIGN-host-orchestrator-world-dispatch-contract.md` exact identity rules | StateStore; CompatibilityReadModel; HostExecutionEpisode; WorldDispatchControl; AutoAttachProjection | Bounded modules under `crates/shell/src/execution/agent_runtime/host_session_authority/`; bounded facade types/re-exports and integration only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only as the bounded persistence/integration surface; bounded explicit-bootstrap-home entry points only in `crates/common/src/paths.rs` and `crates/shell/src/execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; in `agent_runtime/control.rs`, only `HiddenOwnerHelperLaunchPlan` and its write/load/remove/launch integration; in `execution/agents_cmd.rs`, only `Start`/`Attach`/`ResumeOneTurn` intent issuance and `run_owner_helper` consumption; in `repl/async_repl.rs`, only owner-helper validation/application and real REPL adoption; in `agent_runtime/auto_attach.rs`, only `build_auto_attach_launch_plan` and directly required plan integration; focused tests | No general `HostExecutionEpisode` demotion; no endpoint/path redesign; no helper removal; no auto-attach policy, eligibility, or settlement redesign; no receipt/supervisor work; no config/policy/inventory precedence, merge, or interpretation changes; no unrelated lifecycle, transport, or schema cleanup; no unrelated StateStore cleanup or general persistence extraction. | All real CLI and REPL transitions route through `HostSessionAuthority`: Start uses a durable revision-bound V2 intent, while Attach/Resume use the later V3 successor intent; exact applied retry joins without reapplying; helper loss/restart reconciles; stale, substituted, mismatched, expired, or conflicting replay fails closed; stale observations cannot overwrite newer authority; `RG-BASE-01` and `RG-BASE-02` remain green. Do not claim A1 before A1.1–A1.4 and the final proof wall complete. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02` |
| **A2 — HostExecutionEpisode demotion** | Represent REPL/helper/toolbox/recovered processes as episodes whose PID/socket/readiness data are observations only. | `01` invariants 1–2; `02` SurfaceAdapter row; `04` HostExecutionEpisodeV1; `DESIGN-internal-toolbox-transport-and-session-binding.md` | HostSessionAuthority; InternalToolboxTransport; RouterAttachTrigger | `agent_runtime/control.rs`; `execution/agents_cmd.rs`; `repl/async_repl.rs`; episode-focused tests | No deletion of `__owner-helper`; no transport protocol rewrite; no auto-attach redesign. | Killing/orphaning an episode leaves durable session truth intact; stale episode updates are revision-rejected; transport classification is explicit. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01` |
| **A3 — Persistence and compatibility split** | Reduce StateStore to atomic persistence/schema evolution and isolate diagnostic compatibility reads from new authority writes. | `02` StateStore + CompatibilityReadModel rows; `01` authority map; `04` durable revision rules | HostSessionAuthority; InboxProjection; receipt persistence | `agent_runtime/state_store.rs`; a bounded new facade/module under `agent_runtime/`; persistence/read-model tests | No wholesale database rewrite; no conversion of pre-A1 authority artifacts into A1 authority; no behavior changes outside moved ownership. | New authority writes bypass compatibility projection; torn-root/unsupported-state diagnostics remain read-only; newer revisions always win. | `RG-AUTH-02`, `RG-BASE-01`, `RG-BASE-03` |

### A1 bounded packet decomposition

A1 is one independently reviewable slice implemented in the following order. Each packet must remain green and reviewable before the next begins; prerequisites established by an early packet do not satisfy later production-adoption or slice-closeout gates.

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.1 — authority-store core** | Establish only the authority-store substrate: persisted normalized bootstrap-home binding, exact fresh/pending/existing/unsupported/corrupt classification, operation-bound temp reconciliation, exact object/key persistence, immutable greenfield namespace certification, fail-closed pre-A1 authority-state detection, exact durable resolution, and generic root/authority revision-CAS. Keep `StateStore` as atomic persistence. | Bounded modules under `agent_runtime/host_session_authority/`; bounded facade types/re-exports only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only for the minimum persistence/integration choke points that exclude legacy/direct writers; explicit-home API parameterization only in `crates/common/src/paths.rs` and `execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; focused codec, trusted-filesystem, bootstrap/reconciliation, writer-exclusion, resolution, persistence-conflict, and cross-process tests. | `StateRootV1`; `AuthorityStoreInitializationV1`; `GreenfieldNamespaceCertificateV1`; `DurableSessionAuthorityV1`; `CanonicalDirectoryV1`; immutable typed object schemas/refs; key registry/files; operation-bound temp grammar; expected `root_revision`/`authority_revision` for CAS. | No migration, quarantine, evidence, dual-write, or compatibility adoption; no production `ExpectedAbsent` acceptance; no Start reservation, transition-intent issuance/application, participant allocation, or Start-origin authority birth; no helper-plan or consumer adoption; no episode demotion; no resolver precedence, policy/config/inventory interpretation; no unrelated StateStore cleanup or general persistence extraction. | Write failing tests for genuinely absent versus malformed/partial/unreadable/unsafe/unsupported stores, interrupted initialization and copied-home mismatch, exact bootstrap-home reuse, crashes before/after temp fsync and rename, pending-key rename/directory-fsync recovery, key create/rotate/retire crash windows, first-use kind/version directory create/fsync crashes, object rename before root commit with retained-orphan restart/exact retry, safe empty versus present/unreadable/symlinked pre-A1 collections, ancestor symlink/unsafe ACL/cross-device and scan-to-publication replacement, pre-A1 writers serialized before activation and rejected after marker/root publication, cross-kind object substitution, exact resolution, stale revision, and cross-process CAS. Exit when these primitives are deterministic and green; this packet alone cannot create or claim a production Start. | Production Start/`ExpectedAbsent`; all transition intents and `RG-AUTH-03`; real CLI/REPL adoption; helper-loss reconciliation; auto-attach adoption; complete `RG-AUTH-01`/`RG-AUTH-02`; A1 completion. |
| **A1.2 — intent issuance/claim/application** | Add the A1.2-owned durable protocol under `HostSessionAuthority` while preserving landed V1 intent/state bytes as read-only: A1.2a supplies an A1.1e-dependent but B1-independent strict V2 Start intent with no B1-owned type, and A1.2b later supplies a separately reviewed strict V3 root/intent/state extension for Attach/Resume and post-turn fields after B1/B3.1/C1 types are available. Together they cover greenfield-certified Start reservation/issuance/application, exact retry, startup ownership, successor issuance/application, post-turn reconciliation, ledger-owned obligation-cut consumption, and retained transport reprojection; keep the plan a transport projection. The aggregate packet includes parked-successor `Attach`/`ResumeOneTurn`, but the normative split below assigns only greenfield Start establishment/read to A1.2a before the joint closeout; A1.2b retains successor/post-turn mechanics and begins only after the joint closeout, B3.1, and C1 establish its complete semantic-cut prerequisite. | Authority-store core files above; only `HiddenOwnerHelperLaunchPlan` plus its write/load/remove/launch integration in `agent_runtime/control.rs`; focused V1/V2 Start and later V3 successor discrimination, Start reservation/birth, intent lifecycle, serialization, persistence, helper-loss, parked-successor, input-handoff, startup-ownership, post-turn-event/input terminalization, pending obligation-cut consumption, payload-retention/reprojection, and plan-commitment tests. Runtime transport identity, receipt acceptance, supervisor journaling, retained-event semantics, and the canonical snapshot producer remain in B0/B1/B2.1/B3.1/C1 and are not A1.2 implementation allowances. | Exact greenfield certificate and empty pre-A1 collections; unique `intent_id`; intent/state revision; `Start`/`Attach`/`ResumeOneTurn`; exact enumerated postures; `ExpectedAbsent` or exact authority revision/hash; reservation ownership; identity/home/workspace/world/descriptor/object commitments; expiry; canonical payload and transport refs/hashes; `Issued -> Claimed -> Applied` plus terminal `Rejected`/`Expired`; immutable application result; input handoff; actor-bound startup evidence/result; actor-bound post-turn protocol event/completion/result; `AwaitingObligationCut`; ledger-owned Complete snapshot; transport release. | No V1 shape mutation/defaulting; no legacy-state conversion or compatibility-derived absence; no CLI/REPL consumer switch yet; no runtime frame/event identity generation; no receipt acceptance or stream observation; no retained-message semantics; no obligation enumeration/classification/creation/resolution/reinterpretation/overwrite and no claim that event materialization is complete; no C1 implementation; no destructive-read consumption; no endpoint/path redesign; no helper removal; no general transport cleanup; no PID/helper/handle/prompt-derived transition eligibility or transport/process inference as terminal protocol evidence. | Prove strict V1 bytes remain identical/read-only, production Start uses only V2 without B1 imports, and Attach/Resume/post-turn uses only the later strict V3 extension. Prove `Attach` and `ResumeOneTurn` start from exact current parked or other enumerated authority, preserve session lifecycle identity and exact world binding, revision-authorize one successor lineage, and never derive eligibility from PID, helper, handle, readiness, or prompt state. Prove Start/Attach transport cannot release before exact actor-bound startup evidence resolves ownership, every Resume terminal outcome joins an exact actor-bound post-turn event/completion/result and closed input state, exact retries work after payload deletion, terminal Resume can close without a snapshot, and resumable completion persists `AwaitingObligationCut` while retaining transport. A Complete ledger cut must be scoped to exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, authenticated intent/revision/payload commitment and distinct transition run, and authority/event watermark, then select the ledger-owned disposition without A1.2 scanning, inference, or reclassification. Exact stale successor retries fail or join exactly; failure after application never restores a stale snapshot. Because C1 is now a prerequisite, the full packet exits only when that unchanged Complete result drives the one allowed revisioned post-turn application and all A1.2 proofs pass. | Real Start/Attach/Resume consumer adoption; bounded auto-attach adoption; full real-path `RG-AUTH-03`; `RG-BASE-01`; A1 completion. |
| **A1.3 — public/successor adoption and startup-result completion** | Route public CLI issuance, `run_owner_helper` consumption, owner-helper application, and real REPL `Attach`/`ResumeOneTurn` through the authority and claimed intent; finish public Start adoption around the bounded A1.2a-S internal bootstrap without changing its applied identity. This packet owns the exact startup/post-turn protocol-event adapter, resolves the still-Pending A1.2a-S startup result, performs real-path parked-successor adoption, and closes `RG-BASE-01`. | The A1.2 plan integration; only the named intent issuance/consumption regions in `execution/agents_cmd.rs`; only owner-helper validation/application, exact startup/post-turn event submission, remaining public/real REPL adoption, and explicit bootstrap-home threading in `repl/async_repl.rs`; only the A1.1 explicit-home resolver entry points needed by those named CLI/REPL paths; focused CLI, helper, and REPL integration tests. | Mode-specific preconditions; exact caller/source/target lineage; bootstrap-home/workspace/world binding; typed descriptor/attach/resume/policy/input refs; claim ownership; authority/intent revisions; applied result; exact actor-bound ownership-acknowledgement/pre-ownership rejection/failure event identity and result; exact actor-bound post-turn event ID/sequence/outcome/reason and one-turn disposition. | No rewrite of A1.2a-S initial identity or bound capability, resolver precedence, policy/config/inventory interpretation, or broad REPL lifecycle cleanup; no general episode model rewrite or relabeling of readiness/PID/helper/socket/timeout/EOF/local errors as startup or post-turn protocol evidence; no private endpoint/path redesign; no receipt, supervisor, or member-runtime work. | Prove the bounded internal Start's Pending ownership resolves only from exact actor evidence; a public turn from parked truth issues/applies `ResumeOneTurn`; explicit reattach issues/applies `Attach`; and real helper/REPL launch occurs only after durable application. Prove exact `OwnershipAccepted` accepts ownership, exact `RuntimeCreationRejected`/`StartupFailedBeforeOwnership` terminally reconcile with matching actor/evidence/result identity, and timeout/drop/EOF/readiness/process posture or local adapter error remains Pending. Every applied Resume terminal outcome has an immutable exact-scope protocol event, matching completion plus post-turn application, and closed input terminalization; exact retry joins; the exact public start → turn/reattach → stop path becomes green. Retain existing negative tests for helper loss, stale revision, ambient-home/CWD reread, substitution, role mismatch, duplicate application, conflicting startup evidence, and substituted post-turn events. | Bounded auto-attach producer adoption; complete A1 regression closure; A1 completion. |
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, then close A1 without changing projection policy or settlement ownership. | Only `build_auto_attach_launch_plan` and directly required plan integration in `agent_runtime/auto_attach.rs`; focused auto-attach producer/consumer tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, and idempotent applied result. | No auto-attach policy, eligibility, claim, or settlement redesign; no router responsibility expansion; no endpoint/path redesign; no A2 demotion; no subsequent-slice work. | Prove manual and auto-attach cannot substitute or double-apply an intent, retry/restart converges, every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` pass on real CLI and REPL paths; only then may A1 close. | No A1 gate remains claimable until this packet's final wall passes; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows remain unresolved for A2/A3, and later sibling-seam gates remain out of scope. |

#### Corrected A1.2 internal dependency split

The aggregate A1.2 row above remains the full ownership envelope, but its implementation and
dependency order are now split into two independently reviewable packets. This subsection is
normative where the older aggregate wording says that all of A1.2 starts after C1.

| Packet | Exact bounded goal | Explicit exclusions | Exit and dependency |
|---|---|---|---|
| **A1.2a — current-authority establishment/read prerequisite** | First perform the strict greenfield-only V1-to-V2 root upgrade, then implement greenfield-certified Start reservation, issuance, claim, single application/initial authority birth, crash reconciliation, and exact retry. Applied Start retains `claimant_attempt_id` and records startup ownership as `Pending` with the exact run, original application revision, and active participant; resolving that pending substate remains later work. Add one typed exact read result that joins the applied Start descriptor to store/session/active participant/lineage/workspace/world/revision/current-policy truth and exposes the accepted-home bound StateStore capability. | No defaulted/optional V2 field on V1, no B1-owned accepted-work/correlation type, no V3 schema, non-greenfield root conversion, Attach or ResumeOneTurn, startup-ownership resolution or post-turn episode reconciliation, obligation snapshot or disposition, transition correlation supplied to world work, any production consumer/adopter, helper plan, public CLI/REPL/auto-attach adoption, or B receipt/supervisor semantics. | Requires only landed A1.1e. A1.2a is landed and independently review-clean at `b5f2b4f8`; its Host/world-binding write/read correction is isolated in A1.2a-WB before adoption. A1.2a does not close A1.2, A1, or `RG-AUTH-03`. The preserved broad A1.2 checkpoint is not restored. |
| **A1.2a-WB — Host/world-binding validation correction** | Correct only the Start validation relation between descriptor/launch runtime placement and durable session world binding across issuance, application/persistence, and exact current-authority resolution. Descriptor scope must equal requested launch scope. Accept `Host + None`, `Host + Some(exact)`, and `World + Some(exact)`; reject `World + None` and every empty/malformed binding with zero mutation. Persist and return an exact present binding on the resulting durable session authority. | No schema, canonical JSON byte, golden-vector, V3, migration, compatibility bridge, persisted-object rewrite, new authority field, participant placement field, host-to-world placement reinterpretation, world capability/policy change, unrelated Start validation/adoption, or facade behavior outside `HostSessionAuthority::resolve_current_exact`. | Requires landed, review-clean A1.2a. The four-case write/read matrix, mismatch/malformed zero mutation, exact retry, changed-ID/generation conflict, applied and resolved `Host + Some` preservation, host-scoped participant placement, corrupt/substituted persisted combinations failing closed, and unchanged V1/V2 canonical bytes/fixtures are review-clean through `275f9fa2` under existing `RG-AUTH-01`, `RG-AUTH-03`, `RG-BASE-02`, and `RG-DIFF-01`; no new gate was added. |
| **A1.2a-S — bounded internal Start adoption prerequisite** | Make only the ordinary greenfield `prepare_host_orchestrator_runtime_from_resolved` path produce an unpersisted `GreenfieldHostStartProposalV1` carrying the accepted store/home, normalized workspace, descriptor, policy, and shell observation but no authoritative session/participant/run identity. Keep that proposal distinct from the existing fully materialized `PreparedAgentRuntime`. On the real dormant-host launch path, `dispatch_targeted_follow_up_turn` obtains the exact optional initial world binding and calls `apply_greenfield_host_start_from_authority`; before transport or legacy persistence, that adapter derives the issuer key from the greenfield store identity plus complete canonical Start plan and invokes A1.2a. Exact-join/apply Start allocates the authoritative identities and only then constructs the unchanged-shape `PreparedAgentRuntime`, compatibility session/participant projection, and immutable shared `RuntimeAuthorityContext::Bound`. The authority-managed branch of `start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx` skips activated-store legacy session/participant/snapshot writes while preserving launch, toolbox registration, events, blocking behavior, and in-memory observation. Startup ownership remains Pending. | No pending/partial `PreparedAgentRuntime`, persisted/random proposal authority, placeholder identity, fork/member preparation or remote-member consumer change, hidden-owner launch plan, public CLI Start, Attach, ResumeOneTurn, auto-attach, startup acceptance/failure reconciliation, post-turn behavior, lifecycle/posture transition, helper/CWD policy redesign, B receipt/supervisor, or authority construction outside the A1.2a facade. | Requires review-clean A1.2a-WB. Exact real-path `Some`/`None`, deterministic retry/conflict, pre-application escape exclusion, bound-toolbox, unchanged fork/member, zero activated legacy-write, and Pending-observation proof is independently review-clean through `2f2fecb3`. B1/B2.1-R0 is independently review-clean through `bb3eefba`; B3.2a plus B3.2a-WA are independently review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. A1.3 retains public/helper/successor adoption and all exact startup-result/post-turn gates. |
| **A1.2b — successor/post-turn completion** | First freeze and publish the strict V2-to-V3 root/intent/state extension that preserves each V2 Start and both R0 maps without importing B1/C1 types into A1.2a; then complete Attach/ResumeOneTurn issuance, claim/application, actor-bound startup/post-turn reconciliation, pending obligation-cut consumption, release, retry, and optional transition-correlation supply. Startup resolution may authenticate the original Start application revision only through a unique contiguous R0 registration-proof chain to exact current authority; acceptance leaves that authority unchanged and terminal reconciliation preserves every R0-added lineage member/ref. | No in-place V2 widening, arbitrary stale-revision acceptance, no B1/B2.1 implementation, no B3.1/C1 semantics, no public adoption, and no reinterpretation of ledger truth. | Starts only after the B1/B2.1 joint closeout, B3.1, and C1. Its full exit then enables A1.3. |

The exact A1.2a implementation allowlist is:

- `crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/mod.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store_schema.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store_tests.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/key_lifecycle.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/layout.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/object_persistence.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/reachability.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/store/platform/transaction.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/transition.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/transition_tests.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/trusted_fs.rs`

Only Start reservation/intent/application, initial authority birth, crash/exact-retry, and the
typed current-authority/bound read may change in those files. `control.rs`, helper-plan integration,
Attach, ResumeOneTurn, startup/post-turn reconciliation, obligation/correlation work, and every
aggregate A1.2 allowance outside this exact list remain forbidden during A1.2a. A required file
outside the list is `CrossDocumentChangeRequired` before implementation. A1.2a's root-schema work
must preserve strict V1 intent/state decoding and may only publish the exact eligible greenfield V2
conversion, whose registration request index and journal are empty, whose transition map accepts
only `HostSessionTransitionIntentV2`, and whose future application entries support a `None`
`startup_terminal_application`; it may not write either later-owner proof. Key rotation/retirement
and object reachability must preserve identical strict V1 behavior, operate against strict V2
without dropping any application-journal reference, and prove V2 initial-application reachability
and exact rejection of mixed-version state.

The exact A1.2a-WB implementation allowlist is:

- `crates/shell/src/execution/agent_runtime/host_session_authority/transition.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/transition_tests.rs`
- `crates/shell/src/execution/agent_runtime/host_session_authority/facade.rs`, limited to
  `HostSessionAuthority::resolve_current_exact`

Only the Host/world-binding Start validation and exact current-authority read conditions plus the
directly colocated matrix, zero-mutation, retry/conflict, authority-preservation,
placement-separation, corrupt/substituted-state, and unchanged-byte tests may change. All other
facade behavior remains outside scope. Any required file or symbol outside this list is
`ImpactDecisionRequired`.

The exact A1.2a-S implementation allowlist is only
`crates/shell/src/repl/async_repl.rs`, and within it only a new shared
`RuntimeAuthorityContext`, the authority field of `RuntimeOrchestrationContext`, a new
`GreenfieldHostStartProposalV1`, `DormantHostOrchestratorLaunchPlan::into_prepared` (which may be
renamed only to reflect its proposal result), `prepare_host_orchestrator_runtime_startup`,
`prepare_host_orchestrator_runtime_from_resolved`, a new
`apply_greenfield_host_start_from_authority`, `start_host_orchestrator_runtime`, the host branch of
`dispatch_targeted_follow_up_turn`, the authority-managed branch of
`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`, and directly
colocated focused tests. `prepare_hidden_owner_helper_runtime` may change only its
`RuntimeOrchestrationContext` literal to initialize `RuntimeAuthorityContext::Legacy`; no helper
behavior or plan field may change. A required file or symbol outside that list is
`CrossDocumentChangeRequired`. `PreparedAgentRuntime`,
`prepare_fork_child_runtime_startup_for_descriptor`,
`prepare_member_runtime_startup_for_descriptor`, `start_remote_member_runtime_with_prepared`, and
their production semantics are explicitly unchanged; the apply adapter constructs an ordinary
fully materialized `PreparedAgentRuntime` only after authority application. The branch may suppress
legacy writes and construct an in-memory projection only from the exact applied result; it may not
convert process, readiness, endpoint, prompt, or event observation into a HostSessionAuthority
transition. Any mechanically affected existing test in this file must obtain its host
`PreparedAgentRuntime` through the same production apply adapter; no test-only authority writer,
identity filler, or legacy-prepare shortcut is allowed.

`B1/B2.1-R0` and `B1/B2.1-0` are not A1.2 packets. R0 supplies only canonical retained-target
registration/read and deliberately has no production creation bridge.
RetainedWorkerRuntime creates the immutable
descriptor/resume-handle/retained-worker graph only after HostSessionAuthority durably reserves the
existing ingress issuer request ID, validates the participant plan supplied by its caller, and
fixes all registration/object identities,
commitments, expected authority, policy, world, and timestamp. HostSessionAuthority alone then atomically appends the
retained participant ID to lineage, adds the exact typed worker ref, advances authority revision,
and persists the non-transition registration proof consumed by `resolve_exact` while marking the
request Applied; crash retry and lost-response recovery exact-join through the request index, and
changed bytes or stale/conflicting revision fails closed. B3.2a first uses one locked durable
admission CAS to count every nonterminal slot, enforce the current cap, and reserve a fixed
participant ID across processes. A queued `SlotReserved` record with no current head is valid.
Only an exact retry of the complete canonical request for the lowest-sequence queued slot can
acquire the head; current-head R0 reconciliation and release is a separate durable transaction
that never automatically promotes another slot. Changed bytes conflict without mutation, later
slots cannot overtake, no request/prompt/payload preimage is persisted, and no caller/PID/helper/
socket/endpoint/timeout/liveness observation can steal the head. Before transport, the bounded `SpawnWorldWorker` integration
passes that slot plan to R0, exact-joins its reservation,
publishes and commits the graph/proof, and makes the remote retained-start path validate that proof
instead of invoking activated-store legacy session/participant writers. Its exact allowlist names
every file required for that request type/index, object publication/reachability, final CAS,
host-side preregistration, remote proof validation, and tests; it gains no wildcard. Active caller,
posture, workspace, world, policy, origin, spawn policy/outcome, transport events, and unrelated
refs remain unchanged. R0 adds no messaging, accepted-turn observation, active-turn,
park/cancel/stop/fork, or live-admission semantics. B3.2a connects that protocol to the real
Spawn ingress and owns the minimum durable admission/routability record; B3.2a-WA then supplies only
the exact physical same-world ownership adoption required before launch. B1/B2.1-0 consumes the
A1.2a, R0, B3.2a, and B3.2a-WA read results, authorizes at most one
trusted open-and-bind conversion when a caller-supplied bound capability is unavailable, replaces
the missing legacy session/caller inputs for B-owned actions, and splits their prepared view away
from spawn/fork `live_retained_worker_count`. It cannot modify the already-landed B3.2a spawn
creation/admission bridge, spawn/fork steering, manufacture live
retained truth, or broaden the exact two prepared-dispatch files plus the named bounded
`tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1` exception frozen below.
A1.2a, A1.2a-WB, and A1.2a-S are landed
and independently review-clean. B1/B2.1-R0 is independently review-clean through `bb3eefba`;
B3.2a plus B3.2a-WA are independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. B1/B2.1-0 remains gated by its exact allowlist and
independent clean review.

The A1.1d cross-packet integration hold keeps the preserved B0 → B1 receipt core → B2.1 branch
independent while A1.1e → A1.2a → A1.2a-WB → A1.2a-S → B1/B2.1-R0 → B3.2a → B3.2a-WA builds the
authority/retained branch. They join at B1/B2.1-0, then continue through the joint B1/B2.1
closeout → B3.1 → C1 → A1.2b despite open
integrated Linux and native-macOS closeout. The corridor moves only the
minimum facts and owners needed for the semantic cut; it does not move B2.2 foreground early
return, B3.2 retained lifecycle/messaging completion, A2 episode demotion, or A3 persistence
separation. A1.2b remains blocked at `AwaitingObligationCut` until the corridor lands, then resumes
as the consume-only authority client. A1.3 remains blocked until A1.2b exits.

#### A1.1 internal green subpacket decomposition

A1.1 is implemented through the following fixed, independently reviewable, sequential green subpackets. Moving code into the bounded module directory is not itself a completed subpacket. Each subpacket must add its own test-first or test-alongside proof, pass targeted negative cases, formatting, and diff checks, and receive fresh review before the next begins. None may claim the complete A1.1 exit gate until A1.1a–A1.1e all pass together.

For A1.1e only, the implementation allowlist additionally includes
`crates/broker/src/effective_policy.rs`, `crates/broker/src/api.rs`,
`crates/broker/src/lib.rs`, focused broker differential tests, and the directly corresponding shell
explicit-home tests already inside A1.1e scope. This permits exactly one broker-owned
explicit-global-policy-source entry point that reuses the existing parsing, precedence,
explanation, validation, and finalization pipeline. It permits no policy schema, precedence,
default, workspace-discovery, patch-merge, validation/finalization, or enforcement change; no
dispatch narrowing, worker-capability composition, E1/E2/E3 implementation, general broker
refactor, or A1.2 work. A1.1e's existing boundary and exit gate are unchanged.

| Subpacket | Reviewable outcome | Boundary and proof focus | Gates not yet claimable |
|---|---|---|---|
| **A1.1a — repository-owned canonical JSON codec and hash-vector tests** | One closed `CanonicalJsonV1` encoder/decoder implements the exact A1 field, enum, integer, string, timestamp, unknown/duplicate-field, and UTF-8 rules without delegating canonical meaning to a presentation serializer. | Bounded codec modules under `agent_runtime/host_session_authority/`; normative bytes/digest fixtures, alternate-encoding rejection, duplicate/unknown-field negatives, integer boundaries, explicit-null options, and every named A1 hash wrapper required by A1.1. | Trusted filesystem, bootstrap/root/key/object persistence, transaction preflight, CAS, exact resolution, and the complete A1.1 gate. |
| **A1.1b — private opened-directory/openat trusted-store filesystem boundary** | One private trusted-root handle binds `CanonicalDirectoryV1`; after opening it, every authority descendant operation is directory-relative and no-follow, never path-re-resolved or ambient-CWD-derived. | Bounded trusted-filesystem modules; owner/mode/ACL, physical identity, same-filesystem, symlink/reparse, component replacement, scan-to-publication replacement, atomic no-replace/root replacement, file/directory `fsync`, and unsupported-platform fail-closed proof. | Root/key/object semantic persistence, transaction preflight, CAS, exact resolution, and the complete A1.1 gate. |
| **A1.1c — root/key/object bootstrap and deterministic crash reconciliation** | Greenfield initialization, pending recovery, immutable certificate, key lifecycle, typed object publication/verification, retained orphan exact adoption, and closed temp grammar converge deterministically across every specified crash window. | Bounded store modules using only A1.1a/A1.1b primitives; exact fresh/pending/existing/legacy/corrupt classification plus marker/key/root/object/temp crash matrices. | Centralized semantic preflight, writer exclusion, authority CAS, exact resolution, and the complete A1.1 gate. |
| **A1.1d — centralized transaction preflight, cross-process CAS, and legacy/direct-writer exclusion** | Every authority read or mutation passes one semantic preflight; every applicable pre-A1 session/participant read-decide-write transaction retains one opened trusted physical root, directory-relative descendants, exact identity, activation observation, and the same root lock through final file/directory `fsync`; every legacy/direct writer fails before mutation after activation; root and authority writes use expected-revision CAS across processes. | Bounded authority-store transaction modules and bounded new modules beneath `host_session_authority/` when needed for independent reviewability; only the minimum `state_store.rs` persistence/integration choke points required to route all guarded flat/canonical snapshots, leases, removals, compatibility/read-repair persistence, and parent-session mutations through directory-relative/no-follow operations. Include post-root legacy insertion, root rename/replacement/rebind with an untouched replacement tree, unknown tree/key/object state, stale/conflicting subprocess writers, zero-mutation rejection, and contention proof. No unrelated StateStore extraction or semantic redesign is authorized. | Facade integration, complete exact-resolution proof, and the complete A1.1 gate. |
| **A1.1e — HostSessionAuthority facade integration and exact-resolution proof** | Establish the exact facade as the sole A1 authority-store API, consume the same opened bootstrap-home binding as bounded config/policy/inventory entry points, resolve exact durable identity and current authority revision/hash without PID/socket/helper/prompt truth, and expose only the A1.1 primitives required by A1.2. Under the exact cross-packet integration hold above, this work may proceed while A1.1d integrated closeout remains open; it does not repair the real successor path or claim `RG-BASE-01`. | Bounded facade/integration modules; explicit-home shell entry points; the bounded broker entry point and focused tests authorized immediately above; exact session/store/workspace/world/lineage/ref resolution; stale observation rejection; cross-home/CWD negatives; and the full available A1.1 regression wall with the deferred public lifecycle failure kept explicit. The shell derives the explicit source from the accepted home and consumes the canonical broker result; it does not build or finalize effective policy. | Production `ExpectedAbsent`, Start reservation/intent/application, real caller adoption, parked-successor repair, `RG-AUTH-03`, `RG-BASE-01`, A1.1d integrated/cross-platform closeout, policy schema/precedence/default/workspace-discovery/patch-merge/finalization/enforcement changes, dispatch narrowing, worker-capability composition, E1/E2/E3 implementation, general broker refactoring, A1.2, and A1 completion. |

### A1.1d internal review checkpoints

A1.1d remains the canonical packet. The five labels below are sequential internal review checkpoints
under A1.1d, not independent slices, not replacements for A1.1d, and not separate closeout gates.
The A1.1d packet closes only after all five checkpoints pass together. The exact cross-packet
integration hold above historically permitted A1.1e facade/resolution and the now-preserved A1.2
checkpoint while integrated and platform closeout stayed open; it did not close A1.1d, create
A1.1d-6, permit aggregate A1.2 exit before its C1-owned semantic cut, or permit A1.3 work early.
The normative current rule is: never restore or modify the broad A1.2 checkpoint; implement the
new A1.2a prerequisite only from its bounded reviewed contract, and keep A1.2b blocked until the
B1/B2.1 joint production closeout, B3.1, and C1 have landed.

#### A1.1d-1 — Retained opened-root transaction capability

Outcome:

- introduce the bounded `LegacyStateStoreTransactionV1`-equivalent capability;
- retain the trusted root, opened descendants, exact physical identity, activation observation,
  and root lock for the complete transaction lifetime;
- provide only directory-relative/no-follow read, write, temp-publication, rename, removal, and
  file/directory `fsync` operations;
- prove there is no environment/CWD/path re-resolution or lock-lifetime gap and that identity
  uncertainty fails closed;
- do not adopt all production writers yet.

#### A1.1d-2 — Complete guarded legacy-writer adoption

Outcome:

- route every in-repository StateStore transaction that reads or mutates the two pre-A1
  session/participant authority collections through the retained transaction;
- include every read participating in a read-decide-write operation, flat and canonical snapshots,
  participant leases, removals, compatibility/read-repair persistence, and parent-session
  persistence triggered by another operation when those paths mutate a guarded collection;
- keep the read, decision, writes, renames, removals, and final `fsync`s inside one retained
  transaction while preserving existing business semantics;
- keep production A1 activation unavailable until adoption is complete.

#### A1.1d-3 — Exact-retry and publication-candidate closure

Outcome:

- reservation, tombstone, transition-intent, issuer-index, application-journal, and object-index
  state cannot be treated as authority-free;
- exact retry joins only fully matching committed semantic state;
- rotation and retirement fully revalidate the reconciled publication candidate immediately
  before root publication while retaining the same trusted root and lock;
- stale, conflicting, missing, substituted, malformed, or post-reconciliation-invalid candidates
  fail with zero semantic mutation.

#### A1.1d-4 — Cross-process replacement, crash, and regression closure

Outcome:

- prove a physical-root rename, replacement, or rebind cannot redirect a guarded transaction and
  the replacement tree receives no writes, temps, removals, or publication;
- prove crash release, first-creation contention, stale writers, post-activation rejection,
  preactivation serialization, temp reconciliation, publication durability, and exact retry
  converge;
- run the combined Linux proof wall and required product smoke, preserving separate positive and
  unsafe-ACL negative evidence;
- keep A1.1d integrated and platform closeout open until all required proof is complete; the exact
  cross-packet hold alone permits the later-owner facade/resolution work in A1.1e.

#### A1.1d-5 — Private `SUBSTRATE_HOME` creation and capability parity

Outcome:

- make every supported production creation path create `SUBSTRATE_HOME` as private per-user state
  owned by the intended per-user owner with exact mode `0700`, independent of ambient umask;
- retain an already-validated trusted-parent descriptor, create only a candidate child name or join
  `AlreadyExists`, then define the accepted physical root identity at the first successful no-follow
  child open and descriptor validation of owner, exact mode, ACL, type, filesystem, and identity;
- support legitimate concurrent same-UID Substrate creators through create-versus-`AlreadyExists`
  convergence, while keeping every post-acceptance operation descriptor-relative and revalidating
  path-to-descriptor identity at required publication boundaries so later replacement or drift
  fails closed;
- accept an existing or custom root only when it already satisfies the identical contract, with no
  migration, chmod, chown, ACL removal, compatibility adoption, shared-home fallback, or repair;
- limit implementation to live-inventory-derived bounded Rust home bootstrap, production/dev
  installer home creation, Linux provisioning, macOS/Lima provisioning, focused tests/smoke
  fixtures, and installation/configuration/world documentation;
- preserve policy schema/defaults/effective calculation, world requests and enforcement plans,
  filesystem read/discovery/write and containment behavior, host visibility and isolation/sync
  strategy, network and DNS behavior, PTY/non-PTY execution, dependency inventory/synchronization,
  config precedence, gateway/credential handoff, current runtime availability, shims, replay,
  traces/diagnostics, and start/reattach/stop/retained-worker/world-binding behavior byte-for-byte
  or semantically equivalent as applicable;
- treat shared multi-principal homes, install-root/state-root redesign, policy or world filesystem
  changes, network changes, credential-path changes, a privileged creation broker, defense against
  malicious root or malicious same-UID substitution before first child-descriptor acquisition, and
  A1.1e as explicit non-goals; do not claim portable atomic create-and-bind behavior; and
- exit `RG-HOME-01` only after complete Linux creation and world-capability parity proof plus native
  macOS proof on the exact final runtime commit. Linux-only success leaves A1.1d incomplete; it
  cannot close A1.1d or A1, even though the exact integration hold permits A1.1e to proceed.

##### A1.1d-5I — Installer and bootstrap compatibility audit

This is a completed, docs/evidence-only internal investigation packet beneath A1.1d-5. The suffix
is capital `I`; it is not A1.1d-6, creates no sixth checkpoint, promotes no seam, and changes none
of the A1.1e/B1/B2.1/B3.1 corridor or B2.2/A1.2/C1 ownership.

At baseline `b29897e0`, the audit proved four remediation groups: ancestor access/default ACL
semantics and diagnostics; selected prefix/home/root propagation across install and uninstall;
partial-created-home and managed-artifact cleanup; and missing lifecycle regressions. The bounded
implementation order is:

1. **A1.1d-5R1 — Ancestor ACL semantics and diagnostics.** Preserve exact-owner, exact `0700`,
   ACL-free final-root acceptance; limit the unresolved support question to masked non-writing
   ancestor access ACLs and require R1 security review before any support decision; reject any
   effective write bit, every ancestor default ACL before creation, and every malformed, unreadable,
   or ambiguous ACL state; identify the exact offending path/object role/ACL kind/reason without
   repair or secret disclosure. Supporting a default ACL requires a later separately approved
   contract.
2. **A1.1d-5R2 — Prefix propagation across install/uninstall.** Carry the selected
   host prefix/`SUBSTRATE_HOME`/`SUBSTRATE_ROOT`/intended-principal context across every child and
   sudo boundary, including shim, world, service, runtime, generated-file, release, dev, and
   uninstall paths. A platform backend must preserve that host-context commitment through an
   explicit mapping to its platform-native path/principal; it cannot equate a host path with a
   Lima/WSL guest path or invent home authority. An outer `SUBSTRATE_HOME` override is diagnostic
   only and cannot pass the product gate.
3. **A1.1d-5R3 — Partial-install cleanup and idempotency.** This packet is required by the audit:
   a default ACL can leave a newly created invalid candidate, and dev uninstall leaves managed
   gateway symlinks. Candidate cleanup is descriptor-bound and may remove only an empty directory
   created by the current attempt after a no-follow parent/name lookup still joins its exact opened
   identity; it is never recursive and replacement, nonempty state, ambiguity, or `AlreadyExists`
   provenance fails closed. Recorded prefix-local and system-level managed artifacts, including
   helpers, gateways, units/drop-ins, sockets, and runtime directories, remain separately removable
   only by exact manifest identity. Broad name-prefix/wildcard deletion is forbidden. R3 must
   preserve unrelated/pre-existing state and prove convergence after an accepted valid home or a
   synchronous current-attempt rejection whose exact safe rollback completes, plus uninstall →
   reinstall. An abrupt interruption that leaves an unaccepted invalid candidate stays fail-closed
   on rerun and is never automatically repaired or removed; arbitrary crash-window convergence
   requires a separately approved provenance/publication contract if R3 cannot prove that state is
   unreachable after R1.

R1, R2, and R3 are not implemented or review-clean. `RG-HOME-01` and `RG-INSTALL-01` remain open.
Their Linux regression wall and normal product lifecycle smoke are prerequisites for A1.1d Linux
closeout and for the Linux product-smoke portion of the B1/B2.1 joint closeout. They do not reopen
the review-clean B1/B2.1-0 prerequisite or make native macOS A1.1d proof a B1/B2.1 dependency.

The new A1.1 module boundary is organizational only and does not change any A1 contract, semantic gate, or later-slice owner. It enforces these additional reviewability constraints: no path-based authority operation after the trusted root opens; no symlink traversal or ambient-CWD authority; exactly one repository-owned canonical codec; exactly one centralized semantic transaction preflight; no legacy/direct writer may bypass `HostSessionAuthority`; key/object/root crash reconciliation is deterministic; and no new dependency is permitted without explicit dependency review. The bounded modules must not perform A2 episode demotion, helper removal, endpoint redesign, receipt work, unrelated StateStore cleanup, or general persistence extraction.

The `RG-AUTH-01` and `RG-AUTH-02` references in A1 are scoped gates, not whole-ledger closure claims. A1 proves only the `Start`/`Attach`/`ResumeOneTurn`, bounded helper-plan, real-REPL, and bounded auto-attach producer clauses named by A1.1–A1.4. General episode loss/demotion, Stop/Fork adoption, unrelated PID/socket consumers, retained workers, and compatibility/persistence separation remain unresolved for A2/A3 or their named later owners.

### Current A1 sequencing status

- A1.1d-1 through A1.1d-4 implementation remains preserved.
- A1.1d-5 focused private-home implementation is review-clean on Linux; A1.1d integrated Linux
  closeout is open and cross-platform closeout is pending. A1.1d-5I has completed the bounded
  compatibility audit; R1, R2, and R3 remain unimplemented, and no A1.1d-6 exists or is implied.
- The public lifecycle failure is owned by A1.2/A1.3, not an A1.1d heartbeat or successor-work
  subpacket. No A1.1d-6 exists or is implied.
- A1.1e is focused-proof and review clean through `cd676614`; this does not close A1.1d or any
  integrated product gate.
- Broad A1.2 transition-protocol work exposed both the missing producer-to-ledger truth path and
  the missing production current-authority input. Its checkpoint remains preserved and must not be
  restored. A1.2a is landed as a bounded prerequisite packet, not permission to recover the broad WIP;
  A1.2b's resumable post-turn remains `AwaitingObligationCut` until C1.
- The B0 → B1-3a/B1-3b receipt core → B2.1-1/2/3 branch remained independent while
  A1.1e → A1.2a → A1.2a-WB → A1.2a-S → B1/B2.1-R0 → B3.2a → B3.2a-WA builds the authority/retained branch. They join
  only at B1/B2.1-0, then proceed through the joint B1/B2.1
  production closeout, B3.1, C1, and A1.2b. B0, the recovered B1/B2.1 cores, A1.2a, A1.2a-WB,
  A1.2a-S, B1/B2.1-R0, B3.2a, B3.2a-WA, and B1/B2.1-0 are independently review-clean. The joint
  production closeout has not begun. B3.1 still requires that joint
  closeout; no A2/A3 prerequisite moves into the corridor.
- B2.1 may transfer durable observation ownership while the current foreground call remains a
  blocking compatibility waiter. B2.2 owns early return later. A1.3 remains blocked and still owns
  real CLI/helper/REPL adoption plus `RG-BASE-01` closure.
- A1.4 is not ready. A1 remains incomplete and non-landable until the deferred lifecycle gate, the
  complete Linux wall, required native macOS proof, A1.4, and all final A1 gates pass.

## Track B — World-dispatch receipts, supervision, and cancel

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
**B1/B2.1 joint production closeout** proves both accepted families,
legacy-writer exclusion after acceptance, caller-drop/restart survival, exact terminal behavior,
blocking compatibility, the differential baseline, and supported doctor/smoke. B3.1 remains
blocked until this joint closeout passes.

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
| **B1/B2.1 — Joint production integration closeout** | Prove the real ephemeral and retained accepted paths proceed from immutable B1 acceptance into B2.1 supervision without any legacy-writer attempt or observation gap. Use B1/B2.1-0 only for RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait. | HostSessionAuthority/A1.2a own exact current authority, A1.2a-WB owns only its Host/world-binding write/read correction, and A1.2a-S owns only bounded Start adoption; RetainedWorkerRuntime owns R0 object construction/read plus B3.2a admission/routability while HostSessionAuthority owns only R0's atomic lineage/ref mutation and proof; the runtime-family/Linux backend owns only B3.2a-WA physical realization and ownership metadata; ReceiptRegistry and Supervisor retain distinct accepted-work and observation ownership; WorldDispatchControl only validates and joins them. | All B1/B2.1 sections in `01`–`05`; production Spawn setup through both adapters, world-service proof validation, B3.2a-WA exact same-world ownership adoption, live acceptance, full-dispatcher named-path inspect/wait/cancel, drop, restart, and reconciliation. Retained control cases remain full-dispatcher differential inputs but not failure-to-pass closeout proof. | RuntimeEventTransport; A1.2a-S bound authority; B1/B2.1-R0 exact retained-target read; B3.2a creation/admission truth; B3.2a-WA physical ownership truth; compatibility callers. | After prerequisite review, new joint-closeout implementation edits are limited to `crates/shell/src/execution/orchestrator_world_dispatch.rs` and `crates/shell/src/execution/agent_runtime/state_store.rs`, including colocated focused tests; no wildcard. The already-reviewed B1/B2.1-0 tool adapter remains within its prerequisite allowlist and receives no additional joint-closeout authorization. Already-reviewed B1/B2.1, A1.2a-WB, A1.2a-S, R0, B3.2a, and B3.2a-WA code remains within its prior packet allowlists. Closeout rows in this six-file pack only after proof. | No authority creation in closeout code; no transition issuance/application/correlation; no live-retained truth fabrication; no `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, additional spawn/fork, or retained-lifecycle change; no B3.1, C1, A1.2b, B2.2, B4 final outcomes, policy, UAA, retained semantic, obligation, or early-return work. | A1.2a creates exact greenfield authority, A1.2a-WB freezes the exact placement/binding matrix without changing placement, and A1.2a-S adopts it on the internal host/toolbox path with zero activated legacy writes. R0 creates/proves/exact-resolves the retained target, B3.2a creates it through both real Spawn adapters with typed world-service proof validation and exact routability/live admission, and B3.2a-WA durably adopts the exact HSA-bound world before member creation without changing its ID/generation or creating an alternate world. The named action-scoped view excludes `live_retained_worker_count`; both accepted families survive caller drop/restart; no accepted work is followed by legacy active-task registration or activated-store rejection; ephemeral accepted-task Inspect/Wait/Cancel consumes exact receipt/supervisor truth; terminality is exact. Retained Inspect/Cancel/Stop stays on the unchanged full dispatcher and may only remain `FailToSameFailure` until remaining B3.2/B4; a direct resolver substitution is still `RegressionMasked`. Complete differential, doctor/smoke, and fresh review gates pass. | Applicable B1 and B2.1 gates, production legacy-writer exclusion, blocking compatibility, caller-drop/restart, `RG-OBS-01` acceptance/journal joins, `RG-WORLD-ADOPT-01`, and `RG-DIFF-01`. | Review-clean B1/B2.1 cores, A1.2a, A1.2a-WB, A1.2a-S, R0, B3.2a, B3.2a-WA, and B1/B2.1-0. | B3.1 and every later packet. |
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
| **B3.1 — Bounded retained-turn event/causation prerequisite** | Provide only the typed worker-to-host retained event envelope C1 requires, joined to B0 identity and B1 active-run truth, and normalize provider semantics at the producer before `AgentEvent` construction. | WorldWorkerMessagingProtocol. | `01` authority map/invariants 3–6; `02` MessagingProtocol row; `04` acceptance context + producer semantic normalization + retained event envelope; `05` `RG-MSG-01`/`RG-OBL-01`; messaging design envelope/thread/ordering/attention sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; RetainedWorkerRuntime; ObligationLedger. | Add `NormalizedWorldWorkerEventFacetV1`, the optional top-level `AgentEvent.worker_event: Option<WorldWorkerEventV1>`, and their typed fields in `crates/common/src/agent_events.rs`; consume B1's shared types from `crates/common/src/{authority_commitment.rs,lib.rs}`; consume/validate the existing B1 `WorldWorkAcceptanceContextV1` request extension in `crates/transport-api-types/src/lib.rs` without changing `ExecuteStreamFrame`; move the current provider-payload thread/class/attention/payload interpretation into one explicit fail-closed producer normalizer and shape the final member from that facet plus retained request/runtime context in exactly `crates/world-service/src/member_runtime.rs`; bounded host validation, deletion of C1-path JSON-pointer classification, and generic-journal handoff in `crates/shell/src/execution/orchestrator_world_dispatch.rs`; bounded host-facing semantic adapters in `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`; focused tests colocated in those exact files. | No `ExecuteStreamFrame` variant/field change, no upstream provider-wrapper contract change, no new request-envelope semantics beyond consuming B1's context, no host or ledger identity/semantic inference from emitted `AgentEvent.data`, no producer-chosen eligible subset, no omission/untyped event/permissive downgrade for an unknown or malformed provider shape, no separate transport protocol, no host-to-worker message completion, no foreground early return, no worker park/cancel/stop/fork rewrite, no host-transition issuance/authentication/interpretation, no obligation classification/materialization, and no policy broadening. | After exact B1 acknowledgement, every retained `ExecuteStreamFrame::Event` through the terminal cut is in the closed B3.1 domain. Before `AgentEvent` construction, the producer normalizer maps every existing provider `AgentWrapperEvent` kind/payload into an exact typed facet for thread/class/attention/payload; parsing provider payload is permitted only at that named adapter boundary. Explicit supported non-attention shapes receive a typed non-attention class; missing, ambiguous, deferred, unknown, or malformed shapes fail the stream and C1 cut closed rather than being dropped, left untyped, or becoming progress/no-attention. The final typed `AgentEvent.worker_event` joins that facet with exact sources: acceptance ID, request/message causation and target backend come from the retained B1 context; session, source/target participant, source backend, active run, and world come from typed request/runtime context; B0 supplies event/frame identity. B3.1 validates that B2.1's generic event commitment equals the full canonical typed member and that all B0/B1/context fields match before ledger delivery; any opaque correlation is copied unchanged and compared only for equality. Host-side JSON-pointer classification is not a C1 input. Missing or mismatched identity fails closed; request ID or active-run ID never substitutes for transition intent/run. | Bounded producer/consumer clauses of `RG-MSG-01`; prerequisite clauses of `RG-OBL-01`; B3.1 event clause of `RG-OBS-01`. | B1/B2.1 joint production closeout and landed A1.1e. | Remaining `RG-MSG-01`, upstream provider-wrapper taxonomy cleanup, A1.2b production correlation supply, foreground retained receipt return, host-to-worker messaging, retained lifecycle/park/nonzero-exit behavior (`B3.2`), C1 materialization, and seam promotion. |
| **B3.2 — Remaining retained receipt, messaging, and lifecycle completion** | Complete model-facing retained receipts, host-to-worker envelopes, delivery semantics, accepted-turn lifecycle/park behavior, durable resolution of abandoned `SlotReserved` and `AuthorityRegistrationHead` admissions, reconciliation of post-R0/pre-transport and ambiguous transport-claim states, and full creation-observer restart reconciliation without changing B3.1 event truth or B3.2a's already-landed registration/admission identities. | WorldWorkerMessagingProtocol owns message semantics; RetainedWorkerRuntime owns worker lifecycle and every durable admission-state resolution. | `02` MessagingProtocol + RetainedWorkerRuntime rows; `04` retained receipt/manifest, B3.2a admission state, and deferred admission-recovery contract; full messaging and lifecycle designs; `05` `RG-ADMISSION-01`. | Supervisor; SteeringPolicyEngine; ObligationLedger; B4 cancel. | `orchestrator_world_dispatch.rs`; `agent_runtime/{dispatch_contract,session,state_store}.rs`; `agent_runtime/retained_worker_runtime.rs` only for deferred durable admission resolution and reconciliation; `world-service/member_runtime.rs`; focused retained lifecycle tests. | No auto-attach, no rewrite of B3.2a registration identity or exact terminal truth, no obligation ownership transfer, no liveness-inferred admission resolution, and no dispatch narrowing beyond carrying E2 snapshot/cap refs. | Continue returns the B1 accepted active-run receipt completed by E2 through B2.2; host-to-worker messages preserve exact identity/causation; clean turn parks without losing worker continuity. Exact retry rejoins the same admission. Authorized durable resolution exact-joins issuer request, admission record, session, authority, policy, participant, and state; reconciles partial R0 registration or transport ambiguity instead of deleting or blindly rolling back; terminalizes idempotently; and releases the live cap only after exact terminal durable proof, permitting the next eligible queued request. Crashes before/after resolution converge across restart without PID, timeout, caller, helper, socket, endpoint, observer, or process-liveness inference. Creation-observer interruption/restart also reconciles without liveness inference; failure is durable and exact. | Remaining `RG-RECEIPT-02`, `RG-MSG-01`, `RG-BASE-04`, `RG-WORKER-EXIT-01`, and the durable-state clauses of `RG-ADMISSION-01`. | B3.2a, A1, A2/A3, E2, B2.2, B3.1, and C1. | B4 user/tool-facing admission inspect/cancel and worker cancel/inspect/stop outcomes, C2/C3, and auto-attach. |
| **B4 — Receipt-targeted cancel/inspect/stop** | Resolve control against active receipts and worker manifests with distinct durable outcomes, and expose exact inspection and cancellation of pending Spawn admission through WorldDispatchControl without taking ownership of RetainedWorkerRuntime state transitions. | WorldDispatchControl owns the user/tool-facing verb and resolves exact targets; ReceiptRegistry owns immutable accepted identity; Supervisor owns active observation and terminal truth; RetainedWorkerRuntime owns worker stop lifecycle and durable admission-state resolution. | `04` cancel categories, deferred admission-recovery contract, and supervisor cancellation rules; `02` WorldDispatchControl/ReceiptRegistry/RetainedRuntime rows; `05` cancel/closeout rows and `RG-ADMISSION-01`. | HostSessionAuthority; Supervisor; private transport episode status. | `orchestrator_world_dispatch.rs`; `agent_runtime/{state_store,dispatch_contract,control}.rs`; world-service cancel seam; control tests. | No weakening exact identity, no treating no active run as stale linkage, no liveness-based admission resolution, and no stop/cancel/admission-abandonment conflation. | Same-turn continue→cancel targets active run. Pending Spawn inspection/cancellation exact-joins the admission identity and returns distinct outcomes for cancelled-before-registration, registered-before-transport, transport-ambiguous/cancel-pending, already routable/terminal, invalid target, ambiguity, and policy denial. Repeated terminal resolution is idempotent; no-active, terminal, unreachable, invalid, mismatch, ambiguity, and policy denial for active work remain distinct. | `RG-CANCEL-01`, `RG-CANCEL-02`, `RG-CLOSE-01`, `RG-BASE-04`, and the user/tool-facing clauses of `RG-ADMISSION-01`; B4 cancel clause of `RG-OBS-01`. | B2.2 and B3.2. | C2/C3 and unrelated lifecycle/policy work. |

### B1/B2.1-0 recorded prerequisite result

The recovered receipt, supervisor, replay/startup, and authority-store correction commits are
`6436289f`, `c519024b`, `de727091`, and `717579b0`; the action-scoped dispatch prerequisite is
`83101dcb`. Its exact fourteen historical production-path tests retain their names and routes, the
shell differential is `981 passed / 160 failed / 0 ignored` to
`1054 passed / 149 failed / 0 ignored`, and every one of the eleven `FailToPass` transitions is
causally attributed to the real dispatcher or tool-to-dispatch correction. All forbidden
transition classes are zero and all 149 retained normalized failure signatures are unchanged.
This result closes only B1/B2.1-0. The joint production integration closeout has not begun, B3.1
is not dependency-ready, and no seam is promoted.

## Track C — Obligations, inbox, auto-attach, and router attach

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C1 — Event-to-obligation materializer and semantic cut** | Consume durable exact events, idempotently create canonical obligations while the supervisor observes active work, and own the monotonic ledger revision/materialized-event cut plus closed snapshot query consumed by A1.2b. | ObligationLedger. | `01` invariants 4–6; `02` RuntimeEventTransport/Supervisor/Messaging/Obligation rows; `04` runtime carrier, supervisor rules, and `ObligationLedgerSnapshotReadV1`; `05` `RG-OBL-01`/`RG-OBL-02`; obligation-ledger producer/dedupe sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; MessagingProtocol; HostSessionAuthority consume-only client. | `agent_runtime/obligation_ledger.rs`; bounded consumer integration from the receipt-scoped supervisor journal; bounded StateStore persistence only; focused materialization/cut/snapshot tests. | No runtime identity generation, receipt acceptance, observation ownership, retained-envelope or host-transition semantics, producer event eligibility choice, inbox rendering, router launch, direct prompt injection, or transfer of obligation semantics to HostSessionAuthority/StateStore. Stream exhaustion, EOF, timeout, PID/helper/socket state, inbox rows, pending counts, worker flags, and compatibility projections are forbidden completeness inputs. | Each B3.1 attention event creates exactly one canonical obligation before the exact B0 terminal event when applicable; duplicate journal replay creates none. Before classification and snapshot capture, C1 verifies that every post-acknowledgement retained B2.1 `Event` journal ref through the cut commits one full canonical B3.1 target/source/thread/class/attention/request/message/transition/payload envelope. The ledger advances one monotonic session revision and materialized-through event watermark, returns Pending before coverage, then returns a Complete snapshot binding exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, and the complete ordered exhaustive join of all retained `Event` refs through the cut even for `NoUnresolvedAttention`, plus unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run, authority revision, disposition, and sorted canonical-record commitments. Any untyped/omitted/substituted event or stale/mismatched scope, correlation, revision, cut, disposition, or record commitment keeps the cut non-Complete. | `RG-OBL-01`, C1 clauses of `RG-OBL-02`, obligation clauses of `RG-SUP-01`, bounded consumer clauses of `RG-MSG-01`, and C1 clauses of `RG-OBS-01`. | A1.1e, B0, B1, B2.1, and B3.1. | A1.2b production correlation supply and consumption/adoption, C2 projections, C3 router behavior, B2.2 foreground early return, and every seam promotion. |
| **C2 — Inbox and auto-attach projections** | Make inbox and attach eligibility pure projections over obligation state and effective policy. | InboxProjection and AutoAttachProjection. | `02` projection rows; notification inbox design; auto-attach trigger design. | ObligationLedger; CompatibilityReadModel; SteeringPolicyEngine. | `agent_runtime/{host_inbox,obligation_ledger,auto_attach,state_store}.rs`; `host_inbox_materialization.rs`; projection tests. | No host process launch, no worker action, and no removal of compatibility ingress without migration proof. | Deleting/rebuilding projections does not lose obligation truth; session-coalesced claim is deterministic; wrong-host and policy denial fail closed. | Projection clauses of `RG-OBL-02`, `RG-ATTACH-01`, `RG-BASE-03`. | A1, A2/A3, C1, and B4; B4 transitively supplies B2.2/B3.2/E2. | C3 router launch and unrelated worker controls. |
| **C3 — Router ownership restoration** | Consume attach-eligible claims, restore one sanctioned host episode, settle the claim, and stop. | RouterAttachTrigger for triggering; HostSessionAuthority for the attach transition. | `02` RouterAttachTrigger row; router responsibilities/non-responsibilities; `04` HostExecutionEpisodeV1. | HostSessionAuthority; SurfaceAdapter; AutoAttachProjection. | `agent_runtime/auto_attach.rs`; bounded router entrypoint; helper launch adapter; router tests. | No prompt replay, approval/answer/fork/continue, or always-running backend assumption. | Router produces one attach outcome per session claim and cannot invoke worker-control verbs; manual reattach coexists without duplicate ownership. | `RG-ATTACH-01`, `RG-ATTACH-02`, `RG-AUTH-01`. | C2 and A1/A2. | Worker-control, receipt, and unrelated policy work. |

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
