# Phase and Slice Map

## Sequencing rules

Each slice is independently reviewable. A0 is a diagnostic inventory; every later implementation slice must move one authority boundary plus proof. A slice may read sibling context without editing sibling ownership.

Hard dependency spine:

```text
A0 -> A1 -> A2/A3 -> B1 -> B2 -> B3/B4 -> C1 -> C2 -> C3
                       \-> E2
D1 -> D2 -> D3
E1 -> E2 -> E3 -> E4
D2 and E1 must agree on PolicySnapshotV3, but may land in either order behind fail-closed gates.
```

Cross-packet integration hold: a lower-level A1 persistence packet may expose a preexisting
production bypass that only a later canonical owner packet can close. For the exact parked-
successor blocker recorded in `05-debug-regression-ledger.md`, A1.1d implementation is available
to A1.1e without being promoted to integrated closeout. The failing regression remains explicit,
stale lifecycle/world-binding behavior remains rejected, the feature branch remains non-landable,
and A1.2/A1.3 must close the gate before A1 can complete. This exception creates no A1.1d-6,
waives no gate, and does not apply to another blocker.

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
| **A1.2 — intent issuance/claim/application** | Add the complete durable `HostSessionTransitionIntentV1` protocol under `HostSessionAuthority`, including greenfield-certificate-checked production Start `ExpectedAbsent`, atomic Start reservation and issuance, claim, single application and initial authority birth, exact-retry join, expiry/rejection, crash reconciliation, revisioned input handoff, and retained transport reprojection; keep the plan a transport projection. This packet owns parked-successor `Attach`/`ResumeOneTurn` protocol closure. | Authority-store core files above; only `HiddenOwnerHelperLaunchPlan` plus its write/load/remove/launch integration in `agent_runtime/control.rs`; focused Start reservation/birth, intent lifecycle, serialization, persistence, helper-loss, parked-successor, input-handoff, payload-retention/reprojection, and plan-commitment tests. | Exact greenfield certificate and empty pre-A1 collections; unique `intent_id`; intent/state revision; `Start`/`Attach`/`ResumeOneTurn`; exact enumerated postures; `ExpectedAbsent` or exact authority revision/hash; reservation ownership; identity/home/workspace/world/descriptor/object commitments; expiry; canonical payload and transport refs/hashes; `Issued -> Claimed -> Applied` plus terminal `Rejected`/`Expired`; immutable application result; input handoff; transport release. | No legacy-state conversion or compatibility-derived absence; no CLI/REPL consumer switch yet; no destructive-read consumption; no endpoint/path redesign; no helper removal; no general transport cleanup; no PID/helper/handle/prompt-derived transition eligibility. | In addition to the existing lifecycle/crash matrix, prove `Attach` and `ResumeOneTurn` start from exact current parked or other enumerated authority, preserve session lifecycle identity and exact world binding, revision-authorize one successor lineage, and never derive eligibility from PID, helper, handle, readiness, or prompt state. Exact stale successor retries fail or join exactly; failure after application reconciles from intent/application truth without restoring a stale snapshot. Exit when Start and parked-successor application are single, atomic, replay-safe authority operations; A1.3/A1.4 still own exclusive real-producer/consumer adoption. | Real Start/Attach/Resume consumer adoption; bounded auto-attach adoption; full real-path `RG-AUTH-03`; `RG-BASE-01`; A1 completion. |
| **A1.3 — Start/Attach/Resume consumer adoption** | Route public CLI issuance, `run_owner_helper` consumption, owner-helper application, and the real REPL `Start`/`Attach`/`ResumeOneTurn` transitions through the authority and claimed intent. This packet owns real-path parked-successor adoption and `RG-BASE-01` closure. | The A1.2 plan integration; only the named intent issuance/consumption regions in `execution/agents_cmd.rs`; only owner-helper validation/application, real REPL adoption, and explicit bootstrap-home threading in `repl/async_repl.rs`; only the A1.1 explicit-home resolver entry points needed by those named CLI/REPL paths; focused CLI, helper, and REPL integration tests. | Mode-specific preconditions; exact caller/source/target lineage; bootstrap-home/workspace/world binding; typed descriptor/attach/resume/policy/input refs; claim ownership; authority/intent revisions; applied result and one-turn post-turn disposition. | No resolver precedence, policy/config/inventory interpretation, or broad REPL lifecycle cleanup; no episode model rewrite; no private endpoint/path redesign; no receipt, supervisor, or member-runtime work. | Prove a public turn from parked truth issues/applies `ResumeOneTurn`, explicit reattach issues/applies `Attach`, and real helper/REPL launch occurs only after durable application. Startup failure reconciles without returning the session to `Allocating`; exact retry joins; the exact public start → turn/reattach → stop path becomes green. Retain existing negative tests for helper loss, stale revision, ambient-home/CWD reread, substitution, role mismatch, and duplicate application. | Bounded auto-attach producer adoption; complete A1 regression closure; A1 completion. |
| **A1.4 — bounded auto-attach producer adoption and regression closure** | Make the existing auto-attach launch-plan producer issue/reference the same exact `Attach` intent, then close A1 without changing projection policy or settlement ownership. | Only `build_auto_attach_launch_plan` and directly required plan integration in `agent_runtime/auto_attach.rs`; focused auto-attach producer/consumer tests; final A1 CLI/REPL/regression/smoke wall. | Exact `Attach` precondition/revision, claim identity, immutable payload hash, host/session/binding/descriptor commitments, and idempotent applied result. | No auto-attach policy, eligibility, claim, or settlement redesign; no router responsibility expansion; no endpoint/path redesign; no A2 demotion; no subsequent-slice work. | Prove manual and auto-attach cannot substitute or double-apply an intent, retry/restart converges, every A1-scoped clause of `RG-AUTH-01`/`RG-AUTH-02`, all of `RG-AUTH-03`, and `RG-BASE-01`/`RG-BASE-02` pass on real CLI and REPL paths; only then may A1 close. | No A1 gate remains claimable until this packet's final wall passes; the ledger-wide `RG-AUTH-01`/`RG-AUTH-02` rows remain unresolved for A2/A3, and later sibling-seam gates remain out of scope. |

