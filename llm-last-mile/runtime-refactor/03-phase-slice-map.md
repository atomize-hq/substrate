# Phase and Slice Map

## Sequencing rules

Each slice is independently reviewable. A0 is a diagnostic inventory; every later implementation slice must move one authority boundary plus proof. A slice may read sibling context without editing sibling ownership.

Hard dependency spine:

```text
A0 -> A1.1e -> B0 -> B1-3a/B1-3b receipt core
                         -> B2.1-1 -> B2.1-2 -> B2.1-3
                         -> B1/B2.1 joint production closeout
                         -> B3.1 -> C1 -> A1.2 -> A1.3 -> A1.4 -> A1
      A1.1d integrated Linux/native-macOS closeout -----------------------------^
A1 -> A2/A3 -> E2 -> B2.2 -> B3.2 -> B4 -> C2 -> C3
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
production bypass that only a later canonical owner packet can close. For the exact parked-
successor blocker recorded in `05-debug-regression-ledger.md`, the landed A1.1e facade is the
authority foundation for the bounded B0 → B1 receipt core → B2.1 → joint closeout → B3.1 → C1
prerequisite corridor even while A1.1d
integrated Linux and native-macOS closeout remain open. The corridor may not issue or apply host
transitions, demote episodes, extract general persistence, enable foreground early return, or
perform broader retained-worker lifecycle work. A1.2 resumes only after C1 can return the complete
semantic cut from B2.1's durable exact event truth and B3.1's typed retained-event semantics. This
hold does not permit A1.3 consumer adoption. The failing regression remains explicit, stale
lifecycle/world-binding behavior remains rejected, the feature branch remains non-landable, and
A1.2/A1.3 must close their assigned gates before A1 can complete. This exception creates no
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
5. C1 alone classifies and materializes obligations and declares the complete cut. A1.2 consumes
   that result unchanged.

The typed host-transition correlation does not add a reverse dependency on A1.2. B1 places both
the carrier and its equality-only opaque commitment representation in `substrate-common`, adds it
optionally to the typed request context, and keeps it absent on production paths until
HostSessionAuthority supplies it; no new A1.1e hash domain or verifier is required. B3.1 copies it,
B2.1 journals the bytes without interpreting them, B3.1 validates equality with B1, and C1 is fully functional for exact inputs while
refusing a transition-scoped Complete result when the correlation is absent or mismatched. A1.2
later becomes the only production source, validates its own intent/revision/payload commitment,
supplies its exact intent/transition-run correlation through the HostSessionAuthority-owned call
boundary, and consumes the C1 result unchanged. Its real-path adoption is therefore the downstream
integration gate, not a C1 prerequisite.

No A2 prerequisite is pulled forward because process/PID/socket demotion is unnecessary to build
the producer-to-ledger truth path. No A3 prerequisite is pulled forward because StateStore may
provide bounded atomic persistence without deciding receipt, observation, messaging, or obligation
semantics.

All code areas below are allowlists for planning, not permission to edit every listed file. Future implementation must run live impact analysis before symbol changes.

## Track A — Authority and surface neutrality

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **A0 — Authority leak inventory** | Produce a repo-grounded inventory of every process/socket/helper/owner/cwd/env value currently used in durable decisions. | `00`; `01` invariants 1–3; `02` A0 inventory contract plus HostSessionAuthority, StateStore, and SurfaceAdapter rows; helper walkthrough/debug docs | HostSessionAuthority; HostExecutionEpisode; StateStore; CompatibilityReadModel | `agent_runtime/control.rs`; `agent_runtime/state_store.rs`; `execution/agents_cmd.rs`; `execution/orchestrator_world_dispatch.rs`; `repl/async_repl.rs`; UAA launch/member-runtime paths | No behavior changes except diagnostics/tests. No authority facade. No seventh control-pack file. | A committed inventory table in `02-seam-crosswalk.md` classifies every usage as `SignalOnly`, `FastPathTransport`, `AuthorityDecision`, or `CompatibilityRead`, and records proposed owner seam, first migration target, and proof gate. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01` |
| **A1 — HostSessionAuthority facade** | Establish the only API that resolves exact durable session/caller/binding truth, owns revision-checked posture transitions, and durably issues, claims, applies, and reconciles `HostSessionTransitionIntentV1` for every real host `Start`, `Attach`, and `ResumeOneTurn` path. | `01` Authority map + invariants 1–3; `02` HostSessionAuthority and helper-plan/mode/parent-control/private-stop/auto-park rows; `04` `DurableSessionAuthorityV1` + `HostSessionTransitionIntentV1`; `DESIGN-host-orchestrator-world-dispatch-contract.md` exact identity rules | StateStore; CompatibilityReadModel; HostExecutionEpisode; WorldDispatchControl; AutoAttachProjection | Bounded modules under `crates/shell/src/execution/agent_runtime/host_session_authority/`; bounded facade types/re-exports and integration only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only as the bounded persistence/integration surface; bounded explicit-bootstrap-home entry points only in `crates/common/src/paths.rs` and `crates/shell/src/execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; in `agent_runtime/control.rs`, only `HiddenOwnerHelperLaunchPlan` and its write/load/remove/launch integration; in `execution/agents_cmd.rs`, only `Start`/`Attach`/`ResumeOneTurn` intent issuance and `run_owner_helper` consumption; in `repl/async_repl.rs`, only owner-helper validation/application and real REPL adoption; in `agent_runtime/auto_attach.rs`, only `build_auto_attach_launch_plan` and directly required plan integration; focused tests | No general `HostExecutionEpisode` demotion; no endpoint/path redesign; no helper removal; no auto-attach policy, eligibility, or settlement redesign; no receipt/supervisor work; no config/policy/inventory precedence, merge, or interpretation changes; no unrelated lifecycle, transport, or schema cleanup; no unrelated StateStore cleanup or general persistence extraction. | All real CLI and REPL `Start`/`Attach`/`ResumeOneTurn` transitions route through `HostSessionAuthority` and a durable revision-bound intent; exact applied retry joins without reapplying; helper loss/restart reconciles; stale, substituted, mismatched, expired, or conflicting replay fails closed; stale observations cannot overwrite newer authority; `RG-BASE-01` and `RG-BASE-02` remain green. Do not claim A1 before A1.1–A1.4 and the final proof wall complete. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02` |
| **A2 — HostExecutionEpisode demotion** | Represent REPL/helper/toolbox/recovered processes as episodes whose PID/socket/readiness data are observations only. | `01` invariants 1–2; `02` SurfaceAdapter row; `04` HostExecutionEpisodeV1; `DESIGN-internal-toolbox-transport-and-session-binding.md` | HostSessionAuthority; InternalToolboxTransport; RouterAttachTrigger | `agent_runtime/control.rs`; `execution/agents_cmd.rs`; `repl/async_repl.rs`; episode-focused tests | No deletion of `__owner-helper`; no transport protocol rewrite; no auto-attach redesign. | Killing/orphaning an episode leaves durable session truth intact; stale episode updates are revision-rejected; transport classification is explicit. | `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01` |
| **A3 — Persistence and compatibility split** | Reduce StateStore to atomic persistence/schema evolution and isolate diagnostic compatibility reads from new authority writes. | `02` StateStore + CompatibilityReadModel rows; `01` authority map; `04` durable revision rules | HostSessionAuthority; InboxProjection; receipt persistence | `agent_runtime/state_store.rs`; a bounded new facade/module under `agent_runtime/`; persistence/read-model tests | No wholesale database rewrite; no conversion of pre-A1 authority artifacts into A1 authority; no behavior changes outside moved ownership. | New authority writes bypass compatibility projection; torn-root/unsupported-state diagnostics remain read-only; newer revisions always win. | `RG-AUTH-02`, `RG-BASE-01`, `RG-BASE-03` |

