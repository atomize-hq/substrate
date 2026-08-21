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
| **A1.1 — authority-store core** | Establish only the authority-store substrate: persisted normalized bootstrap-home binding, exact fresh/pending/existing/unsupported/corrupt classification, operation-bound temp reconciliation, exact object/key persistence, immutable greenfield namespace certification, fail-closed pre-A1 authority-state detection, exact durable resolution, and generic root/authority revision-CAS. Keep `StateStore` as atomic persistence. | Bounded modules under `agent_runtime/host_session_authority/`; bounded facade types/re-exports only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only for the minimum persistence/integration choke points that exclude legacy/direct writers; explicit-home API parameterization only in `crates/common/src/paths.rs` and `execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; focused codec, trusted-filesystem, bootstrap/reconciliation, writer-exclusion, resolution, persistence-conflict, and cross-process tests. | `StateRootV1`; `AuthorityStoreInitializationV1`; `GreenfieldNamespaceCertificateV1`; `DurableSessionAuthorityV1`; `CanonicalDirectoryV1`; immutable typed object schemas/refs; key registry/files; operation-bound temp grammar; expected `root_revision`/`authority_revision` for CAS. | No migration, quarantine, evidence, dual-write, or compatibility adoption; no production `ExpectedAbsent` acceptance; no Start reservation, transition-intent issuance/application, participant allocation, or Start-origin authority birth; no helper-plan or consumer adoption; no episode demotion; no resolver precedence, policy/config/inventory interpretation; no unrelated StateStore cleanup or general persistence extraction. | Write failing tests for genuinely absent versus malformed/partial/unreadable/unsafe/unsupported stores, interrupted initialization and copied-home mismatch, exact bootstrap-home reuse, crashes before/after temp fsync and rename, pending-key rename/directory-fsync recovery, key create/rotate/retire crash windows, first-use kind/version directory create/fsync crashes, object rename before root commit with retained-orphan restart/exact retry, safe empty versus present/unreadable/symlinked pre-A1 collections, ancestor symlink/unsafe ACL/cross-device and scan-to-publication replacement, pre-A1 writers serialized before activation and rejected after marker/root publication, cross-kind object substitution, exact resolution, stale revision, and cross-process CAS. Exit when these primitives are deterministic and green; this packet alone cannot create or claim a production Start. | Production Start/`ExpectedAbsent`; all transition intents and `RG-AUTH-03`; real CLI/REPL adoption; helper-loss reconciliation; auto-attach adoption; complete `RG-AUTH-01`/`RG-AUTH-02`; A1 completion. |
| **A1.2 — intent issuance/claim/application** | Add the A1.2-owned durable protocol under `HostSessionAuthority` while preserving landed V1 intent/state bytes as read-only: A1.2a supplies an A1.1e-dependent but B1-independent strict V2 Start intent with no B1-owned type, and A1.2b later supplies a separately reviewed strict V3 root/intent/state extension for Attach/Resume and post-turn fields after B1/B3.1/C1 types are available. Together they cover greenfield-certified Start reservation/issuance/application, exact retry, startup ownership, successor issuance/application, post-turn reconciliation, ledger-owned obligation-cut consumption, and retained transport reprojection; keep the plan a transport projection. The aggregate packet includes parked-successor `Attach`/`ResumeOneTurn`, but the normative split below assigns only greenfield Start establishment/read to A1.2a before the joint closeout; A1.2b retains successor/post-turn mechanics and begins only after the joint closeout, B3.1, and C1 establish its complete semantic-cut prerequisite. | Authority-store core files above; the bounded retained-admission compatibility read corridor in `agent_runtime/retained_worker_runtime.rs`; only `HiddenOwnerHelperLaunchPlan` plus its write/load/remove/launch integration in `agent_runtime/control.rs`; focused V1/V2 Start and later V3 successor discrimination, Start reservation/birth, intent lifecycle, serialization, persistence, helper-loss, parked-successor, input-handoff, startup-ownership, post-turn-event/input terminalization, pending obligation-cut consumption, payload-retention/reprojection, retained-admission compatibility, and plan-commitment tests. Runtime transport identity, receipt acceptance, supervisor journaling, retained-event semantics, and the canonical snapshot producer remain in B0/B1/B2.1/B3.1/C1 and are not A1.2 implementation allowances. | Exact greenfield certificate and empty pre-A1 collections; unique `intent_id`; intent/state revision; `Start`/`Attach`/`ResumeOneTurn`; exact enumerated postures; `ExpectedAbsent` or exact authority revision/hash; reservation ownership; identity/home/workspace/world/descriptor/object commitments; expiry; canonical payload and transport refs/hashes; `Issued -> Claimed -> Applied` plus terminal `Rejected`/`Expired`; immutable application result; input handoff; actor-bound startup evidence/result; actor-bound post-turn protocol event/completion/result; `AwaitingObligationCut`; ledger-owned Complete snapshot; transport `ReleaseEligible`/`Released` state. | No V1 shape mutation/defaulting; no legacy-state conversion or compatibility-derived absence; no CLI/REPL consumer switch yet; no runtime frame/event identity generation; no receipt acceptance or stream observation; no retained-message semantics; no obligation enumeration/classification/creation/resolution/reinterpretation/overwrite and no claim that event materialization is complete; no C1 implementation; no destructive-read consumption; no endpoint/path redesign; no helper removal; no general transport cleanup; no PID/helper/handle/prompt-derived transition eligibility or transport/process inference as terminal protocol evidence. | Prove strict V1 bytes remain identical/read-only, production Start uses only V2 without B1 imports, and Attach/Resume/post-turn uses only the later strict V3 extension. Prove `Attach` and `ResumeOneTurn` start from exact current parked or other enumerated authority, preserve session lifecycle identity and exact world binding, revision-authorize one successor lineage, and never derive eligibility from PID, helper, handle, readiness, or prompt state. Prove Start/Attach transport cannot advance beyond `ReleaseEligible` before exact actor-bound startup evidence resolves ownership, every Resume terminal outcome joins an exact actor-bound post-turn event/completion/result and closed input state, exact retries survive a later exact `Released` cleanup, terminal Resume can close without a snapshot, and resumable completion persists `AwaitingObligationCut` while retaining transport. A Complete ledger cut must be scoped to exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, authenticated intent/revision/payload commitment and distinct transition run, and authority/event watermark, then select the ledger-owned disposition without A1.2 scanning, inference, or reclassification. Exact stale successor retries fail or join exactly; failure after application never restores a stale snapshot. Because C1 is now a prerequisite, the full packet exits only when that unchanged Complete result drives the one allowed revisioned post-turn application and all A1.2 proofs pass. | Real Start/Attach/Resume consumer adoption; bounded auto-attach adoption; full real-path `RG-AUTH-03`; `RG-BASE-01`; A1 completion. |
| **A1.3-P0 — held preparatory split** | Preserved scope-expansion evidence only. This row correctly records the legacy launcher corridor and the frozen auto-attach caller but is not an executable prerequisite. The real public path must land transport adoption, exact actor-event resolution, and the mechanical B1 acceptance-context projection atomically. | Documentation/control only; no implementation authority. | Scope-expansion evidence and exclusions only. | No implementation dispatch, no silent widening, no reinterpretation as active successor. | Never dispatch this row as implementation authority; use A1.3-P1 instead. | Nothing claimable. |
| **A1.3 — held historical public/successor fence** | Preserved older public-adoption fence only. Its omitted `agent_runtime/control.rs` public transport work and omitted acceptance-context projection seam make it non-executable; correcting it here would silently widen the fence. | Documentation/control only; no implementation authority. | Historical public/successor boundary only. | No silent widening; no use as active successor. | Never dispatch this row as implementation authority; use A1.3-P1 instead. | Nothing claimable. |
| **A1.3-P1 — atomic public adoption and startup/post-turn completion** | In one bounded packet, land the new authority-managed public transport path for public `Start`, turn, and explicit reattach; consume exact A1.2 authority on the real public CLI/helper/REPL path; resolve exact startup and post-turn actor events; and mechanically project already HSA-authorized `host_transition_correlation` into the existing B1 acceptance-context seam. | Only the new public transport path in `agent_runtime/control.rs`; only the named public `run_start`/`run_turn`/`run_reattach`/`run_owner_helper` regions in `execution/agents_cmd.rs`; only owner-helper validation/application, exact startup/post-turn event submission and reconciliation, retained-turn acceptance-context projection, remaining public/real REPL adoption, and explicit bootstrap-home threading in `repl/async_repl.rs`; only the A1.1 explicit-home resolver entry points needed by those named CLI/REPL paths; focused CLI, helper, and REPL integration tests. | Mode-specific preconditions; exact caller/source/target lineage; bootstrap-home/workspace/world binding; typed descriptor/attach/resume/policy/input refs; claim ownership; authority/intent revisions; applied result; exact actor-bound ownership-acknowledgement/pre-ownership rejection/failure event identity and result; exact actor-bound post-turn event ID/sequence/outcome/reason and one-turn disposition; exact mechanical B1 acceptance-context projection of already HSA-authorized correlation. | No B1 semantic ownership; no C1/B3.1 semantic change; no StateStore change; no legacy helper behavior change; no auto-attach change; no macOS, Windows, or R3 work; no new state, semantic choice, inference, validation, or reinterpretation at the B1 acceptance-context seam. | Prove the bounded internal Start's Pending ownership resolves only from exact actor evidence; a public turn from parked truth issues/applies `ResumeOneTurn`; explicit reattach issues/applies `Attach`; the new authority-managed public transport path launches the helper/REPL only after durable application; exact `OwnershipAccepted` accepts ownership; exact `RuntimeCreationRejected`/`StartupFailedBeforeOwnership` terminally reconcile with matching actor/evidence/result identity; timeout/drop/EOF/readiness/process posture or local adapter error remains Pending; transition-scoped retained world turns project the exact already-authorized correlation into the existing B1 acceptance-context seam without semantic drift; and the exact public start → turn/reattach → stop path becomes green. | Bounded auto-attach producer adoption; complete A1 regression closure; A1 completion. |
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, then close A1 without changing projection policy or settlement ownership. | Only `build_auto_attach_launch_plan` and directly required plan integration in `agent_runtime/auto_attach.rs`; focused auto-attach producer/consumer tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, and idempotent applied result. | No auto-attach policy, eligibility, claim, or settlement redesign; no router responsibility expansion; no endpoint/path redesign; no A2 demotion; no subsequent-slice work. | Prove manual and auto-attach cannot substitute or double-apply an intent, retry/restart converges, every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` pass on real CLI and REPL paths; only then may A1 close. | No A1 gate remains claimable until this packet's final wall passes; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows remain unresolved for A2/A3, and later sibling-seam gates remain out of scope. |

#### Corrected A1.2 internal dependency split

The aggregate A1.2 row above remains the full ownership envelope, but its implementation and
dependency order are now split into two independently reviewable packets. This subsection is
normative where the older aggregate wording says that all of A1.2 starts after C1.

| Packet | Exact bounded goal | Explicit exclusions | Exit and dependency |
|---|---|---|---|
| **A1.2a — current-authority establishment/read prerequisite** | First perform the strict greenfield-only V1-to-V2 root upgrade, then implement greenfield-certified Start reservation, issuance, claim, single application/initial authority birth, crash reconciliation, and exact retry. Applied Start retains `claimant_attempt_id` and records startup ownership as `Pending` with the exact run, original application revision, and active participant; resolving that pending substate remains later work. Add one typed exact read result that joins the applied Start descriptor to store/session/active participant/lineage/workspace/world/revision/current-policy truth and exposes the accepted-home bound StateStore capability. | No defaulted/optional V2 field on V1, no B1-owned accepted-work/correlation type, no V3 schema, non-greenfield root conversion, Attach or ResumeOneTurn, startup-ownership resolution or post-turn episode reconciliation, obligation snapshot or disposition, transition correlation supplied to world work, any production consumer/adopter, helper plan, public CLI/REPL/auto-attach adoption, or B receipt/supervisor semantics. | Requires only landed A1.1e. A1.2a is landed and independently review-clean at `b5f2b4f8`; its Host/world-binding write/read correction is isolated in A1.2a-WB before adoption. A1.2a does not close A1.2, A1, or `RG-AUTH-03`. The preserved broad A1.2 checkpoint is not restored. |
| **A1.2a-WB — Host/world-binding validation correction** | Correct only the Start validation relation between descriptor/launch runtime placement and durable session world binding across issuance, application/persistence, and exact current-authority resolution. Descriptor scope must equal requested launch scope. Accept `Host + None`, `Host + Some(exact)`, and `World + Some(exact)`; reject `World + None` and every empty/malformed binding with zero mutation. Persist and return an exact present binding on the resulting durable session authority. | No schema, canonical JSON byte, golden-vector, V3, migration, compatibility bridge, persisted-object rewrite, new authority field, participant placement field, host-to-world placement reinterpretation, world capability/policy change, unrelated Start validation/adoption, or facade behavior outside `HostSessionAuthority::resolve_current_exact`. | Requires landed, review-clean A1.2a. The four-case write/read matrix, mismatch/malformed zero mutation, exact retry, changed-ID/generation conflict, applied and resolved `Host + Some` preservation, host-scoped participant placement, corrupt/substituted persisted combinations failing closed, and unchanged V1/V2 canonical bytes/fixtures are review-clean through `275f9fa2` under existing `RG-AUTH-01`, `RG-AUTH-03`, `RG-BASE-02`, and `RG-DIFF-01`; no new gate was added. |
| **A1.2a-S — bounded internal Start adoption prerequisite** | Make only the ordinary greenfield `prepare_host_orchestrator_runtime_from_resolved` path produce an unpersisted `GreenfieldHostStartProposalV1` carrying the accepted store/home, normalized workspace, descriptor, policy, and shell observation but no authoritative session/participant/run identity. Keep that proposal distinct from the existing fully materialized `PreparedAgentRuntime`. On the real dormant-host launch path, `dispatch_targeted_follow_up_turn` obtains the exact optional initial world binding and calls `apply_greenfield_host_start_from_authority`; before transport or legacy persistence, that adapter derives the issuer key from the greenfield store identity plus complete canonical Start plan and invokes A1.2a. Exact-join/apply Start allocates the authoritative identities and only then constructs the unchanged-shape `PreparedAgentRuntime`, compatibility session/participant projection, and immutable shared `RuntimeAuthorityContext::Bound`. The authority-managed branch of `start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx` skips activated-store legacy session/participant/snapshot writes while preserving launch, toolbox registration, events, blocking behavior, and in-memory observation. Startup ownership remains Pending. | No pending/partial `PreparedAgentRuntime`, persisted/random proposal authority, placeholder identity, fork/member preparation or remote-member consumer change, hidden-owner launch plan, public CLI Start, Attach, ResumeOneTurn, auto-attach, startup acceptance/failure reconciliation, post-turn behavior, lifecycle/posture transition, helper/CWD policy redesign, B receipt/supervisor, or authority construction outside the A1.2a facade. | Requires review-clean A1.2a-WB. Exact real-path `Some`/`None`, deterministic retry/conflict, pre-application escape exclusion, bound-toolbox, unchanged fork/member, zero activated legacy-write, and Pending-observation proof is independently review-clean through `2f2fecb3`. B1/B2.1-R0 is independently review-clean through `bb3eefba`; B3.2a plus B3.2a-WA are independently review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The active A1.3-P1 packet retains public/helper/successor adoption and all exact startup-result/post-turn gates; the older A1.3 fence remains held only as history. |
| **A1.2b — successor/post-turn completion** | First freeze and publish the strict V2-to-V3 root/intent/state extension that preserves each V2 Start and both R0 maps without importing B1/C1 types into A1.2a; then complete Attach/ResumeOneTurn issuance, claim/application, actor-bound startup/post-turn reconciliation, pending obligation-cut consumption, `ReleaseEligible` transport handoff, retry, and optional transition-correlation supply, while proving exact joins remain valid if a separately validated `Released` transport state is reopened later. Startup resolution may authenticate the original Start application revision only through a unique contiguous R0 registration-proof chain to exact current authority; acceptance leaves that authority unchanged and terminal reconciliation preserves every R0-added lineage member/ref. | No in-place V2 widening, arbitrary stale-revision acceptance, no B1/B2.1 implementation, no B3.1/C1 semantics, no public adoption, and no reinterpretation of ledger truth. | Starts only after the B1/B2.1 joint closeout, B3.1, and C1. On the bound Tuesday, August 4, 2026 source candidate it completed the internal durable successor/post-turn protocol without public adoption, promoted no seam, and left the then-current historical `AUTHORITY_REQUIRED:R3_RESUME` gate. That former macOS-parity replacement is superseded as the global schedule by `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY`. |

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
as the consume-only authority client. The active A1.3-P1 packet remains blocked until A1.2b
exits; the older A1.3 fence remains held as history only.

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

1. **A1.1d-5R1 — Ancestor ACL semantics and diagnostics.** Approve Case A on Linux: replace the
   unprovable physical-ACL-absence requirement with the enforceable no-effective-other-principal-
   authority contract. Model each access/default read as closed `Present(bytes)`, exact-`ENODATA`
   `NoData`, or `Failed(error_class)`. `NoData` means only that the kernel returned no ACL data and
   is accepted solely with descriptor-bound owner/type/stable-identity/no-follow/no-replacement
   proof plus authoritative mode bits. Strictly parse `Present` Linux POSIX access ACLs, apply
   `ACL_MASK`, accept masked non-writing `--x`/`r-x`, and reject any effective write. Reject every
   ancestor default ACL before creation and every observable final-root access/default ACL; keep
   final-root exact owner/`0700` and sensitive descendant `0700`/`0600` rules. Malformed,
   unsupported, non-POSIX, unreadable, or otherwise distinguishable non-`ENODATA` state fails
   closed. Diagnostics distinguish accepted non-writing `Present`, rejected `Present`, qualified
   `NoData`, and failure/unavailability and identify requested/offending paths, path role, ACL kind,
   reason, and candidate-creation state without repair language, principals, or secrets. Supporting
   an ancestor default ACL or proving physical xattr absence requires a later separately approved
   contract/platform-attestation boundary.
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
   unreachable after R1. R3 also owns activation of the already-fixed PM-bound Lima SSH-UDS target,
   but only after PI-101/PI-113/PI-114 make unlink, timeout, teardown, retry, and convergence exact;
   that activation selects no new prefix, home, principal, instance, or transport identity.

##### A1.1d-5R2-0 — Prefix-propagation implementation-packet freeze

R2 remains a bounded acyclic execution graph. The failed R2-2 integration closeout refines that
packet into preserved Routes A–E, F0/F0a/F0b test-isolation prerequisites, one remaining R2-2F increment, and a renewed
join; an implementer may not select a convenient subset:

```text
A1.1d-5R1
  -> A1.1d-5R2-0
  -> A1.1d-5R2-1
  -> A1.1d-5R2-2 Routes A-E
  -> A1.1d-5R2-2F0/F0a/F0b/F0-HC complete
  -> A1.1d-5R2-2F
  -> renewed A1.1d-5R2-2 integration closeout
  -> A1.1d-5R2-3
  -> A1.1d-5R2-4
  -> A1.1d-5R3
```

R2-2E, R2-2F0/F0a/F0b, R2-2F, and renewed R2-2 closeout are deliberately ordered before R2-3: E establishes the
explicit config/policy/network projection, F0/F0a/F0b make the proof wall deterministic without changing
production, F consumes E for runtime/dependency and doctor truth, and the production-fix-free
closeout joins Routes A–F. R2-3 then consumes the carrier and generated-
projection rules proven by R2-1/R2-2. The R2 packets' only permitted final join is the
no-production-change R2-4 matrix. The 118 live edges and their exact owners are the canonical
`PI-001`–`PI-118` inventory in `02-seam-crosswalk.md`; each implementation packet must cite the rows
it changes and must stop before introducing an uninventoried edge.

###### A1.1d-5R2-1 — Host context construction and Unix dev propagation

| Packet field | Frozen requirement |
|---|---|
| Goal | Construct one `InstallBootstrapContextV1` at Unix dev install/uninstall/dev-shim and standalone install-sensitive product-CLI entry; order explicit-context private-home bootstrap after construction; propagate through automatic/explicit shim deploy/remove/status/doctor/repair and generated dev projections; prove custom A under conflicting ambient B on focused unprivileged Linux. |
| Prerequisites | R1 and R2-0 review-clean; clean starting tree; current GitNexus index; no R3 work. |
| Must read | `01-target-architecture.md` installation-prefix invariant; `02-seam-crosswalk.md` PI-001–PI-004, PI-010–PI-011, PI-032–PI-034, PI-050, PI-061–PI-064, PI-067, PI-082–PI-085, PI-104, and PI-117, with PI-092–PI-093 as frozen R3 actions; `04-contracts-and-gates.md` bootstrap/framing/selection contract; `05-debug-regression-ledger.md` R2-UDEV-01, R2-SHIM-01, R2-GEN-01, R2-DIAG-01. |
| Sibling context | R2-2 owns release/sudo/world/runtime propagation; R2-3 owns host/guest mapping plus physical shim/replay factory projection; R3 owns every deletion or rollback action. |
| Exact production-file allowlist | `scripts/substrate/{dev-install-substrate.sh,dev-uninstall-substrate.sh,dev-shim-bootstrap.sh}`; `crates/transport-api-types/{Cargo.toml,src/lib.rs}` host-context/framing sections and exact dependency declarations; `Cargo.lock` only if those declared existing dependency edges change its package dependency list; new `crates/shell/src/execution/install_bootstrap.rs` plus its module declaration only in `crates/shell/src/execution/mod.rs`; `crates/shell/src/execution/{cli.rs,auto_sync.rs,env_scripts.rs,home_bootstrap.rs,manager.rs,routing.rs,shim_deploy.rs}`; `crates/shell/src/execution/{invocation/plan.rs,routing/telemetry.rs}`; `crates/shell/src/scripts/bash_preexec.rs`; `crates/shell/src/builtins/shim_doctor/{mod.rs,output.rs,repair.rs,report.rs}`; `crates/trace/src/{context.rs,util.rs}`. No other production file. |
| Exact production sections/symbols | Shared `InstallBootstrapContextV1`/carrier and fixed line framing in transport types; OS principal/path construction and validation in the one shell module; exact Unix release-link/dev-A-bin-symlink installed-product invocation-witness construction; current Unix principal binding for internal carriers; module declaration only in execution `mod.rs`; `Cli.install_prefix`, hidden `Cli.install_bootstrap_context_v1`, hidden `Cli.install_bootstrap_home_v1`, and the `auto_sync.rs` literal-field compatibility update; `install_bootstrap.rs` hidden-action validation/dispatch intake; `run_shell`/`run_shell_with_cli` construction/child-discrimination/principal-binding order, non-mutating parse/help/`--version`/`--version-json` exits, mandatory checked environment projection, typed leaf dispatch, the hidden bootstrap action's call to the explicit-context `home_bootstrap` intake followed by immediate return, and one pre-manager/policy explicit product trace binding to `A/trace.jsonl` plus policy Git directory A; dev install/uninstall prefix/context/bootstrap, shim invocation, projection writers, and the exact PI-003 copy from the already-selected version manifest to generated base `A/manager_hooks.yaml` only after IH exists; dev-shim parse/context, existing shim-call carrier, and `write_env_file`; `render_env_sh`, `write_env_sh_at`, `MANAGER_ENV_SCRIPT`, `write_manager_env_script_at`; invocation-plan deploy/remove/status carrier plumbing, A-derived preexec path, and trace reuse only; `BASH_PREEXEC_SCRIPT` plus `write_bash_preexec_script` explicit A/self-binding; `ShimDeployer` context/path constructor and automatic deploy call only; shim doctor/repair dispatch signatures, report/path derivation, intended-principal account-home repair target, text/JSON projection, and bound-trace reuse; manager and routing-telemetry trace calls may only reuse the entry-bound projection. The hidden bootstrap action requires the authenticated argv carrier, current-principal equality, and checked H/R, calls only the existing private-home/dependency scaffold, emits no carrier/secrets, and owns no shim/runtime/world/policy/lifecycle behavior. In `trace/context.rs`, R2-1 may add a closed explicit product-bound representation/constructor storing exact `A/trace.jsonl` and policy Git directory A; its bound `init_trace(None)` initializes/reuses A and a conflicting explicit path rejects. `TraceContext::default()` and the exact pre-existing callers retain named `LegacyAmbientCompatibility` behavior, which is not contract-correct and is forbidden on migrated shell paths. `set_global_trace_context` keeps its existing signature and set-once behavior and gains no IH/path/caller logic. In `trace/util.rs`, additive `get_policy_git_hash_at` accepts the explicit directory and never falls back; the existing ambient compatibility function stays behaviorally unchanged. No caller-identity side table. Trace writer, rotation, retention, serialization, span, replay, policy, and environment-hash semantics are byte-frozen. `ShimDeployer::{ensure_deployed,deploy_shims,migrate_old_shims}` replacement/migration predicates/bodies, invocation-plan recursive shim-remove action, old ambient-home `.substrate_preexec` removal, repair write/backup semantics, and all installer deletion/removal/recursive-fallback sections are byte-frozen except carrier/target arguments at existing calls. The A1 `CanonicalJsonV1` implementation and every HostSessionAuthority semantic caller are frozen. |
| Allowed tests | Colocated `#[cfg(test)]` modules in the allowed Rust files; `crates/trace/src/tests.rs`; `crates/shell/src/execution/{invocation/tests.rs,routing/builtin/tests.rs}` context-call updates; `crates/shell/tests/{installer_env_wcu4.rs,scripts.rs,shim_deployment.rs,shim_doctor.rs,shim_health.rs,shim_status_fs_mode.rs}`; `crates/shell/tests/world_deps_home_scaffold_wdh3.rs` only to construct a valid shared IH carrier for its selected root/current Unix principal, replace historical `--version` bootstrap triggers with `--install-bootstrap-home-v1` plus the hidden carrier, apply the exact mapped test rename below, and add version nonmutation proof while preserving every R1 fixture/security assertion; dev-mode portions of `tests/installers/{install_smoke.sh,install_state_smoke.sh}`; new `tests/installers/{prefix_propagation_r2_1.sh,dev_shim_bootstrap_context_r2_1.sh,standalone_cli_prefix_r2_1.sh}`. |
| Explicit non-goals | Release install/uninstall; sudo/systemd/world/runtime-family propagation; macOS/Windows/WSL; cleanup, removal convergence, rollback action, artifact manifests; new path resolver, state table, install-root/state-root split, policy/world/runtime behavior. |
| Exit gate | One entry-owned context; parse failures, help, `--version`, and `--version-json` cause zero scaffold mutation; no successful mutating route bootstraps before context; `--install-bootstrap-home-v1` succeeds only with the authenticated hidden argv carrier, current account+UID equality, and checked H/R, calls only the explicit-context private-home/dependency bootstrap, and returns without normal dispatch; hidden child argv, never the environment carrier, selects internal mode; Unix release-link and dev-A-bin-symlink witnesses self-derive A, while direct repo and zero/multiple witnesses fail; A=B conflict cannot retarget children; both H/R and encoded commitment reach automatic/explicit shim and repair/doctor leaves; repair shell-home target comes from the committed principal; every R2-1 shell product path uses the explicit product-bound posture, writes only `A/trace.jsonl`, reads policy Git only from A, reuses A on repeat, rejects a conflicting path, and never touches B for missing metadata; default/unbound physical-shim/replay/platform compatibility remains byte- and behavior-equivalent, explicitly unpromoted, and outside R2-1 proof; generated env/manager/preexec files and `A/manager_hooks.yaml` are prefix-relative/self-bound projections; dev install/uninstall select the same A; forged-principal carriers fail before mutation; all allowed tests, R2-UDEV-01, the R2-1-owned shell/compatibility clauses of R2-SHIM-01, and R2-GEN-01 pass. |
| Regression gates | Formatting, warnings-denied touched targets, `git diff --check`, GitNexus change detection, private-home R1 focused tests, existing shim and installer differential with no pass-to-fail/name substitution except the single canonical one-to-one mapping `test_bootstrap_scaffolds_deps_on_version` -> `test_install_bootstrap_action_scaffolds_deps`; the mapped test keeps the same fixture and scaffold assertions and changes only its trigger, all other historical names remain exact, and one new passing `test_version_is_non_mutating` is required. |
| GitNexus posture | LOW direct impact is expected for routing/home-bootstrap and MEDIUM/HIGH at shared script/invocation/diagnostic leaves. CRITICAL graph breadth is authorized only for the additive/default-preserving `TraceContext` representation or constructor when the setter is unchanged, default callers remain behavior-equivalent, no shim/replay/platform file changes, only shell selects the new posture, and final detection shows no new process family. A semantic edit to `set_global_trace_context`, compatibility behavior, trace lifecycle, physical shim, or replay is forbidden. `substrate_home` remains frozen. Any other CRITICAL or unexpected world/runtime process is a stop-and-review condition. |
| Independent reviews | Host-context authority/security; Unix installer/uninstaller propagation; allowlist/regression sufficiency. All CLEAN on the final R2-1 commit. |
| Platform evidence | Native unprivileged Linux. Static Unix inspection may supplement it; it does not prove macOS. |
| Stop conditions | Context cannot be single-owner; a required file/symbol is outside the allowlist; cleanup semantics are needed; ambient B must be accepted; a new seam/side table/root split is proposed; dirty or unexpected generated changes. Stop as `ArchitecturalBoundaryDecisionRequired` for the architecture cases, otherwise `CrossDocumentChangeRequired`. |
| Next packet | A1.1d-5R2-2 — Unix release, sudo, Linux service, and runtime propagation. |

The R2-1 dependency surface is closed: `crates/transport-api-types/Cargo.toml` may add exactly
`base64 = "0.22"` and `sha2 = { workspace = true }` for the frozen carrier/base64url and SHA-256
commitment implementation. `Cargo.lock` may add only `base64 0.22.1` and `sha2` to the existing
`transport-api-types` package dependency list; both packages already exist in the lock, so a new
package/version/checksum, any other manifest dependency, or any other lock hunk is a stop. Within
the already-allowed `manager.rs`, `manager_manifest_base_path` and `configure_manager_init` receive
typed IH and select only `A/manager_hooks.yaml` plus `A/manager_hooks.local.yaml` in normal product
mode; ambient/repository selection is forbidden just as it is for PI-116.

The R1 private-home trigger migration is a canonical trigger correction, not a removal,
substitution, or weakening. It is exactly one old/new name mapping; its selected home, fixture
layout, umask matrix, wrong-type/owner/mode/symlink/ACL cases, no-descendant-write/no-repair checks,
exit categories, scaffold assertions, idempotency, and no-overwrite proof remain unchanged. No other
historical test may be renamed, and world-deps production command processing remains R2-2-owned.

**R2-1 closeout:** runtime commits `1acc8c70`, `d9d991b6`, `aea192f6`, `85798bfe`, `fee75ff0`,
`af85adf0`, and `2653c2ef` implement and remediate this packet after the separately published
contract corrections `e42b1a1e` and `1565c2ac`. The exact IH golden/tamper wall, current/forged
principal checks, hidden-action/version nonmutation tests, R1 scaffold wall, A/B installer scripts,
shim/doctor/health/generated projection tests, trace/replay compatibility tests, workspace
all-target check, warnings-denied Clippy, formatting, shell checks, and GitNexus detection pass. The
final shell differential—and the **R2-2 historical starting baseline**—is **1089 passed / 149 failed / 1238 total** against inherited
**1080/149/1229**: `PassToFail = 0`, `NewFail = 0`, no removed/substituted test, and all 149 retained
normalized failure signatures are unchanged. The only historical rename is the authorized
scaffold-trigger mapping; the version-nonmutation test and nine R2-1 unit additions pass. Fresh
final integrated review is CLEAN. At that historical closeout the next packet was R2-2;
R2-3/R2-4/R3 remained unstarted. The live next increment is recorded in the corrected R2-2 rows
below.

###### A1.1d-5R2-2 — Unix release, sudo, Linux service, and runtime propagation