#### A1.1 internal green subpacket decomposition

A1.1 is implemented through the following fixed, independently reviewable, sequential green subpackets. Moving code into the bounded module directory is not itself a completed subpacket. Each subpacket must add its own test-first or test-alongside proof, pass targeted negative cases, formatting, and diff checks, and receive fresh review before the next begins. None may claim the complete A1.1 exit gate until A1.1a–A1.1e all pass together.

| Subpacket | Reviewable outcome | Boundary and proof focus | Gates not yet claimable |
|---|---|---|---|
| **A1.1a — repository-owned canonical JSON codec and hash-vector tests** | One closed `CanonicalJsonV1` encoder/decoder implements the exact A1 field, enum, integer, string, timestamp, unknown/duplicate-field, and UTF-8 rules without delegating canonical meaning to a presentation serializer. | Bounded codec modules under `agent_runtime/host_session_authority/`; normative bytes/digest fixtures, alternate-encoding rejection, duplicate/unknown-field negatives, integer boundaries, explicit-null options, and every named A1 hash wrapper required by A1.1. | Trusted filesystem, bootstrap/root/key/object persistence, transaction preflight, CAS, exact resolution, and the complete A1.1 gate. |
| **A1.1b — private opened-directory/openat trusted-store filesystem boundary** | One private trusted-root handle binds `CanonicalDirectoryV1`; after opening it, every authority descendant operation is directory-relative and no-follow, never path-re-resolved or ambient-CWD-derived. | Bounded trusted-filesystem modules; owner/mode/ACL, physical identity, same-filesystem, symlink/reparse, component replacement, scan-to-publication replacement, atomic no-replace/root replacement, file/directory `fsync`, and unsupported-platform fail-closed proof. | Root/key/object semantic persistence, transaction preflight, CAS, exact resolution, and the complete A1.1 gate. |
| **A1.1c — root/key/object bootstrap and deterministic crash reconciliation** | Greenfield initialization, pending recovery, immutable certificate, key lifecycle, typed object publication/verification, retained orphan exact adoption, and closed temp grammar converge deterministically across every specified crash window. | Bounded store modules using only A1.1a/A1.1b primitives; exact fresh/pending/existing/legacy/corrupt classification plus marker/key/root/object/temp crash matrices. | Centralized semantic preflight, writer exclusion, authority CAS, exact resolution, and the complete A1.1 gate. |
| **A1.1d — centralized transaction preflight, cross-process CAS, and legacy/direct-writer exclusion** | Every authority read or mutation passes one semantic preflight; every applicable pre-A1 session/participant read-decide-write transaction retains one opened trusted physical root, directory-relative descendants, exact identity, activation observation, and the same root lock through final file/directory `fsync`; every legacy/direct writer fails before mutation after activation; root and authority writes use expected-revision CAS across processes. | Bounded authority-store transaction modules and bounded new modules beneath `host_session_authority/` when needed for independent reviewability; only the minimum `state_store.rs` persistence/integration choke points required to route all guarded flat/canonical snapshots, leases, removals, compatibility/read-repair persistence, and parent-session mutations through directory-relative/no-follow operations. Include post-root legacy insertion, root rename/replacement/rebind with an untouched replacement tree, unknown tree/key/object state, stale/conflicting subprocess writers, zero-mutation rejection, and contention proof. No unrelated StateStore extraction or semantic redesign is authorized. | Facade integration, complete exact-resolution proof, and the complete A1.1 gate. |
| **A1.1e — HostSessionAuthority facade integration and exact-resolution proof** | Establish the exact facade as the sole A1 authority-store API, consume the same opened bootstrap-home binding as bounded config/policy/inventory entry points, resolve exact durable identity and current authority revision/hash without PID/socket/helper/prompt truth, and expose only the A1.1 primitives required by A1.2. Under the exact cross-packet integration hold above, this work may proceed while A1.1d integrated closeout remains open; it does not repair the real successor path or claim `RG-BASE-01`. | Bounded facade/integration modules, explicit-home entry points, exact session/store/workspace/world/lineage/ref resolution, stale observation rejection, cross-home/CWD negatives, and the full available A1.1 regression wall with the deferred public lifecycle failure kept explicit. | Production `ExpectedAbsent`, Start reservation/intent/application, real caller adoption, parked-successor repair, `RG-AUTH-03`, `RG-BASE-01`, A1.1d integrated/cross-platform closeout, and A1 completion. |

