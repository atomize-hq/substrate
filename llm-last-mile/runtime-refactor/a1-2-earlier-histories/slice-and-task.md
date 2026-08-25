**Kind:** slice and task record
**Stable ID:** `A1.2-earlier-histories-family`
**Status:** canonical historical/completed-family record
**Authority scope:** exact extracted family-local source bodies only; no implementation authority
**Supersedes:** canonical ownership of the extracted source bodies; source headings/rows remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)
**Canonical for:** A1.2 and earlier A1.1/A1.1d packet decomposition, checkpoints, and chronology only
**Source span:** composite of the six preserved root compatibility spans listed in the extraction ledger

# A1.2 and earlier packet-histories slice and task record

## A1.2 aggregate packet row

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.2 — intent issuance/claim/application** | Add the A1.2-owned durable protocol under `HostSessionAuthority` while preserving landed V1 intent/state bytes as read-only: A1.2a supplies an A1.1e-dependent but B1-independent strict V2 Start intent with no B1-owned type, and A1.2b later supplies a separately reviewed strict V3 root/intent/state extension for Attach/Resume and post-turn fields after B1/B3.1/C1 types are available. Together they cover greenfield-certified Start reservation/issuance/application, exact retry, startup ownership, successor issuance/application, post-turn reconciliation, ledger-owned obligation-cut consumption, and retained transport reprojection; keep the plan a transport projection. The aggregate packet includes parked-successor `Attach`/`ResumeOneTurn`, but the normative split below assigns only greenfield Start establishment/read to A1.2a before the joint closeout; A1.2b retains successor/post-turn mechanics and begins only after the joint closeout, B3.1, and C1 establish its complete semantic-cut prerequisite. | Authority-store core files listed in the A1.1 aggregate packet row below; the bounded retained-admission compatibility read corridor in `agent_runtime/retained_worker_runtime.rs`; only `HiddenOwnerHelperLaunchPlan` plus its write/load/remove/launch integration in `agent_runtime/control.rs`; focused V1/V2 Start and later V3 successor discrimination, Start reservation/birth, intent lifecycle, serialization, persistence, helper-loss, parked-successor, input-handoff, startup-ownership, post-turn-event/input terminalization, pending obligation-cut consumption, payload-retention/reprojection, retained-admission compatibility, and plan-commitment tests. Runtime transport identity, receipt acceptance, supervisor journaling, retained-event semantics, and the canonical snapshot producer remain in B0/B1/B2.1/B3.1/C1 and are not A1.2 implementation allowances. | Exact greenfield certificate and empty pre-A1 collections; unique `intent_id`; intent/state revision; `Start`/`Attach`/`ResumeOneTurn`; exact enumerated postures; `ExpectedAbsent` or exact authority revision/hash; reservation ownership; identity/home/workspace/world/descriptor/object commitments; expiry; canonical payload and transport refs/hashes; `Issued -> Claimed -> Applied` plus terminal `Rejected`/`Expired`; immutable application result; input handoff; actor-bound startup evidence/result; actor-bound post-turn protocol event/completion/result; `AwaitingObligationCut`; ledger-owned Complete snapshot; transport `ReleaseEligible`/`Released` state. | No V1 shape mutation/defaulting; no legacy-state conversion or compatibility-derived absence; no CLI/REPL consumer switch yet; no runtime frame/event identity generation; no receipt acceptance or stream observation; no retained-message semantics; no obligation enumeration/classification/creation/resolution/reinterpretation/overwrite and no claim that event materialization is complete; no C1 implementation; no destructive-read consumption; no endpoint/path redesign; no helper removal; no general transport cleanup; no PID/helper/handle/prompt-derived transition eligibility or transport/process inference as terminal protocol evidence. | Prove strict V1 bytes remain identical/read-only, production Start uses only V2 without B1 imports, and Attach/Resume/post-turn uses only the later strict V3 extension. Prove `Attach` and `ResumeOneTurn` start from exact current parked or other enumerated authority, preserve session lifecycle identity and exact world binding, revision-authorize one successor lineage, and never derive eligibility from PID, helper, handle, readiness, or prompt state. Prove Start/Attach transport cannot advance beyond `ReleaseEligible` before exact actor-bound startup evidence resolves ownership, every Resume terminal outcome joins an exact actor-bound post-turn event/completion/result and closed input state, exact retries survive a later exact `Released` cleanup, terminal Resume can close without a snapshot, and resumable completion persists `AwaitingObligationCut` while retaining transport. A Complete ledger cut must be scoped to exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, authenticated intent/revision/payload commitment and distinct transition run, and authority/event watermark, then select the ledger-owned disposition without A1.2 scanning, inference, or reclassification. Exact stale successor retries fail or join exactly; failure after application never restores a stale snapshot. Because C1 is now a prerequisite, the full packet exits only when that unchanged Complete result drives the one allowed revisioned post-turn application and all A1.2 proofs pass. | Real Start/Attach/Resume consumer adoption; bounded auto-attach adoption; full real-path `RG-AUTH-03`; `RG-BASE-01`; A1 completion. |