| Packet field | Frozen requirement |
|---|---|
| Goal | Carry the proven host context through direct/wrapped Unix release install/uninstall selection, every dev/release/provision sudo boundary, world dependency/runtime-family and direct world-deps/doctor leaves, config/policy/gateway proof producers and consumers, intended-principal PATH/Codex/doctor projections, Linux unit/socket/drop-in generation and restart; derive `ReadWritePaths` from A. |
| Prerequisites | Final R2-1 commit and carrier/projection tests; current GitNexus index; clean tree. |
| Must read | R2-1 closeout; `02-seam-crosswalk.md` PI-005–PI-008, PI-013–PI-021, PI-023–PI-030, PI-035–PI-038, PI-065–PI-066, PI-071–PI-074, PI-080, PI-086–PI-089, PI-105–PI-111, and frozen PI-094–PI-095/PI-103; `04-contracts-and-gates.md` Unix principal, carrier-authority, sudo, uninstall symmetry, diagnostic, and R3 boundary clauses; R2-UREL-01/R2-RUNTIME-01/R2-LINUX-01/R2-DIAG-01. |
| Sibling context | R2-1 owns carrier construction/common generated consumers; R2-3 owns platform-native mappings; R3 owns removal/rollback/convergence even where an allowed script contains those sections. |
| Routes A–D frozen production-file allowlist | `scripts/substrate/{dev-install-substrate.sh,install.sh,install-substrate.sh,uninstall.sh,uninstall-substrate.sh,world-enable.sh}`; `scripts/linux/{world-provision.sh,substrate-apply-socket-acl.sh}`; `crates/shell/src/builtins/{health.rs,world_gateway.rs,world_enable/runner.rs}`; `crates/shell/src/builtins/shim_doctor/mod.rs` only for the Unix crate-private re-export of existing `report::collect_report_for_context` and the item-level Unix `unused_imports` annotation on its existing `collect_report` compatibility re-export; `crates/shell/src/builtins/shim_doctor/report.rs::{collect_report` item-level Unix `dead_code` annotation only, `collect_report_for_context,build_report,gather_world_doctor_snapshot,gather_world_deps_section,try_load_health_fixture,health_fixture_path,run_json_subcommand}` exact Route D carrier/fixture/child-transport closure; `crates/shell/src/builtins/world_deps/{mod.rs,surfaces.rs}`; `crates/shell/src/builtins/world_enable/runner/{paths.rs,manager_env.rs,provision_deps.rs,verify.rs}`; `crates/shell/src/builtins/world_enable/runner/helper_script.rs::run_helper_script`; `crates/shell/src/execution/{config_cmd.rs,policy_cmd.rs}`; `crates/shell/src/execution/config_model.rs::{resolve_effective_config_for_bootstrap_home,resolve_effective_config_with_explain_for_bootstrap_home}`; `crates/shell/src/execution/policy_model.rs::{resolve_effective_policy_for_bootstrap_home,resolve_effective_policy_with_explain_for_bootstrap_home}`; `crates/shell/src/execution/agents_cmd.rs::{handle_agent_command,run_owner_helper}`; `crates/shell/src/execution/invocation/plan.rs::ShellConfig::from_cli`; `crates/shell/src/execution/platform/{mod.rs,linux.rs}`; `crates/shell/src/execution/routing.rs::run_shell_with_cli`; `crates/shell/src/repl/async_repl.rs` exact PI-105 symbols listed below; `crates/shell/src/execution/orchestrator_world_dispatch.rs` exact PI-105 symbols listed below; `crates/shell/src/execution/routing/dispatch/world_ops.rs` exact PI-105 and PI-111 symbols only; `crates/transport-api-types/src/lib.rs::WorldDoctorReportV1`; `crates/world-service/src/handlers.rs::doctor_world`. No other Routes A–D production file or symbol. E/F authority is only in their separate rows below. |
| `run_shell_with_cli` restriction | Read only the already-parsed global `Cli.install_prefix` and nested `WorldAction::Enable(WorldEnableArgs.home)`, normalize and reconcile those public selectors before context construction, pass exactly one selected prefix into the existing R2-1 decode/construct path, and supply the resulting typed IH to existing dispatch. Nested-only or global-only A selects A; equal normalized selectors select A once; unequal selectors fail before context construction, projection, bootstrap, or mutation; when neither selector is present, selection uses only the already-authorized verified installed-product invocation witness and a direct repository binary without one fails before mutation. An internal argv carrier remains authoritative only after strict decode, commitment, and current-principal validation, and every declared selector must match it. For PI-105 only, this function may clone `install_context.context.intended_host_principal` and pass that immutable projection directly to `run_async_repl`; it may not persist or reinterpret it. Ambient home/root, an environment carrier, CWD, generated records, and runner-local state never select A. No `Cli`, `WorldEnableArgs`, IH-constructor, resolver, environment-fallback, runner-reconstruction, or other `routing.rs` change is authorized. |
| `ShellConfig::from_cli` restriction | Unix path only: forward the already-received `install_context` by explicit argument through the existing `SubCommands::World`, `SubCommands::Host`, `SubCommands::Health`, `SubCommands::Config`, and `SubCommands::Policy` branches to their named handlers. No reconstruction, fallback, projection mutation, `ShellConfig::from_args` change, other subcommand branch change, CLI/replay/shim behavior change, process-global/side-table state, or other `ShellConfig` mode is authorized. The previously reported HIGH graph breadth—seven direct callers, the existing `run_shell_with_cli` process family, and the existing Builtin, Execution, and Invocation modules—is approved here only for World's already-reviewed transport plus Host/Health/Config/Policy branch-local argument transport; it does not cover Agent. |
| Shim-doctor module and compatibility-lint restriction | Unix only: `crates/shell/src/builtins/shim_doctor/mod.rs` may expose the already-existing `report::collect_report_for_context` with the narrowest crate-private visibility sufficient for sibling `health.rs`, equivalent to `#[cfg(unix)] pub(crate) use report::collect_report_for_context;`. Its existing crate-private `collect_report` compatibility re-export may receive only an item-level `#[cfg_attr(unix, allow(unused_imports, reason = "…R2-3…"))]`; `report.rs::collect_report` may receive only an item-level `#[cfg_attr(unix, allow(dead_code, reason = "…R2-3…"))]`. Both reasons must name Unix compatibility retention and temporary R2-3 ownership. No file- or module-level allowance, other lint, visibility/signature/body/cfg/caller change, wrapper/resolver, collector/output change, or non-Unix behavior change is authorized. The compatibility collector remains behavior-frozen, barred from typed Health and R2-2 proof, and temporary until R2-3 migration/removal. |
| Bounded Route A CRITICAL aggregate | The immutable production baseline is the eleven-file patch SHA-256 `ea4abf43043013994e698d54f21c04bba58833b3bb3247bf6d5d3b454f51b943`, with the exact manifest/fingerprints recorded in `04`. GitNexus observations over the exact reviewed bytes are historical diagnostics: 33 symbols/18 labels, reverse comparison 11/18, 37/18, 39/18, and the current refreshed CRITICAL result 24/31. Raw symbol or generated process-label counts are not semantic authority. Base containment is decided by exact patch bytes; manifest/fingerprints; source-level production/test diff; manual mapping of every current label to the approved `ShellConfig::from_cli` Host/Health/Config/Policy/context-aware shim-doctor roots or `LineOrHunkAttributionOnly`/`TestColocationOnly`; no new source module, authority source, call path, execution-family root, side table, environment fallback, or lifecycle behavior; and fresh independent semantic review. The current 31-label audit in `04` has no `UnexpectedSemanticPath`. Every future GitNexus run must be retained and remapped; count/label drift alone does not reopen docs while all binding evidence remains exact. Stale-index pinning is forbidden. The test-only successor below is the sole authorized difference from this base; any other patch/fingerprint/source-level difference, unmappable label, new semantic root/module/authority source, or semantic expansion stops as `ImpactDecisionRequired`. This exception never applies to Route B–D, R2-3, or another increment. |
| Route A test-only review-remediation successor | Production hunks and every base hunk not named here remain byte-identical to the preserved patch. The successor may add only: item-level `#[cfg(unix)]` on `config_model::tests::explicit_bootstrap_home_explain_uses_selected_global_config_path` and `policy_model::tests::explicit_bootstrap_home_explain_uses_selected_global_policy_path`, plus the same item-level cfg on each exact test-only `HostSessionAuthority` import and on the policy test module's sole `tempfile::TempDir` import so warnings-denied non-Unix test compilation has no unused import; neither test body nor any other import/module cfg changes; one focused or bounded-strengthened `shim_health.rs` regression in which typed A survives conflicting ambient H/R/HOME=B, reports/reads only A-derived state, uses syscall/path interception to reject any B operation, preserves a complete B-tree snapshot, and cannot be satisfied by the environment-compatible collector; one focused or bounded-strengthened `doctor_scopes_ds0.rs` Host regression using the canonical R2-1 installed-product invocation witness for A, no explicit selector, conflicting ambient B, exact A-derived Host/config output and dependency scaffold, no B dependency artifact, complete B-tree preservation across both installed-witness dispatch and direct-repository no-witness rejection, plus preserved fail-before-mutation proof; and the final test-parent correction described below. Existing tests are not removed, renamed, substituted, ignored, or weakened; the two tests remain active and unchanged on Unix, while their non-Unix cfg exclusion is the authorized correction rather than a skip. Run fresh pre-edit impact analysis on each existing test symbol; record a new exact ordinary/binary patch hash, manifest, fingerprints, deterministic base-to-successor delta, and every successor-only hunk classified to one of the authorized findings. No speculative symbol/process ceiling applies; GitNexus remains required diagnostic evidence. Any production-byte difference or other test hunk is `SuccessorPatchScopeMismatch`. Native non-Linux evidence remains unavailable when blocked by missing Apple SDK/MSVC tooling and is never reported as success. |
| Route A final portable test-parent correction | The preserved current successor is exact patch SHA-256 `0a53e8b60326e21b4237392bfa99671fa8c19c0cd45eb4c573cae5140e808720` at preservation commit `ad139789ef317b978e96c36ee8170dd503bff8a4`, with exactly the existing thirteen-file manifest. In `config_show.rs::config_current_show_uses_declared_prefix_under_conflicting_ambient_home` and `policy_discovery.rs::policy_current_show_uses_declared_prefix_under_conflicting_ambient_home` only, replace the Linux-only `/run/user/<effective uid>` fallback used to parent the unique selected-A test root with this portable secure Unix rule: use nonempty `XDG_RUNTIME_DIR`; otherwise resolve the current account's nonempty home and use `.cache` or a repository-established fixture subdirectory beneath that same account home; create the parent as needed; fail setup explicitly if neither source exists; and retain owner-only `0700` on the selected root. The account home is obtained from the current account identity rather than the command's ambient/conflicting HOME. `/tmp`, `/var/tmp`, `/run/user/<uid>`, CWD, ambient `SUBSTRATE_HOME`, and conflicting B are forbidden fallbacks. Test names, cfg classification, commands, A/B fixtures, config/policy bytes, assertions, output, and production code remain unchanged. Both pre-edit impacts are LOW with zero callers, processes, or modules. Because both files already belong to the manifest, the final manifest remains exactly thirteen files; every production fingerprint and every other test fingerprint remains unchanged. Compute and preserve the new candidate hash only after this two-hunk correction, and classify any GitNexus count drift solely as diagnostic evidence when no production symbol, execution-family root, or module owner changes. Any fourteenth file, production fingerprint change, other test fingerprint change, or other hunk is `SuccessorPatchScopeMismatch`. |
| PI-027/PI-071 carrier-closure audit | The exact mutable path is `run_shell_with_cli` → `ShellConfig::from_cli` → `handle_world_command` → `run_enable`; those same-process hops transport typed IH explicitly. At the child boundary only, `run_enable` or its existing provision-deps path canonically encodes that exact IH and `run_helper_script` appends the hidden carrier option/value to explicit argv for `scripts/substrate/world-enable.sh`; the script validates child mode, carrier/current-principal or exact sudo-origin binding, and normalized `--home` equality before its already-allowed installer/provision calls. The already-allowed runner `paths`/`manager_env`/`provision_deps`/`verify` consumers remain in the same route, and `log_ops.rs` already consumes explicit paths and requires no authority change. No further production file is required. H/R/carrier environment values remain checked projections only and cannot recover authority. |
| `run_helper_script` restriction | Receive only the existing typed context or its exact canonical authenticated carrier from the two existing callers, add the existing hidden carrier option and value to child argv, and preserve `--home` as a required matching projection. It constructs, reinterprets, validates, logs, or emits no authority and changes no other child argument, path, ordering, environment, execution, stdout/stderr, exit, retry, dependency/runtime, platform, or lifecycle behavior. `world-enable.sh` uses carrier-argv presence as the internal-child discriminator, validates before checked projection or dispatch, and never falls from a failed child validation into public mode. |
| `update_manager_env_exports` restriction | The existing world-enable route supplies explicit A derived from typed IH to `update_manager_env_exports`; the helper no longer selects installation authority from environment state. Its existing config read, world-enabled update, env-file rendering, output, and error behavior remain unchanged. The independently reported HIGH impact is authorized only for this explicit-A argument transport through the existing `run_enable` path, including its existing provision-deps branch. No fallback, reconstruction, cleanup, deletion, rollback, migration, convergence, caller behavior, or other HIGH-impact edit is authorized. |
| Bounded world-enable CRITICAL impact authorization | For the exact preserved six-file WIP, GitNexus change detection reported CRITICAL aggregate breadth: 31 changed symbols, 21 affected existing processes, six files, no new file, and no new execution-process family. That result is authorized only when semantic review proves the production diff remains confined to the five named production files and the exact PI-027 route: selector reconciliation in `run_shell_with_cli`; Unix world-branch argument forwarding in `ShellConfig::from_cli`; typed-IH forwarding only to the enable arm of `handle_world_command`; typed-IH-derived A in `run_enable` and its existing provision-deps branch; and explicit-A intake in `update_manager_env_exports`. Every detected process must remain in an existing family; non-enable world arms remain behavior-equivalent; `run_sync_after_provisioning` and other graph-detected but textually untouched symbols remain semantically unchanged; extra breadth may arise only from signature/argument propagation, test colocation, or line shifts. Any new production file, owner, process family, or additional HIGH/CRITICAL symbol stops for a new decision. |
| World-enable child-boundary HIGH authorization | GitNexus reports HIGH impact for `run_helper_script`: two direct callers, three impacted symbols, three affected existing processes, the existing Runner and Execution modules, and no new process family. This result is authorized only for transporting the exact authenticated carrier derived from the callers' typed IH on explicit child argv as restricted above. It grants no other HIGH-impact edit in the function or callers; any additional symbol, module, process family, argument/environment behavior, authority source, capability, or lifecycle change stops for a new decision. |
| PI-105 hidden owner-helper HIGH authorization | A separate semantic authorization applies to the same GitNexus HIGH result for `ShellConfig::from_cli`—seven direct callers, one existing `run_shell_with_cli` process family, Builtin/Execution/Invocation modules, and no new process family—only for forwarding typed IH through the existing hidden `AgentAction::OwnerHelper` arm to `handle_agent_command`/`run_owner_helper`, which extracts the immutable `PlatformPrincipalV1`. Every public Agent action, all parsing, and all other branches remain behavior-equivalent. This is not covered by or an expansion of the Host/Health/Config/Policy authorization, and it grants no other HIGH-impact Agent edit. |
| PI-105 compatibility lint retention | `crates/shell/src/execution/orchestrator_world_dispatch.rs::dispatch_run_world_task_request_with_started_task_run_id_tx` may receive only an item-level `allow(dead_code, reason = "…")` whose concise reason states that the principal-less compatibility entrypoint is intentionally unused and retained until the temporary R2-3 migration. Its visibility, signature, body, cfg, callers, and behavior remain unchanged; it is not called merely to satisfy lint, cannot inject an allowlisted Codex seed without typed intended-principal truth, and remains fail-closed before credential projection. No other lint allowance is authorized. |
| Bounded Route B CRITICAL aggregate | The immutable Route B WIP is exact ordinary/binary patch SHA-256 `f8f845ef6aa6688fdf60be6ed983bb119971d65895a0f38c25fc1dad7643a77f`, preservation commit `a6e29a10ea3dc9cb673a22912002efb83658e7b2`, and exactly the six production files named in the PI-105 allowlist. Current GitNexus detection is CRITICAL with 17 attributed symbols and the 25 existing process labels mapped below; no new process family or module owner. Containment is decided by the exact six-file manifest, production-hunk inventory, mapping every current label to an authorized Route B root, no new semantic execution-family root/module, and fresh independent semantic review—not by a speculative future symbol count. The final candidate may differ from the preserved patch only by the item-level lint annotation above in the already-present `orchestrator_world_dispatch.rs`. Any seventh file, other production hunk, changed compatibility body, unmappable label, new module/owner/family, or semantic expansion is `ImpactDecisionRequired`. This authorization cannot cover Route C, Route D, R2-3, capability, cleanup, or lifecycle work. |
| Route B mechanical rustfmt successor | The reviewed pre-format candidate is exact ordinary/binary patch SHA-256 `e9da85eb206be522645120dfee459ee79ce48793bc61161487d4b5c4d2fda243`, preservation commit `07f3117aa0d2f3d57279d20fec204757bba8383a`, and the same exact six-file Route B manifest. Its sole successor may run repository-default `cargo fmt --all` and retain only canonical rustfmt layout changes in `execution/agents_cmd.rs`, `execution/orchestrator_world_dispatch.rs`, `execution/routing/dispatch/world_ops.rs`, and `repl/async_repl.rs`; `execution/invocation/plan.rs` and `execution/routing.rs` remain byte-identical to `e9da85eb…`. The successor establishes a new exact patch hash and per-file fingerprints after formatting. Completion requires `cargo fmt --all -- --check`, a zero-output semantic comparison with whitespace ignored for the four formatted files, an exact hunk audit proving layout-only movement, unchanged strings/comments/import membership/cfg/tokens/control flow, fresh GitNexus detection with every label mapped to an existing Route B root or formatting attribution, and fresh independent runtime review. Raw symbol/label drift is diagnostic only when the manifest, semantics, owners, and execution-family roots remain fixed. A seventh file, non-rustfmt hunk, changed unaffected-file fingerprint, new module/family/semantic path, or other remediation is respectively `FormattingScopeMismatch`, `CrossDocumentChangeRequired`, or `ImpactDecisionRequired`; this exception applies only to this exact Route B successor and authorizes no general cleanup, Route C, Route D, or R2-3 work. |
| Route B security-remediation successor | The immutable formatted candidate is ordinary/binary patch SHA-256 `ea9cf3e582650007083812ad70e0bf3198405e73d0cde0fb8ecfbedeb49884a2`, preservation commit `57e13d291abf1239aacee0020ac444ec05e11d56`, and the same six-file manifest. Its sole security successor may: in `world_ops.rs::maybe_inject_codex_auth_seed_home_for_policy`, remove `SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME_ENV` before every backend, allowlist, or principal decision and insert only after successful typed-principal account/UID resolution; in `orchestrator_world_dispatch.rs`, carry `Option<&PlatformPrincipalV1>` separately through `spawn_world_worker` and `spawn_prepared_world_worker`, pass `Some(exact principal)` from both principal-aware direct/prepared Spawn entrypoints and `None` from compatibility entrypoints, and forward it into the already-existing `execute_spawn_world_worker_stream` parameter without changing prepared/admission/durable types; and in `world_ops.rs`, replace only the Unix-account resolver/helper gates with `cfg(any(target_os = "linux", all(test, unix)))` plus mechanically matching test/import/constant cfgs. Focused poisoned-key, direct/prepared Spawn, compatibility, and static cfg regressions may be added only within these existing files. The final manifest remains exactly the six files named above; no schema, persistent state, environment recovery, process-global mutation, policy change, capability change, Route C/D, or R2-3 behavior is authorized. Establish and preserve a new exact patch/hash/fingerprint/hunk map after TDD. Any seventh file, additional semantic hunk, durable-principal representation, new module/semantic family, or unmappable impact is a mandatory stop. |
| Route B security impact/source closure | Pre-edit GitNexus reports LOW for `maybe_inject_codex_auth_seed_home_for_policy` (four direct callers, one existing `build_agent_client_and_member_dispatch_request_impl` process root with eleven generated labels, Dispatch module) and LOW for `resolve_host_codex_seed_home` (one direct caller, the same process/module). GitNexus under-resolves the large `orchestrator_world_dispatch.rs` Spawn symbols, so manual closure is binding: compatibility/principal-aware direct dispatch → unchanged `prepare_authority_bound_spawn_world_worker` → `spawn_prepared_world_worker` with separate `None`/`Some`; compatibility/principal-aware prepared dispatch → `spawn_world_worker` with `None`/`Some` → unchanged `prepare_spawn_world_worker_bootstrap` → unchanged Linux `prepare_authority_bound_spawn_world_worker` → `spawn_prepared_world_worker`, with the same separate option carried alongside the prepared result; then `spawn_prepared_world_worker` → existing `execute_spawn_world_worker_stream` → existing member-request construction/injector. Both preparation functions are audited, textually unchanged, receive no principal, and store none in `PreparedSpawnWorldWorkerBootstrap`. Fork and continue-fork already carry the same separate parameter and remain unchanged. HSA, admission, receipt, supervisor, retained-worker, policy, manifest, and transport schemas remain unchanged. Fresh impact/detection output and manual label mapping remain mandatory; any different semantic root stops as `ImpactDecisionRequired`. |
| Route C bounded diagnostic projection | Route B is complete, review-clean, and unchanged at `6cee990f0370013c8b05a5495301db7aea642cd5`, exact ordinary/binary patch `28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674`, across its existing six files. Route C begins only from preservation commit `41b82327e23798719ed9a0b4cae1f557fb593670`, parented by that exact Route B tree, with full-index binary patch `5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277`, the exact five-file PI-035 manifest, and fingerprints in `04`. It may carry typed authenticated IH into existing Host/World doctor collection, add/populate only non-secret optional diagnostic prefix/commitment fields, preserve typed context through those existing paths, and prove ambient B cannot replace A. The preservation-time CRITICAL observation was 25 attributed symbols/32 labels/five files; a fresh identical-byte refresh reported 19/32/five. These are observed attribution evidence, not numerical ceilings. Binding containment is the exact five files and starting bytes/fingerprints; source mapping of every process label to only `handle_host_command`, `host_doctor_main`, `handle_world_command`, or `world_doctor_main`; no new module owner/execution family/authority seam; and fresh semantic review. Attribution-only drift is acceptable only when all binding evidence remains exact and a fresh reviewer confirms it. Review-driven remediation may change bytes only inside the same five-file semantic envelope, then must establish a new patch/fingerprint manifest and repeat impact, proof, differential, and fresh review. A sixth file, actual new call path, new owner/family/seam, ambient authority recovery, sensitive disclosure, or any world/policy/capability/credential/receipt/supervisor/retained-worker/lifecycle/execution change stops as `ImpactDecisionRequired`; remediation requiring broader documentation/ownership stops as `CrossDocumentChangeRequired`. |
| Remaining-carrier closure audit | Route A is `run_shell_with_cli` → `ShellConfig::from_cli` → the exact Host/Health/Config/Policy branch → named handler → typed consumer; Health continues through `health.rs` → crate-private `shim_doctor::collect_report_for_context` → existing `report::collect_report_for_context`, while Config/Policy current-show use the existing opened-bootstrap-home resolver with additive explain parity and CWD still selects only workspace scope. Same-process typed IH is present at every hop. Route B is root IH → immutable `PlatformPrincipalV1` → normal REPL or hidden owner-helper runtime context → prepared world dispatch/member request → Codex seed resolver; every hop is same-process/request-scoped and non-durable. Route C is world-service `doctor_world` (`None`) or Linux legacy producer (`Some` only with typed IH) → optional/defaulted `WorldDoctorReportV1` host fields → host-shell enrichment; world service never reads host context. Route D is typed shell IH → Health/shim-doctor `collect_report_for_context` → `build_report`; its dependency branch retains only A-rooted `world_deps.json` lookup or `gather_world_deps_section` → `collect_doctor_snapshot_v1` → A-derived config/deps paths. Its World-disabled branch remains the frozen `disabled_world_doctor_snapshot` short-circuit with no child or fixture lookup. Under F5-PD, whenever the authenticated Linux World-enabled branch reaches `gather_world_doctor_snapshot`, it never selects a production fixture and continues through `run_json_subcommand` → the hidden passive `world doctor --json --internal-passive-world-doctor-v1` child with the exact authenticated carrier and A-derived checked projections. Linux raw stdout passes duplicate-rejecting exact-wire decoding before value validation. `A/health/world_doctor.json` is `cfg(test)` evidence only on that path. Existing non-Linux fixture/public-child compatibility is unchanged and cannot satisfy F5-PD/F5 proof. Unix `collect_report` remains the named checked-projection compatibility caller, behavior-frozen and barred from R2-2 proof; physical-shim/replay migration remains R2-3. |
| PI-105 exact symbols/restriction | In `async_repl.rs`, only `run_async_repl`, `DormantHostOrchestratorLaunchPlan::{backend_id,into_proposal}`, `prepare_repl_dormant_host_launch_plan`, `RuntimeOrchestrationContext`, `GreenfieldHostStartProposalV1`, `prepare_host_orchestrator_runtime_from_resolved`, `apply_greenfield_host_start_from_authority`, `prepare_hidden_owner_helper_runtime`, `run_hidden_owner_helper`, `dispatch_run_world_task_request_with_binding_retry`, `handle_internal_toolbox_world_dispatch_request`, and `start_remote_member_runtime_with_prepared` may transport the immutable projection. In `orchestrator_world_dispatch.rs`, prepared structures stay unchanged; the projection travels as a separate explicit parameter only through `dispatch_orchestrator_world_request_for_principal`, `dispatch_run_world_task_request_with_started_task_run_id_tx_for_principal`, `dispatch_prepared_orchestrator_world_request_for_principal`, `prepare_task_acceptance_submission`, `run_world_task`, `run_world_task_with_started_task_run_id_tx`, `spawn_world_worker`, `spawn_prepared_world_worker`, `fork_world_worker`, `continue_world_worker`, `continue_world_worker_fork_command_bootstrap_after_delivery`, and `execute_spawn_world_worker_stream`. Compatibility dispatch functions pass `None`; principal-aware direct/prepared Spawn pass `Some(exact principal)` at every hop. Therefore `acquire_continue_world_worker_fork_command_bootstrap_guard` and `resolve_continue_world_dispatch_target_for_routing` remain textually unchanged and cannot drop the separately carried projection. Compatibility entry points may omit the projection only when they cannot satisfy R2-2 credential proof and must fail closed before an allowlisted Codex seed is injected. In `world_ops.rs`, the exact builder/resolver/injector symbols named in PI-105 require `PlatformPrincipalV1::Unix`, call unchanged read-only `install_bootstrap.rs::unix_account_home_for_principal`, and derive only `home/.codex`; `maybe_inject_codex_auth_seed_home_for_policy` first removes the reserved seed key on every path and inserts it only after successful exact resolution. Those Unix account symbols compile only for Linux production or Unix tests; Windows tests cannot compile the Unix account leaf. That terminal resolver remains outside the editable allowlist, and no duplicate resolver is permitted. No HSA/session/receipt/supervisor/retained-worker schema, policy decision, prompt/auth content, log, trace, global, environment, or persistence changes. GitNexus is under-resolved for both large files; exact manual source closure and semantic-diff review are mandatory. |
| Doctor ownership/restriction | `WorldDoctorReportV1` may add only optional, defaulted, omit-when-absent selected-host-prefix and host-context-commitment diagnostics. `world-service::doctor_world` initializes both to `None` and reads no IH/carrier/HOME/prefix/principal/environment authority; its enforcement, Landlock, netfilter, filesystem strategy, and response semantics are otherwise unchanged. `platform/linux.rs::{world_doctor_main,host_doctor_main}` attaches the projection from typed IH to final host-visible output before either JSON serialization or text rendering; `legacy_world_doctor_report_v1_via_execute` may set host fields only when its caller supplies typed IH. Existing JSON without those fields remains valid. Impact counts are diagnostic evidence governed by the Route C semantic-containment row, never authority to add or reject unchanged semantics by number alone. |
| Shim-doctor/high restriction | `collect_report_for_context`, `build_report`, `gather_world_doctor_snapshot`, `gather_world_deps_section`, `try_load_health_fixture`, `health_fixture_path`, `run_json_subcommand`, `collect_doctor_snapshot_v1`, and `resolve_effective_enabled_provisioning_requirements_v1` transport typed IH, A-derived paths, or the exact child carrier/projections only. Selected-A fixture payload, report classifications, world/dependency semantics, and non-Unix behavior are unchanged; ambient-B fixture selection and a contextless Unix child are defects, not compatibility. The audited HIGH result for `resolve_current_inventory_view`—nine direct callers, fifteen impacted symbols, three existing processes (`world_deps::run`, `health::run`, `run_enable_with_provision_deps`), four existing modules (World_deps, Execution, Shim_doctor, Runner), and no new process family—is approved only for an explicit A/context parameter and mechanical forwarding through those existing families. Its nine direct callers are the two `world_deps/mod.rs` functions just named plus `surfaces.rs::{run_current_list,run_current_show,run_current_install,run_current_sync,run_workspace_add,run_current_list_enabled,run_current_list_applied}`; `run_workspace` may forward A only to `run_workspace_add`, and `build_current_show_explain_v1` may receive the same A-derived global patch path solely to preserve explain-source parity. All named callers report LOW individual impact. This authorizes no inventory, config, install/sync, runtime-provider, or unrelated branch behavior change. |
| Historical Route D docs-first source-closure correction | The Route D audit found two pre-carrier branches in the existing Health/shim-doctor family: ambient B could select both health fixtures, and the nested child lacked its authenticated witness. Route D corrected A-rooting and carrier transport. F5-PD now selectively supersedes the authenticated Linux World Doctor part: when World-enabled production reaches the gather function it may not read `A/health/world_doctor.json`, invokes the hidden passive child, performs duplicate-rejecting exact-wire decode, and retains Linux `snapshot_from_value` only for tests. The disabled short-circuit, non-Linux compatibility, and world-deps fixture remain frozen; non-Linux is barred from proof and world-deps remains F5-owned. Child transport omits carrier, credential, request, and sensitive-principal bytes and cannot mutate the parent environment. Historical Route D impacts were LOW and mapped only to the existing Health/shim-doctor family; F5-PD's fresh exact impacts and allowlist appear below. A new module, execution family, authority owner/resolver, schema, ambient fallback, physical-shim/replay/platform migration, or behavior outside these exact envelopes is `CrossDocumentChangeRequired`. |
| First correction evidence | Commit `d98e0f0067e36ffddd4d586f0785c9317bf59323` authorized only `run_shell_with_cli` selector reconciliation. Fresh read-only reviews `docs_authority_review` and `docs_scope_rereview` concluded CLEAN after the one ownership-wording remediation; source/local/remote parity and a clean tree were verified before this correction. |
| Routes A–D frozen production sections/symbols | Dev dependency add/current/sync, existing failure/rollback/remove call **carrier arguments only**, `run_privileged`, `install_packages`, `ensure_substrate_group_membership`, `ensure_socket_group_alignment`, sudo preflight, and Linux-provision call arguments; release wrapper/direct-child prefix-context construction/intake/current-principal binding/bootstrap, shim invocation, projection/runtime/dependency calls, PI-016 copy from the already-selected release manifest to generated base `A/manager_hooks.yaml` only after IH exists, PATH target/upsert/reporting, Linux service templates, every sudo argument, install/reload/restart verification, and doctor invocation; release uninstall wrapper/direct-child selection/context intake/current-principal binding/bootstrap only; `run_shell_with_cli`-owned world-enable selector reconciliation and existing IH decode/construction, explicit forwarding through the authorized Unix `ShellConfig::from_cli` branches and named handlers, then typed-IH-only consumers; Linux provision option/context/principal/bootstrap, unit/socket/drop-in rendering/install, group-member/ACL/linger argument construction, reload/restart/verification, `evaluate_gateway_lifecycle_proof_eligibility`, `run_gateway_lifecycle_proof`, and `prepare_gateway_smoke_auth` target argument; ACL helper argument parsing may only reject every tuple except the exact three PI-030 mode/target/`substrate` combinations, while `collect_authorized_users`, ACL construction/application, warning, and degraded-exit bodies stay frozen; the exact remaining-carrier symbols/restrictions above; `world_deps::{run,run_current,run_global,run_workspace,collect_doctor_snapshot_v1,resolve_effective_enabled_provisioning_requirements_v1}`; surfaces `run_current_list`, `run_current_show`, `run_current_install`, `run_current_sync`, `run_workspace_add`, `run_current_list_enabled`, `run_current_list_applied`, `build_current_show_explain_v1`, `run_global_list`, `run_global_add`, `run_global_remove`, `run_global_reset`, `resolve_global_available_inventory_view`, and `resolve_current_inventory_view` explicit A/path intake; config/policy handler/current-show and explicit-bootstrap-home explain adapters only; gateway `run`, `build_gateway_request_context`, `macos_default_world_socket_path`, and `codex_auth_state_path` typed context/path intake only; PI-105 exact in-memory/separate-argument transport and builder/resolver/injector symbols only, with unchanged `unix_account_home_for_principal` as the non-editable terminal leaf; optional doctor fields, mechanical world-service `None`, health/shim aggregation, and Linux host projection only. Context-aware Substrate sudo children validate the full argv carrier/current principal; arbitrary tools receive only exact context-derived argv after parent revalidation; the fixed ACL leaf receives no carrier/principal and accepts only its closed tuples. Config/policy/gateway/world/runtime/orchestration/receipt/supervisor/retained-worker semantics do not change. All named dev/release replacement sections, `cleanup_gateway_smoke_auth`, deletion predicates/actions, disable, legacy replacement, rollback action/convergence, PATH removal, wildcard/recursive removal, and artifact ownership are forbidden. The sole operational exception is the unchanged fixed `/run/substrate.sock` unlink immediately before the same-attempt Linux restart; changing its target/predicate or any legacy/uninstall cleanup is R3. E/F edits are authorized only by the later increment rows. |
| Routes A–D frozen allowed tests | Colocated tests in the named world-deps, shim-doctor, gateway, config, policy, platform, world-ops, PI-105 async/orchestrator, world-enable runner, health, Linux-platform, transport-api, and exact `world-service::doctor_world` files; `crates/shell/tests/{config_show.rs,config_world_deps_phase_b.rs,doctor_scopes_ds0.rs,installer_env_wcu4.rs,policy_discovery.rs,shim_doctor.rs,shim_health.rs,world_deps_enabled_wdp1.rs,world_deps_home_scaffold_wdh3.rs,world_deps_inventory_views.rs,world_enable.rs,world_enable_provision_deps_wdap0.rs,world_gateway.rs}`; `tests/installers/{install_smoke.sh,install_state_smoke.sh,install_wrapper_smoke.sh,provision_agent_runtime_smoke.sh,world_provision_smoke.sh}`; new `tests/installers/{prefix_propagation_r2_2.sh,world_provision_context_r2_2.sh}`. E/F tests are authorized only by their later exact rows. |
| Explicit non-goals | Managed-artifact deletion/manifest, current-attempt rollback action, uninstall convergence, macOS/Windows/WSL mapping, policy/world capability changes, runtime availability/provider semantics, B1 receipt/B2.1 supervisor work. |
| Exit gate | Release install/uninstall select one symmetric `IH`; Unix account+UID remain exact at every release, dev, and provision sudo boundary under the context-helper/arbitrary-tool/fixed-ACL-leaf split; the ACL leaf rejects noncanonical tuples and preserves all-member enumeration without reading H/R/principal; dependency/runtime and world-deps/doctor leaves plus both sides of config/policy/gateway proof receive typed IH or its checked projection. Route D's world-deps fixture derives only from A; after F5-PD an authenticated Linux World-enabled nested World Doctor uses the passive child with the same authenticated witness, rejects duplicate/unknown raw JSON fields, and treats its fixture as test-only, while World-disabled composition preserves its no-child disabled snapshot. Non-Linux compatibility remains unchanged and unproven. Generated release base manifest and global deps/config/policy/socket paths derive from A; member-dispatch and gateway Codex homes derive from the committed account database; synthetic Codex-auth creation targets that principal home while cleanup remains frozen for R3; unit environment contains H=A, R=A, commitment, and intended-principal projection and `ReadWritePaths` derives from A; PATH upsert targets the intended account home; shared/Linux doctors report A; default/custom-prefix focused Linux integration passes without outer overrides. No removal command changes beyond carrier arguments and the frozen operational socket-restart exception. |
| Regression gates | R2-1 gates; R2-UREL-01/R2-RUNTIME-01/R2-LINUX-01; targeted world-enable tests; Route D typed A under ambient B with conflicting B world/deps fixtures, then F5-PD authenticated Linux passive-child proof with World Doctor fixtures test-only, missing/malformed/tampered/contextless fail-closed proof, parent/B-tree nonmutation, sensitive-output/error/log/trace scan, compatibility-collector stability, and unchanged/unproven non-Linux cfg behavior; formatting/Clippy/check/diff/GitNexus; broad shell differential with zero new/pass-to-fail/removed/renamed/substituted/weakened tests and unchanged retained failure signatures; world/policy/B1/B2.1 differentials unchanged. Privileged product smoke is assigned to R2-4, not silently claimed here. |
| GitNexus posture | HIGH is expected around runtime/world-enable and service orchestration; impact must be reported before edits. The bounded world-enable CRITICAL aggregate, manager-environment HIGH, and child-boundary HIGH apply only to the preserved PI-027/PI-071 closure. Route A production authority remains fixed to exact base patch `ea4abf43043013994e698d54f21c04bba58833b3bb3247bf6d5d3b454f51b943`; only the exact test-only successor delta above may extend it. GitNexus counts/labels remain mandatory diagnostics but are not raw completion ceilings; each edited existing test symbol receives normal pre-edit impact analysis, and final detection is mapped semantically against the base-to-successor hunk table. The Host/Health/Config/Policy `ShellConfig::from_cli` HIGH, separately authorized hidden owner-helper use of that same HIGH graph result, additive `WorldDoctorReportV1` HIGH, and `resolve_current_inventory_view` HIGH apply only to the exact branch/field/argument transports recorded above. Large-file PI-105 symbols remain graph-under-resolved and require manual closure. Any unmappable semantic path, new root/module/owner, production-byte change, or test change outside the successor list stops for explicit review. Stale-index pinning is prohibited, and this exception cannot cover Route B–D or R2-3. |
| Independent reviews | Host/principal/sudo security; install/uninstall R2-versus-R3 lifecycle; Linux service/runtime allowlist and regression plan. All CLEAN. |
| Platform evidence | Static and focused native Linux integration. Native non-Linux Route A proof is unavailable in this Linux environment: the Windows target lacks MSVC `lib.exe`, and the Darwin target lacks the Apple SDK header `TargetConditionals.h`. These are tooling limitations, not successful checks or product regressions. Dedicated-host privileged Linux product proof remains R2-4. No macOS/Windows product claim. |
| Stop conditions | Any deletion/rollback ownership is required; service behavior cannot be fixed by selected-context propagation alone; a runtime/world capability or policy seam would change; allowlist is insufficient; context/principal cannot remain exact across sudo. Use the same architecture/cross-document stops as R2-1. |
| Historical next packet | At that checkpoint R2-2E and F0/F0a/F0b/F0-HC were implemented, proof-complete, review-clean, canonically closed out, awaiting their packet-specific replay, and preserved; Routes A–E were immutable prior evidence. **A1.1d-5R2-2F — Authenticated world-deps and truthful doctor composition** was the next packet. The completed-F and renewed-closeout sections below supersede this status. |

Route B's refreshed exact-WIP process-label audit is closed as follows. “Root semantic; terminal
attribution only” means the named root owns the authorized PI-105 argument/projection change while
the generated terminal label names an existing unchanged callee or branch. No row creates a new
execution-family root.

| GitNexus label | Root symbol | Authorized Route B hop | Semantic change or attribution only |
|---|---|---|---|
| `proc_4_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → Canonicalize_net_allowed` | `build_agent_client_and_member_dispatch_request_impl` | Prepared member request receives the explicit principal before policy-permitted seed projection. | Root semantic; unchanged network canonicalization is terminal attribution only. |
| `proc_5_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → New` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; existing config-model/user-error construction is terminal attribution only. |
| `proc_34_run_shell_with_cli` — `Run_shell_with_cli → Lookup_unix_account_by_uid` | `run_shell_with_cli` | Clone immutable intended principal from validated IH into the normal REPL route. | Root semantic; existing account lookup is terminal attribution only. |
| `proc_35_run_shell_with_cli` — `Run_shell_with_cli → Validate` | `run_shell_with_cli` | Same shell-root principal extraction. | Root semantic; existing validation is terminal attribution only. |
| `proc_36_run_shell_with_cli` — `Run_shell_with_cli → Encode` | `run_shell_with_cli` | Same shell-root principal extraction. | Root semantic; existing carrier encoding is terminal attribution only. |
| `proc_45_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → PolicySnapshotWorldFsDimensionV3` | `build_agent_client_and_member_dispatch_request_impl` | Explicit principal reaches policy-gated member request construction. | Root semantic; policy snapshot type construction is unchanged attribution. |
| `proc_46_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → PolicySnapshotV3` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; policy snapshot construction is unchanged attribution. |
| `proc_47_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → PolicySnapshotWorldFsV3` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; world-fs snapshot construction is unchanged attribution. |
| `proc_48_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → PolicySnapshotWorldFsFailClosedV3` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; fail-closed policy representation is unchanged attribution. |
| `proc_49_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → Canonicalize` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; existing policy-snapshot/network canonicalization is terminal attribution only. |
| `proc_50_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → Resolve_world_network_routing` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; network-routing policy remains unchanged attribution. |
| `proc_114_handle_agent_command` — `Handle_agent_command → Current_dir` | `handle_agent_command` | Typed IH is forwarded only through hidden `OwnerHelper`, then reduced to the immutable principal. | Owner-helper arm semantic; public CWD-dependent Agent behavior is attribution only. |
| `proc_115_handle_agent_command` — `Handle_agent_command → Normalized_role_filter` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; role filtering is unchanged attribution. |
| `proc_116_handle_agent_command` — `Handle_agent_command → Role_for_entry` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; public role projection is unchanged attribution. |
| `proc_117_handle_agent_command` — `Handle_agent_command → Matches_scope` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; public scope matching is unchanged attribution. |
| `proc_118_handle_agent_command` — `Handle_agent_command → Matches_role` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; public role matching is unchanged attribution. |
| `proc_119_handle_agent_command` — `Handle_agent_command → Capabilities_label` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; capability rendering is unchanged attribution. |
| `proc_120_handle_agent_command` — `Handle_agent_command → StatusReportJson` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; status representation is unchanged attribution. |
| `proc_140_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → For_path` | `build_agent_client_and_member_dispatch_request_impl` | Explicit principal reaches the member request before account-home seed resolution. | Root semantic; existing path-based policy load is terminal attribution only. |
| `proc_141_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → ResolvedPolicySnapshot` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop. | Root semantic; resolved policy representation is unchanged attribution. |
| `proc_142_build_agent_client_a` — `Build_agent_client_and_member_dispatch_request_impl → Substrate_home` | `build_agent_client_and_member_dispatch_request_impl` | Same prepared-member request hop; A remains unrelated to credential-home selection. | Root semantic; existing Substrate-home projection is terminal attribution only. |
| `proc_280_handle_agent_command` — `Handle_agent_command → Render_status_report` | `handle_agent_command` | Typed IH remains confined to hidden owner-helper. | Owner-helper arm semantic; public status rendering is unchanged attribution. |
| `proc_281_handle_agent_command` — `Handle_agent_command → Doctor_exit_code` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; public doctor exit classification is unchanged attribution. |
| `proc_282_handle_agent_command` — `Handle_agent_command → Render_doctor_report` | `handle_agent_command` | Same hidden owner-helper hop. | Owner-helper arm semantic; public doctor rendering is unchanged attribution. |
| `proc_290_run_shell_with_cli` — `Run_shell_with_cli → Normalize_unix_install_bootstrap_path` | `run_shell_with_cli` | Clone immutable intended principal only after validated IH construction. | Root semantic; existing install-path normalization is terminal attribution only. |

###### A1.1d-5R2-2E — Authenticated world-gateway projection