### A1.1d internal review checkpoints

A1.1d remains the canonical packet. The five labels below are sequential internal review checkpoints
under A1.1d, not independent slices, not replacements for A1.1d, and not separate closeout gates.
The A1.1d packet closes only after all five checkpoints pass together. The exact cross-packet
integration hold above permits A1.1e facade/resolution work to begin while integrated and platform
closeout stay open; it does not close A1.1d, create A1.1d-6, or permit A1.2/A1.3 work early.

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
- A1.1e is implementation-ready after this documentation correction and has not begun.
- A1.2 remains blocked on A1.1e and owns durable transition-protocol closure. A1.3 remains blocked
  on A1.2 and owns real CLI/helper/REPL adoption plus `RG-BASE-01` closure.
- A1.4 is not ready. A1 remains incomplete and non-landable until the deferred lifecycle gate, the
  complete Linux wall, required native macOS proof, A1.4, and all final A1 gates pass.

## Track B — World-dispatch receipts, supervision, and cancel

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **B1 — Receipt registry and acceptance records** | Persist `ActiveEphemeralTaskReceiptV1` from the exact runtime acceptance source and make it inspectable without yet changing foreground blocking behavior. | `02` WorldDispatchControl + ReceiptRegistry rows; `04` active receipt, receipt acceptance source, and snapshot acceptance rules; `DESIGN-host-orchestrator-tool-invocation-surface.md` receipt model | HostSessionAuthority; EffectivePolicyResolver; B2 supervisor handoff | `execution/orchestrator_world_dispatch.rs`; `agent_runtime/{dispatch_contract,tool_invocation_contract,state_store}.rs`; focused transport tests | No retained-turn receipt yet; no foreground early return; no UAA mediation; no terminal-only event relabeled as acceptance. | Accepted receipt is durably persisted from a valid pre-terminal runtime acknowledgement, survives caller drop/restart, and is inspectable by exact task ID in tests. Existing blocking UX may remain until B2. | `RG-RECEIPT-03`, `RG-RECEIPT-04`, `RG-BASE-02` |
| **B2 — Supervisor handoff and foreground early return** | Transfer terminal-framed stream ownership, event persistence, reconciliation, and closeout to a restart-safe supervisor, then return the accepted receipt before exit. | `04` receipt acceptance source + supervisor rules; `02` supervisor row; `05` blocking and obligation-timing rows; internal toolbox transport terminality section | ReceiptRegistry; MessagingProtocol; ObligationLedger | bounded supervisor module under `crates/shell/src/execution/`; `orchestrator_world_dispatch.rs`; transport client/types; supervisor tests | No router behavior; no provider-specific policy broker; no new event taxonomy beyond required causation fields. | Foreground returns only after durable receipt/observation handoff and before terminal exit; supervisor continues; duplicate frames are idempotent; crash/restart resumes or reconciles; terminal closeout is monotonic. | `RG-RECEIPT-01`, `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`, `RG-OBS-01` |
| **B3 — Retained turn receipts and messaging** | Add `ActiveRetainedTurnReceiptV1`, typed durable message/event envelopes, and worker lifecycle ownership for accepted turns. | `02` MessagingProtocol + RetainedWorkerRuntime rows; `04` retained receipt/manifest; `DESIGN-retained-world-worker-messaging-and-steering-contract.md`; `DESIGN-world-worker-lifecycle-model.md` | Supervisor; SteeringPolicyEngine; ObligationLedger | `orchestrator_world_dispatch.rs`; `agent_runtime/{dispatch_contract,session,state_store}.rs`; `world-service/member_runtime.rs`; focused tests | No auto-attach; no broad lifecycle rewrite outside accepted turn/park path; no dispatch narrowing beyond carrying snapshot refs. | Continue returns accepted active-run receipt before exit; worker remains routable after clean turn; events retain exact target/thread/causation identity. | `RG-RECEIPT-02`, `RG-MSG-01`, `RG-BASE-04`, `RG-WORKER-EXIT-01` |
| **B4 — Receipt-targeted cancel/inspect/stop** | Resolve control against active receipts and worker manifests with distinct durable outcomes. | `04` cancel categories + supervisor cancellation rules; `02` WorldDispatchControl/ReceiptRegistry/RetainedRuntime rows; `05` cancel and closeout rows | HostSessionAuthority; supervisor; private transport episode status | `orchestrator_world_dispatch.rs`; `agent_runtime/{state_store,dispatch_contract,control}.rs`; world-service cancel seam; control tests | No weakening exact identity; no treating no active run as stale linkage; no stop/cancel conflation. | Same-turn continue→cancel targets active run; no-active, terminal, unreachable, invalid, mismatch, ambiguity, policy denial are distinct; closeout is idempotent. | `RG-CANCEL-01`, `RG-CANCEL-02`, `RG-CLOSE-01`, `RG-BASE-04` |