## A1.1 aggregate packet row

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.1 — authority-store core** | Establish only the authority-store substrate: persisted normalized bootstrap-home binding, exact fresh/pending/existing/unsupported/corrupt classification, operation-bound temp reconciliation, exact object/key persistence, immutable greenfield namespace certification, fail-closed pre-A1 authority-state detection, exact durable resolution, and generic root/authority revision-CAS. Keep `StateStore` as atomic persistence. | Bounded modules under `agent_runtime/host_session_authority/`; bounded facade types/re-exports only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only for the minimum persistence/integration choke points that exclude legacy/direct writers; explicit-home API parameterization only in `crates/common/src/paths.rs` and `execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; focused codec, trusted-filesystem, bootstrap/reconciliation, writer-exclusion, resolution, persistence-conflict, and cross-process tests. | `StateRootV1`; `AuthorityStoreInitializationV1`; `GreenfieldNamespaceCertificateV1`; `DurableSessionAuthorityV1`; `CanonicalDirectoryV1`; immutable typed object schemas/refs; key registry/files; operation-bound temp grammar; expected `root_revision`/`authority_revision` for CAS. | No migration, quarantine, evidence, dual-write, or compatibility adoption; no production `ExpectedAbsent` acceptance; no Start reservation, transition-intent issuance/application, participant allocation, or Start-origin authority birth; no helper-plan or consumer adoption; no episode demotion; no resolver precedence, policy/config/inventory interpretation; no unrelated StateStore cleanup or general persistence extraction. | Write failing tests for genuinely absent versus malformed/partial/unreadable/unsafe/unsupported stores, interrupted initialization and copied-home mismatch, exact bootstrap-home reuse, crashes before/after temp fsync and rename, pending-key rename/directory-fsync recovery, key create/rotate/retire crash windows, first-use kind/version directory create/fsync crashes, object rename before root commit with retained-orphan restart/exact retry, safe empty versus present/unreadable/symlinked pre-A1 collections, ancestor symlink/unsafe ACL/cross-device and scan-to-publication replacement, pre-A1 writers serialized before activation and rejected after marker/root publication, cross-kind object substitution, exact resolution, stale revision, and cross-process CAS. Exit when these primitives are deterministic and green; this packet alone cannot create or claim a production Start. | Production Start/`ExpectedAbsent`; all transition intents and `RG-AUTH-03`; real CLI/REPL adoption; helper-loss reconciliation; auto-attach adoption; complete `RG-AUTH-01`/`RG-AUTH-02`; A1 completion. |

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
`tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1` exception preserved in the
[B1/B2.1 production-ingress ownership audit](../b1-b2-1/crosswalk.md#b1b21-dispatch-authority-ownership-audit).
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

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2--unix-release-sudo-linux-service-and-runtime-propagation`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2--unix-release-sudo-linux-service-and-runtime-propagation).

###### A1.1d-5R2-2E — Authenticated world-gateway projection

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2e--authenticated-world-gateway-projection`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2e--authenticated-world-gateway-projection).

###### A1.1d-5R2-2F0 — Deterministic world-socket test isolation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0--deterministic-world-socket-test-isolation`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0--deterministic-world-socket-test-isolation).

###### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0a--substrate_home-test-isolation`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0a--substrate_home-test-isolation).

###### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0b--deterministic-renderer-output-test-isolation`](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f0b--deterministic-renderer-output-test-isolation).

###### Historical A1.1d-5R2-2F — Authenticated world-deps and truthful doctor composition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/slice-and-task.md#historical-a11d-5r2-2f--authenticated-world-deps-and-truthful-doctor-composition`](../a1.1d-5r2-2f/slice-and-task.md#historical-a11d-5r2-2f--authenticated-world-deps-and-truthful-doctor-composition).
###### A1.1d-5R2-3 — Platform-native mapping adapters

Compatibility anchor only; canonical content: [`a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3--platform-native-mapping-adapters`](../a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3--platform-native-mapping-adapters).

###### A1.1d-5R2-3 bounded docs-first subdivision

Compatibility anchor only; canonical content: [`a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3-bounded-docs-first-subdivision`](../a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3-bounded-docs-first-subdivision).
###### A1.1d-5R2-4 — R2 integration and closeout

Canonical content: [`a1.1d-5r2-4/slice-and-task.md#a11d-5r2-4--r2-integration-and-closeout`](../a1.1d-5r2-4/slice-and-task.md#a11d-5r2-4--r2-integration-and-closeout).

###### A1.1d-5R1 exact allowlist

R1 may edit only
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
R2-2F was the next packet; the
[completed-F phase record](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f--completed-phase-record)
supersedes that status. `RG-HOME-01` and
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
  Authenticated world-deps and truthful doctor composition**; the
  [completed-F phase record](../a1.1d-5r2-2f/slice-and-task.md#a11d-5r2-2f--completed-phase-record)
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