| Packet field | Frozen requirement |
|---|---|
| Goal | Close PI-111 by making gateway status/sync/restart consume already-authenticated A for configuration, effective policy, policy snapshot, network policy, runtime-family inventory, Codex home, and client selection before any unavailable classification, mutation, forwarding, or launch. |
| Status and exact identity | **Implemented, proof-complete, and review-clean.** Commit `7e8e83802885c0ece93efcaacccc26503eeb6715`; tree `02a1a2f6b7e4be47ed9ec38537c805ae348c96b6`; ordinary patch SHA-256 `fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff`; full-index patch SHA-256 `c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862`; preservation `feat/preserve-a1-1d-5r2-2e-c712f467`. |
| Historical prerequisites and manifest | Implemented on review-clean Routes A–D at `3bf59b30`, `1b5219d`, `0290b829`, and `6452d3a`. The exact five-file manifest is `crates/shell/src/execution/platform/mod.rs`, `crates/shell/src/builtins/world_gateway.rs`, `crates/shell/src/execution/agent_inventory.rs`, `crates/shell/src/execution/policy_snapshot.rs`, and `crates/shell/tests/world_gateway.rs`. |
| Exact production allowlist | `crates/shell/src/execution/platform/mod.rs::handle_world_command` Gateway arm only; `crates/shell/src/builtins/world_gateway.rs::{GatewayLifecycleRequestContext,run,run_inner,run_typed_action_with_status_args,run_typed_action,call_gateway_action,build_gateway_request_context,validate_gateway_backend_selection,resolve_integrated_auth_payload,resolve_cli_codex_integrated_auth,codex_auth_state_path,world_routing_disabled,build_gateway_client` Linux cfg only, `synthesized_unavailable_response_without_context` DELETE}; `crates/shell/src/execution/policy_snapshot.rs::{resolve_policy_snapshot_for_bootstrap_home` reuse/signature closure only, one additive explicit-bootstrap-home world-network resolver}; `crates/shell/src/execution/agent_inventory.rs::{resolve_gateway_backend_inventory_entry,load_effective_agent_inventory_for_bootstrap_home` reuse/signature closure only, one additive bootstrap-home gateway resolver}. The E fail-closed cfg decision is inside the allowlisted `call_gateway_action` path before any client construction. `build_macos_gateway_client`, production-compiled/dead test-support `resolve_macos_gateway_client_endpoint`, `resolve_macos_host_gateway_socket`, and `macos_default_world_socket_path` are source-closure-inspected but frozen R2-3 compatibility symbols; none may be deleted, edited, called, or used for endpoint authority by E. No other production file or symbol. |
| Exact test allowlist | Colocated tests in `world_gateway.rs`, `policy_snapshot.rs`, and `agent_inventory.rs`; `crates/shell/tests/{world_gateway.rs,agent_successor_contract_ahcsitc0.rs}`. `tests/installers/world_provision_smoke.sh` is run unchanged as propagation evidence. |
| Canonical owner | `policy_snapshot.rs` remains the only policy-snapshot/network-policy owner. Gateway code combines explicit results but does not parse, layer, finalize, or duplicate policy. Existing explicit config/policy bootstrap-home resolvers and account-database home resolver are reuse-only. |
| Contract | Validate and bind A before `world_routing_disabled` or unavailable synthesis. A alone selects global config/policy/inventory roots; CWD is explicit workspace scope only. The committed Unix principal selects Codex home. Linux authenticated gateway uses fixed `/run/substrate.sock`, rejecting ambient socket overrides. E has no authenticated macOS endpoint source, so the E route fails before client construction or `world_mac_lima::forwarding::auto_select`; Windows/other contextless entries fail before config, policy, inventory, disabled-state classification, or client selection. Ambient non-Linux clients stay frozen/unreachable and lower forwarding/known-hosts/transport remain R2-3. Existing request/response schemas, service/transport handlers, network allow/deny semantics, placement semantics, and launch-time credential handoff stay unchanged. Runtime-native files are projections only; no host credential file becomes authority. |
| Completed proof | Custom A under conflicting H/R/HOME/CWD/CODEX_HOME/B; config only from A; effective policy and network policy only from A; runtime-family inventory and Codex projection only from A/principal; environment-only invocation rejected; malformed/tampered/mismatched carrier rejected; no mutation/forwarding/launch before validation; disabled routing still requires valid A; no carrier/credential/token/prompt/principal disclosure; unchanged network allow/deny matrix; Linux fixed socket; macOS fails before client/ambient forwarding; Windows/other entries fail before ambient selection; non-Unix build/static cfg preservation without A-bound/native-product claim. Focused results are 13/13 gateway classification, 21/21 config resolution, 14/14 effective policy, 10/10 policy snapshot/network, 17/17 inventory, 8/8 install-bootstrap context, 1/1 explicit `HostSessionAuthority` composition, 3/3 new integration negatives, 7/7 managed auth bundle, 32/32 world-service gateway runtime, and 18/18 gateway receiver/server. The clean Route D comparison is `1101 passed / 149 failed / 0 ignored`; final E produced the genuine post-E success observation `1114 passed / 149 failed / 0 ignored`, with zero pass-to-fail, new-fail, changed-failure, or fail-to-pass transitions and the identical 149-name failure set. F0 records that this observation was not deterministic because the same E runtime also produced `1113 passed / 150 failed / 0 ignored`. |
| Credential and review disposition | The managed-gateway secure-FD producer, receiver, bundle schema, and lifecycle are landed, regression-proven, and unchanged by E. Direct-member Codex/UAA gateway adoption remains unresolved, transitional, non-promotable, and E3/D1/D3-owned; `RG-CONFIG-02`, `RG-CONFIG-04`, `RG-UAA-02`, and `RG-UAA-03` remain open. Final isolated read-only reviews `e_final_gateway_authority`, `e_final_policy_network`, `e_final_credential_boundary`, and `e_final_platform_regression_replacement` are CLEAN; the original platform reviewer is excluded for violating the required read-only process. |
| Impact posture | Reviewed existing symbols are LOW, including `world_routing_disabled` (two direct/four total), `GatewayLifecycleRequestContext` (one/five), and `synthesized_unavailable_response_without_context` (two/four). GitNexus under-resolves cfg-specific `build_gateway_client`; manual cfg closure is binding. A new owner, process family, request schema, lower adapter edit, network semantic change, credential lifecycle change, or platform-control fallback is `ImpactDecisionRequired`. |
| Frozen exclusions | Config/policy resolver bodies; `install_bootstrap::unix_account_home_for_principal`; gateway request/response and network types; ambient macOS/Windows compatibility clients and all transport clients; world-service handlers; `world-mac-lima::{forwarding,transport}`; Windows/other lower adapters; policy/network meaning; credential carrier/content; gateway service lifecycle; physical shim; replay/global trace; R2-3/R2-4/R3 behavior. |
| Stop conditions | A cannot be validated before disabled/unavailable handling; correct projection requires duplicating the policy resolver; the fixed Linux endpoint would change; macOS would construct a client/enter ambient forwarding; Windows/other contextless entries would select ambient state or claim A-bound success; any HIGH/CRITICAL existing symbol appears; any extra production file/symbol or semantic capability is required; a credential or carrier could be disclosed. Stop before edits or continue only after a new cross-document/impact decision. |
| Historical exit/next | At that checkpoint the focused proof and reviews were CLEAN, Linux proof was recorded, non-Linux remained unproven, PI-111's bounded R2-2E clause was complete, and A1.1d-5R2-2F was the next unstarted packet. The completed-F section below supersedes that next-task status; the full credential/config architecture remains open at E3/D1/D3. |

###### A1.1d-5R2-2F0 — Deterministic world-socket test isolation

| Packet field | Frozen requirement |
|---|---|
| Status | **Complete.** The historical `TestIsolationDefectConfirmed` classification and `1118/150` versus `1119/149` observations remain causal evidence; the exact combined implementation is proof-complete, review-clean, canonically closed out, and preserved. |
| Minimal reproducer | `continue_world_worker_classifies_real_retained_member_turn_streams` plus `b21_retained_production_handoff_claims_before_next_frame`; target failed 12/20 with `--exact --test-threads=2`, failed 0/10 with `--exact --test-threads=1`, and fixed serial order passes. Overlap through process-global `SUBSTRATE_WORLD_SOCKET` is required. The competing mutation first appears at `c519024bd91b6ca6e332d0b8881f7d13ded940e0`. |
| Test-process contract | Acquire the shared world-environment lock; capture exact prior `OsString` or absence; install the test-owned value or absence; retain the lock through socket use, child/server lifetime, cleanup, and restoration; restore on return and panic unwind; release only after restoration. Same-thread nesting must restore in stack order without deadlock. `parking_lot::ReentrantMutex` is non-poisoning; a panicking holder releases normally and later acquisition must be proved. `#[serial]` may remain but is never the only boundary. |
| Complete mutation inventory | Sixty-six shell library-test mutators: 49 orchestrator local-guard call sites, two orchestrator manual blocks, four macOS platform closure-helper call sites, two persistent-session direct call sites, two routing direct call sites, one world-enable path manual block, four already-compliant world-gateway RAII calls, one noncompliant world-gateway classification-test closure call, and one already-compliant async-REPL call. Sixty-one migrate; five remain source-reviewed unchanged. Integration tests only inject child `Command` environments in separate processes and do not require migration. Production reads remain frozen. |
| Exact implementation allowlist | Test-only hunks in `crates/shell/src/execution/mod.rs`, `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/execution/platform/macos.rs`, `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`, `crates/shell/src/execution/routing/world.rs`, `crates/shell/src/builtins/world_enable/runner/paths.rs`, and the classification-test helper/call in `crates/shell/src/builtins/world_gateway.rs`. No production-compiled behavior, integration-test name/assertion, dependency, runtime resolver, readiness logic, retained-worker path, retry validator, capability, policy, or secure-FD path may change. |
| Required focused proof | Exact prior non-Unicode-capable value and absence restoration; panic restoration; non-poisoning recovery; nested acquisition; concurrent blocking and cleanup-before-release; no competitor-socket observation; no deadlock or leaked socket/process/root; exact pair at least 100 parallel and 20 serial iterations; neighboring mutator combinations; deliberate retained-registration loser remains internal while its parent passes. |
| Required broad proof | Combined with F0a/F0b: at least three independent exact final-candidate default-parallel shell broad walls and one canonical `--test-threads=1` wall. No recurrence of socket, HOME, or renderer-output interference; no new or changed inherited failure; no removed/renamed/substituted/weakened/newly ignored test; no unexplained count variance; and identical normalized inherited signatures. Any added passing tests define an explicit count delta and the deterministic post-F0/F0a/F0b F comparison baseline. Shell/workspace all-target checks, warnings-denied Clippy, format, diff, and GitNexus test-only containment are mandatory. |
| Historical stop/next | The production boundary remained intact and the checkpoint closed cleanly. A1.1d-5R2-2F was the next increment there; the completed-F section below supersedes that next-task status. |

###### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

| Packet field | Frozen requirement |
|---|---|
| Status | **Complete.** F0's broad wall established the second historical `TestIsolationDefectConfirmed`; the authorized F0a implementation, proof, review, canonical closeout, and preservation are complete. |
| Exact minimal pair | Target `execution::agent_runtime::tool_invocation_contract::tests::dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`; competitor `execution::agent_runtime::control::tests::prompt_submit_continuity_prefers_persisted_session_contract`. Parallel same-process pair failed 20/20. The stable same-process serial run controlled competitor-then-target and failed 0/10; separate-process sequential runs passed in both directions. Stable libtest could not force reverse same-process order, so that result is not claimed. Three other unannotated HOME mutators independently reproduce 20/20 and remain in the migration inventory. Commit `f5a150f94d585b1f55ec0067845cd5d715773c78` first adds the selected competitor; `83101dcbcc750e6e8fb8979bea19f1f777792188` later adds the target and its mutating fixture and is the first source commit where the exact pair coexists. |
| Shared resource and topology | Process-global `SUBSTRATE_HOME`. Non-source tracing proves the competitor can remove the target's private HOME during overlap. One authority-environment lock jointly protects HOME and `SUBSTRATE_WORLD_SOCKET`; 88 joint users and opposite existing acquisition orders reject separate locks. Exact `OsString`/absence, panic restoration, explicit poison/recovery, safe nesting, bounded child inheritance, and stable-reader participation are mandatory. `#[serial]` is supplemental only. |
| Shell-library mutation closure | 435 mutating test functions across 19 files. Five `with_store` helper families cover 223 dependent tests; two `agents_cmd` helpers cover five more. All direct/local guard/manual sites are included. The four unannotated mutators are the two control continuity tests and the two agents-command toolbox-status tests. |
| Integration disposition | Parent mutation exists only in `crates/shell/tests/shim_deployment.rs`, `crates/shell/tests/agent_successor_contract_ahcsitc0.rs`, and `crates/shell/tests/support/mod.rs`, each a separate test binary with a per-binary disposition. Other integration occurrences are reads or child-only environment injection and require no cross-process lock. |
| F0 async cleanup closure | Nine socket-owning orchestrator tests require explicit `server.abort()` then awaited termination and fixture cleanup before environment restoration/unlock: `dispatch_contract_cancel_world_work_ephemeral_routes_exact_active_task_over_execute_cancel`; `active_ephemeral_terminal_wait_allows_multiple_waiters_to_observe_same_terminal_truth`; `active_ephemeral_terminal_wait_registration_guard_releases_non_happy_path_registrations`; `active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start`; `dispatch_contract_cancel_world_work_ephemeral_retry_reuses_shared_terminal_truth`; `dispatch_contract_cancel_world_work_ephemeral_fails_closed_when_execute_cancel_is_not_delivered`; `continue_world_worker_classifies_real_retained_member_turn_streams`; `continue_world_worker_dispatch_returns_real_typed_internal_outcome`; and `dispatch_contract_fork_world_worker_rolls_back_child_when_lineage_persist_fails`. |
| Exact combined test-only allowlist | Existing F0 seven-file allowlist plus HOME-only shell-library files `crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs,world_enable/runner/manager_env.rs}`, `crates/shell/src/execution/{agent_inventory.rs,agents_cmd.rs,config_model.rs,env_scripts.rs,host_inbox_materialization.rs,invocation/tests.rs,routing/builtin/tests.rs,settings/tests.rs}`, `crates/shell/src/execution/agent_runtime/{auto_attach.rs,control.rs,state_store.rs,tool_invocation_contract.rs,host_session_authority/store_tests.rs}`, and `crates/shell/src/repl/async_repl.rs`; integration dispositions may edit only `crates/shell/tests/{shim_deployment.rs,agent_successor_contract_ahcsitc0.rs,support/mod.rs}`. Every hunk must compile only as test/harness behavior. |
| Impact boundary | `world_env_guard` is CRITICAL at 35 direct/70 total dependents; tool-contract `with_store` is HIGH at 12 direct dependents; other reviewed helpers are MEDIUM/LOW or graph-under-resolved and require source-level dependent audit. No production execution family, resolver, socket owner, retained-registration validator, managed secure-FD path, capability, policy, or user behavior may change. |
| Required focused/combined proof | F0 socket pair at least 100 parallel/20 serial; F0a HOME pair at least 100 parallel/20 serial; combined HOME/socket neighbor matrix; prior absence/value/non-Unicode restoration; panic, poison/recovery, nesting, exclusion, bounded subprocess inheritance, async termination/cleanup ordering, and zero leaked task/socket/helper/temp-root proof. Then three exact final-candidate parallel broad walls and one serial wall with a stable inherited failure-name/signature set and no unexplained count variance. |
| Historical stop/next | No unauthorized production hunk existed and the checkpoint closed cleanly; A1.1d-5R2-2F was next there. The completed-F section below supersedes that next-task status. |

###### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

| Packet field | Frozen requirement |
|---|---|
| Status | **Complete.** The primary historical classification is `TestIsolationDefectConfirmed`. The bounded explicit-writer remediation is implemented, proof-complete, review-clean, canonically closed out, and preserved. |
| Exact blocker | The stdout fallback test's `capture_stdout_once` redirects process fd 1 with `dup2`; libtest's parallel reporter writes through that descriptor. Exact captured bytes: `".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`. The post-fork candidate walls were `1134 passed / 146 failed / 0 ignored`, `1134 passed / 146 failed / 0 ignored`, and `1133 passed / 147 failed / 0 ignored`; only wall 3 added `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`, invalidating closeout. |
| Causal controls | Forced same-process parallel: 376 pass/124 fail across 500, all the exact wall signature. Isolated target: 100/100. Same-process serial with identical neighbors: 100/100. Separate-process control: 100/100. Parallel pretty reporter: 99/100. Candidate introduction is unnecessary; the helper and target are byte-identical to clean E. |
| Exact future file/symbol allowlist | One file: `crates/shell/src/execution/agent_runtime/control.rs`. EDIT `PublicPromptRenderer::render` only for mechanical delegation. ADD private Unix-only explicit-writer core/output adapter symbols. DELETE `capture_stdout_once` and `capture_stderr_once`. EDIT only `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails` and `public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails` to use private in-memory stdout/stderr writers and assert complete exact bytes plus an empty nonselected stream. |
| Frozen production boundary | `PublicPromptRenderer` remains private; `PublicPromptRenderer::new` and `render` signatures remain unchanged; `render` remains the production entry point. Bodies of `run_hidden_owner_helper_startup_prompt_stream_with_projection` and `run_public_prompt_command` are frozen. JSON envelopes, normal completed/event output, warnings/failures/stderr events, stream ordering, newline, flush, propagated versus ignored errors, redaction, and bounded fallback remain byte-for-byte equivalent. Production uses only real stdout/stderr. |
| Same-owner stderr disposition | `capture_stderr_once` performs the same process-global fd replacement and is migrated in F0b despite no observed wall failure. Unrelated raw-fd helpers outside this renderer/test boundary remain deferred evidence and are not authorized. |
| Forbidden remediation | Public writer API or transport/schema; global writer lock/registry/side table; environment-selected output; raw descriptor capture as the final mechanism; filtering/stripping/searching around reporter bytes; sleeps, retries, reporter suppression, thread reduction, whole-suite serialization, ignore, removal, rename, substitution, or assertion weakening. `#[serial]` may remain supplemental only because it cannot exclude libtest reporter writes. |
| Impact and containment | GitNexus: `PublicPromptRenderer::render` MEDIUM, four direct graph dependents, 39 total, one `handle_agent_command` process family, Agent_runtime direct and Execution indirect; source closure resolves exactly two production callers and two tests. `PublicPromptRenderer::new` reports HIGH (19 direct/43 total, two process labels, three modules) but source closure proves generic-`new` over-attribution and its body is frozen. The renderer type, both capture helpers, and both tests are LOW/zero-process. No CRITICAL impact, new production process family, or changed production process semantics are authorized. |
| Required focused proof | Both exact tests isolated; exact private-buffer stdout/stderr assertions; same-process renderer pair at least 100/100 parallel and 20/20 serial; forced reporter-overlap proof; unchanged JSON/completed/warning/failed/stdout-event/stderr-event selection, order, newline, flush, and error semantics. No unrelated reporter byte can enter either private writer. |
| Completed combined closeout | F0/F0a/F0b together passed all authority-environment, async termination, renderer-isolation, shell-module, compile, Clippy, format, and diff gates, followed by three exact default-parallel broad walls and one serial wall with identical discovery/pass/fail/ignored counts, failure names, and normalized signatures. That closeout establishes the deterministic F comparison baseline. |
| Historical stop/next | No public API, global output lock/registry, changed renderer contract/caller, nonallowlisted file/symbol, new production process family, changed production process semantics, or weaker test was introduced. F0b was complete and F was the next packet there; the completed-F section below supersedes that status. |

###### Historical A1.1d-5R2-2F — Authenticated world-deps and truthful doctor composition

This table is the pre-F5-PD F contract. Its World Doctor fixture/child, read-only, and immediate-next
statements were superseded by the later F5-PD packet below. Its world-deps and later F5 composition
scope remains frozen; at that checkpoint F5 could not start until F5-PD was implemented and
re-reviewed. The completed-F record supersedes that status.

| Packet field | Frozen requirement |
|---|---|
| Goal | Close the unresolved PI-106/PI-107 production paths: normal current/global/workspace world-deps, runtime probe/install/sync, provision-deps/post-sync, and Host/World/Health/shim-doctor composition consume one authenticated context and never report coherent A over mixed A/B inputs. |
| Prerequisites | Review-clean Routes A–E and E's canonical explicit config/policy/network projection convention; F0/F0a/F0b jointly implemented, proof-complete, review-clean, canonically closed out, exactly replayed, and preserved; current GitNexus; clean tree. |
| Starting point and comparison baseline | F starts only from the post-F0/F0a/F0b-closeout replayed runtime head. Routes A–E and combined F0/F0a/F0b are immutable prior evidence. F's clean comparison baseline is the deterministic value recorded by combined closeout. The genuine but nondeterministic post-E observations `1114 passed / 149 failed / 0 ignored` and `1113 passed / 150 failed / 0 ignored`, F0-candidate `1118/150` and `1119/149`, and post-fork candidate `1134/146`, `1134/146`, and `1133/147` walls remain historical evidence, not an F gate; none may replace the **R2-2 historical starting baseline**, `1089 passed / 149 failed`, or the **clean Route D comparison baseline**, `1101 passed / 149 failed / 0 ignored`. |
| Shared context | Add one request-scoped `AuthenticatedWorldDepsContextV1`-equivalent in `world_deps/mod.rs`. It validates the existing carrier/A once and holds the trusted bootstrap home/root, non-secret commitment, explicit launch CWD/workspace scope, effective config/policy, explicit global config/deps paths, and authenticated runtime-request projection. No environment binding, process-global side table, duplicate resolver, or reconstruction in leaves. |
| Exact production allowlist | `crates/shell/src/execution/platform/mod.rs::{handle_world_command` Doctor/Deps arms, `handle_host_command` Doctor arm}; `crates/shell/src/execution/platform/linux.rs::{host_doctor_main,world_doctor_main}`; `crates/shell/src/builtins/world_deps/mod.rs::{AuthenticatedWorldDepsContextV1`-equivalent and sole binder, `WorldDepsDoctorSnapshotV1,collect_doctor_snapshot_v1,resolve_effective_enabled_provisioning_requirements_v1}`; the exact `surfaces.rs` symbol list below; `crates/shell/src/builtins/world_enable/runner.rs::{run_enable_with_provision_deps,run_sync_after_provisioning}`; `crates/shell/src/builtins/world_enable/runner/provision_deps.rs::{probe_world_manager,probe_requirements,provision_apt_requirements,provision_pacman_requirements,execute_with_profile}`; `crates/shell/src/execution/routing/dispatch/world_ops.rs` one additive authenticated request builder and its private cfg implementations only; `crates/shell/src/execution/routing/dispatch/prelude.rs` and `crates/shell/src/execution/routing.rs` export-only wiring for that new builder; `crates/shell/src/builtins/shim_doctor/report.rs::{gather_world_doctor_snapshot,gather_world_deps_section,status_for_world_deps_report,snapshot_from_value,snapshot_from_command` plus one shared non-secret identity validator}. No other production file or symbol. |
| Exact `surfaces.rs` list | `run,run_current,run_global,run_workspace,run_current_list,run_current_show,run_current_install,run_current_sync,run_global_list,run_global_add,run_global_remove,run_global_reset,run_workspace_list,run_workspace_add,run_workspace_remove,run_workspace_reset,resolve_global_available_inventory_view,build_current_show_explain_v1,run_current_list_applied,compute_current_applied_items_v1,preflight_runtime_system_requirements_v1,probe_world_apt_requirements_v1,probe_world_pacman_requirements_v1,apply_install_plan_v1,reconcile_world_deps_bin_v1,apply_apt_entrypoint_wrappers_v1,apply_script_package_v1,resolve_script_body_for_package_v1,current_codex_runtime_target_triple_v1,run_world_command_output_for_deps,run_world_command_output_for_deps_with_profile,run_world_command_checked_for_deps,query_world_package_presence,query_world_package_entrypoint_presence,run_world_presence_check_v1,ensure_world_backend_available,run_world_command_for_deps,run_world_command_for_deps_at`. `run_current_list_enabled` and `resolve_current_inventory_view` remain frozen explicit-input/reuse helpers. |
| Exact test allowlist | Colocated tests in the named production modules plus `crates/shell/tests/{common.rs,config_world_deps_phase_b.rs,doctor_scopes_ds0.rs,shim_doctor.rs,shim_health.rs,world_deps_applied_wdp2.rs,world_deps_apt_fail_early_wdap1.rs,world_deps_apt_install_wdp5.rs,world_deps_current_dry_run_wdp3.rs,world_deps_enabled_wdp1.rs,world_deps_inventory_validation_wdp0.rs,world_deps_inventory_views.rs,world_deps_present_semantics_wdh1.rs,world_deps_script_install_wdp4.rs,world_enable.rs,world_enable_provision_deps_wdap0.rs}`. No existing test may be removed, renamed, substituted, ignored, weakened, or made environment-authoritative. |
| Runtime request rule | Only `surfaces.rs::run_world_command_for_deps_at` and `provision_deps.rs::execute_with_profile` consume the additive authenticated builder. It reuses E's explicit projection entrypoint. Existing `build_agent_client_and_request` (HIGH), `build_agent_client_and_request_with_trace_metadata` (CRITICAL), and their ambient platform implementations remain textually/semantically frozen for R2-3; modifying or redirecting their other callers is a stop. |
| Doctor coherence | `WorldDepsDoctorSnapshotV1` carries required non-secret selected-prefix/commitment evidence. Fixture and child decoders require exact A evidence independently of exit code; missing identity, mismatched identity, missing `ok`, or mixed config/policy/inventory/deps/runtime constituents becomes `NeedsAttention`/unavailable or a closed error with `ok=false`. A rejected report is never retained as healthy `Some(report)`. Host/World doctor receive A-derived world-fs policy instead of ambient `detect_profile`/`world_fs_policy`. Health rendering remains frozen because it already propagates constituent errors. World-deps retains its separate F5-owned readiness/execution behavior; the nested World Doctor becomes passive only through F5-PD. |
| Required proof | A under conflicting B for normal current, global, workspace, and runtime Unix/Linux paths; both mutation and nonmutation; provisioning manager probes, apt/pacman install requests, and post-provision sync use A; direct no-witness and malformed/tampered/mismatched input fail before mutation; A/B config/inventory/deps roots; current applied/show probes; Codex-runtime target/install request; fixture and child identity match; missing/mixed identity rejected; truthful unavailable diagnostics; existing A-rooted snapshot behavior; explicit compatibility behavior; no sensitive disclosure. The broad differential is against the deterministic F comparison baseline recorded by combined F0/F0a/F0b closeout and requires zero pass-to-fail, new-fail, changed-failure, removed, renamed, substituted, weakened, or newly ignored tests plus its identical retained failure-name set. Non-Unix cfg proof is build/static preservation plus unavailable/fail-closed or explicit ambient-compatibility labeling, never A-bound/native proof. |
| Impact posture | Most reviewed existing symbols are LOW. `resolve_current_inventory_view` is a frozen HIGH reuse boundary. The shared ambient request builders are HIGH/CRITICAL and frozen. New authenticated context/builder symbols have no pre-edit graph node. Any need to edit a frozen HIGH/CRITICAL symbol, change another shared caller, alter network semantics/schema/service/platform lifecycle, or add an unreviewed file/symbol stops as `ImpactDecisionRequired`. |
| Frozen exclusions | Existing config/policy explicit-home resolver bodies; `resolve_current_inventory_view`; inventory parser/models; pure install-plan/command-render/output helpers; ambient request builders and implementations; request/transport schemas; world-service; Health summary/output; macOS/Windows/fallback doctor and world-deps adapters; all platform-native adapters; physical shim; replay/global trace; filesystem/network/caging/placement/capability/service/receipt/supervisor/retained-worker/lifecycle behavior. Non-Linux ambient compatibility is explicitly R2-3-owned/unproven and cannot satisfy F. R2-3/R2-4/R3 ownership is unchanged. |
| Exit/next | Focused proof, full affected world-deps/doctor suite, warnings-denied touched-target Clippy, shell/workspace all-target checks, format/diff checks, GitNexus detection, and fresh security/call-path/doctor-coherence reviews are CLEAN. Then run a renewed production-fix-free Routes A–F R2-2 integration closeout. R2-3 remains after that closeout. |

R2-2E, R2-2F0/F0a/F0b, and R2-2F are logically separable, but the selected order is binding: E has
established the canonical explicit config/policy/network projection; F0/F0a/F0b/F0-HC are
canonically complete. F was the exact next unstarted packet at that checkpoint; the controlling
sequence now inserts F5-PD before F5. F retains its later use of E's projection for
authenticated runtime requests and then proving doctor constituent coherence. The
renewed R2-2 closeout reruns the complete Routes A–F product wall and may not repair production code.
Routes A–E are preserved evidence, not reopened implementation. R2-3, R2-4, R3, `RG-HOME-01`, and
`RG-INSTALL-01` retain their existing ownership and open status; no seam is promoted.

###### A1.1d-5R2-3 — Platform-native mapping adapters

| Packet field | Frozen requirement |
|---|---|
| Goal | Implement and verify `PlatformBootstrapMappingV1` for macOS host-to-Lima and Windows host-to-WSL, including two-stage Lima realization/mapping, the fixed future V1 SSH-UDS target plus an R3 activation prerequisite before any macOS forwarding, one mapped pipe/forwarder scope, direct platform-helper construction, every backend-factory caller (shell, shim telemetry, replay), physical-shim trace/policy projections, Windows `-NoAutoSource`, and byte-identical WSL capability guards, without host/guest path or principal equality. |
| Prerequisites | Final R2-2; stable encoded host commitment and sudo/child contract; native-proof assignments booked even if unavailable during implementation. |
| Must read | R2-1/R2-2 closeouts; `02-seam-crosswalk.md` PI-009, PI-022, PI-039–PI-060, PI-068–PI-070, PI-075–PI-081, PI-090–PI-102, PI-112, PI-115–PI-116, PI-118, and frozen PI-113–PI-114; `04-contracts-and-gates.md` two-stage platform instance/transport identity, mapping/factory verification, physical-shim manifest/trace, WSL child, WSL guard, and shared-state scope; R2-MAP-MAC-01/R2-MAP-WIN-01/R2-GEN-01/R2-DIAG-01. |
| Sibling context | Host A remains authoritative; adapters realize platform-native paths/principals. R3 alone removes Lima/WSL/forwarder/shim/socket state. |
| Exact production-file allowlist | `crates/transport-api-types/src/lib.rs` mapping/framing section; `crates/shell/Cargo.toml` only for the frozen `windows-sys` feature additions named below; `crates/shell/src/execution/install_bootstrap.rs` mapping construction/validation only; `scripts/mac/{lima-warm.sh,lima-doctor.sh}`; `scripts/mac/lima/units/{substrate-world-service.service.tmpl,substrate-world-service.socket}`; `scripts/substrate/{dev-install-substrate.sh,install-substrate.sh}` macOS call sections only; `crates/world-mac-lima/src/{lib.rs,forwarding.rs,transport.rs,limactl.rs,vm.rs}`; `crates/world-backend-factory/{Cargo.toml,src/lib.rs}`; `crates/forwarder/Cargo.toml`; `crates/shim/Cargo.toml`; `Cargo.lock` only for the exact package dependency-list additions frozen below (the three internal `transport-api-types` edges plus existing `windows-sys 0.52.0` in the shim list; no package/version/checksum change); `crates/shell/src/execution/{invocation/plan.rs,platform/macos.rs,platform/windows.rs,platform_world/mod.rs,platform_world/windows.rs,routing/replay.rs}` exact platform/context-call sections; `crates/shell/src/builtins/world_gateway.rs` platform transport selector only; `crates/shim/src/{context.rs,logger.rs,exec/mod.rs,exec/logging.rs,exec/policy.rs}` only for invocation/factory projection, PI-116 manager-manifest intake, and PI-118 trace binding/reuse; `crates/replay/src/{lib.rs,replay/mod.rs,replay/planner.rs,replay/executor.rs}` factory-context parameter plumbing only; `crates/trace/src/{context.rs,util.rs}` only for PI-118 compatibility-posture removal/final unbound-init rule after owned shim/replay/platform callers migrate; `scripts/windows/{dev-install-substrate.ps1,dev-uninstall-substrate.ps1,install-substrate.ps1,uninstall-substrate.ps1,start-forwarder.ps1,pipe-status.ps1,wsl-warm.ps1,wsl-doctor.ps1}`; `crates/world-windows-wsl/src/{backend.rs,lib.rs,paths.rs,transport.rs,warm.rs}`; `crates/forwarder/src/{bridge.rs,config.rs,logging.rs,windows.rs,main.rs,pipe.rs,wsl.rs}`. Lima base profiles, `scripts/mac/lima-stop.sh`, `scripts/windows/wsl-stop.ps1`, and `scripts/wsl/provision.sh` are explicitly excluded for world/R3/capability ownership. No other production file. |
| Exact production sections/symbols | Mapping wire model/framing, including `host_platform_control_root` and `WindowsForwarderScopeV1`; Windows release-copy installed-product witness and current account/SID binding in `install_bootstrap.rs`; the existing target-Windows `windows-sys = "0.52"` dependency in `crates/shell/Cargo.toml` may add exactly `Win32_Security`, `Win32_Storage_FileSystem`, `Win32_System_Threading`, `Win32_System_Com`, and `Win32_UI_Shell` to its existing `Win32_Foundation` and `Win32_System_Console` features for token/SID and file identity plus `SHGetKnownFolderPath(FOLDERID_LocalAppData)`/`CoTaskMemFree`. No dependency version, new dependency, target scope, or other feature may change; this feature-only edit must not change the package graph, and any resulting `Cargo.lock` change is an unexpected stop-and-review condition rather than shell-manifest authority. Context/mapping intake; host commitment/instance/transport/control-root verification; platform account/home resolution; unit/env/socket/pipe/forwarder projection; doctor/pipe-status projection; backend factory/constructor parameters; gateway/platform-world call sites; shim unique invocation-witness construction, token-bound Known Folder projection, `ManagerHintEngine::new`/`manifest_paths` typed-IH intake and exact A-base/A-overlay selection with ambient/repo fallback removed from normal product mode, `collect_world_telemetry` explicit factory projection, and pre-manager/policy construction of the PI-117 explicit product posture from exact `A/trace.jsonl` plus policy Git directory A; `evaluate_policy` and `write_log_entry` may only reuse that already-bound trace and cannot initialize from ambient state; already-frozen replay/platform callers migrate to explicit binding; only after those owned migrations may `LegacyAmbientCompatibility` be removed/made unreachable and the final global unbound-`init_trace(None)` rule land in `trace/context.rs`, with legacy policy lookup migration/removal in `trace/util.rs`. `set_global_trace_context` remains neutral and no caller-identity table is allowed. Windows dev/release projection of the already-selected manager manifest to `A/manager_hooks.yaml` after IH construction; replay public bootstrap config and shell route through planner/executor factory argument only; shim invocation; `Transport::auto_select`, `forwarding::auto_select`, and `MacLimaBackend::{new,ensure_forwarding,get_agent_endpoint}` only to remove auto-selection from the validated normal path, consume the PM-fixed future SSH-UDS identity, and fail before forwarder launch with an explicit R3 lifecycle prerequisite; `lima_home_dir`/`lima_ssh_config_path` become typed-control-root consumers and every `limactl`/SSH-config child receives scrubbed/overwritten `HOME` and `LIMA_HOME`; `create_ssh_uds_forwarding` may receive future mapping/known-hosts parameters but is not called by the R2 validated path; `start-forwarder.ps1` IH/PM child carrier, current-token validation, exact A-scoped `--config`/`--log-dir`, and typed shared PID-root projection; `Cli::resolve_log_dir` and `ForwarderConfig::{load,default_config_path}` require the explicit PM/IH-derived paths in internal mode and may not use `LOCALAPPDATA`; `ForwarderConfig` carrier/PM/distro/pipe/target validation; `spawn_bridge` and `wsl::spawn` exact typed distro/target/commitment projection with overwritten child environment/`WSLENV`; and `-NoAutoSource` carrier. The PI-117 additive representation and all trace writer/rotation/retention/span/policy semantics are frozen in R2-3 except the exact PI-118 compatibility closure just named. Lima VSock, SSH UDS, and SSH-TCP constructors remain direct diagnostic/test-only compatibility paths in R2 and cannot satisfy PM/product proof. R3 alone may activate PM-bound SSH UDS after exact PI-101/PI-113/PI-114 lifecycle semantics land; R2 may not launch a forwarder, create/unlink a socket, set `StreamLocalBindUnlink`, kill/wait a forwarding child, or exercise handle-drop cleanup as proof. Existing explicit Windows TCP target mode likewise may remain only as labeled diagnostic input after PM validation and cannot satisfy normal mapping proof; target/config/environment cannot select normal product mapping. Lima helper construction follows PI-081: IH plus instance selector, derived committed control root, and fixed transport target before Stage 1, PM only after a running instance is observable. Stage 1 permits only status, fixed-profile render, absent-instance create or same-instance start, and wait; Stage 2 must finalize PM before R2-owned guest projection. `destroy_vm`, mismatch delete/rebuild, staged-tree/temp/unit/socket cleanup, explicit/SSH-side forwarding unlink, `ForwardingHandle::drop`, and SSH timeout kill are byte-frozen R3 sections. `start-forwarder.ps1` timeout kill is byte-frozen. Forwarder stream/wait behavior and replay command/state/origin/policy/timeout/strategy semantics are byte-frozen. In uninstallers, only context intake, non-mutating classification, current-token binding, and carrier arguments on an existing shim-remove invocation are editable. In `wsl-warm.ps1`, only parameter/context/control-root validation before the guard is editable; the guard and unreachable code are byte-frozen. `scripts/wsl/provision.sh` remains byte-for-byte fail-closed. No WSL capability, base Lima profile, or transport-protocol redesign. |
| Allowed tests | Colocated tests in each allowed Rust file, including `crates/trace/src/tests.rs` only for PI-118 compatibility closure; `crates/world-windows-wsl/src/tests.rs`; `crates/world-mac-lima/examples/mac_backend_smoke.rs` call-site update only; `crates/shim/tests/integration.rs`; `crates/replay/tests/{integration.rs,planner_executor.rs}`; `crates/shell/tests/replay_world.rs` factory-context cases only; `tests/mac/lima_doctor_fixture.sh`; mapping/context-only cases in `tests/mac/installer_parity_fixture.sh` with lifecycle expectations frozen; new `tests/mac/prefix_mapping_r2_3.sh`; new `scripts/windows/prefix-mapping-r2-3.Tests.ps1`. Neither `scripts/mac/smoke.sh` nor `scripts/windows/wsl-smoke.ps1` is an R2-3 runner: both execute R3 lifecycle actions, and Windows smoke also reaches the frozen warm guard. Native R2-3 uses only the focused non-mutating existing-instance mapping tests. No tests in excluded stop/provision/base-profile files. |
| Explicit non-goals | Host/guest equality; backend home selection; cross-platform common-home side table; WSL/Lima lifecycle deletion; wildcard removal; forwarder ownership manifest; activating macOS forwarding before R3; deleting VSock/TCP support or admitting either into V1 normal-product proof; new transport protocol; policy/world capability semantics; claiming static review as native proof. |
| Exit gate | Verified host commitment binds one named platform instance, OS-resolved host platform-control root, native account/home/principal, and the future normalized SSH-UDS transport target. Lima control root derives only from the committed host principal's account-database home; public parents scrub and overwrite child `HOME`/`LIMA_HOME`, internal conflicts reject, and no Lima command reads ambient selection. Lima two-stage order is proven without requiring PM before Stage 1 or executing delete/rebuild/staging/forwarding cleanup; mapping mismatch fails closed; the validated Lima path never auto-selects VSock/TCP and stops before forwarder launch with the explicit R3 lifecycle prerequisite; no socket/process lifecycle action becomes newly reachable. The future mac host socket and SSH known-hosts projection are A-scoped; unique shim invocation-witness recovery works without PATH precedence and fails on zero/multiple candidates; physical shim manager hints read only A-base/A-overlay under ambient B and never repo fallback; physical-shim spans/execution logs write only `A/trace.jsonl`, policy-commit lookup reads only A, and neither repeated init nor missing Git metadata touches B; Windows release-copy standalone self-derivation selects A under ambient B and rejects a forged account/SID carrier; token Known Folder plus exact SID/instance/pipe scope fixes the shared PID root, while config/logs are exactly A-scoped and every internal forwarder path is explicit; `LOCALAPPDATA`/`USERPROFILE` cannot select. Shell/shim/replay factory calls are explicit and no contextless platform factory remains; Windows `-NoAutoSource` propagates context; pipe producer/consumer/doctor/backend share one selector; forwarder validates PM/current token and its WSL child receives the exact PM distro/target/commitment despite conflicting ambient target/`WSLENV`; shared Windows artifacts are classified without deletion authority; WSL guards remain byte-identical; all static/focused tests clean. |
| Regression gates | R2-1/R2-2 gates; R2-MAP-MAC-01/R2-MAP-WIN-01/R2-GEN-01/R2-DIAG-01; physical-shim A-versus-B manager-hint loading, no-repo-fallback, trace-output, policy-Git-source, missing-metadata, and unbound-init fail-closed cases, including zero filesystem access under B; Lima tests prove account-database control-root derivation, scrubbed/overwritten child `HOME`/`LIMA_HOME`, rejection of a directly injected conflicting internal projection, A-scoped future SSH UDS despite `vsock-proxy`/TCP availability, the R3 prerequisite before command spawn/socket mutation, and diagnostic transport output excluded from PM/product proof; Windows tests prove token Known Folder and canonical scope hash select the PID root, exact A selects config/logs, conflicting `LOCALAPPDATA`/`USERPROFILE` cannot retarget, and PM-derived WSL argv/environment wins over ambient target/`WSLENV`; existing mapping/backend/forwarder tests do not exercise macOS forwarding lifecycle; native evidence records exact OS/backend/tool versions, A/B setup, commitment, control root, mapping output, and the unactivated target. Static-only results remain explicitly pending. |
| GitNexus posture | HIGH/CRITICAL is expected for `managed_host_socket_path`, platform backend constructors, and shared transport state. Warn before editing; unrelated execution-flow impact, side-table selection, or a new backend authority stops the packet. |
| Independent reviews | Host-context/mapping security; cross-platform lifecycle R2/R3 ownership; allowlist, native-proof honesty, and regression sufficiency. All CLEAN. |
| Platform evidence | Static review on any host; separately assigned native supported macOS+pre-existing-Lima no-forwarder mapping-only run and native supported Windows+pre-existing-WSL mapping-only run. The macOS record may read the existing instance and must show the future A-scoped Unix socket, `/run/substrate.sock`, ambient transport availability non-authority, and the explicit R3 prerequisite; it must not start a forwarder, create/unlink a socket, kill/wait a child, or exercise handle drop. Full warm/smoke/lifecycle and macOS product transport are not R2-3 evidence. A missing native runner leaves the native assignment pending and never converts static evidence into native evidence. |
| Stop conditions | Platform instance/transport identity cannot be observed coherently; guest home/principal must be guessed; shared/per-prefix classification would require deletion semantics; a host path/UID is imposed in the guest; either WSL fail-closed guard would move or unreachable provisioning would activate; native evidence is mislabeled; allowlist expands. Architecture ambiguity/capability activation is `ArchitecturalBoundaryDecisionRequired`. |
| Next packet | A1.1d-5R2-4 — R2 integration and closeout. |