## Track C — Obligations, inbox, auto-attach, and router attach

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **C1 — Event-to-obligation materializer** | Persist runtime events and idempotently create canonical obligations while the supervisor is still observing active work. | `02` ObligationLedger row; `04` supervisor idempotency; `DESIGN-durable-orchestration-obligation-ledger.md`; messaging design attention sections | Supervisor; MessagingProtocol; HostSessionAuthority posture derivation | `agent_runtime/obligation_ledger.rs`; receipt/supervisor event path; bounded StateStore persistence | No inbox rendering; no router launch; no direct prompt injection. | Attention event creates one obligation before terminal exit; duplicates do not duplicate it; `awaiting_attention` derives from unresolved obligations. | `RG-OBL-01`, `RG-OBL-02`, `RG-SUP-01` |
| **C2 — Inbox and auto-attach projections** | Make inbox and attach eligibility pure projections over obligation state and effective policy. | `02` InboxProjection + AutoAttachProjection rows; notification inbox design; auto-attach trigger design | ObligationLedger; CompatibilityReadModel; SteeringPolicyEngine | `agent_runtime/{host_inbox,obligation_ledger,auto_attach,state_store}.rs`; `host_inbox_materialization.rs`; projection tests | No host process launch; no worker action; no removal of compatibility ingress without migration proof. | Deleting/rebuilding projections does not lose obligation truth; session-coalesced claim is deterministic; wrong-host and policy denial fail closed. | `RG-OBL-02`, `RG-ATTACH-01`, `RG-BASE-03` |
| **C3 — Router ownership restoration** | Consume attach-eligible claims, restore one sanctioned host episode, settle the claim, and stop. | `02` RouterAttachTrigger row; router integration design responsibilities/non-responsibilities; `04` HostExecutionEpisodeV1 | HostSessionAuthority; SurfaceAdapter; AutoAttachProjection | `agent_runtime/auto_attach.rs`; bounded router entrypoint; helper launch adapter; router tests | No prompt replay; no approval/answer/fork/continue; no always-running backend assumption. | Router produces one attach outcome per session claim and cannot invoke worker-control verbs; manual reattach coexists without duplicate ownership. | `RG-ATTACH-01`, `RG-ATTACH-02`, `RG-AUTH-01` |