### A1 bounded packet decomposition

A1 is one independently reviewable slice implemented in the following order. Each packet must remain green and reviewable before the next begins; prerequisites established by an early packet do not satisfy later production-adoption or slice-closeout gates.

| Packet | Goal and authority boundary | Exact code and test areas | Contract fields or transitions | Explicit non-goals | Test-first or test-alongside proof and packet exit | Slice gates not yet claimable |
|---|---|---|---|---|---|---|
| **A1.1 — authority-store core** | Establish only the authority-store substrate: persisted normalized bootstrap-home binding, exact fresh/pending/existing/unsupported/corrupt classification, operation-bound temp reconciliation, exact object/key persistence, immutable greenfield namespace certification, fail-closed pre-A1 authority-state detection, exact durable resolution, and generic root/authority revision-CAS. Keep `StateStore` as atomic persistence. | Bounded modules under `agent_runtime/host_session_authority/`; bounded facade types/re-exports only in `agent_runtime/{orchestration_session,mod}.rs`; `agent_runtime/state_store.rs` only for the minimum persistence/integration choke points that exclude legacy/direct writers; explicit-home API parameterization only in `crates/common/src/paths.rs` and `execution/{config_model,policy_model,policy_snapshot,agent_inventory}.rs`; focused codec, trusted-filesystem, bootstrap/reconciliation, writer-exclusion, resolution, persistence-conflict, and cross-process tests. | `StateRootV1`; `AuthorityStoreInitializationV1`; `GreenfieldNamespaceCertificateV1`; `DurableSessionAuthorityV1`; `CanonicalDirectoryV1`; immutable typed object schemas/refs; key registry/files; operation-bound temp grammar; expected `root_revision`/`authority_revision` for CAS. | No migration, quarantine, evidence, dual-write, or compatibility adoption; no production `ExpectedAbsent` acceptance; no Start reservation, transition-intent issuance/application, participant allocation, or Start-origin authority birth; no helper-plan or consumer adoption; no episode demotion; no resolver precedence, policy/config/inventory interpretation; no unrelated StateStore cleanup or general persistence extraction. | Write failing tests for genuinely absent versus malformed/partial/unreadable/unsafe/unsupported stores, interrupted initialization and copied-home mismatch, exact bootstrap-home reuse, crashes before/after temp fsync and rename, pending-key rename/directory-fsync recovery, key create/rotate/retire crash windows, first-use kind/version directory create/fsync crashes, object rename before root commit with retained-orphan restart/exact retry, safe empty versus present/unreadable/symlinked pre-A1 collections, ancestor symlink/unsafe ACL/cross-device and scan-to-publication replacement, pre-A1 writers serialized before activation and rejected after marker/root publication, cross-kind object substitution, exact resolution, stale revision, and cross-process CAS. Exit when these primitives are deterministic and green; this packet alone cannot create or claim a production Start. | Production Start/`ExpectedAbsent`; all transition intents and `RG-AUTH-03`; real CLI/REPL adoption; helper-loss reconciliation; auto-attach adoption; complete `RG-AUTH-01`/`RG-AUTH-02`; A1 completion. |
| **A1.2 — intent issuance/claim/application** | Add the A1.2-owned durable `HostSessionTransitionIntentV1` protocol under `HostSessionAuthority`, including greenfield-certificate-checked production Start `ExpectedAbsent`, atomic Start reservation and issuance, claim, single application and initial authority birth, exact-retry join, expiry/rejection, crash reconciliation, revisioned input handoff, actor-bound startup-ownership and post-turn resolution, ledger-owned obligation-cut consumption, persistent pending post-turn reconciliation, and retained transport reprojection; keep the plan a transport projection. This packet owns parked-successor `Attach`/`ResumeOneTurn` protocol mechanics and resumes only after the B1/B2.1 joint production closeout, B3.1, and C1 establish its complete semantic-cut prerequisite. | Authority-store core files above; only `HiddenOwnerHelperLaunchPlan` plus its write/load/remove/launch integration in `agent_runtime/control.rs`; focused Start reservation/birth, intent lifecycle, serialization, persistence, helper-loss, parked-successor, input-handoff, startup-ownership, post-turn-event/input terminalization, pending obligation-cut consumption, payload-retention/reprojection, and plan-commitment tests. Runtime transport identity, receipt acceptance, supervisor journaling, retained-event semantics, and the canonical snapshot producer remain in B0/B1/B2.1/B3.1/C1 and are not A1.2 implementation allowances. | Exact greenfield certificate and empty pre-A1 collections; unique `intent_id`; intent/state revision; `Start`/`Attach`/`ResumeOneTurn`; exact enumerated postures; `ExpectedAbsent` or exact authority revision/hash; reservation ownership; identity/home/workspace/world/descriptor/object commitments; expiry; canonical payload and transport refs/hashes; `Issued -> Claimed -> Applied` plus terminal `Rejected`/`Expired`; immutable application result; input handoff; actor-bound startup evidence/result; actor-bound post-turn protocol event/completion/result; `AwaitingObligationCut`; ledger-owned Complete snapshot; transport release. | No legacy-state conversion or compatibility-derived absence; no CLI/REPL consumer switch yet; no runtime frame/event identity generation; no receipt acceptance or stream observation; no retained-message semantics; no obligation enumeration/classification/creation/resolution/reinterpretation/overwrite and no claim that event materialization is complete; no C1 implementation; no destructive-read consumption; no endpoint/path redesign; no helper removal; no general transport cleanup; no PID/helper/handle/prompt-derived transition eligibility or transport/process inference as terminal protocol evidence. | Prove `Attach` and `ResumeOneTurn` start from exact current parked or other enumerated authority, preserve session lifecycle identity and exact world binding, revision-authorize one successor lineage, and never derive eligibility from PID, helper, handle, readiness, or prompt state. Prove Start/Attach transport cannot release before exact actor-bound startup evidence resolves ownership, every Resume terminal outcome joins an exact actor-bound post-turn event/completion/result and closed input state, exact retries work after payload deletion, terminal Resume can close without a snapshot, and resumable completion persists `AwaitingObligationCut` while retaining transport. A Complete ledger cut must be scoped to exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, authenticated intent/revision/payload commitment and distinct transition run, and authority/event watermark, then select the ledger-owned disposition without A1.2 scanning, inference, or reclassification. Exact stale successor retries fail or join exactly; failure after application never restores a stale snapshot. Because C1 is now a prerequisite, the full packet exits only when that unchanged Complete result drives the one allowed revisioned post-turn application and all A1.2 proofs pass. | Real Start/Attach/Resume consumer adoption; bounded auto-attach adoption; full real-path `RG-AUTH-03`; `RG-BASE-01`; A1 completion. |
| **A1.3 — Start/Attach/Resume consumer adoption** | Route public CLI issuance, `run_owner_helper` consumption, owner-helper application, and the real REPL `Start`/`Attach`/`ResumeOneTurn` transitions through the authority and claimed intent. This packet owns the bounded exact startup/post-turn protocol-event adapter, real-path parked-successor adoption, and `RG-BASE-01` closure. | The A1.2 plan integration; only the named intent issuance/consumption regions in `execution/agents_cmd.rs`; only owner-helper validation/application, exact startup/post-turn event submission, real REPL adoption, and explicit bootstrap-home threading in `repl/async_repl.rs`; only the A1.1 explicit-home resolver entry points needed by those named CLI/REPL paths; focused CLI, helper, and REPL integration tests. | Mode-specific preconditions; exact caller/source/target lineage; bootstrap-home/workspace/world binding; typed descriptor/attach/resume/policy/input refs; claim ownership; authority/intent revisions; applied result; exact actor-bound ownership-acknowledgement/pre-ownership rejection/failure event identity and result; exact actor-bound post-turn event ID/sequence/outcome/reason and one-turn disposition. | No resolver precedence, policy/config/inventory interpretation, or broad REPL lifecycle cleanup; no general episode model rewrite or relabeling of readiness/PID/helper/socket/timeout/EOF/local errors as startup or post-turn protocol evidence; no private endpoint/path redesign; no receipt, supervisor, or member-runtime work. | Prove a public turn from parked truth issues/applies `ResumeOneTurn`, explicit reattach issues/applies `Attach`, and real helper/REPL launch occurs only after durable application. Prove exact `OwnershipAccepted` accepts ownership, exact `RuntimeCreationRejected`/`StartupFailedBeforeOwnership` terminally reconcile with matching actor/evidence/result identity, and timeout/drop/EOF/readiness/process posture or local adapter error remains Pending. Every applied Resume terminal outcome has an immutable exact-scope protocol event, matching completion plus post-turn application, and closed input terminalization; exact retry joins; the exact public start → turn/reattach → stop path becomes green. Retain existing negative tests for helper loss, stale revision, ambient-home/CWD reread, substitution, role mismatch, duplicate application, conflicting startup evidence, and substituted post-turn events. | Bounded auto-attach producer adoption; complete A1 regression closure; A1 completion. |
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, then close A1 without changing projection policy or settlement ownership. | Only `build_auto_attach_launch_plan` and directly required plan integration in `agent_runtime/auto_attach.rs`; focused auto-attach producer/consumer tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, and idempotent applied result. | No auto-attach policy, eligibility, claim, or settlement redesign; no router responsibility expansion; no endpoint/path redesign; no A2 demotion; no subsequent-slice work. | Prove manual and auto-attach cannot substitute or double-apply an intent, retry/restart converges, every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` pass on real CLI and REPL paths; only then may A1 close. | No A1 gate remains claimable until this packet's final wall passes; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows remain unresolved for A2/A3, and later sibling-seam gates remain out of scope. |

The A1.1d cross-packet integration hold permits the bounded B0 → B1 receipt core → B2.1 → joint
B1/B2.1 closeout → B3.1 → C1 prerequisite corridor despite open integrated Linux and native-macOS
closeout. The corridor moves only the
minimum facts and owners needed for the semantic cut; it does not move B2.2 foreground early
return, B3.2 retained lifecycle/messaging completion, A2 episode demotion, or A3 persistence
separation. A1.2 remains blocked at `AwaitingObligationCut` until the corridor lands, then resumes
as the consume-only authority client. A1.3 remains blocked until A1.2 exits.

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
A1.1d-6, permit A1.2 packet exit before its C1-owned semantic cut, or permit A1.3 work early. The
normative current rule is stricter: do not restore, modify, or resume A1.2 until the B1/B2.1 joint
production closeout, B3.1, and C1 have landed.

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

The new A1.1 module boundary is organizational only and does not change any A1 contract, semantic gate, or later-slice owner. It enforces these additional reviewability constraints: no path-based authority operation after the trusted root opens; no symlink traversal or ambient-CWD authority; exactly one repository-owned canonical codec; exactly one centralized semantic transaction preflight; no legacy/direct writer may bypass `HostSessionAuthority`; key/object/root crash reconciliation is deterministic; and no new dependency is permitted without explicit dependency review. The bounded modules must not perform A2 episode demotion, helper removal, endpoint redesign, receipt work, unrelated StateStore cleanup, or general persistence extraction.

The `RG-AUTH-01` and `RG-AUTH-02` references in A1 are scoped gates, not whole-ledger closure claims. A1 proves only the `Start`/`Attach`/`ResumeOneTurn`, bounded helper-plan, real-REPL, and bounded auto-attach producer clauses named by A1.1–A1.4. General episode loss/demotion, Stop/Fork adoption, unrelated PID/socket consumers, retained workers, and compatibility/persistence separation remain unresolved for A2/A3 or their named later owners.

### Current A1 sequencing status

- A1.1d-1 through A1.1d-4 implementation remains preserved.
- A1.1d-5 focused private-home implementation is review-clean on Linux; A1.1d integrated Linux
  closeout is open and cross-platform closeout is pending.
- The public lifecycle failure is owned by A1.2/A1.3, not an A1.1d heartbeat or successor-work
  subpacket. No A1.1d-6 exists or is implied.
- A1.1e is focused-proof and review clean through `cd676614`; this does not close A1.1d or any
  integrated product gate.
- A1.2 transition-protocol work exposed the missing producer-to-ledger truth path and remains
  preserved out of the source branch. Its resumable post-turn must remain
  `AwaitingObligationCut`; do not restore or resume it before the corrected prerequisite corridor.
- The newly authorized implementation order is B0, B1-3a/B1-3b receipt core, B2.1-1/2/3, joint
  B1/B2.1 production closeout, B3.1, C1, then A1.2. B0 is landed and receipt-core recovery/review is
  the exact next packet. B2.1 requires that review-clean core, while B1 production completion and
  B3.1 require the joint closeout; no A2/A3 prerequisite moves into the corridor.
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
cannot read or mutate `StateRootV1`, session-authority objects, transition intents, journals, keys,
or typed authority objects. The existing legacy writer remains rejected after the initialization
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
   terminal closeout immutable and idempotent. EOF, timeout, PID, helper, socket, readiness,
   process death, or local error never fabricates terminal truth.

After all three subpackets, **B1/B2.1 joint production closeout** proves both accepted families,
legacy-writer exclusion after acceptance, caller-drop/restart survival, exact terminal behavior,
blocking compatibility, the differential baseline, and supported doctor/smoke. B3.1 remains
blocked until this joint closeout passes.

For that closeout, "the differential baseline" means the monotonic transition gate in `04`,
recorded as `RG-DIFF-01` in `05`, not raw pass/fail totals. In particular, a historical failure
becoming a pass is accepted only with exact causal-resolution proof; it is neither automatic
success nor automatic regression. Any unresolved transition is `BaselineRegressionAmbiguous`,
keeps the joint closeout open, and leaves B3.1 blocked.

| Packet | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **B0 — Durable runtime event identity and ordering carrier** | Make the runtime producer assign stable stream/frame/event/terminal identity and monotonic ordering before any host observer sees a frame. | Runtime event transport. | `01` invariants 3–6; `02` RuntimeEventTransport row; `04` runtime carrier contract; `05` `RG-EVENT-01`; internal toolbox transport framing/terminality. | ReceiptRegistry; Supervisor; MessagingProtocol; world-service execution producers. | Bounded `AgentEvent` fields in `crates/common/src/agent_events.rs`; bounded `ExecuteStreamFrame` fields in `crates/transport-api-types/src/lib.rs`; frame/event producers in `crates/world-service/src/{service,member_runtime}.rs`; carrier-only decode/match compatibility in exactly `crates/shell/src/execution/orchestrator_world_dispatch.rs`, `crates/shell/src/execution/agent_runtime/control.rs`, `crates/shell/src/execution/routing/dispatch/world_ops.rs`, and `crates/shell/src/repl/async_repl.rs`; focused tests adjacent to those areas. | No receipt persistence, supervisor/journal or consumer-side duplicate rejection, semantic changes in the named shell files, retained-message classification, obligation work, unrelated transport redesign, or model-facing UX change. | Every accepted stream has one stable `stream_id`; every frame has a strictly monotonic producer sequence; every semantic event has a stable event ID/sequence; the terminal frame names the exact terminal event ID/sequence and is last. The producer never originates a gap, reorder, conflicting identity reuse, or post-terminal frame. Exact replay preserves identity and bytes; B2.1, not B0, accepts that replay as a no-op or rejects a conflict. | Producer clauses of `RG-EVENT-01`; prerequisite clauses of `RG-SUP-01` and `RG-MSG-01`; B0 carrier clause of `RG-OBS-01`. | Landed A1.1e exact authority facade; no A2/A3 prerequisite. | Consumer replay/deduplication/rejection, receipt acceptance, durable observation/restart, retained semantics, obligation materialization/cut, foreground early return, and every seam promotion. |
| **B1-3a/B1-3b — Receipt registry core (B1 not closed)** | Persist `WorldWorkAcceptanceRecordV1` for both ephemeral tasks and retained turns from an exact B0 pre-terminal acknowledgement; keep the immutable accepted anchor inspectable independently of foreground scope. | WorldWorkReceiptRegistry. | `02` ReceiptRegistry row; `04` acceptance context/record + active receipts + receipt acceptance; `05` receipt rows; tool-invocation async model. | HostSessionAuthority; RuntimeEventTransport; EffectivePolicyResolver; B2.1 supervisor handoff; RetainedWorkerRuntime; E2 policy commitments. | Exactly the canonical ownership, activated-store substrate, and carrier-constructor/test files enumerated in **B1 receipt core and activated-store allowlist** above; no implicit wildcard or additional integration site is authorized. | No host transition issuance/authentication/interpretation, generic activated-store writer, weakened legacy-writer exclusion, `ExecuteStreamFrame` change, accepted record before acknowledgement, active observation/journal ownership, foreground early return, retained lifecycle, obligation materialization, or UAA mediation. | Proposal, exact retry, task and retained acknowledgement, activated-store persistence, immutable accepted inspection, and legacy-writer rejection are review-clean. The proposal alone is never accepted truth. This exit authorizes B2.1 consumption but explicitly does not claim B1 production completion. | Acceptance clauses of `RG-RECEIPT-03` and `RG-BASE-02`; B1 acceptance clause of `RG-OBS-01`. | B0 and landed A1.1e; optional correlation does not require A1.2 and cannot authenticate, mint, verify, or interpret it. | Joint B1/B2.1 production closeout; E2 final receipt/cap commitments; A1.2 correlation supply; foreground early return; retained lifecycle; obligations; seam promotion. |
| **B2.1-1/B2.1-2/B2.1-3 — Supervisor handoff and durable observation** | Transfer sole post-acceptance stream ownership for both accepted families into a receipt-scoped durable claim and journal with no-gap handoff, restart-safe observation, exact replay handling, and terminal reconciliation while preserving blocking foreground behavior. | WorldWorkExecutionSupervisor. | `01` invariants 4–6; `02` supervisor row; `04` runtime carrier, receipt acceptance, and supervisor rules; `05` `RG-SUP-01`/`RG-SUP-02`/`RG-OBS-01`. | RuntimeEventTransport; ReceiptRegistry; WorldDispatchControl; MessagingProtocol; ObligationLedger; RetainedWorkerRuntime. | Bounded supervisor/journal module under `crates/shell/src/execution/`; `execution/orchestrator_world_dispatch.rs`; narrow construction/integration wiring in `agent_runtime/state_store.rs` to the separately scoped opaque supervisor physical capability in `agent_runtime/host_session_authority/store.rs`, `store/platform/transaction.rs`, and focused `store_tests.rs`; all supervisor schemas, validation, transitions, and reconciliation remain in the supervisor module; bounded transport API/client and `crates/world-service/src/{service,member_runtime}.rs` replay/reconciliation plumbing; focused claim/no-gap/dedupe/waiter/caller-drop/restart/terminal tests. | No foreground early return, fixed-path active-task side table, generic activated-store writer, weakened legacy-writer rejection, runtime identity generation, retained-message taxonomy, obligation materialization, inbox/router behavior, final B4 cancel outcomes, policy/UAA work, or terminal inference from local/process/transport state. | B2.1-1 durably exact-joins or creates the claim at acceptance, excludes ephemeral legacy registration, and transfers retained observation from the foreground; B2.1-2 journals exact canonical B0 frames/events before waiter delivery and rejects all non-exact order/replay; B2.1-3 recovers every nonterminal claim/cursor plus any exact accepted-but-unclaimed interrupted handoff and reconciles only from exact producer truth. Waiter drop preserves work. Only exact B0 terminal identity closes immutable observation. | Consumer clauses of `RG-EVENT-01`; `RG-SUP-01` observation/receipt clauses; `RG-SUP-02`; `RG-RECEIPT-03`; `RG-RECEIPT-04`; B2.1 clauses of `RG-OBS-01`. | Review-clean B1-3a/B1-3b receipt core, B0, and landed A1.1e; B1 need not yet be production-closed. | Joint B1/B2.1 production closeout; model-facing early return (`B2.2`); retained semantic envelope (`B3.1`); C1 cut; final cancel semantics; seam promotion. |
| **B1/B2.1 — Joint production integration closeout** | Prove the real ephemeral and retained accepted paths proceed from immutable B1 acceptance into B2.1 supervision without any legacy-writer attempt or observation gap. | ReceiptRegistry and Supervisor retain distinct ownership; WorldDispatchControl only integrates them. | All B1/B2.1 sections in `01`–`05`; the live acceptance, inspect, wait, cancel, drop, restart, and reconciliation paths. | RuntimeEventTransport; HostSessionAuthority physical storage; compatibility callers. | Production-path integration and focused tests within the union of the already authorized B1 and B2.1 code areas; closeout rows in this six-file pack only after proof. | No B3.1, C1, A1.2, B2.2, B4 final outcomes, policy, UAA, host transition, retained semantic, obligation, or early-return work. | Both accepted families survive caller drop/restart; no accepted work is followed by legacy active-task registration or activated-store rejection; inspect/wait and narrowly adapted cancel compatibility consume exact receipt/supervisor truth; terminality is exact; complete differential, doctor/smoke, and fresh review gates pass. | Applicable B1 and B2.1 gates, production legacy-writer exclusion, blocking compatibility, caller-drop/restart, `RG-OBS-01` acceptance/journal joins, and `RG-DIFF-01`. | Review-clean B1-3a/B1-3b plus review-clean B2.1-1, B2.1-2, and B2.1-3. | B3.1 and every later packet. |
| **B2.2 — Foreground receipt return** | Switch long-running ingress surfaces from a blocking compatibility wait to returning the already-durable, E2-complete receipt after supervisor handoff. | WorldDispatchControl for the ingress contract, consuming ReceiptRegistry and Supervisor truth unchanged. | `02` WorldDispatchControl/ReceiptRegistry/Supervisor rows; `04` receipt acceptance; `05` blocking receipt rows; tool-invocation async model. | RuntimeToolInvocationAdapter; InternalToolboxTransport; B3.2 retained lifecycle; B4 cancel. | `crates/shell/src/execution/orchestrator_world_dispatch.rs`; `crates/shell/src/execution/agent_runtime/{dispatch_contract.rs,tool_invocation_contract.rs}`; receipt-response consumption only in `crates/shell/src/repl/async_repl.rs`; focused early-return tests colocated in those exact files. | No new receipt identity or policy commitment, no journal/reconciliation change, no obligation semantics, no A2 episode demotion, and no retained lifecycle rewrite. | `run_world_task` and `continue_world_worker` return the exact accepted receipt only after durable B1 acceptance, E2 policy completion, and B2.1 handoff and before terminal exit; caller drop does not stop observation; blocking compatibility may remain only on explicitly named non-core UX. | `RG-RECEIPT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`. | A1, A2/A3, E2, B2.1, and C1. | B3.2 lifecycle/park completion, B4 cancel outcomes, C2/C3, and unrelated surfaces. |
| **B3.1 — Bounded retained-turn event/causation prerequisite** | Provide only the typed worker-to-host retained event envelope C1 requires, joined to B0 identity and B1 active-run truth, and normalize provider semantics at the producer before `AgentEvent` construction. | WorldWorkerMessagingProtocol. | `01` authority map/invariants 3–6; `02` MessagingProtocol row; `04` acceptance context + producer semantic normalization + retained event envelope; `05` `RG-MSG-01`/`RG-OBL-01`; messaging design envelope/thread/ordering/attention sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; RetainedWorkerRuntime; ObligationLedger. | Add `NormalizedWorldWorkerEventFacetV1`, the optional top-level `AgentEvent.worker_event: Option<WorldWorkerEventV1>`, and their typed fields in `crates/common/src/agent_events.rs`; consume B1's shared types from `crates/common/src/{authority_commitment.rs,lib.rs}`; consume/validate the existing B1 `WorldWorkAcceptanceContextV1` request extension in `crates/transport-api-types/src/lib.rs` without changing `ExecuteStreamFrame`; move the current provider-payload thread/class/attention/payload interpretation into one explicit fail-closed producer normalizer and shape the final member from that facet plus retained request/runtime context in exactly `crates/world-service/src/member_runtime.rs`; bounded host validation, deletion of C1-path JSON-pointer classification, and generic-journal handoff in `crates/shell/src/execution/orchestrator_world_dispatch.rs`; bounded host-facing semantic adapters in `crates/shell/src/execution/agent_runtime/dispatch_contract.rs`; focused tests colocated in those exact files. | No `ExecuteStreamFrame` variant/field change, no upstream provider-wrapper contract change, no new request-envelope semantics beyond consuming B1's context, no host or ledger identity/semantic inference from emitted `AgentEvent.data`, no producer-chosen eligible subset, no omission/untyped event/permissive downgrade for an unknown or malformed provider shape, no separate transport protocol, no host-to-worker message completion, no foreground early return, no worker park/cancel/stop/fork rewrite, no host-transition issuance/authentication/interpretation, no obligation classification/materialization, and no policy broadening. | After exact B1 acknowledgement, every retained `ExecuteStreamFrame::Event` through the terminal cut is in the closed B3.1 domain. Before `AgentEvent` construction, the producer normalizer maps every existing provider `AgentWrapperEvent` kind/payload into an exact typed facet for thread/class/attention/payload; parsing provider payload is permitted only at that named adapter boundary. Explicit supported non-attention shapes receive a typed non-attention class; missing, ambiguous, deferred, unknown, or malformed shapes fail the stream and C1 cut closed rather than being dropped, left untyped, or becoming progress/no-attention. The final typed `AgentEvent.worker_event` joins that facet with exact sources: acceptance ID, request/message causation and target backend come from the retained B1 context; session, source/target participant, source backend, active run, and world come from typed request/runtime context; B0 supplies event/frame identity. B3.1 validates that B2.1's generic event commitment equals the full canonical typed member and that all B0/B1/context fields match before ledger delivery; any opaque correlation is copied unchanged and compared only for equality. Host-side JSON-pointer classification is not a C1 input. Missing or mismatched identity fails closed; request ID or active-run ID never substitutes for transition intent/run. | Bounded producer/consumer clauses of `RG-MSG-01`; prerequisite clauses of `RG-OBL-01`; B3.1 event clause of `RG-OBS-01`. | B1/B2.1 joint production closeout and landed A1.1e. | Remaining `RG-MSG-01`, upstream provider-wrapper taxonomy cleanup, A1.2 production correlation supply, foreground retained receipt return, host-to-worker messaging, retained lifecycle/park/nonzero-exit behavior (`B3.2`), C1 materialization, and seam promotion. |
| **B3.2 — Retained receipt, messaging, and lifecycle completion** | Complete model-facing retained receipts, host-to-worker envelopes, delivery semantics, and accepted-turn lifecycle/park behavior without changing B3.1 event truth. | WorldWorkerMessagingProtocol owns message semantics; RetainedWorkerRuntime owns worker lifecycle. | `02` MessagingProtocol + RetainedWorkerRuntime rows; `04` retained receipt/manifest; full messaging and lifecycle designs. | Supervisor; SteeringPolicyEngine; ObligationLedger; B4 cancel. | `orchestrator_world_dispatch.rs`; `agent_runtime/{dispatch_contract,session,state_store}.rs`; `world-service/member_runtime.rs`; focused retained lifecycle tests. | No auto-attach, no broad lifecycle rewrite outside accepted turn/park, no obligation ownership transfer, and no dispatch narrowing beyond carrying E2 snapshot/cap refs. | Continue returns the B1 accepted active-run receipt completed by E2 through B2.2; host-to-worker messages preserve exact identity/causation; clean turn parks without losing worker continuity; failure is durable and exact. | Remaining `RG-RECEIPT-02`, `RG-MSG-01`, `RG-BASE-04`, `RG-WORKER-EXIT-01`. | A1, A2/A3, E2, B2.2, B3.1, and C1. | B4 cancel/inspect/stop, C2/C3, and auto-attach. |
| **B4 — Receipt-targeted cancel/inspect/stop** | Resolve control against active receipts and worker manifests with distinct durable outcomes. | WorldDispatchControl resolves the verb; ReceiptRegistry owns immutable accepted identity; Supervisor owns active observation and terminal truth; RetainedWorkerRuntime owns worker stop lifecycle. | `04` cancel categories + supervisor cancellation rules; `02` WorldDispatchControl/ReceiptRegistry/RetainedRuntime rows; `05` cancel/closeout rows. | HostSessionAuthority; Supervisor; private transport episode status. | `orchestrator_world_dispatch.rs`; `agent_runtime/{state_store,dispatch_contract,control}.rs`; world-service cancel seam; control tests. | No weakening exact identity, no treating no active run as stale linkage, and no stop/cancel conflation. | Same-turn continue→cancel targets active run; no-active, terminal, unreachable, invalid, mismatch, ambiguity, and policy denial are distinct; closeout is idempotent. | `RG-CANCEL-01`, `RG-CANCEL-02`, `RG-CLOSE-01`, `RG-BASE-04`; B4 cancel clause of `RG-OBS-01`. | B2.2 and B3.2. | C2/C3 and unrelated lifecycle/policy work. |

## Track C — Obligations, inbox, auto-attach, and router attach

| Slice | Goal | Authority owner | Must-read sections | Sibling context | Exact allowed code areas | Explicit non-goals | Exit gate | Regression gates | Prerequisites | Gates deliberately deferred |
|---|---|---|---|---|---|---|---|---|---|---|
| **C1 — Event-to-obligation materializer and semantic cut** | Consume durable exact events, idempotently create canonical obligations while the supervisor observes active work, and own the monotonic ledger revision/materialized-event cut plus closed snapshot query consumed by A1.2. | ObligationLedger. | `01` invariants 4–6; `02` RuntimeEventTransport/Supervisor/Messaging/Obligation rows; `04` runtime carrier, supervisor rules, and `ObligationLedgerSnapshotReadV1`; `05` `RG-OBL-01`/`RG-OBL-02`; obligation-ledger producer/dedupe sections. | RuntimeEventTransport; ReceiptRegistry; Supervisor; MessagingProtocol; HostSessionAuthority consume-only client. | `agent_runtime/obligation_ledger.rs`; bounded consumer integration from the receipt-scoped supervisor journal; bounded StateStore persistence only; focused materialization/cut/snapshot tests. | No runtime identity generation, receipt acceptance, observation ownership, retained-envelope or host-transition semantics, producer event eligibility choice, inbox rendering, router launch, direct prompt injection, or transfer of obligation semantics to HostSessionAuthority/StateStore. Stream exhaustion, EOF, timeout, PID/helper/socket state, inbox rows, pending counts, worker flags, and compatibility projections are forbidden completeness inputs. | Each B3.1 attention event creates exactly one canonical obligation before the exact B0 terminal event when applicable; duplicate journal replay creates none. Before classification and snapshot capture, C1 verifies that every post-acknowledgement retained B2.1 `Event` journal ref through the cut commits one full canonical B3.1 target/source/thread/class/attention/request/message/transition/payload envelope. The ledger advances one monotonic session revision and materialized-through event watermark, returns Pending before coverage, then returns a Complete snapshot binding exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream/terminal event, and the complete ordered exhaustive join of all retained `Event` refs through the cut even for `NoUnresolvedAttention`, plus unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run, authority revision, disposition, and sorted canonical-record commitments. Any untyped/omitted/substituted event or stale/mismatched scope, correlation, revision, cut, disposition, or record commitment keeps the cut non-Complete. | `RG-OBL-01`, C1 clauses of `RG-OBL-02`, obligation clauses of `RG-SUP-01`, bounded consumer clauses of `RG-MSG-01`, and C1 clauses of `RG-OBS-01`. | A1.1e, B0, B1, B2.1, and B3.1. | A1.2 production correlation supply and consumption/adoption, C2 projections, C3 router behavior, B2.2 foreground early return, and every seam promotion. |
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