The R2-3 dependency surface is also closed. `crates/world-backend-factory/Cargo.toml`,
`crates/forwarder/Cargo.toml`, and `crates/shim/Cargo.toml` may each add exactly
`transport-api-types = { version = "0.2.8", path = "../transport-api-types" }`, respectively so
the factory accepts the shared typed mapping, the forwarder authenticates shared IH/PM carriers,
and the physical shim receives shared IH without a local duplicate. `Cargo.lock` may add only
`transport-api-types` to the existing dependency lists for `world-backend-factory` and
`substrate-forwarder`, and only `transport-api-types` plus `windows-sys 0.52.0` to
`substrate-shim`; no package/version/checksum or other package dependency list may change. The
shell `windows-sys` feature-only edit does not authorize a lock hunk. Any other dependency,
feature, target section, version, or lock change stops the packet.

The physical shim is the only additional OS-observation owner: within `context.rs`, the exact
invocation-witness constructor also resolves the current Unix account+UID or Windows account+SID
and performs the same no-follow Windows file-identity checks before constructing IH through the
shared type. `crates/shim/Cargo.toml` may add only `user` to its existing Unix
`nix = { version = "0.29", features = ["fs", "process"] }` feature list, and may add only
`windows-sys = { version = "0.52", features = ["Win32_Foundation", "Win32_Security",
"Win32_Storage_FileSystem", "Win32_System_Threading", "Win32_System_Com",
"Win32_UI_Shell"] }` under target Windows. The last two features are limited to token-bound
`FOLDERID_LocalAppData` resolution and freeing its returned buffer. The feature-only
`nix` change creates no lock hunk; the `windows-sys 0.52.0` shim lock entry above is its sole lock
effect. Native Unix tests cover canonical account/UID and
forged-principal rejection; static Windows compilation plus assigned native Windows tests cover
account/SID, no-follow file identity, and forged-principal rejection. No raw FFI, shim-local
principal type, ambient account variable, or other feature/dependency is authorized.

###### A1.1d-5R2-3 bounded docs-first subdivision

R2-3 is too broad for one implementation/review subject. The binding order is:

```text
R2-3D
  -> R2-3A -> R2-3B -> R2-3C
  -> R2-3M1 -> R2-3M2 -> R2-3M3 -> R2-3M4
  -> R2-3W1 -> R2-3W2 -> R2-3W3 -> R2-3W4 -> R2-3W5
  -> R2-3F -> R2-3R -> R2-3S1 -> R2-3S2 -> R2-3T
  -> R2-3ZD1 -> R2-3ZT1 -> R2-3ZH1 -> R2-3ZM5
  -> fresh native macOS/Windows evidence -> R2-3Z
```

This order is sequential. A later increment may be re-subdivided before implementation if its
pre-edit impact or source-closure evidence exceeds the frozen risk ceiling; it may not absorb an
earlier incomplete outcome. `R2-3D` alone was authorized by the initiating docs-first session. It
completed only the subdivision below and did not authorize or complete `R2-3A` by itself.

At that checkpoint, the published tip included four frozen prerequisites after the historical R2-2 publication:
`ebaf8941` keeps installer configuration private, `a009959b` stages gateway-smoke manifests as
trusted files, `343844c9` preserves install context in agent doctor, and `43e528af` calibrates the
bounded-review process. They remain prior work, not R2-3 implementation or proof. Reopening any of
them requires a concrete contradiction against current repository truth.

The parent R2-3 row above remains controlling. In the child records below, **owns** means the named
increment is the primary implementation/proof owner for that PI row; **supports** means the
increment supplies a prerequisite but may not claim that PI complete. PI-059 is a harness-only
call-site update; PI-077 and PI-078 are frozen capability guards; PI-050 remains an R2-4 guardrail;
PI-080 is already satisfied by R2-2. PI-047, PI-051, PI-053, PI-092–PI-102, and PI-113–PI-114
remain R3-exclusive. No child may claim the parent packet complete before R2-3Z.

`R2-3ZP3` is permanently deferred from the blocking path. Its unpublished commits
`2cb796ffef68c2b049376984a90ce0382e5f3980`, `9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`,
`50948dbeb921582515a34bb6b7be21c46f29d008`, and
`868994efaf132bb04c6cdd7c82da9433333c94e5` are preserved only as diagnostic proof-infrastructure
evidence, are not accepted product source, and must not be cherry-picked, pushed, or represented
as landed. `R2-3ZD1` was the closeout-policy amendment only and completed no product proof by
itself.

Every child uses this same fingerprint/review method:

1. freeze the pre-edit commit and exact child file/test allowlist;
2. before its discovery fingerprint, run format or parser checks, focused tests, `git diff --check`,
   and an allowlist/status check;
3. hash a sorted manifest containing the pre-edit commit plus, for every exact subject path, its
   repository-relative path, Git mode (or `NEW`), and `git hash-object --no-filters` blob ID (or
   `MISSING`); SHA-256 of that manifest is the `subject_fingerprint`;
4. open a new V1 review-cycle record with packet ID equal to the child ID and validate it with
   `review-control/validate_review_cycle.py` after every returned cycle;
5. permit one complete-subject discovery burst, one consolidated P1/P2 remediation, one
   different-fresh closure, and at most two immediately causal supplemental cycles; demonstrated
   P1/P2 block, valid unfixed P3/P4 are added to or deduplicated in `06`, and CLEAN is terminal; and
6. run `gitnexus_detect_changes()` before any child commit. An unexpected production symbol,
   execution flow, file, dependency, or generated artifact stops the child.

Before editing any named existing symbol, run upstream GitNexus impact with tests included for that
exact symbol/file disambiguation and record direct callers, affected processes/modules, and risk.
New symbols record `N/A (new)` plus impact on the existing constructor/caller they replace or feed.
Shell/PowerShell functions that GitNexus does not index require the same pre-edit source-caller
closure by exact name. HIGH or CRITICAL is a warning-and-confirm checkpoint; an unreviewed
authority root, side table, lifecycle edge, or unrelated flow is a stop. `managed_host_socket_path`,
platform backend constructors, factory migration, shared pipe state, and forwarding boundaries are
presumptively HIGH/CRITICAL even if an index under-reports cfg-specific callers.

Across every child, the following remain frozen: host/guest path or principal equality; ambient
home/control-root/pipe selection; backend-selected home; WSL or Lima capability enablement; base
Lima profiles; VSock/TCP admission into V1 normal-product proof; policy/world/replay semantics not
named by the parent row; and all R3 deletion, replacement, rollback, teardown, timeout-kill,
wildcard, recursive, manifest, retry, and convergence authority. The exact WSL warm guard and
`scripts/wsl/provision.sh` exit-4 body remain byte-identical. Mac forwarding activation stops before
child launch until R3 lands PI-101/PI-113/PI-114 together. A frozen-byte change, allowlist
expansion, false native claim, or need to guess guest identity is
`ArchitecturalBoundaryDecisionRequired`.

**R2-3D — docs-only subdivision.**

- **Outcome/claim:** make the published current status truthful and freeze this exact child
  sequence. It completes planning only.
- **PI ownership:** owns no PI and completes no product gate.
- **Files/symbols/tests:** only `00-README.md` current-status text and this R2-3 subdivision in
  `03-phase-slice-map.md`; no production symbol, test, manifest, lockfile, installer, or script.
- **Dependencies/evidence:** requires published R2-2 plus the four frozen post-publication fixes.
  Markdown/diff/allowlist checks and GitNexus change detection are the complete D gate; no R2
  product proof gate is claimable and native evidence is not applicable.
- **Impact/review/stop:** no symbol impact call is required. Fingerprint exactly the two docs and
  use review record packet ID `A1.1d-5R2-3D`. Any product/test byte, PI completion claim, edit to
  `02`/`04`/`05`/`06` without a concrete contradiction, or authorization of R2-3A stops D.

**R2-3A — shared PM wire model.**

- **Outcome/claim:** add only the canonical, strict mapping/instance/transport/scope value model;
  no runtime consumer or platform mutation. This is the narrowest correct first product increment.
- **PI ownership:** supports PI-039–PI-041, PI-048–PI-049, PI-052, PI-054–PI-060, PI-075–PI-076,
  PI-079, PI-081, PI-090–PI-091, PI-112, and PI-115; completes none of those consumer rows.
- **Files:** only `crates/transport-api-types/src/lib.rs`.
- **Existing/new symbols:** existing IH types, path normalizers, and their behavior remain
  unchanged. Add exactly `PlatformInstanceIdentityV1`, `PlatformTransportIdentityV1`,
  `PlatformBootstrapMappingV1`, `WindowsForwarderScopeV1`, mapping member operations
  `new_lima`, `new_wsl`, `validate`, `encode`, and `decode`, scope member `derive`, and
  `normalize_windows_pipe_path`. No other public symbol.
- **Tests/proof:** colocated tests only:
  `platform_bootstrap_mapping_lima_golden_vector_is_exact`,
  `platform_bootstrap_mapping_wsl_golden_vector_is_exact`,
  `platform_bootstrap_mapping_rejects_noncanonical_or_tampered_records`, and
  `windows_forwarder_scope_and_pipe_normalization_are_canonical`; run the
  `transport-api-types` crate tests. These are the shared type prerequisites for
  R2-MAP-MAC-01/R2-MAP-WIN-01 and prove exact thirteen-line framing, outer unpadded base64url,
  commitment equality, canonical re-encoding, guest/host non-equality, pipe normalization, digest,
  and unknown/duplicate/reordered/tampered rejection.
- **Dependencies/native:** depends only on D and the existing IH model. Static wire proof only;
  native macOS/Windows mapping remains assigned to Z.
- **Impact/review/stop:** impact `normalize_windows_install_bootstrap_path`, `encode_inner_field`,
  `decode_inner_field`, `record_value`, `require_record_value`, and `is_lower_hex_digest` before
  reuse edits; new symbols are N/A and require context on `InstallBootstrapContextCarrierV1`.
  Packet ID `A1.1d-5R2-3A`. Any consumer/dependency/platform edit, alternate home selector, mapping
  side table, or lifecycle authority stops A.

**R2-3B — exact dependency edges.**

- **Outcome/claim:** make only the closed dependency/feature edges needed by later typed consumers;
  no runtime behavior.
- **PI ownership:** supports PI-054, PI-060, PI-090, PI-115, and PI-116; completes no PI.
- **Files/symbols:** only `crates/world-backend-factory/Cargo.toml`,
  `crates/forwarder/Cargo.toml`, `crates/shim/Cargo.toml`, and `Cargo.lock`; no Rust symbol.
  Add only the three exact `transport-api-types = { version = "0.2.8", path =
  "../transport-api-types" }` edges, shim `nix` feature `user`, shim target-Windows
  `windows-sys 0.52` features named by the parent, and the exact three lock dependency-list changes.
- **Tests/proof:** no test file and no product gate. Run the parent dependency-closure gate with
  locked metadata/build resolution and prove no package/version/checksum/target/other feature or
  dependency-list change.
- **Dependencies/native:** depends on A; native evidence is not applicable.
- **Impact/review/stop:** manifest-only, so no symbol impact call; inspect consumers
  `factory`, `PipeListener::new`, `spawn_bridge`, `ShimContext::from_current_exe`, and
  `ManagerHintEngine::new` before asserting need. Packet ID `A1.1d-5R2-3B`. Any shell manifest or
  unexpected lock hunk stops B.

**R2-3C — shell Windows principal and installed-witness observation.**

- **Outcome/claim:** provide the target-Windows OS observation needed to bind the current
  account+SID, token Known Folder, and installed release-copy witness; no installer or world
  consumer.
- **PI ownership:** supports PI-042–PI-046, PI-048–PI-049, PI-052, PI-068–PI-070, PI-076,
  PI-079, PI-081, and PI-090; completes none.
- **Files:** only `crates/shell/Cargo.toml` and
  `crates/shell/src/execution/install_bootstrap.rs`.
- **Existing/new symbols:** preserve all Unix functions. Add exact Windows counterparts
  `construct_windows_install_bootstrap_context`,
  `decode_and_bind_windows_install_bootstrap_context`,
  `bind_windows_install_bootstrap_context`,
  `current_windows_principal_and_known_folder`,
  `windows_known_folder_for_principal`,
  `resolve_windows_install_prefix_from_invocation`, and
  `require_same_windows_file_identity`; no raw FFI and no ambient account/folder variable.
  The shell manifest may add only the parent-row `windows-sys` features; no lock hunk.
- **Tests/proof:** the Windows-observation prerequisites of R2-MAP-WIN-01/R2-DIAG-01 use
  colocated target-Windows tests for canonical account/SID and token Known Folder, release-copy
  self-derivation under ambient B, no-follow exact file identity, forged principal, zero/multiple
  witness, and conflicting projection rejection; static Windows compilation where available.
  Native Windows use remains assigned to Z.
- **Dependencies/impact/review:** depends on A. Impact every existing Unix analogue before any
  shared-helper edit, especially `construct_unix_install_bootstrap_context`,
  `decode_and_bind_unix_install_bootstrap_context`,
  `bind_unix_install_bootstrap_context`, `current_unix_principal_and_home`,
  `resolve_unix_install_prefix_from_invocation`, and `require_same_file_identity`.
  Packet ID `A1.1d-5R2-3C`. A Unix behavior change, environment-selected principal/Known Folder,
  routing-file edit, new dependency/version, or lock hunk stops C.

**R2-3M1 — macOS helper mapping and two-stage observation.**

- **Outcome/claim:** make direct macOS helpers consume IH, derive the account-database Lima control
  root, perform only declared-instance Stage 1, and finalize/diagnose PM from an already-running
  guest before R2 projection.
- **PI ownership:** owns PI-039 and the macOS half of PI-081; supports PI-040–PI-041 and PI-075.
- **Files:** only `scripts/mac/lima-warm.sh` and `scripts/mac/lima-doctor.sh`.
- **Existing/new symbols:** edit `check_only_status`, `render_profile`, `vm_exists`, `vm_status`,
  `create_vm`, `start_vm`, `wait_for_running`, `ensure_vm_ready`, `configure_guest`, `diagnose`,
  `check_rendered_unit_parity`, and `run_breakglass_guest_checks`; add shell functions
  `resolve_install_bootstrap_context_v1`, `resolve_lima_control_root_v1`,
  `run_limactl_with_mapping_env_v1`, `observe_lima_mapping_v1`, and
  `verify_lima_mapping_v1`. `destroy_vm`, `stage_workspace`, cleanup bodies in
  `write_systemd_units`/`enable_socket_activation`, and all forwarding actions are frozen.
- **Tests/proof:** R2-MAP-MAC-01/R2-DIAG-01 use
  `tests/mac/lima_doctor_fixture.sh` plus new
  `tests/mac/prefix_mapping_r2_3.sh`; prove public B scrubbing, internal projection mismatch
  rejection, exact child `HOME`/`LIMA_HOME`, machine ID/account/UID/home observation, Stage order,
  and no delete/stage/unit/socket/forwarder action.
- **Dependencies/native:** follows A–C in the binding order but consumes only A and the existing IH
  conventions, not Windows-only C behavior. Static/focused tests only; Z owns the native
  pre-existing-Lima mapping-only record.
- **Impact/review/stop:** run GitNexus/file-qualified impact where indexed for every edited function
  and exact shell caller closure otherwise; `ensure_vm_ready` and `configure_guest` are
  HIGH-posture. Packet ID `A1.1d-5R2-3M1`. Need for guessed guest home, delete/rebuild, base-profile
  change, unit/socket cleanup, or forwarder launch stops M1.

**R2-3M2 — macOS typed backend primitives.**

- **Outcome/claim:** make Lima command, VM, socket, and backend construction consume typed
  control-root/mapping inputs without selecting ambient state; no forwarding activation.
- **PI ownership:** supports PI-041, PI-054–PI-056, PI-075, and PI-090–PI-091; completes no
  cross-caller PI.
- **Files:** only `crates/world-mac-lima/src/lib.rs`,
  `crates/world-mac-lima/src/transport.rs`, `crates/world-mac-lima/src/limactl.rs`, and
  `crates/world-mac-lima/src/vm.rs`.
- **Existing/new symbols:** edit `MacLimaBackend::new`, `MacLimaBackend::new_with_vm_name`,
  `ensure_vm_running`, `get_agent_endpoint`, `managed_host_socket_path`,
  `managed_host_socket_path_from`, `Transport::auto_select`, `limactl::path`,
  `limactl::command`, `LimaVM::new`, `LimaVM::status`, `LimaVM::ensure_running`,
  `LimaVM::start`, `LimaVM::wait_for_running`, `LimaVM::exec`, `LimaVM::info`, and
  `run_limactl`; add `MacLimaBackend::new_with_mapping`,
  `managed_host_socket_path_for_mapping`, and `limactl::command_for_control_root`.
  Existing diagnostic constructors may remain only explicitly labeled non-product.
- **Tests/proof:** the backend-primitive portion of R2-MAP-MAC-01 uses colocated tests only; prove
  A-scoped future socket, typed control root, scrubbed child environment, mapping mismatch
  rejection, and diagnostic transport non-authority without starting forwarding.
- **Dependencies/native:** depends on M1 and B. Static Rust proof only; Z owns native evidence.
- **Impact/review/stop:** impact every named existing Rust symbol; treat
  `managed_host_socket_path` and both backend constructors as CRITICAL regardless of cfg graph
  under-reporting. Packet ID `A1.1d-5R2-3M2`. An ambient fallback, backend-selected home,
  contextless normal constructor, or forwarding/lifecycle reachability stops M2.

**R2-3M3 — macOS forwarding boundary without activation.**

- **Outcome/claim:** fix the future PM-bound SSH-UDS target and A-scoped known-hosts projection,
  remove auto-selection from the validated path, and fail before forwarding child launch with the
  explicit R3 prerequisite.
- **PI ownership:** owns PI-041 and PI-112; PI-101/PI-113/PI-114 remain frozen R3 rows.
- **Files:** only `crates/world-mac-lima/src/forwarding.rs`,
  `crates/world-mac-lima/src/transport.rs`, and `crates/world-mac-lima/src/lib.rs`.
- **Existing/new symbols:** edit `forwarding::auto_select`, `create_ssh_uds_forwarding`,
  `lima_home_dir`, `lima_ssh_config_path`, `Transport::auto_select`,
  `MacLimaBackend::ensure_forwarding`, and `MacLimaBackend::get_agent_endpoint`; add
  `lima_home_dir_for_mapping`, `lima_ssh_config_path_for_mapping`, and
  `r3_forwarding_activation_required`. `ForwardingHandle::drop`, explicit/SSH-side unlink,
  `StreamLocalBindUnlink`, timeout kill/wait, VSock/TCP constructors, and retry/teardown bodies are
  byte-frozen.
- **Tests/proof:** R2-MAP-MAC-01/R2-GEN-01 use colocated tests only; prove fixed
  `A/sock/agent.sock` to `/run/substrate.sock`, exact `A/lima_known_hosts`, ambient
  VSock/TCP/SSH availability non-authority, and failure before command spawn or socket mutation.
  No current forwarding test may count as product proof.
- **Dependencies/native:** depends on M2. Static no-forwarder proof only; Z owns native mapping and
  macOS product transport remains pending R3.
- **Impact/review/stop:** impact every named function and `ForwardingHandle::drop` for frozen
  reachability; HIGH/CRITICAL is expected. Packet ID `A1.1d-5R2-3M3`. Any call to the current SSH
  constructor from the validated path or any lifecycle-byte change stops M3.

**R2-3M4 — macOS installer, unit, shell, and proof projection.**

- **Outcome/claim:** carry the already-selected IH/PM through dev/release macOS call sections,
  generated guest unit/socket projections, and host doctor/invocation surfaces without activating
  forwarding.
- **PI ownership:** owns PI-009, PI-022, and PI-040; supports PI-055 and PI-075, which F completes;
  PI-059 is harness-only.
- **Files:** only macOS call sections in
  `scripts/substrate/dev-install-substrate.sh` and
  `scripts/substrate/install-substrate.sh`,
  mapping/projection-only sections in `scripts/mac/lima-warm.sh`,
  `scripts/mac/lima/units/substrate-world-service.service.tmpl`,
  `scripts/mac/lima/units/substrate-world-service.socket`,
  `crates/shell/src/execution/invocation/plan.rs`,
  `crates/shell/src/execution/platform/macos.rs`, and call-site-only
  `crates/world-mac-lima/examples/mac_backend_smoke.rs`.
- **Existing/new symbols:** edit `provision_macos_world`, `install_macos`, the dev macOS
  `lima-warm.sh` call section, `write_systemd_units`, `configure_guest`,
  `ShellConfig::from_args`, `ShellConfig::from_cli`,
  `host_doctor_main`, `world_doctor_main`, `resolve_lima_vm_name`,
  `selected_host_visible_transports`, `try_bootstrap_host_visible_transport`,
  `collect_world_doctor_assessment`, and example `main`; add no new symbol. Cleanup statements
  inside `write_systemd_units`/`configure_guest` remain byte-frozen. Unit templates may project
  only the verified commitment/mapping/service/socket fields.
- **Tests/proof:** R2-MAP-MAC-01/R2-GEN-01/R2-DIAG-01 use mapping/context-only cases in
  `tests/mac/installer_parity_fixture.sh`,
  `tests/mac/lima_doctor_fixture.sh`, `tests/mac/prefix_mapping_r2_3.sh`, colocated shell tests, and
  example compile/call update. Prove no outer override, A/B isolation, unit/doctor commitment
  agreement, explicit R3 prerequisite, and no lifecycle action.
- **Dependencies/native:** depends on M1–M3. Static/focused proof only; Z owns native macOS record.
- **Impact/review/stop:** impact every named Rust symbol and exact shell caller closure; treat
  `ShellConfig::from_cli`, doctor transport selection, and backend construction as HIGH posture.
  Packet ID `A1.1d-5R2-3M4`. Non-mac installer edits, forwarding activation, unit/socket cleanup, or
  product-transport claim stops M4.

**R2-3W1 — Windows installer context and child propagation.**

- **Outcome/claim:** construct the same Windows IH for dev/release install and uninstall, carry it
  through profile/shim/doctor/guarded-WSL child calls including `-NoAutoSource`, and keep deletion
  semantics unchanged.
- **PI ownership:** owns PI-042–PI-046 and PI-068–PI-070.
- **Files:** only `scripts/windows/dev-install-substrate.ps1`,
  `scripts/windows/dev-uninstall-substrate.ps1`,
  `scripts/windows/install-substrate.ps1`, and
  `scripts/windows/uninstall-substrate.ps1`.
- **Existing/new symbols:** edit only prefix/context setup and existing shim/profile/doctor/WSL
  call sections; add PowerShell functions `Resolve-InstallBootstrapContextV1`,
  `Assert-InstallBootstrapContextV1`, and `Invoke-SubstrateWithInstallContextV1` where needed.
  Release-copy self-derivation must bind the installed binary and current token. Every recursive,
  wildcard, profile-removal, version/bin replacement, stop/unregister, and cleanup block is frozen.
- **Tests/proof:** R2-MAP-WIN-01/R2-GEN-01/R2-DIAG-01 use new
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove default/custom A, ambient B, repeat
  propagation, `-NoAutoSource`, install/uninstall selection symmetry, current account+SID, forged
  carrier rejection, and zero B mutation. No lifecycle deletion is proof.
- **Dependencies/native:** depends on A–C. Static PowerShell proof only; Z owns native Windows.
- **Impact/review/stop:** exact source-caller closure for all edited top-level blocks and new
  functions; impact the shell Windows observation symbols they invoke. Packet ID
  `A1.1d-5R2-3W1`. A wildcard/recursive/action-predicate change, environment-selected Known Folder,
  guard movement, or deletion claim stops W1.

**R2-3W2 — Windows public helper and diagnostic mapping.**

- **Outcome/claim:** select/normalize the pipe once, validate IH/PM/current token at public helper
  boundaries, project the typed control root/scope, and preserve the WSL guard before mutation.
- **PI ownership:** owns PI-052 and the Windows half of PI-081; owns the selection edge of PI-079
  and supports PI-045, PI-049, PI-076, and PI-115.
- **Files:** only `scripts/windows/start-forwarder.ps1`,
  `scripts/windows/pipe-status.ps1`, `scripts/windows/wsl-warm.ps1`, and
  `scripts/windows/wsl-doctor.ps1`.
- **Existing/new symbols:** edit parameter/intake/diagnostic/child-argument sections and
  `Get-PipeNameFromPath`, `Normalize-WSLName`, `Get-InstalledWslDistros`,
  `Test-NamedPipe`, and `Get-ForwarderTargetInfo`; add
  `Resolve-PlatformBootstrapMappingV1`, `Assert-PlatformBootstrapMappingV1`,
  `Get-WindowsForwarderScopeV1`, and `Assert-CurrentWindowsPrincipalV1`.
  `start-forwarder.ps1` timeout kill and every statement from the `wsl-warm.ps1` guard onward are
  byte-frozen.
- **Tests/proof:** R2-MAP-WIN-01/R2-DIAG-01 use new
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove canonical pipe, exact registered distro
  spelling/machine ID/guest identity, token Known Folder, scope digest, A-scoped future
  config/logs, shared PID target, ambient mismatch rejection, and exact guard bytes.
- **Dependencies/native:** depends on W1. Static/focused only; Z owns native existing-WSL evidence.
- **Impact/review/stop:** source-caller closure for each edited PowerShell function and impact
  `normalize_windows_pipe_path`/`WindowsForwarderScopeV1::derive`. Shared pipe/PID state is
  HIGH/CRITICAL posture. Packet ID `A1.1d-5R2-3W2`. Guard movement, provisioning activation,
  timeout kill, PID removal, or deletion-manifest semantics stops W2.

**R2-3W3 — Windows WSL typed backend.**

- **Outcome/claim:** make the backend/paths/transport/warm interfaces consume and revalidate the
  verified PM without environment/default reselection or enabling guarded provisioning.
- **PI ownership:** owns PI-048 and supports PI-057, PI-076, PI-079, PI-090–PI-091, and PI-115.
- **Files:** only `crates/world-windows-wsl/src/backend.rs`,
  `crates/world-windows-wsl/src/lib.rs`, `crates/world-windows-wsl/src/paths.rs`,
  `crates/world-windows-wsl/src/transport.rs`, and
  `crates/world-windows-wsl/src/warm.rs`.
- **Existing/new symbols:** edit `WindowsWslBackend::new`, `WindowsWslBackend::build`,
  `agent_transport`, `build_agent_client`, `ensure_agent_ready`, `ensure_ready`,
  `ensure_persistent_session_ready`, `WarmCmd::enabled`, `WarmCmd::run`,
  `detect_tcp_forwarder`, `to_wsl_path`, and `to_windows_display_path`; add
  `WindowsWslBackend::new_with_mapping`, `observe_wsl_mapping_v1`, and
  `validate_wsl_mapping_v1`. Existing TCP mode remains diagnostic-only.
- **Tests/proof:** the backend portion of R2-MAP-WIN-01/R2-DIAG-01 uses colocated tests and
  `crates/world-windows-wsl/src/tests.rs`, plus mapping-only cases in
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove exact distro/guest
  machine/principal/home/pipe/commitment and failure before warm/provision on missing or conflicting
  PM.
- **Dependencies/native:** depends on W2 and B. Static backend proof only; Z owns native evidence.
- **Impact/review/stop:** impact every named Rust symbol; constructor and warm paths are
  HIGH/CRITICAL even when cfg indexing is incomplete. Packet ID `A1.1d-5R2-3W3`. Environment
  selection, guarded provisioning, guest-home guess, or transport redesign stops W3.

**R2-3W4 — Windows forwarder internal boundary.**

- **Outcome/claim:** require authenticated IH/PM and explicit A-scoped config/log paths in internal
  forwarder mode, validate token/distro/pipe/target/commitment, and transport but never delete the
  shared PID target.
- **PI ownership:** owns PI-049 and the forwarder-config consumption slice of PI-079; supports
  PI-060 and PI-115.
- **Files:** only `scripts/windows/start-forwarder.ps1`,
  `crates/forwarder/src/config.rs`, `crates/forwarder/src/logging.rs`,
  `crates/forwarder/src/windows.rs`, and `crates/forwarder/src/main.rs`.
- **Existing/new symbols:** edit `Cli`, `Cli::resolve_log_dir`, `windows::run`,
  `ForwarderConfig::load`, `default_config_path`, `resolve_target`, `logging::init`, and both cfg
  `main` entries; add `ForwarderConfig::load_internal` and
  `ForwarderConfig::validate_mapping`. Internal mode has no `LOCALAPPDATA`/`USERPROFILE` or target
  environment default. Timeout kill/PID removal/process termination remain frozen.
- **Tests/proof:** R2-MAP-WIN-01/R2-GEN-01 use colocated forwarder tests plus
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove exact explicit config/log/PID paths,
  token/carrier/PM equality, environment conflict rejection, and diagnostic TCP non-promotion.
- **Dependencies/native:** depends on W2–W3 and B. Static proof only; Z owns native Windows.
- **Impact/review/stop:** impact all named Rust symbols and source-close the PowerShell launch.
  Shared-state and CLI default removal are HIGH posture. Packet ID `A1.1d-5R2-3W4`. Ambient
  selection, PID deletion/ownership manifest, timeout action, or stream behavior change stops W4.

**R2-3W5 — Windows pipe listener and PM-derived WSL leaf.**

- **Outcome/claim:** bind the named-pipe listener to the normalized PM pipe and make the WSL bridge
  child receive only PM-derived distro/target/commitment with overwritten target environment and
  `WSLENV`.
- **PI ownership:** owns PI-060 and PI-115 and completes the downstream-consumer slice of PI-079.
- **Files:** only `crates/forwarder/src/pipe.rs`, `crates/forwarder/src/bridge.rs`, and
  `crates/forwarder/src/wsl.rs`.
- **Existing/new symbols:** edit `PipeListener::new`, `normalize_path`, `serve`,
  `run_pipe_session`, `run_tcp_session`, `spawn_bridge`, and `wsl::spawn`; add no public symbol.
  `finalize_bridge`, `bridge_copy`, `WslStream` read/write/shutdown, and wait/stream semantics are
  frozen except exact argument plumbing required by the named callers.