## Track D — UAA execution envelope and side-effect mediation

Staging rule: the managed gateway secure-FD carrier is already landed and must be preserved. Until D2 and direct world-UAA adoption of that carrier land, D1 fail-closed behavior applies to world-scoped side-effect-capable or credential-requiring UAA operations that claim Substrate policy mediation. Existing direct world-UAA behavior may remain only behind an explicitly named and logged compatibility mode. Copied host credentials/config must be identified as a compatibility copy bridge. Compatibility mode must be excluded from `ContractCorrectAndProven` claims and from mediation, caging, gateway-adoption, or credential-handoff acceptance evidence.

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **D1 — World adapter execution envelope** | Materialize and persist `WorldRuntimeAdapterExecutionEnvelopeV1` with guest entrypoint, projection identity, world binding, policy snapshot, broker requirement, credential posture, secret-handoff ref, and explicit mediation/compatibility posture. | `02` envelope + realization rows; `04` envelope and `LaunchTimeSecretHandoffV1` contracts; existing gateway auth-handoff internals/verification docs; Codex home/auth mapping design; config projection design | AgentConfigProjectionService; EffectivePolicyResolver; RetainedWorkerManifest; in-world Substrate gateway | `agent_runtime/{validator,dispatch_contract}.rs`; `execution/agent_inventory.rs`; `world-service/member_runtime.rs`; existing managed gateway launch/handoff types; `config/agents/*`; envelope tests | No per-operation broker yet; no host runtime behavior change; no reimplementation of the existing FD carrier; no unnamed credential fallback; no compatibility evidence counted as contract proof. | Host/world envelopes are distinct; world Codex cannot use host path; sibling workers cannot alias envelope/projection identity. Every world envelope records `SecureGatewayHandoff`, `CompatibilityCopyBridge`, or `NoCredentialsRequired`; secure posture joins the existing carrier to the exact gateway receiver and non-secret evidence; Codex receives that gateway's endpoint/session contract; compatibility copy is named/logged/non-promotable; credential-requiring mediated mode fails closed without exact adoption. | `RG-UAA-01`, `RG-UAA-02`, `RG-CONFIG-01`, `RG-CONFIG-03`, `RG-CONFIG-04` |
| **D2 — WorldCommandExecutionBroker** | Mediate every world-UAA side-effect intent through the accepted `PolicySnapshotV3` and existing world-service enforcement path. | `02` broker row; `04` envelope and snapshot acceptance; `01` invariants 8–11 | EffectivePolicyResolver; runtime-family adapters; world-service enforcement | `crates/gateway`; relevant `agent-api-*`; `world-service` execute/guard/fs/network paths; bounded shell adapter glue | No parallel sandbox; no env-only enforcement; no shell-only claim while edit/MCP/write channels bypass. | Shell, apply/edit, direct write, write-capable tool/MCP, process, and network channels are brokered or disabled; snapshot mismatch/unbrokered intent fails closed. | `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-02` |
| **D3 — Codex/UAA end-to-end closure** | Prove world Codex/UAA placement, adoption of the existing secure gateway credential bootstrap, and side-effect mediation, including non-zero turn closeout and diagnostics. | `05` credential carrier/adoption, UAA caging, external-sandbox, and non-zero rows; `04` secret handoff + supervisor terminal rules; Codex mapping design | Supervisor; broker; envelope; config projection; in-world gateway | targeted Codex/UAA runtime integration tests and smoke harnesses; only defects exposed within D1/D2/E3 boundaries | No broad provider feature expansion; no FD-carrier rewrite without a separately proven defect; no advisory-only success; no copied-credential compatibility evidence counted as proof; no suppression of non-zero exits. | Real worker turn points Codex at the managed gateway, joins its one-time credential consumption to the exact envelope, creates no copied secret file or inherited child FD, proves caged/uncaged policy contrast and no bypassing edit/tool channel, records durable failure on non-zero, and retains session authority; traces contain handoff state but no secret material. | `RG-UAA-03`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-WORKER-EXIT-01`, `RG-OBS-01` |

## Track E — Dispatch-scoped policy narrowing and config projection

| Slice | Goal | Must-read docs | Sibling context | Allowed code areas | Explicit non-goals | Exit gate | Regression gates |
|---|---|---|---|---|---|---|---|
| **E1 — Restricted world_fs narrowing** | Accept a request-scoped restricted `PolicyPatch.world_fs`, validate path-containment monotonicity, and materialize a canonical narrowed snapshot. | `02` EffectivePolicyResolver + NarrowingPatch rows; `04` narrowing contract/rules; steering policy matrix capability section | WorldCommandExecutionBroker; receipt acceptance; agent inventory overlay logic | `crates/broker`; `execution/{policy_model,policy_snapshot,agent_inventory}.rs`; transport API policy types; resolver tests | No new filesystem policy model; no broadening dimensions; no receipt/manifests yet. | Gate=false rejects; gate=true accepts only narrowing; file-under-directory containment works; escapes/symlinks/broadening fail closed. | `RG-POLICY-01`, `RG-POLICY-02` |
| **E2 — Policy commitments on work and workers** | Persist immutable active-run snapshots and retained-worker caps; recompute future turns as parent ∧ cap ∧ turn patch. | `04` receipt/manifest and immutable snapshot rules; `02` ReceiptRegistry/RetainedRuntime rows | B1/B3 receipts; E1 resolver; fork lifecycle | receipt/manifest persistence; `orchestrator_world_dispatch.rs`; retained lifecycle code; policy tests | No mutation of accepted snapshots; no automatic worker broadening; no config rendering. | Receipts/manifests record hash/ref/revision/reason; parent narrowing affects future turns; parent broadening does not widen worker; fork inherits cap. | `RG-POLICY-03`, `RG-RECEIPT-02`, `RG-CANCEL-01` |
| **E3 — AgentConfigProjectionService and gateway adoption** | Make logical/effective/native config projection first-class per retained worker and point world Codex at the existing in-world gateway secure-FD credential/session boundary. | `02` config projection + envelope + realization rows; `04` envelope projection, existing-carrier adoption note, and `LaunchTimeSecretHandoffV1`; existing gateway auth-handoff internals/verification docs; config projection, Codex mapping, and workspace overlay designs | Envelope; retained manifest; EffectivePolicyResolver; in-world gateway; RuntimeFamilyRealizationAdapter | `execution/agent_inventory.rs`; `crates/codex`; `world-service/member_runtime.rs`; existing in-world gateway endpoint/session integration points; bounded projection module; projection tests | No reimplementation of the landed FD carrier; no ambient runtime file as authority; no secret payload in projected files, manifests, traces, or UAA child FDs; no workspace-sync policy decision. | Sibling workers get isolated non-secret projections; `CODEX_HOME`, `config.toml`, and provider wiring can be rebuilt from Substrate truth plus accepted policy; Codex uses the exact managed gateway whose credentials arrived through the existing one-time FD carrier; copied auth/config runs only as named/logged/non-promotable compatibility; narrowed hints never substitute for enforcement. | `RG-CONFIG-01`, `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02` |
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