- **Tests/proof:** R2-MAP-WIN-01 uses colocated tests plus
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`; prove commitment-bound pipe, conflict rejection,
  exact `wsl -d` registered spelling, exact UDS target/commitment, full overwrite of inherited
  target variables/`WSLENV`, and diagnostic TCP non-promotion.
- **Dependencies/native:** depends on W4. Static leaf proof only; Z owns native Windows/WSL.
- **Impact/review/stop:** impact every named Rust symbol; pipe listener and child spawn are
  HIGH/CRITICAL posture. Packet ID `A1.1d-5R2-3W5`. Pipe reselection, inherited target authority,
  provisioning, timeout kill, stop/unregister, or stream/wait semantic change stops W5.

**R2-3F — typed factory and shell platform callers.**

- **Outcome/claim:** require the explicit typed factory projection on macOS/Windows and migrate all
  shell platform, platform-world, gateway, and doctor callers; no contextless platform factory
  remains reachable.
- **PI ownership:** owns PI-054–PI-058 and PI-075–PI-076; supports PI-090–PI-091.
- **Files:** only `crates/world-backend-factory/src/lib.rs`,
  `crates/shell/src/execution/invocation/plan.rs`,
  `crates/shell/src/execution/platform/macos.rs`,
  `crates/shell/src/execution/platform/windows.rs`,
  `crates/shell/src/execution/platform_world/mod.rs`,
  `crates/shell/src/execution/platform_world/windows.rs`, and
  `crates/shell/src/builtins/world_gateway.rs`.
- **Existing/new symbols:** edit every cfg `factory`, `ShellConfig::from_args`,
  `ShellConfig::from_cli`, both platform `host_doctor_main`/`world_doctor_main`,
  `connect_transport_stream_ws`, platform `detect`, Windows `context`, `get_backend`,
  `build_agent_client`, `build_macos_gateway_client`, `resolve_macos_gateway_client_endpoint`,
  `resolve_macos_host_gateway_socket`, `macos_default_world_socket_path`, and
  `build_gateway_client`; add only the `PlatformWorldContext::bootstrap_mapping` field. Linux
  factory behavior stays platform-independent.
- **Tests/proof:** R2-MAP-MAC-01/R2-MAP-WIN-01/R2-DIAG-01 and the applicable unchanged
  R2-RUNTIME-01 differential use colocated tests in the named files, mapping-only macOS/Windows
  focused tests, and the unchanged gateway differential; prove explicit shell callers,
  contextless platform failure before client/backend selection, Linux parity, and no
  forwarding/provisioning action.
- **Dependencies/native:** depends on M4 and W5. Static call-graph proof only; Z owns native runs.
- **Impact/review/stop:** impact every named Rust symbol; factory's indexed LOW result does not
  waive manual cfg source closure, and platform constructors remain HIGH/CRITICAL posture. Packet
  ID `A1.1d-5R2-3F`. A missing caller, new backend authority, Linux semantic change, lifecycle
  reachability, or gateway contract change stops F.

**R2-3R — replay factory-context migration.**

- **Outcome/claim:** carry explicit bootstrap/factory input through shell replay, public replay
  config, planner, and executor only; macOS/Windows direct library world calls without it fail before
  factory construction.
- **PI ownership:** owns PI-091.
- **Files:** only `crates/shell/src/execution/routing/replay.rs`,
  `crates/replay/src/lib.rs`, `crates/replay/src/replay/mod.rs`,
  `crates/replay/src/replay/planner.rs`, and
  `crates/replay/src/replay/executor.rs`.
- **Existing/new symbols:** edit `handle_replay_command`, `ReplayConfig`, `replay_span`,
  `replay_batch`, `ExecutionState`, `execute_in_world`, `replay_sequence`,
  `execute_with_world_backends`, and `try_world_backend`; add only
  `ReplayConfig::platform_bootstrap_mapping` and its exact internal `ExecutionState` projection.
  Recorded command/environment/origin/policy/timeout/strategy and agent-fallback semantics are
  byte-behavior frozen.
- **Tests/proof:** the factory-caller portions of
  R2-MAP-MAC-01/R2-MAP-WIN-01/R2-SHIM-01 use `crates/replay/tests/integration.rs`,
  `crates/replay/tests/planner_executor.rs`, and factory-context-only cases in
  `crates/shell/tests/replay_world.rs`; prove explicit platform projection, pre-factory failure when
  absent, Linux parity, and unchanged replay differentials.
- **Dependencies/native:** depends on F. Static/focused replay proof; Z owns native platform
  mapping, not replay semantic reproof.
- **Impact/review/stop:** impact every named symbol, especially `ReplayConfig`,
  `execute_with_world_backends`, and `try_world_backend`. Packet ID `A1.1d-5R2-3R`. Any replay
  semantic/schema/timeout/strategy change or ambient factory fallback stops R.

**R2-3S1 — physical-shim IH, manager, and factory binding.**

- **Outcome/claim:** recover exactly one no-follow invocation witness for the physical shim, bind
  the current OS principal, and use that IH for A-only manager manifests and explicit telemetry
  factory projection before dispatch.
- **PI ownership:** owns PI-090 and PI-116; supports PI-118.
- **Files:** only `crates/shim/src/context.rs`, `crates/shim/src/exec/mod.rs`, and
  `crates/shim/src/exec/logging.rs`.
- **Existing/new symbols:** edit `ShimContext::from_current_exe`, `resolve_invoked_path`,
  both cfg `find_candidate_in_dir`, `check_candidate`, `run_shim`,
  `collect_world_telemetry`, `ManagerHintEngine::new`, `manifest_paths`,
  `manifest_overlay_path`, and `repo_manifest_path`; add
  `resolve_install_bootstrap_context_from_invocation` and
  `current_platform_principal_v1`. PATH/CWD may resolve a unique witness but never prefix
  precedence; repo/ambient/manifest-environment fallback is unreachable in normal product mode.
- **Tests/proof:** R2-SHIM-01/R2-GEN-01 and the shim factory portions of
  R2-MAP-MAC-01/R2-MAP-WIN-01 use colocated tests and
  `crates/shim/tests/integration.rs`; prove absolute/relative/bare invocation, zero/multiple
  candidates, exact current Unix account+UID or Windows account+SID, no-follow identity, forged
  principal rejection, A-base/A-overlay manager hints, no repo fallback, explicit telemetry
  mapping, and zero B access.
- **Dependencies/native:** depends on B, C, and F. Native Unix shim proof may be recorded here;
  native Windows mapping remains assigned to Z.
- **Impact/review/stop:** impact all named symbols; graph LOW for `run_shim` does not waive the
  physical process-root review. Packet ID `A1.1d-5R2-3S1`. Multiple-witness guessing, ambient/repo
  selection, manager semantic change, recursive shim action, or lifecycle change stops S1.

**R2-3S2 — physical-shim trace and policy projection.**

- **Outcome/claim:** bind explicit product trace/policy inputs from the S1 IH before manager,
  policy, telemetry, span, or execution logging and reuse that one binding throughout the physical
  shim.
- **PI ownership:** owns the physical-shim portion of PI-118; T remains the final compatibility
  owner.
- **Files:** only `crates/shim/src/exec/mod.rs`,
  `crates/shim/src/exec/policy.rs`, `crates/shim/src/logger.rs`, and exact invocation/factory
  projection in `crates/shim/src/exec/logging.rs`.
- **Existing/new symbols:** edit `run_shim`, `evaluate_policy`, `start_span`,
  `log_execution`, `write_log_entry`, and `collect_world_telemetry`; add no public symbol.
  `evaluate_policy` and logging reuse the already-bound context and cannot initialize or select
  ambient state.
- **Tests/proof:** R2-SHIM-01/R2-DIAG-01 use colocated tests and
  `crates/shim/tests/integration.rs`; prove only
  `A/trace.jsonl`, policy Git A, repeated A reuse, conflicting-path rejection, missing-metadata
  no-fallback, and zero B filesystem access. Rotation/retention/writer/span/policy semantics are
  frozen.
- **Dependencies/native:** depends on S1 and R. Native Unix physical-shim evidence may land here;
  Z owns final cross-platform accounting.
- **Impact/review/stop:** impact every named symbol and `TraceContext::explicit_product` as the
  consumed boundary. Packet ID `A1.1d-5R2-3S2`. Any default trace init, environment-selected policy
  directory, writer/rotation/retention change, or side table stops S2.

**R2-3T — final trace compatibility closure.**

- **Outcome/claim:** only after F/R/S1/S2 callers are migrated, remove or make unreachable
  `LegacyAmbientCompatibility`, make global unbound `init_trace(None)` fail, and remove legacy
  ambient policy lookup without changing the neutral setter.
- **PI ownership:** owns final completion of PI-118.
- **Files:** only `crates/trace/src/context.rs`, `crates/trace/src/util.rs`, and
  `crates/trace/src/tests.rs`.
- **Existing/new symbols:** edit `TraceContextBindingV1`, `TraceContext::default`,
  `TraceContext::init_trace`, global `init_trace`, `get_policy_git_hash`, and exact tests;
  `set_global_trace_context`, `TraceContext::explicit_product`,
  `get_policy_git_hash_at`, writer/rotation/retention/span/replay bodies, and signatures remain
  frozen. Add no public symbol.
- **Tests/proof:** final R2-SHIM-01/R2-DIAG-01 closure uses `crates/trace/src/tests.rs`; replace the
  legacy-default expectation with final unbound failure and retain explicit-product A/B, policy
  Git, repeated-init, conflict, symlink, missing-metadata, writer, rotation, and retention tests.
- **Dependencies/native:** depends on F, R, S1, and S2 plus source closure proving no owned legacy
  caller remains. Native evidence is not independently required beyond migrated caller proof.
- **Impact/review/stop:** impact every edited symbol and query all callers of global `init_trace`
  and `get_policy_git_hash` before edit. Packet ID `A1.1d-5R2-3T`. Any unmigrated owned caller,
  setter semantic change, caller-identity table, lifecycle/writer change, or unrelated trace schema
  change stops T.

**R2-3Z — integration evidence and closeout.**

- **Outcome/claim:** `R2-3ZT1`, `R2-3ZH1`, `R2-3ZM5`, and the refreshed native macOS/Windows
  evidence are complete. This closeout joins the completed child increments, proves the complete
  R2-3 parent exit gate through accepted direct product/static checks plus fresh native evidence,
  records the canonical-runner provenance limitation truthfully, and hands only later-owner R2-4
  work plus R3-owned lifecycle/forwarding/provisioning/cleanup/rollback/convergence work forward.
  No production repair is allowed.
- **PI ownership:** closes only after code+proof for PI-009, PI-022, PI-039–PI-046, PI-048–PI-049,
  PI-052, PI-054–PI-058, PI-060, PI-068–PI-070, PI-075–PI-076, PI-079, PI-081, PI-090–PI-091,
  PI-112, PI-115–PI-116, and PI-118 is present. PI-059 remains harness-only; PI-077/PI-078 remain
  byte-frozen fail-closed guards; PI-050 remains the R2-4 guardrail; PI-080 remains satisfied by earlier R2-2 Linux
  restart-scope work and is not reopened here; and every R3 lifecycle/forwarding/provisioning/
  cleanup/rollback/convergence row retains the ownership above.
- **Files/symbols:** no production file or symbol. Evidence transcription may touch only the five
  existing runtime-refactor control-pack Markdown files in this bounded closeout subject and only
  to record current proof; any
  product defect returns to its owning child under a new fingerprint/review sequence.
- **Canonical-runner limitation:** `R2-3ZP3` is permanently deferred from the blocking path.
  Unpublished commits `2cb796ffef68c2b049376984a90ce0382e5f3980`,
  `9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`,
  `50948dbeb921582515a34bb6b7be21c46f29d008`, and
  `868994efaf132bb04c6cdd7c82da9433333c94e5` remain diagnostic evidence only. Honest
  authenticated wall results that still finalize ineligible with `evidence_write_failed` and
  `mount_teardown_failed` remain proof-infrastructure work outside the R2-3 gate.
- **Tests/proof:** the accepted direct shell-library result at final source
  `c583c5f293644fab75d8d42bd3bcad63f114d4fe` is
  `1322 discovered / 1274 passed / 48 failed / 0 ignored`, with failure-name SHA-256
  `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
  SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`. Count-only
  equivalence remains insufficient: the historical 45-failure inventory must remain present, and
  the only additional failures may be the three separately classified non-R2-3 world-deps/report
  expectations. `R2-3ZH1` owns only the host-inbox trusted-root test helper. `R2-3ZM5` owns only
  the macOS contextless-constructor removal and typed pre-R3 smoke contract. Their focused tests,
  package checks, formatting/Clippy/workspace checks proportionate to the touched crates, exact
  installer/script parser and frozen-byte checks, R2-SHIM-01, R2-GEN-01, R2-DIAG-01,
  R2-MAP-MAC-01, R2-MAP-WIN-01, applicable R2-RUNTIME-01 regression, allowlist/diff/GitNexus
  checks, and the parent independent review lenses remain mandatory.
- **Current broad-wall authority:** the only normative public entrypoints for this exact source are
  `make shell-lib-wall` and `make shell-lib-wall-serial`. The tracked Python runner remains
  historical diagnostic evidence only and its provenance-ineligible `1322 / 1277 / 45 / 0` result
  never overrides the accepted direct Make baseline.
- **Native macOS:** the refreshed source-bound macOS receipt
  `sha256:3b44f6387070b7aaea4306ae58d7f280b1cee3e163ee59219d4900ce3f53dfaf` is
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-MAC-01` at source
  `c583c5f293644fab75d8d42bd3bcad63f114d4fe` / tree
  `a070f5f5787c27180f13dece9a1c3c3241728fda`, with artifact digest
  `sha256:64a726d45b8bb6724fe43f566903d7b5e5ed6ef16119f14d30cd5a03edcc7e50`. It records supported
  macOS/Lima/tool versions, the typed account-database-derived Lima control root, and the future
  `A/sock/agent.sock` to `/run/substrate.sock` target without starting a forwarder or claiming any
  socket/process lifecycle action.
- **Native Windows:** the refreshed source-bound Windows receipt
  `sha256:4ee942690655c1fac185244438d14e2561df52c306dea7e5428d556b530fd28c` is
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-WIN-01` at that same source/tree, with artifact
  digest `sha256:2c36a8dae9f3bcfa1c240c62a1e44f93aee0798702ae78a0076d43778b93d46e`. It records the exact
  account+SID, registered distro/machine ID, normalized pipe/scope digest/shared PID root, and
  PM-derived WSL argv/environment under conflicting ambient values without provisioning, timeout
  kill, stop, PID deletion, unregister, or cleanup claims. Static proof never substitutes for a
  missing native assignment.
- **Impact/review/stop:** no pre-edit symbol impact unless a defect is returned to an owner.
  The accepted closeout subject uses packet ID `A1.1d-5R2-3Z` and a fresh bounded-review record
  over host-context/mapping-security, lifecycle/R2-versus-R3, and
  allowlist/native-honesty/regression lenses. Any production fix, false native claim, false
  canonical-runner-eligibility claim, incomplete PI/proof, changed guard/frozen byte, open P1/P2,
  or R3 authority leakage still prevents closeout. This row is complete only for R2-3; R2-4 and
  later R3 work remain next.

###### A1.1d-5R2-4 — R2 integration and closeout

| Packet field | Frozen requirement |
|---|---|
| Goal | Close the complete propagation matrix: default/custom A without outer overrides, ambient B conflict, repeat install propagation, install/uninstall context symmetry, generated and doctor correctness, and Linux product proof; hand cleanup/idempotency to R3. |
| Prerequisites | Final review-clean R2-1, R2-2, and R2-3 commits; all inventory rows implemented or explicitly R3-owned; clean tree and current GitNexus. |
| Must read | All six control-pack files; all three subpacket closeouts; PI-001–PI-118; every R2 proof gate and A1.1d-5I mapping. |
| Sibling context | This is a join/checkpoint only. A production defect returns to its owning subpacket. R3 remains next and owns all cleanup/convergence. |
| Exact production-file allowlist | **None.** Production fixes, generated checked-in artifact changes, installer changes, and platform changes are forbidden in R2-4. |
| Exact evidence/test allowlist | The existing six `llm-last-mile/runtime-refactor/*.md` control-pack files; the test files/runners already authorized by R2-1/R2-2/R2-3; no new production file. Test edits only close matrix coverage and must cite a missing proof row. |
| Explicit non-goals | Any production fix; deletion/rollback/manifest/convergence; R3 or `RG-INSTALL-01` closure; A1.1d/A1 closeout; B1/B2.1 joint closeout; B3.1 unblock; world capability/policy/supervisor/receipt changes; seam promotion. |
| Exit gate | Every R2-owned PI row has code+proof; supported Linux product install works at default and custom A with no outer override and with conflicting B; repeat install preserves commitment; install/uninstall select identical context; generated/doctor projections identify A; dedicated supported Linux full-world/Codex service proof is recorded; macOS mapping stops before forwarding and records the R3 activation prerequisite; R3 handoff lists its exclusive actions. |
| Regression gates | R2-UDEV-01, R2-UREL-01, R2-SHIM-01, R2-GEN-01, R2-RUNTIME-01, R2-LINUX-01, R2-DIAG-01 and applicable platform mapping gates; full format/Clippy/workspace/installer/world/doctor wall; GitNexus detection; `git diff --check`; unchanged B1/B2.1 and world/policy differentials. |
| GitNexus posture | Documentation/tests-only expected. Any affected production symbol or unexpected execution flow fails the packet and returns it to its owner. |
| Independent reviews | Fresh final authority/security, lifecycle/R2-versus-R3, and cross-platform/allowlist/regression reviews, all CLEAN. |
| Platform evidence | Linux native product proof is mandatory before R3. Native macOS/Windows assignments are reported truthfully as complete or still pending; static evidence never substitutes. |
| Stop conditions | An R2 row lacks an owner/proof; production repair is attempted; cleanup leaked into R2; Linux product host is unavailable; a native claim lacks native evidence; a seam/root architecture must change. |
| Next packet | A1.1d-5R3 — Partial-install cleanup and idempotency. |

**Terminal host status:** the historical conditional assignment in the
[booking and restoration contract](review-control/r2-4-linux-host-booking.md) remains preserved as
pre-start authority. The bounded Linux proof and exact restoration later completed on
`spenser-linux` from source `316ee5c6cf12c060388c9d9376e0a79537f2094a` / tree
`1eae07018caef023b2f27ef22892825b140e9a4d`; the product correction that removed its PATH-bypass
and public-wrapper limitations landed as `d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32` / tree
`b2d68905580d35d7d63aea8f36f274f723c26733`. The
[R2-4 closeout evidence record](review-control/r2-4-closeout-evidence.md) binds the immutable
artifacts and the later `NARROW_R2_4` authority: the historical full-world/Codex phrase is adopted
only as a bounded supported-Linux world-backed command and Codex-runtime reachability record,
without authenticated execution or architectural promotion. No production-file allowlist is
created by this closeout.

R2 transports the exact context that R3 will later use, but R3 exclusively owns deleting partial
candidates; current-attempt rollback; removal of managed gateway/helper/unit/socket artifacts;
managed-artifact ownership manifests used for deletion; removal of recursive/wildcard deletion;
uninstall convergence; restoration of installer-created group, membership, ACL, and linger state;
crash-window cleanup; uninstall-to-reinstall convergence; and preservation of unrelated or
pre-existing artifacts. No R2 exit gate may count any of those actions as complete.
The inventory additionally assigns macOS/WSL stop actions that touch shared platform state
(PI-051/PI-053), recursive dev-shim fallback removal (PI-064), shell-profile snippet removal
(PI-074), every action in PI-092–PI-103, and forwarding teardown/timeout PI-113–PI-114 to R3; R2
may transport their future exact target only. R3 may activate the PM-bound Lima SSH-UDS target only
after PI-101/PI-113/PI-114 land together; activation cannot reselect any R2 context or mapping field.

**A1.1d-5R1 exact allowlist.** R1 may edit only
`crates/shell/src/execution/agent_runtime/host_session_authority/trusted_fs.rs` and
`crates/shell/src/execution/home_bootstrap.rs`, with tests only in the colocated `#[cfg(test)]`
modules in those two files. Control-pack changes are limited to the existing six files under
`llm-last-mile/runtime-refactor/` and only for this allowlist or compact R1 closeout. Within that
file boundary, R1 permits only strict Linux ancestor access/default ACL parsing and validation,
structured private-home error provenance owned by HostSessionAuthority in `trusted_fs.rs`,
rendering of that typed canonical diagnostic by `home_bootstrap.rs`, and directly colocated tests.
`home_bootstrap.rs` gains no ACL-semantic or provenance authority. R1 explicitly excludes installer
or uninstaller propagation, shim behavior,
cleanup or deletion, prefix selection, generated configuration, provisioning, services, world or
policy behavior, macOS or Windows implementation, new schema or persistence formats, and general
trusted-filesystem refactoring. Requiring any implementation file, integration-test file, script,
fixture, or new module outside this allowlist is `CrossDocumentChangeRequired` rather than implicit
scope expansion.

**R1 exit/regression gate.** R1 exits only after the two authorized runtime files prove the closed
observation model; strict version/length/order/duplicate/base/mask/tag parsing; qualified `NoData`
versus unsafe mode; effective `--x`, `r-x`, masked-away raw write, multiple named entries, and all
effective-write rejection; ancestor/final access/default separation; non-`ENODATA` and unsupported
failure; exact diagnostic provenance and wording; final-root exact owner/`0700` across umasks
`000`, `022`, `027`, `077`, and `777`; no-follow identity/replacement rejection; invalid-root
nonmutation; no ACL cleanup; and the real `libvirt-qemu:--x` shape. Focused trusted-filesystem,
home-bootstrap, complete HostSessionAuthority, product bootstrap scaffold, and relevant installer
environment tests must pass, as must formatting, warnings-denied Clippy for touched targets,
workspace all-target check, `git diff --check`, and GitNexus change detection. The inherited shell
differential remains `1054 passed / 149 failed` with `PassToFail = 0`, `NewFail = 0`, `Removed = 0`,
and `RenamedOrSubstituted = 0`; retained failing names/signatures are unchanged and any
`FailToPass` is causally audited. This is not the later privileged Linux installer/product wall,
R2/R3 proof, native macOS proof, or cross-platform closeout.

The R1 contract correction and bounded runtime are review-clean through
`4d0acff68e20d86b97fe5367b8a4617554f33ef4`; R2-0 planning and R2-1 implementation are complete.
R2-2 Routes A–E are individually review-clean; R2-2E and F0/F0a/F0b/F0-HC are implemented,
proof-complete, review-clean, canonically closed out, and preserved. At that historical checkpoint
R2-2F was the next packet; the completed-F section below supersedes that status. `RG-HOME-01` and
`RG-INSTALL-01` remain open, A1.1d/A1 remain incomplete, and B3.1 remains blocked.
Their Linux regression wall and normal product lifecycle smoke are prerequisites for A1.1d Linux
closeout and for the Linux product-smoke portion of the B1/B2.1 joint closeout. They do not reopen
the review-clean B1/B2.1-0 prerequisite or make native macOS A1.1d proof a B1/B2.1 dependency.

The new A1.1 module boundary is organizational only and does not change any A1 contract, semantic gate, or later-slice owner. It enforces these additional reviewability constraints: no path-based authority operation after the trusted root opens; no symlink traversal or ambient-CWD authority; exactly one repository-owned canonical codec; exactly one centralized semantic transaction preflight; no legacy/direct writer may bypass `HostSessionAuthority`; key/object/root crash reconciliation is deterministic; and no new dependency is permitted without explicit dependency review. The bounded modules must not perform A2 episode demotion, helper removal, endpoint redesign, receipt work, unrelated StateStore cleanup, or general persistence extraction.

The `RG-AUTH-01` and `RG-AUTH-02` references in A1 are scoped gates, not whole-ledger closure claims. A1 proves only the `Start`/`Attach`/`ResumeOneTurn`, bounded helper-plan, real-REPL, and bounded auto-attach producer clauses named by A1.1–A1.4. General episode loss/demotion, Stop/Fork adoption, unrelated PID/socket consumers, retained workers, and compatibility/persistence separation remain unresolved for A2/A3 or their named later owners.

### Historical A1 sequencing checkpoint

- A1.1d-1 through A1.1d-4 implementation remains preserved.
- A1.1d-5 focused private-home implementation is review-clean on Linux; A1.1d integrated Linux
  closeout is open and cross-platform closeout is pending. A1.1d-5I has completed the bounded
  compatibility audit; R1 is review-clean through `4d0acff68e20d86b97fe5367b8a4617554f33ef4`,
  R2-0 planning is complete; R2-1 is review-clean through `2653c2ef`; R2-2 Routes A–D are
  individually review-clean but their integration closeout failed source closure. R2-2E is now
  implemented, proof-complete, review-clean, and preserved; F0/F0a/F0b/F0-HC are implemented,
  proof-complete, review-clean, canonically closed out, and preserved at runtime commit
  `770a6a9de9f537f7bc179c75421abbc3fff05b8d`. R2-2 remains incomplete pending F and the renewed
  closeout. R2-2F, renewed closeout, R2-3, R2-4, and R3 were then unstarted, and no A1.1d-6 exists
  or is implied. The next production increment at that checkpoint was **A1.1d-5R2-2F —
  Authenticated world-deps and truthful doctor composition**; the completed-F section below
  supersedes that next-task status.
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
  production closeout is now recorded against the bound 2026-08-03 source snapshot without
  additional product/test bytes. B3.1 is dependency-ready; no A2/A3 prerequisite moves into the
  corridor.
- B2.1 may transfer durable observation ownership while the current foreground call remains a
  blocking compatibility waiter. B2.2 owns early return later. The active A1.3-P1 packet remains
  open and still owns real CLI/helper/REPL adoption plus `RG-BASE-01` closure; the older A1.3 and
  A1.3-P0 records stay held as historical fences only.
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
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
| **B3.1 — Bounded retained-turn event/causation prerequisite** | Provide only the typed worker-to-host retained event envelope C1 requires, joined to B0 identity and B1 active-run truth, and normalize provider semantics at the producer before `AgentEvent` construction. | WorldWorkerMessagingProtocol. | `01` authority map/invariants 3–6; `02` MessagingProtocol row; `04` acceptance context + producer semantic normalization + retained event envelope; `05` `RG-MSG-01`/`RG-OBL-01`; messaging design envelope/thread/ordering/attention sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; RetainedWorkerRuntime; ObligationLedger. | Add `NormalizedWorldWorkerEventFacetV1`, the optional top-level `AgentEvent.worker_event: Option<WorldWorkerEventV1>`, and their typed fields in `crates/common/src/agent_events.rs`; consume B1's shared types from `crates/common/src/{authority_commitment.rs,lib.rs}`; consume/validate the existing B1 `WorldWorkAcceptanceContextV1` request extension in `crates/transport-api-types/src/lib.rs` without changing `ExecuteStreamFrame`; move the current provider-payload thread/class/attention/payload interpretation into one explicit fail-closed producer normalizer and shape the final member from that facet plus retained request/runtime context in exactly `crates/world-service/src/member_runtime.rs`; bounded host validation, deletion of C1-path JSON-pointer classification, and generic-journal handoff in `crates/shell/src/execution/orchestrator_world_dispatch.rs`; bounded host-facing semantic adapters in `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`; only mechanical `worker_event: None` compatibility updates in `crates/shell/src/execution/agent_runtime/world_work_execution_supervisor.rs`, `crates/shell/src/repl/async_repl.rs`, `crates/shell/tests/repl_world_first_routing_v1.rs`, `crates/shell/tests/support/repl_world_service.rs`, and `crates/world-service/src/service.rs`; focused tests colocated in those exact files. | No `ExecuteStreamFrame` variant/field change, no upstream provider-wrapper contract change, no new request-envelope semantics beyond consuming B1's context, no host or ledger identity/semantic inference from emitted `AgentEvent.data`, no producer-chosen eligible subset, no omission/untyped event/permissive downgrade for an unknown or malformed provider shape, no separate transport protocol, no host-to-worker message completion, no foreground early return, no worker park/cancel/stop/fork rewrite, no host-transition issuance/authentication/interpretation, no obligation classification/materialization, and no policy broadening. | After exact B1 acknowledgement, every retained `ExecuteStreamFrame::Event` through the terminal cut is in the closed B3.1 domain. Before `AgentEvent` construction, the producer normalizer maps every existing provider `AgentWrapperEvent` kind/payload into an exact typed facet for thread/class/attention/payload; parsing provider payload is permitted only at that named adapter boundary. Explicit supported non-attention shapes receive a typed non-attention class; missing, ambiguous, deferred, unknown, or malformed shapes fail the stream and C1 cut closed rather than being dropped, left untyped, or becoming progress/no-attention. The final typed `AgentEvent.worker_event` joins that facet with exact sources: acceptance ID, request/message causation and target backend come from the retained B1 context; session, source/target participant, source backend, active run, and world come from typed request/runtime context; B0 supplies event/frame identity. B3.1 validates that B2.1's generic event commitment equals the full canonical typed member and that all B0/B1/context fields match before generic-journal/C1 handoff; any opaque correlation is copied unchanged and compared only for equality. Host-side JSON-pointer classification is not a C1 input. Missing or mismatched identity fails closed; request ID or active-run ID never substitutes for transition intent/run. | Bounded producer/consumer clauses of `RG-MSG-01`; prerequisite clauses of `RG-OBL-01`; B3.1 event clause of `RG-OBS-01`. | B1/B2.1 joint production closeout and landed A1.1e. | Remaining `RG-MSG-01`, upstream provider-wrapper taxonomy cleanup, A1.2b production correlation supply, foreground retained receipt return, host-to-worker messaging, retained lifecycle/park/nonzero-exit behavior (`B3.2`), C1 materialization, and seam promotion. |
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
[`review-control/b1-b2-1-joint-closeout-linux-evidence.md`](review-control/b1-b2-1-joint-closeout-linux-evidence.md);
the differential record is in
[`review-control/b1-b2-1-joint-closeout-differential-evidence.json`](review-control/b1-b2-1-joint-closeout-differential-evidence.json).
The closeout changes only this control-pack/review-control surface, promotes no seam, and makes
B3.1 dependency-ready.

## B3.1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, B3.1 completes the retained
worker-to-host event/causation packet without changing `ExecuteStreamFrame`. `world-service` now
normalizes supported retained provider wrapper events fail-closed before `AgentEvent`
construction, joins one `NormalizedWorldWorkerEventFacetV1` with exact B1 accepted context,
typed runtime context, and copied B0 stream/frame/event identity, and emits the typed
`AgentEvent.worker_event` member on every post-acknowledgement retained `Event` frame through the
terminal cut. Shell consumers validate the full typed envelope, reject mismatched scope or
correlation, delete the old C1-path JSON-pointer inference, and hand the canonical commitment to
the existing B2.1 generic journal without interpreting it there.

Focused common, transport, world-service, and shell retained-event tests are green. Both
`make shell-lib-wall` and `make shell-lib-wall-serial` retain
`1324 discovered / 1276 passed / 48 failed / 0 ignored` and failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`. That inventory growth is
exactly two new passing shell tests:
`execution::orchestrator_world_dispatch::tests::accepted_retained_typed_event_validation_precedes_generic_journaling`
and
`execution::orchestrator_world_dispatch::tests::typed_control_ack_projection_uses_top_level_worker_event_class`.
The historical normalized-signature SHA-256
`2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90` first moved to the
independently authorized intermediate candidate SHA-256
`a1753d3d2dcd19f36a9350660ce88d9f2713871dd7734414c005ab8e52e70470`, then to the superseded
serial candidate SHA-256 `401fd5d38e059961de7fd4f773f7a817b9f2c1790ded0500228ed1168f7e811a`,
and finally to the accepted candidate SHA-256
`167807acacbf51c8507ef1c6a20d195b5d1a19e66ea62e5d612de1c2ef340f17`. The accepted final
differential against the superseded serial candidate is limited to 23 FILE:LINE-only movements in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`; test names, file paths, columns, and
normalized panic bodies remain unchanged, and the exact historical-to-final differential remains
recorded in
[`review-control/b3-1-differential-evidence.json`](review-control/b3-1-differential-evidence.json).
No test name, failure message, assertion, or production behavior is removed, renamed, substituted,
ignored, or weakened. B3.1 is complete, C1 is complete on the same bound Tuesday candidate,
A1.2b is complete on that same bound candidate as the internal durable successor/post-turn
protocol only, and no seam is promoted.

## Track C — Obligations, inbox, auto-attach, and router attach

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C1 — Event-to-obligation materializer and semantic cut** | Consume durable exact events, idempotently create canonical obligations while the supervisor observes active work, and own the monotonic ledger revision/materialized-event cut plus closed snapshot query consumed by A1.2b. | ObligationLedger. | `01` invariants 4–6; `02` RuntimeEventTransport/Supervisor/Messaging/Obligation rows; `04` runtime carrier, supervisor rules, and `ObligationLedgerSnapshotReadV1`; `05` `RG-OBL-01`/`RG-OBL-02`; obligation-ledger producer/dedupe sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; MessagingProtocol; HostSessionAuthority consume-only client. | `agent_runtime/obligation_ledger.rs`; bounded consumer integration from the receipt-scoped supervisor journal; bounded StateStore persistence only; focused materialization/cut/snapshot tests. | No runtime identity generation, receipt acceptance, observation ownership, retained-envelope or host-transition semantics, producer event eligibility choice, inbox rendering, router launch, direct prompt injection, or transfer of obligation semantics to HostSessionAuthority/StateStore. Stream exhaustion, EOF, timeout, PID/helper/socket state, inbox rows, pending counts, worker flags, and compatibility projections are forbidden completeness inputs. | Each B3.1 attention event creates exactly one canonical obligation before the exact B0 terminal event when applicable; duplicate journal replay creates none. Before classification and snapshot capture, C1 verifies that every post-acknowledgement retained B2.1 `Event` journal ref through the cut commits one full canonical B3.1 target/source/thread/class/attention/request/message/transition/payload envelope. The ledger advances one monotonic session revision and materialized-through event watermark, returns Pending before coverage, then returns a Complete snapshot binding exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, and the complete ordered exhaustive join of all retained `Event` refs through the cut even for `NoUnresolvedAttention`, plus unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run, authority revision, disposition, and sorted canonical-record commitments. Any untyped/omitted/substituted event or stale/mismatched scope, correlation, revision, cut, disposition, or record commitment keeps the cut non-Complete. | `RG-OBL-01`, C1 clauses of `RG-OBL-02`, obligation clauses of `RG-SUP-01`, bounded consumer clauses of `RG-MSG-01`, and C1 clauses of `RG-OBS-01`. | A1.1e, B0, B1, B2.1, and B3.1. | A1.2b production correlation supply and consumption/adoption, C2 projections, C3 router behavior, B2.2 foreground early return, and every seam promotion. |
| **C2 — Inbox and auto-attach projections** | Make inbox and attach eligibility pure projections over obligation state and effective policy. | InboxProjection and AutoAttachProjection. | `02` projection rows; notification inbox design; auto-attach trigger design. | ObligationLedger; CompatibilityReadModel; SteeringPolicyEngine. | `agent_runtime/{host_inbox,obligation_ledger,auto_attach,state_store}.rs`; `host_inbox_materialization.rs`; projection tests. | No host process launch, no worker action, and no removal of compatibility ingress without migration proof. | Deleting/rebuilding projections does not lose obligation truth; session-coalesced claim is deterministic; wrong-host and policy denial fail closed. | Projection clauses of `RG-OBL-02`, `RG-ATTACH-01`, `RG-BASE-03`. | A1, A2/A3, C1, and B4; B4 transitively supplies B2.2/B3.2/E2. | C3 router launch and unrelated worker controls. |
| **C3 — Router ownership restoration** | Consume attach-eligible claims, restore one sanctioned host episode, settle the claim, and stop. | RouterAttachTrigger for triggering; HostSessionAuthority for the attach transition. | `02` RouterAttachTrigger row; router responsibilities/non-responsibilities; `04` HostExecutionEpisodeV1. | HostSessionAuthority; SurfaceAdapter; AutoAttachProjection. | `agent_runtime/auto_attach.rs`; bounded router entrypoint; helper launch adapter; router tests. | No prompt replay, approval/answer/fork/continue, or always-running backend assumption. | Router produces one attach outcome per session claim and cannot invoke worker-control verbs; manual reattach coexists without duplicate ownership. | `RG-ATTACH-01`, `RG-ATTACH-02`, `RG-AUTH-01`. | C2 and A1/A2. | Worker-control, receipt, and unrelated policy work. |

At the bound Tuesday, August 4, 2026 source candidate, C1 is complete. The accepted retained path
now feeds exact B1 acceptance plus B2.1 durable retained `Event` refs and validated B3.1 envelopes
into `ObligationLedger` as each typed `Event` or terminal `Exit` is durably accepted, removes the
old terminal-coupled production writer from that C1-owned accepted path, and advances the
monotonic ledger revision/materialized-through watermark without reopening B0/B1/B2.1/B3.1
ownership. Broad proof remains monotonic through both `make shell-lib-wall` and
`make shell-lib-wall-serial` at `1330 discovered / 1282 passed / 48 failed / 0 ignored`, with
failure-name SHA-256 `c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and
normalized-signature SHA-256
`e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40`. That inventory growth is
exactly six new passing shell tests, and the retained 48-failure differential is limited to 23
FILE:LINE-only movements in `crates/shell/src/execution/orchestrator_world_dispatch.rs` recorded in
[`review-control/c1-differential-evidence.json`](review-control/c1-differential-evidence.json).
A1.2b is complete on that same bound Tuesday candidate as the internal durable successor/post-turn
protocol only, the then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`, and no seam
was promoted. The later reentry gate is closed by the selected
[A1.3 Linux-first packet](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md), later
narrowed by the held
[A1.3-P0 Linux-first preparatory packet](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md),
and finally corrected to the active
[A1.3-P1 Linux-first atomic public-adoption packet](linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md).

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

| Packet field | Frozen result |
|---|---|
| Identity | `A1.1d-5R2-2F0-HC`; internal A1 harness prerequisite. This row preserves the corrected authorization contract; the authorized implementation, proof, review, and canonical closeout are now complete. This is a correction to F0-HC, not F0c/F0d. |
| Audit process | The single process running the shell-library wall under the [canonical broad-wall invocation contract](04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract), its concurrently executable shell-library unit tests, same-binary helpers/readers/writers/background work, inherited child state, and shared test-support helpers. Integration parent mutation was inspected only for shared helper/inheritance contracts. |
| Inventory | 38 rows: A environment 4; B descriptors 3; C working state 3; D hooks/subscribers 4; E registries/singletons 11; F filesystem/sockets/ports 4; G time/scheduling 3; H background lifetime 3; I runner coordination 3. The corrected lexical scan covered 1,303 source test functions; the Linux wall discovered 1,263 tests. Environment closure is exactly 86 names: 74 parent-mutated and 12 child-only/read-only. Complete call closure corrects the parent-mutating total from 518 to 534 tests across the same 35 files. A2 is 129 direct tests across 22 files, comprising 116 additive plus 13 existing-source F0a tests; 27 additive tests lack `#[serial]`. The manifest adds the XDG gateway test, the Codex-auth guard test, both macOS `update_world_env_sets_*_flags` cases, and fourteen `with_test_mode`-only PTY cases, while removing two non-mutating B1 acknowledgement false positives. Other counts remain 100 CWD-dependent tests across 10 files, 120 sleep-dependent tests, 94 timeout-dependent tests, 127 Tokio-spawn-dependent tests, 48 thread-spawn-dependent tests, 14 child-spawn-dependent tests, 103 listener-dependent tests, 38 abort-dependent tests, 22 umask-dependent tests, two fd-replacement tests, one fd-flag test, one terminal/console-mode test, and one active-PTY-registry test. |
| Environment correction | The prior exact-name phase closed direct primitive arguments but not wrapper arguments/callsites. Corrected closure adds `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, `XDG_STATE_HOME`, `SUBSTRATE_OVERRIDE_ANCHOR_MODE`, `SUBSTRATE_OVERRIDE_ANCHOR_PATH`, and `SUBSTRATE_OVERRIDE_CAGED`, and reclassifies already-listed `SUBSTRATE_SHELL` as parent-mutated through the shared routing-test helper. The gateway XDG negative-authority test is added to the migration manifest; the settings and host-replay tests were already present. The validator fails on an absent wrapper callsite, unclassified literal/constant/dynamic name, parent-as-child misclassification, or missing mutating test. |
| Primary dispositions | `UnifiedProcessStateLock` 7; `ExplicitDependencyInjection` 4; `TerminationConfirmedTeardown` 1; `DeterministicSynchronization` 1; `SubprocessIsolation` 7; `ProvenConcurrencySafe` 17; `SeparatelyOwnedDeferred` 1. Total 38; no row is unowned. |
| New proven families | `SUBSTRATE_FORCE_PTY` mutator versus `is_force_pty_command`; CWD mutator versus `WorldRootSettings::effective_root`; global trace-output owner versus a second test initializer; PATH/timeout environment versus non-keyed `REPORT_CACHE`; one private retry hook versus another; repeated fixture session ID versus the dispatch concurrency tracker; global broker policy mutator versus `policy_mode`; first `ActivePtyGuard` owner versus a second registration. A tenth no-await abort and predictable stale stop-socket paths extend existing teardown/path findings. |
| Lock topology | One reentrant test-only authority-environment lane and a separate reentrant CWD lane. Required order: environment → CWD. Exact `OsString`/absence or `PathBuf` restoration precedes unlock. Panic, poison, nesting, non-Unicode, background lifetime, and inherited child state are explicit. Event registry, FDs, hooks, trace, cache, and sockets are not folded into a giant lock. |
| Other mechanisms | F0b explicit writers; explicit private retry callback, dispatch ID, and transport root; bounded subprocesses for the event registry, trace retargeting, non-keyed cache, `PTY_ACTIVE`, `ACTIVE_PTY`, and the global broker; the fd-flag case receives a fresh null/pipe open-file description while the terminal-mode case receives a child-owned PTY slave/console, never inherited parent fd 0; termination-confirmed server teardown; deterministic barriers/readiness instead of ordering sleeps. Every helper child uses a recursion-proof sentinel, bounded timeout, exact exit/signal propagation, and kill-then-wait/reap on timeout. |
| Existing authorization | The published F0 seven-file socket and F0b one-file renderer allowlists remain unchanged. The F0a 19-file HOME plus three integration-helper file boundary is unchanged, while its exact A2 test manifest is corrected from 12 to 13 by adding the named gateway XDG test. The additive source/symbol allowlist is exact in `02`. No new tracked helper file is permitted. |
| Updated async cleanup closure | The existing nine named F0 tests remain, and `wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible` is added. All ten are test bodies in `execution/orchestrator_world_dispatch.rs`; every `abort()` must be followed by awaited confirmed termination and fixture cleanup before process-state restoration/unlock. |
| Evidence wall | Forced overlap and controls are binding as recorded in `05`. The historical clean-code parallel wall `1113/150/0` and serial wall `1202/61/0` diagnose the pre-remediation harness only. The same-run parallel fragments recover 150 names but complete panic output for only 37; they cannot supply transitions. Final authority is three parallel walls and one serial wall, each exactly `1280 discovered / 1235 passed / 45 failed / 0 ignored`, with identical names and normalized signatures. |
| Required implementation proof | **Satisfied.** Focused forced-overlap stress covers every proven pair; exact prior/absence/non-Unicode restoration; panic/poison/nesting; stable-reader exclusion; child inheritance; exact stdout/stderr buffers; unique path/ID isolation; confirmed task/socket cleanup; deterministic readiness; and three independent default-parallel walls plus one canonical serial wall with identical discovery/pass/fail/ignored counts, failure names, and normalized signatures. |
| Frozen exclusions | No runtime code in F0-HC; no product global registry/side table; no production reader/cache/trace/retry/timeout/socket/PTY/lifecycle change; no reporter suppression, thread-count reduction, sleep increase, retry-until-green, ignored test, assertion weakening, or suite-wide subprocess; no installer cleanup, F, renewed R2-2 closeout, R2-3, R2-4, or R3. |
| Historical status and next | `F0/F0a/F0b/F0-HC complete`. At that checkpoint the corrected inventory, implementation, differential proof, four-wall concurrency proof, reviews, preservation, and canonical closeout were complete, and F was the next packet. The completed-F section below supersedes that next-task status. |

The resulting DAG is acyclic and binding:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F` →
`renewed R2-2 closeout` → `R2-3` → `R2-4` → `R3`.

## A1.1d-5R2-2F0 historical differential authority gate

`HistoricalParallelArtifactUnavailable` is a provenance result, not permission to infer missing
signatures or promote the recovered name union. The same-run historical transcript fragments
recover all 150 names, but only 37 names retain the complete panic output needed for normalization. The exact
historical serial artifact is semantic authority. Its binding transition matrix is:

| Transition | Count |
|---|---:|
| `PassToPass` | 1,202 |
| `PassToFail` | 0 |
| `FailToSameFailure` | 45 |
| `FailToChangedFailure` | 0 |
| `FailToPass` | 16 |
| `Removed` | 0 |
| `RenamedOrSubstituted` | 0 |
| `NewPass` | 17 |
| `NewFail` | 0 |
| `NewIgnored` | 0 |

The matrix reconciles: `1202 + 61 = 1263` historical tests, `1202 + 16 + 17 = 1235` final
passes, `45` final failures, and `1263 + 17 = 1280` final tests. Every `FailToPass` is causally
audited; `02` names every added authorized `NewPass`. Deterministic historical-serial and final
listings prove zero removal and zero rename/substitution.

Concurrency authority is the exact identity of three final parallel walls and one final serial
wall: each discovered 1,280, passed 1,235, failed 45, ignored 0, and retained the same failure
names and normalized signatures. The historical parallel `1263/1113/150/0` aggregate remains
diagnostic only. It does not support a named 105-transition claim, historical parallel signatures,
parallel `PassToFail`, or membership claims about the final 45 failures.

This correction waives none of the zero gates. The exact candidate is committed as
`770a6a9de9f537f7bc179c75421abbc3fff05b8d` with tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`; its manifest, fingerprints, ordinary patch, and
full-index patch remain the preserved values. All 16 `FailToPass` rows are causally audited, all 17
`NewPass` rows are the authorized tests named in `02`, and fresh containment reviewer
`/root/final_containment_corrected_authority` (`019f8681-7957-7cc3-88fa-37ab3ad2fc87`) returned
CLEAN. GitNexus/source closure found no changed production execution flow; the authorized F0b
delegation preserved output bytes. At that historical checkpoint F0/F0a/F0b/F0-HC were complete,
product and user-facing behavior did not change, F remained unstarted, the DAG was acyclic, and no
seam was promoted. The completed-F record below supersedes that packet status.

## A1.1d-5R2-2F historical F3/F4 correction and allowlist

At that checkpoint the DAG was:

```text
Routes A-E + harness
    -> F1 complete (authenticated context binding)
    -> F2 complete (authenticated scope propagation)
    -> F3/F4 blocked candidate preserved as evidence
    -> F3/F4 readiness/environment remediation complete
    -> F5-PD
    -> F5
    -> final F walls
    -> F closeout
    -> renewed R2-2 integration closeout
    -> R2-3 -> R2-4 -> R3
```

F1/F2 and the corrected F3/F4 nodes are complete. The preservation commit
`a343f0796d19d66c168c5bb2797856710cff5708` records the exact blocked candidate without promoting
it. F5-PD, F5, final F walls, F closeout, renewed integration closeout, R2-3, R2-4, and R3 are
unstarted. This ordering is acyclic, and no seam is promoted.

The existing F table rows labeled `Shared context`, `Exact production allowlist`, `Exact
surfaces.rs list`, `Exact test allowlist`, and `Runtime request rule` remain the boundary for the
historical F3/F4 propagation work. The `Runtime request rule` was narrowed and extended as follows:

| Future file | Exact future authority | Forbidden expansion |
|---|---|---|
| `crates/shell/src/execution/routing/dispatch/world_ops.rs` | EDIT `ensure_world_service_ready` only as a compatibility wrapper/delegator; ADD one private Linux explicit-target core or equivalent and one private non-secret service-posture input; EDIT the additive authenticated builder/private cfg implementation; ADD exact colocated readiness/environment/security tests. | No second readiness implementation; no semantic change to existing callers, probes, activation, stale cleanup, timeouts, spawn, or errors; no ambient selection in F. |
| `crates/shell/src/execution/routing/dispatch/prelude.rs`; `crates/shell/src/execution/routing.rs` | Export-only wiring for the additive authenticated builder. | No redirect of another caller and no lifecycle behavior. |
| `crates/shell/src/builtins/world_deps/surfaces.rs` | `run_world_command_for_deps_at` is the sole normal world-deps consumer of the additive builder; previously authorized propagation mechanics and tests remain bounded by `Exact production allowlist`, `Exact surfaces.rs list`, and `Exact test allowlist`. | No other request-builder consumer. |
| `crates/shell/src/builtins/world_enable/runner/provision_deps.rs` | `execute_with_profile` is the sole provision-deps consumer of the additive builder; previously authorized propagation mechanics and tests remain bounded by `Exact production allowlist` and `Exact test allowlist`. | No other request-builder consumer. |
| `crates/shell/src/builtins/world_deps/mod.rs`; `crates/shell/src/builtins/world_enable/runner.rs` | Only the already-authorized F3/F4 propagation/coherence mechanics and exact colocated tests recorded by `Shared context`, `Exact production allowlist`, and `Exact test allowlist`. | No readiness, environment-selection, or lifecycle owner. |

All other existing callers of `ensure_world_service_ready` remain source-identical unless a future
cross-document source-closure review proves a strictly mechanical signature adaptation unavoidable
and names that exact callsite first. The current audit found no such adaptation necessary.
`socket_activation.rs`, macOS/Windows adapters, world-service, service/install scripts, transport
schemas, policy/capability code, receipt/supervisor/retained-worker owners, and all other lifecycle
owners are frozen.

### Mandatory F3/F4 entry and exit gates

Entry requires clean F2 plus the preserved candidate as evidence only, fresh impact on every symbol
to be edited, and proof that invalid authority fails before any readiness side effect. Exit requires
the complete security/readiness test wall in `04`, exact two-consumer closure, no wildcard or parent
environment forwarding, compatibility-caller equivalence, no non-Linux behavior change, and fresh
review. Passing focused tests did not complete F; F5-PD, F5, and final F walls remain separate
nodes.

## A1.1d-5R2-2F5-PD — Non-mutating nested doctor boundary

The preceding F3/F4 entry language is historical. F3/F4 were complete at the pre-documentation-
replay runtime commits `54a5ce663cf2724b69b0c124acf2c2758ff622dd` and
`7802c44198625ea6849933140c390900ebe84190`. At that historical checkpoint F5-PD was the only
authorized next implementation packet. The completed-F record below supersedes that packet status;
F5-PD remains a bounded prerequisite beneath F5, not a top-level slice.

```text
F3/F4 complete
  -> F5-PD
  -> F5
  -> final F walls
  -> F closeout
  -> renewed R2-2 integration closeout
  -> R2-3 -> R2-4 -> R3
```

| Packet field | Exact boundary |
|---|---|
| Goal | Preserve the existing authenticated Linux child-process proof while selecting a non-mutating internal World Doctor observation before ordinary CLI bootstrap. Normal public `world doctor --json` and all non-Linux compatibility remain unchanged. |
| Production files | Exactly `crates/shell/src/execution/cli.rs`, `crates/shell/src/execution/routing.rs`, `crates/shell/src/execution/platform/mod.rs`, and `crates/shell/src/builtins/shim_doctor/report.rs`. No new production file. |
| Existing editable symbols | `cli.rs::WorldAction::Doctor` only for hidden bool `internal_passive_world_doctor_v1`; `routing.rs::run_shell_with_cli` only for Linux hidden-action validation/rejection and the post-auth/pre-projection early branch; `platform/mod.rs::handle_world_command` only for a fail-closed leaked-hidden-bit guard in the Doctor pattern; `report.rs::{gather_world_doctor_snapshot,run_json_subcommand,snapshot_from_command,snapshot_from_value}` only in Linux cfg sections for enabled-path production-fixture removal, hidden child argv, duplicate-rejecting raw decode, exact child validation, bounded snapshot construction, and `cfg(test)` closure. `build_report`, `disabled_world_doctor_snapshot`, `JsonCommandOutput`, and existing non-Linux implementations remain byte-for-byte unchanged. No other existing production symbol. |
| Additive private symbols | Exactly `routing.rs::passive_world_doctor_action_is_exclusive`, `platform/mod.rs::emit_authenticated_passive_world_doctor_v1`, and `report.rs::{PassiveWorldDoctorChildV1,PassiveWorldDoctorHostV1,PassiveWorldDoctorWorldV1,decode_passive_world_doctor_child_v1,validate_passive_world_doctor_child_v1}`, or mechanically different names with identical ownership/signatures. All are Linux-only; the predicate, wire structs, decoder, and validator are private, and the emitter is at most `pub(super)`. Any additional helper is a stop pending impact and control-pack correction. |
| Permitted signatures/visibility | The public `WorldAction::Doctor` Rust enum layout gains exactly the hidden bool; this is the sole public type-layout change and does not change normal CLI grammar/help/behavior. Linux-private `snapshot_from_command` and the Linux `cfg(test)` value decoder may add expected A prefix/commitment parameters through exact cfg-specific implementations. The emitter accepts `&InstallBootstrapContextCarrierV1` and returns the existing CLI exit-code/result form. All non-Linux signatures/bodies and all other public/crate-visible signatures, `Cli`, exports, and re-exports remain unchanged. |
| Exact branch | On Linux only, after successful `decode_and_bind_unix_install_bootstrap_context`, including current-principal and conflicting-projection validation, and before `install_bootstrap_projections`. It cannot call the ordinary Doctor arm. Every non-Linux target rejects the hidden bit before ordinary dispatch while leaving the no-bit path unchanged. |
| Result | On Linux, the child emits exactly schema 1, compile-time platform, top-level/host/world `ok=false`, exact duplicate non-secret A identity, and `world.status=unavailable`, then exits 4 with empty stderr. Parent validates and discards those bytes, producing existing `NeedsAttention`, `ok=false`, source `command`, exit 4, no stderr/details, and bounded `passive world doctor unavailable`; invalid input produces bounded `passive world doctor incoherent`. No coherent success without adequate permitted durable evidence. Non-Linux output remains unchanged and unproven. |
| Exact test files | Colocated tests only in the four production files above, plus exactly `crates/shell/tests/doctor_scopes_ds0.rs`, `crates/shell/tests/shim_doctor.rs`, and `crates/shell/tests/shim_health.rs`. No new test file. |
| HIGH/CRITICAL authorization | None. `install_bootstrap_projections`, `ShellConfig::from_cli`, `WorldDoctorSnapshot`, `WorldDoctorReportV1`, `ensure_world_service_ready`, and `socket_activation_report` are HIGH/CRITICAL frozen owners. F5-PD is authorized to precede/bypass them only for the authenticated hidden action, never to edit or call them. The LOW exact edit authorizations are `WorldAction` 0/0, `run_shell_with_cli` 1/2, `handle_world_command` 0/0, `gather_world_doctor_snapshot` 0/0, `run_json_subcommand` 0/0, and `snapshot_from_command`/`snapshot_from_value` 1/1 each. Frozen `build_report` is 1/5 and `disabled_world_doctor_snapshot` is 1/6. No edited symbol reaches an attributed process beyond the existing Execution or Shim-doctor module. The test-inclusive `maxDepth=5` CRITICAL totals are exactly `ensure_world_service_ready` 4 direct/10 total and `socket_activation_report` 6/12. |
| Frozen callers | `run_shell`, `main`, every ordinary/no-hidden-bit `run_shell_with_cli` caller/action, `build_report` and `disabled_world_doctor_snapshot`, Host Doctor, public World Doctor, Health/shim entrypoints outside the exact Linux gather cfg, config/policy/world-deps resolvers, readiness callers, macOS/Windows/fallback platform doctors, transport clients, world-service handlers, and overlay/world execution remain behaviorally unchanged. `handle_world_command` changes only for the leaked-bit guard. |
| Platform cfg | Linux alone accepts and validates the hidden passive mode. macOS, Windows, and every other non-Linux target reject the hidden bit and retain their existing fixture/public-child compatibility bytes and behavior, which cannot satisfy F5-PD/F5 proof or fabricate native/A-bound truth. |
| Exclusions | No schema, endpoint, daemon, privileged broker, persistence, side table, fixture authority, service-manager owner, config/policy/platform duplicate, socket metadata owner, readiness/start/restart, socket connect, HTTP/WebSocket, endpoint, execute, world creation, active probe, install/provision/sync/repair/cleanup, credential behavior, or public Doctor change. |

The hidden argv shape is exclusive: carrier plus `--internal-passive-world-doctor-v1` on the Doctor
action plus exactly `world doctor --json`, with no command/script/version/bootstrap-home/shim/
replay/workspace/agent or behavior flag.
Missing, malformed, duplicate, tampered, wrong-principal, conflicting-selector, environment-only, or
nonexclusive input rejects before observation. Parent environment projections may be overwritten
for child transport but cannot select the mode or supply missing authority.

When Linux World-enabled production reaches `gather_world_doctor_snapshot`, it must not consult
`A/health/world_doctor.json` and spawns the authenticated child. World-disabled composition keeps
the existing no-child disabled snapshot. The Linux raw-stdout decoder rejects duplicate/unknown
fields before conversion to `Value`, and the value decoder is reachable only under `cfg(test)`
for direct validator fixtures. Non-Linux gather/decoder behavior remains unchanged and cannot
supply proof. The validator rejects unexpected fields plus case/separator variants of
credential, token, API/private-key, authorization, password, secret, prompt, request/body/bytes/
input, carrier/auth-bundle, parent/full-environment, and commitment-preimage keys. Raw child JSON,
stderr, and rejected bytes are never retained or echoed. The separate world-deps fixture and
composition remain F5-owned and are not authorized by F5-PD.

### F5-PD required regression wall

The implementation must add or preserve exact tests proving all of the following:

1. Missing or malformed authenticated carrier rejects before observation.
2. Environment-only authority cannot select passive mode.
3. Conflicting ambient B cannot affect the passive child.
4. An inactive service remains inactive; the frozen World-disabled branch returns its current
   disabled snapshot without spawning a child.
5. No activation-socket connection occurs.
6. No service process starts.
7. No `/v1/doctor/world` request occurs.
8. No `/v1/execute` request occurs.
9. No world or filesystem probe is created.
10. Config, policy, metadata, socket, fixture, and service state remain byte/identity unchanged.
11. The exact authenticated A identity plus absent runtime evidence composes truthfully as unavailable; any future adequate permitted evidence for A must compose truthfully without fabricating success.
12. Missing evidence returns unavailable.
13. Mixed A/B evidence returns incoherent or the exact contract-owned fail-closed class.
14. Malformed, tampered, duplicate-key, or unknown-field evidence fails closed before lossy value
    construction; include conflicting top-level and nested duplicates and B-then-A identity keys.
15. JSON and human classifications agree.
16. Credentials, prompts, request/carrier bytes, preimages, unselected host paths, and synthetic markers do not leak; only already-public selected-prefix/commitment fields may appear.
17. Normal public `world doctor --json` compatibility tests remain byte/semantically unchanged.
18. F3/F4 exact environment and readiness tests remain unchanged and passing.
19. Non-Linux compatibility remains frozen and no native proof is fabricated.
20. No test is removed, renamed, ignored, substituted, or weakened.

Tests must use observation-only sentinels: a listening activation-socket trap that counts accepts,
bounded fake service/HTTP endpoints that count requests without starting product infrastructure,
process and filesystem before/after snapshots, and exact marker scans. Fixtures and injected
reports are test evidence only. No test may satisfy the gate by deleting the nested child, forcing
unconditional parent synthesis, or weakening current public Doctor expectations.

### F5-PD mandatory stops

Stop without implementation closeout on repository mismatch; stale or contradictory control-pack
state; any unapproved HIGH/CRITICAL edit; a new module or process-family owner; need for a
transport/report schema or world-service change; inability to branch before projection/scaffold or
avoid activation-socket connection; inability to represent unavailable/incoherent truthfully in
the existing fail-closed report; credential/request/carrier/preimage or unselected-path leakage;
product/public-Doctor behavior regression; non-deterministic differential; or review
infrastructure failure. No passing test waives a stop. F5 and later nodes remain unstarted until
F5-PD implementation and fresh review are complete.

## A1.1d-5R2-2F — completed phase record

The F5-PD prerequisite and its fresh review completed before F5. The condition in the preceding
sentence is therefore satisfied. At that completed-F checkpoint, F1–F5 and F5-PD were complete
and the renewed R2-2 integration closeout was the exact historical next unstarted node. The
closeout-remediation insertion below supersedes only that next-node disposition.

| Phase | Commit | Tree | Ordinary patch / full-index patch | Stable patch ID |
|---|---|---|---|---|
| F1 | `922e1792fe886ebe871400e999858e451f8fcb01` | `b91ba4cc37785a0f62dc07917d69812d1e09ca13` | `829b8a19c0d2deb53af0b9c2f277e7eab53b5001f41c59ebc3e7075c1a3e8b54` / `f7c16056ce5b379848203b066bffec37ef9f6f46fdcb2753af6af4feac01886f` | `155df010725c49c5a5720361d97d021e57464004` |
| F2 | `77fdcd8a139f3b72f7994f1a1692ddb4dfbf47e4` | `3f71d64a78376f7addb1cf063a8e78412ffd0a51` | `42f5278a6cab739c9d5b5e5ca7ee2c3a960ba8d8197e2f772192a65e391ed31b` / `b65e3146e870ac7579e43508fb87b4c1bdc63afae96bf67473b75a7285e8bfe8` | `0edc6a453575cffbf126514552fe6609777c6c82` |
| F3 | `0cd1d7af40347f3a149f2b84aca4d26ff1f09a3d` | `a2b1ce4632d85c5514dccdea83279cb473960c90` | `c9ec972b7098ce8b5d633f7793678fccbb7ffebda4d2769bc3119a4791990c61` / `3db155ae7e308f50e9f9c80b74ea5ace50b2fcc6e8ffd5cf34e0c288955343d2` | `a8ad65e71975b236f082f4abeb3b0fdcd37f216e` |
| F4 | `af2a3da6d85a6ca3f3f05bedf6768e4f89fd8e30` | `97cade96e9c6e763fcba7deb5154cf4c78ca8031` | `179e67b16316ae62683085cf17d5e4e56c30fec7c51ca12b4edeb47d78d36e65` / `3996fba97a44fc20c2e95d9395971b21172ceeca963a8d235b2ab7a57579fd2a` | `016e4edb9ecd21150a4e788ab93b7ba0dc0df4c6` |
| F5-PD | `653a7d91489563bc2a8e3395feeb53e159254240` | `4e3457444c64bbddda7bcf19300c4e7e81082678` | `b7a5b60f8c7dbd1960b7fec4870de047c827ed611ea1d4bfba33e6a9ed22e32d` / `b6e7e4a61c24815ef24fb74660d3cbc499de256771d1857bb963b0fd5f31b03d` | `90dafa73a5887457114ad2d4073fab8980e5630e` |
| F5 | `2bb4696d7181d974c1b02e33d82e09422cdae7de` | `313a8a613a6cd91e72c0cc1664f4b9842fefa305` | `991859870123dca56f3ee080876742abb2e556f9ee0bdf62f3577fdf1ccfb189` / `8e2be2f2b84dc5d3cd573b9670e4d3eccc3060893f478a414d057e39b002a943` | `b768cb94c51802e2440fbf1eb7f29fc0693a446b` |

The exact name-only manifests and their SHA-256 values are:

- F1 `583b56fbd6a174c477507f954f01258137312f7694e1a168b0b956625f06bc2e`:
  `builtins/world_deps/mod.rs`;
- F2 `cbd53734ce75e81a3be0d0e95ad7039ba1bab5a97724fb546cd873fe076ee024`:
  `builtins/world_deps/{mod.rs,surfaces.rs}`, `builtins/world_enable/runner.rs`, and
  `execution/platform/{linux.rs,mod.rs}`;
- F3 `3e4446f23d3deec5073c9076e246ae5f73dfa3ea4faa3fe3dd0e268b6ad65ace`:
  `execution/routing.rs` and `execution/routing/dispatch/{prelude.rs,world_ops.rs}`;
- F4 `546d3eab0828c2064a20a425bf5a44cbc17321c1c0bba315ff31bcc4137838c1`:
  `builtins/world_deps/{mod.rs,surfaces.rs}`, `builtins/world_enable/runner.rs`,
  `builtins/world_enable/runner/provision_deps.rs`, `execution/routing.rs`, and
  `execution/routing/dispatch/world_ops.rs`;
- F5-PD `ee52b673713f6b0d654858b81bfd966f0a2768da1a73381b775490b74dc71c75`:
  `builtins/shim_doctor/report.rs`, `execution/{cli.rs,platform/mod.rs,routing.rs}`, and
  `tests/{doctor_scopes_ds0.rs,shim_doctor.rs,shim_health.rs}`;
- F5 `3716e05de411351b410c31fba9b37aebf98f470a1e1170f6d6a5c35658dca883`:
  `builtins/{shim_doctor/report.rs,world_deps/mod.rs}` and
  `tests/{shim_doctor.rs,shim_health.rs}`.

All abbreviated paths in this list are relative to `crates/shell/src/` except entries beginning
with `tests/`, which are relative to `crates/shell/`.

F5's exact production manifest is two files:

- `crates/shell/src/builtins/shim_doctor/report.rs`: Linux cfg
  `gather_world_deps_section` and `status_for_world_deps_report` composition, plus mechanical
  `cfg(not(target_os = "linux"))` separation and equivalent path qualification on the existing
  non-Linux definitions with behavior/wire output unchanged;
- `crates/shell/src/builtins/world_deps/mod.rs`: Linux cfg identity fields/strict top-level decode
  on `WorldDepsDoctorSnapshotV1` and passive `collect_doctor_snapshot_v1` behavior only.

Its exact test manifest is `crates/shell/tests/{shim_doctor.rs,shim_health.rs}` plus colocated tests
in those two production files. The five added tests are exactly two library tests
(`world_deps_fixture_cannot_establish_runtime_health_or_cross_a` and
`world_deps_section_drops_unavailable_application_payloads`) and three integration tests
(`shim_doctor_surfaces_bounded_incoherent_world_deps_truth`,
`shim_doctor_no_fixture_uses_only_passive_child_and_unavailable_deps`, and
`health_and_shim_doctor_share_bounded_non_mutating_world_deps_truth`). Two pre-existing Linux
assertion branches were strengthened; no test was removed, renamed, substituted, ignored, or
weakened.

The complete phase gate is satisfied:

- exact A identity/commitment joins config, inventory, dependency scope, passive child, and output;
- enabled no-fixture composition executes exactly the passive authenticated child and returns
  bounded unavailable truth without active service/socket/transport/execute/probe behavior;
- disabled composition remains child- and fixture-free;
- fixture evidence is A-bound test/compatibility evidence only and cannot establish runtime health;
- mixed, malformed, duplicate, unknown, stale, partial, tampered, conflicting, and secret-bearing
  evidence fails closed without disclosure;
- Host, World, world-deps, shim-doctor, and Health JSON/human surfaces agree;
- F3/F4 readiness and exact seven-entry environment, public Doctor, managed secure-FD, policy,
  network, filesystem, capability, lifecycle, cleanup, and platform boundaries remain frozen.

Three parallel shell-library walls and one serial wall each report 1,309 discovered, 1,264 passed,
45 failed, and 0 ignored with exact inherited name/signature hashes. Shell/workspace all-target
checks, differential Clippy, formatting, diff checks, and GitNexus zero-flow containment pass.
Linux proof is complete; non-Linux remains explicit compatibility/unproven and privileged product
smoke remains R2-4-owned.

At that completed-F checkpoint, the historical sequence was `Routes A–F complete` →
**`A1.1d-5R2-2 renewed production-fix-free integration closeout`** → `R2-3` → `R2-4` → `R3`.
The remediation insertion below supersedes only that next-node disposition.

## A1.1d-5R2-2-B1 broad-wall provenance phase

This six-file documentation-only phase follows exact publication-authority commit
`928f94e7b4c498273b40385f7bffea9e4f949700` and precedes the renewed integration wall. Every live
broad shell wall, parallel/serial wall, canonical or differential baseline wall, final
change-detection wall, renewed closeout wall, and F/Harness wall in this phase map uses the
[canonical shell-library broad-wall invocation contract](04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).

| B1 phase field | Frozen requirement |
|---|---|
| Defect | The renewed-closeout prompt omitted the private `TMPDIR`/`XDG_RUNTIME_DIR` provenance used by the canonical F wall. Rust `TempDir` roots fell beneath normal mode-`1777` `/tmp`, and trusted-root validation correctly rejected that ancestor. Serial and isolated controls failed identically; exact private-root parity restored the canonical wall. |
| Classification | `BaselineCommandMismatchConfirmed`; runtime source/tests unchanged; not environment drift, test isolation, or production regression. |
| Root/command authority | Create one fresh validated private `R` per wall; bind exact `R/tmp` and `R/xdg-runtime` before Cargo starts; preserve root/command/toolchain/result/hash provenance; clean only the revalidated disposable root after every child exits and evidence is retained. Ambient temporary-root selection is ineligible. |
| Canonical evidence | `1309/1264/45/0`; failure names `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; normalized signatures `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`. |
| Ineligible evidence | `1309/1205/104/0`; failure names `7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5`; normalized signatures `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579`; 59 additions excluded from baseline authority. |
| Allowed files | Exactly the six canonical `llm-last-mile/runtime-refactor/00`–`05` Markdown files. |
| Forbidden work | Production, tests, fixtures, scripts, schemas, dependencies, generated files, trusted-root weakening, baseline replacement, renewed wall execution, source push, replay/rebase/rewrite/cherry-pick/merge/force push, integration-clean branch, R2-3/R2-4/R3. |
| Preservation | Commit directly on parent `928f94e7b4c498273b40385f7bffea9e4f949700`; preserve only through `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722`; leave the source remote at `2f6f1f69b3519dafff01ef543e7d260da2c37700`. |
| Historical B1 exit/next | Six-file validation and three fresh isolated read-only reviews CLEAN; local source 19 ahead and clean; then stop. The exact next task at that checkpoint was **Resume A1.1d-5R2-2 renewed production-fix-free integration closeout using provenance-validated private roots**; the remediation insertion below supersedes only that next-task disposition. |

The historical B1-only DAG was:

```text
publication-authority commit 928f94e7
  ↓
B1 broad-wall invocation docs correction
  ↓
renewed production-fix-free integration wall using validated private roots
  ↓
fresh independent reviews
  ↓
final six-file integration-closeout docs
  ↓
one ordinary fast-forward source push
```

R2-2 remained incomplete throughout B1. No integration wall ran in that phase. The current
controlling DAG is the remediation/publication map below.

## A1.1d-5R2-2 renewed closeout publication phases

The selected publication model is **Docs-on-top → one fast-forward publication**. Execute the
renewed closeout in exactly these phases:

| Phase | Allowed work | Completion evidence | Stop boundary |
|---|---|---|---|
| 1. Publication-authority correction | The six canonical control-pack files are committed as `928f94e7b4c498273b40385f7bffea9e4f949700` on exact runtime HEAD `e5fbd2d4441d248e137d52c44e493fb0abe158f8`; only dedicated preservation branch `feat/preserve-a1-1d-5r2-2-publication-authority-20260722` carries it remotely | Source remote still `2f6f1f69b3519dafff01ef543e7d260da2c37700`; local/remote preservation parity; original 17 identities unchanged | Complete; does not authorize an ambient-root integration wall |
| 1B. Broad-wall invocation correction | Commit exactly the six canonical files on parent `928f94e7b4c498273b40385f7bffea9e4f949700`; preserve only at `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` | Clean source worktree; source remote still `2f6f1f69b3519dafff01ef543e7d260da2c37700`; local source 19 ahead; runtime and publication-authority identities unchanged; private-root contract frozen | Do not run the renewed integration wall or push the source in B1 |
| 1C. Remediation planning | Commit/review exactly the six control-pack files on `f7ded83ef147b748678ba6b028eea959870a04fe`; preserve only the planning branch | Planning ref parity; source remote unchanged; exact R1/P1 allowlists and sequence frozen | No implementation, wall, integration-clean ref, source push, history rewrite, or stale-wall eligibility |
| 1D. R1 implementation | One bounded two-file R1 commit, focused proof, and fresh authority/security review | Exact non-disclosure/argv/diagnostic/xtrace contract and review CLEAN | No P1 work, broader caller/helper change, weak validation, wall, or source push |
| 1E. P1 implementation | One bounded two-file P1 commit, adversarial self-tests, and fresh provenance/security review | Authenticated runner, containment, continuous-authority deletion, and review CLEAN | No product semantic/dependency/privileged change, fallback harness, wall eligibility claim, or source push |
| 2. Fresh baseline and integration wall | First run three parallel and one serial canonical wall through P1 with distinct roots; only after that baseline is CLEAN, run the complete renewed production-fix-free closeout | Every required baseline/integration artifact has complete P1 provenance and is proof-clean; closeout makes zero implementation/test change | Any invalid root/containment/deletion proof makes a wall ineligible; any failure/stop blocks source push, is classified, and requires docs-first future remediation |
| 3. Independent reviews | Four fresh read-only integration reviews of the exact proof state: authority/security, lifecycle/product, inventory/source closure, and baseline/platform/publication | Historical pre-disposition publication gate: all four reviewers CLEAN; a purely documentary finding is remediated docs-only and re-reviewed by a fresh replacement | Any finding that invalidates proof or topology stops publication and requires the affected proof/review to rerun from a corrected preserved state; no source push on any unresolved finding |
| 4. Final documentation | Draft the final integration-closeout update across the same six canonical files after the certified range/proof; validate/review to CLEAN; commit the exact reviewed bytes; rerun final documentation validation/reviews | Final committed docs prove the wall, keep R2-2 status truthful, and match the pre-commit reviewed bytes; any authorized remediation successor is append-only and six-file-only | Fix pre-commit documentary findings in the worktree; a post-commit documentary finding permits only an append-only remediation-doc successor plus renewed docs validation/review; a proof/topology-invalidating finding reruns affected proof/review; no rewrite or implementation/product/seam change |
| 5. Publication | Freshly read the source remote and require exact old OID `2f6f1f69b3519dafff01ef543e7d260da2c37700`; bind an explicit expected-old-OID CAS/lease to that exact OID; independently prove the update is an ordinary fast-forward | Source contains the original 17 runtime commits, authority docs, B1 docs, remediation-planning docs, bounded R1 and P1 commits, final-closeout docs, and only review-required append-only remediation-doc successors; local HEAD, upstream, and remote are identical at zero ahead/zero behind | The CAS/lease cannot authorize a forced/non-fast-forward update; any old-OID change, non-fast-forward, second/staged push, or identity mismatch is a hard stop |

F's completed replay and historical Route D docs-first replay remain scoped to their own packets.
The renewed closeout certifies the already assembled range, so it must not replay that range.
The docs-on-top -> one ordinary fast-forward publication model and phase ordering above remain
controlling, but phase 3's pre-disposition four-review-CLEAN gate is historical only. RP3 has
since completed with the unchanged canonical `1309/1264/45/0` baseline and zero pairwise
differentials. RP4 product proof on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` / `5358497a8baec6f36e15aaef58415a759e64977d` is
accepted clean by human disposition; the raw persistence review remains `REQUEST_CHANGES` and is
preserved as three cache-only orchestration/attestation findings carried as non-blocking
process-audit debt outside RP4 product-proof scope. This exact six-file change was the bounded RP5
closeout docs packet. Source publication had not yet occurred in that run and was the exact next
step after the reviewed/committed RP5 docs; it later completed before the accepted R2-3 suffix and
refreshed native evidence closed R2-3, with R2-4 and R3 still later.

## R2-2 remediation insertion before renewed closeout (historical RP0-RP4 plan)

The failed renewed closeout inserts two docs-first remediation increments without changing the
existing commit identities or the final docs-on-top/one-fast-forward publication model. The table
below preserves the historical RP0-RP5 sequencing model that governed RP1-RP4; the controlling
current RP5 state is recorded above.

| Phase | Entry state | Authorized work | Exit gate | Forbidden |
|---|---|---|---|---|
| RP0 — planning authority (historical checkpoint) | Clean `f7ded83ef147b748678ba6b028eea959870a04fe`; source still at `2f6f1f69b3519dafff01ef543e7d260da2c37700`; two prior blockers | Source-close R1/P1; edit/review/commit exactly six control-pack Markdown files; preserve only the planning ref | Six-file scope and cross-document checks CLEAN; three fresh read-only docs reviewers CLEAN; planning preservation parity; source remote unchanged | Implementation/tests/scripts, wall rerun, integration-clean ref, source push, history rewrite, R2-3/R2-4/R3 |
| RP1 — R1 carrier non-disclosure | Reviewed RP0 authority | Add the exact structured redaction helper, change only `deploy_shims`, and extend only `prefix_propagation_r2_2.sh` | Focused adversarial proof CLEAN; byte-exact execution; carrier absent from every display/evidence surface; fresh authority/security review CLEAN | Carrier removal, weaker validation, generic regex redaction, `eval`, other production/test files, P1 work |
| RP2 — P1 tracked provenance runner | R1 implementation/review CLEAN | Add the exact two Linux-only test-harness files and no dependency; bind the reviewed P1 commit trailers to its bootstrap and three argv-template constants, then record separately the substituted actual argv hashes; implement only the `04`-frozen authenticated host → Stage A → Stage B topology, sealed runner projections, fixed private peer-credential socket, Stage-B `--as-pid-1` worker, explicit Stage-A output pipe, immutable seeds plus confined writable Cargo runtime, and host-finalized teardown | All 95 runner self-tests CLEAN; exact combined-output EOF/backpressure/bounds/hash proof; worker reap-to-`ECHILD`; Stage-A private-tree absence; host proof of Stage-A namespace/backing absence; containment/deletion/security review CLEAN; no product-flow effect | Product runtime/test semantics, service or privileged state, mutable external invocation artifact, ambient/unnamed supervisor, parent-only waiting, assumed arbitrary control/evidence FD inheritance, Stage-A self-finalization, pathname-only cleanup, or wall eligibility claim before every host gate |
| RP3 — canonical baseline | R1/P1 CLEAN | Run three parallel and one serial wall, each through P1 with a fresh validated private root | Every wall independently provenance-valid; canonical counts, hashes, and zero differential; no masked route | Reuse of old roots/results, fallback harness, implementation remediation |
| RP4 — renewed production-fix-free closeout | Fresh RP3 baseline CLEAN | Re-run the complete focused/integration/review wall without product/test changes | All gates and four fresh integration reviewers CLEAN | Any implementation/test remediation; classification of stale walls as eligible |
| RP5 — historical final docs + publication plan | RP4 CLEAN | Add final six-file closeout docs, validate/review, then one ordinary source fast-forward | Local/upstream/remote parity; final docs are the only commit added after the RP4-certified R1/P1 source state; R2-2 complete | Replay/rebase/merge/cherry-pick/squash/force/staged source publication |

The immutable order is:

```text
f7ded83
  -> RP0 / reviewed remediation-planning docs
  -> RP1 / R1
  -> RP2 / P1
  -> RP3 / fresh canonical baseline
  -> RP4 / renewed production-fix-free closeout
  -> RP5 / final six-file closeout docs
  -> one ordinary fast-forward source publication
  -> A1.1d-5R2-3
```

Historical closeout results stay historical: the four earlier matching walls remain ineligible and
the original NOT CLEAN/BLOCKED reviews remain findings, not passes. The table above is preserved as
historical sequencing only. RP3, RP4, RP5, and the ordinary fast-forward source publication later
completed at `0f1e147fb735791b44a65099a65167cbdc1803af`; the process-only bounded-review
calibration also completed. At that checkpoint, the published tip
`43e528af8c71c4f42a7b1238f729078f20ee3760` authorized only the docs-first R2-3D subdivision above.
R2-3A product code remained a separately authorized future control-pack node.

## A1.1d-5R3 authoritative implementation index

This entire R3 implementation index is archived preserved planning evidence, not the current next
dispatch. It is superseded for active scheduling by
[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md). Planning was complete at
`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with
planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`.
R3 implementation is `PARKED_BY_USER`, no R3 implementation task has been dispatched, and the
previous authority wall `AUTHORITY_REQUIRED:B1_B2_1_JOINT_CLOSEOUT` is now closed. B3.1, C1, and
the bounded internal A1.2b packet are complete on the bound Tuesday, August 4, 2026 candidate,
and the then-current historical gate was `AUTHORITY_REQUIRED:R3_RESUME`. The former edge below is
archived and does not govern current scheduling. At that historical checkpoint, only after fresh
user/meta authority revalidated live repository truth could it reopen the preserved future edge
`AUTHORITY_REQUIRED:R3_IMPLEMENTATION -> A1.1d-5R3-HOME`; R3 must then complete before A1.3, A1.4,
or A1 closeout. Every later edge remains an explicit authority gate.
No task is pre-created.

```text
AUTHORITY_REQUIRED:R3_IMPLEMENTATION
  -> HOME -> MANIFEST
  -> LINUX -> EVIDENCE:R3-LINUX-IMP-01 -> LINUX-CLOSEOUT
  -> MAC -> EVIDENCE:R3-MAC-IMP-01 -> MAC-CLOSEOUT
  -> WIN -> EVIDENCE:R3-WIN-IMP-01 -> WIN-CLOSEOUT
  -> UNIX
  -> EVIDENCE:R3-NATIVE-LINUX-01
  -> EVIDENCE:R3-NATIVE-MAC-01
  -> EVIDENCE:R3-NATIVE-WIN-01
  -> CLOSEOUT
```

The three final native evidence nodes all bind the same published `UNIX` commit/tree/ref; the
linear drawing is dispatch order, not a change of checkpoint. Evidence nodes are read-only
platform tasks and publish no repository bytes. Each validates a native
`codex.top-level-evidence-receipt.v1` with the orchestration skill validator; that validator binds
the receipt and artifact digest but has no successor field. The separate native evidence artifact
is independently validated with the exact expected gated successor under `04`. The user/meta
orchestrator must separately authorize each edge.

Every future evidence dispatch is correlated to product project
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` and discovers the repository by exact origin plus target
ref. Its dispatch binds a fresh nonce, source implementation task thread/host, evidence task
thread/host, return meta thread/host, evidence ID, source commit/tree/ref, artifact paths, and
gated successor. Those exact correlation values appear in artifact and receipt; the evidence task
sends the validated native JSON to the bound return meta task with `send_message_to_thread` as its
final tool action. Planning-task IDs or a different project/host are never inferred or reused.

Each packet starts from the exact landed predecessor, permits only the commit count named in its
contract and one normal fast-forward push of that linear range to the bound target ref after every
gate, and ends with a validated `codex.top-level-task-receipt.v1`. Merge, rebase, cherry-pick,
reset, amend/rewrite, force push, staged publication, or an orchestration-branch push is forbidden.
A packet maps every terminal condition through the closed status table in `04`; no invented status
is permitted.

Every implementation packet must run fresh GitNexus query/context exploration, impact every
existing function/method before edit, manually close script/cfg/generated/platform callers, and
run `gitnexus_detect_changes()` before commit. Any HIGH/CRITICAL result, new caller outside the
fence, or graph/manual disagreement requires explicit risk review; a new authority domain,
selector, execution family, or destructive primitive is `BLOCKED_SCOPE_EXPANSION`.

In the packet contracts below, canonical documentation set `R3-DOCS` means exactly
`llm-last-mile/runtime-refactor/00-README.md`,
`02-seam-crosswalk.md`, `03-phase-slice-map.md`, `04-contracts-and-gates.md`, and
`05-debug-regression-ledger.md`, with the same directory prefix on the last four paths.
`01-target-architecture.md` is frozen through implementation and evidence packets. Only
`A1.1d-5R3-CLOSEOUT` may append its bounded terminal R3 status in that file; changing any frozen
architecture or earlier text is `BLOCKED_SCOPE_EXPANSION`. Each named review set means exactly one
`review-control/r3-<slug>-review-cycle-record.json` plus
`review-control/r3-<slug>-review-authority-security.md`,
`review-control/r3-<slug>-review-lifecycle-convergence.md`, and
`review-control/r3-<slug>-review-allowlist-evidence.md`; angle-bracket substitution is descriptive
notation here, and each packet's literal slug is its lower-case suffix.
`R3-DOCS` is a sequential status/receipt surface: each packet may append only its own landed
result and may not rewrite another packet's row, proof, or architecture.
Every provider/final evidence task and closeout validates the native artifact with the
MANIFEST-owned `scripts/ci/validate_r3_native_evidence.py` and exact expected evidence ID,
source commit/tree/ref, and artifact `gated_successor`; it then validates the separate
`codex.top-level-evidence-receipt.v1` with the skill validator's single positional receipt
argument and exact-joins receipt field `evidence.artifact_sha256` to the validated artifact digest.
The skill validator is never passed or
credited with a successor check.

### `A1.1d-5R3-HOME`

**Completion claim.** Implement only synchronous exact current-attempt private-home candidate
rollback. Unknown-provenance crash residue remains unchanged/fail-closed. Predecessor:
`A1.1d-5R3-PLAN`; successor:
`AUTHORITY_REQUIRED:A1.1d-5R3-MANIFEST`.

- Ownership: `A1D5I-HOME-04`, `A1D5I-HOME-05`, the implementation clause of `RG-HOME-01`.
- Production allowlist:
  `crates/shell/src/execution/agent_runtime/host_session_authority/trusted_fs.rs` symbols
  `ensure_private_substrate_home`, `ensure_private_substrate_home_with`,
  `PrivateHomeCandidateProvenance`, `PrivateHomeError`, and new
  `PrivateHomeCandidateRollback`/`rollback_created_private_home_candidate`; and
  `crates/shell/src/execution/home_bootstrap.rs` symbols
  `ensure_substrate_home_deps_scaffold_at`, `HomeBootstrapError`, and
  `map_trusted_scaffold_error`.
- Test allowlist: embedded tests in those two files only. Required negatives cover
  `AlreadyExists`, pre-existing, replaced, nonempty, wrong type/owner/mode/ACL, unsupported or
  ambiguous lookup, symlink, descriptor/name/identity mismatch, and `Unknown`; kill points bracket
  create, first open, validation, cleanup, and acceptance.
- Documentation/control surface: exact `R3-DOCS` plus the exact `home` review set.
- Checks: focused private-home/home-bootstrap tests, `cargo test -p shell --lib`, formatting,
  clippy for `shell`, `git diff --check`, allowlist, secret review, change detection, and a
  non-privileged secure-fixture bootstrap smoke. GitNexus currently reports the wrapper HIGH
  (16 graph-visible direct callers; manual production chain reaches `run_shell_with_cli`) and the
  inner helper MEDIUM; the packet must preserve that caller set or stop.
- Platform/privilege: Unix implementation; Linux ACL/identity behavioral proof, no sudo. One
  commit/push. `BLOCKED_SCOPE_EXPANSION` if durable provenance is required;
  `BLOCKED_CONTRADICTION` for any attempted recursive/path-only cleanup.
- Non-goals/freeze: all installers, manifests, shims, services, Lima, Windows, passive health, and
  pre-existing-invalid-home remediation.

### `A1.1d-5R3-MANIFEST`

**Completion claim.** Land a non-destructive, canonical managed-artifact manifest/parser/
publication/state-transition core and hidden authenticated CLI surface. It validates and records
authority but performs no artifact deletion, replacement, stop, kill, unregister, or restoration.
Predecessor: `HOME`; successor: `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX`.

- Ownership: prerequisite for `RG-INSTALL-01`, `R3-LIFE-01`, and `R3-WIN-01`; no PI row and no
  destructive finding closure. It cites as consumers all 22 current R3 rows and findings
  INSTALL-02/03/06/07/08/10 while their sole owners remain exactly those in `02`.
- Production allowlist:
  new `crates/common/src/managed_artifact.rs` with
  `ManagedArtifactManifestV1`, `ManagedArtifactEntryV1`, `ManagedArtifactIdentityV1`,
  `ManagedArtifactRoleV1`, `ManagedArtifactDispositionV1`, `ManagedLifecycleStateV1`,
  `ManagedActionPreparedRecordV1`, `ManagedActionReceiptV1`, `ManagedActionReceiptIndexV1`,
  `ManagedActionReceiptIndexEntryV1`, `ManagedManifestHeadV1`, `ManagedSharedClaimsV1`,
  `ManagedSharedClaimV1`, `ManagedExecutorIdentityV1`,
  `LifecycleSignatureV1`, `LifecyclePublisherAnchorV1`,
  `LifecyclePublisherProtectedStateV1`, `ManagedLifecyclePublisherRequestV1`,
  `PublisherBootstrapAuthorizationV1`, `PublisherBootstrapComponentV1`,
  `PublisherBootstrapComponentRoleV1`,
  `PublisherTestRetirementCommitmentV1`,
  `PublisherTestRetirementAuthorizationV1`, `PublisherTestRetirementReceiptV1`,
  `PublisherTestRetirementAcknowledgementV1`,
  `GuestPublisherRetirementReservationV1`,
  `GuestPublisherReservationUnusedProofV1`,
  `GuestPublisherReservationUnusedAcknowledgementV1`,
  `GuestPublisherTestRetirementCommitmentV1`,
  `GuestPublisherTestRetirementAuthorizationV1`,
  `GuestPublisherTestRetirementReceiptV1`,
  `GuestPublisherTestRetirementAcknowledgementV1`, `PublisherTransportFrameV1`,
  `GuestPublisherPairingChallengeV1`, `GuestPublisherPairingTicketV1`,
  `GuestPublisherPairingHostRecordV1`, `GuestPublisherPairingGuestIntentV1`,
  `GuestPublisherBootstrapHelloV1`,
  `GuestPublisherBootstrapTranscriptV1`, `ExecutorBuildEvidenceV1`,
  `LimaStageOneAuthorizationV1`, `ManagedActionV1`,
  `CanonicalManifestBytesV1`, `parse_and_validate_manifest_v1`,
  `canonical_manifest_bytes_v1`, `canonical_publisher_bootstrap_authorization_v1`,
  `canonical_publisher_bootstrap_core_v1`,
  `canonical_managed_action_prepared_record_v1`,
  `canonical_lifecycle_publisher_protected_state_v1`,
  `validate_lifecycle_publisher_protected_state_v1`,
  `canonical_action_receipt_bytes_v1`, `canonical_managed_action_receipt_signature_payload_v1`,
  `validate_managed_action_receipt_signature_v1`, `action_receipt_artifact_sha256_v1`,
  `canonical_action_receipt_index_bytes_v1`,
  `compare_and_swap_action_receipt_index_v1`,
  `commit_action_receipt_index_to_head_v1`,
  `retirement_receipt_artifact_sha256_v1`,
  `validate_publisher_test_retirement_acknowledgement_v1`,
  `canonical_guest_publisher_test_retirement_ticket_core_v1`,
  `canonical_guest_publisher_reservation_unused_proof_v1`,
  `validate_guest_publisher_reservation_unused_proof_v1`,
  `guest_publisher_reservation_unused_proof_artifact_sha256_v1`,
  `canonical_guest_publisher_reservation_unused_acknowledgement_v1`,
  `validate_guest_publisher_reservation_unused_acknowledgement_v1`,
  `guest_publisher_reservation_unused_acknowledgement_artifact_sha256_v1`,
  `validate_guest_publisher_test_retirement_authorization_v1`,
  `validate_guest_publisher_test_retirement_acknowledgement_v1`,
  `canonical_guest_publisher_pairing_ticket_v1`,
  `validate_guest_publisher_pairing_ticket_v1`,
  `canonical_lifecycle_signature_payload_v1`, `verify_lifecycle_signature_v1`,
  `parse_p256_spki_der_v1`, `verify_p256_p1363_low_s_v1`,
  `verify_ed25519_fixed_v1`,
  `parse_publisher_bootstrap_authorization_v1`, and
  `validate_publisher_bootstrap_authorization_v1`,
  `derive_publisher_bootstrap_component_target_v1`,
  `validate_complete_publisher_bootstrap_component_set_v1`,
  `canonical_lima_stage_one_authorization_v1`, and
  `validate_lima_stage_one_authorization_v1`; `crates/common/src/lib.rs` module/export only;
  new
  `crates/shell/src/execution/managed_lifecycle.rs` with
  `ManagedLifecycleControlRequestV1`, `publish_manifest_v1`, `load_manifest_v1`,
  `derive_lifecycle_capsule_locator_v1`, `open_trusted_lifecycle_capsule_v1`,
  `compare_and_swap_head_v1`, `transition_manifest_v1`, `update_shared_claims_v1`,
  `publish_action_receipt_index_v1`, `resume_action_receipt_commit_v1`,
  `issue_publisher_bootstrap_authorization_v1`, `open_publisher_bootstrap_channel_v1`, and
  `validate_publisher_response_v1`, plus trait `LifecyclePublisherClientV1` with closed
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, and
  `issue_guest_publisher_pairing_ticket_v1` methods; new platform-client
  stub files `crates/shell/src/execution/managed_lifecycle/linux_client.rs`,
  `macos_client.rs`, and `windows_client.rs`, each exposing only
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, and
  `issue_guest_publisher_pairing_ticket_v1`, returning a typed provider-unavailable error until
  its sole platform packet replaces those stub bodies;
  `crates/shell/src/execution/mod.rs` module declaration/re-exports only;
  `crates/shell/src/lib.rs` re-exports only; new non-destructive control binary
  `src/bin/substrate-lifecycle-control.rs` symbols `main`,
  `publisher_bootstrap_direct_interactive_v1`, `read_exact_bootstrap_confirmation_v1`, and
  `guest_publisher_pairing_direct_interactive_v1`, `display_guest_pairing_challenge_v1`, and
  `submit_managed_lifecycle_request_v1`; new evidence validator
  `scripts/ci/validate_r3_native_evidence.py` and focused test
  `scripts/ci/test_validate_r3_native_evidence.py`; `Cargo.toml` only to add direct root-package
  dependencies `substrate-common = { version = "0.2.8", path = "crates/common" }`, `serde` with
  derive, `serde_json = { workspace = true }`, `ed25519-dalek = { workspace = true }`,
  `rand_core = { workspace = true }`, and
  `libc = 0.2`; its `[workspace.dependencies]` may add `base64 = 0.22`,
  `ed25519-dalek = 2.1` with `rand_core` and `pkcs8`,
  `p256 = { version = "=0.13.2", default-features = false, features = ["ecdsa", "pkcs8", "std"] }`,
  and `rand_core = 0.6` with `getrandom`; target-Windows root dependencies may add
  `windows-sys = 0.52` with exactly
  `Win32_Foundation`, `Win32_Security`, `Win32_Security_Cryptography`,
  `Win32_Storage_FileSystem`, `Win32_System_IO`,
  `Win32_System_Pipes`, `Win32_System_Registry`, `Win32_System_Services`, and
  `Win32_System_Threading`; `crates/common/Cargo.toml` only to add workspace `serde_json`, `sha2`,
  `uuid`, `base64`, `ed25519-dalek`, `p256`, and `rand_core`; and `Cargo.lock` only for the resulting
  dependency graph. No other
  manifest key, version, feature, package, checksum, or lock entry is mutable.
- Mutable test allowlist: embedded tests in
  `crates/common/src/managed_artifact.rs` and
  `crates/shell/src/execution/managed_lifecycle.rs`, plus new
  `crates/shell/tests/managed_lifecycle_v1.rs` and the exact evidence-validator test above.
  Required cases cover canonical/golden encoding,
  unknown/duplicate/missing/wrong-type fields, invalid Unicode/NUL, path normalization, owner/
  version/context/platform/principal/object/type/disposition/metadata joins, missing/malformed/
  tampered/stale/cross-principal/cross-instance/partial manifests, and kill points before/after
  temp write, temp sync, publication rename, parent sync, head CAS, shared-claim CAS, transition,
  and receipt. Add coherent forgery, valid-old replay, parent replacement, locator substitution,
  unprivileged publisher, role scope escape, publisher bootstrap field/confirmation/peer/framing/
  replay negatives, evidence artifact source/successor/correlation joins,
  bootstrap-core/retirement-precommit joins with no digest cycle,
  publisher-signed and externally hashed action receipts plus externally hashed retirement
  receipts and exact acknowledgement joins with no self or future digest field, manifest-
  preallocated per-generation receipt names, unsigned/wrong-key/wrong-algorithm/wrong-counter/
  wrong-prepared-record/action-observation and same-principal receipt substitution negatives,
  full `LifecyclePublisherProtectedStateV1` bytes at each exact platform locator, prepared-slot/
  counter/revision/final-anchor CAS and terminal retention, with detached-digest, alternate-locator,
  torn/rollback/replay, and clear-before-final-anchor negatives; extra-file and wrong-generation
  preservation, kill-before/after receipt signature and durable
  receipt publication, action-receipt signature/index/head/anchor CAS crash joins, and rejection
  of a final-anchor digest embedded back into its index, Ed25519 and
  P-256 SPKI/P1363/
  low-S/domain-separation golden vectors, host-publisher pairing ticket canonical/signature/scope/
  expiry/consumption vectors, guest-TTY pinning, and ticket/transcript replay negatives,
  including malformed/substituted SPKI, wrong curve/algorithm/encoding, SPKI/fingerprint and
  SPKI/anchor-key mismatch, DER/high-S/malleable P-256 signatures, and fixed Ed25519 harness-
  authorization/acknowledgement key/signature encodings; exhaustive host/guest test-retirement
  DAGs, null-slot guest ticket-core construction, commitment copying through ticket/host record/
  intent/hello/transcript/generation one, guest receipt/acknowledgement before reverse-order guest-
  then-host removal, kill points at every removal/parity join, and missing/cross-ticket/late
  commitment, early seed-intent/pairing-record/protected-state removal, omitted-role, and non-parity
  negatives; harness-only pre-host-bootstrap reservation of exactly zero Linux or one MAC/WIN
  challenge/pairing-record/guest-component tuple, its reserved/issued/consumed/unused/retired
  transitions, exact `Reserved` and `TicketIssued` unused proofs against each target's precommitted
  before-state, signed external acknowledgement before pairing-record restoration/removal, and
  expiry/EOF/cancellation/kill boundaries; unreserved/extra/reused/reordered/omitted/cross-
  bootstrap/second-ticket/nonterminal-host-teardown, liveness-only unused proof, missing
  acknowledgement, pre-existing-as-absent, guest-effect-before-consumption, wrong proof or
  acknowledgement key/algorithm/hash/file/parent/fsync/reservation/ticket/pairing-record/dispatch/
  scope/nonce, replay, cross-scope, unknown-field, resulting/future CAS-or-anchor field, and self/
  future-digest negatives; golden construction order is prior state -> proof fsync/hash ->
  acknowledgement fsync/hash -> `ProvenUnused` CAS -> `Retired` CAS;
  exhaustive `PublisherBootstrapComponentRoleV1` platform/
  guest variants, derived targets/types/metadata/dependencies/durability, complete-set golden
  vectors, authorization-level greenfield publisher absence, per-role `ExactAbsentCreate` versus
  `ExactPreExistingDependency` golden vectors, complete sets containing both dispositions,
  created-only retirement membership, and forbidden pre-existing publisher identity, unsupported
  pre-existing dependency, disposition/observation mismatch, omitted/unlisted-parent/container/
  role-target preserving negatives,
  creator-first/two-install last-claim, and legacy
  adoption rejection. Every role variant, parameter type, domain, target, object type, dependency,
  and action in the exhaustive `R3-MANIFEST-01` table has a positive golden vector; unknown roles,
  unlisted actions, caller paths, and every role/target mismatch are preserving negatives.
- Documentation/control surface: exact `R3-DOCS` plus the exact `manifest` review set.
- Checks: common and shell focused tests, `cargo test -p substrate-common`, the new shell
  integration test, shell library regression, format/clippy, schema golden vectors, diff/
  allowlist/secret/change detection. Impact `Cli`, `run_shell_with_cli`, and every edited existing
  symbol; the ordinary `Cli` and `run_shell_with_cli` are frozen, and any new public selector or change to
  `InstallBootstrapContextCarrierV1::validate` is a hard stop.
- Platform/privilege: portable, no privilege or platform mutation. One commit/push. Parser or
  publication uncertainty is terminal-preserving; durability unavailable on a supported
  filesystem uses `BLOCKED_SCOPE_EXPANSION` when new durability authority is required or
  `BLOCKED_CONTRADICTION` when the supported filesystem violates the frozen contract.
- Non-goals/freeze: no consumer integration, recursive subtree, deletion, adoption of observed
  state, IH/PM redesign, or new top-level shared root. Dependency edits are limited to the exact
  root-package keys and lock closure above; every other dependency change is a stop. Pre-R3 collisions
  preserve state and terminate `BLOCKED_SCOPE_EXPANSION`.

### `A1.1d-5R3-LINUX`

**Completion claim.** Implement the Linux managed-system provider and exact pre-state restoration
without editing Unix row-owner orchestrators. Predecessor: `MANIFEST`; successor:
`EVIDENCE:R3-LINUX-IMP-01`.

- Ownership: PI-031, PI-103, `A1D5I-INSTALL-06`, `A1D5I-ENV-01`,
  `R3-ACT-LNX-012`, `R3-ACT-LNX-026`, `R3-ACT-LNX-095`, `R3-ACT-080-LNX`, and
  `R3-ACT-089-LNX`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-linux.rs` symbols `main`,
  `LinuxManagedArtifactExecutorV1`, `open_linux_lifecycle_capsule_v1`,
  `join_linux_role_identity_v1`, `execute_linux_managed_action_v1`,
  `restore_linux_managed_role_v1`, `publish_linux_action_receipt_v1`,
  `open_linux_publisher_protected_state_v1`, and
  `compare_and_swap_linux_publisher_protected_state_v1`; new
  executor symbols `bootstrap_linux_publisher_v1`, `resume_linux_publisher_bootstrap_v1`,
  `run_linux_publisher_v1`, `accept_linux_publisher_connection_v1`,
  `attest_linux_publisher_peer_v1`, `relay_linux_publisher_request_v1`,
  `handle_linux_publisher_request_v1`, `begin_guest_publisher_bootstrap_v1`,
  `open_guest_controlling_tty_v1`, `read_guest_publisher_pairing_confirmation_v1`,
  `verify_guest_publisher_pairing_ticket_v1`, `emit_guest_publisher_bootstrap_hello_v1`,
  `publish_guest_pairing_intent_v1`, `open_guest_pairing_intent_parent_v1`,
  `publish_guest_pairing_intent_otmpfile_v1`, `open_or_join_guest_signing_key_v1`,
  `resume_guest_publisher_pairing_v1`, `verify_guest_publisher_bootstrap_transcript_v1`,
  `join_guest_host_consumption_v1`,
  `commit_guest_publisher_bootstrap_v1`, and
  `retire_linux_test_publisher_v1`; its unsafe/FFI fence is exactly the small wrappers
  `linux_seqpacket_socket_v1`, `linux_peer_credentials_v1`, `linux_openat2_nofollow_v1`,
  `linux_otmpfile_linkat_v1`, and `linux_fsync_parent_v1`, while Ed25519 use is confined to
  `load_or_create_linux_signing_key_v1` and `sign_linux_anchor_v1`; new fixed units
  `scripts/linux/substrate-lifecycle-publisher-v1.service` and
  `scripts/linux/substrate-lifecycle-publisher-v1.socket`; new
  `crates/shell/src/execution/managed_lifecycle/linux_client.rs` replacement symbols
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, `open_linux_seqpacket_channel_v1`, and
  `attest_linux_publisher_response_v1`; `submit_publisher_request_v1` must use the fixed sudo relay
  and never open the root-only publisher socket directly. No other control/core symbol may change;
  new
  `scripts/linux/world-lifecycle.sh` functions `record_linux_managed_state`,
  `install_linux_managed_state`, `restore_linux_managed_state`,
  `invoke_linux_lifecycle_executor`, and its `main` dispatcher; and
  `scripts/linux/world-provision.sh` existing functions
  `prepare_gateway_smoke_auth`, `cleanup_gateway_smoke_auth`,
  `run_gateway_lifecycle_proof`, `ensure_substrate_group_exists`, `ensure_user_in_group`,
  `install_unit`, `verify_socket_acl_bridge`, and `verify_world_deps_acl_bridge`; the exact build
  argv at the current `cargo build -p substrate --bin substrate` sites, extended only with
  `--bin substrate-lifecycle-linux`; `LIFECYCLE_EXECUTOR_BIN_PATH` derivation and executable
  check; plus the exact
  top-level creation/legacy-cleanup range from the first
  `ensure_substrate_group_exists` call through
  `sudo_cmd systemctl start substrate-world-service.service`. That
  range may only be replaced by one call to `world-lifecycle.sh`; the exact
  `SOCKET_UNIT_CONTENT` heredoc removes only `PartOf=substrate-world-service.service` and admits
  no replacement propagation relationship; no installer file beneath
  `scripts/substrate` may change.
- Mutable test/fixture allowlist:
  `tests/installers/world_provision_context_r2_2.sh`,
  `tests/installers/world_provision_smoke.sh`,
  `tests/installers/linux_lifecycle_r3.sh`. Negative cases cover created and
  pre-existing-preserved state plus preserving rejection of schema-reserved `adopted`,
  units, drop-ins, helpers, gateway, sockets, dirs, group/memberships, ACLs, linger, exact
  synthetic-auth A/B mismatch, partial install, retry, repeat restore, and unrelated siblings.
  They prove one protected world and disposable-publisher socket-state/coupled-endpoint prepared
  transaction and receipt,
  endpoint non-requestability, missing/replaced/ambiguous endpoint zero-change, kill/retry at each
  activation/endpoint-observation/receipt boundary, exact absence of service-to-socket propagation,
  and service stop/restart/restore with socket state and endpoint identity unchanged.
  Guest-bootstrap cases use a test host key and TTY harness to prove full fingerprint/challenge/
  literal confirmation, ticket signature/scope/expiry/one-use state, absent/non-TTY rejection,
  malformed/substituted SPKI, wrong curve/algorithm/encoding, SPKI/fingerprint and anchor-key
  mismatch, DER/high-S signatures, request-key/child/channel/nonce/transcript substitution, replay,
  expiry-before-intent rejection versus exact completed-state post-intent resume, root-only
  external unnamed-file/link/fsync intent publication, seed-confined intent/signed-hello/
  transcript/final-key joins, publisher-directory and final-key create/identity-CAS gaps with the
  required terminal-preserving result, and every pairing kill point;
  no automated or stdin confirmation path is accepted.
  Run-only regression paths are `tests/installers/install_state_smoke.sh` and
  `tests/installers/prefix_propagation_r2_2.sh`; their bytes must not change.
- Documentation/control surface: exact `R3-DOCS` and the exact `linux` review set. Evidence bytes
  are not created or committed in this implementation packet.
- Checks: shell syntax/static checks, every listed fixture, non-privileged model/fixture baseline-
  restoration assertions, diff/allowlist/secret/change detection. The fixture directly builds
  `substrate-lifecycle-linux`; world-provision's
  amended build argv produces it, and the first exact-absence bootstrap installs its retained
  root-owned copy before any other role. Native privileged lifecycle, sudo invalidation,
  restoration parity, and the product behavior wall for service/socket/world/gateway plus existing
  filesystem/network/policy/config/dependency/runtime/shim/replay/trace/diagnostic and PTY/
  non-PTY behavior are exclusively `EVIDENCE:R3-LINUX-IMP-01`.
- Platform/privilege: Linux implementation and fixture validation; no native lifecycle action in
  this packet. Exactly one product/test/review commit and one normal fast-forward push. The
  receipt successor is exactly `EVIDENCE:R3-LINUX-IMP-01`; the implementation may not claim
  native evidence or authorize MAC.
- Non-goals/freeze: PI-012/026/095 orchestrator bodies, prefix payload/shims/profiles, macOS,
  Windows, passive health, new account policy, and broad process kill.

### `EVIDENCE:R3-LINUX-IMP-01` and `A1.1d-5R3-LINUX-CLOSEOUT`

The evidence task is read-only with respect to the repository. It discovers the product project
by origin plus exact target ref, requires the live remote, checkout HEAD, commit, and tree all
equal the published `LINUX` receipt, and runs only the Linux IMP actions in `05` on a dedicated
supported sudo host. It emits and validates `codex.top-level-evidence-receipt.v1`; source
`live_remote` must equal source `commit`. The separately validated evidence artifact's exact
`gated_successor` is
`AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT`. Platform absence returns
`BLOCKED_PLATFORM_HANDOFF_REQUIRED`; an available failed/restoration-inexact run returns
`BLOCKED_NATIVE_EVIDENCE`.

`LINUX-CLOSEOUT` starts only from the same published Linux commit after the clean evidence receipt.
Production/test allowlists are empty. It may add exactly
`review-control/r3-linux-imp-01-evidence.json`,
`review-control/r3-linux-imp-01-receipt.json`, the exact `linux-closeout` review set, and append its
status to `R3-DOCS`; it validates the evidence receipt again with the orchestration skill. One
documentation/evidence commit and one fast-forward push are allowed. Its receipt successor is
exactly `AUTHORITY_REQUIRED:A1.1d-5R3-MAC`.

### `A1.1d-5R3-MAC`

**Completion claim.** Implement manifest-bound Lima staging/teardown and activate only the
already-fixed PM-bound SSH-UDS path after exact unlink/timeout/drop/retry ownership. Predecessor:
`LINUX-CLOSEOUT`; successor: `EVIDENCE:R3-MAC-IMP-01`.

- Ownership: PI-051, PI-098, PI-099, PI-100, PI-101, PI-113, PI-114,
  `A1D5I-INSTALL-08`, `R3-ACT-MAC-012`, and `R3-ACT-MAC-026`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-macos.rs` symbols `main`,
  `MacManagedArtifactExecutorV1`, `open_mac_lifecycle_capsule_v1`,
  `join_mac_role_identity_v1`, `execute_mac_managed_action_v1`,
  `restore_mac_managed_role_v1`, `publish_mac_action_receipt_v1`,
  `MacLifecyclePublisherServiceV1`, `open_system_keychain_protected_state_v1`, and
  `compare_and_swap_mac_publisher_protected_state_v1`, `bootstrap_mac_publisher_v1`,
  `export_mac_p256_spki_der_v1`, `normalize_mac_p256_signature_p1363_low_s_v1`,
  `resume_mac_publisher_bootstrap_v1`, `run_mac_xpc_publisher_v1`,
  `accept_mac_xpc_connection_v1`, `attest_mac_xpc_audit_token_v1`,
  `verify_mac_control_designated_requirement_v1`, `handle_mac_publisher_request_v1`, and
  `issue_lima_guest_pairing_ticket_v1`, `consume_lima_guest_pairing_ticket_v1`,
  `open_mac_guest_pairing_record_v1`, `compare_and_swap_mac_guest_pairing_record_v1`,
  `prove_lima_guest_reservation_unused_v1`,
  `commit_lima_guest_reservation_unused_acknowledgement_v1`,
  `retire_mac_test_publisher_v1`, `publish_lima_stage_one_intent_v1`,
  `attach_lima_stage_one_machine_identity_v1`, and `close_lima_stage_one_intent_v1`; its
  unsafe/FFI fence is exactly
  `mac_xpc_listener_ffi_v1`, `mac_audit_token_ffi_v1`, `mac_security_key_ffi_v1`,
  `mac_keychain_anchor_ffi_v1`, and `mac_atomic_file_ffi_v1`, with link declarations only for
  `Security`, `CoreFoundation`, and the system XPC library; new
  `crates/shell/src/execution/managed_lifecycle/macos_client.rs` replacement symbols
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`, `open_mac_xpc_channel_v1`, and
  `attest_mac_publisher_response_v1`, `issue_guest_publisher_pairing_ticket_v1`; no other
  control/core symbol may change; new fixed plist
  `scripts/mac/com.substrate.lifecycle.publisher.v1.plist`; new
  `scripts/mac/lima-lifecycle.sh` functions `load_mapped_lifecycle_v1`,
  `invoke_mac_lifecycle_executor`, `install_mapped_lima_state_v1`,
  `restore_mapped_lima_state_v1`, `install_mac_publisher_v1`,
  `bootstrap_lima_guest_publisher_v1`, `retain_lima_guest_bootstrap_channel_v1`,
  `display_lima_guest_pairing_challenge_v1`,
  `join_lima_guest_bootstrap_transcript_v1`,
  `record_lima_guest_pre_state_v1`, `install_exact_guest_artifacts_v1`,
  `persist_lima_guest_unused_proof_v1`, `acknowledge_lima_guest_unused_proof_v1`,
  `restore_lima_guest_state_v1`, `retire_lima_guest_test_publisher_v1`, and `main`; existing `scripts/mac/lima-stop.sh` parameter/intake
  surface may add required prefix, `InstallBootstrapContextCarrierV1`, and
  `PlatformBootstrapMappingV1` arguments plus `resolve_lima_stop_authority_v1` and
  `invoke_mapped_lima_stop_v1`; its complete executable body after `set -euo pipefail`, including
  the first `limactl list substrate` through final `fi`, is replaced by one exact mapped-executor
  request. No hard-coded/default instance remains on the destructive path;
  `scripts/mac/lima-warm.sh` functions
  `destroy_vm`, `ensure_vm_ready`, `ensure_substrate_group`, `stage_workspace`,
  `install_agent_from_host`, `install_cli_from_host`, `install_gateway_from_host`,
  `build_missing_components_inside_vm`, `install_guest_binaries`,
  `bootstrap_guest_private_home`, `write_systemd_units`, `enable_socket_activation`,
  `write_layout_sentinel`, and `configure_guest`; the exact `configure_guest` mutation range from
  `ensure_substrate_group` through `write_layout_sentinel` is replaced by one
  `lima-lifecycle.sh` call. `build_missing_components_inside_vm` and its nested `fix_dns`, package,
  Rustup, Cargo-build, `/etc/resolv.conf`, and DNS-service mutation branches are tombstoned; absence
  of an exact `ExecutorBuildEvidenceV1`-joined guest artifact stops before guest mutation. Existing mapping observation,
  check-only/status, socket summary, and linger guidance remain read-only and frozen;
  `scripts/mac/lima/units/substrate-world-service.socket` removes exactly
  `PartOf=substrate-world-service.service` and admits no replacement propagation relationship;
  `crates/world-mac-lima/src/forwarding.rs` symbols
  `ForwardingHandle`, `ForwardingHandle::drop`, `ForwardingKind::SshUds`,
  `create_ssh_uds_forwarding`, and new
  `MappedSshUdsAttemptV1`, `create_mapped_ssh_uds_forwarding_v1`,
  `record_mapped_known_hosts_entry_v1`, and `restore_mapped_known_hosts_entry_v1`; and
  `crates/world-mac-lima/src/lib.rs` symbols `MacLimaBackend::ensure_forwarding`,
  `ensure_session_setup`, `get_agent_endpoint`, and removal of the exact
  `r3_forwarding_activation_required` gate. `new_with_mapping`, mapping validation,
  `auto_select`, VSock, TCP, and ambient selectors are frozen.
- Mutable test/fixture allowlist: embedded tests in
  `crates/world-mac-lima/src/forwarding.rs` and
  `crates/world-mac-lima/src/lib.rs`,
  `tests/mac/installer_parity_fixture.sh`, and new `tests/mac/lifecycle_r3.sh`. Run-only regression
  paths are `tests/mac/prefix_mapping_r2_3.sh` and `tests/mac/lima_doctor_fixture.sh`; their bytes
  must not change. Required negatives
  cover exact socket/replacement/symlink identity, `StreamLocalBindUnlink`, timeout kill/wait,
  Drop teardown, handle loss, retry, partial staging/unit/socket state, pre-existing VM,
  protected host-publisher plus guest world/publisher service-state/coupled-endpoint prepared
  transactions and receipts, endpoint
  non-requestability, missing/replaced/ambiguous endpoint zero-change, kill/retry at each activation/
  endpoint-observation/receipt boundary, exact absence of service-to-socket propagation, and
  service stop/restart/restore with socket state and endpoint identity unchanged,
  absent-instance stage-one authorization/create/finalize-PM kill points and preserving ambiguous
  create/finalize failure,
  group/membership/private-home/layout-sentinel/service-state and A-local known-hosts restoration,
  missing or build-evidence-mismatched artifact with zero mutation, prohibited DNS/toolchain/package remediation,
  host-key fingerprint/challenge confirmation only through independent host and guest controlling
  terminals; signed ticket PM/machine/artifact scope, expiry, one-use consumption, exact retry,
  wrong/automated/non-TTY confirmation, malformed/substituted SPKI, wrong curve/encoding,
  SPKI/fingerprint/anchor mismatch, high-S signature, alternate child/key/channel/transcript,
  durable host-record/guest-intent restart, pairing kill points, exact pre-existing versus created
  Lima guest state-root/lifecycle-container/publisher-directory/executor-parent roles, root create/
  identity-CAS preserving gaps, reserved challenge/component tuple consumption; both `Reserved`
  and `TicketIssued` unused revocation with exact pre-existing/absent target-before-state,
  protected-host P-256 proof, external file/parent fsync/hash, harness-Ed25519 acknowledgement,
  Keychain pairing-record restoration/removal, and kill points; liveness/timeout/pathname proof,
  missing/wrong acknowledgement, cross-reservation/replay/file/parent/fsync/digest negatives;
  reverse created-empty parent retirement after guest acknowledgement, and final guest parity,
  preserving rejection of schema-reserved `adopted`,
  unrelated Lima instance/state, and no alternate transport.
- Documentation/control surface: exact `R3-DOCS` and the exact `mac` review set. Evidence bytes
  are not created or committed in this implementation packet.
- Checks: focused crate tests, mac fixture scripts, non-native lifecycle-model/restoration
  assertions, available target static-check only, diff/allowlist/secret/change detection. IMP
  fixtures use a non-executing mock artifact identity; only `EVIDENCE:R3-MAC-IMP-01` may natively
  build/code-sign the host executor and build the Linux guest executor from the exact published
  source under `ExecutorBuildEvidenceV1` before baseline. It supplies those exact digests to the
  disposable exact-absence publisher bootstrap; guest bootstrap additionally requires a live
  operator to pin the full host-key fingerprint/challenge between independent host and guest
  controlling TTYs. Absence of that operator/TTY boundary is
  `BLOCKED_PLATFORM_HANDOFF_REQUIRED`. Final release/dev
  archive staging remains UNIX-owned. Native supported-Lima publisher/lifecycle/product/
  restoration proof is exclusively `EVIDENCE:R3-MAC-IMP-01`. Treat the graph-visible LOW results
  for private/cfg/Drop symbols as manual HIGH review surfaces; any edit to HIGH
  `new_with_mapping` or any new selector/caller stops.
- Platform/privilege: implementation and non-native fixtures only; publisher installation and
  Lima lifecycle are deferred to the evidence task. Exactly one product/test/review commit and one
  normal fast-forward push. Receipt successor is exactly `EVIDENCE:R3-MAC-IMP-01`.
- Non-goals/freeze: Unix uninstall bodies, Linux, Windows, ambient compatibility activation,
  alternate/newly selected VM, endpoint, transport, principal, or prefix, and unowned VM deletion.
  The sole absent-instance create path is the already-selected R2 Stage-1 identity under
  `LimaStageOneAuthorizationV1`.

### `EVIDENCE:R3-MAC-IMP-01` and `A1.1d-5R3-MAC-CLOSEOUT`

The evidence task discovers a native supported macOS checkout whose HEAD/tree/ref and live remote
equal the published `MAC` receipt, then runs only the MAC IMP actions in `05`, including exact
publisher bootstrap, human-pinned signed guest pairing, and externally receipted test-retirement
back to LaunchDaemon/System-Keychain
baseline. It changes no repository byte and validates a
`codex.top-level-evidence-receipt.v1` whose source remote equals its source commit. The separately
validated artifact's exact `gated_successor` is
`AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT`; platform absence and failed native proof map to the
same two platform statuses as Linux.

`MAC-CLOSEOUT` has empty production/test allowlists. It may add exactly
`review-control/r3-mac-imp-01-evidence.json`,
`review-control/r3-mac-imp-01-receipt.json`, the exact `mac-closeout` review set, and append its
status to `R3-DOCS`. It revalidates the evidence receipt, makes one documentation/evidence commit,
and performs one fast-forward push. Its receipt successor is exactly
`AUTHORITY_REQUIRED:A1.1d-5R3-WIN`.

### `A1.1d-5R3-WIN`

**Completion claim.** Implement exact A-local versus SID+instance+machine-ID+pipe shared Windows
lifecycle and convergence. Predecessor: `MAC-CLOSEOUT`; successor:
`EVIDENCE:R3-WIN-IMP-01`.

- Ownership: PI-047, PI-053, PI-096, PI-097, PI-102,
  `A1D5I-INSTALL-07`, `A1D5I-INSTALL-10`, `R3-ACT-049-WIN`, and `R3-ACT-070-WIN`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-windows.rs` symbols `main`,
  `WindowsManagedArtifactExecutorV1`, `open_windows_lifecycle_capsule_v1`,
  `join_windows_role_identity_v1`, `execute_windows_managed_action_v1`,
  `restore_windows_managed_role_v1`, `publish_windows_action_receipt_v1`,
  `WindowsLifecyclePublisherServiceV1`, `open_machine_protected_state_v1`, and
  `compare_and_swap_windows_publisher_protected_state_v1`, `bootstrap_windows_publisher_v1`,
  `export_windows_p256_spki_der_v1`, `normalize_windows_p256_signature_p1363_low_s_v1`,
  `resume_windows_publisher_bootstrap_v1`, `run_windows_named_pipe_publisher_v1`,
  `accept_windows_publisher_client_v1`, `impersonate_windows_publisher_client_v1`,
  `attest_windows_publisher_client_image_v1`, `handle_windows_publisher_request_v1`,
  `issue_wsl_guest_pairing_ticket_v1`, `consume_wsl_guest_pairing_ticket_v1`,
  `open_windows_guest_pairing_record_v1`,
  `compare_and_swap_windows_guest_pairing_record_v1`,
  `prove_wsl_guest_reservation_unused_v1`,
  `commit_wsl_guest_reservation_unused_acknowledgement_v1`,
  `issue_wsl_guest_bootstrap_v1`, `join_wsl_guest_anchor_v1`, and
  `retain_wsl_guest_bootstrap_channel_v1`, `join_wsl_guest_bootstrap_transcript_v1`, and
  `retire_windows_test_publisher_v1`, `retire_wsl_guest_test_publisher_v1`; its unsafe/Win32 fence is exactly
  `windows_named_pipe_ffi_v1`, `windows_client_attestation_ffi_v1`,
  `windows_cng_key_ffi_v1`, `windows_registry_anchor_ffi_v1`,
  `windows_service_control_ffi_v1`, and `windows_flush_file_ffi_v1`;
  `crates/shell/src/execution/managed_lifecycle/windows_client.rs` replacement symbols
  `bootstrap_publisher_v1`, `submit_publisher_request_v1`,
  `open_windows_publisher_pipe_v1`, `attest_windows_publisher_response_v1`, and
  `issue_guest_publisher_pairing_ticket_v1`; no other
  control/core symbol may change;
  `scripts/windows/install-substrate.ps1` new
  `Open-CurrentAttemptTempRootV1`, `Register-CurrentAttemptTempMemberV1`,
  `Expand-CurrentAttemptTempArchiveV1`, `Rollback-CurrentAttemptTempRootV1`, and
  `Install-ManagedWindowsPrefixV1`, replacing exactly the top-level range from
  `$tempRoot = Join-Path ([System.IO.Path]::GetTempPath())` through the closing brace of the
  existing `finally` block. The replacement acquires and retains the temp parent/root handles,
  registers the checksum, payload, bundle, and every extracted descendant before bytes are
  published, and invokes `CurrentAttemptTempRollbackV1` on every handled-error/finally exit; only
  after that safe-rollback fence may it route version/bin/profile/shim/WSL provisioning and doctor
  mutations through the typed publisher. No path-only `Remove-Item -Recurse` remains;
  `scripts/windows/dev-install-substrate.ps1` new
  `Install-ManagedWindowsDevPrefixV1` replacing exactly the top-level range from the
  `Get-Command cargo` prerequisite through `Write-Log 'Substrate dev install complete.'`,
  including build argv, shim deployment, and profile-helper creation;
  `scripts/windows/uninstall-substrate.ps1` new `Invoke-ManagedWindowsUninstallV1` replacing the
  top-level range from `$forwarderPidPath =` through the final WSL unregister block; the replacement
  removes the unregister action entirely and returns a preserving scope-expansion result if a
  caller requests instance/install-tree deletion;
  `scripts/windows/dev-uninstall-substrate.ps1` new `Invoke-ManagedWindowsDevUninstallV1`
  replacing the top-level range from `$repoRoot =` through the final dev-uninstall status block,
  including the existing `--shim-remove` invocation;
  `scripts/windows/wsl-stop.ps1` parameter/intake surface adding mandatory `InstallPrefix`,
  `InstallBootstrapContextV1`, `PlatformBootstrapMappingV1`, and committed `PipePath`, plus new
  `Resolve-ManagedWindowsStopAuthorityV1`, `Assert-ManagedWindowsStopAuthorityV1`, and
  `Stop-ManagedWindowsPlatformV1`; the complete destructive body from the first status line through
  final pipe observation is replaced by one request. The broad process-name scan is tombstoned and
  distro termination is allowed only for the exact already-registered PM identity; and
  `scripts/windows/wsl-warm.ps1` new `Start-ManagedWarmForwarderV1` replacing the top-level range
  from `$logDir = Join-Path $env:LOCALAPPDATA 'Substrate\\logs'` through the final forwarder
  capabilities probe. The replacement derives both log directory and PID record only from the
  already-validated PM (`MappingState.ForwarderLogDir` and `MappingState.SharedForwarderPidPath`),
  manifest-records the exact log-directory before-state before creation, and includes stale-PID
  handling and PID-record publication; new `Save-WslGuestUnusedProofV1` and
  `Confirm-WslGuestUnusedProofV1` persist only to the precommitted external harness parent and
  accept only the exact LocalSystem-signed proof/HKLM reservation CAS plus the precommitted
  harness acknowledgement; new
  `Enter-ManagedExistingWslLifecycleV1` replacing only the unconditional fail-closed call
  immediately after the already-landed mapping assertion; it requires that mapping's exact
  already-registered distro and machine ID. `Reject-UnmanagedWslCreationV1` replaces the complete
  absent-distro import branch and returns `BLOCKED_SCOPE_EXPANSION` without download, directory
  creation, import, or cleanup. New `Provision-ManagedExistingWslV1` replaces the complete
  provisioning range from the provisioning-script lookup through both the unhealthy/force branch
  and the healthy binary-refresh branch's closing brace; it is syntactically closed through the
  current final `Install-GuestWorldBinaries` call and may act only through joined host/guest
  publisher requests. The mapping resolver/assertion and every selector are frozen. New
  `scripts/wsl/units/substrate-world-service.service.tmpl` and
  `scripts/wsl/units/substrate-world-service.socket` contain only the byte-exact canonical
  templates frozen in `04`; no other unit generator/template path is allowed. New
  `scripts/wsl/provision.sh` bounded replacement of its unconditional live guard/body with
  `render_managed_wsl_world_service_unit_v1` and
  `invoke_managed_wsl_guest_lifecycle_v1` is allowed only after the Windows publisher has issued a
  single-use host-signed pairing ticket, the exact staged Linux executor has been verified
  against the source receipt, an operator has pinned the host-key fingerprint/challenge through
  the guest controlling TTY, and the guest root publisher has joined that ticket/transcript; no ambient
  guest selection or package/passive-health remediation is allowed; and
  `scripts/windows/start-forwarder.ps1` new `Start-ManagedForwarderV1` replacing the range from
  `$logDir = $mappingState.ForwarderLogDir` through both readiness and wait-mode terminal
  branches. It records or joins `windows.prefix.forwarder-log-directory` before creation and then
  starts/stops only the exact manifest-bound child; all R2 mapping construction, validation,
  assertion, and selection logic before this range remains frozen.
- Mutable test/fixture allowlist:
  `scripts/windows/prefix-mapping-r2-3.Tests.ps1`,
  `crates/shell/tests/installer_env_wcu4.rs`, new
  `scripts/windows/dev-lifecycle-r3.Tests.ps1`, and new
  `scripts/windows/lifecycle-r3.Tests.ps1`. Required cases cover default/custom A under hostile
  B, two prefixes, unrelated `.substrate*`, reparse points, missing/malformed/tampered/stale/
  cross-SID/cross-instance/partial manifests, version/bin/profile/shim identity, PID/image/start
  identity, pipe scope, timeout/stop/uninstall order, partial install, repeat lifecycle, other
  distros, exact existing-instance start/stop/restoration, preserving rejection of WSL import,
  install-tree deletion, or unregister; exact A-local forwarder-log before-state/restoration under
  hostile `LOCALAPPDATA`; pre-existing/replaced/reparse log directories; and
  `CurrentAttemptTempRollbackV1` kill points at root create/open, every checksum/bundle/payload/
  extracted-child registration and write, reverse removal, and empty-root proof without reparse
  traversal; plus pairing ticket PM/machine/artifact scope, full fingerprint/challenge/literal
  confirmation on independent host/guest terminals, one-use consumption/exact retry, non-TTY or
  automated confirmation rejection, malformed/substituted SPKI, wrong curve/encoding,
  SPKI/fingerprint/anchor mismatch, high-S signature, alternate child/key/channel/transcript,
  durable host-record/guest-intent restart, and every pairing kill point; exact pre-existing versus
  created WSL `/run/substrate`, `/var/lib/substrate`, lifecycle-container, publisher-directory, and
  executor-parent roles; exact `substrate` group, PM-mapped guest membership, and
  root:`substrate` `0660` `/run/substrate.sock` create/readback/restore order; distinct
  service-template-source digest, exact PM render inputs, rendered-service digest, socket-source
  digest, and unchanged installed-socket digest; PM-bound template substitution/escaping,
  installed-template-marker and rendered/modified-socket rejection, hostile ambient HOME/context/mapping,
  root:root `0644` unit metadata, absence of implicit directory-management directives and proof
  that activation does not mutate either root/lifecycle subtree, wrong path/content/owner/mode
  preservation, daemon-reload-before-state ordering; one protected prepared transaction binding
  the socket service-state and its fixed coupled endpoint before-states before activation, exact
  endpoint identity/ACL observation in the same receipt, no independently requestable endpoint
  action, kill/retry at every activation/observation/receipt boundary, exact absence of
  service-to-socket propagation, and service stop/restart/restore with socket state and endpoint
  identity unchanged; the LocalSystem publisher and WSL guest publisher prove the same protected
  service-state/coupled-endpoint rule inside bootstrap/retirement; root pending/create/identity-CAS
  kill gaps; replacement/nonempty/wrong owner/mode/group/membership/socket identity or
  ACL preservation; both `Reserved` and `TicketIssued` challenge/component tuple unused
  revocation, the LocalSystem-signed proof and its externally fsynced hash, the precommitted
  harness acknowledgement and its externally fsynced hash, the exact HKLM reservation CAS, and
  pairing-record before-state restoration/removal, including replay, cross-reservation,
  missing-acknowledgement, and future/resulting-CAS-or-anchor-digest rejection; guest receipt/
  acknowledgement before reverse created-empty parent removal; and final WSL baseline parity.
- Documentation/control surface: exact `R3-DOCS` and the exact `win` review set. Evidence bytes
  are not created or committed in this implementation packet.
- Checks: Pester/static tests, shell installer static integration, non-native lifecycle-model and
  restoration assertions, diff/allowlist/secret/change detection. IMP fixtures use non-executing
  mock artifact identities. Only `EVIDENCE:R3-WIN-IMP-01` may natively build the Windows executor
  and build the Linux guest executor from the exact published source under
  `ExecutorBuildEvidenceV1`; publisher bootstrap verifies and copies that exact hash to the
  LocalSystem service identity before any other role. Guest bootstrap additionally requires a
  live operator to pin the full host-key fingerprint/challenge between independent host and guest
  controlling TTYs; absence is `BLOCKED_PLATFORM_HANDOFF_REQUIRED`. Native Windows/WSL lifecycle/product/
  restoration proof is
  exclusively `EVIDENCE:R3-WIN-IMP-01`. Final cargo-dist archive wiring remains UNIX-owned.
  PowerShell/script actions
  require manual caller closure because GitNexus under-resolves them.
- Platform/privilege: implementation and static/portable fixtures only; LocalSystem publisher and
  WSL lifecycle are deferred to the evidence task. Exactly one product/test/review commit and one
  normal fast-forward push. Receipt successor is exactly `EVIDENCE:R3-WIN-IMP-01`.
- Non-goals/freeze: any change to the existing R2 WSL mapping resolver/assertion/selector,
  missing-distro import or unregister, unmanifested WSL adoption, wildcard/default/ambient
  selection, process-name or stale-PID authority,
  Unix/Linux/macOS, policy redesign.

### `EVIDENCE:R3-WIN-IMP-01` and `A1.1d-5R3-WIN-CLOSEOUT`

The evidence task discovers a native supported Windows/WSL checkout whose HEAD/tree/ref and live
remote equal the published `WIN` receipt. It runs only the WIN IMP actions in `05`, including
LocalSystem/CNG/HKLM publisher bootstrap, human-pinned signed guest pairing, and externally
receipted test-retirement to baseline,
changes no repository byte, and validates
`codex.top-level-evidence-receipt.v1`. The separately validated artifact's exact
`gated_successor` is
`AUTHORITY_REQUIRED:A1.1d-5R3-WIN-CLOSEOUT`; absence and failure use the closed platform statuses.

`WIN-CLOSEOUT` has empty production/test allowlists. It may add exactly
`review-control/r3-win-imp-01-evidence.json`,
`review-control/r3-win-imp-01-receipt.json`, the exact `win-closeout` review set, and append its
status to `R3-DOCS`. It revalidates the native receipt, makes one documentation/evidence commit,
and performs one fast-forward push. Its receipt successor is exactly
`AUTHORITY_REQUIRED:A1.1d-5R3-UNIX`.

### `A1.1d-5R3-UNIX`

**Completion claim.** Convert Unix prefix/shim/payload/profile install, replacement, and uninstall
to the manifest core and integrate only the already-landed Linux/macOS providers. This packet owns
the row-level convergence of bundled PI-012/026/095. Predecessor: `WIN-CLOSEOUT`; successor:
`EVIDENCE:R3-NATIVE-LINUX-01`.

- Ownership: PI-012, PI-026, PI-064, PI-074, PI-092, PI-093, PI-094, PI-095,
  `A1D5I-INSTALL-02`, `A1D5I-INSTALL-03`, `R3-ACT-007-RB`, and `R3-ACT-019-RB`.
- Production allowlist: new typed executor
  `src/bin/substrate-lifecycle-unix.rs` symbols `main`,
  `UnixManagedArtifactExecutorV1`, `open_unix_lifecycle_capsule_v1`,
  `join_unix_role_identity_v1`, `execute_unix_managed_action_v1`,
  `restore_unix_managed_role_v1`, and `publish_unix_action_receipt_v1`;
  `Cargo.toml` only the exact `[package.metadata.dist.binaries]."*"` value, adding
  `substrate-lifecycle-control`, `substrate-lifecycle-linux`, `substrate-lifecycle-macos`,
  `substrate-lifecycle-windows`, and `substrate-lifecycle-unix` to the existing two names;
  `scripts/substrate/dev-install-substrate.sh` functions
  `write_host_state_metadata`, `stage_managed_bundle_symlink`,
  `stage_managed_linux_binary_copy`, `clear_managed_prefix_linux_binary_cache`, and
  `cleanup_legacy_world_enable_helper_bridge`, plus new
  `append_platform_lifecycle_build_flags_v1`, `stage_lifecycle_executors_v1`, and
  `run_managed_dev_install_v1`; the exact top-level range from
  `TARGET_DIR="${PROFILE}"` through the final `write_host_state_metadata` call is replaced by the
  one orchestrator call, so build argv, config/payload/bin/shim/cache/profile creation and both
  provider integrations cannot bypass the manifest;
  `scripts/substrate/dev-uninstall-substrate.sh` existing
  `kill_live_dev_owner_helpers`, `remove_managed_symlink`,
  `remove_managed_prefix_linux_binary_copies`, `load_host_state_metadata`,
  `perform_auto_cleanup`, and new `run_managed_dev_uninstall_v1`, which replaces exactly the
  top-level range from `kill_live_dev_owner_helpers` through the final
  `perform_auto_cleanup "${cleanup_user}"` call; `kill_live_dev_owner_helpers` and the public
  `--kill-live-processes` branch are tombstoned into a preserving
  `BLOCKED_SCOPE_EXPANSION` result before enumeration or signal. Hidden owner-helper launch/runtime
  authority belongs to the excluded retained-worker/session program and is not widened here; R3
  never infers or kills that process. Uninstall proceeds only when the separately observed exact
  lifecycle prerequisite says no live owner helper consumes the prefix;
  `scripts/substrate/install-substrate.sh` functions
  `prepare_tmpdir`, `cleanup`, `write_host_state_metadata`, `prepare_bundle_payload`,
  `link_binaries`, `deploy_shims`,
  `harden_shim_symlinks`, `provision_linux_world`, and new
  `install_managed_release_prefix_v1`; in `install_macos` and `install_linux`, the exact range from
  each `prepare_bundle_payload` call through its final `write_host_state_metadata` call is replaced
  by one orchestrator call, including payload/version/bin/executor/shim/profile/provider staging;
  `provision_linux_world` removes exactly `PartOf=substrate-world-service.service` from its socket
  source and adds no replacement propagation relationship; `prepare_tmpdir`, every
  `prepare_bundle_payload` child-creation/extraction call, and `cleanup`
  jointly implement `CurrentAttemptTempRollbackV1`: retain parent/root descriptors and physical
  identities before the first download, register/rejoin the closed descendant set, and perform
  reverse descriptor-relative removal. Pre-existing, replaced, symlinked, unregistered, non-
  joined, or ambient `TMPDIR` is never recursively removed;
  `scripts/substrate/uninstall-substrate.sh` existing `remove_path_snippet`,
  `remove_shell_path_snippets`, `load_host_state_metadata`, `perform_auto_cleanup`, and new
  `run_managed_release_uninstall_v1`, which replaces exactly the top-level range from
  `log "Stopping substrate processes (if any)..."` through the final
  `perform_auto_cleanup "${cleanup_user}"` call;
  `scripts/substrate/dev-shim-bootstrap.sh::{install_shims,write_env_file,uninstall_shims,
  remove_env_file}`;
  `crates/shell/src/execution/shim_deploy.rs::{ShimDeployer::with_context,ensure_deployed,
  deploy_shims,migrate_old_shims}`; and
  `crates/shell/src/execution/invocation/plan.rs` new
  `remove_managed_shims_v1` plus only the `cli.shim_remove` callsite inside
  `ShellConfig::from_cli`.
- Mutable test/fixture allowlist:
  `crates/shell/tests/shim_deployment.rs`,
  `tests/installers/dev_shim_bootstrap_context_r2_1.sh`,
  `tests/installers/install_state_smoke.sh`,
  `tests/installers/prefix_propagation_r2_1.sh`,
  `tests/installers/prefix_propagation_r2_2.sh`,
  `tests/installers/install_smoke.sh`, and new
  `tests/installers/unix_lifecycle_r3.sh`. Required cases cover first/repeat/partial/retry install,
  first/repeat uninstall, uninstall after failure, reinstall, hostile B/two-home/two-prefix,
  link/target and profile bytes/metadata, closed subtree, unrelated sibling, and every manifest
  rejection class, plus preserving rejection of `--kill-live-processes` before process
  enumeration/signal and `CurrentAttemptTempRollbackV1` kill points before/after root creation/open,
  every child registration/write/extract, reverse removal, and empty-root proof; generated Linux
  socket bytes omit service-to-socket propagation, and service stop/restart/restore leaves the
  independently owned socket state and endpoint identity unchanged.
  `tests/mac/installer_parity_fixture.sh`
  is a run-only regression path owned for
  mutation by `MAC`; its bytes must not change here.
- Documentation/control surface: exact `R3-DOCS` plus the exact `unix` review set.
- Checks: all listed fixtures, focused and broad shell tests, format/clippy, non-privileged install
  matrix, provider-integration dry fixtures, `cargo dist plan` and archive-content checks proving
  every target contains the five lifecycle binaries (wrong-platform binaries exit before parsing
  or mutation), exact dev build/staging assertions,
  diff/allowlist/secret/change detection. GitNexus
  currently reports `ShellConfig::from_cli` HIGH (8 direct/10 total graph-visible) and
  `ShimDeployer::ensure_deployed` at least MEDIUM under full caller closure; any edit outside the
  shim action block or new production caller is a stop.
- Platform/privilege: Unix implementation, no native platform mutation in non-native checks.
  Native effects are deferred to the final three evidence tasks. One commit/push. The receipt
  successor is `EVIDENCE:R3-NATIVE-LINUX-01`; the two subsequent evidence tasks are separately
  authorized and all three bind this same published UNIX checkpoint.
- Non-goals/freeze: internals of the Linux/macOS providers, Windows, private-home rollback,
  recursive unmanifested removal, broad kill, ambient-home cleanup, passive health.

### Final native evidence task sequence

`EVIDENCE:R3-NATIVE-LINUX-01`, `EVIDENCE:R3-NATIVE-MAC-01`, and
`EVIDENCE:R3-NATIVE-WIN-01` are three read-only repository tasks with the platform/privilege,
action, proof, restoration, and artifact contracts in `05`. Each independently discovers a clean
checkout whose HEAD/tree/ref and live remote equal the exact published `UNIX` receipt; none may
use the preceding evidence task as a changed source checkpoint. Each validates a native
`codex.top-level-evidence-receipt.v1` with the orchestration skill's evidence validator. Their
successors are respectively `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-MAC-01`,
`AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-WIN-01`, and
`AUTHORITY_REQUIRED:A1.1d-5R3-CLOSEOUT`. They change no repository byte and create no commit or
push. Platform absence returns `BLOCKED_PLATFORM_HANDOFF_REQUIRED` with the full handoff; a run
failure or restoration mismatch returns `BLOCKED_NATIVE_EVIDENCE`.

### `A1.1d-5R3-CLOSEOUT`

**Completion claim.** Ingest and revalidate the three clean remote-equal native evidence receipts
for the exact landed `UNIX` checkpoint, run only non-mutating cross-platform regression/review
walls, close all R3 findings/gates, and update documentation/review control. No production or test
behavior may change. Predecessors: all three final native evidence tasks; successor:
`AUTHORITY_REQUIRED:A1.1d-CLOSEOUT`.

- Ownership: `A1D5I-REG-01`; final joins for `RG-HOME-01`, `RG-INSTALL-01`, `R3-LIFE-01`,
  `R3-WIN-01`; verification only for every R3 PI/finding.
- Production/test allowlist: empty. Tests and native smokes may run but no source/test/fixture
  byte may change.
- Documentation/control allowlist: exact `R3-DOCS`,
  `llm-last-mile/runtime-refactor/01-target-architecture.md`,
  `review-control/r3-closeout-review-cycle-record.json`,
  `review-control/r3-closeout-review-authority-security.md`,
  `review-control/r3-closeout-review-lifecycle-convergence.md`,
  `review-control/r3-closeout-review-allowlist-evidence.md`,
  `review-control/r3-native-linux-01-evidence.json`,
  `review-control/r3-native-linux-01-receipt.json`,
  `review-control/r3-native-mac-01-evidence.json`,
  `review-control/r3-native-mac-01-receipt.json`,
  `review-control/r3-native-win-01-evidence.json`, and
  `review-control/r3-native-win-01-receipt.json`.
- Gates: `R3-NATIVE-LINUX-01`, `R3-NATIVE-MAC-01`, and `R3-NATIVE-WIN-01` as specified in `05`;
  broad filesystem/network/policy/config/dependency/gateway/runtime/shim/replay/trace/diagnostic
  and PTY/non-PTY regression; installer/uninstaller symmetry; exact restoration; independent
  bounded review CLEAN with zero unresolved P1-P4.
- Publication: one docs/evidence commit and one normal fast-forward push only after live target
  revalidation. Missing platform or evidence yields `BLOCKED_PLATFORM_HANDOFF_REQUIRED`;
  mismatch, incomplete restoration, or open finding blocks publication. No weakened/static
  substitute is accepted.
- Non-goals/freeze: every production symbol, test, fixture, script, dependency, schema, generated
  product artifact, platform selection, passive health, and later runtime-refactor packet.

## R3 implementation status append

This status append is preserved historical chronology. Its MAC evidence/recovery successors remain
outside the active schedule under `AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY`; nothing in this
append authorizes protected-lifecycle, Windows, or evidence work.

### `A1.1d-5R3-MANIFEST`

`A1.1d-5R3-MANIFEST` is landed and closes the packet contract above without widening scope. The
landed subject is the exact allowlisted common/shell/root/CI surface for canonical managed-artifact
types, canonical encoders/validators, manifest publication/head/index helpers, hidden
`substrate-lifecycle-control`, platform-provider preserving stubs, and the native-evidence
validator. The packet remains non-destructive, records only manifest/bootstrap authority, and
leaves the next implementation authority at `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX`.

### `A1.1d-5R3-LINUX`

`A1.1d-5R3-LINUX` is landed and closes the Linux provider contract above without widening scope.
The landed subject is the exact Linux executor/client surface, fixed publisher service/socket
units, bounded `world-provision.sh` replacement with one `world-lifecycle.sh` call, and focused
fixtures proving created versus pre-existing state preservation, protected/disposable publisher
transactions, endpoint non-requestability, service/socket non-propagation, and guest bootstrap
pairing joins. Unix row-owner orchestrators remain frozen, the packet claims no native evidence,
and the next authority is `EVIDENCE:R3-LINUX-IMP-01`.

### `A1.1d-5R3-LINUX-CLOSEOUT`

`A1.1d-5R3-LINUX-CLOSEOUT` is landed and closes the Linux evidence-closeout contract above without
widening scope. The landed subject is the exact copy of
`review-control/r3-linux-imp-01-evidence.json` and
`review-control/r3-linux-imp-01-receipt.json`, the exact `linux-closeout` review set, and this
bounded `R3-DOCS` status append. Before publication, the closeout revalidated
`EVIDENCE:R3-LINUX-IMP-01` against source commit `fef5bf688ade61bfaf40e43d21fb77ae492fa5fe`,
tree `827e88f2c069cd27a04e99a57894bd5a753b2e55`, ref
`refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`, and gated successor
`AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT`, then revalidated the separate evidence receipt.
No production or test bytes changed, no MAC packet is authorized here, and the authoritative
successor for this orchestration closeout is `COMPLETE`.

### `A1.1d-5R3-MAC`

This implementation subject owns only the MAC allowlisted executor/client, mapped lifecycle
wrappers, SSH-UDS activation/teardown, socket non-propagation, and non-native fixture proof. It
does not execute native provisioning or create evidence bytes. Its sole successor is
`EVIDENCE:R3-MAC-IMP-01`.

## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Under `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d` amendment
`0003-fresh-mac-review-epoch.json`, this new nonce-bound epoch reconstructs the verified
19-path attempt-3 baseline solely to remediate the six mandatory P1/P2 findings. It binds the
actual XPC peer audit token before request decoding, repeats canonical carrier/mapping/role and
Stage-1 joins before any mapped mutation, removes the standalone retire operation, and refuses
pre-spawn SSH-UDS replacement by disabling SSH-side unlink. The added checks are non-native only:
no Lima, launchd, Keychain, code-signing, publisher installation, or evidence artifact is run or
created here. Fresh review is recorded only in the exact MAC review-control set; the sole
successor remains `EVIDENCE:R3-MAC-IMP-01`.

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN packet status (2026-08-07)

Before any renewed MAC implementation, the published recovery plan requires independently landable
R1 Bash descriptor, R2 macOS compile, R3 trusted evidence-project-ID, R4 protected MAC
Keychain/P-256/Stage-1, R5 typed mapped-submit, and R6 dual PM-bound data/TTY session packets.
They are exact-hunk recreation packets, not a donor merge. R6 is the first possible predecessor of
a separately authorized `EVIDENCE:R3-MAC-IMP-01`; each packet has its own predecessor, review,
remote publication gate, and explicit authority stop. `lima-stdio-v1`, raw `lima-action`, a direct
helper relay, an unbound channel, a Windows behavior change, and an ordinary Linux host pairing
route are excluded.

## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN current evidence-command and successor rule (2026-08-07)

For every recovery-era provider/final evidence validation, the authoritative invocation is
`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid>
--expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id
<dispatch-bound-project-id> --expected-gated-successor <value>`. The expected project ID is supplied
by the fresh evidence dispatch and is never inferred from the artifact. The historical Linux
artifact uses its declared historical ID `2ccb802f-301c-4af4-9bd5-51d22808f0a2`.

This rule supersedes only the older present-tense R3 MAC succession for the bound recovery route:
historical attempt-4 and the 8,821-line donor cannot authorize evidence. A separately authorized
`EVIDENCE:R3-MAC-IMP-01` may begin only after the future R1–R6 recovery implementation has one
reviewed remote-equal receipt; no plan, donor, or partial R1–R5 receipt is an evidence predecessor.


## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

`validate_r3_native_evidence.py <artifact> --expected-evidence-id <id> --expected-source-commit <oid> --expected-source-tree <tree> --expected-source-ref <ref> --expected-product-project-id <dispatch-bound-project-id> --expected-gated-successor <value>`

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION (2026-08-10)

- **Bound predecessor:** the remote-equal R3 recovery checkpoint plus reviewed root-LaunchDaemon
  probes showing `errSecNotAvailable` for default/Data-Protection Secure Enclave lookup,
  successful explicit legacy System-Keychain operations, software P-256 persistence/signing with
  private export capability, and rejection of Secure Enclave routing to the explicit legacy
  System Keychain.
- **Owned correction:** replace private `kSecUseSystemKeychain` and non-exportability assumptions
  with one public-API software P-256 signer in the explicitly opened and path-verified
  `/Library/Keychains/System.keychain`. Adds use `kSecUseKeychain`; searches and exact deletion use
  a one-element `kSecMatchSearchList`; every applicable item operation fails rather than allows
  authorization UI. Exact tag/service/type/size/permanence/signing/SPKI joins, duplicate and
  mismatch preserving stops, signed wrapper/anchor, monotonic CAS, canonical P1363 low-S
  signatures, receipts, and byte-identical retry remain mandatory.
- **Threat posture:** sufficiently privileged root with System-Keychain access may export the
  software private key. This matches the accepted Linux root-protected-key posture and must never
  be described as Secure-Enclave-backed, hardware-backed, or non-exportable.
- **Retirement:** a surviving protected wrapper blocks key deletion; otherwise only the exact tag
  in the exact opened System Keychain may be deleted through the validated item reference, followed
  by an exact final-absence check. Retirement and wrapper CAS share the exact current-anchor
  durable lock from pre-key validation through terminal readback, so wrapper/key ownership cannot
  cross between their checks. Ambiguity or mismatch preserves every remaining record.
- **Excluded:** Intel/T2, user/default/ambient/file/environment/caller-selected stores,
  software-file fallback, unsigned mode, product installation, Lima, pairing,
  `EVIDENCE:R3-MAC-IMP-01`, and MAC closeout. Secure Enclave/Data Protection Keychain and a user
  LaunchAgent signer are deferred hardening, not implied successor authority.
- **Proof/publication:** focused static and Rust regressions, locked/offline Apple-Silicon macOS
  build/tests, exact inventory/allowlist/secret/containment gates, fresh causal-cascading review,
  and one normal fast-forward commit only. A clean correction returns
  `next_increment=EVIDENCE:R3-MAC-IMP-01` with `successor_dispatched=false`.

## R3 macOS retirement/recovery serial-gate amendment (2026-08-13; docs-only)

This amendment controls the next R3 macOS retirement/recovery sequence over any earlier wording that would move directly from the preserved failed attempt to prospective implementation or evidence. It records no implementation, experiment, recovery, cleanup, evidence, mirror refresh, or `MAC-CLOSEOUT` authority.

- **Prospective direction:** Candidate D remains conditionally sound as the structure to close: a pre-baseline external evidence finalizer may execute one frozen terminal host-removal suffix only after external receipt/acknowledgement durability, protected-CAS binding, and durable successor acceptance. Its exact noninteractive delete-only signer capability is a mandatory experiment gate.
- **Operational order:** Candidate C is mandatory. Exact preserved-orphan recovery and baseline parity precede final prospective contract closure and any prospective implementation.
- **Separation:** the prospective V2 protocol and the precommit-less orphan lane share no authority, schema owner/version, route, executable, journal, parser, signature domain, idempotency key, or target decoder. Neither is product uninstall or generalized recovery.

The serial gates are:

1. **`R3-MAC-RETIRE-G0-FREEZE` — historical stop for the archived protected lane.** Preserve the signer, failed-attempt roots/logs, prefix, publisher/Lima state, and mirror pin. The observed SecurityAgent contradiction remains unresolved. No live Keychain query or mutation is allowed. This stop remains evidence/quarantine posture, not the active developer-parity schedule.
2. **`R3-MAC-RETIRE-G1-PLANNING-ACCEPTANCE`.** Explicitly accept the corrected external-durability, successor-acceptance, receipt-chronology, guest/host state machines, finalizer boundary, experiment plan, destructive-edge matrix, and closed stop states. This docs-only amendment does not self-accept or authorize effects.
3. **`R3-MAC-RETIRE-G2-EXPERIMENT-AUTHORIZATION`.** A separate task must bind a rollback-safe disposable native-macOS environment, fresh surrogate identities, exact arms/repetitions, process/query UI controls, SecurityAgent observations, rollback, and receipt locations. It cannot touch the orphan or product state.
4. **`R3-MAC-RETIRE-G3-EXPERIMENT-CLOSURE`.** Independent receipts prove exact rollback/baseline restoration and record two separate conclusions: creator-route no-UI behavior for recovery, and exact prospective-finalizer delete-only capability. The experiment grants no live authority.
5. **`R3-MAC-RETIRE-G4-EXACT-RECOVERY-AUTHORIZATION`.** After G3, a new incident-specific authority binds exactly one old-attempt route, immutable target identity, executor/code identity, journal, UI posture, retry states, stop policy, and review wall. A one-prompt fallback requires an additional explicit exact authorization; it is never inferred.
6. **`R3-MAC-RETIRE-G5-ORPHAN-PARITY`.** The separately authorized recovery, if any, produces a durable receipt and independently verified exact target-scope parity proof. Mirror movement is later and separately authorized; it cannot backfill parity.
7. **`R3-MAC-RETIRE-G6-PROSPECTIVE-CONTRACT-CLOSURE`.** Only after G5, freeze the literal V2 owner/version values, canonical fields/signature domains, external-finalizer caller/route/endpoint/identities, capability evidence, exact component ledger/holdbacks, journal/CAS transfer, framing constants, and fault matrix in authoritative docs and golden vectors.
8. **`R3-MAC-RETIRE-G7-PROSPECTIVE-IMPLEMENTATION-AUTHORIZATION`.** A later top-level task must name exact source/test path and symbol fences, review checks, landing authority, and stop conditions. It cannot implement recovery or authorize evidence.
9. **`R3-MAC-RETIRE-G8-NATIVE-EVIDENCE`.** Only a later explicit operator authority may run one fresh evidence attempt. `MAC-CLOSEOUT` remains a distinct later gate.

No gate dispatches, accepts, or authorizes its successor. A stop, blocked recovery, failed capability gate, or parity failure preserves the current state and does not permit schedule compression, cross-lane substitution, broader cleanup, or retrospective authority.

## Current Linux-first scheduling reset (2026-08-20; controlling)

The former R3 MAC → MAC evidence/closeout → Windows → cross-platform closeout sequence and the
retirement/finalizer/E03 serial gates above are archived and superseded **for active scheduling**.
The 2026-08-19 macOS decision retains its strict developer-corridor boundary, but its former global
blocking order is now superseded. These records remain truthful chronology; no source removal,
archive-ref change, cleanup, or completion claim is implied.

```text
Linux-first scheduling decision
  -> AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY (closed as docs-only selection)
  -> A1.3-P1 Linux-first atomic public-adoption packet
  -> bounded implementation and proof under its exact fence

Separate macOS lane
  -> AUTHORITY_REQUIRED:MACOS_DEV_PARITY
  -> separately authorized macOS Phase 2 implementation
  -> native install -> exercise world -> uninstall -> verify -> reinstall close
```

`AUTHORITY_REQUIRED:RUNTIME_REFACTOR_REENTRY` closed as a documentation/control-plane rebind that
historically selected [A1.3](linux-first-runtime-resumption/A1.3-LINUX-FIRST-PACKET.md). The held
[A1.3-P0](linux-first-runtime-resumption/A1.3-P0-LINUX-FIRST-PREPARATORY-PACKET.md) record keeps
the scope-expansion evidence, but the controlling correction now makes
[A1.3-P1](linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md) the
active bounded Linux-first implementation/proof fence while preserving both older records as held
historical fences only.
`AUTHORITY_REQUIRED:MACOS_DEV_PARITY` owns only the separate current-product user-prefix developer corridor: current
shims/configuration/binary staging, current Lima/`world-service`, typed selected-prefix and
Lima-instance mapping, and safe forwarding. It excludes System Keychain, protected publisher,
macOS lifecycle LaunchDaemon/privileged host helper, terminal retirement/finalizer, E03, freeze,
identity rotation, and assurance-evidence machinery.

Before macOS Phase 2 effects, a later exact read-only, non-Keychain overlap check must prove the new
developer resources disjoint from Attempt 4. Collision is a stop for separate disposition; the
parity task cannot inspect/mutate Attempt 4 records or retire/migrate/overwrite/adopt/clean its
fixed privileged artifacts. Linux R3 remains landed historical fact with behavior preserved and no
new claim. Windows remains untouched, incomplete where applicable, and deferred until a later
post-runtime-refactor scheduling decision.

Neither lane dispatches implementation or the other's successor. The held A1.3 and A1.3-P0
records do not dispatch implementation; A1.4, Windows, E03, and all other product work remain
undispatched until their own fresh exact authority packets activate them. See
[`linux-first-runtime-resumption/DECISION.md`](linux-first-runtime-resumption/DECISION.md) and
[`macos-dev-parity/DECISION.md`](macos-dev-parity/DECISION.md).
