# Debug Regression Ledger

## How to read this ledger

- **Resolved baseline** means a specific observed failure has a trustworthy fix or live proof that must not regress. It does **not** promote the surrounding architecture seam.
- **Partially resolved** means a narrow behavior works while the target ownership model remains wrong or unproven.
- **Unresolved** means the target behavior lacks an implementation and proof gate.
- Historical diagnoses are retained only when they define a permanent negative or regression test.

Primary source memos:

- [`../../RUN_WORLD_TASK_DEBUG_CANONICAL.md`](../../RUN_WORLD_TASK_DEBUG_CANONICAL.md)
- [`../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md`](../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md)
- [`../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md`](../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md)

## Canonical issue ledger

| Gate ID | Debug issue | Current classification | Repo-truth basis | Permanent regression gate | Owning slices |
|---|---|---|---|---|---|
| **RG-HOME-01** | Private home creation and world capability parity | **Unresolved product gate; R1 review-clean, R3/product proof open** | The authority transaction range and focused final-root implementation remain preserved. The `b29897e0` Linux audit proved masked named access `--x` and `r-x` were rejected despite no effective write authority; a zero-effective default ACL could be inherited into an invalid candidate; and diagnostics named the requested final home instead of the actual ancestor. Case A and runtime commit `4d0acff68e20d86b97fe5367b8a4617554f33ef4` correct Linux V1 from unprovable physical ACL absence to no effective other-principal authority: exact owner/`0700`, observable final-root access/default ACL rejection, qualified `NoData`, no-follow identity/replacement, owner-only descendants, and no-repair remain mandatory. Integrated Linux and native macOS closeout remain open. | Preserve the review-clean R1 closed `Present`/`NoData`/`Failed` observations, strict Linux POSIX access-ACL parsing and mask evaluation, masked non-writing support, effective-write/default/final-root/failure rejection, and exact bounded diagnostics without physical-absence wording or sensitive principals. R3 may remove only an empty current-attempt candidate after descriptor-bound exact-identity rejoin; it never recursively removes a candidate and fails closed on replacement, nonempty state, ambiguity, or pre-existing provenance. Preserve the descriptor-bound creation/open/identity/replacement wall and malicious-root/same-UID exclusions. Then run the complete Linux regression/product wall and later native macOS proof, including filesystem, network, config/dependency, gateway, runtime, shim, replay, trace/diagnostic, PTY/non-PTY, and lifecycle/world-binding parity. | A1.1d-5R1, A1.1d-5R3 |
| **RG-INSTALL-01** | Selected install authority context and lifecycle convergence | **Unresolved; A1.1d-5I proves R2/R3 implementation is required** | Normal Unix dev custom-prefix install fails at shim deploy because only `SUBSTRATE_ROOT` is passed; release/dev Unix and Windows paths contain additional ambient-home/context omissions. Unix release service sandbox allowlisting derives from ambient `$HOME`, not the selected home. Unix and Windows uninstall can target ambient/wildcard state, generated manager scripts prefer conflicting ambient home, and Linux cleanup omits prefix-local and system-managed artifacts. Shim doctor reports a default-home trace path under a selected custom home. No native macOS/Windows proof is claimed. | Carry one host `InstallBootstrapContextV1` across private-home bootstrap, shim deploy/remove/status/doctor, generated consumers, uninstall, rollback, and every sudo boundary; bind distinct Lima/WSL/native realization through `PlatformBootstrapMappingV1`. Prove fresh default and custom-prefix install without outer overrides; conflicting ambient home; repeat install; accepted-home/later-stage partial failure and synchronous safe rollback rerun; uninstall/reinstall symmetry; fail-closed unaccepted-invalid-candidate behavior; no wildcard or ambient-home deletion; exact prefix/system managed cleanup and pre/post service parity; release full-world custom prefix outside ambient home; and world/shim/health doctor plus full Linux product smoke. Static platform parity is not native proof. | A1.1d-5R2, A1.1d-5R3 |
| **RG-AUTH-01** | Helper/session authority and orphaned helper behavior | **Unresolved architecture; narrow continuity behavior partially resolved** | The A0 inventory in `02-seam-crosswalk.md` traces initial attached-posture birth, home/env/CWD-selected state/config/policy/inventory, helper-plan payload plus mode and parent-role selection, launch/join coordination, exit/retry, startup-prompt and auto-park delivery, process-local startup acceptance, private-stop-ownership mode, public-stop delivery gating, terminal-loss park/stop choice, parking/reconciliation, fork-successor allocation, resumed-turn release, auto-attach verification, endpoint-registration startup failure, stream/process completion, whole-session snapshot arbitration, process-local snapshot-write arbitration, episode-local replacement lineage, and world-binding birth/repair/replacement. The parked-successor diagnostic additionally proves that current `Active`/`ParkedResumable` authority survives with no attached episode and that a reconstructed successor snapshot is not authority. Session/posture logic remains spread across `orchestration_session.rs`, `state_store.rs`, `agents_cmd.rs`, `control.rs`, `async_repl.rs`, and the in-world member registry. | Kill or orphan each episode kind after durable authority exists. Exact session, parked posture, world binding, retained workers, receipts, and obligations remain. Process/helper/prompt state cannot create or regress ownership, episode absence does not erase parked authority, and a stale episode cannot rewrite a newer authority revision. | A0, A1, A2, A3 |
| **RG-AUTH-02** | Process/socket liveness used as durable truth | **Unresolved** | A0 classifies the exact PID/ownership/attachment/heartbeat/socket and ambient-host consumers: constructor-created attached truth, posture/session discovery, helper plan/mode/parent-role gating, participant and parent-session snapshot freshness, process-local snapshot-write arbitration, relative/fallback/re-read authority and workspace paths, effective-config env overrides, UID-selected binding birth/repair/replacement, process-local dispatch and auto-park registries, toolbox cancel's PID-filtered preflight, retained-runtime reuse/exit/replacement, naming-prefix continuity, daemon-memory admission/routing, helper reconciliation, private-stop-ownership-gated startup/parking, public-stop delivery gating, fork/resume PID-zero sentinels, endpoint/startup-prompt state used as durable truth, remote member startup around transport acceptance, fork lineage gated by stop-socket publication, lossy/long-path endpoint identity, last-writer toolbox rebind, hostname-targeted auto-attach, and private-stop fallback. The exact blocker shows `PID=0`, no handle, completed prior prompt, and pending successor prompt are non-authoritative observations. | Table-test every authority use of PID/socket/heartbeat/attached-client/helper/handle/readiness/prompt/ambient-path state and competing process-local writers: availability may classify delivery, but neither presence nor absence may create, delete, regress, rebind, or terminalize durable truth. Episode absence must not erase current parked authority. | A0, A1, A2, A3 |
| **RG-AUTH-03** | Revision-bound host transition intent proof | **Unresolved** | `HiddenOwnerHelperLaunchPlan` currently carries authoritative `Start`/`Attach`/`ResumeOneTurn` mode, identity, lineage, workspace/world binding, descriptor, attach contract, resume handle, and input through arbitrary JSON. Plan load/removal and helper execution do not join a durable authority revision or greenfield-certificate-proven expected absence to a unique replay-safe intent. The parked-successor regression proves stale lifecycle/world-binding rejection is correct: pre-A1.1d `c800436d` and pre-rejection `f73e8a81` passed only while stale whole-session overwrite remained possible; `bdb1796d` and later revisions fail closed. A1.1d-1 through A1.1d-4 remain preserved and A1.1d-5 is focused-review clean on Linux, but integrated and native macOS closeout remain open; no seam promotion is claimed. | First prove A1.1 primitives without claiming production Start: genuinely absent versus initialization-pending, valid-existing, unsupported-pre-A1, and corrupt/unsupported roots; the same physical bootstrap home persisted in marker/root/certificate and used for StateStore/config/policy/inventory, including copied-home rejection; recognized temp cleanup and fail-closed unknown temps across crashes before/after temp fsync and rename; component-by-component no-follow traversal and complete safe enumeration of both pre-A1 authority collections, including ancestor identity/owner/mode/ACL and scan-to-publication replacement; any artifact rejected without parsing/conversion and any unreadable or unsafe collection failed closed. Every pre-A1 read-decide-write transaction must retain the opened physical root, descendants, exact identity, activation observation, and root lock through final `fsync`; a subprocess rename/replacement/rebind after lock acquisition must fail closed with the replacement root untouched and no successful transaction outcome. Post-activation writers reject before mutation and preactivation writers serialize on that same root lock. Exact retry treats reservations, tombstones, intents, issuer indexes, application journals, and object indexes as semantic occupancy and joins only complete matches. Rotation and retirement revalidate the complete reconciled candidate immediately before publication. Crash/restart, contention, temp reconciliation, publication durability, and exact retry converge. Then prove the A1.2 API alone can accept `ExpectedAbsent` only from the verified greenfield certificate, empty pre-A1 collections, and exact absence of the complete namespace-map key; it atomically issues its self-owned reservation and creates initial authority only while applying that intent. A1.3/A1.4 must still prove exclusive real public CLI/REPL/auto-attach adoption. Prove `Attach`/`ResumeOneTurn` from exact current parked or other enumerated authority against exact revision/hash with exact caller/source/target lineage, bootstrap-home/workspace/world binding, descriptor/attach/resume/policy/input commitments, unique intent/request identity, fixed expiry, canonical payload hashes, and retained transport reprojection. Application preserves lifecycle identity and world binding, revision-authorizes successor lineage and immutable input handoff, and never derives eligibility from PID/helper/handle/readiness/prompt state. Exercise every intent/input state and crashes after initialization marker/key/root publication, key publication/root selection/retirement, object publication, issue, plan load/removal, claim, authority commit, input acceptance, `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit with transport present/already absent. Exact retry joins without a second namespace, authority revision, participant allocation, or delivery. Stale revisions, substituted plans, cross-kind refs, mismatched hash/identity/home/binding/descriptor, missing/copied certificate, compatibility-derived absence, pre-A1 artifacts, missing keys, expired intent, superseded claim, and conflicting replay fail closed. A1.3 must prove exclusive real public CLI/helper/REPL parked-turn and explicit-reattach adoption before the open, mandatory `RG-BASE-01` gate can close. The bounded A1.4 auto-attach producer uses the same contract. A1 closes only its scoped clauses of `RG-AUTH-01`/`RG-AUTH-02`; those ledger-wide gates remain unresolved for A2/A3. | A1.2, A1.3, A1.4 |
| **RG-ADMISSION-01** | Abandoned retained-spawn admission recovery | **Unresolved; B3.2a deliberately supplies exact retry and conservative head-of-line blocking only** | B3.2a retains every abandoned `SlotReserved` or `AuthorityRegistrationHead` record as live, permits no later-slot overtaking, and authorizes no resolution from PID, timeout, caller disappearance, helper state, socket/endpoint state, observer loss, or process liveness. It does not add cancellation, abandonment, expiry, or a user-facing recovery verb. | Exact request retry rejoins the same admission. Remaining B3.2 supplies an authorized durable control that resolves an abandoned admission only by exact issuer request/admission-record/session/authority/policy/participant/state identity; repeated resolution is idempotent; terminal durable resolution alone releases the live admission count and permits the next eligible queued request; crashes before/after resolution converge. PID, timeout, caller disappearance, helper state, socket state, endpoint state, observer loss, and process liveness cannot resolve it. Cancellation/abandonment remains distinct from R0 rejection and from stopping an already-created worker. Partial R0 registration and post-R0/pre-transport or transport-ambiguous claims are exactly reconciled rather than deleted or blindly rolled back. B4 exposes the exact user/tool-facing inspect/cancel operation and distinct outcomes without owning the underlying state transition. | Remaining B3.2, B4 |
| **RG-WORLD-ADOPT-01** | Exact HSA-bound generic world ownership adoption | **Resolved for bounded Linux B3.2a-WA; non-Linux remains unclaimed** | The first bounded Linux attempt correctly failed closed before member creation when legacy `AttachOrCreate` selected a different shared-owner world; its admission remains preserved `InterruptedNonterminal`. The review-clean implementation through `d0a70727c2bec2b2d6fe0754ea469c4682684dda` instead strict-validates `Some(exact proof)`, durably adopts the exact HSA-bound generic world before process creation, and preserves the Registered readiness contract. The final production-toolbox smoke joined HSA, receipt, and backend metadata on session `aos_27f14606e443f73927d2f892beabc704`, world `wld_019f6a96-bdf1-7db1-a5cc-58cbd0f370c9`, generation `0`, with one retained admission and no alternate world. | `ExactBoundWorldOwnershipAdoptionV1` resolves the exact HSA world by ID/generation and durably adopts only that generic world as the exact session's active shared owner. World ID/generation and all HSA/RetainedWorkerRuntime bytes remain unchanged; no alternate world is created. Exact retry joins without rewrite; conflicting owner/session/policy/project/spec/generation and missing/corrupt/ambiguous/partial metadata fail closed without mutation. Crash/reopen plus exact retry is proved before temp creation/persistence, after temp write, after temp-file fsync, immediately before atomic rename, after rename but before parent-directory fsync, and after parent-directory fsync but before response. Temp presence is never semantic ownership; incomplete recognized temps receive only operation-bound cleanup and complete exact temps are revalidated and re-fsynced before publication. After rename but before parent-directory fsync, reopen proof covers both allowed shapes: an exact original generic final restarts publication, while an exact adopted final is revalidated and re-fsynced before join. Conflicting, missing, or ambiguous temp/final evidence fails without mutation; no success precedes required file and parent-directory durability. Adoption proves no launch/Registered/routability/terminal state. Compatibility `None`, ordinary execution, world enforcement/capability behavior, and non-Linux posture remain unchanged. Prompt/request markers are absent from metadata, storage, logs, traces, errors, and journal. Linux world-service/backend suites, doctor, ordinary execution, and bounded managed Spawn smoke pass. | B3.2a-WA, B3.2a closeout |
| **RG-CLOSE-01** | Private stop unreachable versus durable closeout | **Resolved baseline for exact retained stop; generalize without regression** | The debug chain proved exact retained source stop can succeed by detached durable closeout when private owner transport is unreachable; A0 records that bounded recovery and separately records public host stop's active-session delivery gate, where missing/refused transport blocks closeout. Neither current behavior is treated as the target owner model. | Exact valid target + unavailable private stop produces durable closeout when rules allow; live transport remains a fast path; invalid/ambiguous/mismatched target still fails closed; repeated closeout is idempotent. | A0, A2, B4 |
| **RG-EVENT-01** | Runtime event/frame identity, ordering, and exact terminal event | **Resolved for B0 + B1/B2.1 accepted production paths; producer replay remains bounded transport state** | B0 landed the canonical V1 carrier in `common`/`transport-api-types` and producer assignment in ordinary `world-service` task streams plus retained launch/turn streams. Each accepted producer stream has one non-empty UUIDv7 stream ID; all frames share one positive gap-free frame counter; semantic Event/Exit items share one positive event counter and stable UUIDv7 IDs; Exit is the final semantic event and exactly matches its terminal identity. Canonical clone/re-emission preserves bytes. Concurrent stdout/stderr assignment is serialized through enqueue. Typed host decoding forwards identity unchanged, rejects missing/malformed runtime identity, and does not convert Error, EOF, stream exhaustion, or diagnostic non-live state into terminal truth. The joint production integration closeout proves the recovered B2.1 consumer on every named accepted production path while preserving byte-identical replay no-op, conflict/gap/reorder/post-terminal rejection, and restart-safe durable observation. Producer replay retention remains separately bounded process-memory world-service transport state and is not promoted into durable authority. | Preserve all B0 producer and negative-host gates. Later packets may consume the proven carrier/consumer truth, but they may not reinterpret EOF/error/process state as completion or widen producer replay into durable lifecycle truth. | B0, B2.1 |
| **RG-RECEIPT-01** | `run_world_task` blocks until terminal exit | **Resolved for current blocking compatibility; B2.2 early return remains deferred** | B1/B2.1-0 prepares RunWorldTask from exact HSA, immutable acceptance, and supervisor truth and excludes the legacy active-task registration path; the foreground call still blocks over supervisor truth. | The recovered B1/B2.1 cores, B1/B2.1-0, and the later joint closeout prove that every accepted production path enters the durable supervisor without a legacy writer while preserving blocking waiter behavior. Only B2.2 later returns `ActiveEphemeralTaskReceiptV1` before exit. | B1, B2.1, B2.2 |
| **RG-RECEIPT-02** | `continue_world_worker` blocks until terminal exit | **Unresolved; confirmed design deviation** | `execute_continue_world_worker_stream_for_turn_kind` consumes until `Exit`; `continue_world_worker` returns only afterward. Existing model-visible outcome has no active-run receipt or immutable policy commitment. | After B1 acceptance, B2.1 observation, and C1 event truth, B2.2 returns `ActiveRetainedTurnReceiptV1` before exit; B3.2 preserves clean-turn worker continuity and the host can continue or park. | B2.2, B3.2 |
| **RG-RECEIPT-03** | Active receipt inspection and caller-drop survival | **Resolved for the named accepted production paths; B2.2 early return remains deferred** | Recovered B1 acceptance and B2.1 claims survive real guard, waiter, and caller drop on the RunWorldTask production path. B1/B2.1-0 routes ephemeral Inspect/Cancel/Wait through exact receipt/supervisor truth and ordinary retained Continue through exact R0+B3.2a target/routability truth without legacy active-task synthesis; retained foreground-waiter drop is separately proved at the accepted-stream boundary. | B1-3a/B1-3b preallocate and durably create only the immutable acknowledged acceptance record. B2.1-1 claims that exact record for both work families before any later frame, B2.1-2 supplies inspect/wait compatibility over supervisor truth, and B2.1-3 preserves observation across restart. Caller/waiter/guard drop never deletes accepted or supervised truth. The joint closeout proved real guard/waiter destruction on the production-owned RunWorldTask and ephemeral accepted-task routes. Ordinary retained Continue proved the full dispatcher handoff into supervision; retained foreground-waiter drop remains proved compositionally at `execute_accepted_continue_world_worker_stream_for_turn_kind(...)`, and lower-level receipt or supervisor substitutions remain `RegressionMasked`. B2.2 later exposes early return. | B1, B2.1, B2.2 |
| **RG-RECEIPT-04** | Ephemeral `needs_retained_followup` remains a terminal non-retained result | **Partially implemented; target receipt semantics unproven** | The lifecycle design and current terminal enum preserve `NeedsRetainedFollowup`, but the durable active receipt contract did not state how it closes without becoming retained work. | B2.1 records `NeedsRetainedFollowup` only in immutable supervisor terminal closeout while leaving the B1 acceptance record unchanged; it promises no retained participant or continue route, creates no hidden retained worker/conversational obligation, and requires an explicit policy-checked spawn for ongoing work. | B2.1 |
| **RG-SUP-01** | Supervisor idempotent frame/event processing | **Resolved for B1/B2.1 accepted production ingress; C1 duplicate-obligation proof remains separate** | The recovered B2.1 core claims exact B1 acceptance, durably journals canonical B0 frames/events, makes byte-identical duplicates no-ops, and rejects gaps, reorder, conflicts, stale observers, and post-terminal frames. B1/B2.1-0 routes the named production actions to that owner while foreground callers remain blocking waiters. | B2.1-1 establishes the no-gap claim from exact B1 acceptance; B2.1-2 makes exact duplicate frames/events no-ops, rejects gaps/reorder/conflicts/post-terminal frames, and prevents stale observer writes. The joint closeout proved those invariants on the real accepted production paths. C1 later proves duplicate replay creates no duplicate obligation without reopening B2.1 ownership. | B0, B1, B2.1, C1 |
| **RG-SUP-02** | Supervisor restart and reconciliation | **Resolved for supported host/shell restart and unavailable-producer nonterminal boundaries** | The recovered supervisor durably owns nonterminal claims and exact cursors, and the bounded startup hook invokes canonical recovery. Surviving process-memory producer replay resumes exact frames; unavailable replay remains unresolved and nonterminal. | B2.1-3 alone discovers every nonterminal claim from durable supervisor state, resumes the exact B0 cursor through exact acceptance-record/stream/cursor producer replay when the process-memory world-service registry survives a host/shell restart, rejects stale observers, and never fabricates terminal success, cancellation, deletion, cursor advance, or a complete cut from EOF, timeout, PID/helper/socket/readiness/process/caller/endpoint state, local error, producer unavailability, or ambiguous/missing terminal truth. World-service restart or unavailable replay leaves the claim durably nonterminal and unresolved. The startup hook invokes only the canonical recovery entry point and distinguishes fatal store/claim corruption from a valid unresolved producer. The joint closeout proved these boundaries on the named accepted production paths. | B2.1 |
| **RG-CANCEL-01** | Same-turn continue → cancel | **Unresolved** | Current retained cancel resolves worker state, `authoritative_live`, owner PID, `latest_run_id`, and `Running`; the blocking continue returns after that active window is gone. | In one host turn, continue returns accepted active-run ID and immediate cancel targets it. Final receipt is cancelled or pending closeout, never `stale_linkage` merely because the worker/owner parked. | B3.2, B4, E2 |
| **RG-CANCEL-02** | Cancel outcome semantics | **Unresolved** | Current retained cancel terminal enum only expresses `Cancelled`; no-active work is reported through target-not-cancelable/stale-linkage-shaped errors. | Prove all categories from `04`: live cancel, pending closeout, already terminal, no active work, owner unreachable, invalid target, binding mismatch, ambiguity, and policy denial. | B4 |
| **RG-MSG-01** | Typed retained-worker messaging and causation identity | **Partially resolved; B3.1 producer and C1 consumer clauses complete, host-to-worker completion remains open** | B0 gives every retained runtime Event exact producer-assigned event/frame identity and ordering, B3.1 now normalizes supported retained provider wrapper events fail-closed before `AgentEvent` construction, and C1 consumes the validated typed envelope from the accepted durable supervisor journal without JSON-pointer repair. The remaining open scope is broader host-to-worker delivery semantics, not retained event typing or causation identity. | `world-service` maps every supported retained provider wrapper event into `NormalizedWorldWorkerEventFacetV1` with exact thread/class/attention/payload and B1 request/message causation before `AgentEvent` construction. The shell validates the typed top-level `AgentEvent.worker_event`, exact acceptance-record/worker-to-host target/active-run/thread/class/attention/causation join, and copied opaque B1 correlation before generic journaling and C1 materialization. Every post-ack retained `Event` frame in scope is typed or the stream and C1 cut fail closed. B3.2 alone still owns host-to-worker and broader immediate/durable delivery semantics. | B3.1, B3.2, C1 |
| **RG-OBL-01** | Obligation materialization before terminal exit | **Resolved for C1 on the accepted retained path** | The accepted retained path now routes each durably accepted typed B3.1 `Event` through C1 materialization immediately after the B2.1 generic journal commit, and the old terminal-coupled writer is removed from that accepted C1-owned path. The only surviving compatibility writer is the non-C1 `WorkerContinueForkCommand` branch, which does not compete with accepted retained-event truth. | A B3.1 attention event with exact B1 acceptance and B0 stream identity is durable in B2.1 and creates one C1 canonical obligation before the exact B0 terminal event; byte-identical replay creates no duplicate and terminal failure does not erase it. Before classification, C1 verifies that every post-ack retained B2.1 `Event` ref is typed and commits the complete canonical B3.1 event. C1 advances a monotonic ledger revision/materialized watermark, returns Pending before coverage, and Complete only through the exhaustive acceptance-record/stream/terminal-event cut; an untyped or omitted event prevents Complete. | B0, B1, B2.1, B3.1, C1 |
| **RG-OBL-02** | Obligation as canonical truth; inbox/attention as projections | **Partially resolved; C1 semantic cut complete, C2 projections remain open** | C1 now owns canonical obligation records, monotonic ledger revision/materialized-through watermark, and the closed `ObligationLedgerSnapshotReadV1` Pending/Complete semantic cut. Compatibility pending-count/session-posture and attach-state projections still exist, while durable inbox-item materialization remains legacy-only; none of those projections may create, classify, repair, or complete C1 truth, and none has yet been rebuilt as a pure C2 projection. | The closed C1 snapshot query binds exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream and terminal event/cut, and an exhaustive ordered join to every post-ack retained B2.1 `Event` ref plus canonical commitments to its B3.1 source/target/thread/class/attention/causation/payload envelope through the cut even when no attention obligation exists, along with unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run, authority/ledger revisions, disposition, and sorted canonical-record commitments. Any untyped/omitted/substituted event or stale/mismatched scope, correlation, cut, disposition, or record fails closed, and both empty/no-event-or-no-attention and non-empty/attention dispositions are proven. HostSessionAuthority consumes that result unchanged and cannot invent or reinterpret event or transition semantics. C2 later rebuilds inbox/auto-attach projections from obligations; host `AwaitingAttention` exactly follows unresolved attention-driving obligations; worker `AttentionPending` remains separate. | C1, C2 |
| **RG-ATTACH-01** | Auto-attach claim and router trigger | **Partially implemented, unproven** | Eligibility, claim, settle, wrong-host checks, and helper launch exist; trigger is spawned after terminal-coupled obligation creation and has not been proven in the target event flow. | Multiple eligible obligations coalesce to one session claim; wrong-host/policy denial fail closed; restart/retry is idempotent; manual reattach and auto-attach do not create duplicate owners. | C2, C3 |
| **RG-ATTACH-02** | Router must restore host ownership only | **Unresolved proof gate** | Current code launches a hidden owner helper, but no end-to-end proof establishes the permanent non-responsibility boundary. | Instrument the router entrypoint and assert it cannot submit prompts, approve, answer, fork, continue, cancel, or stop worker work; it records attach outcome and stops. | C3 |
| **RG-UAA-01** | Codex/UAA guest runtime realizability | **Partially resolved; keep as baseline** | World Codex config and validator require `/var/lib/substrate/world-deps/bin/codex`; the old host-NVM exit-127 diagnosis is historical for current placement-aware config. | World Codex rejects host-local binary paths and succeeds only with guest-visible runtime deps; host and world envelope kinds remain distinct. | D1 |
| **RG-CONFIG-01** | Per-worker projection identity and no sibling aliasing | **Unresolved; defensive scaffolding only** | Isolated Codex home/config exists, but projection authority and per-retained-worker ownership are not proven on the real runtime path. | Sibling retained workers cannot share or alias projected runtime config/home state unless an explicit policy and projection identity permit it. | D1, E3 |
| **RG-CONFIG-02** | Runtime-native files are projection, not authority | **Unresolved; defensive scaffolding only** | Runtime homes and workspace config files can exist independently of canonical projection truth; current bounded Codex auth/config seeding is a compatibility bridge. | `.codex`, `CODEX_HOME`, `config.toml`, `.mcp.json`, and equivalent native files can be rebuilt as non-secret projections from Substrate truth. Credentials enter only through launch-time gateway handoff; copied auth/config is named compatibility only; narrowed hints never replace broker/world-service enforcement. | E3 |
| **RG-CONFIG-03** | Managed in-world gateway secure-FD credential carrier | **Resolved implementation baseline; focused launcher/consumer integration proof exists** | `GatewayAuthBundleV1` is validated in `crates/common/src/gateway_auth_bundle.rs`; `crates/world-service/src/gateway_runtime.rs` sends it through an inherited pipe, exposes only `SUBSTRATE_LLM_AUTH_BUNDLE_FD`, and scrubs known raw-secret env vars; `crates/gateway/src/server/mod.rs` consumes and validates it once. Gateway consumer and world-service parity tests cover one-time read and fresh restart delivery. | Preserve bundle validation, raw-secret env scrubbing, one-time gateway consumption/FD close, no raw-secret fallback in the child, and fresh handoff on restart. Targeted launcher/consumer tests remain green; any live gateway smoke must agree with those invariants. | D1, E3, D3 |
| **RG-CONFIG-04** | Direct world Codex gateway adoption and Substrate-owned config projection | **Unresolved; current direct member path is a compatibility bridge** | `crates/world-service/src/member_runtime.rs::prepare_codex_runtime_env` still seeds isolated `CODEX_HOME` auth and a bounded host-derived `config.toml`; it does not prove that direct world Codex receives the managed gateway endpoint/session contract or a projection derived from accepted Substrate policy/config. | Credential-requiring world Codex uses the exact managed gateway covered by `RG-CONFIG-03`; per-worker `CODEX_HOME`, `config.toml`, provider endpoint, and related native files are rebuildable non-secret projections from Substrate truth; no host auth file is copied in contract-correct mode; the envelope joins projection, policy snapshot, gateway receiver, and non-secret handoff evidence. | D1, E3, D3 |
| **RG-UAA-02** | Codex/UAA world command caging and side-effect mediation | **Unresolved** | Initial runtime is launched in world context and existing world-service commands have cage/fs/network enforcement, but Codex/UAA shell/edit/write/tool side effects are not forced through a per-operation broker. The managed gateway carrier exists, while direct world Codex credential/provider wiring still uses copied runtime-home material instead of adopting it. | For shell, patch/edit, direct write, write-capable MCP/tool, process spawn, and network: broker under receipt snapshot or fail closed. Credential-dependent provider access uses the existing in-world gateway session, never copied secrets. `cwd`, env, initial placement, or isolated home alone cannot pass. | D1, D2, E1, E3 |
| **RG-UAA-03** | `external_sandbox` means Substrate-owned mediation | **Unresolved; current behavior is unsafe as architectural proof** | `gateway/adapter_runtime.rs` defaults Codex to `agent_api.exec.external_sandbox.v1=true` and allows external sandbox execution, but no `WorldCommandExecutionBroker` proves Substrate owns every side effect. The secure-FD carrier exists for the managed gateway, but the direct world-UAA path is not yet joined to it through the target envelope/projection path. | With external sandbox enabled, removing/interrupting broker capability or required gateway adoption makes world-UAA startup/side effects fail closed. With both active, every side effect joins to the accepted policy hash and every credentialed session joins to the existing carrier's non-secret handoff evidence. | D2, D3, E3 |
| **RG-POLICY-01** | Dispatch-scoped `world_fs` narrowing carrier | **Unresolved** | Policy has `allow_capability_narrowing`; inventory overlays and boolean capability overrides exist; no request-scoped restricted `PolicyPatch.world_fs` reaches task/turn receipts. | Gate=false rejects patch; gate=true accepts only restricted world_fs narrowing; resulting hash/ref/reason is persisted on receipt/manifest. | E1, E2 |
| **RG-POLICY-02** | Narrowed policy enforcement and path containment | **Unresolved** | Existing inventory overlay subset check is equality-based; existing `PolicySnapshotV3` and world-service enforcement are useful primitives but not wired to dispatch-scoped narrowing. | Base `.` or `src` may narrow to `src/parser.rs`; file-to-directory/root, absolute paths, `..`, symlink escape, denial removal, disabled-write enablement, and weakened cage/fail-closed/enforcement are rejected. Allowed file succeeds and sibling file fails at runtime. | D2, E1, E4 |
| **RG-POLICY-03** | Immutable active snapshots and retained worker caps | **Unresolved** | Current task/continue outcomes do not commit immutable dispatch policy snapshots; retained manifests do not hold the target worker-cap contract. | Active run retains accepted hash across parent changes; future turn recomputes parent ∧ cap ∧ turn patch; parent broadening never broadens worker; parent narrowing narrows or invalidates next turn; fork inherits cap. | E2 |
| **RG-SYNC-01** | Host-visible write sync semantics | **Unresolved/open debug bucket** | Debug evidence warns that task completion does not prove a host-visible file; current docs/runtime distinguish host-visible overlay behavior from full isolation/reconciliation. | Matrix proves: host-visible allowed write visibility, host-visible denied write, full-isolation non-visibility before reconciliation, explicit reconciliation result, retained-turn behavior, and narrowed allowlist behavior. | E4 |
| **RG-WORKER-EXIT-01** | Non-zero Codex/UAA worker-turn exit semantics | **Unresolved/open debug bucket** | `codex exited non-zero` remains separate from repaired routing/stop seams; current blocking outcome can conflate runtime failure with authority/liveness loss. | Non-zero turn closes active receipt as durable `Failed` with exit diagnostics and policy/session/world joins; retained worker/session authority is preserved or explicitly invalidated for a stated lifecycle reason; no false success/attention loss. | B3.2, D3 |
| **RG-OBS-01** | First-class world-dispatch observability | **Unresolved overall; B0/B1/B2.1/R0/B3.2a/B1/B2.1-0 joint-closeout, B3.1, and C1 clauses satisfied** | B0 exposes exact producer-originated stream/frame/event/terminal identity. The recovered B1 receipt and B2.1 supervisor cores supply immutable acceptance plus durable claim/journal/cursor/terminal truth. R0+B3.2a supply exact retained target/routability, B1/B2.1-0 joins those sources for its named full-dispatch and tool routes without legacy active-task writes, B3.1 supplies the complete typed retained-event envelope before generic journaling, and C1 now materializes canonical obligations plus the closed Pending/Complete semantic cut from that exact durable event corridor. | Remaining clause owners are explicit: B4 cancel, D2 broker operations, E2 policy commitments, and E3 non-secret credential-handoff state. World-service replay transports only exact producer facts, and `run_async_repl` only activates canonical supervisor recovery; neither is observation or lifecycle authority. Full-dispatcher, real guard/waiter-drop, and real tool-to-dispatch assertions remain mandatory, and direct resolver, transport, receipt, or supervisor substitutions remain `RegressionMasked`. D3 alone owns final end-to-end integration closure, proving those facts join by request/active-run/session/world IDs without secret payloads or secret-derived fingerprints. | B0, B1, B2.1, B3.1, C1, B4, D2, E2, E3, D3 |

`RG-AUTH-03` A1.2 proof additionally requires Start/Attach transport to remain retained until
closed startup evidence binds exact store/session/intent/claim/claimant-attempt/run/application/authority/
participant identity, the exact target-participant or launch-claimant protocol actor, and durably
records the matching acceptance or definitive terminal reason. Timeout,
EOF, helper/PID/socket/handle/readiness loss, and ambiguity remain Pending. The later
payload-deletion and root-advance proof from `ReleaseEligible` to `Released` remains part of the
overall gate, but the current A1.2b packet closes only `ReleaseEligible` plus exact join tolerance
if a separately validated `Released` state is reopened. Every Resume terminal outcome references one immutable, exact-scope
post-turn protocol event with the matching actor, event ID/sequence, outcome/reason, input state,
completion, and application result; transport/process/readiness/timeout/EOF/local-error inference
cannot construct it. Resume terminal completion may close without an obligation snapshot, but
resumable completion persists `AwaitingObligationCut` and retains transport until the
`ObligationLedger` supplies a Complete per-session revision/event-materialization cut scoped to
the exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream
and terminal event, authenticated intent/revision/payload commitment and distinct transition run,
and authority revision. A1.2 consumes that closed disposition unchanged and cannot infer or repair
the correlation or scan, classify, create, resolve, reinterpret, or overwrite obligations.
Inbox/count/worker/compatibility truth is forbidden. C1 owns the canonical records and complete
cut. The corrected prerequisite order is B0 runtime identity/order, B1 accepted task/active-run
identity, B2.1 durable observation/reconciliation, B3.1 exact retained-event semantics, and C1
materialization/completeness before A1.2b resumes. This moves no A2/A3 ownership, permits the
foreground to remain blocking until B2.2, and closes none of `RG-EVENT-01`, `RG-RECEIPT-03`,
`RG-SUP-01`, `RG-SUP-02`, `RG-MSG-01`, `RG-OBL-01`, or `RG-OBL-02` by documentation alone. The
crash matrix includes startup evidence,
snapshot publication/orphan staleness, terminal exact retry, and payload deletion/object-directory
`fsync` windows.

### A1.2a-WB gate assignment

A1.2a-WB is a bounded correction under existing gates, not a new gate. `RG-AUTH-01` and
`RG-AUTH-03` own exact session-binding preservation, Start request equality, exact retry, and
zero-mutation rejection through Start issuance, application/persistence, and exact
current-authority resolution. `RG-BASE-02` owns preservation and exact readback of world binding
without generic synthesis or participant-placement leakage. `RG-DIFF-01` owns the broad
before/after differential. Their combined matrix requires `Host + None`, `Host + Some(exact)`, and
`World + Some(exact)` to apply, exact-join, and resolve; `World + None`, descriptor/launch mismatch,
malformed binding, changed world ID or generation, and corrupt/substituted persisted combinations
fail closed without mutation. Applied and resolved `Host + Some` keeps the descriptor and launch
scope `Host`, returns the exact binding only from durable session authority, and leaves the host
participant manifest without world placement fields. No schema/version/migration/golden-vector gate
is implicated because no shape or canonical byte changes.

## A0 closeout evidence

A0 changed documentation only. It moved no authority decision, changed no production call path or enforcement point, added no runtime primitive, and preserved all compatibility behavior. The inventory in `02-seam-crosswalk.md` is therefore evidence of current ownership and coverage, not evidence that HostSessionAuthority or HostExecutionEpisode has landed.

The following 49 focused test commands passed on Linux on 2026-07-10; every command reported `1 passed; 0 failed`:

```text
cargo test -p shell public_start_persists_detached_session_when_hidden_owner_helper_exits -- --nocapture
cargo test -p shell --lib list_live_participants_filters_dead_owner_pid_rows -- --nocapture
cargo test -p shell --lib resolve_internal_continue_world_dispatch_target_accepts_retained_worker_after_owner_pid_exit -- --nocapture
cargo test -p shell --lib resolve_internal_stop_world_dispatch_target_accepts_non_authoritative_live_worker -- --nocapture
cargo test -p shell --lib dispatch_contract_stop_world_worker_spec64_recovery_harness_drives_the_real_refreshed_transport_failure_branch -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_internal_toolbox_stop_world_worker_treats_disappearing_private_stop_delivery_as_fail_closed -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_internal_toolbox_stop_world_worker_keeps_refused_transport_text_distinct_from_missing_transport -- --nocapture
cargo test -p shell --test repl_world_first_routing_v1 c3_startup_fail_closed_when_persistent_session_cannot_reach_ready -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_cleanly_closes_same_durable_session_after_reattach -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_fails_closed_without_same_episode_terminal_proof_even_if_later_state_reads_stopped -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_refused_transport_stays_on_existing_connect_failure_surface -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_stop_timeout_wording_stays_distinct_from_missing_transport_and_stale_authority -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_reattach_and_fork_preserve_exact_session_and_lineage_contracts -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_turn_uses_persisted_attach_continuity_selector_when_recovering_detached_host_turns -- --nocapture
cargo test -p shell --lib helper_readiness_accepts_resume_one_turn_after_fast_detach_once_prompt_is_terminal -- --nocapture
cargo test -p shell --lib persist_orchestration_session_rejects_stale_active_snapshot_after_terminal_snapshot -- --nocapture
cargo test -p shell --lib dispatch_contract_fork_world_worker_times_out_when_stop_transport_publication_never_completes -- --nocapture
cargo test -p shell --lib wait_for_fork_child_durable_publication_allows_late_child_visibility_without_extending_stop_transport_budget -- --nocapture
cargo test -p shell --lib manual_reattach_verification_failure_releases_claimed_auto_attach_obligation -- --nocapture
cargo test -p shell --lib finalize_session_auto_attach_after_launch_returns_failed_closed_when_restore_check_fails -- --nocapture
cargo test -p shell --lib synchronize_repl_authoritative_world_binding_repairs_from_persisted_session_truth -- --nocapture
cargo test -p shell --lib synchronize_repl_authoritative_world_binding_prefers_shared_world_metadata_over_stale_session_truth -- --nocapture
cargo test -p shell --lib synchronize_repl_authoritative_world_binding_fails_closed_when_metadata_is_unreadable_even_if_live_member_truth_exists -- --nocapture
cargo test -p shell --lib start_member_runtime_reuses_parent_session_and_persists_world_binding -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 public_root_start_world_scope_starts_attached_host_session_with_world_binding_truth -- --nocapture
cargo test -p world-service bootstrap_completion_without_session_handle_emits_only_exit -- --nocapture
cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture
cargo test -p world-service finish_bootstrap_preserves_retained_slot_when_session_handle_exists -- --nocapture
cargo test -p world-service register_member_replaces_stale_fork_child_in_same_slot -- --nocapture
cargo test -p shell --lib ensure_router_auto_attach_targets_exact_local_host_allows_untargeted_obligations -- --nocapture
cargo test -p shell --lib ensure_router_auto_attach_targets_exact_local_host_allows_same_host_targets -- --nocapture
cargo test -p shell --lib ensure_router_auto_attach_targets_exact_local_host_rejects_foreign_targets -- --nocapture
cargo test -p shell --lib dispatch_contract_persist_continue_world_worker_obligation_projects_supported_events_into_canonical_state -- --nocapture
cargo test -p shell --lib dispatch_contract_steering_policy_rejects_ephemeral_concurrency_cap_exceeded -- --nocapture
cargo test -p shell --lib active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start -- --nocapture
cargo test -p shell --lib active_ephemeral_world_task_registry_round_trips_live_task_identity -- --nocapture
cargo test -p shell --lib prepare_member_replacement_runtime_preserves_resumed_from_lineage -- --nocapture
cargo test -p shell --lib terminalize_startup_prompt_failure_marks_accepted_prompt_failed -- --nocapture
cargo test -p shell --lib private_prompt_bridge_emits_terminal_failed_after_accepted_owner_drop -- --nocapture
cargo test -p world-service --test member_runtime_retained_lifecycle_v1 member_runtime_non_zero_submitted_turn_exit_cleans_active_turn_slot_without_deleting_retained_worker -- --nocapture
cargo test -p shell --lib inflight_attach_join_returns_authoritative_receipt_when_ready_state_already_exists -- --nocapture
cargo test -p shell --lib inflight_attach_join_does_not_accept_detached_live_owner_as_attached_success -- --nocapture
cargo test -p shell --lib classify_prompt_worker_error_treats_common_terminal_loss_errors_as_abnormal -- --nocapture
cargo test -p shell --lib shutdown_disposition_tracks_termination_cause -- --nocapture
cargo test -p shell --lib hidden_owner_private_stop_fails_closed_when_completion_never_resolves -- --nocapture
cargo test -p shell --lib shutdown_host_orchestrator_runtime_parks_resumable_host_session_on_detach -- --nocapture
cargo test -p shell --lib can_park_host_runtime_after_detach_accepts_completed_one_turn_when_store_lags -- --nocapture
cargo test -p shell --lib host_inbox_state_store_materialization_is_serialized_for_concurrent_repeat_runs -- --nocapture
cargo test -p substrate-common test_substrate_home -- --nocapture
```

These exercise selected current behaviors relevant to `RG-AUTH-01`, expose selected still-current `RG-AUTH-02` dead-PID, PID-zero sentinel, and socket-gated startup/fork/public-stop behavior without normalizing it, and cover selected positive and negative `RG-CLOSE-01` baseline cases. They do not satisfy any complete regression gate or promote a seam. The session-snapshot test proves the narrow terminal-state guard only; no test submits conflicting `Active` snapshots and then a later heartbeat from the stale episode, so same-state revision safety remains an explicit A1 proof gap.

No production-path test proves revision-safe, idempotent binding establishment and failure clearing at startup, covers every effective-UID/XDG/`/run/user`/`/tmp` metadata namespace, or crashes/fails between world replacement, binding persistence, and older-generation worker invalidation, so binding reconciliation remains an explicit A1 proof gap. No test fails after initial `ActiveAttached` session persistence but before hidden-helper gateway/ownership establishment and then restarts to prove reconciliation without phantom attachment or stale rewrite. Arbitrary-path, stale/replayed helper plan, mismatched identity/binding, restart, and stale-revision rejection are also unproven. No focused test covers startup-prompt connect/EOF/duplicate/stale/restart phases, process-local startup-signal loss/timeout/duplicate/stale delivery, attach leader process loss/duplicate launch, or initial prompt/stop/cancel/toolbox registration failure while proving availability stays an episode observation; terminal-loss classification tests also do not prove revision-safe park/stop across restart, so those remain A1/A2 proof gaps. The selected private-stop tests cover individual helper failure, ordinary detach, completed one-turn, and public missing/refused/timeout branches, but no Start/Attach/ResumeOneTurn by host/member-role matrix combines stale `Running`, event/completion ordering, signal/helper/transport loss, stop, restart, stale revision, and repeated closeout while proving revision-safe outcomes. Lost/disconnected/duplicate/delayed auto-park delivery and completion ordering are likewise unproven, so helper intent/plan, parent-role gating, auto-park, and private-stop ownership/delivery remain explicit A1/A2/B3.2 proof gaps.

Remote member client-build/stream-open and post-accept ownership-persistence failure are not proven to retry/restart without a stranded local row, daemon slot, or duplicate; process-local dispatch caps, ephemeral identity/terminal wait, submitted-turn maps, and retained member admission/routing likewise lack multi-process/daemon-restart survival, so receipt/supervisor/runtime ownership remains a B0/B1/B2.1/B2.2/B3.1/B3.2/B4 gap. Existing member-runtime tests cover selected clean-exit/session-handle and stale-child cases; retained/ephemeral action-versus-`ash_` prefix mismatch, malformed identity, daemon restart followed by exact continue, duplicate participant/retained-key retry, the full clean/nonzero/cancel/missing-identity/stale-observer matrix, and restart between durable invalidation and replacement remain B3.2 proof gaps. No exact runtime-toolbox cancel test proves owner-PID loss reaches receipt-targeted cancel with distinct no-active, unavailable-owner, and terminal outcomes, so the pre-resolver PID gate remains a B4 gap. The fork tests prove current missing/late stop-socket handling, not that a durable child's lineage is independent from transport publication. Lossy fragments and the 100-byte shared-`/tmp` fallback lack colliding-ID, two-long-home, cross-UID/store, stale-path/symlink, permission, and cross-route negative proof; the session toolbox also lacks competing/stale-episode rebind proof, so exact plan/attach/startup/prompt/stop/cancel/toolbox addressing remains an A2/B3.2/B4 gap.

Existing host-target tests exercise the current untargeted/same-host/wrong-host behavior, but hostname absence/change/collision across restart is unproven and remains a C2/C3 gap. The common-path unit proves only the ordinary absolute `SUBSTRATE_HOME` result; no test creates conflicting state/config/policy/agent inventory under two homes or global/workspace precedence and crosses create/control, supplies a relative home across CWD changes, exercises empty/unset with missing `dirs::home_dir()`, forces CWD lookup failure/`.` fallback, or changes CWD between contract resolution, workspace persistence, and existing-session launch. Normalized store/workspace/config/policy/inventory identity and cross-home/cross-CWD fail-closed behavior therefore remain explicit A1/A3/E2/E3 proof gaps. The `SUBSTRATE_OVERRIDE_*` effective-config family also lacks changed/invalid/empty/removed create-control-restart proof. Anchor projection lacks invalid/empty mode/path, Project/FollowCwd/Custom, explicit project override, fallback, and CWD-change coverage. The concurrent inbox materialization test exercises threads within one process only; no proof runs two independent StateStore writers against one snapshot and demonstrates that conflicting lifecycle, binding, claim, inbox, or obligation updates cannot overwrite each other, so cross-process atomicity and revision safety remain explicit A1/A3 gaps. `SUBSTRATE_SOCKET_ACTIVATION_OVERRIDE` and `SUBSTRATE_WORLD_EXEC_FORCE_DIRECT` have no production-ingress negative matrix or live transport/isolation smoke here; A0 records their actual roles without treating that missing B/D/E proof as success. `cargo fmt --all -- --check` and `git diff --check` passed. No external live-world doctor/smoke/e2e, full workspace test, or clippy was run because A0 changes no runtime behavior; those gates remain unavailable, and no focused unit/integration result is promoted to whole-seam proof. Remaining gaps are the unchanged A1/A2/A3 ownership moves and later retained-runtime/config/receipt work named by the inventory.

## A1.1d mandatory-review state

A1.1d-1 through A1.1d-4 are Linux implementation/review clean and preserved through
`ca9429e5`; the corrected A1.1d-5 private-home implementation is focused-test and implementation-
review clean through `faabed16`. A1.1d remains incomplete because the positive `RG-BASE-01` product
wall exposes a preexisting owner-transition handoff defect. A1.1d integrated Linux closeout remains
open and cross-platform closeout remains pending. The exact cross-packet hold makes A1.1e
implementation-ready without declaring A1.1d complete; A1.1e is now focused-proof and review clean
through `cd676614`. This does not close A1.1d, its integrated/cross-platform proof, or any
cross-document seam.

The later A1.1d-5R1 Case A contract is recorded by
`17ea3a839345cd47a5b2409cde0d4facdde09446` and its bounded Linux runtime is review-clean through
`4d0acff68e20d86b97fe5367b8a4617554f33ef4`. This corrects only effective-authority semantics and
diagnostics; it does not supersede the historical checkpoints above, close A1.1d/A1, or start R2/R3.

The rejected A1.1d-6 heartbeat hypothesis is recorded as `PreexistingExposedByA1d`. With the same
hermetic lifecycle fixture, pre-A1.1d `c800436d` and pre-rejection `f73e8a81` pass three of three
runs, while stale-rejection commit `bdb1796d` and later `ca9429e5`, `c5117b51`, and `faabed16` fail
three of three. A focused observation-only heartbeat operation passed five StateStore tests, but the
public lifecycle still failed unchanged, proving heartbeat persistence is not the blocker. The
heartbeat remediation was withdrawn; no runtime heartbeat change remains and no A1.1d-6 is implied.

Bounded successor diagnostics show exact incompatible authority records. Current durable truth is
`Active`/`ParkedResumable`, owner PID zero, the prior authoritative participant, and a completed
startup prompt. The successor reconstructs and supplies `Allocating`/`ActiveAttached`, a new owner
PID and participant, no active handle, and a pending prompt. `persist_orchestration_session`
correctly refuses that lifecycle regression; `current_world_binding_session` then correctly rejects
the supplied whole-session record as `stale_world_binding_session_snapshot`. Pre-A1.1d `c800436d`
and pre-rejection `f73e8a81` passed only because stale overwrite remained possible. From
`bdb1796d` onward, stale-rejection and later revisions fail closed; that does not make the old
passing behavior correct.

Correct closure requires a revision-bound HostSessionAuthority transition that adopts exact current
parked truth into the successor episode or rejects/reconciles it idempotently. It cannot be achieved
by heartbeat-only writes, timeout inflation, retrying the stale snapshot, last-writer-wins, or
weakening stale checks. That is a broader owner seam than the proposed A1.1d-6 allowed scope, so the
current stop classification is `ArchitecturalOwnerChangeRequired`. A1.2 owns the durable
transition-protocol closure, including the rule that every applied Resume terminal outcome carries
an exact completion/post-turn-application pair. A1.3 owns its real CLI/helper/REPL adoption, the
bounded transport of exact startup ownership acknowledgement or typed pre-ownership
rejection/failure (never readiness, PID/helper/socket posture, timeout, EOF, or local-error
inference), and `RG-BASE-01` closure.

Unsafe ACL rejection is a negative security success, not positive product smoke. The positive
`RG-BASE-01` smoke must run separately against a valid owner-only private bootstrap home with mode
`0700`, no observable POSIX access/default ACL, and qualified mode-authority observations. A
Linux `ENODATA` result in that proof means only that the kernel returned no ACL data, never that an
ACL xattr is physically absent. A1.1d-5I captured the blocked real default path and isolated
effective/default ACL matrix: the supported host ancestor carried only a masked non-writing access
entry, while the final root was absent; a separate isolated default-ACL case proved inheritance and
candidate residue. This is now a product compatibility decision, not fixture-only or environmental
contamination. R1 Case A approves only strictly parsed masked non-writing Linux POSIX ancestor
access ACLs; every default ACL, effective write, unsupported model, and uncertain ACL state remains
rejected, and no enforcement is weakened by the audit.

The corrected A1 V1 identity boundary starts at the first successful no-follow child open beneath
the retained, validated parent. Candidate creation or `AlreadyExists` convergence precedes that
boundary; all later operations remain descriptor-relative and later replacement fails closed.
Neither malicious root nor malicious same-UID substitution before the first open is in scope, and
no portable atomic create-and-bind or privileged-broker claim is part of `RG-HOME-01`.

## A1.1d-5I installer/bootstrap audit record

**Packet/status:** A1.1d-5I is a completed investigation beneath A1.1d-5 at baseline
`b29897e00194e7d3c27a43dc749fe29731d96f78`. Capital `I` means investigation. It is not
A1.1d-6, authorizes no implementation, closes no gate, promotes no seam, and changes no broader
sequence. The source branch, upstream, and remote all matched the baseline with a clean index and
worktree before evidence collection. The host was Manjaro Linux, kernel
`6.16.8-1-MANJARO`, effective UID `1000` (`spenser`), no sudo intended-user variables, umask
`0077`, and `getfacl`/`setfacl` `2.3.2`. Passwordless sudo was unavailable, so full provisioning
and Codex runtime installation were deliberately not run; the absent/inactive world service,
socket units, `/run/substrate.sock`, and `/run/substrate` were snapshotted and left unchanged.

**Focused context capsule:** `PrivateSubstrateHomeV1` in HostSessionAuthority owns final-root and
ancestor acceptance. The selected home then supplies StateStore, config, policy, and inventory;
those consumers cannot select a fallback. Dev/release installers bootstrap the root, write
installation projections, deploy shims, optionally provision platform services/world dependencies,
and optionally provision the Codex runtime. In host V1 the declared prefix, `SUBSTRATE_HOME`, and
`SUBSTRATE_ROOT` identify the same host root, and privileged paths must also carry the intended
platform principal. A Lima/WSL/native realization may use a distinct platform path/principal only
through an explicit mapping bound to that host-context commitment and platform instance.
The final root remains exact owner, directory, exact `0700`, rejects every observable access/default
ACL, and retains no-follow/stable physical identity; qualified `NoData` is accepted only under that
descriptor and mode authority and is not physical-absence proof. Ancestors must exclude
other-principal replacement
authority and uncertainty. Malicious root and malicious same-UID substitution before the first
accepted descriptor, shared multi-principal homes, separate install-root design, world/policy
changes, native platform proof, and remediation implementation are non-goals. `RG-HOME-01`,
`RG-INSTALL-01`, integrated Linux product proof, and native macOS proof remain open.

### Static stage and authority inventory

| Stage | Product path and boundary | Selected-context propagation and partial-failure truth | Coverage/evidence |
|---|---|---|---|
| Private-home bootstrap | Dev and release Unix installers invoke the built/extracted `substrate --version`; Linux provisioning repeats it for the invoking user. HostSessionAuthority is the acceptance owner. | Dev and release bootstrap pass `SUBSTRATE_HOME=<prefix>`; release/Linux also pass the resolved intended user where privileged. An invalid existing root is not repaired. A default-ACL-created candidate can nevertheless remain after post-create rejection. | Focused private-home tests cover wrong owner, mode, symlink, replacement, malformed ACL, concurrency, and effective-zero named ACL, but not non-writing effective access ACLs or inherited default ACL cleanup. |
| Shim deploy/remove | Dev install/uninstall, release install, and Windows install/uninstall paths launch or remove shim state. This child boundary is not home authority. | Unix dev deploy/remove and Windows dev deploy/remove set root but omit home; Unix release deploy passes neither explicitly. Windows release normally relies on profile sourcing, but `-NoAutoSource` still deploys without explicit context. Unix/Windows release uninstall has no selected-context shim-remove boundary before deletion. | `installer_env_wcu4` checks selected post-install doctor/sync strings and generated Windows profile content, not these child boundaries or lifecycle symmetry. |
| World dependency add/remove/current sync/rollback | Dev/release installer invokes the world-deps CLI; the runtime-family adapter consumes the selection. | Unix dev and release paths pass both root and home. Dev sync failure removes a global enable only when the installer added it; guest-side changes are explicitly not rolled back. | Static source confirmation only in 5I; no world mutation was authorized on this host. |
| World provisioning/enable/disable | Dev Linux calls `world-provision.sh`; release installs units; macOS maps the host selection to the Lima guest user's private home; Windows delegates to WSL. | Linux dev passes home, resolves intended user, validates the root, and embeds it in the unit. Unix release embeds the selected home but derives `ReadWritePaths` from ambient `$HOME`, so a custom prefix outside that allowlist can be denied by `ProtectSystem=strict`. Platform mappings cross distinct path/principal domains. | Full live provisioning requires a dedicated sudo-capable host and post-state restoration proof; static macOS/Windows comparison is not native proof. |
| Service install/restart | Linux provisioner installs world-service/gateway/ACL helper, units, socket drop-in, group/ACL bridge, and runtime/state directories, then reloads/restarts. | Dev uninstall can remove world-service/unit basics only when opted in, but omits installed gateway, ACL helper, and socket drop-in; release uninstall also omits the installed gateway. Failure/uninstall can therefore leave exact system-managed artifacts. | Static source and unchanged pre/post service snapshot only; R3 requires exact manifest and product parity. |
| Intended principal, group, and linger state | Linux provisioning resolves the invoking account across sudo, creates/uses the `substrate` group, may add membership/ACL bridge state, and reports linger requirements; release install-state can record selected host changes. | Dev uninstall gives manual group guidance; release automatic host-state cleanup is opt-in and acts from recorded metadata. Neither path may infer a principal from an unrelated ambient home or remove pre-existing membership/linger state. | Static inventory only; dedicated-host R2/R3 smoke must compare recorded intended principal and exact pre/post group, ACL-bridge, and linger state. |
| Codex runtime provisioning | Dev/release paths enable `codex-runtime` and run current sync only with world enabled. | Both root and home are passed. If a newly added global enable cannot sync, removal is attempted; any guest-side effects may remain. | Static source only; not native product proof and not evidence of runtime capability loss. |
| Generated projections/install state | Dev/release writers produce `env.sh`, manager environment, configuration, dev shim helper, versions, dependency scaffold, and install-state metadata. | `env.sh` and the dev shim helper encode selected values; manager scripts can self-derive install location; other artifacts may only be located beneath the prefix. Both manager variants prefer conflicting ambient `SUBSTRATE_HOME` over self-location. Shim doctor also projects the trace log from default home. | File-content/location/mode inspection plus world/shim/health doctor; R2 needs custom A versus ambient B consumer proof. |
| Uninstall/rerun/reinstall | Dev uninstaller removes selected artifacts and optionally services; Unix release wrapper locates by root but its child deletes from ambient home; Windows release accepts `-Prefix`. | Dev shim removal loses home without the override and leaves two gateway symlinks. Unix release can miss the selected tree while targeting default-home state. Windows release uses a prefix-independent forwarder PID path, recursively deletes every `$USERPROFILE/.substrate*` match, and has no selected shim-remove boundary. macOS dev host-socket cleanup uses `${HOME}/.substrate`, not the selected prefix. | Live dev custom-prefix matrix; Unix/macOS/Windows destructive paths are static evidence only and were not executed against real/native homes. |

### Isolated ACL matrix

All mutable cases used the recorded audit-owned root
`/run/user/1000/substrate-a1d5i-audit-019f6d60`; `/`, `/home`, and `/home/spenser` were read
only. The real home was owner `1000`, mode `0710`, with a named `libvirt-qemu:--x` access entry and
mask `--x`; its default-ACL read returned `NoData`, which is not proof of physical xattr absence.
The real `/home/spenser/.substrate` did not exist and was never
created, repaired, moved, or removed. Linux ACL semantics were checked against
[`acl(5)`](https://man7.org/linux/man-pages/man5/acl.5.html),
[`path_resolution(7)`](https://man7.org/linux/man-pages/man7/path_resolution.7.html),
[`mkdir(2)`](https://man7.org/linux/man-pages/man2/mkdir.2.html),
[`unlink(2)`](https://man7.org/linux/man-pages/man2/unlink.2.html), and
[`rename(2)`](https://man7.org/linux/man-pages/man2/rename.2.html): the ACL mask limits named
entries, search permits traversal, directory write plus search authorizes creation/removal/rename,
and a default ACL is inherited separately from the access ACL.

| Case | Expected security semantics | Actual at `b29897e0` |
|---|---|---|
| Ancestor named access ACL, effective `--x` | Traversal only; no create/delete/rename/replacement authority. Supportable only after explicit R1 contract/proof. | Rejected exit `5` as `foreign-acl`; final child not created. |
| Ancestor named access ACL, effective `r-x` | Read/search but no write/replacement authority; disclosure posture must be explicit, not conflated with write. | Rejected exit `5` as `foreign-acl`; final child not created. |
| Ancestor named access ACL, effective `rwx` | Write plus search is unsafe replacement authority and must fail closed. | Rejected before child creation (`wrong-mode` from ACL-correlated mode bits); security outcome correct. |
| Ancestor default ACL with effective `--x` | Separate inheritance surface; reject before candidate creation in V1. Later support requires a separately approved contract. | Rejected before child creation as `foreign-acl`. |
| Ancestor default named entry masked to `---` | Still inheritable; must not be accepted merely because current effective access is zero. | Ancestor passed; child inherited access/default ACLs; final validation failed `foreign-acl`; invalid candidate remained and identical rerun failed again. |
| Final root exact owner, `0700`, no observable access/default ACL (`NoData` under mode authority) | Accept and idempotently scaffold without claiming physical absence. | Accepted; `substrate --version` succeeded. |
| Final root with named access ACL | Reject every extended access entry, even masked ineffective. | Rejected `foreign-acl`. |
| Final root with default ACL | Reject every default ACL. | Rejected `foreign-acl`. |
| Wrong owner | Reject exact owner mismatch. | Focused intended-owner test passed; no privileged live mutation was attempted. |
| Wrong mode (`0750`) | Reject without repair. | Rejected `wrong-mode`. |
| Symlink candidate | Reject no-follow. | Rejected `symlink`. |
| Replacement/identity drift | Fail closed and do not write the replacement. | Focused byte-exact rename and facade-handoff replacement tests passed. |
| Malformed/unreadable ACL | Fail closed as malformed or validation unavailable. | Malformed parser test passed; source maps unreadable/unexpected xattr state to fail-closed validation. No kernel ACL corruption was manufactured. |

### Product installer matrix

`<audit>` below means the exact audit-owned root above. No wildcard deletion or real-home ACL
mutation was used. After evidence capture, that exact root was removed with a same-filesystem,
owner-checked traversal; it no longer exists. The real default home remained absent and the world
service/socket/runtime paths matched the initial absent/inactive snapshot.

| Exercise | Result |
|---|---|
| `cargo build -p substrate --bin substrate` | Passed. |
| Fresh default `dev-install-substrate.sh --profile debug --no-world` | Failed at private-home bootstrap on the real non-writing ancestor access ACL; real default root remained absent. |
| Custom `--prefix <audit>/product-prefix-only`, no outer home | Private bootstrap and generated projections used the prefix; shim deploy fell back to the default home and failed. Rerun failed at the same stage. |
| Same custom prefix with outer `SUBSTRATE_HOME=<prefix>` | Diagnostic workaround succeeded; repeat install succeeded and shim deploy reported up-to-date. This is not product-gate proof. |
| Shim status/deploy/remove with explicit selected context | Selected shim directory and 25 commands were reported; repeat deploy joined; removal succeeded. |
| Generated files | `env.sh` and the dev shim helper encoded the selected prefix; manager environment could self-locate; configuration/install state/version/dependency artifacts were inspected for prefix-relative placement and modes rather than falsely treated as encoded references. Root/shim directories were `0700`; the sensitive helper was `0600`. Conflicting ambient-home consumption remains unproven and defective by source. |
| Doctors after diagnostic install | `world doctor --json` exited `4` because world was intentionally disabled/unprovisioned; `shim doctor --json` exited `0` but projected `trace_log` from `/home/spenser/.substrate`; `health --json` exited `0` with world disabled. |
| Uninstall same prefix, no outer home | Shim removal fell back to default home and warned; manual selected-prefix cleanup returned success but left managed host and Linux `substrate-gateway` symlinks. |
| Uninstall with outer home | Shim removal used the selected prefix, but the same two gateway symlinks remained. |
| Uninstall followed by reinstall | Reinstall with the diagnostic override succeeded, proving remaining artifacts are currently tolerated rather than proving correct cleanup; final diagnostic uninstall reproduced the leftover symlinks. |
| Full world/Codex provisioning | Not run: passwordless sudo was unavailable and this was not established as a dedicated mutable service host. Static review proved the Unix release selected-home/`ReadWritePaths` mismatch and incomplete system-artifact cleanup; live product proof remains mandatory. |
| `cargo test -p shell --lib private_home_ -- --nocapture` and focused ACL/owner/replacement tests | Passed. |
| `cargo test -p shell --test installer_env_wcu4 -- --nocapture` | Four static tests passed; `config_current_show_is_not_affected_without_override_inputs` failed before its assertion because its private-home fixture is hard-coded below other-writable `/tmp`. This is a stale harness/missing-regression result, not positive installer proof. |

### Stable findings and bounded next proof

| Finding ID | Classification | Expected versus actual / security and product impact | Owner and smallest remediation | Required regression/product smoke | Platform / closeout block |
|---|---|---|---|---|---|
| **A1D5I-HOME-01** | `ContractGap` | Ancestor access and default ACLs are not separated by effective authority. A masked `--x`/`r-x` access entry lacks write/replacement authority, while a default ACL can alter a child. | HostSessionAuthority; R1 defines masked access semantics, rejects any effective write bit and every ancestor default ACL in V1, and preserves fail-closed uncertainty without weakening the final root. | Masked `---`/`--x`/`r-x`/write-only/combined-write matrix plus default ACL and malformed/unreadable cases; normal Linux install on the observed host. | Linux proven; static Unix applicability; both A1.1d Linux and B1/B2.1 Linux product smoke blocked. |
| **A1D5I-HOME-02** | `ImplementationBug` | `parent_acl_grants_named_principal` rejects every nonzero masked permission, so supportable non-writing traversal blocks default install before the approved Case A decision can be represented. | HostSessionAuthority; R1 implements strictly parsed masked non-writing Linux POSIX access-ACL support and rejects effective write or uncertainty. | Unit/property tests over ACL mask/effective rights and live bootstrap under each approved/rejected access-ACL case. | Linux proven; macOS requires native ACL mapping/proof; both blocked. |
| **A1D5I-HOME-03** | `DiagnosticBug` | An ancestor failure is attributed to the final requested home as generic `foreign-acl`, without path, role, ACL kind, or effective authority; it can also say “Existing roots” when no final root exists. | HostSessionAuthority error boundary; R1 structured diagnostic with sensitive-principal suppression. | Exact diagnostic assertions for ancestor/final/access/default/unavailable/new-candidate cases. | Unix-facing; both blocked because diagnostic/product proof is required. |
| **A1D5I-HOME-04** | `ImplementationBug` | A zero-effective default ACL passes ancestor validation, is inherited, fails final validation, and leaves a non-convergent current-attempt candidate. | HostSessionAuthority R1 rejects every ancestor default ACL before creation; R3 may remove only an exact descriptor-rejoined empty candidate created by the current attempt, never recursively or from pre-existing provenance. | Inherited-default pre-create rejection and synchronous current-attempt empty-candidate rollback/rerun tests; replacement/nonempty/ambiguous/`AlreadyExists` candidates and pre-existing invalid roots remain byte/metadata unchanged. | Linux proven; both blocked. |
| **A1D5I-HOME-05** | `ContractGap` | After an abrupt interruption, an unaccepted invalid candidate has no trustworthy current-attempt provenance on rerun. The no-repair rule forbids treating `AlreadyExists` as deletion authority, so arbitrary crash-window convergence cannot be promised. | R3 proves every reachable post-R1 crash residue is either already valid and exact-joinable or stays fail-closed. If invalid residue is still reachable and product convergence is required, stop for a separately approved provenance/publication mechanism; never infer provenance from the name. | Kill-point matrix around parent validation, creation, first open, validation, cleanup, and acceptance; valid residues join, exact synchronous failures clean safely, invalid unknown-provenance residues remain unchanged/fail-closed. | Unix contract; both gates remain blocked until R3 resolves or explicitly bounds every reachable state. |
| **A1D5I-INSTALL-01** | `ImplementationBug` | Selected custom prefix is lost at shim deploy/remove. Outer `SUBSTRATE_HOME` makes the same install work, proving propagation—not final-root validity—is the failure. Unix release deploy and Windows dev deploy/remove have the same static omission. | Product child adapter; R2 carries the host `InstallBootstrapContextV1` explicitly everywhere. | Dev/release custom-prefix install/uninstall without outer overrides; shim status/deploy/remove and encoded/self-derived/prefix-relative generated-artifact assertions; native platform runs where supported. | Linux live; Unix/Windows static; both blocked. |
| **A1D5I-INSTALL-02** | `ImplementationBug` | Unix release uninstall locates by `SUBSTRATE_ROOT` but `uninstall-substrate.sh` deletes `.substrate` relative to ambient `HOME`; a custom-prefix uninstall can miss the selected tree and target unrelated default-home state. | Release uninstall boundary; R2 selects exact matching context and R3 limits deletion to recorded managed state. | Hermetic two-home negative test proving selected-only removal and zero ambient-home mutation; release install→uninstall→reinstall product smoke. | Unix static high-impact path; both blocked. It was not executed against real home. |
| **A1D5I-INSTALL-03** | `ImplementationBug` | Dev uninstall omits managed host and Linux `substrate-gateway` symlinks while removing sibling managed links. | Dev lifecycle cleanup; R3 exact managed-artifact inventory and convergence. | Install→uninstall exact-artifact diff, repeat uninstall, reinstall, and unrelated-symlink preservation. | Linux live; both blocked. |
| **A1D5I-INSTALL-04** | `DiagnosticBug` | With selected custom home, shim doctor reports the trace log under default `$HOME/.substrate`; focused bootstrap success can therefore misstate diagnostic authority. | Shim diagnostic projection; R2 consumes selected context consistently. | Custom-home shim/trace doctor assertions and emitted trace-location product check. | Linux live; cross-platform static review required; both blocked. |
| **A1D5I-INSTALL-05** | `ImplementationBug` | Unix release unit sets `SUBSTRATE_HOME` to the selected prefix but derives `ReadWritePaths` from ambient `$HOME`; `ProtectSystem=strict` can deny a valid custom home outside that allowlist. | Release Linux service projection; R2 derives the sandbox allowlist from the selected host context. | Full-world release smoke with a custom prefix outside ambient home, service write/bootstrap proof, and exact unit assertions. | Linux static; both blocked. |
| **A1D5I-INSTALL-06** | `ImplementationBug` | Linux provisioning installs the gateway, ACL helper, and socket drop-in, but dev removal omits them; release removal also omits the installed gateway. Prefix-local success cannot stand in for system lifecycle convergence. | Linux managed-system lifecycle; R3 records exact pre-state/managed manifest and restores binaries, helpers, units/drop-ins, sockets, runtime/state paths, and installer-created account-state changes without broad deletion or removing pre-existing state. | Dedicated-host install/uninstall pre/post diff, repeat uninstall, accepted-home partial provisioning/rerun, exact group/ACL-bridge/linger accounting, and preservation of pre-existing system artifacts. | Linux static; both blocked. |
| **A1D5I-INSTALL-07** | `ImplementationBug` | Windows release uninstall recursively deletes every `$USERPROFILE/.substrate*` match independent of `-Prefix`, so unrelated backups/state can be destroyed. | Windows uninstall boundary; R2 selects exact context and R3 forbids wildcard cleanup in favor of exact managed identity. | Hermetic native Windows negative test with selected prefix plus unrelated `.substrate*` siblings; only selected recorded artifacts may change. | Windows static only; native Windows product proof pending; neither named Linux closeout gate is blocked by this row alone. |
| **A1D5I-INSTALL-08** | `ImplementationBug` | Windows release `-NoAutoSource` deploys shims without explicit selected context; macOS dev cleanup targets `${HOME}/.substrate/sock/agent.sock` instead of the selected host prefix. | Platform lifecycle adapters; R2 carries host context and exact platform mapping through deploy and transports the selected macOS socket target; R3 alone performs socket cleanup and preservation/convergence. | Native Windows `-NoAutoSource` custom-prefix shim proof, R2 native macOS selected-socket mapping proof, and R3 native macOS cleanup/preservation proof. | Static Windows/macOS only; native macOS/product proof remains open; neither named Linux closeout gate is blocked by this row alone. |
| **A1D5I-INSTALL-09** | `ImplementationBug` | Generated release/dev manager scripts prefer ambient `SUBSTRATE_HOME=B` over their installed self-location A, so a custom-prefix projection can consume a different authority home. | Generated projection consumer; R2 binds encoded/self-derived location to the selected host context and rejects conflicting ambient selection. | Install at A, source under ambient B, and prove only A is consumed; include missing/malformed A and B negative cases. | Unix static; both blocked. |
| **A1D5I-INSTALL-10** | `ContractGap` | Windows release uses a prefix-independent forwarder PID path and removes the prefix tree without a selected-context shim-remove boundary. The current pack does not define which artifacts are per-prefix versus intentionally per-user shared. | R2 classifies and transports exact prefix-scoped versus SID+platform-instance+pipe-scoped context without deletion authority; R3 alone defines any managed-artifact ownership manifest used for removal and removes only exact matching provenance. | Two-prefix native Windows lifecycle matrix covering shared/per-prefix forwarder and shim state, conflict, partial install, uninstall order, and unrelated-state preservation. | Windows static only; native Windows product proof pending; neither named Linux closeout gate is blocked by this row alone. |
| **A1D5I-REG-01** | `MissingRegression` | Existing installer tests check selected strings but not real child propagation/lifecycle; one fixture uses `/tmp`, which is invalid under the current ancestor contract and masks its intended assertion. | R1/R2/R3 regression suites. | Secure audit-owned fixture roots; default/custom, accepted-home partial/rerun, synchronous rollback, crash-window fail-closed, uninstall/reinstall, and negative two-home matrices. | Linux proven; both blocked. |
| **A1D5I-ENV-01** | `EnvironmentUnsupported` | This audit host could not safely run privileged service/Codex provisioning without a sudo prompt/dedicated-host restoration contract. This is an evidence limitation, not a product defect or capability regression. | Later R2/R3 product-smoke environment. | Snapshot, provision, doctor/smoke, runtime sync, uninstall/restore, and post-snapshot parity on a dedicated supported Linux host. | Linux proof still required; gate stays open, but classification itself blocks neither architecture core. |
| **A1D5I-FP-01** | `FalsePositive` | Install/bootstrap unavailability does not prove loss of world filesystem/network capability, changed policy/enforcement, or regression of review-clean B1 receipt/B2.1 supervisor semantics. | No remediation outside R1/R2/R3. | Preserve the differential world/policy wall and later product smoke; do not reopen B1/B2.1-0. | All platforms; neither semantic core is regressed. |

### A1.1d-5R2-0 propagation planning record

**Decision/status:** the docs/evidence-only planning packet starts at
`6ab2a515e13946324d0aac25b144e1c3408cb2c1` and freezes 118 live propagation edges as
PI-001–PI-118 in `02-seam-crosswalk.md`. It changes only the existing six control-pack files,
implements no runtime behavior, performs no privileged/platform mutation, promotes no seam, and
claims no native macOS or Windows proof. At that planning point the corrected R2 implementation
order was R2-1 -> R2-2 Routes A-D -> R2-2E -> R2-2F -> R2-2 integration closeout -> R2-3 -> R2-4;
F0's later evidence-gate insertion supersedes that outgoing sequence without changing inventory
ownership. R3 follows and remains unimplemented.

Every A1.1d-5I finding has an explicit terminal owner:

| A1.1d-5I finding | Frozen owner/packet | Closure rule |
|---|---|---|
| HOME-01, HOME-02, HOME-03 | R1, already review-clean | Preserve effective-authority and diagnostic contract; R2/R3 do not reopen it. |
| HOME-04 | R1 pre-create rejection; R3 synchronous exact-candidate cleanup | R2 may carry context only; it cannot remove the candidate. |
| HOME-05 | R3 crash-window reachability/convergence decision | Unknown-provenance invalid residues remain unchanged and fail closed absent a separately approved boundary. |
| INSTALL-01 | R2-1 Unix dev/shim; R2-2 Unix release; R2-3 Windows/platform | Complete context reaches deploy/remove/status/doctor without outer override. |
| INSTALL-02 | R2-2 exact release-uninstall selection; R3 deletion/convergence | Selection and deletion authority stay separate. |
| INSTALL-03 | R3 | Exact managed gateway/symlink cleanup only. |
| INSTALL-04 | R2-1, joined in R2-4 | Shim/trace diagnostic reports selected A. |
| INSTALL-05 | R2-2, proven in R2-4 | Linux `ReadWritePaths` is derived from selected context. |
| INSTALL-06 | R3 | System helper/gateway/unit/drop-in/socket/account-state cleanup. |
| INSTALL-07 | R2-3 exact Windows selection; R3 deletion | Wildcard removal is never R2 authority. |
| INSTALL-08 | R2-3 context/mapping/target transport; R3 cleanup action | Windows `-NoAutoSource` and macOS selected-prefix socket mapping are R2; socket removal/preservation/convergence is R3; native evidence remains assigned. |
| INSTALL-09 | R2-1/R2-2, joined in R2-4 | Generated manager binds A and cannot accept ambient B as authority. |
| INSTALL-10 | R2-3 scope classification/context transport; R3 deletion manifest/action | R2 records prefix versus SID+instance+pipe scope without inventing a removal manifest. |
| REG-01 | R1 preserved; R2-1/R2-2/R2-3 tests; R2-4 join; R3 lifecycle suite | Secure fixture roots and the named two-home/lifecycle matrices are mandatory. |
| ENV-01 | R2-4 Linux product host; native platform assignments remain separate | Evidence limitation is not positive product proof. |
| FP-01 | Out of scope; preserve review-clean behavior | No world capability, policy, B1 receipt, or B2.1 supervisor reopening. |

The R2 regression gates are exact labels used by the inventory and packet allowlists:

| Gate | Required proof before the owning packet can exit |
|---|---|
| **R2-UDEV-01** | Unix dev install, uninstall, `dev-shim-bootstrap.sh`, and successful install-sensitive standalone CLI modes each construct exactly one equal A/H/R context from the declared/installed-witness/account-default source before explicit-context home bootstrap; parse failures, help, `--version`, and `--version-json` mutate no scaffold; installers and focused R1 private-home tests bootstrap only through hidden `--install-bootstrap-home-v1` plus the authenticated argv carrier, current account+UID equality, and checked H/R, after which the process returns without normal dispatch; environment alone cannot select the action; custom A/bin dev symlink self-derives A without an outer override; direct repo and zero/multiple witnesses fail; conflicting ambient B/dev-prefix cannot retarget bootstrap, children, preexec, or uninstall selection; malformed/tampered/forged-principal carriers fail before mutation. |
| **R2-UREL-01** | Unix release wrapper, direct installer/uninstaller, and installed child distinguish constructor/child modes and preserve the same context across install/uninstall; Unix release A/bin witness self-derives A; every child binds the committed principal to current Unix identity/sudo origin and cannot reinterpret prefix; repeat install preserves commitment; no removal/convergence claim. |
| **R2-SHIM-01** | Installer-managed, automatic CLI, standalone declared/self-derived deploy/remove/status/doctor/repair, and physical shim paths receive one complete context or verified mapping projection; a bare physical shim recovers exactly one no-follow invocation witness without PATH precedence, resolves the canonical current Unix account+UID or Windows account+SID through its exact OS observation surface, and rejects zero/multiple witnesses or a forged principal before dispatch; repair target derives from the committed principal; telemetry and manager-hint manifest/overlay consume custom A under ambient B, with no compiled-repo or manifest-environment fallback in normal product mode. R2-1 proves only shell `ExplicitProduct`: trace output `A/trace.jsonl`, policy Git A, repeated A reuse, conflicting-path rejection, missing-metadata no-fallback, and zero B access while the setter/default physical-shim/replay/platform behavior remains unchanged and unpromoted. R2-3 migrates physical shim and already-frozen replay/platform callers, removes/makes unreachable `LegacyAmbientCompatibility`, and proves the final global unbound-init failure rule. Legacy H/R/carriers remain consistency checks. Replacement/migration/recursive removal actions and trace rotation/retention semantics remain byte-frozen in R2. |
| **R2-GEN-01** | Dev/release `env.sh`, manager, Bash preexec, helper, generated `A/manager_hooks.yaml`, configuration, version, install-state, dependency, service, intended-principal PATH, and Lima known-hosts projections are prefix-relative or self-derived, encode/project the same context where consumed, and never treat a conflicting ambient B, repository path, or generated value as selection authority. |
| **R2-RUNTIME-01** | Before world-enable dispatch, nested `--home` or global `--install-prefix` selects A, equal normalized selectors join one context, conflicting selectors fail before context construction or mutation, absence of both selectors permits only a verified installed-product witness, and every declared selector on an authenticated internal carrier must match it; ambient B and runner-local state never select A. Typed IH crosses `run_shell_with_cli` → the existing Unix `ShellConfig::from_cli` world branch → `handle_world_command` → `run_enable` by explicit argument at every hop; H/R/carrier environment values are checked projections only, and the runner performs no reconstruction. At the child boundary, `run_enable` or its existing provision-deps path canonically encodes that IH and `run_helper_script` transports the exact authenticated carrier on hidden argv without interpreting or logging it; `world-enable.sh` discriminates child mode by argv presence, validates the carrier and current principal/exact sudo origin, requires normalized `--home` equality, installs checked projections, and only then dispatches. Missing, malformed, duplicate, tampered, reordered, forged-principal, or conflicting input fails before action and never falls into public mode; environment-only state cannot recover authority. `run_enable`, including its existing provision-deps branch, derives A from typed IH and passes explicit A to `update_manager_env_exports`; conflicting ambient B cannot retarget the generated manager export. Routes A–D preserve their reviewed behavior but do not prove non-enable Gateway/Deps or `run_sync_after_provisioning`: R2-2E owns authenticated gateway projection and R2-2F owns normal world-deps/provision/post-sync propagation. E has an authenticated Linux Gateway route through fixed `/run/substrate.sock`; because source closure found no authenticated macOS platform-endpoint source in E, macOS must fail before client construction or `auto_select`, and Windows/other contextless Gateway cfgs must fail before config, policy, inventory, disabled-response, or client selection. Their ambient clients remain frozen R2-3 compatibility. F's authenticated world-deps and doctor guarantee is likewise Unix/Linux only; macOS, Windows, fallback, and other non-Unix adapters remain explicit unproven R2-3 compatibility or unavailable/fail-closed paths. Host/Health/Config/Policy same-process branches receive exact typed IH; Unix Health reaches the existing `report::collect_report_for_context` only through a crate-private `shim_doctor::collect_report_for_context` name exposure, then carries typed IH through the snapshot path. The context-aware export changes visibility only. The existing `collect_report` compatibility re-export and collector may receive only item-level Unix `unused_imports`/`dead_code` annotations whose reasons name temporary R2-3 compatibility ownership; they have no runtime or non-Unix effect, and no broader lint allowance is permitted. Named checked-projection `collect_report` remains behavior-frozen, cannot be selected by typed Health, and is barred from product proof pending R2-3 migration/removal. Route A production proof remains patch-bound to immutable SHA-256 `ea4abf43043013994e698d54f21c04bba58833b3bb3247bf6d5d3b454f51b943` and its exact eleven-file fingerprints/source-level diff. Its sole successor may add item-level Unix cfg only to the two exact tests and their sole test-only `HostSessionAuthority` imports, plus focused Health conflicting-A/B and Host installed-witness regressions specified in `03`/`04`; completion requires a new exact candidate hash/manifest/fingerprints, deterministic base-to-successor hunk map, unchanged production bytes, complete fresh-label attribution mapping, and CLEAN review. Historical GitNexus observations are 33/18, reverse 11/18, 37/18, 39/18, and current base CRITICAL 24/31; counts and generated labels remain required evidence but are not raw completion gates. Stale-index pinning is forbidden, and every authorized test edit receives normal pre-edit impact. Native non-Linux proof is unavailable here when MSVC `lib.exe` or the Apple SDK `TargetConditionals.h` is missing; that is neither success nor a product regression. Completion requires E to bind gateway config/effective-policy/network/runtime-family/Codex projections to A and F to bind every current/global/workspace/runtime/provision/post-sync path plus doctor constituent identity to the same A. PI-105 derives process-scoped `PlatformPrincipalV1` only from validated IH, carries it through normal REPL/hidden owner-helper and prepared member dispatch without durable authority changes, requires Unix account+UID database round-trip, and targets only that account's `.codex`; ambient/root home and A cannot retarget it. World-service host diagnostic fields remain absent while the host shell attaches optional selected-prefix/commitment fields from typed IH. Mixed-authority diagnostics cannot report success. Rollback and synthetic-auth deletion remain R3. Runtime, policy, gateway, provider, orchestration, receipt, supervisor, retained-worker, and world capability semantics do not change. |
| **R2-LINUX-01** | Unix account+UID and context survive every release, dev-install, and provision sudo boundary: context-aware Substrate helpers validate the full argv carrier; arbitrary tools receive only exact context-derived argv after parent revalidation and no preserved environment; the ACL bridge receives no context and rejects every tuple except its exact three fixed mode/target/`substrate` combinations while preserving group-member enumeration. Linux unit environment carries H=A, R=A, commitment, and intended-principal projection; socket/drop-in and `ReadWritePaths` derive from A; only the fixed same-attempt Linux socket-restart unlink is exercised; focused/static proof passes in R2-2 and dedicated-host service/world/Codex product proof passes in R2-4. |
| **R2-DIAG-01** | Trace, shim, repair, world, host, health, world-deps, config/policy proof, gateway, Lima/WSL, and pipe diagnostics report selected host commitment, verified platform mapping/transport, and host platform-control root where applicable; direct mode constructs from declared/principal/OS Known Folder inputs, internal mode validates hidden argv and matching scrubbed projections, and no default-home/guest/pipe/`LIMA_HOME`/`LOCALAPPDATA` reconstruction is represented as authority. `WorldDoctorReportV1` host fields are optional/defaulted and omitted by the world-service producer; only the host shell enriches them from typed IH, preserving old wire JSON. Health/shim snapshots receive typed IH/A-derived paths. The world-deps fixture remains A-rooted and F5-owned. Under F5-PD, authenticated Linux production never selects the World Doctor fixture and instead runs the hidden passive child with the same carrier plus checked A projections; that fixture is Linux test evidence only. Existing non-Linux fixture/public-child compatibility remains behavior-frozen, unproven, and unable to satisfy F5-PD/F5 authority or native proof. R2-2F additionally requires exact non-secret prefix/commitment equality for every Linux world/config/policy/inventory/dependency/runtime constituent: missing `ok`, missing identity, or mixed A/B becomes unavailable/incoherent with `ok=false`, never healthy A-bound truth. Host/World doctor consumes A-derived world-fs policy rather than ambient broker state. The named checked-projection shim compatibility caller remains behavior-frozen and cannot satisfy R2-2 proof. Migrated shell live trace/policy production uses only entry-bound A; `LegacyAmbientCompatibility` remains named, temporary, behavior-equivalent, and non-promotable until R2-3 migration/removal. `SHIM_TRACE_LOG` cannot satisfy R2-1 product proof. |
| **R2-MAP-MAC-01** | IH plus the VM selector and host account-database-derived `/.lima` control root performs only Stage-1 declared-instance realization; parents scrub/overwrite child `HOME`/`LIMA_HOME`, direct internal mismatch rejects, and PM is finalized from running-guest machine ID/account+UID+home before R2 guest projection. R2 fixes the future V1 SSH-UDS target from `A/sock/agent.sock` to `/run/substrate.sock`; backend auto-selection, `vsock-proxy`, TCP, ambient endpoints, and ambient Lima store cannot replace it. The validated R2 path stops before forwarder launch with an explicit R3 prerequisite, so socket unlink, `StreamLocalBindUnlink`, timeout kill/wait, handle-drop teardown, retry, and convergence remain exclusively R3 and are not exercised as R2 proof. Host commitment also binds the guest unit and future A-scoped known-hosts projection; shell/shim/replay factory callers are explicit; no path/principal equality. Static proof is labeled static and a native pre-existing-Lima no-forwarder A/B mapping-only run is assigned separately; macOS product transport remains pending R3. |
| **R2-MAP-WIN-01** | Windows account+SID host context survives dev/release/uninstall and `-NoAutoSource`; the release A/bin copy self-derives A under ambient B and every internal carrier is bound to the current account+SID, with forged-principal rejection. It binds one WSL instance, WSL-native account+UID+home, one normalized public-selected/default pipe, and an OS Known Folder control root. A canonical SID+registered-distro+machine-ID+pipe digest scopes the shared PID root; config/logs are under A and all paths are explicit, so `LOCALAPPDATA`/`USERPROFILE` cannot select them. Warm/forwarder/backend/status/doctor plus shell/shim/replay consume the same mapping. The forwarder validates PM/current token before `wsl -d`; its WSL child receives only PM-derived distro, normal-product guest target, and commitment despite conflicting ambient config/target/`WSLENV`. The fail-closed WSL guards remain byte-identical; creation, replacement, timeout kill, stop, PID deletion, and convergence remain R3; static proof is labeled static and native existing-instance A/B mapping-only proof is assigned without claiming provisioning. |
| **R3-LIFE-01** | R3-only candidate, rollback, manifest, managed-system cleanup, account-state restoration, crash-window, uninstall/reinstall, shim/payload/bin/cache/helper/unit/socket/platform-staging/forwarder unlink/drop/timeout/synthetic-auth cleanup, and unrelated-state preservation matrix. Referencing this gate from R2 transports a target only. |
| **R3-WIN-01** | R3-only native Windows two-prefix cleanup matrix: no wildcard removal; exact per-prefix versus shared state; version/bin/profile replacement, timeout rollback, uninstall order, WSL/forwarder/shim cleanup, partial install, and unrelated-state preservation. |

For Route B, the immutable WIP is exact patch SHA-256
`f8f845ef6aa6688fdf60be6ed983bb119971d65895a0f38c25fc1dad7643a77f` across the six PI-105
production files at preservation commit `a6e29a10ea3dc9cb673a22912002efb83658e7b2`. Its refreshed
GitNexus observation is CRITICAL, 17 attributed symbols, and 25 existing process labels; every
label maps in `03-phase-slice-map.md` to the shell-root principal extraction, hidden owner-helper
transport, or prepared member-request/seed-resolution root, with downstream public Agent,
policy/filesystem/network, validation, carrier, and path labels classified as unchanged
attribution. The final candidate may add only one item-level `dead_code` annotation to the
principal-less compatibility entrypoint, with a reason stating intentional non-use and temporary
R2-3 ownership. That entrypoint remains uncalled by typed product routes and fail-closed before
allowlisted Codex credential projection. Completion requires exact six-file/hunk containment,
fresh semantic review, and no new module, execution family, authority source, schema, secret
surface, capability, lifecycle, Route C, Route D, or R2-3 behavior.

Route B's exact post-lint, pre-format candidate is preserved at ordinary/binary SHA-256
`e9da85eb206be522645120dfee459ee79ce48793bc61161487d4b5c4d2fda243`, commit
`07f3117aa0d2f3d57279d20fec204757bba8383a`, with the same six-file manifest. Its only permitted
successor is repository-default `cargo fmt --all` output in `agents_cmd.rs`,
`orchestrator_world_dispatch.rs`, `world_ops.rs`, and `async_repl.rs`; `invocation/plan.rs` and
`routing.rs` remain byte-identical. A new candidate hash and fingerprints are recorded after
proving every successor-only hunk is whitespace/layout, the whitespace-ignored source diff is
empty, format check passes, and every refreshed GitNexus label remains an existing Route B root or
formatting attribution. Raw GitNexus count drift does not authorize semantics. A seventh file,
non-formatting hunk, changed unaffected-file fingerprint, new module/family/path, or remediation
beyond canonical formatting stops before runtime review or commit. Fresh runtime review remains
mandatory; this exception imports no Route C, Route D, R2-3, capability, cleanup, or lifecycle work.

The formatted Route B candidate is preserved at ordinary/binary SHA-256
`ea9cf3e582650007083812ad70e0bf3198405e73d0cde0fb8ecfbedeb49884a2`, commit
`57e13d291abf1239aacee0020ac444ec05e11d56`, with the same six files. Security review found three
valid R2-2 defects: an ambient reserved seed value could survive injector early returns;
principal-aware direct/prepared Spawn dropped the typed principal before member-request
construction; and `cfg(any(target_os = "linux", test))` exposed Unix account calls in Windows test
builds. The only authorized successor clears the reserved key before every decision, explicitly
passes `Some(exact principal)` through both live principal-aware Spawn routes while compatibility
passes `None`, and narrows the account resolver/helper plus mechanically required test/import cfgs
to Linux production or Unix tests. Poisoned-input, principal-aware direct/prepared Spawn,
compatibility fail-closed, and static cfg regressions are mandatory. The six-file manifest is fixed;
no durable authority/schema change, ambient recovery, policy/capability change, Route C/D, R2-3,
cleanup, or lifecycle work is imported. GitNexus raw counts remain evidence, while the exact patch,
fingerprints, hunk map, complete manual under-resolved Spawn closure, and CLEAN reviews are the
semantic containment boundary.

### Historical Route C authorization checkpoint

Route B is preserved exactly and remains review-clean at
`6cee990f0370013c8b05a5495301db7aea642cd5`, ordinary/binary patch SHA-256
`28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674`, with its reviewed
six-file semantics unchanged. At that checkpoint, the seven unpushed runtime commits from published
baseline `dab71d816f8e3a51d841c2293b646463e2d28cfc` were replay inputs only: the docs correction had
to land on that published baseline first, and each runtime commit had to replay with identical
ordinary/binary patch and file scope before Route C proof resumed. That historical replay completed
and is not renewed-closeout authority.

The preserved Route C candidate is commit `41b82327e23798719ed9a0b4cae1f557fb593670`, parented by
the exact Route B tree, with canonical full-index binary SHA-256
`5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277`, 182 insertions/five
deletions, and the exact five-file manifest/fingerprints in `04-contracts-and-gates.md`. It is
authorized only as an authenticated, non-authoritative Host/World doctor projection. The four
existing diagnostic roots are `handle_host_command`, `host_doctor_main`, `handle_world_command`,
and `world_doctor_main`; no world-service, representation, helper, or test symbol becomes another
execution root or authority owner. The preservation-time GitNexus result was CRITICAL 25 symbols/32
labels/five files; a byte-identical refreshed index reported CRITICAL 19/32/five. Review must decide
semantic containment from the patch, fingerprints, exact manifest, source mapping, owner/family
set, and authority behavior—not a brittle numerical ceiling—and must explicitly confirm any
attribution-only drift.

Route C cannot establish context, recover it from ambient or generated projections, expose hidden
carrier/credential/request/secret bytes or sensitive principal material, or mutate installation,
service, world, policy, capability, filesystem/network enforcement, placement, caging, credential,
receipt, supervisor, retained-worker, cleanup, lifecycle, or execution semantics. Route D and R2-3
remain unstarted and no seam is promoted. A sixth file, actual new semantic path, new module owner
or execution family, or broader authority owner is respectively `ImpactDecisionRequired` or
`CrossDocumentChangeRequired`. Review remediation may stay within the exact five-file semantic
envelope only after recording a new patch/fingerprint set and rerunning impact, proof,
differential, and fresh reviews.

The Route C regression ledger requires exact reproduction of the inherited disabled-world doctor
failure on the clean replayed Route B baseline; transport API and world-service proof; installed-
witness Host A and World typed-A JSON under ambient B; applicable malformed/missing-context
fail-closed proof; warnings-denied touched-crate Clippy; shell/workspace all-target compilation;
format/diff checks; and a broad shell differential with zero `PassToFail`, `NewFail`, removed,
renamed, substituted, weakened, or newly ignored tests. Retained failures keep normalized
signatures, and each `FailToPass` receives a no-bypass causal audit. Three fresh read-only review
perspectives—authority/security, call-path/impact, and cross-platform/regression—must be CLEAN
before one local Route C commit and its remote preservation ref. Source runtime commits remain
unpublished until complete R2-2 closeout authorizes publication.

### Historical Route D docs-first source-closure correction

Route D started from replayed Route C commit `fe288d233b5a198e19c2afba857d02e982ef6e1b`, tree
`1703a0e0f57c8c994a3fd59f276c0f1a28dfbb50`. Routes A, B, and C were frozen. The eight then-unpushed
runtime commits above published docs commit `f2131f13424906a167f58795f8eb6175cc384f14` were replay
inputs only after that correction published first; their per-commit and aggregate ordinary/full-
index binary patch identities and file manifests remained exact. That Route D replay completed and
is packet-scoped historical evidence, not renewed-closeout authority.

The read-only Route D audit found two ambient pre-carrier branches in the existing
`crates/shell/src/builtins/shim_doctor/report.rs` family. `try_load_health_fixture` and
`health_fixture_path` could select `B/health/world_deps.json` or
`B/health/world_doctor.json` before typed collection, while `gather_world_doctor_snapshot` and
`run_json_subcommand` could launch the current repository binary as `world doctor --json` without
the authenticated witness. A malicious or malformed B fixture could therefore replace A's report
or disclose B's absolute path, and the contextless child could fail or execute against B.

At that checkpoint the correction authorized only the Route D carrier mechanics inside that
already-allowed report file: `build_report` passed the typed carrier to both gather branches, both
fixture lookups derived only from A, and the existing Unix world-doctor child received the same
canonical hidden argv carrier plus context-derived checked child projections. F5-PD later
supersedes the authenticated Linux World Doctor portion: its production fixture is removed and its
child becomes passive. Non-Linux compatibility remains unchanged and cannot satisfy proof. The
existing Unix `collect_report` compatibility
body, visibility, cfg, re-export, caller, output, and behavior remain frozen until R2-3.

GitNexus reports LOW upstream impact for `gather_world_doctor_snapshot` (one direct/five impacted),
`try_load_health_fixture` (two/four), `health_fixture_path` (one/four), and
`run_json_subcommand` (one/three). Each maps to the existing `health::run`/shim-doctor family; raw
counts are observed evidence, not a brittle ceiling. Completion requires the exact approved
source/test manifest, source-level caller/cfg mapping, no new owner/module/execution family, final
change detection, and fresh semantic containment review.

Historical Route D regressions create conflicting B world/deps fixtures under typed A, prove neither
fixture can win or leak, and prove the nested world snapshot receives A's authenticated witness.
F5-PD replaces the Linux World Doctor fixture/success expectation with test-only fixture and
truthful passive unavailable proof. The combined gates scan JSON, human output, errors, logs,
traces, snapshots, and fixtures for carrier, credential,
request, non-public commitment, and sensitive-principal bytes. Missing/malformed/tampered/contextless
input fails closed; success and failure preserve complete A/B trees and the parent environment;
environment-only input cannot invoke the typed route; compatibility collection remains unchanged;
non-Unix cfg compilation remains unchanged. Focused diagnostic suites, warnings-denied Clippy,
shell/workspace all-target checks, format/diff checks, final GitNexus mapping, and the broad shell
differential remain mandatory. Any new resolver/schema/authority owner, ambient fallback, physical
shim/replay/platform migration, world/policy/capability change, or installation, cleanup, service,
credential, receipt, supervisor, retained-worker, or lifecycle change is
`CrossDocumentChangeRequired`.

The optional named checkpoint and cross-document verification skills are unavailable for this
increment. Their explicit substitutes are immutable commit/tree/patch/file checkpoints at every
phase and a six-file structure, relative-link, table, fence, gate, sequence, and stale-status
cross-check. The security skill's missing supplemental checklist is replaced by the complete
embedded security checklist. These substitutions do not weaken any gate.

### R2-2 historical failed integration closeout and remaining-seam correction

The integration attempt began from source branch
`feat/internal-host-orchestrator-world-dispatch-bootstrap` at Route D commit
`7a3e6ee424e726e3600d39e40b64df2d02cfff73`, tree
`945923975dbe85cfee42e1acf76bb11fcc64ddf2`, zero behind and nine runtime commits ahead of published
`b503f05053daf902649924e73c62d7cd0f44e665`. It stopped as
`A1.1d-5R2-2 cross-document change required`. No integration runtime commit, closeout documentation
commit, runtime push, seam promotion, or R2-3/R2-4/R3 implementation followed Route D.

Routes A–D remain individually review-clean and preserved. Their current pre-E replay commits and
ordinary/full-index binary patch SHA-256 pairs are below. The named preservation branches retain
patch-equivalent reviewed commits; Route D's replay preservation points at the listed commit
exactly.

| Route | Current pre-E replay commit | Ordinary patch | Full-index binary patch | Equivalent preservation branch |
|---|---|---|---|---|
| A | `3bf59b30e4c7348b8ff6315e3eb3658d74af2552` | `1d04e54cabc705b2070b9d104e779278b79723602c24488246b310c1fa32fe7d` | `649004caed540970b72d265c44b9f05b1c0370175bd1c83aaa1c86d8443e78b9` | `feat/preserve-a1-1d-5r2-2-route-a-successor-1d04e54c` |
| B | `1b5219d5c7471f492865ab55e4efc0d1ab0cac49` | `28e6b9da35f96b5f93c49369cbde0eda77e9a145b54f9c412bae8e5a83871674` | `671d7f35d1d988b920863198f45cd94be0c2eb53fcd2ef9d94357e68f2d365e1` | `feat/preserve-a1-1d-5r2-2-route-b-security-reviewed-28e6b9da` |
| C | `0290b829ebaf222fcdc942678178a5e4545de62c` | `7c5a908a6760244d9ad6922dfcf7e944cfce3c723d994e279a3aeaded7a8ffa0` | `5b436d5dfeaf6e513cbaa306de65848cbe3d5b003fa8c00589be09e54b630277` | `feat/preserve-a1-1d-5r2-2-route-c-final-5b436d5d` |
| D | `6452d3a0650df4075c3ef720bad37920a8e5859d` | `278a679060d79dbf6c2728fda65122e9bb5b4b5d92384f8c1ea69dac57a8241b` | `7be7a562b983d42c57466818428df98e53f1e36eb38e985653ee4e5eea578f97` | `feat/preserve-a1-1d-5r2-2-route-d-replayed-6452d3a0` |

The stopped tree also contained four reviewed comment-only ShellCheck remediations: eleven inserted
`# shellcheck disable=SC2034` comments, no deletion or semantic token/command/variable/control-flow/
test/behavior change, and identical ordinary/binary patch SHA-256
`0cb0ef48c76336fdab70582ae1200568916febfb0e9e26fd0188cd79854d33c6`. They are preserved outside
this correction at commit `90ff6056195207598a2d77358f5206b1d5385700` on remote branch
`feat/preserve-a1-1d-5r2-2-shellcheck-comments-0cb0ef48`, rooted at exact Route D, across only
`scripts/substrate/{uninstall-substrate.sh,uninstall.sh}` and
`tests/installers/{prefix_propagation_r2_2.sh,world_provision_smoke.sh}`. They remain renewed-closeout
WIP and are not part of R2-2E, R2-2F, or this docs patch.

At that historical closeout, the blockers were exact:

1. PI-111: `world_gateway.rs` still used ambient config, broker policy, network-policy config,
   runtime-family inventory, Codex home, routing-disable toggles, and platform client selection.
2. PI-106/PI-107: ordinary current/global/workspace world-deps, runtime probe/install/sync,
   provision-deps, and post-provision sync still dropped A or called an ambient request builder.
3. Doctor truth: World/Host world-fs policy and Health/shim fixture/child composition could combine
   an A label with ambient B config/policy/inventory/dependency inputs or missing identity evidence.

Source closure also fixes the platform posture. R2-2E's Linux Gateway path uses the fixed
`/run/substrate.sock`. No authenticated macOS platform-endpoint source exists inside E, so the
macOS route must reject before client construction or `auto_select`; Windows and other contextless
Gateway cfgs reject before any authority-bearing selection. The production-compiled
`resolve_macos_gateway_client_endpoint` helper and the other macOS builders/resolvers are source-
closure-inspected but frozen R2-3 compatibility; they cannot be edited, deleted, called, or used to
select an endpoint in the authenticated route. R2-2F's
authenticated world-deps and truthful-doctor guarantee is Unix/Linux only. macOS, Windows,
fallback, and other non-Unix adapters remain R2-3 compatibility, unproven, or unavailable/fail-
closed, and cannot report authenticated A from E or F.

The unavailable `$feature-seam-extractor` was replaced by read-only GitNexus query/context/impact,
manual caller/cfg/source closure, and two fresh isolated read-only reviews. Initial reviewers
`/root/r2_2e_source_review` and `/root/r2_2f_source_review` found the disabled-route/Linux/macOS
gateway boundary, workspace/runtime/provision request chain, and doctor identity omissions. Fresh
rereviewers `/root/r2_2e_source_rereview` and `/root/r2_2f_source_rereview` returned CLEAN after the
exact corrections recorded in `03`/`04`; neither modified repository state. The F audit records the
shared ambient `build_agent_client_and_request` as HIGH and its trace-metadata variant as CRITICAL;
both are frozen. The earlier HIGH result for `resolve_current_inventory_view` is likewise a frozen
reuse boundary. Counts are diagnostic; the exact symbol/caller/cfg tables are binding.

R2-2E's PI-111 implementation/proof clause and F0/F0a/F0b/F0-HC are now complete. R2-2F owns the
unresolved PI-106/PI-107 production paths. The required order is Routes A–E -> F0/F0a/F0b/F0-HC
complete -> R2-2F -> renewed R2-2 integration closeout -> R2-3 -> R2-4 -> R3. The
renewed closeout is production-fix-free and reruns
the entire Routes A–F wall. `RG-HOME-01` and `RG-INSTALL-01` remain open. No privileged, macOS, or
Windows proof is claimed, and no seam is promoted.

### A1.1d-5R2-2E authenticated gateway closeout

R2-2E is **implemented, proof-complete, review-clean, committed, and preserved**. Its exact original
runtime identity is:

| Evidence | Exact value |
|---|---|
| Commit | `7e8e83802885c0ece93efcaacccc26503eeb6715` |
| Tree | `02a1a2f6b7e4be47ed9ec38537c805ae348c96b6` |
| Ordinary patch SHA-256 | `fb7b65b02cdac46857750b64ebd5ceab651e8e99910c05240b187c3e729444ff` |
| Full-index patch SHA-256 | `c712f467ac92efb0de7b524272479741ac6b318201cf45dbd994116ec0e8b862` |
| Preservation branch | `feat/preserve-a1-1d-5r2-2e-c712f467` |
| Exact manifest | `crates/shell/src/execution/platform/mod.rs`; `crates/shell/src/builtins/world_gateway.rs`; `crates/shell/src/execution/agent_inventory.rs`; `crates/shell/src/execution/policy_snapshot.rs`; `crates/shell/tests/world_gateway.rs` |

The owner/symbol delta is confined to the `handle_world_command` Gateway binding; the existing
gateway request-context, validation, action, authentication, and client-selection chain; additive
explicit-bootstrap-home inventory and network projection; and focused tests. Only
`synthesized_unavailable_response_without_context` was deleted. Routes A–D are patch-identical;
frozen direct-member surfaces, the managed-gateway secure-FD producer and receiver, its bundle
schema, and its lifecycle are blob-identical to E's parent. Canonical config/policy resolver
ownership remains unchanged.

The resulting path is `handle_world_command` -> authenticated carrier validation -> trusted A
root/principal binding -> canonical explicit config -> canonical effective policy -> canonical
policy snapshot/network policy -> explicit inventory -> request construction -> fixed Linux gateway
client. Config, effective policy, network policy, inventory, and committed account-home auth
provenance use explicit A. Ambient HOME, XDG, CWD, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`, `CODEX_HOME`,
`.codex`, `config.toml`, world toggles, and socket overrides cannot become authority; CWD remains
explicit workspace scope only. Linux remains fixed to `/run/substrate.sock`; network allow/deny
meaning, credential transport, gateway lifecycle, user/world capability, and policy meaning are
unchanged. Compatibility is not promoted.

Focused proof is exact: world-gateway classification 13/13; config resolution 21/21; effective
policy 14/14; policy snapshot/network 10/10; inventory 17/17; install-bootstrap context 8/8;
explicit `HostSessionAuthority` composition 1/1; new integration negatives 3/3; managed auth bundle
7/7; world-service gateway runtime 32/32; gateway receiver/server 18/18.

The baseline terms remain distinct:

1. **R2-2 historical starting baseline:** `1089 passed / 149 failed`.
2. **Clean Route D comparison baseline:** `1101 passed / 149 failed / 0 ignored`.
3. **Genuine post-E pre-F0 success observation:** `1114 passed / 149 failed / 0 ignored`.
4. **Known post-E interference observation:** `1113 passed / 150 failed / 0 ignored`.

The third value is reproducible but was not deterministic before F0 and therefore never became the
F comparison baseline. The later completed F0/F0a/F0b/F0-HC closeout establishes the deterministic
`1280/1235/45/0` F baseline without deleting or rewriting either post-E observation or the later
`1118/150` versus `1119/149` F0 evidence.

The Route D-to-E differential has `PassToFail=0`, `NewFail=0`,
`FailToChangedFailure=0`, and `FailToPass=0`, with the identical 149 failure-name set. After
normalization, only the recorded nondeterministic orchestration identifiers differ. No test was
removed, renamed, substituted, weakened, or newly ignored. The non-reproducible parallel
retained-member-stream failure is retained only as a historical observation; it is counted as
neither success nor regression.

GitNexus final detection is semantically contained to the approved owners and existing process
families: aggregate CRITICAL adjacency is diagnostic over-attribution, while exact edited existing
symbols were LOW and no new resolver owner, schema, capability, or execution-process family was
introduced. Final isolated read-only reviews `e_final_gateway_authority`,
`e_final_policy_network`, `e_final_credential_boundary`, and
`e_final_platform_regression_replacement` are CLEAN. The original platform reviewer is excluded
because it violated the required read-only process.

Linux fixed-socket and regression proof is recorded. macOS fails before client construction or
ambient forwarding; Windows/other entries fail before ambient authority selection; non-Unix static
cfg preservation is recorded. None is a native macOS/Windows product claim, and privileged product
proof remains R2-4-owned.

The managed-gateway secure-FD path is **landed, regression-proven, and unchanged by R2-2E**.
Direct-member Codex/UAA gateway adoption is separately **unresolved, transitional compatibility,
non-promotable, and owned by E3/D1/D3**. Accordingly `RG-CONFIG-02`, `RG-CONFIG-04`, `RG-UAA-02`,
and `RG-UAA-03` remain open and unchanged. PI-111's R2-2E clause is complete, but the full gateway
credential/config architecture is not.

### A1.1d-5R2-2F0 deterministic world-socket test isolation authorization

The F preflight baseline investigation is classified `TestIsolationDefectConfirmed`. The exact
minimal pair is `continue_world_worker_classifies_real_retained_member_turn_streams` plus
`b21_retained_production_handoff_claims_before_next_frame`: the target failed 12/20 parallel
two-thread runs and 0/10 serial runs, while fixed serial order passed. Both fixtures mutate the
process-global `SUBSTRATE_WORLD_SOCKET`; the first holds `world_env_guard` but is unannotated, while
the B2.1 competitor is `#[serial]` but uses an independent local environment guard. Because
`#[serial]` does not exclude unannotated tests, the target can observe the peer's private, restored,
or deleted socket. The competing mutation first appears at
`c519024bd91b6ca6e332d0b8881f7d13ded940e0`, before Routes A–E.

The complete shell library-test-process inventory contains 66 mutating call sites. Sixty-one require
migration: 49 orchestrator local-guard sites, two orchestrator manual restoration blocks, four
macOS platform closure-helper sites, two persistent-session direct sites, two routing direct sites,
one world-enable path manual site, and one world-gateway classification-test closure call. Four
other world-gateway RAII sites and one async-REPL site already acquire the same shared lock and
restore exact `OsString`/absence during unwinding. Shell integration tests only
use child `Command::env`/`env_remove` in separate test processes and require no cross-process lock.
Production socket readers remain frozen.

F0 is authorized only for test-compiled hunks in `crates/shell/src/execution/mod.rs`,
`crates/shell/src/execution/orchestrator_world_dispatch.rs`,
`crates/shell/src/execution/platform/macos.rs`,
`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`,
`crates/shell/src/execution/routing/world.rs`, and
`crates/shell/src/builtins/world_enable/runner/paths.rs`, plus the classification-test helper/call in
`crates/shell/src/builtins/world_gateway.rs`. One RAII guard must acquire the existing
reentrant lock, capture exact prior state, hold it through socket/server/helper lifecycle and
cleanup, restore on return or panic, and release only afterward. Same-thread nesting restores in
stack order. `parking_lot::ReentrantMutex` does not poison; panic recovery and later acquisition are
mandatory proof. `#[serial]` alone, sleeps, retries, global single-thread forcing, assertion
weakening, readiness changes, and production edits are forbidden.

Required proof is prior-value/absence restoration, non-Unicode prior value on Unix, panic and
non-poisoning recovery, nesting, concurrent blocking/cleanup ordering, no peer-socket observation,
no leaks/deadlocks, at least 100 parallel and 20 serial exact-pair iterations, neighboring mutator
combinations, three default-parallel broad walls, one serial broad wall, stable inherited names and
normalized signatures, and unchanged deliberate retained-registration loser/parent behavior.
Routes A–E, production retained-registration, the managed secure-FD path, capabilities, policies,
and user behavior are unaffected.

At this authorization checkpoint, the exact combined harness candidate was preserved and
unrestored. It has since passed identity revalidation, corrected differential proof, fresh
containment review, canonical closeout, and runtime preservation, recording the deterministic
comparison baseline. At that historical checkpoint R2-2F was the next packet and renewed R2-2
integration closeout remained after F; the completed-F ledger below supersedes that status. R2-3,
R2-4, and R3 remained unstarted; R2-2 remained incomplete; no seam was promoted.

### A1.1d-5R2-2F0a — SUBSTRATE_HOME test isolation

F0's exact uncommitted candidate was preserved before diagnosis and removed from the source
worktree. Preservation ref `feat/preserve-a1-1d-5r2-2f0-blocked-candidate-69fe8c08` points to commit
`32513cf41f98006020702f1eca8d6931bc6708ce`, parent
`0b9f8474a414d3ed24aff9603e9ff68c37569bd5`, tree
`fa920cc04e62d379a6f999485cf93e4b436b44ec`, ordinary patch SHA-256
`6fce969756c371ccd6b20b5c6c35e9159100939c484f07dd7ecb1c66b573ebc0`, and full-index/binary
patch SHA-256 `69fe8c08d47787294b7f43ddad56bddf6cfd3d2d5ce99dce539e01b539925888`.
Its exact manifest remains `crates/shell/src/builtins/world_enable/runner/paths.rs`,
`crates/shell/src/builtins/world_gateway.rs`, `crates/shell/src/execution/mod.rs`,
`crates/shell/src/execution/orchestrator_world_dispatch.rs`,
`crates/shell/src/execution/platform/macos.rs`,
`crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`, and
`crates/shell/src/execution/routing/world.rs`. It is evidence only: no F0 runtime commit or closeout
exists on the source branch.

Focused F0 proof remains valid: the exact socket pair passed 100/100 parallel and 20/20 serial
runs; the four-test socket-neighbor matrix passed 20/20; prior absence, non-Unicode prior value,
panic restoration/reacquisition, nested stack restoration, concurrent exclusion/restoration, and
the deliberate retained-registration conflicting-child parent all passed. Final broad proof did
not close: the two exact candidate walls were `1118 passed / 150 failed` and `1119 passed / 149
failed`. The extra failure was
`execution::agent_runtime::tool_invocation_contract::tests::dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`,
with normalized panic `tool_invocation_contract.rs:3554: resolve exact B-owned acceptance authority:
open activated versioned authority layout`; it passes in isolation.

Read-only same-process reduction proves the selected minimal HOME pair:

1. target: `dispatch_contract_adapter_active_task_resolution_requires_supervisor_claim`;
2. competitor: `prompt_submit_continuity_prefers_persisted_session_contract`.

The canonical shell-library test binary ran both exact filters with `--test-threads=2`: target
failure 20/20, always the same authority-layout signature. The stable same-process
`--test-threads=1` harness controlled competitor-then-target and failed 0/10. Separate-process
sequential competitor-then-target and target-then-competitor executions pass. Stable libtest could
not force reverse same-process order, so this investigation claims no reverse same-process result.
Three other unannotated mutators—the second control continuity test and both agents-command
toolbox-status tests—each independently failed the parallel pair 20/20. Non-source transition
tracing recorded separate thread identities and exact ordering: competitor sets its private test
HOME; target sets a distinct owner-only test HOME; competitor removes HOME; target fails while
opening its expected activated authority layout. Separate root device/inode identities prove this
is not one reused directory. No product state was read, changed, or cleaned.

Commit `f5a150f94d585b1f55ec0067845cd5d715773c78` first adds the selected unannotated competitor and
its manual `with_store` HOME mutation. The target and its HOME-mutating fixture appear later at
`83101dcbcc750e6e8fb8979bea19f1f777792188`, which is therefore the first source commit where the
exact pair coexists and the first source-proven pair-causal commit. A same-commit execution replay
was attempted in an isolated worktree but
stopped before linking because the host `/tmp` tmpfs had insufficient free space; only that exact
test-owned worktree/build tree was removed. The causal claim is source-history-proven, not presented
as an executed first-red-commit result.

The complete shell-library mutation inventory contains 435 mutating test functions across 19 files.
Five `with_store` families account for 223 dependent tests: auto-attach 10, control 11, state-store
183, tool-contract 12, and host-inbox 7. Two agents-command helpers account for five more. The
remaining direct/local-guard/manual owners are shim-doctor 1, world-deps 2, manager-env 1,
world-gateway 11, agent-inventory 6, direct state-store tests 2, direct tool-contract tests 10,
HostSessionAuthority store tests 1, config-model 11, environment scripts 1, invocation 2,
orchestrator dispatch 106, routing builtin tests 4, settings 8, and async REPL 41, with every test
function counted once in the 435 total.
All recognized direct owners are `#[serial]`; the only unannotated mutating callers are the four
named above. Existing restoration varies: some local guards preserve exact `OsString`, others store
UTF-8 `String`, and manual set/remove helpers are not panic-safe. `#[serial]` cannot exclude the
four unannotated mutators.

Integration occurrences are process-separated. Parent HOME mutation is confined to
`crates/shell/tests/shim_deployment.rs` (nine serialized callers),
`crates/shell/tests/agent_successor_contract_ahcsitc0.rs` (one caller), and
`crates/shell/tests/support/mod.rs` (one ignored one-test binary plus one serialized caller). Other
integration occurrences are read-only or configure spawned children with `Command::env`/
`env_remove`. They require intentional per-binary disposition, not a cross-process lock. The
non-Unix production `world_enable` HOME assignment remains product code and is frozen; tests must
hold their test boundary outside it.

Lock-topology source closure selects **Option A: one process-global authority-environment lock**.
At least 88 shell-library tests depend jointly on HOME and world socket: world-gateway 5,
agents-command 2, invocation 2, orchestrator dispatch 39, and async REPL 40. Existing helper paths
already acquire HOME then socket in some modules and socket then HOME in others. Separate locks
would therefore admit mixed authority snapshots and lock-order inversion. The combined guard must
capture exact `OsString`/absence, support safe same-thread nesting with stack-order restoration,
make poison/non-poison recovery explicit, hold through dependent async/process lifetime and cleanup,
and restore before unlock. `#[serial]` remains supplemental only.

Async cleanup source closure expands the known six locations to nine exact F0-migrated socket
fixtures in `crates/shell/src/execution/orchestrator_world_dispatch.rs`:

1. `dispatch_contract_cancel_world_work_ephemeral_routes_exact_active_task_over_execute_cancel`;
2. `active_ephemeral_terminal_wait_allows_multiple_waiters_to_observe_same_terminal_truth`;
3. `active_ephemeral_terminal_wait_registration_guard_releases_non_happy_path_registrations`;
4. `active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start`;
5. `dispatch_contract_cancel_world_work_ephemeral_retry_reuses_shared_terminal_truth`;
6. `dispatch_contract_cancel_world_work_ephemeral_fails_closed_when_execute_cancel_is_not_delivered`;
7. `continue_world_worker_classifies_real_retained_member_turn_streams`;
8. `continue_world_worker_dispatch_returns_real_typed_internal_outcome`;
9. `dispatch_contract_fork_world_worker_rolls_back_child_when_lineage_persist_fails`.

Each declares a socket-owning Tokio server before or alongside the environment guard and calls
`server.abort()` without awaiting termination; one merely yields once. The authorized correction is
strictly test-only: abort, await confirmed task termination/cancellation, complete and confirm
fixture-owned socket/task cleanup, restore environment, then release the lock. Reverse declaration/
drop order is not termination proof. The already-awaited server at the earlier compatibility test
and unrelated aborts without a guarded socket lifecycle are source-reviewed exclusions.

GitNexus reports `world_env_guard` CRITICAL at 35 direct/70 total dependents and five affected
process groups; tool-contract `with_store` HIGH at 12 direct dependents; auto-attach/control/
host-inbox helpers MEDIUM at 10/11/7; agents-command helpers LOW at 4/1. State-store and the nine
large-module test symbols are graph-under-resolved, so their complete source caller/body audit is
binding. These breadth labels authorize no production edit.

The isolated documentation-worktree GitNexus refresh changed only generated symbol/relationship
count lines in `AGENTS.md` and `CLAUDE.md`; those exact lines were restored before review and commit.
Record: `GeneratedIndexDriftRemediated`.

The exact combined test-only allowlist is the preserved seven-file F0 manifest plus the HOME-only
shell-library files named in `03`/`04` and all three inventoried parent-mutating integration-test
families in `crates/shell/tests/{shim_deployment.rs,agent_successor_contract_ahcsitc0.rs,support/mod.rs}`.
No
other integration file, production symbol, resolver, socket owner, readiness path, retained-worker
or retained-registration validator, managed secure-FD path, capability, policy, or user behavior
may change. F0a and the F0 cleanup correction are not implemented by this authorization.

Combined F0/F0a/F0b closeout requires the F0 socket pair and F0a HOME pair at 100 parallel/20 serial minimum,
a mixed HOME/socket neighbor matrix, exact prior absence/value/non-Unicode restoration, panic,
poison/recovery, nesting, bounded subprocess inheritance, termination-before-restoration, and zero
leak proof. Three exact final-candidate default-parallel broad walls plus one serial wall must have
one stable inherited failure-name/signature set and no unexplained count variance. Historical
`1114/149`, `1113/150`, `1118/150`, and `1119/149` observations remain distinct evidence and are not
lowered, erased, or substituted for the deterministic F baseline.

`TestIsolationDefectConfirmed` remains the historical classification. The exact combined candidate
later passed restoration and identity revalidation, corrected differential proof, fresh containment
review, canonical closeout, and runtime preservation. At that historical F0a checkpoint F and the
later nodes remained unstarted and no seam was promoted; the completed-F ledger below supersedes
that packet status.

### A1.1d-5R2-2F0b — deterministic renderer-output test isolation

The post-fork-remediation F0/F0a candidate was preserved before this read-only source-closure and
removed from the source worktree. Preservation ref
`feat/preserve-a1-1d-5r2-2f0-f0a-post-fork-remediation-9e012c68` points locally and remotely to
commit `07ce368cc02d5a4e80db9c1ca80efee93009d064`, exact parent
`d52ad6de2db7f5ed7fa0ee68ff2e3fc98332021c`, tree
`51737000a17954a605f091f00f9b0c71f38a596c`, ordinary patch SHA-256
`5530e58a7289db4e9a62fc9934510b87add71853728a0311af2160328af3cbeb`, and full-index/binary
patch SHA-256 `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`.
Its exact 27-file manifest is
`crates/shell/src/builtins/{shim_doctor/report.rs,world_deps/mod.rs,world_enable/runner/manager_env.rs,world_enable/runner/paths.rs,world_gateway.rs}`,
`crates/shell/src/execution/{agent_inventory.rs,agents_cmd.rs,config_model.rs,env_scripts.rs,host_inbox_materialization.rs,invocation/tests.rs,mod.rs,orchestrator_world_dispatch.rs,platform/macos.rs,routing/builtin/tests.rs,routing/dispatch/world_persistent_session.rs,routing/world.rs,settings/tests.rs}`,
`crates/shell/src/execution/agent_runtime/{auto_attach.rs,control.rs,host_session_authority/store_tests.rs,state_store.rs,tool_invocation_contract.rs}`,
`crates/shell/src/repl/async_repl.rs`, and
`crates/shell/tests/{agent_successor_contract_ahcsitc0.rs,shim_deployment.rs,support/mod.rs}`.
The preservation commit records the per-file SHA-256 fingerprints.

The original pre-remediation candidate remains preserved locally and remotely at
`feat/preserve-a1-1d-5r2-2f0-f0a-broad-blocker-9978655e`, commit
`78353f3383fdd80c176fe5638328c10f2805dcf0`, exact parent
`d52ad6de2db7f5ed7fa0ee68ff2e3fc98332021c`, tree
`55476fa63a6ac2d116758a0aad5476ffea655a3f`, ordinary patch SHA-256
`577a34da903992d0566fa51d164410caeca98592dd486f5c1d5520e34baaa5a1`, and full-index/binary
patch SHA-256 `9978655e0f4095880c49d863a93b8fc99fb2b769afd42554c758b3970103f710`.
Both refs are evidence only. Neither candidate is committed to or restored on the source branch,
and no F0b runtime implementation begins in this planning packet.

The mandatory post-fork broad wall was not deterministic:

| Run | Passed | Failed | Ignored | Exact distinguishing result |
|---|---:|---:|---:|---|
| Parallel wall 1 | 1134 | 146 | 0 | inherited failure set |
| Parallel wall 2 | 1134 | 146 | 0 | identical to wall 1 |
| Parallel wall 3 | 1133 | 147 | 0 | added `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails` |

The third wall invalidates canonical closeout even though the first two agree. Its complete
captured output was
`".[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n"`. Read-only reduction proves
that `capture_stdout_once` creates a pipe, duplicates stdout, replaces process fd 1 with `dup2`,
runs the closure, flushes, restores fd 1, and reads the pipe. Libtest's parallel reporter writes its
progress `.` through the same process-global descriptor during that interval. The current test
then searches for a line beginning with `[codex]`; the reporter prefix makes the only line begin
with `.`. `#[serial]` cannot correct this boundary because it serializes only enrolled tests,
not libtest's reporter.

The causal matrix is exact:

| Control | Result | Interpretation |
|---|---|---|
| Forced same-process parallel | 376 passed / 124 failed across 500 | Every failure had the broad-wall name and normalized signature |
| Isolated target | 100/100 passed | Renderer behavior itself is stable |
| Same-process serial, identical neighbors | 100/100 passed | Removing descriptor overlap removes the failure |
| Separate-process control | 100/100 passed | Process isolation removes shared-fd contamination |
| Parallel pretty reporter | 99/100 passed | Reporter scheduling can enter the fd-capture interval |

Candidate introduction is unnecessary: the target and capture helper are byte-identical to clean
E. This establishes `TestIsolationDefectConfirmed`, not product regression and not random
flakiness.

Read-only source closure is complete for the renderer boundary:

- private `#[cfg(unix)] PublicPromptRenderer` owns `new` and `render`;
- exact production construction/call paths are
  `run_hidden_owner_helper_startup_prompt_stream_with_projection` and
  `run_public_prompt_command`; each calls `render` for normal envelopes and a failure envelope;
- `capture_stdout_once` has exactly one caller,
  `public_prompt_renderer_renders_bounded_structured_fallback_when_decode_fails`;
- `capture_stderr_once` has exactly one caller,
  `public_prompt_renderer_renders_bounded_structured_stderr_fallback_when_decode_fails`;
- `capture_stderr_once` performs the same process-global replacement on fd 2 and therefore has
  the same structural race even though it did not fail in the recorded wall;
- no other raw `dup`/`dup2` capture helper exists in this renderer/test ownership boundary.
  Unrelated raw-fd operations elsewhere are deferred evidence and do not widen F0b.

Current production behavior is frozen exactly. JSON envelopes serialize to stdout and propagate
serialization/write failure with the existing context; Accepted emits nothing; Completed emits to
stdout; Warning and Failed emit to stderr; Event selects stderr only for the exact stderr kind and
otherwise stdout; decoded lines and bounded structured fallback retain their exact bytes and
newlines. Existing completed/event/warning/failure write-error suppression and all existing flush
result treatment remain unchanged. Stream selection and locking remain lazy and ordered as today;
an implementation must not eagerly lock both streams. Redaction and bounded fallback remain
unchanged.

GitNexus and source closure give this containment:

| Symbol | GitNexus upstream impact | Source-closure disposition |
|---|---|---|
| `PublicPromptRenderer` | LOW; 0 direct, 0 process, 0 module | Private Unix production type; retained |
| `PublicPromptRenderer::render` | MEDIUM; 4 direct, 39 total; one `handle_agent_command` process family; Agent_runtime direct, Execution indirect | Only existing production symbol permitted a mechanical delegation edit |
| `PublicPromptRenderer::new` | HIGH; 19 direct, 43 total; two process labels; three modules | Generic-`new` graph over-attribution; exact source has four construction sites; signature and body frozen |
| `capture_stdout_once` | LOW; one direct test caller; zero process | Delete after stdout test migration |
| `capture_stderr_once` | LOW; one direct test caller; zero process | Delete after same-owner stderr test migration |
| two exact fallback tests | LOW; zero process | Migrate only capture mechanism; names and behavioral assertions retained and strengthened to complete bytes |

There is no CRITICAL impact. The proposed one-file allowlist is exact:
`crates/shell/src/execution/agent_runtime/control.rs`. It may EDIT
`PublicPromptRenderer::render` only for delegation; ADD a private Unix-only explicit-writer
rendering core, private output-sink adapter, or equivalent bounded internal symbols; DELETE
`capture_stdout_once` and `capture_stderr_once`; and EDIT only the two exact fallback tests named
above. `PublicPromptRenderer` remains private, `PublicPromptRenderer::new` and both production
caller bodies are frozen, and no other file, symbol, dependency, or test is authorized.

The future tests provide their own in-memory stdout and stderr writers and assert the exact complete
selected bytes and exact empty nonselected buffer. They may not strip, search around, or tolerate
an unrelated prefix. Production `render` remains the sole entry point and delegates through the
default real streams with byte-for-byte equivalent bytes, stream choice, order, newline, flush,
errors, redaction, and fallback. A public API, output transport/schema, process-global output lock,
writer registry, side table, environment-selected sink, reporter filtering/suppression, sleeps,
retries, larger timeouts, test-thread reduction, whole-suite serialization, `#[ignore]`, test
removal/rename/substitution, or assertion weakening is forbidden. `#[serial]` may remain
supplemental only.

F0b is test-isolation infrastructure, not a product rendering change. It changes no production
execution, user-visible behavior, world, policy, credential, secure-FD, gateway, receipt,
supervisor, retained-worker, placement, caging, lifecycle, or capability semantic and promotes no
seam. F0/F0a/F0b/F0-HC are complete: the preserved candidate was restored and identity-checked,
the corrected transition audit and fresh containment review are CLEAN, and canonical closeout and
runtime preservation are complete. At that historical F0b checkpoint F and the later nodes remained
unstarted. The binding sequence was Routes A–E → F0/F0a/F0b/F0-HC complete → F → renewed R2-2
integration closeout → R2-3 → R2-4 → R3. The completed-F ledger below supersedes that packet
status.

For the Route A successor clause of `R2-RUNTIME-01`, the authorized Unix-only import cfg set also
includes the policy test module's sole `tempfile::TempDir` import; no other import or module gate is
authorized. Health proof must combine fail-closed syscall/path interception with a complete B-tree
before/after snapshot. Host proof must show the expected A dependency scaffold, no B dependency
artifact, and complete B-tree preservation across both the installed-witness dispatch and the
direct-repository no-witness rejection. This test-only clarification supersedes the narrower import
wording in the row without changing any production byte, Route B boundary, or R2-3 ownership.
The final portable-parent paragraph below extends that gate's sole successor list only with the two
named existing test setup hunks; it supersedes the row's earlier closed four-category enumeration.

The final Route A test-parent remediation is bounded against preserved successor patch SHA-256
`0a53e8b60326e21b4237392bfa99671fa8c19c0cd45eb4c573cae5140e808720` at commit
`ad139789ef317b978e96c36ee8170dd503bff8a4`. Only
`config_show::config_current_show_uses_declared_prefix_under_conflicting_ambient_home` and
`policy_discovery::policy_current_show_uses_declared_prefix_under_conflicting_ambient_home` may
replace their Linux-only `/run/user/<effective uid>` test-parent fallback. Each uses nonempty
`XDG_RUNTIME_DIR`; otherwise it resolves the current account's nonempty home and uses `.cache` or a
repository-established fixture subdirectory beneath that same account home. This is not a third
source, and the command's ambient/conflicting HOME cannot select it. Each creates that parent as
required and retains a unique `0700` selected-A root. Missing sources fail setup explicitly; `/tmp`, `/var/tmp`, `/run/user/<uid>`, CWD,
ambient `SUBSTRATE_HOME`, and B cannot substitute. Both pre-edit impacts are LOW with zero callers,
processes, or modules. The manifest remains exactly thirteen files because both tests are already
present; every production fingerprint and every other test fingerprint remains fixed. The final hash
is established after remediation, with any other hunk or a fourteenth file rejected as
`SuccessorPatchScopeMismatch`. Native Darwin/Windows proof remains unavailable when the Apple SDK or
MSVC tooling is missing and is recorded as unavailable rather than success or product regression.

R2-0 cross-document checkpoint rules are: inventory counts and owner/packet columns must agree;
every R2 packet has an exact production/test allowlist; R2-4 has no production allowlist; every R3
action above stays exclusive to R3; host and Lima/WSL paths/principals are never equated;
`InstallBootstrapContextV1` commitment framing is canonical and unambiguous; and the serial graph is
acyclic. At that historical R2-0 checkpoint, the next packet after a clean R2-0 commit was
**A1.1d-5R2-1 — Host context construction and Unix dev propagation**.

**Frozen planning validation record:** the inventory contains exactly 118 unique, contiguous rows
and dispositions: 19 owned by R2-1, 41 by R2-2, 35 by R2-3, one by R2-4, and 22 by R3. Each row
has exactly one approved edge class and one packet owner; the class totals are 16
`ContextConstruction`, 26 `ExplicitHostPropagation`, five `PrivilegeBoundaryPropagation`, 17
`PlatformMapping`, 13 `GeneratedProjectionConsumption`, 15 `DiagnosticProjection`, 22
`R3CleanupOnly`, and four `OutOfScope`. The R2-0-era frozen DAG was historically
R1 -> R2-0 -> R2-1 -> R2-2 -> R2-3 -> R2-4 -> R3. The corrected canonical DAG replaces the
R2-2 outgoing edge without changing inventory ownership; F0/F0a/F0b and F0-HC are the completed
proof prerequisites: R1 -> R2-0 -> R2-1 -> R2-2 Routes A-E -> F0/F0a/F0b/F0-HC complete ->
R2-2F -> renewed
R2-2 integration closeout -> R2-3 -> R2-4 -> R3. Mechanical validation covers row-ID,
field-count, class, owner, packet-count, table-column, fence-pair, relative-link, and allowlist-path
checks across 43 tables/749 pipe rows, 134 fence markers, and 12 relative links, plus
`git diff --check`, `cargo fmt --all -- --check`, and GitNexus change detection. The
GitNexus index was refreshed at the starting commit; only generated count lines changed during
refresh and those exact lines were restored, while final change detection reports six documentation
files, zero affected execution flows, and LOW risk. Successive isolated read-only review rounds found and
closed missing CLI pre-bootstrap ordering, automatic shim/repair paths, dev sudo and provisioning
children, shim/release/Windows/Lima cleanup ownership, shim-telemetry and replay factory callers,
two-stage Lima realization, Windows timeout-kill ownership, invalid native-smoke assignments,
symlink-shim invocation recovery, generated preexec and Codex-home projections, direct
world-deps/doctor/config/policy/gateway consumers, exact helper-versus-system-tool sudo carriers,
Lima known-hosts placement, and forwarding unlink/drop/timeout ownership.
The post-push fresh authority audit additionally closed source-layout omissions for the Unix dev
A/bin symlink and Windows release A/bin copy, and made current Unix account+UID/Windows account+SID
equality mandatory for every internal carrier, including forged-valid-carrier negative proof.

The R2-1 preimplementation trigger correction preserves that inventory and DAG. It authorizes the
hidden authenticated `--install-bootstrap-home-v1` action only inside the existing R2-1 production
files and adds only `crates/shell/tests/world_deps_home_scaffold_wdh3.rs` to the focused test
allowlist. That file may construct the shared IH carrier/current Unix principal, replace historical
`--version` bootstrap triggers, add version nonmutation proof, and perform the exact one-to-one rename
`test_bootstrap_scaffolds_deps_on_version` ->
`test_install_bootstrap_action_scaffolds_deps`. This is a canonical trigger rename, not removal or
substitution: the old/new fixture and scaffold assertions are identical and only the command trigger
changes; all other historical test names and all R1 security assertions remain unchanged. No
world-deps production symbol, cleanup/R3 behavior, user-facing feature, or capability is authorized.
The subsequent lifecycle allowlist review closed the remaining Windows implementation-surface gap:
R2-3 alone may add the exact `windows-sys` 0.52 features `Win32_Security`,
`Win32_Storage_FileSystem`, `Win32_System_Threading`, `Win32_System_Com`, and `Win32_UI_Shell` in
`crates/shell/Cargo.toml`; the last two are only for token-bound Known Folder resolution. Dependency,
version, target-scope, extra-feature, package-graph, and lockfile changes remain outside that authority.
The next fresh source audit corrected PI-030 to model the ACL bridge's actual mode/target/group-only
boundary and froze its three accepted tuples, then added PI-115 for the forwarder-to-WSL spawn and
PI-116 for physical-shim manager-manifest consumption. Those rows require PM-derived WSL child
argv/environment and A-scoped generated manager data; neither ambient target/`WSLENV` nor ambient
home/repository manifest fallback is authority.
The same review froze the only dependency edits: R2-1 adds existing `base64` 0.22/workspace `sha2`
to `transport-api-types`; R2-3 adds the existing `transport-api-types` 0.2.8 path edge to the
backend factory, forwarder, and physical shim. Only those four package dependency lists may change in
`Cargo.lock`; the shim list may additionally gain existing `windows-sys 0.52.0`, while its existing
`nix` dependency adds feature `user` without a lock hunk. No package/version/checksum change is
authorized. The physical shim's exact `context.rs` witness functions are the only added
OS-observation owner; they construct the shared IH and may not define a local principal/context
type or raw-FFI/ambient fallback.
The transport-identity review first froze SSH UDS as the only V1 normal-product target. A later
lifecycle review correctly found that activating it in R2 would newly reach the current
constructor's socket unlink, `StreamLocalBindUnlink`, timeout kill/wait, and handle-drop cleanup.
The corrected sequence therefore freezes PM and the A-scoped target in R2-3 but stops before
forwarder launch; R3 alone activates it after PI-101/PI-113/PI-114 lifecycle ownership lands.
Existing VSock and SSH-TCP code is not removed, but it remains diagnostic/test-only and cannot
satisfy PM, product, or native R2 proof.
The next fresh authority/platform review found two remaining ambient platform-control selectors:
macOS SSH config lookup still honored `LIMA_HOME`, and Windows forwarder config/log paths still
honored `LOCALAPPDATA`. The corrected PM now commits `host_platform_control_root`: Lima derives it
from the committed host principal's account-database home and scrubs/overwrites child
`HOME`/`LIMA_HOME`; WSL derives it from the current-token Known Folder plus the exact canonical
SID+registered-distro+machine-ID+pipe scope digest. Windows config/log projections are A-scoped and
explicit; the shared PID target is control-root-scoped. Ambient conflicts never select either
platform, and no R2 path gains replacement, rollback, removal, termination, or deletion authority.
The final platform allowlist audit also corrected one wording inconsistency: the R2-3 `Cargo.lock`
allowlist now names the three internal `transport-api-types` package-list edges and the already
frozen existing `windows-sys 0.52.0` shim package-list entry, while still forbidding every
package/version/checksum change.
The final authority review found two trace-specific ambient reads that the broad CLI/shim rows had
not made independently reviewable. The later preimplementation impact review then exposed a
CRITICAL shared-setter fan-out spanning shell, physical shim, replay/platform, tests, and unrelated
compatibility callers. The corrected sequence therefore leaves `set_global_trace_context`
signature and behavior unchanged. PI-117 gives R2-1 only an additive/default-preserving explicit
product-bound representation/constructor, shell binding to exact `A/trace.jsonl` plus policy Git A,
and additive `get_policy_git_hash_at`; all unmigrated default callers retain named non-promotable
`LegacyAmbientCompatibility`. PI-118 plus already-frozen replay/platform migration gives R2-3 the
physical-shim binding, compatibility removal/unreachability, and final global unbound-init failure
rule. No caller-identity side table is allowed. R2-1 CRITICAL breadth is authorized only when the
setter/default behavior and shim/replay/platform files and outputs are unchanged, only shell selects
the explicit posture, and final detection finds no new process family. Trace writer, serialization,
flush, rotation, retention, span/replay, environment-hash, and policy semantics remain frozen.
No reviewer modified the repository. Because the built-in reviewer-thread allocator would not
admit new threads, the required fresh isolated read-only `default` review turns used three distinct
existing built-in identities; each re-read the current diff/source rather than carrying forward its
prior verdict. The terminal verdicts are `/root/final_authority_security_retry` — **CLEAN** for
host-context authority/security,
`/root/final_lifecycle_ownership_retry/final_control_authority/final_control_lifecycle_retry` —
**CLEAN** for installer/uninstaller lifecycle and R2-versus-R3 ownership, and
`/root/final_lifecycle_ownership_retry/final_control_authority/final_control_platform_retry` —
**CLEAN** for cross-platform mapping, allowlists, regression assignments, and native-proof honesty.
No runtime implementation or native platform proof can be inferred from this record.

### A1.1d-5R1 Linux effective-authority contract correction

**Decision/status:** Case A is implemented and review-clean for the bounded Linux R1 scope. This is a
security-contract correction from **physically ACL-free** to **no effective other-principal
authority**. R2-0 planning is complete; R2 implementation and R3 remain separately sequenced and
unstarted. No seam is added or promoted.

The bounded primary-source proof is:

1. [`getxattr(2)`](https://man7.org/linux/man-pages/man2/getxattr.2.html) defines `ENODATA` for
   either a nonexistent named attribute or lack of process access. R1 therefore records exact
   `ENODATA` as `NoData`—only “the kernel returned no ACL data”—and never as physical absence.
2. [`acl(5)`](https://man7.org/linux/man-pages/man5/acl.5.html) maps `ACL_MASK` to group-class mode
   bits and applies the mask to named-user, group-object, and named-group access. Linux
   [`fs/posix_acl.c`](https://github.com/torvalds/linux/blob/master/fs/posix_acl.c) performs the same
   mask intersection and mode mapping. Thus, for supported Linux POSIX access ACLs, a named
   principal cannot retain effective write while the descriptor's authoritative group-write mode
   bit is clear; raw write fully removed by the mask is not effective authority.
3. `acl(5)` distinguishes an access ACL, which governs the current object, from a default ACL,
   which initializes a created child's access ACL. A default ACL does not grant access to its
   directory. Exact final-root `0700` prevents other-principal traversal, and exact owner-only
   descendant `0700`/`0600` modes prevent inherited Linux POSIX entries from granting effective
   other-principal authority.
4. Linux [`security/security.c`](https://github.com/torvalds/linux/blob/master/security/security.c)
   mediates POSIX-ACL and xattr reads. `ENOTSUP`, malformed/unsupported bytes or model, and every
   distinguishable non-`ENODATA` read failure therefore remain `Failed` and fail closed.
5. The [XDG Base Directory Specification](https://specifications.freedesktop.org/basedir-spec/latest/)
   continues to govern the existing user-specific placement/default inputs; it supplies no ACL or
   authority attestation. Physical ACL-xattr absence, if ever required, needs a separately approved
   privileged platform-attestation boundary. R1 neither designs nor implies one.

The canonical observation is `Present(bytes) | NoData | Failed(error_class)`. Ancestor access
`Present` is strictly parsed and mask-evaluated: non-writing `--x`/`r-x` is accepted, any effective
write rejects, and all base/mask/tag/order/uniqueness/version/length invariants are mandatory.
Ancestor default `Present` rejects before creation. Final-root access or default `Present` rejects;
qualified `NoData` requires exact owner, directory type, exact `0700`, stable descriptor identity,
no-follow traversal, no replacement, and authoritative safe mode. Existing invalid roots remain
unchanged and no ACL cleanup/repair is permitted.

Diagnostics use at least `PresentAcceptedNoEffectiveWrite` (ancestor access only),
`PresentRejected`, `NoDataAcceptedUnderModeAuthority`, and `FailedOrUnavailable`. They carry the
requested path, actual offending path, path role, ACL kind, reason class, and candidate-creation
state (`yes`, `no`, or `unknown`). They never call `NoData` absent/ACL-free, disclose ACL principals
or authority payloads, imply repair, or attribute an ancestor failure to the final root.

**R1 regression gate:** the authorized two-file runtime change must prove all of the following
without removing, renaming, substituting, or weakening inherited tests:

1. safe ancestor `ENODATA` accepts as `NoData`, never `Absent`;
2. ancestor `ENODATA` with group/world write rejects by mode authority;
3. effective-write ancestor access `Present` rejects;
4. effective `--x` access `Present` accepts;
5. effective `r-x` access `Present` accepts;
6. raw write fully removed by `ACL_MASK` accepts;
7. multiple named entries are mask-evaluated correctly;
8. final-root access `Present` rejects;
9. ancestor default `Present` rejects before candidate creation;
10. final-root default `Present` rejects;
11. non-`ENODATA` retrieval failure rejects;
12. unsupported ACL state rejects;
13. malformed version, length, order, duplicate, missing base/mask, and unsupported tag reject;
14. diagnostics never claim `NoData` proves absence;
15. diagnostics identify the actual offending ancestor and path role;
16. final-root owner and `0700` remain exact under umasks `000`, `022`, `027`, `077`, and `777`;
17. final-root traversal remains no-follow and identity-stable;
18. replacement and identity drift reject;
19. invalid existing roots retain exact metadata and contents;
20. no ACL cleanup or repair occurs;
21. a real `libvirt-qemu:--x` ancestor fixture passes; and
22. world, policy, receipt, supervisor, and retained-runtime behavior remains untouched.

The proof wall additionally requires focused trusted-filesystem and home-bootstrap tests, complete
HostSessionAuthority tests, product-path bootstrap scaffold tests, relevant installer-environment
regressions, formatting, warnings-denied Clippy for touched targets, workspace all-target check,
`git diff --check`, and GitNexus change detection. The shell differential must preserve the
inherited `1054 passed / 149 failed` baseline with `PassToFail = 0`, `NewFail = 0`, `Removed = 0`,
`RenamedOrSubstituted = 0`, unchanged retained failure names/signatures, audited `FailToPass`, and
passing new R1 tests. This does not claim the later privileged Linux installer/product wall, R2
prefix propagation, R3 cleanup/idempotency, native macOS proof, or cross-platform completion.

**R1 recorded result (2026-07-17):** the source began at
`ebbc5d5649be5ac0c006aac5806e60701e9903da`. The original two-file WIP is preserved on
`feat/preserve-a1-1d-5r1-linux-acl-wip-20260717` at
`395705a5b893aa7704e3a424904a8cd42f247dd3` with exact parent `ebbc5d56`. Its binary and ordinary
patch SHA-256 are both `e852cb8da883e2bffaecde3b46502ee374010a74d56584f1846707c94749719f`;
the original file SHA-256 values are `c0655e3fd0a7b5d5b31cdbdc9219265c52fbc5be9fbde084c66cd171dba1767e`
for `trusted_fs.rs` and `0aad8f5bc01a41b98de174842e92ec2910f12b9a946ad965788d33bab10ed391`
for `home_bootstrap.rs`. The docs-only contract commit is
`17ea3a839345cd47a5b2409cde0d4facdde09446`; the separate two-file runtime commit is
`4d0acff68e20d86b97fe5367b8a4617554f33ef4`. Its final per-file SHA-256 values are
`eeb55bef5c4ef74cf2cb6d0ff5ca723bc8b35417a4d8b12cc0f1e0f660edc994` and
`861e5c5e71e1cde9480ea9d502565edac217eb8db10866e98cffcc51fff8eb9a`, respectively; its binary
diff SHA-256 is `1c9be9810e7f0048d8265b8fb79e7fd3920a0b6e0b34e2e05d4d377f088b7c99`.

The exact proof commands/results were:
`cargo test -p shell --lib execution::agent_runtime::host_session_authority::trusted_fs::platform::tests`
**48 passed**; `cargo test -p shell --lib execution::home_bootstrap::tests` **9 passed**;
`cargo test -p shell --lib execution::agent_runtime::host_session_authority` **165 passed**;
`cargo test -p shell --test world_deps_home_scaffold_wdh3` **14 passed**;
`cargo test -p shell --test installer_env_wcu4` **4 passed / 1 inherited hard-coded `/tmp`
failure unchanged**; `cargo test -p shell --test world_deps_scaffold_wdh3` **0 passed / 2 inherited
hard-coded `/tmp` failures unchanged**; `cargo clippy -p shell --lib --tests -- -D warnings`,
`cargo check --workspace --all-targets`, `cargo fmt --all -- --check`, and `git diff --check` all
passed. The live shell comparator moved from inherited **1054 passed / 149 failed / 1203 total** to
**1080 passed / 149 failed / 1229 total**: `PassToFail = 0`, `FailToPass = 0`, `NewFail = 0`,
`Removed = 0`, and `RenamedOrSubstituted = 0`; all 149 retained names and normalized failure
signatures were unchanged, and all 26 added R1 tests passed.

R2-1 differential comparison uses that 1080/149/1229 result with one explicit accounting
exception: exactly the canonical old/new trigger-name mapping above is accepted, and one new
`test_version_is_non_mutating` must pass. `PassToFail = 0`, `NewFail = 0`, retained failure names and
normalized signatures remain unchanged, no test is removed or substituted, and no other rename is
accepted. The mapped test preserves the R1 scaffold fixture/assertions and changes only its
authenticated bootstrap trigger.

GitNexus change detection mapped the two authorized runtime files to 132 changed symbols and 24
existing downstream flows, with aggregate CRITICAL risk from the already authorized
HostSessionAuthority revalidation surface; it found no new world/policy/gateway product-flow owner.
The docs proof reviewers `/root/docs_acl_proof_review_2` and
`/root/docs_security_boundary_review_2` returned CLEAN after one stale phrase was corrected. Initial
runtime reviewers `/root/runtime_acl_vfs_review_1`, `/root/runtime_security_race_review_1`, and
`/root/runtime_diagnostics_compat_review_1` identified final-snapshot/mask/race and candidate-
provenance gaps; those were remediated with red/green tests. Fresh reviewers
`/root/runtime_acl_vfs_review_2`, `/root/runtime_security_race_review_2`,
`/root/runtime_diagnostics_compat_review_2`, and `/root/runtime_integrated_r1_review_1` then returned
CLEAN against the frozen runtime hashes. No privileged installer/product wall, native macOS proof,
or cross-platform completion is claimed.

### A1.1d-5R2-1 host-context and Unix-dev closeout

**Decision/status:** A1.1d-5R2-1 is implementation- and review-complete through runtime commit
`2653c2ef20ae2e119a444811e6fb46e86d1a6ec6`. The separately published contract corrections are
`e42b1a1ead9dbbebd399e6ba6ab056e342a3b4db` (authenticated hidden home-bootstrap action) and
`1565c2ac0af358b25ef8499cddee1daea051c7b0` (staged trace binding). Runtime commits are
`1acc8c70`, `d9d991b6`, `aea192f6`, `85798bfe`, `fee75ff0`, `af85adf0`, and `2653c2ef`.

The shared transport codec proves exact IH line framing/base64url/SHA-256 commitment and strict
missing, duplicate, unknown, malformed, reordered, path, commitment, and principal rejection. Unix
dev install/uninstall/dev-shim and standalone witnesses construct or validate one context; hidden
argv alone selects internal mode; the authenticated `--install-bootstrap-home-v1` action binds the
current account+UID and checked H/R before the unchanged explicit-context R1 bootstrap. The
unprivileged A/B product matrix proves custom A remains authoritative for bootstrap, shim
deploy/remove/status/doctor/repair, generated env/manager/preexec/helper projections, uninstall,
shell `A/trace.jsonl`, and policy Git A, with no product fallback/access under B. Parse/help/version
exits are non-mutating. The one scaffold-trigger rename preserves its fixture/assertions, and the
new version-nonmutation test passes.

Focused transport, IH, R1 private-home/HostSessionAuthority, installer, script, shim
deploy/status/doctor/health, trace, replay, and A/B suites passed. `cargo check --workspace
--all-targets`, shell all-target warnings-denied Clippy, `cargo fmt --all -- --check`, Bash syntax,
available ShellCheck, `git diff --check`, and GitNexus detection passed. The final broad shell run,
which became the **R2-2 historical starting baseline**, is **1089 passed / 149 failed / 1238 total**
versus inherited **1080/149/1229**: `PassToFail = 0`,
`NewFail = 0`, no removed/substituted test, and all 149 retained normalized failure signatures are
byte-identical after the already-audited dynamic orchestration-ID normalization
(`c818c4f2c4acce52ea5cb057020419bb10dd871148bafcc38112f13437aba6e9`). Windows source-only
comparison removes all 44 R2-1-introduced errors and adds none; the 47 retained errors are inherited,
so no Windows IH or platform capability is claimed.

Fresh read-only reviewers `/root/uid_lint_remediation_rereview`,
`/root/windows_cfg_remediation_review`, and `/root/final_integrated_r2_1_rereview` returned CLEAN.
The neutral trace setter, default compatibility callers, physical-shim/replay production files,
world-deps production, trace lifecycle semantics, shim replacement/migration/removal predicates,
R1 private-home security behavior, and every release/sudo/service/world/platform/lifecycle owner are
unchanged. PI-118 and compatibility removal remain R2-3-owned. `RG-HOME-01`, `RG-INSTALL-01`,
A1.1d, A1, the B1/B2.1 joint closeout, and B3.1 remain open; no seam is promoted.

At R2-1 closeout, the historical next packet was **A1.1d-5R2-2 — Unix release, sudo, Linux service,
and runtime propagation**. Routes A–D have since become individually review-clean, their integration
closeout failed source closure, and R2-2E has since become review-clean. F0/F0a/F0b/F0-HC were then
review-clean, canonically closed out, and preserved. At that historical checkpoint R2-2F was the
next packet. After F and the renewed closeout, the plan then required the complete Linux
regression and normal product lifecycle smoke without outer overrides. That proof can unblock
A1.1d Linux closeout and the Linux
product-smoke portion of the B1/B2.1 joint closeout. Native macOS proof remains separately required
to close A1.1d, but is not a prerequisite for the B1/B2.1 architectural corridor. B3.1 remains
blocked on the joint closeout, and no A1.1d-5I evidence authorizes B3.1 or remediation work.

## A1.1e explicit-home policy proof requirement

A1.1e must route accepted-home policy input through the canonical broker resolver; shell-local
parsing, layering, validation, finalization, or explanation is a blocking authority error. Focused
differential proof must show that ambient and explicit APIs converge for identical defaults,
global-only, workspace-only, global-plus-workspace, replacement/merge, and explain/no-explain
inputs, including unchanged source-layer classification, provenance paths, derived legacy and V3
world-filesystem fields, network/backend/dispatch values, validation errors, and finalization.
Conflicting explicit home A and ambient home B plus CWD changes after acceptance must show that the
explicit result reads neither ambient home nor a newly selected home. Missing global policy,
malformed global/workspace policy, unsafe input, and finalization failure must fail consistently.

The same wall must prove that config retains its existing conditional policy-parsing behavior and
that absent descriptor entries still perform complete expected store/workspace/session/reference
and policy/snapshot identity validation. No global environment mutation may stand in for explicit
resolution. Passing these focused gates does not close `RG-AUTH-03`, `RG-HOME-01`, `RG-POLICY-03`,
or `RG-BASE-01`; dispatch narrowing, immutable active-work policy, A1.1d integrated Linux closeout,
and A1.1d cross-platform proof remain open. At the time of this A1.1e proof, A1.2 was unstarted;
that sentence is historical evidence, not current sequencing authorization.

### A1.1e recorded result

The docs-only allowlist correction is `69db6b29`. P1 is `8826270a` plus test remediation
`d2aac84f`; it adds the exact `HostSessionAuthority` resolution facade, exact root/authority
revision and commitment observations, full-store exact-current validation, and stale-observation
rejection without adding production `ExpectedAbsent` or transition issuance. P2 is `49876412` plus
remediations `80b0f9e0`, `839f78b4`, `182dd4af`, `1e356654`, and `cd676614`. The broker API
`resolve_effective_policy_with_explain_from_global_source` accepts explicit broker-appropriate
source path/bytes and enters the same private canonical core as the ambient API; shell code does not
parse, layer, validate, finalize, or explain a parallel policy. The accepted bootstrap home also
owns descriptor-backed config/policy/inventory reads and a distinct bound StateStore capability;
inventory provenance uses the accepted physical path and the bound StateStore capability exposes
no descendant-path API.

Recorded proof is broker differential `4/4`, broker `75/75`, config `20/20`, inventory `16/16`,
policy `13/13`, policy snapshot `9/9`, HostSessionAuthority `95/95`, StateStore `188/188`, and the
explicit-home group `19/19`. These cover defaults/global/workspace precedence and replacement,
explain provenance, malformed and finalization errors, conflicting ambient/explicit homes,
post-acceptance CWD input, conditional config policy parsing, absent and unsafe descriptors, exact
physical inventory provenance, stale observation, exact current-root identity, and bound
StateStore replacement rejection. Format, broker/shell check, Clippy with warnings denied, diff
check, and scoped GitNexus change detection passed; GitNexus classifies the complete authority and
policy range as critical breadth (`118` changed symbols, `67` affected processes), consistent with
the focused proof wall and without authorizing sibling work.

Exact focused command mapping:

- broker differential: `cargo test -p substrate-broker --lib a11e_explicit_global_source`;
- broker full: `cargo test -p substrate-broker --lib`;
- config/inventory/policy/snapshot: `cargo test -p shell --lib execution::config_model::tests`,
  `execution::agent_inventory::tests`, `execution::policy_model::tests`, and
  `execution::policy_snapshot::tests` respectively;
- authority/StateStore: `cargo test -p shell --lib execution::agent_runtime::host_session_authority`
  and `cargo test -p shell --lib execution::agent_runtime::state_store::tests`;
- explicit-home group: `cargo test -p shell --lib explicit_`;
- static wall: `cargo fmt --all -- --check`, `cargo check -p substrate-broker -p shell`,
  `cargo clippy -p substrate-broker -p shell --lib -- -D warnings`, and `git diff --check`.

The broad shell library wall remains non-green at `843 passed / 171 failed`; the sorted 171-test
failure set is exactly unchanged from the accepted `842/171` deferred legacy-writer/lifecycle
baseline at `49876412`, both from `cargo test -p shell --lib -- --nocapture`. Task evidence captured
the baseline log as `/tmp/substrate-a11e-shell-lib.log` with SHA-256
`ca3da7f69e4b53e8009900927c96abf20152127b2dab14e37b90caa3cb7416ad` and the final log as
`/tmp/substrate-a1-1e-shell-lib-combined-final.log` with SHA-256
`d9abaedc11fc8960a7fa53ac4c5d8cba74d842fe7fe8612b533e0ea56a47c6d7`; sorted failure-name lists
were compared with `diff -u` and produced no output. These task-local paths are evidence provenance,
not durable product artifacts. No unit/component result is promoted to integrated product proof. A1.1d integrated Linux
closeout remains open, native macOS closeout remains pending, `RG-AUTH-03`, `RG-HOME-01`,
`RG-POLICY-03`, and `RG-BASE-01` remain open, and no seam is promoted. Real CLI/helper/REPL
adoption, Start reservation, transition-intent issuance/claim/application, parked-successor repair,
dispatch narrowing/enforcement, auto-attach adoption, and all A1.2 work were deliberately excluded
from that A1.1e closeout. A1.2 work exposed the cycle and remains preserved out of the source
branch. This paragraph records the A1.1e closeout conclusion at that time; its old next-packet/order
statement is superseded by the Case B production-ingress audit below. Do not restore or modify the
broad A1.2 checkpoint, and do not begin A1.2b before the joint closeout → B3.1 → C1 corridor lands.
That historical next-packet statement is now superseded: bounded A1.2a, A1.2a-WB, and A1.2a-S are
landed and independently review-clean. B1/B2.1-R0 is also landed and independently review-clean
through `bb3eefba`; B3.2a plus B3.2a-WA are independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda`. The recovered B1/B2.1 cores and B1/B2.1-0 are
review-clean through `6436289f`, `c519024b`, `de727091`, `717579b0`, and `83101dcb`, and their
later joint production integration closeout is recorded against the bound 2026-08-03 source
snapshot without additional product/test edits. B3.1 is dependency-ready, A1.3 is not
dependency-ready, and A1 as a whole remains incomplete and non-landable.

## A1.2a, A1.2a-WB, and A1.2a-S recorded result

A1.2a is landed and published through `b5f2b4f8dd7d9f650c462cd4626a562cacc1d27f`. Its bounded
commit chain is strict V2 greenfield upgrade `ab2af5a4`, strict V2 dispatch `91d7e491`, atomic Start
issuance `eba02f17`, atomic claim/application `e25896a9`, terminal-handoff hash-input documentation
`a66096be`, and terminal/current-authority read completion `b5f2b4f8`. Independent review is clean.
The exact shell-library baseline at that commit is `876 passed / 171 failed / 0 ignored`; the 171
failures are the inherited legacy-writer/preflight set.

The newly bounded prerequisite order is **A1.2a → A1.2a-WB → A1.2a-S → B1/B2.1-R0**.
A1.2a-WB owns only the Host/world-binding write/read correction in `transition.rs`, its colocated
`transition_tests.rs`, and `facade.rs::HostSessionAuthority::resolve_current_exact`. The reader must
accept and return the exact authority persisted under the same four-case matrix; all other facade
behavior remains outside scope. The packet does not change schemas, canonical JSON bytes, golden
vectors, V1/V2 fixtures, migrations, compatibility behavior, already-persisted objects, authority
fields, or world capability/policy enforcement. Its docs commits are `09b7ba6d` and `7ab20f1c`;
runtime commit `275f9fa2` is independently review-clean.

A1.2a-S runtime commit `2f2fecb3` is independently review-clean. It changes only
`crates/shell/src/repl/async_repl.rs`: the ordinary internal greenfield host path now creates an
identity-free unpersisted proposal, obtains the exact optional world binding, applies or exact-joins
Start before transport or legacy persistence, materializes `PreparedAgentRuntime` only from the
applied authority, carries `RuntimeAuthorityContext::Bound` to the internal toolbox, performs zero
activated legacy session/participant/snapshot writes, and leaves startup ownership Pending. Exact
`Host + Some` and `Host + None` tests, the six host runtime lifecycle tests, and all 132
HostSessionAuthority tests pass; formatting, focused Clippy with warnings denied, and diff checks
pass. The post-review serial shell wall is `898 passed / 161 failed / 0 ignored` against the exact
starting `876 / 171 / 0`: 10 exact `FailToPass`, zero `PassToFail`, zero `NewFail`, and no removed or
renamed test. The 10 improvements are the six existing startup/shutdown proofs now exercising
Pending authority with zero activated legacy writes and four existing dispatch-validation proofs
that now reach their unchanged assertions after canonical Start application. The remaining 161 are
all inherited failure names, but they are not all the same normalized failure: a first-panic
signature audit classifies 137 as `FailToSameFailure` and 24 as `FailToChangedFailure`. Those 24
advance past the removed early Start preflight to separately gated seams: 10 reach the canonical
live-orchestrator-parent requirement, four reach the still-missing canonical orchestration-session
consumer, eight reach a later legacy-writer boundary, and two reach their unchanged downstream
assertions. No new failure name, assertion change, fixture weakening, or test substitution is
involved. These progressed failures remain explicit inputs to their later owning packets and are
not counted as A1.2a-S closure of the B1/B2.1 joint differential gate. Reviewer
`/root/a12a_s_runtime_review_1` completed read-only with verdict CLEAN. No seam is promoted;
B1/B2.1-R0 is independently review-clean through `bb3eefba`, and B3.2a plus B3.2a-WA are
independently review-clean through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`.

## B0 closeout evidence

B0 landed the runtime-owned identity and ordering carrier without starting B1, B2.1, B3.1, C1,
or A1.2. `RuntimeFrameIdentityV1`, `RuntimeEventIdentityV1`, and
`RuntimeTerminalIdentityV1` are canonical shared types in `crates/common/src/agent_events.rs` and
are re-exported without a second representation by `crates/transport-api-types/src/lib.rs`.
`ExecuteStreamFrame` carries frame identity on every variant, semantic identity on Event and Exit,
and exact matching terminal identity on Exit. Strict runtime-frame decoding rejects missing,
empty, zero, malformed, unknown nested, conflicting duplicate, and terminal-mismatch identity.
Standalone legacy `AgentEvent` decoding remains bounded-compatible, but a runtime Event cannot
omit identity and no host decoder repairs it.

The exact producer paths are ordinary execution/deny/error/stdout/stderr/status/Exit emission and
`StreamingSink` in `crates/world-service/src/service.rs`, plus retained member launch, wrapper-event,
completion, and submitted-turn emission in `crates/world-service/src/member_runtime.rs`. Each path
constructs one `RuntimeEventStreamProducer` before the first frame. Its UUIDv7 stream ID remains
stable for the accepted stream; frame sequence starts at one and advances once for every
Start/stdout/stderr/Event/Exit/Error frame; semantic sequence starts at one and advances only for
Event and Exit; Exit is the final semantic event and its identity exactly equals terminal identity.
The producer rejects post-terminal/post-error emission. Concurrent stdout/stderr writers retain the
producer lock through channel enqueue, so assigned order cannot be reversed before observation.
Canonical NDJSON clone/re-emission preserves the complete identity and exact bytes.

Carrier-only shell compatibility changed in the allowed consumers. Identity is decoded and
forwarded unchanged. Missing identity fails closed. Error, body/decode failure, EOF, timeout, and
stream exhaustion cannot synthesize completion, and diagnostic `Failed`/`Invalidated` state without
an explicit terminal observation no longer yields `AlreadyTerminal`. This is negative B0 proof, not
B2.1 dedupe, gap detection, journaling, replay acceptance, or restart-safe observation.

Linux proof on 2026-07-13:

- `cargo test -p substrate-common --test agent_hub_event_envelope_schema`: 26 passed;
- `cargo test -p transport-api-types --lib`: 53 passed;
- `cargo test -p world-service --lib`: 120 passed, including ordinary producer, distinct-stream,
  post-terminal, retained completion, canonical replay, and concurrent stdout/stderr ordering cases;
- focused shell carrier tests passed for unchanged identity forwarding, missing-identity rejection,
  Error/EOF non-completion, malformed-Event then Exit rejection, Error then Exit rejection, explicit
  terminal observation, and corrected multi-event fixtures;
- `cargo test -p world-service --test member_runtime_world_placement_v1 -- --nocapture`: one passed,
  one pre-existing full-isolation case explicitly ignored;
- `cargo test -p world-service --test member_runtime_retained_lifecycle_v1 -- --nocapture`: four
  passed;
- `cargo check --workspace --all-targets`, warnings-denied Clippy for `substrate-common`,
  `transport-api-types`, `world-service`, and `shell`, `cargo fmt --all -- --check`, and
  `git diff --check` passed;
- live `substrate world doctor --json` reported `ok = true`, active socket/service, and a successful
  socket probe; a real `substrate --world --shim-skip -c` command completed successfully.

GitNexus detected 214 changed symbols in 12 code/test files, zero resolved execution-flow hits, and
LOW graph risk. Because schema breadth is under-resolved by that index, manual review treated the
carrier change as HIGH breadth and traced every in-scope producer and decoder. Fresh built-in
reviews found and closed concurrent stdout/stderr enqueue reordering, Error/EOF false terminality,
malformed-Event skipping, duplicate fixture sequences, and non-live-state `AlreadyTerminal`.
The final fresh review found no actionable B0 issue.

Proof limitations are explicit: no native macOS run was required or performed; producer canonical
re-emission is proven, but durable reconnect/replay acceptance belongs to B2.1; the broad shell
library run remained non-green because 171 legacy StateStore/lifecycle tests fail at the existing
authority-writer preflight, and one adjacent obligation-era fixture fails before stream construction
at the same legacy StateStore-root preflight. Neither is counted as B0 proof. The B0-owned clauses of
`RG-EVENT-01`, prerequisite clauses of `RG-SUP-01` and `RG-MSG-01`, and carrier clause of
`RG-OBS-01` are satisfied. The recovered B1 receipt core, B2.1 durable consumer core,
replay/startup activation, authority-store correction, and B1/B2.1-0 are review-clean, while
the later joint production closeout is now recorded against the bound 2026-08-03 source snapshot.
The A1.2a-WB,
A1.2a-S, B1/B2.1-R0, B3.2a, and B3.2a-WA blockers are satisfied. B3.1 is ready; C1 and A1.2b are
not. A1.2a, A1.2a-WB, and A1.2a-S are landed; the preserved broad A1.2
checkpoint remains
untouched at
`18bea80b75ad2c59c7b635851b14552e380585f2`. No seam is promoted.

## B1 production-path sequencing blocker and disposition

The preserved donor checkpoint proved proposal, exact acknowledgement, and activated-store-safe
`WorldWorkReceiptRegistry` persistence. Before recovery, the real ephemeral `Start` branch called
`persist_world_work_acceptance`; after it succeeded, the next active-work operation was
`AgentRuntimeStateStore::register_active_ephemeral_world_task`. That legacy sessions-collection
writer correctly rejects an activated HostSessionAuthority store. The failure must not be ignored,
and activated-store rejection must not be weakened.

The legacy record was also the inspect/cancel route; ephemeral cancel additionally waited on
the process-local `ActiveEphemeralTerminalWaitTracker`, and `ActiveEphemeralWorldTaskGuard::drop`
deleted the record. The retained-turn stream had no corresponding durable accepted-run observer and
remained foreground-owned through `Exit`. Deleting registration alone would therefore regress
inspect/cancel/wait behavior, while moving those mixed responsibilities into the receipt registry,
a fixed active-task side table, or generic activated-store writer would violate ownership.

The completed prerequisite repair kept the B1-3a/B1-3b receipt core → B2.1-1/2/3 supervisor branch
independent while A1.1e → A1.2a current-authority protocol → A1.2a-WB → A1.2a-S bounded
internal Start adoption → B1/B2.1-R0 → B3.2a → B3.2a-WA builds the authority/retained branch. They first join
at B1/B2.1-0 action-scoped dispatch
preparation → one joint B1/B2.1 production closeout → B3.1 →
C1 → A1.2b. The receipt/supervisor cores may be review-clean without closing B1.
`WorldWorkReceiptRegistry` retains proposal/immutable acceptance truth;
`WorldWorkExecutionSupervisor` owns active observation, journal, terminal reconciliation, restart,
and caller-drop survival; `WorldDispatchControl` consumes those truths for bounded compatibility;
HostSessionAuthority/StateStore supply only their explicitly bounded authority/read and physical
persistence capabilities. B1/B2.1-0 now routes the named B-owned production actions without the
legacy active-task writer; the later joint closeout still owns complete production integration
proof and no early return was introduced.

## B1/B2.1 `RegressionMasked` stop and ownership disposition

The post-review production-ingress audit selected **Case B — canonical production input is
missing** because A1.1e provided only an exact reader whose `resolve_exact` required an
already-current `SessionNamespaceRecordV1::Authority`; at that audit point no production path
created the authority. A1.2a and A1.2a-S now boundedly satisfy that creator/adopter gap for the
ordinary internal greenfield host path. The prepared dispatcher still consumes legacy
session/caller/target record shapes absent from the exact retained-target read, and
`live_retained_worker_count` drives steering even though
`retained_worker_refs` carry no live/terminal state. Neither a legacy `authoritative_live`
composite nor a count of all refs is an acceptable replacement.

The audit therefore rejects the earlier ownership claim that
`prepare_orchestrator_world_dispatch` belongs to A1.2. A1.2a owns only greenfield Start
issuance/application and exact read, A1.2a-S owns only bounded internal Start adoption, and the
prepared dispatcher remains a B-owned consumer after those and the retained prerequisites exist.

The corrected acyclic sequence keeps the preserved **B1/B2.1 core** branch independent while
**A1.1e → A1.2a current-authority protocol → A1.2a-WB Host/world-binding correction →
A1.2a-S bounded internal Start adoption →
B1/B2.1-R0 canonical retained target protocol → B3.2a retained creation/admission →
B3.2a-WA exact bound-world ownership adoption** builds the
authority/retained branch. They first join at **B1/B2.1-0 action-scoped dispatch preparation** →
joint closeout → B3.1 → C1 → **A1.2b
successor/post-turn completion**. A1.2a owns only the strict greenfield V1-to-V2 root upgrade,
followed by greenfield Start reservation/issuance/claim, single application/initial authority
birth, crash-safe exact retry, and a typed read result joining the applied caller descriptor and
bound home/store to exact authority. A semantic/non-greenfield V1 root cannot be converted.
A1.2a-S alone adopts that result on the ordinary internal host bootstrap: the identity-free proposal
is applied after the real dormant-launch adapter has the optional world binding, and only the
applied result constructs the existing fully materialized `PreparedAgentRuntime`. It carries the
bound capability to the live toolbox and suppresses activated-store legacy
session/participant/snapshot writes while leaving startup ownership Pending. Fork/member prepared
runtime paths remain unchanged. It does not adopt hidden-owner plans, public
Attach/Resume, startup-result reconciliation, or post-turn behavior.
B1/B2.1-R0 supplies only the durable canonical retained-target registration protocol; by design it
has no production ingress and cannot satisfy full-dispatcher proof alone. Its complete immutable
descriptor/resume/worker graph is published only after an HSA-owned issuer-request reservation
validates the caller-fixed participant plan and fixes every remaining replay identity/commitment,
then becomes authority only through the sole HSA-owned
atomic lineage/ref mutation, request completion, and non-transition proof. Crash and lost-response
retry rejoin that index; no later messaging or lifecycle semantics are introduced. B3.2a is the
separate RetainedWorkerRuntime-owned production bridge: both real Spawn adapters first atomically
fingerprint the complete validated request/authority/runtime/policy plan under the separate
RetainedWorkerRuntime admission key, whose crash-stable lifetime is independent of HSA keys; count all durable
nonterminal admission slots, enforce the cap, and persist fixed participant/bootstrap-run identity.
Only one durable per-session registration head fixes current authority. Queued `SlotReserved`
records with no current head are valid and remain live for cap accounting. Reconciliation of the
completed current head advances only that record and releases the head; it never automatically
promotes another slot. Only exact re-presentation of the complete canonical request for the
lowest-sequence queued slot can acquire the next head after re-verifying its stored HMAC and the
complete admission-to-current R0-only ancestry. The registry stores no request/prompt/payload
preimage; digest equality alone cannot promote. Changed bytes conflict without mutation, later
slots cannot overtake, and caller/PID/helper/socket/endpoint/timeout/EOF/liveness/observer state
cannot acquire, replace, renew, or steal. An abandoned earliest slot conservatively blocks later
slots until its exact request retries. Remaining B3.2 owns exact durable admission-state resolution
and restart reconciliation, while B4 owns the user/tool-facing inspect/cancel verb and distinct
outcomes; B3.2a adds neither protocol. The bridge passes the head plan to R0, exact-joins R0 before
transport, and carries one transport-neutral typed equality proof through the direct dispatcher and
live internal-toolbox adapter into the real transport-api `Service::execute_stream` member branch
and `MemberRuntimeManager::launch` validation. The live adapter cannot allocate a retry-local
participant or invoke activated-store legacy writers. Exact Registered
truth makes the target routable; only exact B0 terminal truth removes it from the live count, while
every ambiguous interruption stays nonterminal and counted. Spawn policy/outcome/events remain
unchanged.

The first bounded Linux authority-managed live Spawn then exposed `RG-WORLD-ADOPT-01`. The exact HSA
binding and typed launch proof named the already-running generic REPL world; world-service inferred
shared-owner `AttachOrCreate`, created a different world, and placement validation rejected the
mismatch before member creation. This is correct fail-closed behavior but blocks product proof. The
durable admission remained `InterruptedNonterminal`, and the B3.2a-WA correction was required not
to rewrite it or report launch success. The docs-first correction assigns only physical ownership metadata to world-service/Linux
backend: after existing strict `Some(exact proof)` validation, adopt the exact HSA-bound generic
world as the same-ID/generation shared owner under trusted crash-durable publication, then launch.
Exact retry joins; conflicts fail without mutation; no alternate world, HSA rebinding, world-api
field/schema, request preimage, liveness authority, compatibility-path change, recovery/resend, or
platform claim is permitted. Adoption does not prove transport submission, Registered, routability,
terminal success, or member launch, and no seam is promoted.

The B3.2a typed-carrier compatibility allowlist includes one exact mechanical correction because
`#[serde(default)]` does not initialize Rust struct literals. RunWorldTask and ForkWorldWorker may
set the new optional proof field to explicit `None` only in their existing transport builders, and
macOS `convert_member_dispatch` may set it to explicit `None` only while preserving its existing
world-api conversion. Those paths gain no retained-worker launch authority and retain every prior
identity, lineage, backend, protocol, run, session, world, prompt, runtime, route, policy,
placement, lifecycle, error, and outcome behavior. No world-api edit or macOS authority-managed
Spawn adoption is authorized. Authority-managed B3.2a Spawn alone requires `Some(exact proof)` and
fails closed before process creation on missing, malformed, or mismatched proof; no default helper,
side table, environment carrier, alternate route, or hidden proof synthesis is permitted. This
mechanical correction closes only the compiler-required literal scope and does not complete
B3.2a.

The carrier widening also requires exactly seven mechanical internal initialization sites. In
`orchestrator_world_dispatch.rs`, `fork_world_worker` and
`continue_world_worker_fork_command_bootstrap_after_delivery` pass only explicit `None` to the
widened stream helper. In `async_repl.rs`, `apply_greenfield_host_start_from_authority`,
`prepare_hidden_owner_helper_runtime`,
`start_host_orchestrator_runtime_with_prepared_prompt_and_toolbox_request_tx`,
`prepare_fork_child_runtime_startup_for_descriptor`, and
`prepare_member_runtime_startup_for_descriptor` only initialize, destructure, or preserve the new
optional retained-worker authority fields as `None`. This is compiler-required argument/field
plumbing, not authority adoption: host Start keeps its established host-session authority,
hidden-helper behavior is unchanged, fork stays outside B3.2a adoption, and legacy member
preparation stays distinct from `prepare_member_runtime_startup_from_authority_registration`.
Only the latter may carry `Some(exact proof/admission context)` for authority-managed retained
Spawn. Missing authority on that managed path fails closed with no legacy fallback. No ownership,
policy, lifecycle, transport, identity, lineage, error, outcome, or runtime behavior changes, and
no further symbol or structural scope is authorized. That mechanical authorization did not itself
complete B3.2a; the recorded B3.2a/B3.2a-WA result below now does.

B1/B2.1-0 removes the
missing live-retained field and legacy session/caller inputs from B-owned RunWorldTask, ordinary
retained ContinueWorldWorker, and ephemeral accepted-task Inspect/Cancel/Wait preparation while
leaving `WorkerContinueForkCommand`, retained Inspect/Cancel/Stop, fork, the already-landed B3.2a
spawn creation/admission bridge, the intervening B3.2a-WA exact same-world physical-ownership
bridge, and remaining retained lifecycle steering unchanged and unpromoted. A1.2b first owns the separately reviewed strict
V2-to-V3 root/intent/state extension after B1/B3.1/C1 types are available, then retains
Attach/Resume, startup/post-turn reconciliation, obligation-cut consumption, release, retry, and
optional world-work correlation after C1. Its startup resolution
may join the original Start application revision to current authority only through the unique
contiguous R0 registration-proof chain; acceptance leaves current authority unchanged and terminal
reconciliation preserves every R0-added lineage member/ref. Arbitrary stale revision still fails.
The broad preserved
A1.2 checkpoint is not restored or modified.

The current preserved runtime WIP does not satisfy the differential gate. Donor diff and review
evidence show fourteen historical tests whose production-level proof was masked: the eleven
previously identified dispatcher/inspect/cancel/wait tests plus two guard-drop tests and one
tool-invocation test. The exact inventory is:

1. `dispatch_contract_inspect_world_worker_ephemeral_returns_authoritative_snapshot_without_mutation`
2. `dispatch_contract_inspect_world_worker_ephemeral_resolves_exact_session_binding_when_task_run_id_is_reused`
3. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_unknown_task_run_id`
4. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_stale_linkage`
5. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_backend_mismatch`
6. `dispatch_contract_inspect_world_worker_ephemeral_fails_closed_for_world_binding_mismatch`
7. `dispatch_contract_inspect_world_worker_ephemeral_teardown_removes_routability`
8. `dispatch_contract_cancel_world_work_ephemeral_routes_exact_active_task_over_execute_cancel`
9. `active_ephemeral_terminal_wait_allows_multiple_waiters_to_observe_same_terminal_truth`
10. `dispatch_contract_cancel_world_work_ephemeral_retry_reuses_shared_terminal_truth`
11. `dispatch_contract_cancel_world_work_ephemeral_fails_closed_when_execute_cancel_is_not_delivered`
12. `active_ephemeral_terminal_wait_registration_guard_releases_non_happy_path_registrations`
13. `active_ephemeral_terminal_truth_guard_publishes_failed_terminal_truth_on_drop_after_start`
14. `dispatch_contract_adapter_follow_up_resolution_uses_authoritative_active_task_state`

All fourteen retain these exact historical names and their original production-level assertions.
Tests 1–11 must re-enter the full dispatcher. Tests 12–13 must prove accepted and supervised truth
survives actual guard/waiter destruction; aborting a helper waiter or calling a lower-level owner
directly is not equivalent. Test 14 must exercise the real host tool-invocation-to-dispatch route;
establish current authority through production HostSessionAuthority APIs, acceptance through
`WorldWorkReceiptRegistry`, and observation/terminal truth through
`WorldWorkExecutionSupervisor`; and prove no legacy active-task writer or resolver is used. Direct
receipt/supervisor authority resolution followed by request construction is not equivalent.
Direct resolver, transport, receipt, supervisor, or helper substitutions remain
`RegressionMasked`, even when the lower-level owner is correct.

The bounded control-pack correction authorizes only
`tool_invocation_contract.rs::resolve_follow_up_dispatch_authority_v1`'s active-task branch,
mechanical imports for that branch, mechanical relocation of the existing shared pre-`match`
legacy caller/world resolution unchanged into the retained-worker branch, the unchanged-name
historical test 14, and focused negative tests. The active branch must prove exact task-ID reuse across sessions, unknown task, stale
receipt/supervisor linkage, caller/backend mismatch, world ID/generation mismatch, acceptance
without a supervisor claim, unresolved producer replay, and exact terminal truth. The retained
branch must prove unchanged compatibility behavior, target selection, and error categories. The
function signature and callers remain unchanged. The approved GitNexus HIGH impact is bounded to
12 direct callers, four affected execution processes, and 14 impacted symbols; any broader impact
is a new stop. These additions sharpen `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`, `RG-OBS-01`, and
`RG-DIFF-01` but resolve none of them.

Test renaming, replacement, newly ignored status, assertion deletion or weakening, or lower-level
substitution cannot count as `FailToPass` proof. Within the bounded closeout, only the named
ephemeral accepted-task controls are eligible for failure-to-pass remediation; retained
Inspect/Cancel/Stop may only preserve their exact historical failure and normalized signature until
their later lifecycle owner lands. The historical duplicate-registration test may remain a
legitimate isolated fixture correction only if its production semantics and exact assertions are
unchanged; it does not count as receipt/supervisor remediation merely because the fixture becomes
runnable.

This disposition sharpens the open gates without closing them:

- `RG-RECEIPT-03` requires full-dispatcher exact-session/world/cross-session reuse proof for the
  named ephemeral accepted-task controls over immutable acceptance and supervisor truth, including
  actual caller/waiter/guard drop survival. A direct lower-level owner call cannot stand in for the
  dropped production guard or waiter. It does not promote retained control lifecycle.
- `RG-SUP-01` requires the accepted production path and full compatibility dispatcher to use the
  exact claim/journal without a legacy active-task registration or resolver fallback for the
  action-scoped closeout paths. The tool adapter may read but cannot create, mutate, terminalize,
  or substitute for that supervisor truth.
- `RG-SUP-02` requires restart, actual waiter/guard-drop, terminal-closeout, and
  acceptance-retention proof through the named ephemeral production integration path; replay
  unavailability after world-service restart remains durably unresolved rather than terminalized,
  and retained closeout remains later-owned. The active-task tool projection must preserve the
  distinct unresolved-producer state.
- `RG-OBS-01` requires the real tool-to-dispatch and prepared-dispatch paths to join the A1.2a-S production-bound current-authority read,
  R0's HSA-proven retained target where applicable, B1 acceptance, and B2.1 observation identity;
  episode/liveness composites and compatibility
  projections are not observability authority. Test 14 must reach that join through the real host
  tool route; a direct call to the corrected resolver is not production proof.
- `RG-DIFF-01` requires all fourteen historical names and original production-level assertions.
  The eleven dispatcher tests re-enter the full dispatcher, the two guard tests exercise actual
  drop, and the tool test uses the real tool-to-dispatch route. Retained control cases may remain
  `FailToSameFailure`, but direct resolver/transport/receipt/supervisor substitutions remain
  `RegressionMasked` even when they exercise individually correct owner APIs. Test 14 additionally
  retains its distinct unknown/stale/backend/world/nonterminal/terminal/unresolved outcomes and
  cannot weaken or replace them with a generic result.

Only a fresh broad differential classified `ExpectedBaselineResolution` after that real-path proof
may close the joint packet. `PassToFail`, `FailToChangedFailure`, `Removed`,
`RenamedOrSubstituted`, `NewFail`, and `NewIgnored` must all remain zero, and every retained failure
must preserve its normalized signature.

Historical A1.2a-WB authorization record: the preceding docs-only correction recorded landed
A1.2a and authorized only the subsequently reviewed A1.2a-WB implementation in `transition.rs`,
`transition_tests.rs`, and the exact `facade.rs::HostSessionAuthority::resolve_current_exact`
boundary. At that point it did not authorize any other facade behavior or A1.2a-S and required WB
to be implemented from its bounded contract rather than by restoring the broad A1.2 checkpoint.
That authorization state is superseded by the recorded review-clean WB and A1.2a-S closeout above.
B1/B2.1-R0 is review-clean through `bb3eefba`; B3.2a plus B3.2a-WA are independently review-clean
through `d0a70727c2bec2b2d6fe0754ea469c4682684dda`. At that point B1/B2.1-0 had not begun, and its
implementation remained unauthorized until the docs-only control-pack correction was independently
review-clean. That historical authorization condition is now satisfied: the correction was
published as `3741cadd`, the bounded runtime result is review-clean through `83101dcb`, and later
packets remain unauthorized.

## B1/B2.1-R0 recorded result

B1/B2.1-R0 started from exact source commit
`f630835ab3715f957b0f49697e104c40d591305e`; preservation branch
`feat/preserve-b1-b2-1-r0-f630835a` was pushed at that exact commit before editing. R0-1 is
`ace6cebd`, `e3768c3f`, `6154dc97`, and `6ec5eb78`; R0-2 is `5582039a`; R0-3 is
`3f4464b7`; R0-4 is `23993184` plus review remediation `0d5225f3`; final integration remediation
is `bb3eefba`. The source range changes only the eight-file subset of the authorized R0 allowlist:
`agent_runtime/{mod.rs,retained_worker_runtime.rs}` plus
`host_session_authority/{facade.rs,store.rs,store_schema.rs,store_tests.rs}` and
`host_session_authority/store/platform/{layout.rs,object_persistence.rs}`.

The component durably reserves the domain-separated retained-registration request before any
object write, fixes the caller-supplied participant and every registration/object identity,
commitment, expected authority value, policy, world, backend, protocol, and timestamp, publishes
only the exact immutable descriptor/resume/worker graph, and atomically applies one non-transition
authority link. Exact resolution proves store/session/lineage/ref/object/policy/world joins and one
unique contiguous Start-to-R0 ancestry whose highest proof revision equals current authority.
Crash/restart windows, lost response, deterministic Reserved-to-Applied publication interleaving,
and identical/conflicting two-process contention converge or fail closed with no unauthorized root
or object mutation. An Applied retry with a different valid authority-store ID is rejected without
mutation. Startup ownership stays Pending and retains its original expected revision.

Final Linux component proof is retained runtime `25 passed / 0 failed`, full
HostSessionAuthority `132 / 0`, explicit strict V1/V2 decoding, object reachability/index/orphan,
key lifecycle/rotation, crash/restart, and cross-process checks green, shell all-target Clippy with
warnings denied green, and `cargo check --workspace --all-targets`, formatting, and diff checks
green. The final serial shell wall is `923 passed / 161 failed / 0 ignored` against exact starting
`898 / 161 / 0`: all 25 additions are R0-only passing tests; `PassToFail`, `NewFail`,
`FailToPass`, removed/renamed/substituted/weakened tests, and ignored-test changes are zero. All 161
inherited failure names and normalized bodies are identical after replacing only generated
`aos_<ID>` values. There is no unexplained production-path `FailToPass`, so `RegressionMasked` did
not trigger.

Fresh read-only packet reviewers `/root/r0_1_clean_review`, `/root/r0_2_review`,
`/root/r0_3_review`, and `/root/r0_4_rereview` returned CLEAN. The first R0-4 reviewer found the
Reserved-to-Applied publication race fixed by `0d5225f3`; the first final integration reviewer
found the Applied retry store-ID gap fixed by `bb3eefba`. Fresh reviewer
`/root/r0_final_integration_rereview` returned final CLEAN. This is component proof only: R0 has no
production ingress caller, supplies no e2e or live doctor/smoke proof, and has no native macOS
claim. No seam was promoted by R0. B3.2a plus B3.2a-WA are now independently review-clean through
`d0a70727c2bec2b2d6fe0754ea469c4682684dda` as recorded below.

## B3.2a and B3.2a-WA recorded result

B3.2a began from source commit `69b6cddc61578437d1cb1af95055fdb7e0b9c776`. The docs-first
corrections are queued promotion `8726dface3e314c2e24aa95305cd69f01c8bb389`, abandoned-admission
ownership `ba6253483ccabec151ce6077d1df1ce7067ac42b`, carrier compatibility literals
`e1a248589a0d2cee1cedec33227f8b070515c664`, exact seven-symbol carrier plumbing
`47e9ea0daa7982ac3fc3d4c85384af1b0490d563`, and exact-bound-world adoption
`aa38c66badb46ee8d17855432438de1b66accdce`. The final prerequisite replay preserved the original
20-commit runtime range at `feat/preserve-b3-2a-wa-runtime-1d2543c8` pointing to
`1d2543c8ded723f22b1612bf5cb58be2d86fcc12`, then replayed it in exact order as
`73bdd5640588c685c67e73cc4c9425e44c6213da` through
`106f9c85c60ee1649b5c7ae88b47338da0402d5e`. The preserved and replayed aggregate ordinary patch
SHA-256 is `ed417fdca7c26da46b6a51ba26af3e8939f997e9632eec7608931fe7a514107a`; the binary/full-index
patch SHA-256 is `6c574c1655c1668537a159a01dd1dac120cad666ea9924a0206a05ac4d3ee671`.
No preservation commit was merged.

The replayed range supplies the RetainedWorkerRuntime-owned admission key and registry, exact
complete-request HMAC fingerprint, conservative live cap, fixed participant/bootstrap run,
serialized registration head, exact R0 join, and unique durable transport claim. Reconciliation
advances and releases only the current head. A queued `SlotReserved` record with no head is valid;
only complete exact re-presentation of the earliest request may acquire the head. Changed bytes,
digest-only input, later-slot retry, PID/caller/helper/socket/endpoint/timeout/EOF/liveness state,
and observer loss cannot promote, replace, renew, or steal. Request/prompt/payload preimages are
not persisted. Publication-boundary, restart, contention, exact/conflicting retry, no-steal, cap,
and complete fingerprint mismatch tests are green. `RG-ADMISSION-01` deliberately remains open for
remaining B3.2/B4; B3.2a adds no cancellation, abandonment, reclamation, or recovery protocol.

The typed carrier requires `Some(exact RetainedWorkerLaunchAuthorityProofV1)` on both
authority-managed Spawn adapters and strict-validates it in the real transport-api
`Service::execute_stream` member branch and again at `MemberRuntimeManager::launch`. RunWorldTask,
ForkWorldWorker, fork continuation, Host Start, hidden helper, legacy member preparation, and macOS
compatibility conversion carry explicit `None` and gain no retained-worker authority. The sole
durable claimant launches transport; an exact joined claim does not reproject request bytes or
resend. Exact Registered truth makes the admission routable, exact B0 terminal truth terminalizes
it, and interruption/ambiguity remains nonterminal and counted. Authority-managed readiness now
emits the exact Registered session-handle event before any bounded buffered pre-Registered status
event; compatibility `None` retains its old ordering. WA commit
`11933b310c6424fc01e5aea18121f39ddf807f6a` implements exact same-world ownership publication, and
readiness remediation `d0a70727c2bec2b2d6fe0754ea469c4682684dda` closes the final live-path
ordering defect.

The Linux proof wall is green: complete HostSessionAuthority `132/132`, complete
RetainedWorkerRuntime `81/81`, transport-api-types `54/54`, world `113 passed / 1 pre-existing
ignored`, world-service `126/126` library tests plus all applicable integration targets, and the
authority-managed readiness integration `5/5`. The direct internal-toolbox production-adapter test
is green. Workspace all-target check, focused Clippy with warnings denied, formatting, and
`git diff --check` pass. The serial shell differential is exact baseline
`923 passed / 161 failed / 0 ignored` to current `981 / 160 / 0`: 57 added passing shell tests,
zero removed/renamed/substituted/weakened tests, zero `PassToFail`, zero `NewFail`, and one
causally proven `FailToPass`,
`repl::async_repl::tests::orchestrator_world_dispatch_surface_spawns_authoritative_member_runtime`.
All 160 retained failures preserve their normalized first-panic signatures. The additional
readiness regression is a world-service integration test and does not alter that shell inventory.

The final bounded Linux product proof used the reviewed installed world-service binary SHA-256
`9f02fc27fa455c53ed8ab03a49125cb24b7aac38f528b8a8b3326d68d96dcf63`. World doctor returned
`ok: true`; ordinary world execution returned `B3_2A_WA_ORDINARY_REVIEWED_OK`. The production
internal-toolbox Spawn returned one successful receipt for session
`aos_27f14606e443f73927d2f892beabc704`, participant
`rwp_dd8856889e050e3b6d78795c2f1e8ad8`, world
`wld_019f6a96-bdf1-7db1-a5cc-58cbd0f370c9`, generation `0`, and launch span
`spn_019f6a97-ee72-7b92-a30d-7cdc98e7144c`. Backend metadata records that same world as the sole
active shared owner for the same session; no alternate world exists. The unique prompt marker is
absent from authority/admission objects, trace, backend metadata, workspace, service storage,
errors, response, and system journal. Clean REPL shutdown produced exact terminal truth for this
successful smoke; the earlier failed attempt remains separately preserved
`InterruptedNonterminal` and was not rewritten.

Fresh read-only reviewers `/root/b3_2a_3_review_3`, `/root/b3_2a_wa_docs_final_review_5`,
`/root/b3_2a_wa_runtime_review_3`, `/root/b3_2a_readiness_review_5`, and
`/root/b3_2a_final_integration_review_6` returned CLEAN for their final assigned ranges. The final
integration reviewer inspected the complete `aa38c66b..d0a70727` runtime range plus differential
and live artifacts and authorized documentation closeout. B3.2a and B3.2a-WA are review-clean on
Linux. No seam is promoted, no native macOS or Windows proof is claimed, remaining B3.2/B4 work is
unchanged. The later B1/B2.1-0 result is recorded below.

## B1/B2.1 core recovery and B1/B2.1-0 recorded result

The packet began from published source `8aaa1214c07885e66778e9ba045aa6c9472c9768` after the
B2.1 replay/startup contract correction. The active-task adapter authorization landed first as the
docs-only commit `3741caddfa17f8ccb3b6f272731b5df0614a7405`. Runtime preservation remained on
`feat/preserve-b1-b21-0-crossdoc-20260716T175740Z` at
`8234f056b2bb65d5324f8269999d87ca86bbfc1d` and was never merged or pushed as incomplete source.
Because the named checkpoint and cross-document verification skills were unavailable, the packet
used explicit substitutes: remote preservation plus commit/tree/file fingerprints, old-to-new
replay mapping, and a six-file structure/link/table/gate/sequencing/stale-status validation.

Donor `feat/b1-b21-post-review-preservation-20260714T133418Z` commit
`2f5a73168832127e4ec25fa0385246d932552566` with parent
`ba802c76383b81fd015d88b8ac31b2d588d9a2c8` was read-only salvage evidence, not a merge source.
Its B1 receipt hunks became `6436289fd9dd55ea516b96ef3299e4055d1ea718`; B2.1-1/2 supervisor
hunks became `c519024bd91b6ca6e332d0b8881f7d13ded940e0`; B2.1-3 replay/startup hunks became
`de727091a39c884044179a89135df3db5d566778`; and the current-source versioned authority-store
binding correction is `717579b0744154d343985ad439fb8756158f376f`. Historical direct-resolver or
transport substitutions, weakened assertions, test-only authority writers, obsolete A1/R0/B3
hunks, and unrelated fixture/platform changes were excluded. The four recovered commits replayed
from `0f641cde`, `051699d6`, `67476ec5`, and `9cd7f6be` to `6436289f`, `c519024b`, `de727091`,
and `717579b0` with identical aggregate binary patch fingerprint and unchanged per-file content.

B1/B2.1-0 is `83101dcbcc750e6e8fb8979bea19f1f777792188`. Its production changes are confined to
`orchestrator_world_dispatch.rs`, `agent_runtime/state_store.rs`, and only the approved active-task
branch plus colocated proof in `agent_runtime/tool_invocation_contract.rs`. The prepared B-owned
view is used only by RunWorldTask, ordinary retained ContinueWorldWorker, and ephemeral accepted-
task Inspect/Cancel/Wait. It exact-joins current HostSessionAuthority, immutable acceptance,
supervisor claim/cursor/terminal state, store/session/caller/backend/world/task identity, and R0+
B3.2a target/routability where applicable. It requires exact authority revision and an accepted
supervisor cursor, performs no legacy active-task or obligation write, and preserves blocking
foreground behavior. Continue-fork, retained Inspect/Cancel/Stop, fork, and the B3.2a/B3.2a-WA
Spawn path retain their compatibility behavior.

The approved GitNexus impact for `resolve_follow_up_dispatch_authority_v1` remained HIGH with 12
direct callers, 14 impacted symbols, and four execution-process roots: the parked/resumable
retained-worker continue case, outside-lineage rejection, successor fork after owner exit, and
retained fork/cancel successor-authority case. Final change detection expanded to 18 downstream
flows but found no new process family; independent review confirmed that adjacent unchanged
symbols in the large colocated diff caused the broader mapping.

Fresh Linux shell proof moved from `981 passed / 160 failed / 0 ignored` across 1141 tests to
`1054 passed / 149 failed / 0 ignored` across 1203 tests. The exact fourteen historical names and
production routes remain: eleven dispatcher tests enter the full dispatcher, guard/waiter tests
exercise real drop, and the tool test enters the real tool-to-dispatch path. All eleven
`FailToPass` transitions have an exact real-route cause; `PassToFail`, `FailToChangedFailure`,
`Removed`, `RenamedOrSubstituted`, `NewFail`, and `NewIgnored` are zero. All 149 retained normalized
failure signatures are byte-identical after normalization. Receipt, supervisor, replay/client,
world-service, full-dispatch action, active-task negative, retained-runtime, and Spawn regressions
are green; workspace all-target check, focused warnings-denied Clippy, formatting, and diff checks
are green. No joint doctor/live-smoke wall is claimed.

Fresh read-only docs reviewers `/root/docs_authority_scope_clean_review` and
`/root/docs_test_impact_clean_review` returned CLEAN for the authorization amendment. Initial
implementation reviewer `/root/implementation_final_review` found stale-revision acceptance,
missing durable-start-cursor validation, and a legacy obligation-writer/panic path; all three were
fixed with focused red/green proof. Fresh reviewers `/root/implementation_impact_review` and
`/root/implementation_remediation_final_review` then returned CLEAN for the approved HIGH-impact
adapter and complete recovered-core/B1/B2.1-0 range.

B1 receipt core recovered/review-clean: **yes**. B2.1 supervisor core recovered/review-clean:
**yes**. B1/B2.1-0 review-clean: **yes**. B1/B2.1 joint production integration closeout:
**complete**. B3.1 complete: **yes**. C1 complete: **yes**. Seam promotions: **none**. Within the
retained-turn follow-on corridor, A1.2b is now the next architectural packet. At the
B1/B2.1-0 closeout, the repository's exact next packet was A1.1d-5R2-1 — Host context construction
and Unix dev propagation; after review-clean R2-2 Routes A–E, the failed integration closeout, and
the completed exact harness closeout, the next packet at that historical checkpoint was
**A1.1d-5R2-2F — Authenticated world-deps and truthful doctor composition**, followed by renewed R2-2
integration closeout, R2-3, R2-4, and R3. The completed-F ledger below supersedes that next-task
status. Only the joint closeout's Linux
product-smoke portion waits for those remediations and their required smoke, and its receipt and
supervisor semantics are not reopened.

## B3.1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, B3.1 closes the retained worker-to-host
typed event prerequisite without changing `ExecuteStreamFrame`. Producer normalization now happens
only in `world-service` before `AgentEvent` construction, every retained post-acknowledgement
`Event` frame in scope carries the typed `worker_event` member or fails closed before emission, and
shell consumers validate the complete envelope instead of using JSON-pointer repair on
`AgentEvent.data`.

Broad proof remained monotonic through both `make shell-lib-wall` and `make shell-lib-wall-serial`
at `1324 discovered / 1276 passed / 48 failed / 0 ignored` with failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`. That inventory growth is
exactly two new passing shell tests:
`execution::orchestrator_world_dispatch::tests::accepted_retained_typed_event_validation_precedes_generic_journaling`
and
`execution::orchestrator_world_dispatch::tests::typed_control_ack_projection_uses_top_level_worker_event_class`.
The historical normalized-signature SHA-256
`2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90` first moved to the earlier
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
No failure name, failure message, assertion, test identity, or production behavior was removed,
renamed, substituted, ignored, or weakened. B3.1 is complete, C1 is recorded below, A1.2b is also
recorded below as complete on the same bound Tuesday candidate, and no seam is promoted.

## C1 recorded result

At the bound Tuesday, August 4, 2026 source candidate, C1 completes the event-to-obligation
materializer and semantic-cut packet without implementing A1.2b. The accepted retained path now
reconciles exact B1 acceptance plus B2.1 durable retained `Event` refs and validated B3.1
envelopes into `ObligationLedger` as each typed `Event` or terminal `Exit` is durably accepted.
Attention-driving retained events materialize exactly one canonical obligation before the exact B0
terminal event; typed non-attention events advance the classified set and
`materialized_through_event_sequence` watermark without hidden attention; byte-identical replay is
a no-op. `ObligationLedgerSnapshotReadV1` remains `Pending` before exact terminal-cut coverage and
returns one `Complete` snapshot only at the exact B2.1 terminal cut, including the exhaustive
ordered retained-event vector, closed attention disposition, and sorted unresolved
canonical-record commitments. The old terminal-coupled production writer is removed from the
accepted C1 path and retained only on the non-C1 `WorkerContinueForkCommand` compatibility branch
so no competing accepted-path writer remains.

Focused ledger, StateStore, retained-event, and accepted-dispatch tests are green. Both
`make shell-lib-wall` and `make shell-lib-wall-serial` retain
`1330 discovered / 1282 passed / 48 failed / 0 ignored` and failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9`. That inventory growth is
exactly six new passing shell tests:
`execution::agent_runtime::obligation_ledger::tests::unresolved_attention_entries_sort_by_canonical_commitment_before_obligation_id`,
`execution::agent_runtime::state_store::tests::close_prepared_internal_continue_approval_response_obligation_updates_c1_materialized_canonical_record`,
`execution::agent_runtime::state_store::tests::obligation_ledger_plan_persists_to_nonlegacy_root_and_cursor_derives_from_state`,
`execution::agent_runtime::state_store::tests::obligation_ledger_plan_rebases_session_revision_after_unrelated_acceptance_advance`,
`execution::orchestrator_world_dispatch::tests::b21_retained_materializes_attention_obligation_before_terminal_cut_and_replay_is_noop`,
and
`execution::orchestrator_world_dispatch::tests::b21_retained_snapshot_is_pending_until_exact_terminal_cut_then_completes`.
The prior accepted normalized-signature SHA-256
`167807acacbf51c8507ef1c6a20d195b5d1a19e66ea62e5d612de1c2ef340f17` moved to the accepted C1
candidate SHA-256 `e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40`. The accepted
retained-failure differential is limited to 23 FILE:LINE-only movements in
`crates/shell/src/execution/orchestrator_world_dispatch.rs`; failure names, file paths, columns,
and normalized panic bodies remain unchanged, and the exact baseline-to-final differential is
recorded in
[`review-control/c1-differential-evidence.json`](review-control/c1-differential-evidence.json).
No failure name, failure message, assertion, test identity, or production behavior is removed,
renamed, substituted, ignored, or weakened. C1 is complete, A1.2b is recorded below as complete
on the same bound Tuesday candidate, and no seam is promoted.

## A1.2b recorded result

At the bound Tuesday, August 4, 2026 source candidate, A1.2b completes the bounded internal
successor/post-turn packet without public CLI/helper/REPL/auto-attach adoption and without
beginning R3 or A1.3. HostSessionAuthority now publishes only one strict V2-to-V3 root extension
that preserves every strict V2 Start intent, both R0 retained-registration maps, existing
application-proof bytes, and the exact current-authority/R0 lineage corridor without widening
older roots or down-converting V3. Exact current-authority resolution is closed-version aware, the
retained-admission corridor consumes the preserved V2 Start view under V3, and startup acceptance
continues to authenticate the original Start application revision only through one unique
contiguous R0 registration-proof chain to exact current authority.

Successor `Attach`/`ResumeOneTurn` issuance, claim/application, input acceptance, startup/post-turn
reconciliation, `AwaitingObligationCut`, unchanged C1 `Pending`/`Complete` consumption, immutable
journals/results, exact retry, transport reprojection, and `ReleaseEligible` handoff now remain
entirely inside HostSessionAuthority without classifying or rewriting ledger truth. Complete
snapshots use the unchanged C1 disposition directly; pending, incomplete, mismatched, or
substituted cuts remain pending or fail closed. Exact result join survives a separately validated
`Released` transport state through durable reprojection, but this packet does not claim the
destructive payload-deletion/root-advance step itself.

Focused host-session-authority strict codec/store/transition/reconciliation tests and retained
admission compatibility tests are green. Both `make shell-lib-wall` and
`make shell-lib-wall-serial` retain the exact 48-failure inventory and the accepted hashes
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` /
`e53ffb35dbd4fe32ea60ad8da88efe449edc510a5bd0b3fe446e8a368d5beb40` while growing broad proof to
`1351 discovered / 1303 passed / 48 failed / 0 ignored`, exactly twenty-one new passing shell
tests, and zero retained failure-name/message/assertion/test-identity/behavior drift. The accepted
differential is recorded in
[`review-control/a1-2b-differential-evidence.json`](review-control/a1-2b-differential-evidence.json).
A1.2b is complete only as the bounded internal durable successor/post-turn protocol; no seam is
promoted, public consumer adoption remains A1.3/A1.4-owned, and the next authority gate is
`AUTHORITY_REQUIRED:R3_RESUME`.

## Baseline behaviors that all tracks preserve

These rows define permanent behavior contracts, not current pass claims. Any row explicitly marked
open remains a blocking regression gate until its named owner and real-path proof close it.

| Gate ID | Baseline | Required proof |
|---|---|---|
| **RG-BASE-01** | Public world-scoped start → turn/reattach → stop; **open, blocking, and not waived** | A1.2 must supply exact parked-successor `Attach`/`ResumeOneTurn` application and A1.3 must adopt it on the real public CLI/helper/REPL path. Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. Stale lifecycle/world-binding overwrite remains rejected. Prove unsafe other-principal authority fails closed separately, then run the positive smoke against an owner-only private bootstrap home with exact type/`0700`, descriptor identity/replacement safety, qualified access/default `NoData` under authoritative mode bits, and no effective other-principal authority; `NoData` is not physical-absence proof. This gate is neither permanently expected to fail nor successful until that real-path wall is green. |
| **RG-BASE-02** | REPL first-dispatch `run_world_task` binding repair | Correct binding succeeds; stale/mismatched world generation fails closed; no generic binding synthesis on unrelated surfaces. A host runtime may carry exact parent-session world binding while its descriptor and participant manifest remain host-scoped. |
| **RG-BASE-03** | Parked host ordinary-command and continuity parity | Unprefixed `ls`/`pwd` remain usable; policy-required `cd ../` cage denial remains; later targeted host turn reuses session/UAA continuity; public CLI parity stays green. |
| **RG-BASE-04** | Retained spawn/fork/exact continue/exact stop plus ambiguity close | Exact source and child handles route correctly; backend-only follow-up with multiple retained workers fails closed; source detached stop and child live-transport stop remain valid. |
| **RG-DIFF-01** | Monotonic broad-suite differential; a historical failure becoming a pass is neither automatic success nor automatic regression | Apply the exact-name and normalized-signature transition gate in `04`. Preserve complete inventories and artifact hashes; prove every historical failure-to-pass transition through the exact baseline and current full production dispatcher; inspect test and assertion diffs; map the causal change to the owning slice and symbol; and prove no direct resolver, transport, receipt, supervisor, helper, bypass, weakened enforcement, renamed/replaced/ignored test, weakened assertion, removed behavior, or unrelated capability loss. The exact fourteen-test B1/B2.1 inventory above retains every historical name and original production-level assertion. A lower-level substitute is `RegressionMasked`; any otherwise uncertain transition is `BaselineRegressionAmbiguous`. Either classification keeps the owning closeout open, and neither can count as `FailToPass` proof. Independent review is mandatory. |

## Cross-gate smoke scenarios

### S1 — Authority survives episode loss

1. Start a world-scoped host session and retain a worker.
2. Kill the current helper/episode and remove its private socket.
3. Read durable authority, worker manifest, and any active receipt.
4. Reattach or perform sanctioned durable closeout.
5. Prove no stale episode write regressed the authority revision.

Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-CLOSE-01`, `RG-BASE-01`.

### S1A — Revision-bound host transitions survive helper loss and replay

1. Before any production Start, classify genuinely absent, interrupted-initialization, valid-existing, unsupported-pre-A1, and corrupt/unsupported stores. Reject malformed, partial, unreadable, symlinked, permission-invalid, and unsupported roots without generating a replacement store ID. Persist the same normalized physical bootstrap home in the init marker, greenfield certificate, and root; pass its opened handle through the bounded StateStore/config/policy/inventory entry points; change/remove ambient home values between each call and prove no reread; reject a copied/rebound authority tree; and crash/restart after marker, key, root, and marker-cleanup steps without changing store/home identity.
2. Completely and safely enumerate both pre-A1 session/participant authority collections before certificate publication and every semantic transaction. Prove component-by-component no-follow traversal from the trusted home rejects ancestor symlinks, unsafe owner/mode/ACL, cross-device rebinding, and scan-to-publication identity replacement. Prove validated empty collections permit greenfield certification, while any artifact is `UnsupportedLegacyState` without parsing/conversion and unreadable, permission-invalid, symlinked, or partially enumerable collections fail closed. Every pre-A1 read-decide-write-remove-rename transaction retains the opened trusted physical root, descendant handles, exact identity, activation observation, and root lock through final file/directory `fsync`; it never rereads `SUBSTRATE_HOME`, uses ambient CWD, reconstructs an absolute descendant path, or releases the lock early. In a real subprocess, rename/replace/rebind the lexical root after lock acquisition and prove original/replacement identities explicitly: the replacement receives zero writes, temps, removals, or `fsync`-dependent publication, and a lock on the former root is never represented as protection for the replacement. Serialize every in-repository pre-A1 writer on the same root lock before activation; after init-marker or root publication it rejects before mutation. No missing row or compatibility projection satisfies absence.
3. Exercise every A1 object kind against its exact V1 schema, file bytes, typed path, and commitment variant/domain. Reject cross-kind/schema/domain substitution; require `run_present=0x01` and the exact non-empty parent-intent run ID for every sensitive domain, with golden negative fixtures for absent/empty/different runs. Crash before/after each temp-file fsync and rename: recognized operation-bound temps are removed and never promoted, while unsafe/unrecognized temps fail closed. For pending initialization, validate a reused key only against the init marker and re-fsync the file plus `keys/` before root publication. Crash between first-use kind/version directory creation and every directory/parent fsync; only safe empty closed directories survive without object authority. Crash after object rename but before root commit; restart must classify the safe unindexed file as a retained non-authoritative orphan, and only a retry with the complete ref, exact parent intent/run/store context, and verified bytes may adopt it. Exact retry must treat every reservation, tombstone, transition intent, issuer-request index, application-journal, and object-index entry as semantic authority occupancy and may join only complete matching committed state. Rotate and retire HMAC keys across crashes before/after key publication, root selection, retirement root commit, and key deletion; after reconciliation and immediately before root publication, validate the complete candidate against the exact locked root/revision, home/store identity, key registry/files, active/verification/retired constraints, reachable objects, semantic maps/indexes/journal, empty pre-A1 collections, and unchanged physical root. Old referenced keys remain verification-only, key loss or post-reconciliation invalidity fails before publication with zero semantic mutation, and crash/restart plus exact retry converge.
4. Issue a public A1.2 `Start` intent against greenfield-certificate-proven `ExpectedAbsent`, then terminate delivery once while `Issued`, once after plan load/removal but before claim, once while `Claimed`, once after initial authority birth, and once after input acceptance but before success is observed.
5. Restart and retry the exact intent; prove missing plan transport is reproduced from the retained payload, pre-application cases safely resume/reconcile, and committed/accepted cases join one immutable result without another namespace, authority revision, participant allocation, or input delivery. Prove payload cleanup occurs only after exact terminal handoff, with crashes/failures before and after the `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit for both present and already-absent transport bytes.
6. Repeat public CLI and real REPL `Attach` and `ResumeOneTurn` against exact revisions, including bounded auto-attach plan production, and prove the same bootstrap-home/workspace/world/descriptor/attach/resume/policy commitments reach application.
7. Reject stale revision, substituted plan, cross-kind ref, payload/hash mismatch, wrong caller/source/target/lineage, wrong bootstrap home/workspace/world generation, missing/copied greenfield certificate, compatibility-derived absence, pre-A1 artifacts, missing key, expired intent, superseded claim, and conflicting replay before authority mutation.
8. Run public world-scoped start → reattach → stop and prove exact session/world binding plus durable terminal truth are unchanged.

The deterministic parked-successor portion of this scenario is:

1. Persist current `Active` / `ParkedResumable` durable authority.
2. Remove all episode-local PID, handle, helper-readiness, and prompt-stream state.
3. Invoke the public prompt-bearing turn or explicit reattach path.
4. Resolve the exact current authority revision/hash and authoritative lineage.
5. Issue and apply `ResumeOneTurn` for the public turn or `Attach` for explicit reattach.
6. Preserve the exact session identity and world binding.
7. Revision-authorize exactly one successor participant and retain immutable input handoff when
   present.
8. Launch the helper/REPL execution episode only after durable application.
9. Retry the exact request and join the committed intent/application without a second successor.
10. Stop the same session and reach durable terminal truth.

Covers: `RG-AUTH-01`, `RG-AUTH-02`, `RG-AUTH-03`, `RG-BASE-01`, `RG-BASE-02`.

### S2 — Receipt, supervisor, obligation, cancel

1. Accept one ephemeral task from its exact pre-terminal B0 acknowledgement, durably claim the
   stream in B2.1 before any subsequent frame, and prove the production path never attempts legacy
   active-task registration. Blocking inspect/wait and narrowly adapted cancel consume exact
   receipt/supervisor truth, and foreground drop does not erase the accepted or supervised work.
2. Continue an exact retained worker, persist its B1 accepted active-run identity, hand the same B0
   stream to B2.1 with no observation gap, and prove the foreground may remain a blocking waiter
   without retaining observation ownership.
3. Normalize one supported provider attention shape before `AgentEvent` construction, emit one
   B3.1 event with exact target/run/thread/class/attention/causation identity, and prove B2.1
   journals it durably and C1 materializes one obligation before the terminal event. Prove an
   ambiguous or malformed attention-driving provider shape fails before emission rather than
   becoming progress or no-attention, and prove host JSON-pointer parsing cannot repair it. For
   every post-ack retained `Event` frame, prove either the typed member reaches the journal/cut or
   the stream and Complete cut fail closed; no event is silently dropped or left untyped.
4. Drop callers and restart the supervisor during both accepted families, then replay duplicate
   frames/events and prove the claim, journal, receipt, obligation, ledger revision, watermark, and
   terminal state remain idempotent. Missing exact terminal truth remains interrupted/nonterminal.
5. Query C1 before and after the exact terminal cut; prove Pending first, then Complete for both
   empty/no-attention and non-empty/attention dispositions without consulting projections. The
   Complete snapshot's ordered event vector must join exhaustively to every retained B2.1 `Event`
   ref in scope, including typed non-attention events.
6. After B2.2/B3.2, receive the accepted receipt before terminal exit and cancel by
   `active_run_id` in the same host turn.

Covers: `RG-EVENT-01`, `RG-RECEIPT-02`, `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`,
`RG-MSG-01`, `RG-CANCEL-01`, `RG-OBL-01`, `RG-OBL-02`, `RG-OBS-01`.

### S3 — World-UAA mediation under narrowed policy

1. Spawn/continue world Codex with write narrowed to `src/parser.rs`.
2. Prove shell and provider-native edit can modify that file.
3. Prove shell, direct edit, and write-capable tool cannot modify `README.md` or escape root.
4. Repeat with broker unavailable and require fail-closed behavior.
5. Join each operation to the receipt's immutable policy hash.

Covers: `RG-UAA-01`, `RG-UAA-02`, `RG-UAA-03`, `RG-POLICY-01`, `RG-POLICY-02`.

### S4 — Host visibility and failure truth

1. Run equivalent writes under host-visible and full-isolation policies.
2. Verify exact visibility/reconciliation semantics.
3. Force a non-zero UAA turn after a permitted write.
4. Prove durable failed active-run state, retained authority behavior, diagnostics, and file visibility outcome independently.

Covers: `RG-SYNC-01`, `RG-WORKER-EXIT-01`, `RG-OBS-01`.

### S5 — Existing gateway carrier preservation and direct Codex adoption

1. Run the existing launcher/consumer regression baseline: validated `GatewayAuthBundleV1`, raw-secret env scrubbing, one-time gateway consumption, and fresh bundle delivery on restart.
2. Resolve a credential source on the host and launch the exact managed in-world gateway without copying the secret payload into a world-visible runtime home.
3. Start direct world Codex with a per-worker Substrate-owned projection that points provider traffic at that gateway; do not seed a host auth file in contract-correct mode.
4. Join the envelope, accepted policy snapshot, projection identity, exact gateway receiver, and non-secret handoff evidence for the same world generation.
5. Prove the UAA child and descendants cannot inherit the secret FD or raw secret and that no secret-bearing runtime-native file is created.
6. Exercise duplicate consume, wrong gateway/world generation, expiry, unavailable handoff, and unavailable gateway adoption; require fail-closed behavior.
7. Exercise the explicitly named compatibility copy mode separately and prove it is logged, has retirement metadata, and cannot satisfy contract-promotion evidence.
8. Inspect logs/traces/manifests for secret payloads, secret-bearing paths, and reusable secret-derived hashes; none may appear.

Covers: `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-CONFIG-04`, `RG-UAA-02`, `RG-UAA-03`, `RG-OBS-01`.

## Closeout rule

An implementation PR may mark a ledger row resolved only when:

1. its owning crosswalk seam has the correct owner and call path for that behavior;
2. the named permanent gate passes on the real path;
3. adjacent resolved baselines remain green; and
4. the evidence distinguishes durable success from transport/process success.

## A1.1d-5R2-2F0-HC empirical closure record

### Environment-inventory correction evidence

The current correction began from clean source branch
`feat/internal-host-orchestrator-world-dispatch-bootstrap` at
`a6c4a5a52c7e33d4efcc18f70dc2b7301ab0fcca` (tree
`17b83d26b3f9a6258d0234461543528c269338bb`), with source remote
`9a3b54edcf1560d50b6aafda7293699fce85c719` and divergence 0 behind / 10 ahead.
Before audit work, the exact 44-file partial harness WIP was preserved on
`feat/preserve-a1-1d-5r2-2-harness-xdg-blocker-6d1e303f` at commit
`8b203b6ef9bda407a1299f8208bdd0ef1a7a1997`, parent the exact source commit, tree
`ba80d26ba2c6a69036c4eb77e212f29e908dc70a`, ordinary patch SHA-256
`6d1e303f29b18f966cc61f710cae6bf84eed03bd890a0bce3e9f8d5c1d31b0d3`, and full-index/binary
patch SHA-256 `ec0d1a091ad324777f4d0447fa36eece6f64d7b883e7a1b6c1562d4283248023`.
Local/remote commit, parent, tree, 44-file manifest, and hashes matched before the source worktree
returned clean to the immutable commit. The earlier donor remains separately preserved with
ordinary patch `5530e58a7289db4e9a62fc9934510b87add71853728a0311af2160328af3cbeb` and
full-index patch `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`.

The actionable correction classifications are `HarnessClosureIncomplete` and
`ControlPackStateMismatch`. The prior exact-name pass found direct
`std::env::{set_var,remove_var}` literals but did not source-close wrapper arguments. Its manual
dynamic-wrapper repair recognized twelve tests, yet treated the settings family as a
`SUBSTRATE_ROOT` dependency without extracting `SUBSTRATE_OVERRIDE_ANCHOR_MODE`,
`SUBSTRATE_OVERRIDE_ANCHOR_PATH`, or `SUBSTRATE_OVERRIDE_CAGED`; it also omitted the sole
`AmbientSelectionGuard::set` callsite and therefore `XDG_CONFIG_HOME`, `XDG_DATA_HOME`, and
`XDG_STATE_HOME`. GitNexus's `cfg(test)` graph had no incoming edge for that guard and could not
substitute for lexical callsite closure. Immutable source proves the XDG values are parent-process
mutations, not `Command::env` projections. Independent correction review then rejected the first
405-call evidence manifest because it omitted `with_store`, `ProjectionEnvGuard::capture`,
`update_world_env`, and the macOS `restore` family; the later 604-row pass also omitted
`install_bootstrap_projections`, `apply_world_root_env`, `export_runtime_config_env`, and four
additional `with_store` families. These were callsite-closure defects, not new environment names.

Two independent methods now reconcile: lexical/static inspection accounts for all 395 direct
parent-mutation primitive calls, including 73 dynamic-key calls; wrapper/callsite plus GitNexus and
source closure resolves the 86-name universe and every dependent test. Exactly 74 names are
parent-mutated and 12 are child-only or read-only. The parent-mutating union is corrected from 518
to 534 tests in the same 35 files. A2 is corrected from 113 to 129 direct tests: 116 additive plus
13 existing-source F0a tests. The prior 518 was 506 same-file/literal results plus twelve disjoint
manual rows. Corrected closure adds eighteen real mutators and removes two false positives from
ambiguous same-file bare-name fanout. The additions are
`authenticated_gateway_projection_ignores_named_ambient_roots_and_uses_account_db_codex_home`,
`codex_auth_projection_errors_redact_committed_account_home`,
`update_world_env_sets_enabled_flags`, `update_world_env_sets_disabled_flags`, and the fourteen
`with_test_mode`-only PTY tests enumerated in `02`. The removed non-mutators are
`b1_task_acknowledgement_rejects_nonstart_eof_mismatch_and_repetition_before_persist` and
`b1_retained_acknowledgement_joins_exact_request_and_rejects_terminal_or_drift`;
the two settings tests `resolve_world_root_respects_env_when_no_configs` and
`resolve_world_root_env_overrides_global_config` were already present but now map to their exact
three override names.

GitNexus reports the newly allowlisted Codex-auth test and each of the fourteen newly closed PTY
test symbols LOW with zero callers, zero affected processes, and zero affected modules. The shared
`with_test_mode` helper retains the prior HIGH adjacency result. These results authorize only the
exact test/harness boundaries in `02`; production PTY classification, broker behavior, and gateway
auth resolution remain frozen.

Full closure also reclassifies already-listed `SUBSTRATE_SHELL` as parent-mutated.
`execution/routing/dispatch/tests/host_replay.rs::async_repl_host_commands_record_replay_context`
passes it to `execution/routing/test_utils.rs::{set_env,restore_env}`, and
`crates/trace/src/span.rs::SpanBuilder::new` is the overlapping stable reader. The test's three
early returns precede manual restoration, while `set_env` captures `Option<String>`; current
protection is therefore early-return/unwind unsafe and loses a prior non-Unicode value. The test
and file were already in the 116-test A2 manifest and 35-file union. The future migration is exact
`OsString`/absence RAII under `UnifiedProcessStateLock`; the trace reader remains frozen.

The two macOS tests already hold `world_env_guard`, but their `snapshot`/`restore` helper captures
only `Option<String>` and restores only `SUBSTRATE_WORLD` and `SUBSTRATE_WORLD_ENABLED` even
though production-frozen `update_world_env` changes six names. Their exact future allowlist is
`execution/platform/macos.rs::platform_tests::{snapshot,restore,update_world_env_sets_enabled_flags,update_world_env_sets_disabled_flags}`;
all six prior values or absence must be restored exactly under the same coordinator. GitNexus
reports both helpers LOW with two direct test callers and zero processes, and both tests LOW with
zero callers/processes.

The XDG negative-authority semantics are frozen: conflicting ambient XDG roots remain installed;
authenticated A and the account-database Codex home remain authoritative; ambient XDG values gain
no configuration, credential, or path authority; all assertions remain; and exact prior
`OsString` values or absence are restored. The exact six corrected names and their test-only
file/symbol/test allowlist are binding in `02`. All six belong to the existing A2/E1
`UnifiedProcessStateLock`; no XDG-specific lock, production owner, side table, reader edit, or
behavior change is authorized.

The prevention rule is mechanical: direct primitives and wrapper definitions/callsites are
separate inventories; literal, constant, array/table, loop, and parameterized arguments must all
resolve; dynamic names fail if unclassified; parent mutation and child-only projection are checked
separately; and every mutating test plus overlapping stable reader must map to an exact migration
or isolation disposition. The deterministic validator is evidence-only outside tracked repository
state and authorizes no seventh control-pack file.

The validator binds the normalized 395-call primitive, 1,005-call resolved mutation, and 534-test
manifests at SHA-256
`df0254488f37d4f53f05c81bbbd47c05bbbe410496d465827daade80922e0f4a`,
`350ccd1876ecc07cced81df89b523e3d24f9674f5dc6a5e33f052d65f4489772`, and
`de7e2bc43c0fcab74096dc272340726c919aeb72004fd0208ab5b4dffb802f4a`, respectively. It also
freezes the reproduced 506-test defective scanner at
`a9fbbe89128a2cbe9c341a0ec78a9fb256840d5ef65ad78f47e9d404e20cafed` and 43 dynamic mutation-sink owners at
`133e28a495e5be69016fb7f6483018cfb406b44c8ca5e2d8779fdf82ded0c66c`, parses fixed projection
and local-key tables from source, and runs negative perturbation checks. The rejected 604-row
manifest remains historical evidence, not a completeness claim. The validator parses the 35-file,
116-test, and 13-test canonical lists; verifies all 129 named source test
symbols exactly once; and checks the host-replay early-return, routing-helper capture, and
`SpanBuilder::new` reader closure. Counts without exact normalized manifests are not sufficient.

### Preflight and preservation

The earlier F0/F0a/F0b preflight remains preserved as historical evidence; the current correction
preflight and 44-file preservation are recorded above. That earlier preflight was:

- source branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`; local HEAD
  `06c928443a93579899e5e5827b151f530e1be933`; tree
  `dfe15d93366655e9855bca40ad0607887f545637`; upstream/remote
  `c079aeed748120cff9b03079e996d00109a05800`; divergence 0 behind / 10 ahead; clean index,
  worktree, and untracked set;
- published F0b docs commit `c079aeed748120cff9b03079e996d00109a05800`, tree
  `905cf110f648188b1a2e9a0087b4ffbb4c89e168`, subject
  `docs: authorize deterministic renderer test isolation`;
- replay preservation `feat/preserve-a1-1d-5r2-2e-replayed-f0b-docs-c079aeed` resolves locally and
  remotely to `06c928443a93579899e5e5827b151f530e1be933` and therefore serves as the immutable start
  checkpoint without a redundant branch;
- broad candidate branch `feat/preserve-a1-1d-5r2-2f0-f0a-broad-blocker-9978655e`, commit
  `78353f3383fdd80c176fe5638328c10f2805dcf0`, ordinary patch
  `577a34da903992d0566fa51d164410caeca98592dd486f5c1d5520e34baaa5a1`, full-index patch
  `9978655e0f4095880c49d863a93b8fc99fb2b769afd42554c758b3970103f710`;
- post-fork-remediation branch
  `feat/preserve-a1-1d-5r2-2f0-f0a-post-fork-remediation-9e012c68`, commit
  `07ce368cc02d5a4e80db9c1ca80efee93009d064`, ordinary patch
  `5530e58a7289db4e9a62fc9934510b87add71853728a0311af2160328af3cbeb`, corrected full-index
  patch `9e012c68ea33107443cd38f2051aa40aae9f0f03d5c6abb4c630d418b0d7dee2`;
- exactly ten Routes A–E runtime commits were ahead; no F subject, change, or candidate restoration
  existed. Neither preserved candidate was checked out, restored, merged, cherry-picked, or replayed.

Refreshing GitNexus changed only generated symbol/relationship/flow count lines in `AGENTS.md` and
`CLAUDE.md`; those exact generated changes were restored before evidence collection. Recorded
status: `GeneratedIndexDriftRemediated`.

### Forced overlap matrix

All F0-HC probe tests named below were temporary, uncommitted diagnostic hooks. They used barriers
to force the precise overlap, exposed no values or secrets, and were removed exactly before the
documentation worktree was created. “Serial” is the same-process ordered control; “separate” is a
fresh-process control.

| Pair / exact diagnostic test | Forced same-process result | Serial control | Separate-process/control result | Normalized signature and classification |
|---|---:|---:|---:|---|
| `SUBSTRATE_FORCE_PTY` mutator → `is_force_pty_command`; `f0hc_env_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_ENV_OVERLAP: stable reader observed concurrent SUBSTRATE_FORCE_PTY mutation`; `TestIsolationDefectConfirmed` |
| CWD mutator → `WorldRootSettings::effective_root`; `f0hc_cwd_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_CWD_OVERLAP: stable reader observed concurrent process CWD mutation`; `TestIsolationDefectConfirmed` |
| routing trace owner → second global trace owner; `f0hc_trace_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_TRACE_OVERLAP: first owner wrote through the second test's global trace output`; `TestIsolationDefectConfirmed` |
| mutated PATH/timeout → `REPORT_CACHE`; `f0hc_socket_cache_persistence_signature` | 20/20 failed after exact env restore | cache-refresh control 0/20 failed | `systemctl_timeout_is_fail_fast` fresh process 0/10 failed | `F0HC_SOCKET_CACHE_PERSISTENCE: cached override survived exact environment restoration`; `TestIsolationDefectConfirmed` |
| first private retry hook → second hook install; `f0hc_private_retry_hook_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_PRIVATE_HOOK_OVERLAP: first test's hook was replaced by a concurrent test`; `TestIsolationDefectConfirmed` |
| repeated session fixture ID → `WorldDispatchConcurrencyTracker`; `f0hc_dispatch_tracker_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_DISPATCH_TRACKER_OVERLAP: reused fixture session ID consumed another test's cap`; `TestIsolationDefectConfirmed` |
| global broker policy mutator → stable `policy_mode` reader; `f0hc_broker_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_BROKER_OVERLAP: stable reader observed another test's global broker policy mutation`; `TestIsolationDefectConfirmed` |
| first `ActivePtyGuard` owner → second `ACTIVE_PTY` registration; `f0hc_active_pty_same_process_overlap_signature` | 20/20 failed | 0/20 failed | 0/20 failed | `F0HC_ACTIVE_PTY_OVERLAP: first test's active PTY control was replaced by another test`; `TestIsolationDefectConfirmed` |

The matrix proves eight new interference pairs. The already-established rows remain binding: the world-socket pair failed 12/20 clean parallel
runs and 0/10 serial runs, while the preserved candidate proof passed 100/100 parallel and 20/20
serial; the exact HOME pair failed 20/20 forced same-process runs, 0/10 stable serial runs, and
passed in separate processes, with three other unannotated HOME mutators independently reproducing
20/20; stdout reporter overlap passed 376 and failed 124 of 500, while isolated, identical-neighbor
serial, and separate-process controls passed 100/100 and pretty reporting passed 99/100. Its exact
captured contamination was `.[codex] task_progress: fields=alpha, beta, gamma (+1 more)\n`.

Static/source lifetime closure added the exact tenth no-await case
`wait_for_fork_child_durable_publication_keeps_stop_transport_timeout_short_once_child_is_visible`.
During the broad evidence wall, predictable private stop paths also left unowned inactive socket
nodes, including `sessdispatch-ashmember.sock`, and a later bind reported `Address already in use`.
This corroborates termination/path ownership; it does not authorize product socket cleanup.

### Bounded clean-code broad evidence

Exactly one clean-code default-parallel wall and one clean-code serial wall were run. They are
correlation evidence, not final closeout:

| Wall | Outcome | Retained diagnostic/semantic evidence |
|---|---|---|
| `cargo test -p shell --lib` | 1,263 discovered; `1113 passed / 150 failed / 0 ignored`; 165.43 s | The retained partial output exhibits legacy `StateStore`/authority-root interference, fixed private stop-path collisions, authority-layout absence, downstream cascades, and one environmental `/tmp` `ENOSPC`. All 150 names are recoverable from the same-run fragment union, but complete panic output survives for only 37; no complete normalized-signature artifact was retained, so this evidence is diagnostic only and is not an exact transition matrix. |
| `TMPDIR=<private disk-backed root> cargo test -p shell --lib -- --test-threads=1` | 1,263 discovered; `1202 passed / 61 failed / 0 ignored`; 388.62 s | The 89-count improvement proves a parallel-sensitive set. The retained 61 normalized into persistent authority-layout/legacy-writer-disabled-or-no-active-state failures, predictable private stop-socket collision, and their async/session cascades. Representative exact retained names were `continue_world_worker_dispatch_returns_real_typed_internal_outcome`, `detached_stop_world_worker_closeout_availability_recheck_accepts_parked_truth_without_sanctioned_owner`, and `prepare_member_runtime_startup_for_descriptor_accepts_parked_detached_orchestrator_parent`. |

Those representative serial failures were also run in fresh processes with unique `TMPDIR` roots.
They remained failures because the tests still consumed the real ambient HOME or a fixed `/tmp`
socket contract; separate-process scheduling alone does not isolate persistent parent paths. No
user-owned authority state was deleted or modified. The evidence therefore distinguishes two
classes: parallel process-global overlap and persistent predictable fixture/path leakage. Neither
class is a production concurrency or lifecycle regression, so `ProductRegressionDecisionRequired`
was not triggered.

Rows proved concurrency-safe include child-only command environments, child-contained umask,
owned descriptors, keyed config/policy caches, immutable OnceLocks/regexes/instance values,
unique TempDir listeners and kernel-selected ports, private-root cross-process lock/CAS tests,
readiness-established protocol timeouts, awaited abort paths, and joined threads/children. The sole
deferred row is production signal/Ctrl-C/SIGWINCH lifecycle, owned by the production PTY/signal
lifecycle packet. Source call closure proves no shell-library unit test calls `execute_with_pty`,
so `initialize_global_sigwinch_handler_impl` and its background thread are never installed in this
binary and cannot perturb this wall.

At that audit checkpoint, no fix or deterministic F comparison baseline existed and F had not
started. The later preserved candidate supplied the focused and three-parallel/one-serial proof
wall frozen in `03` and `04` without completing harness closeout.

The environment-inventory correction itself performed no runtime edit. The exact combined
candidate has since been restored, identity-checked, review-cleaned, canonically closed out, and
preserved. F remained unstarted and was the next packet at that historical checkpoint, before
renewed R2-2 integration closeout, R2-3, R2-4, and R3. The completed-F ledger below supersedes that
next-task status.

## A1.1d-5R2-2F0 historical parallel artifact recovery and authority correction

### Bounded recovery result

The terminal classification is `HistoricalParallelArtifactUnavailable`. One bounded, read-only
inventory exhausted the plausible retained evidence stores; it did not rerun historical code,
reconstruct names from source, combine different runs, or select a convenient later wall.

| Evidence location searched | Result |
|---|---|
| Saved packet checkpoints and current evidence under `/home/spenser/.codex/visualizations/2026/07/{20,21}` | Final-candidate walls, final manifests/fingerprints, and inventory validators exist; no complete historical `1113/150` parallel normalized-signature artifact exists. |
| Prior session transcripts under `/home/spenser/.codex/sessions/2026/07/{20,21}` | One authenticated historical execution was found. Its same-run fragment union yields all 150 names, but truncation leaves complete panic output for only 37; details follow. |
| Repository, worktrees, refs, and commit history under `/home/spenser/__Active_code/substrate` | Source commits and trees are retained. No tracked broad-wall log, differential name/signature manifest, or archived failure artifact exists in the searched history. |
| Preserved local case logs under `/home/spenser/a1-f-case-*` | Complete serial and later broad walls exist, but none has the historical parallel `1263/1113/150/0` identity. Later parallel results include `1206/57`, `1203/60`, `1205/58`, and `1204/59`; they are not substitutes. |
| Explicit temporary artifacts and manifests under `/tmp`, including `a1-f-case-*`, `substrate-f0f0a-walls.*`, `substrate-a1-f0f0a-walls.*`, and `f0hc-*` | Focused logs, inventory data, and later 1,279/1,280-test walls exist. None is the authenticated historical parallel artifact, and no matching complete name/signature manifest exists. |
| Build outputs under `target` and the retained case worktree target | Cargo fingerprints and binaries only; no historical broad-wall output or differential manifest. |
| `/home/spenser/.bash_history` and available shell-history stores | No retained complete output or artifact path. |
| Current application terminal transcript | No terminal session was attached. |
| GitHub Actions runs and artifacts for 2026-07-18 through 2026-07-22 and the relevant source commits | No branch/commit run or Actions artifact retained the historical wall. |
| Bounded content search under `/home/spenser` for the exact aggregate and representative historical failure material | Found the complete serial artifact and the truncated session transcript only; no complete parallel artifact. |

The authentic parallel command was `cargo test -p shell --lib`, run from a clean source at commit
`06c928443a93579899e5e5827b151f530e1be933`, tree
`dfe15d93366655e9855bca40ad0607887f545637`. The retained session file is
`/home/spenser/.codex/sessions/2026/07/20/rollout-2026-07-20T19-43-44-019f81e9-ff42-7ff0-a8a8-296d924a02fb.jsonl`,
SHA-256 `23e8400a26334d3561010f54b71797f06839f4d26f296ad709a675729f9e7e6e`, and records execution
session `94311`. Its final retained wrapper fragment has SHA-256
`c0a90bbe63e99753be61b50cc8f7c10377541243d9c3369ae3251be7aa9b0df2` and the exact aggregate
1,263 discovered, 1,113 passed, 150 failed, and 0 ignored.

That transcript contains an explicit `…2200 tokens truncated…` marker inside a panic body. It
retains 38 complete panic headers and 137 complete names in the final-summary tail; 25 names
overlap, so the same-run union is exactly 150 unambiguous failure names. Only 37 of those headers
retain complete panic bodies sufficient to derive normalized signatures, leaving 113 names without that
semantic material. The commit/tree, command, mode, counts, names, and transcript integrity are
authenticated, but the complete normalized-signature payload is absent. A digest without its input
manifest would not repair that absence. Rerunning the commit now would produce a new sample from a harness
already proven nondeterministic and is not historical recovery.

### Corrected semantic and concurrency authority

The complete serial artifact
`/home/spenser/a1-f-case-b-serial-corrected.F6aLQa/output.log`, SHA-256
`8cf7109d02d738c89bf446d6810f7e06d0140c681da803b88ba4d8adeda13bc7`, is historical semantic
authority. It contains 1,263 discovered, 1,202 passed, 61 failed, 0 ignored, complete failure names,
and sufficient output for normalized signatures. Its exact transition matrix is:

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

All 16 `FailToPass` rows have causal audits in the saved differential evidence. All 17 `NewPass`
rows are the added authorized tests named in `02`. Deterministic historical-serial and final test
listings prove zero removed and zero renamed/substituted tests. Arithmetic closes exactly:
`1202 + 61 = 1263`, `1202 + 16 + 17 = 1235`, `45` final failures, and `1263 + 17 = 1280`.

Final concurrency authority is the exact final candidate's three parallel walls and one serial
wall. Each discovered 1,280, passed 1,235, failed 45, ignored 0, and produced identical failure
names and normalized signatures. The retained wall hashes are:

| Wall | SHA-256 |
|---|---|
| parallel 1 | `89e8805528ad91ccc13153459cf79c7533e12e9036c29118f6a170bb12400e58` |
| parallel 2 | `f956cbaf4f575b99bb9d3bdd7418eb08438f8897464798572c8e2b880ccea68e` |
| parallel 3 | `7965550a83becfcf252a1a145c1e5ce7138836863d9e46e27d361db33b771fa7` |
| serial | `4d4c9cf00cf05e6f58129ae1b8752505dd8cfeb9dd9d8efb79e6d515eb085718` |

The different raw-log hashes reflect wall-local output while the parsed counts, complete
failure-name sets, and normalized signatures agree exactly. The final deterministic test list has
SHA-256 `a7b795a76c5ed7f158fa0b181facc701b87a4c1d141c6d1909cea1c3f62fe022`; its normalized test-name
list has SHA-256 `7aa03922dcc727ea8d67e4084ada3d93da926d95b95eb4f292776e55fa33e2b5`.
Every wall's failure-name set has SHA-256
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; every normalized-signature
set has SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
The exact 17-test `NewPass` list has SHA-256
`490d41b6025f3a6a09e58dfa7fcd5b739b5fbf82ea00cf9e150d9c98520fae5a`; the exact 16-test
`FailToPass` list has SHA-256
`dafcc5a0417ed32917fcb0d81ff028f126f8acd9c94c9c08640cc48be9ab1801`.

The historical parallel aggregate remains useful diagnostic evidence of prior interference, but
it is not transition authority. This record does not claim that the final 45 failures belonged to
the historical parallel set, that exactly 105 named historical parallel failures became passes,
a historical parallel `PassToFail` count, or any historical parallel name/signature comparison.

### Canonical candidate preservation, proof, review, and stop boundary

The exact final candidate was committed without modification and remotely verified on
`feat/preserve-a1-1d-5r2-2-harness-final-baseline-blocker-7ab220a7` at commit
`86ed6f5620787121b1c2e5b033ee8d6f9ff369d3`, parent
`1e1be221ca8a9f4e94af93fbda7bda6e94901d01`, tree
`f6480f3986d43bb41e5387fa1ba5b68ae53f598b`. It changes exactly 45 files with 5,630 insertions
and 2,104 deletions. The exact manifest, per-file fingerprints, ordinary patch, and full-index
patch are respectively:

- `b9e3a44dd671409f66e2d62d48cb494ab147069a74ed71f2308e031f56372ae6`;
- `41cb1a325add4efd8198872b456c3ae0fba73f8c4943e4b59c06e2bc76d8b479`;
- `7ab220a715f4ae3389be314da2e6b0e614fcffb7a92166f04fde999570801861`; and
- `adc1c5952e2ac4cc97881a1bf4df8e24b00d4d4ba337e692bf21965151d5c0c4`.

That exact patch was restored onto corrected source head `4eca8773bf1cfaac1e4fcb7ba09b8a06c310ab7c`
and committed without byte changes as `770a6a9de9f537f7bc179c75421abbc3fff05b8d`, tree
`61fdd2e9476f1ce3e041720ce106f7c3427895be`. The dedicated review-clean runtime preservation branch
is `feat/preserve-a1-1d-5r2-2-harness-review-clean-7ab220a7`. The two whole-tree hashes differ only
because the corrected source base contains the six published differential-authority documents;
the 45-file patch and all four candidate hashes are identical.

The corrected inventory is exactly 86 names, 74 parent-mutated, 12 child-only/read-only, 395
primitives, 73 dynamic calls, 43 sinks, 1,005 resolved callsites, 534 tests across 35 files, 38
resource rows, and zero unresolved dynamic rows. Its validator and negative wrapper-callsite,
migration-row, unresolved-dynamic-key, and parent-as-child perturbations all behave correctly. All
38 resource dispositions are implemented or retained.

Focused proof passes for HOME/socket overlap; stdout/stderr renderer isolation; XDG/account-home
negative projection; environment/CWD restoration; trace/report-cache/retry-hook/tracker/broker and
`ACTIVE_PTY` isolation; deterministic fork publication; async termination confirmation;
subprocess isolation; exact absence and non-Unicode restoration; panic, poison, nesting, and
reacquisition; and child failure propagation. Shell and workspace all-target checks, relevant
Clippy with `-D warnings`, `cargo fmt --all -- --check`, and `git diff --check` pass.

The 16 `FailToPass` rows are causally assigned to seven detached-availability fixes, two
deterministic fork-publication fixes, six stop-dispatch fixes, and one hidden-owner fix. The 17
`NewPass` entries are exactly the authorized tests listed in `02`. No test was removed, renamed,
substituted, weakened, or newly ignored.

Environment closure, renderer/injection, and lifecycle/subprocess reviews
`/root/final_review_env_closure`, `/root/final_review_renderer_injection`, and
`/root/final_review_lifecycle_subprocess` are CLEAN for the exact patch. The prior blocked
containment review is not reused. Fresh isolated read-only reviewer
`/root/final_containment_corrected_authority`, task
`019f8681-7957-7cc3-88fa-37ab3ad2fc87`, returned CLEAN under the corrected authority contract.
GitNexus change detection reports one generated MEDIUM process label, resolved by source closure to
test-only `AuthorityEnvTestTempDir::new`; there is no changed production execution flow. The sole
production hunk is the authorized mechanical F0b writer delegation with byte-identical output.

This is an evidence-authority correction, not a waiver. `PassToFail`, `FailToChangedFailure`,
`Removed`, `RenamedOrSubstituted`, `NewFail`, and `NewIgnored` are satisfied zero gates. F0, F0a,
F0b, and F0-HC are complete. The runtime changes no product or user-facing behavior, promotes no
seam, and does not start F.
The binding sequence is:

`Routes A–E` → `F0/F0a/F0b/F0-HC complete` → `F` →
`renewed R2-2 closeout` → `R2-3` → `R2-4` → `R3`.

At that historical F0 closeout, F was unstarted and the next packet was **A1.1d-5R2-2F —
Authenticated world-deps and truthful doctor composition**. The completed-F ledger below
supersedes that next-task status.

## A1.1d-5R2-2F readiness-boundary evidence ledger

The prior line records the historical F0 closeout state. The current checkpoint is F1/F2 complete,
F3/F4 blocked, and F incomplete.

| Evidence | Exact result |
|---|---|
| Source base before local F commits | `7d642606205b4f0f93e157fbde27507a7c86e655` |
| F1 | `eae02af959f0b7066015bb242ffa45fc7a01d591`; tree `7bd7e8b5ca2d563b2991b4fea62ca4ea26266b0c`; message `feat: bind authenticated world-deps context`; ordinary patch `829b8a19c0d2deb53af0b9c2f277e7eab53b5001f41c59ebc3e7075c1a3e8b54`; full-index patch `f7c16056ce5b379848203b066bffec37ef9f6f46fdcb2753af6af4feac01886f`. |
| F2 | `d30d8cec764e2338fb48475747733091d3af22bf`; tree `a6161c7664fa6ca1173302dd3570b5d8d7e09ecc`; message `feat: propagate authenticated world-deps scope`; ordinary patch `42f5278a6cab739c9d5b5e5ca7ee2c3a960ba8d8197e2f772192a65e391ed31b`; full-index patch `b65e3146e870ac7579e43508fb87b4c1bdc63afae96bf67473b75a7285e8bfe8`. |
| F1/F2 preservation | `feat/preserve-a1-1d-5r2-2f-f1-f2-d30d8cec` points to exact F2. |
| Blocked F3/F4 preservation | Branch `feat/preserve-a1-1d-5r2-2f-f3-f4-blocked-7224dd30`; commit `a343f0796d19d66c168c5bb2797856710cff5708`; parent exact F2; tree `a377daa6f41693454e38c39163cc9895bbf3e828`. |
| Blocked manifest | `world_deps/mod.rs`; `world_deps/surfaces.rs`; `world_enable/runner.rs`; `world_enable/runner/provision_deps.rs`; `execution/routing.rs`; `execution/routing/dispatch/prelude.rs`; `execution/routing/dispatch/world_ops.rs` (all beneath `crates/shell/src/`). |
| Blocked fingerprints | Manifest `23a1dc1566d395ae5e606ebb4bae0d7e7d4c9c3f976958d7e3c7ce8a094f0cc8`; per-file aggregate `a8399a8a0989c675b05788fb12f2f91105cfa1e15ac0dca911df918594033edd`; ordinary patch `7224dd30c04e5fcd85913bfdddb62deb497f0b3eaba416b2f689d392f47b9b0c`; full-index patch `76fb0c5c183880dca8f31576647c8e9372d1ad026e2a731b48d5ef2642ef0a22`; 613 insertions/128 deletions. |
| Canonical harness baseline | Exact base wall: 1,280 discovered, 1,235 passed, 45 failed, 0 ignored. F1/F2 add three passing tests without changing the 45-failure set. |
| Generated index drift | `GeneratedIndexDriftRemediated`: GitNexus refresh changed only generated `AGENTS.md`/`CLAUDE.md` count lines; both exact generated changes were restored before preservation. |

### Reviewer finding and correction disposition

The preserved candidate's prefix-based ambient environment copy is rejected. It can disclose
credentials or request material and can pass authority/backend/policy selectors from B beside an
authenticated A request. The candidate also bypasses the canonical readiness owner and therefore
cannot provide correct Linux service activation/spawn behavior. These are security and ownership
defects, not permission to duplicate lifecycle logic.

GitNexus reported `ensure_world_service_ready` **HIGH**, with three graph-visible direct callers and
three affected existing process groups. Manual source closure found two additional direct uses: the
persistent-session WebSocket setup and Linux initialization's function-pointer probe. Impact on
each existing caller label, the additive F builder, `run_world_command_for_deps_at`, and
`execute_with_profile` was LOW with zero upstream impacts. The HIGH owner is approved only for
private extraction plus compatibility delegation; all existing callers remain unchanged.

The exact corrected disposition is:

- one readiness owner in `world_ops.rs`;
- one private explicit-target Linux core, with no copied probe/activation/stale/spawn/timeout logic;
- unchanged no-argument compatibility entry point and callers;
- fixed product socket and immutable installed-product service posture for authenticated F;
- exact seven-entry generated guest environment and zero ambient forwarding;
- authenticated failure before readiness side effects;
- exactly two additive-builder consumers;
- no production capability or policy change, no non-Linux change, and no seam promotion.

F3/F4 must be rebuilt from clean F2 under this contract; preservation bytes are not restored or
approved wholesale. F5 and final F walls remained unstarted. Renewed R2-2 integration closeout,
R2-3, R2-4, and R3 remained blocked. The exact next task at that historical checkpoint was
**Resume A1.1d-5R2-2F3/F4 under the corrected readiness and environment contract**. The F5-PD and
completed-F ledgers below supersede that next-task status.

## A1.1d-5R2-2F5-PD evidence and decision ledger

This latest ledger supersedes the live status/next-task conclusion above. F3/F4 were rebuilt from
the corrected boundary and are complete. F5 then stopped before closeout because its mandatory
nested child reached the active public Doctor path. No F5 implementation byte is accepted.

### Verified starting state and retained evidence

| Evidence | Verified result |
|---|---|
| Source branch before preservation | `feat/internal-host-orchestrator-world-dispatch-bootstrap`; HEAD `7802c44198625ea6849933140c390900ebe84190`; tree `21395f927779a49dce2b692a7f407c72a9964105`; parent `54a5ce663cf2724b69b0c124acf2c2758ff622dd`; source remote `2a982292de794dd0e4c4e00f151fe83dfb5b0b63`; divergence 0 behind / 15 ahead |
| F3 commit | `54a5ce663cf2724b69b0c124acf2c2758ff622dd`; tree `db4a51b153bc93b258d25ce4289c4ef3d477a646`; subject `feat: add authenticated world-deps request builder`; ordinary/full-index patch SHA-256 `c9ec972b7098ce8b5d633f7793678fccbb7ffebda4d2769bc3119a4791990c61` / `3db155ae7e308f50e9f9c80b74ea5ace50b2fcc6e8ffd5cf34e0c288955343d2`; stable patch ID `a8ad65e71975b236f082f4abeb3b0fdcd37f216e`; three files, 978 insertions, 51 deletions |
| F4 commit | `7802c44198625ea6849933140c390900ebe84190`; tree `21395f927779a49dce2b692a7f407c72a9964105`; subject `feat: propagate authenticated world-deps execution`; ordinary/full-index patch SHA-256 `179e67b16316ae62683085cf17d5e4e56c30fec7c51ca12b4edeb47d78d36e65` / `3996fba97a44fc20c2e95d9395971b21172ceeca963a8d235b2ab7a57579fd2a`; stable patch ID `016e4edb9ecd21150a4e788ab93b7ba0dc0df4c6`; six files, 741 insertions, 114 deletions |
| Clean F4 wall | 1,300 discovered / 1,255 passed / 45 failed / 0 ignored; failure-name SHA-256 `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; normalized-signature SHA-256 `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` |
| Attempted F5 focused evidence | Decoder tests: 13 passed. Shim-doctor integration: 17 passed / 1 failed. These are diagnostic evidence only and do not waive the boundary stop. |
| Boundary adjudication | Fresh isolated reviewer `/root/f5_boundary_adjudication` returned exact status `CrossDocumentChangeRequired`. |

### Exact blocked-candidate preservation

The original source worktree contained exactly four unstaged modified files and no staged or
untracked path. They were committed without byte changes to remote branch
`feat/preserve-a1-f5-blocked-candidate-20260722` at
`c4b93506480ae5e9708417067980e08e0e754b7a`, parent
`7802c44198625ea6849933140c390900ebe84190`, tree
`03bec2f517866e5213b8a0bed9ad99058c25ebc3`. Remote ref, parent, manifest, per-file bytes, ordinary
patch, full-index patch, and statistics were re-read and matched. The source branch was then
returned clean to its exact original HEAD; the candidate was not restored.

| Candidate file | SHA-256 |
|---|---|
| `crates/shell/src/builtins/shim_doctor/report.rs` | `e41ba1e79b451280af9135c7bfdb8523250848850d929957260a02b891cf92f5` |
| `crates/shell/src/builtins/world_deps/mod.rs` | `89283c62780c9fc58ff4d22cfaf10abf990ce4b039238b742eb343ce505e958c` |
| `crates/shell/tests/shim_doctor.rs` | `13b06c4db570193f68ecb6e72d37062474f61ee59e4a9610f261d76a4595069a` |
| `crates/shell/tests/shim_health.rs` | `e81005319e737c0ed2fcd50c26793aa28cff96496da807ec4fbadaf056df6be5` |

Ordinary patch SHA-256 is
`81151bdad6aa9e7dd1963b7b22f1f3dad66b46399174ff26bfa33ec08859e03a`; full-index/binary patch
SHA-256 is `7805a28defc11866902fadaaf99d7c752800c5aab257b9c927187920bd6cf908`.
Statistics are exactly four files, 1,326 insertions, and 65 deletions: `report.rs` 882/60,
`world_deps/mod.rs` 12/0, `shim_doctor.rs` 256/3, and `shim_health.rs` 176/2.

The preservation operation did not inspect, modify, move, or remove
`/home/spenser/.cache/substrate-user-home-archive/20260722-084820-substrate`.

### Control-pack contradiction and source decision

The pre-correction control pack required the no-fixture branch to run the exact existing child at
`00-README.md:292-301`, `02-seam-crosswalk.md:292`, `03-phase-slice-map.md:531-541`,
`04-contracts-and-gates.md:591-594,808-815`, and
`05-debug-regression-ledger.md:498-519`. It simultaneously required a side-effect-free/read-only
diagnostic at `01-target-architecture.md:395-409`, `03-phase-slice-map.md:655`,
`04-contracts-and-gates.md:1089-1099`, and `05-debug-regression-ledger.md:389`, while freezing the
platform/service/lifecycle owners at `01-target-architecture.md:414-420`,
`03-phase-slice-map.md:658,1231-1236`, and
`04-contracts-and-gates.md:820-826,1097-1103,1115-1122`.

Source closure proved the child was public `world doctor --json`: CLI routing entered the public
Doctor arm; Linux doctor gathered socket-activation state and connected the socket; the client
called `/v1/doctor/world`; world-service selected/mounted and mutated an enumeration probe; and a
404 invoked the `/v1/execute` filesystem fallback. A Unix socket connection can activate
world-service. Therefore the old pair of requirements could not both hold. Deleting the child or
synthesizing unavailable solely in the parent would have broken the separately required hidden
argv carrier/child-process proof.

On Linux, the earliest bounded branch is after
`routing.rs::decode_and_bind_unix_install_bootstrap_context` succeeds and before
`install_bootstrap_projections` and `ensure_substrate_home_deps_scaffold_for_context`. That branch
preserves a real authenticated child while making every active downstream owner unreachable. No
new lifecycle owner is necessary, so `PassiveDiagnosticArchitectureDecisionRequired` does not
apply. The existing fail-closed report can represent the outcome without a schema change, so the
schema-specific stop does not apply either.

Three review defects were source-closed before publication. First, whenever Linux World-enabled
production reaches `gather_world_doctor_snapshot`, it may not select
`A/health/world_doctor.json`: F5-PD removes that branch and runs the authenticated child, while
Linux value decoding becomes `cfg(test)`-only. The frozen World-disabled `build_report` branch
continues to return `disabled_world_doctor_snapshot` without a child or fixture lookup. The fixture
is therefore Linux test evidence rather than product authority. Non-Linux fixture/public-child
compatibility remains behavior-frozen and cannot satisfy proof. The separate world-deps fixture
remains F5-owned and outside F5-PD. Second, the existing Linux snapshot decoders cannot be frozen
because they infer success from missing `ok`/exit 0 and retain arbitrary details/stderr. F5-PD
authorizes Linux `snapshot_from_command`, the test-only value decoder, and one private validator to
require exact schema 1, compile-time platform, explicit false status, duplicate exact A identity,
exit 4, empty stderr, no extra or forbidden key, and bounded no-payload unavailable/incoherent
construction. Their GitNexus impacts are LOW at one direct/one total caller each, zero attributed
processes, and the Shim-doctor module only.

Third, the live Linux `run_json_subcommand` parses raw stdout directly into
`serde_json::Value`, whose last-wins duplicate-key behavior could collapse conflicting B-then-A
identity or `ok` fields before the validator observed them. The exact correction authorizes only
that symbol's Linux raw-stdout parse expression plus three private exact wire structs and one
private decoder using `deny_unknown_fields`; all duplicate/unknown top-level or nested keys reject
before value construction. `run_json_subcommand` is LOW 0/0. Its child construction, carrier and
projection transport, signature, process/exit/stderr behavior, selected-access redaction, and all
non-Linux code remain frozen. Regression 14 now includes conflicting top-level/nested duplicate
keys and B-then-A identity ordering. Frozen `build_report` is LOW one direct/five total and
`disabled_world_doctor_snapshot` is LOW one/six. The test-inclusive `maxDepth=5` lifecycle totals
are `ensure_world_service_ready` CRITICAL 4 direct/10 total and `socket_activation_report`
CRITICAL 6/12; neither is authorized for edit or passive call.

The hidden discriminator is carried by the existing `WorldAction::Doctor` variant, not the
top-level `Cli` struct. This source-closed choice leaves `auto_sync.rs::cli_for_auto_sync`
untouched. It is one additive hidden Rust enum field and an ordinary-path fail-closed guard in
`handle_world_command`; normal public CLI grammar/help/behavior remain unchanged. GitNexus reports
LOW 0/0 impact for both `WorldAction` and `handle_world_command`.

### Security decision

The threat boundary is the hidden child argv carrier versus inherited process environment and
ambient B. Spoofing is blocked by carrier commitment and current-principal validation; tampering
and mixed identity fail closed; duplicate/unknown raw JSON fields reject before lossy value
construction; elevation through environment-only mode selection is impossible;
information disclosure is bounded by rejecting secret/request/carrier/preimage markers and not
retaining rejected payloads; active endpoint/probe denial-of-service and lifecycle effects are
removed by the pre-bootstrap branch. No logs, traces, or side tables are created by passive mode.
Only the already-public selected-prefix/commitment diagnostic identity may be emitted.

The exact parent result for a valid child is existing status `needs_attention`, `ok=false`, source
`command`, exit 4, no stderr/details, and bounded error `passive world doctor unavailable`. Invalid
child evidence uses the same closed shape with error `passive world doctor incoherent`. The
recursive validator rejects case/separator variants of credentials, tokens, API/private keys,
authorization, passwords, secrets, prompts, request bodies/bytes/input, carrier/auth-bundle bytes,
parent/full environment, and commitment preimages without echoing the rejected key or value. JSON
and human output therefore share one existing representation without a transport/report-schema
change.

F5-PD changed documentation only at that checkpoint. Its runtime implementation and the F5 work
that followed are recorded by the controlling closeout below.

## A1.1d-5R2-2F final evidence and decision ledger

### Verified source and preservation identities

F5 started from clean F5-PD HEAD `653a7d91489563bc2a8e3395feeb53e159254240`, tree
`4e3457444c64bbddda7bcf19300c4e7e81082678`, on
`feat/internal-host-orchestrator-world-dispatch-bootstrap`. The published source remote remained
`2de15dfba9f11d874eccfdca5438c600be1045b2`; divergence was 0 behind / 16 ahead and the worktree,
index, and untracked set were clean. F5-PD preservation
`feat/preserve-a1-f5pd-runtime-20260722` remained exact at that commit with ordinary/full-index/
stable patch identities `b7a5b60f8c7dbd1960b7fec4870de047c827ed611ea1d4bfba33e6a9ed22e32d`,
`b6e7e4a61c24815ef24fb74660d3cbc499de256771d1857bb963b0fd5f31b03d`, and
`90dafa73a5887457114ad2d4073fab8980e5630e`.

The blocked donor preservation `feat/preserve-a1-f5-blocked-candidate-20260722` remained exact at
`c4b93506480ae5e9708417067980e08e0e754b7a`, parent
`7802c44198625ea6849933140c390900ebe84190`, tree
`03bec2f517866e5213b8a0bed9ad99058c25ebc3`, ordinary/full-index patches
`81151bdad6aa9e7dd1963b7b22f1f3dad66b46399174ff26bfa33ec08859e03a` and
`7805a28defc11866902fadaaf99d7c752800c5aab257b9c927187920bd6cf908`. It is not an ancestor of
the source branch and was not merged, cherry-picked, reset onto the source, or restored wholesale.

F5 is `2bb4696d7181d974c1b02e33d82e09422cdae7de`, parent exact F5-PD, tree
`313a8a613a6cd91e72c0cc1664f4b9842fefa305`, complete message
`fix: compose truthful authenticated doctor evidence`. Its ordinary patch is
`991859870123dca56f3ee080876742abb2e556f9ee0bdf62f3577fdf1ccfb189`, full-index patch is
`8e2be2f2b84dc5d3cd573b9670e4d3eccc3060893f478a414d057e39b002a943`, and stable patch ID is
`b768cb94c51802e2440fbf1eb7f29fc0693a446b`. The name-only manifest hash is
`3716e05de411351b410c31fba9b37aebf98f470a1e1170f6d6a5c35658dca883`; the complete name-status
manifest hash recorded during commit proof is
`69285de3d080d277cc73232173e6fb035ca898f336e818960fd036c862164c13`.
The per-file blob SHA-256 values below are the exact fingerprints.

| F5 file | Final blob SHA-256 | Insertions / deletions |
|---|---|---:|
| `crates/shell/src/builtins/shim_doctor/report.rs` | `f02cff00312eb847d3bce47ab1311fe6938b231d0fe7430015bc18232a1bfde1` | 427 / 1 |
| `crates/shell/src/builtins/world_deps/mod.rs` | `e4c22293ff8aeb541a5e0115996006c6368931803dd900e86aed558fd741020b` | 37 / 7 |
| `crates/shell/tests/shim_doctor.rs` | `d4af75efc5da8bb17427b5a006e2dc33e1762d82d929665a671ffb7d236c671a` | 134 / 0 |
| `crates/shell/tests/shim_health.rs` | `f135c7cffd9c236f642ef119887df09ada7330c55c853cf324e14630f2ce91db` | 155 / 0 |

Dedicated preservation branches `feat/preserve-a1-f5-runtime-20260722` and
`feat/preserve-a1-f-complete-runtime-20260722` both point remotely to exact F5. The source runtime
remained local/unpushed while preservation branches carried immutable proof.

### Blocked-donor hunk disposition

Every donor hunk was classified before production edits:

| Donor hunk family | Final disposition |
|---|---|
| World-deps selected-prefix/commitment fields | Applicable only after rebasing onto F5-PD; retained under Linux cfg with strict top-level decoding |
| Parent world-deps identity/status checks | Applicable after semantic correction; exact A/CWD/enums/duplicates are validated and absent health stays unavailable |
| Fixture selection and direct fixture payload retention | Requires semantic correction; fixture is physical A-bound compatibility/test evidence only, runtime claims reject, retained output is recomposed |
| Donor no-fixture public World Doctor child | Forbidden and replaced by F5-PD's authenticated passive child |
| Donor generic `serde_json::Value`-first decoding | Forbidden; typed/duplicate-rejecting validation precedes nested generic shape inspection |
| Donor inference of success from identity, exit, empty error, or missing runtime evidence | Forbidden; missing evidence is bounded unavailable |
| Active public Doctor, service/socket/readiness/HTTP/execute/probe behavior | Forbidden and absent |
| Ambient/contextless identity or fixture authority | Forbidden; request-scoped A is the sole identity owner |
| F5-PD hidden selector, early authenticated branch, passive emitter, and strict child decoder hunks | Replaced by completed F5-PD and frozen |
| Non-Linux compatibility, report/transport schema expansion, F3/F4 request/readiness/environment, secure-FD, policy/capability, lifecycle, cleanup, or later-packet hunks | Obsolete, outside F5, or forbidden |

### Final runtime behavior and proof

Linux `WorldDepsDoctorSnapshotV1` rejects unknown top-level fields and carries A's selected prefix
and non-secret commitment. Linux `collect_doctor_snapshot_v1` validates the authenticated context,
reads A-derived config/global inventory plus request-scoped launch-CWD/workspace inventory through
that shared context, and deliberately does not call the applied/runtime probe. It
returns no applied items and the bounded reason `passive runtime health evidence unavailable`.

Linux `gather_world_deps_section` keeps the existing disabled early return. On the enabled route it
accepts only the exact physical `A/health/world_deps.json` compatibility fixture, decodes it
strictly, rejects runtime claims and mixed/malformed evidence, then discards its payload and
recomposes from the passive A collector. Without a fixture it calls the same passive collector.
The result is unavailable until adequate permitted runtime evidence exists; no identity field or
fixture can fabricate coherent health. A→B fixture symlinks, raw applied errors, fixture counts,
known-field secret markers, nested unknown fields, duplicates, partial sets, and identity/
commitment/CWD mismatches fail closed without disclosure.

The exact no-fixture shim-doctor integration proves one authenticated passive child and zero public
Doctor, socket/service, HTTP, execute, probe, world, provisioning, repair, cleanup, or mutation
activity. Shim-doctor and Health JSON/human views agree. The public World Doctor tests remain
semantically unchanged. F3/F4 readiness and exact seven-entry environment are frozen.

The clean F5-PD shell-library baseline was 1,307 discovered, 1,262 passed, 45 failed, 0 ignored.
Each of three final parallel walls and the final serial wall was exactly 1,309 discovered, 1,264
passed, 45 failed, 0 ignored. The 45 failure names and normalized signatures retained SHA-256
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70` and
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3` on every wall. The serial
normalizer excludes libtest's standalone `FAILED` harness token; after that format-only exclusion
its panic evidence is byte-identical to the parallel authority.

| Transition from clean F5-PD | Count |
|---|---:|
| Existing pass → fail | 0 |
| New fail | 0 |
| Fail → changed failure | 0 |
| Fail → pass | 0 |
| New library pass | 2 |
| New focused integration pass | 3 |
| Removed | 0 |
| Renamed or substituted | 0 |
| Newly ignored | 0 |
| Weakened or redirected | 0 |

Focused F proof includes world-deps, current/global/workspace/runtime/provision/post-sync,
authenticated builder, exact readiness/environment, passive child/decoder, Host/World/Health/
shim-doctor, public Doctor compatibility, policy/network/world-fs, and managed secure-FD regression
suites. Shell and workspace all-target checks pass. Raw shell all-target Clippy remains non-green
only for the exact 20 inherited `needless_borrow` findings in untouched/frozen code; the documented
differential permits only that lint and otherwise denies every warning, and passes. Formatting and
diff checks pass.

The auxiliary full world-service unit wall completed 131 tests without failure and then stalled in
two existing FUSE doctor tests after an environmental busy-unmount diagnostic. It was terminated;
no complete world-service wall, privileged smoke, or product capability proof is claimed from that
run. The bounded secure-FD, request-routing, capability-gate, authority, and netfilter filters pass.

GitNexus final F5 detection is LOW with four changed files, 27 mapped changed symbols, zero affected
execution flows, and no new process/module owner. Fresh F5 increment reviewers
`/root/f5_inc_authority_r2`, `/root/f5_inc_lifecycle_r2`, and `/root/f5_inc_scope_r2` are CLEAN.
Fresh complete-F reviewers `/root/f_final_authority_security`,
`/root/f_final_lifecycle_regression`, and `/root/f_final_source_platform` are CLEAN.
After the first docs-only review findings were remediated, fresh replacement reviewers
`/root/f_docs_arch_authority_r2`, `/root/f_docs_evidence_diff_r2`, and
`/root/f_docs_sequence_stale_r2` returned CLEAN for architecture/authority,
evidence/differential, and cross-document sequencing/stale-status cleanup respectively.

### Final decision and next node

F1–F5 and F5-PD are runtime-complete, proof-complete, review-clean, preserved, and canonically
closed out. Linux authenticated truth is proven; macOS, Windows, and other non-Linux routes remain
frozen compatibility/unavailable/unproven. Public Doctor, managed secure-FD, direct-member
credential compatibility, policy, network, world-fs, caging, capability, service/lifecycle,
receipt/supervisor, cleanup/rollback, and platform ownership are unchanged. Open RG gates remain
open, direct-member Codex/UAA gateway adoption is not begun, privileged product smoke is not
claimed, and no seam is promoted.

The successful terminal state at that checkpoint was `A1.1d-5R2-2FComplete`. Its exact historical
next node was **A1.1d-5R2-2 renewed production-fix-free integration closeout**. The remediation
planning ledger below supersedes only that next-node disposition; R2-3, R2-4, and R3 remain after
the later closeout/publication sequence.

## Renewed R2-2 publication decision ledger

| Decision/gate | Status | Evidence and distinction | Binding resolution / next action |
|---|---|---|---|
| `PublicationDecisionRequired` | Resolved and preserved at `928f94e7b4c498273b40385f7bffea9e4f949700` | The completed F closeout required its documentation followed by exact replay, and that replay produced the current linear 17-commit range from `2f6f1f69b3519dafff01ef543e7d260da2c37700` through `e5fbd2d4441d248e137d52c44e493fb0abe158f8`. Historical Route D docs-first replay governed only its own pre-carrier correction. The selected docs-on-top publication authority is preserved locally/remotely at `feat/preserve-a1-1d-5r2-2-publication-authority-20260722`. | Keep commit and range identities unchanged. The later RP4 proof and current RP5 packet do not rewrite this preserved publication-authority checkpoint. |
| `BaselineCommandMismatchConfirmed` | Historical B1 correction complete; later renewed wall evidence ineligible | The first renewed-closeout prompt omitted the private-root invocation provenance. That wall was ineligible and added 59 correct trusted-root rejections below normal mode-`1777` `/tmp`; runtime source and tests did not change. B1 corrected that invocation contract. A later renewed attempt produced four count/hash-matching walls, but their parent-only wait and descriptor-before-delete closure made them provenance-ineligible. | Historical B1 disposition was to commit the [canonical broad-wall invocation contract](04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract) on exact `928f94e7`, preserve it only on the B1 branch, and stop. The current RP4/RP5 disposition is recorded below; no historical result is rewritten, and source publication was still the next step at that checkpoint. |
| Existing runtime identity | Frozen | All 17 current commits are linear, exact, locally reviewed, and preserved. They already rest on the latest F documentation. Replaying them again would create new identities without changing semantic content or proof value. | No replay, rebase, rewrite, cherry-pick, merge commit, or force push. Preserve every runtime byte and commit identity; keep blocked and preservation-only donors outside source ancestry. |
| Renewed integration proof | Historical ineligible attempt preserved; later RP4 product proof accepted clean by human disposition | Neither the publication-authority commit nor the B1 broad-wall correction itself ran an integration wall. The earlier renewed attempt ran four matching walls but was provenance-ineligible because R1 was authority/security NOT CLEAN and P1 failed the descendant/continuous-authority contract. The preserved RP4 packet later established focused exact PASS, authenticated runner self-test `99/99`, four eligible canonical walls, zero differentials, `6 launched / 6 reaped / 0 live`, and exact original/integration restoration on proofed integration commit/tree `8c46135c861a468dea316cf9fd7d6c6bb15bddac` / `5358497a8baec6f36e15aaef58415a759e64977d`. | Raw review files remain unchanged: gates/authority/sequencing `APPROVE`; persistence `REQUEST_CHANGES` preserved as three cache-only non-blocking process-audit debt items outside RP4 product-proof scope. This exact six-file RP5 docs packet preserves that distinction; source publication was still the next step at that checkpoint. |
| Final publication | Historical blocked state later resolved by the single ordinary fast-forward publication | A closeout document had to follow the already assembled range and evidence it certified. The source remote intentionally contained documentation only because source publication had not yet occurred in that RP5 run. | This exact six-file RP5 docs packet had to be reviewed/committed first. After those reviewed bytes were committed, source publication later completed through one ordinary fast-forward without rewrite. That completed publication then allowed the later R2-3 suffix chain and refreshed native evidence to close R2-3. |

This decision does not alter any implementation status recorded above. F remains complete under its
own packet-specific replay history. RP3 is complete, RP4 product proof is human-accepted clean, and
this exact six-file change is the bounded RP5 closeout packet. Source publication had not yet
occurred at that checkpoint; it later completed, after which the R2-3 suffix chain and refreshed
native evidence closed R2-3. R2-4 and R3
remain later, privileged product smoke remains unclaimed, and no seam is promoted.

## A1.1d-5R2-2-B1 broad-wall invocation evidence correction

Every live broad shell wall, parallel/serial authority wall, canonical or differential baseline
wall, final change-detection wall, renewed closeout wall, and F/Harness wall in this ledger uses the
[canonical shell-library broad-wall invocation contract](04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).
Historical commands below are retained as exact evidence and are non-normative when they omit that
contract.

### Verified B0 evidence and causal reconstruction

The immutable B0 packet is
`/home/spenser/.gstack/projects/atomize-hq-substrate/checkpoints/20260722-a1-r2-2-b0-evidence.md`,
SHA-256 `75a15639ccb60d4d1b8206d6fa23622d5146213ebf8409b1e78fcc6d0417d776`.
It binds branch `feat/internal-host-orchestrator-world-dispatch-bootstrap`, HEAD
`928f94e7b4c498273b40385f7bffea9e4f949700`, tree
`9f0f9b2362ba878f45a30ab169a7c1e3e4569df2`, unchanged source remote
`2f6f1f69b3519dafff01ef543e7d260da2c37700`, and runtime ordered-commit hash
`ffe57dc349b462b318752a346e6577cff80bd15ba264720b967b888778f45700`.

The failed renewed-closeout command was historically:

```text
cargo test -p shell --lib
```

It discovered 1,309 tests, passed 1,205, failed 104, ignored 0, and produced failure-name hash
`7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5` plus normalized-signature
hash `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579`.
Omitting `TMPDIR` made Rust `TempDir` roots for six StateStore fixture families direct descendants
of `/tmp`. `/tmp` had its normal root-owned mode `1777`; retained trusted-root validation walked the
ancestor chain and correctly rejected its world-writable authority before persistent state
mutation.

The causally correct historical control was:

```text
XDG_RUNTIME_DIR=/home/spenser/t \
TMPDIR=/home/spenser/t \
cargo test -p shell --lib -- --nocapture
```

`/home/spenser/t` is historical evidence only and must never become the canonical machine-specific
path. The control restored 1,309 discovered, 1,264 passed, 45 failed, and 0 ignored, with canonical
failure-name hash `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`
and normalized-signature hash
`33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.

All 59 additions reduce to this single root-opening cause. Exact serial and isolated tests fail
identically without private `TMPDIR`; private-root parity makes all six families pass. Runtime
source, tests, manifests, lockfile, toolchain, target, profile, and features are unchanged. This is
not environment drift, process-global test isolation, or a production regression, and it provides
no reason to change fixture behavior or trusted-root validation.

### Baseline and future classification authority

The terminal classification is `BaselineCommandMismatchConfirmed`. The 59 additions remain
separate ineligible evidence. They are not added to the 45-failure baseline, waived, treated as
product failures or environment drift, used to weaken security, or used to justify serial-only
proof.

Future wall processing validates root and command provenance before result comparison. A missing
or invalid `TMPDIR`, missing or invalid `XDG_RUNTIME_DIR`, unsafe/sticky ancestor, incomplete
owner/mode/no-symlink/identity/ACL proof, or incomplete command/toolchain/result/hash record makes
the entire wall ineligible. Correct the invocation and rerun the complete required wall. Only a
provenance-valid wall can create a `PassToFail`, `NewFail`, changed-failure, or baseline transition.

The B1 correction changed exactly the six canonical Markdown files and no runtime/test artifact.
It ran no integration wall, did not create an integration-clean branch, and left the source
remote unchanged. Its historical B1-only binding sequence was:

```text
B1 broad-wall invocation docs correction
  -> renewed production-fix-free integration wall using validated private roots
  -> fresh independent reviews
  -> final six-file integration-closeout docs
  -> one ordinary fast-forward source push
```

At that B1 checkpoint, R2-2 remained incomplete and the exact historical next task was **Resume
A1.1d-5R2-2 renewed production-fix-free integration closeout using provenance-validated private
roots**. The current planning ledger below supersedes only that next-task disposition.

## Closeout-remediation planning ledger (historical RP0 checkpoint)

### Immutable planning checkpoint

| Field | Verified value |
|---|---|
| Branch | `feat/internal-host-orchestrator-world-dispatch-bootstrap` |
| HEAD / parent / tree | `f7ded83ef147b748678ba6b028eea959870a04fe` / `928f94e7b4c498273b40385f7bffea9e4f949700` / `e0836009318259e168acf3fe7837c57ec61c73fd` |
| Source remote / divergence | `2f6f1f69b3519dafff01ef543e7d260da2c37700`; `0 behind / 19 ahead` |
| Repository state | worktree, index, and untracked set CLEAN |
| Publication preservation | `feat/preserve-a1-1d-5r2-2-publication-authority-20260722` → `928f94e7b4c498273b40385f7bffea9e4f949700` |
| Invocation preservation | `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` → `f7ded83ef147b748678ba6b028eea959870a04fe` |
| GitNexus | repository `substrate`, indexed at exact `f7ded83`; 1,442 files, 34,536 graph nodes, 70,731 graph edges, 1,564 communities, 300 execution flows |

The runtime and documentation range was unchanged at checkpoint. No source checkpoint commit,
integration-clean preservation, wall invocation, implementation/test edit, or source publication
occurred in this planning investigation.

### Renewed-closeout blocker record

| Review | Verdict | Controlling evidence |
|---|---|---|
| Authority/security | NOT CLEAN | `deploy_shims` passes the authenticated carrier to `run_cmd`; dry-run `$*` emitted the marker and `carrier_marker_disclosed=true` |
| Lifecycle/product | CLEAN | No separate lifecycle/product blocker found; this does not override R1/P1 |
| Inventory/source | CLEAN | Owned production routes closed; this does not override R1/P1 |
| Baseline/publication | BLOCKED | Harness did not prove descendant exit and released descriptor authority before deletion |

The four prior walls each reported `1309 discovered / 1264 passed / 45 failed / 0 ignored`,
failure-name hash
`b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`, and normalized-signature
hash `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`.
They remain **EVIDENCE-INELIGIBLE**. No baseline transition is inferred from them and no historical
review is rewritten.

### R1 source-closure evidence

At the checkpointed source:

| Source | Current behavior | Closure |
|---|---|---|
| `run_cmd` | Dry-run prints `"$*"`; live mode executes `"$@"` | Generic behavior retained; four direct calls inventoried |
| `run_with_sudo` no-sudo call | Finite current bare-tool set resolved through fixed privileged PATH, or one permitted absolute ACL helper; account/group, package, artifact/TMP/destination path, mode/owner, unit/socket, and ACL operands | No protected value class found |
| `run_with_sudo` `sudo -n` call | Exact sudo/env wrapper plus that current tool/operand set | No protected value class found |
| `run_with_sudo` interactive-sudo call | Exact sudo/env wrapper plus that current tool/operand set; no-TTY error may render original operands | No protected value class found |
| `deploy_shims` call | Carrier plus `--shim-deploy`; xtrace disabled/restored around `run_cmd` | Sole current sensitive direct caller; R1 target |
| Other install carrier routes | Dedicated fixed dry-run placeholders and xtrace suppression | No additional live disclosure found |
| `world-provision.sh::show_install_context_cmd` | `%q` display helper redacts the separate carrier and environment-assignment forms | Not reusable: display-only, no equals-form or structural-rejection grammar, and not coupled to exact execution; frozen |
| `crates/common::{redact_sensitive, redact_process_argv}` | Rust logging redactors use heuristic sensitive-name/value matching; raw logging behavior exists | Not exact Bash carrier authority; frozen |
| Uninstaller | Separate carrier handling; no `run_cmd` | Frozen and required as unchanged compatibility proof |

The search covered carriers, credentials, authorization values, commitment preimages,
prompt/request values, private host paths/selectors, dry-run/live output, failure rendering,
xtrace, and installer/uninstaller compatibility. `run_with_sudo` has no named bare-tool allowlist;
the static current callers above are the closed set. GitNexus could not resolve these shell
symbols, so the exact manual call-site inventory controls. R1 may not expand beyond the one named
production helper/caller and one focused test without new evidence and docs-first authorization.
Every helper structural rejection has one exact argument-independent result: stderr
`[install-substrate][ERROR] invalid authenticated bootstrap carrier arguments\n` and status `2`;
valid child status propagation and exact post-call xtrace restoration remain unchanged.

### P1 failed-harness evidence

The prompt-local harness:

1. used `subprocess.run` and waited only for Cargo;
2. recorded `process_and_children_exited=true` without a descendant-authority observation;
3. could not prove reparented or process-group-changing descendants were gone;
4. closed root/child validation descriptors before cleanup;
5. called pathname `shutil.rmtree` after losing continuous identity;
6. checked pathname absence only after that unauthoritative deletion.

The shell wrapper's PID namespace eventually bounded teardown, but it did not connect namespace
emptiness to the Python worker's wall-eligibility decision. Matching output cannot repair those
facts.

### P1 source-closure and decision record

Searches covered installer and test harnesses, checkpoint helpers, Rust test utilities, Python and
shell helpers, production trusted-root utilities, process groups, cgroups, namespaces, subreapers,
log/hash preservation, and deletion. No tracked helper met the combined contract. Reusing
production trusted-root, cgroup, service, or provisioning code would exceed the test-only
allowlist. The selected minimum is the two-file Linux standard-library runner in
`04-contracts-and-gates.md`.

The proof basis is:

- exact installed Bubblewrap plus the pinned root-owned OS/Python startup closure are the explicit
  immutable platform TCB; exact direct Python/V2 plus authenticated in-memory `host_main` is the
  concrete host controller; the exact reviewed P1 commit binds bootstrap and three argv-template
  trailers to independently verified runner-blob constants, while each runtime instantiation
  records separate actual argv hashes, with no mutable external authority artifact; a
  clean nested unprivileged probe supplies namespace authority, and Python does not claim to
  preauthenticate already-executed startup bytes;
- reviewed V2 bootstrap bytes execute retained Git by FD, independently verify the
  expected-commit/tree/blob chain, authenticate runner/test blobs for in-memory execution, and
  supply Stage-A/Stage-B runner bytes only by sealed memfd plus `--ro-bind-data`;
- Stage A constructs private checksum-authenticated repository/rustup/Cargo immutable seeds and a
  separate writable Cargo runtime; nested Stage B read-only projects repository/rustup and
  writable-projects only the private Cargo runtime at canonical paths, so verified
  `cargo -> rustup` plus repository `rust-toolchain.toml` still selects Rust 1.89 while preserving
  exact Cargo argv/CWD/E0 except TMPDIR/XDG. The seed omits Cargo metadata; only exact regular
  `.package-cache`, `.package-cache-mutate`, and `.global-cache` may be created or changed;
  host modify/replace/restore races
  cannot become Cargo input;
- exact nested Bubblewrap bounds every wall process in a private PID namespace; the
  detached-descendant probe proved default Bubblewrap parent wait/PID 1 is fail-safe teardown
  only, so Stage B uses `--as-pid-1` and the authenticated worker itself is both the
  status-reported application and namespace adoption root;
- the Stage B worker connects to a fixed private `SOCK_SEQPACKET` path and is accepted only when
  `SO_PEERCRED` matches Bubblewrap's retained child pidfd; both endpoints are close-on-exec, the
  pathname is absent before START, Cargo retains original stdin, one exact Stage-A-owned combined
  stdout/stderr pipe, and no control/evidence FD; Stage A drains that pipe with bounded
  backpressure through EOF after Stage-B reap and preserves exact eligible bytes/hash; the worker
  becomes the verified pre-Cargo child subreaper that receives reparented descendants;
- worker reap-to-`ECHILD`, its authenticated completion record, and clean retained Bubblewrap
  status together prove containment emptiness;
- timeout/survivor is ineligible before scoped teardown;
- open no-follow parent/root/child descriptors retain identity through final validation,
  descriptor-relative unlink, and pathname-absence proof; Stage A removes all private entries but
  never claims its own enclosing mount is gone;
- authenticated host `host_main` retains separate backing/evidence authority, proves Stage-A
  process/namespace teardown, removes and proves absence of the exact underlying mountpoint, and
  alone finalizes eligibility;
- evidence is preserved outside `R`, and unrelated sentinels survive.

The future runner's own self-tests precede every product wall. Only after those tests and fresh
P1 review are CLEAN may a new three-parallel/one-serial baseline be established.

## RP3/RP4/RP5 closeout ledger

This section is the controlling current status. It preserves the historical RP0 planning ledger
above without rewriting any raw review verdict or historical wall result.

| Packet / evidence | Current status | Preserved evidence | Binding boundary |
|---|---|---|---|
| RP3 — canonical baseline | Complete | Canonical baseline remains `1309 discovered / 1264 passed / 45 failed / 0 ignored`, failure-name hash `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`, normalized-signature hash `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`, and six pairwise differentials `0`. | The historical 45-failure set is unchanged. |
| RP4 — renewed production-fix-free closeout | Product proof accepted clean by human disposition | Packet `/home/spenser/.cache/substrate-runtime-refactor/rp4-rp5-closeout-20260725T184536Z/rp4-proof-es7gs6z2/rp4-closeout-run-packet.json`; packet SHA `ea2935d04062c9e7e1327d5ecf88122513164b14fb19c7a2122c32b71b326072`; summary SHA `a51cf659647d10803004e7ad49c94ce580f9f23d5e4526608d60eb1a70789684`; supervisor result SHA `8e087dc6a4b627f61a97ade65bd4a13680113d79a9c718974671e5e91f20cee9`; supervisor launch manifest SHA `efbdd4ee8cf60aaa2b9128ca4c6844b82bb804f73dde5d53bcd8369906d173b8`; focused exact PASS sentinel; authenticated runner self-test `99/99`; four distinct canonical walls each `1309/1264/45/0`, `eligible=true`, `wall_gate=clean`, `canonical_match=true`; proof processes `6 launched / 6 reaped / 0 live`; exact original/integration restoration plus no-edit/config/branch/commit/publication guards passed; proofed integration commit/tree `8c46135c861a468dea316cf9fd7d6c6bb15bddac` / `5358497a8baec6f36e15aaef58415a759e64977d`. | No product/runtime rerun or remediation is authorized in this packet. |
| RP4 raw review distinction | Raw verdicts preserved unchanged | `rp4-attempt8-review-gates.txt` SHA `0e482944222da337ce3f6b1b4a3d2d243e550b8091d7720282c3b08b0dc361dc` `APPROVE`; `rp4-attempt8-review-authority.txt` SHA `5ecadc8fa03c546722be0f854665e49db30520610c60670a0abaddda8df11c67` `APPROVE`; `rp4-attempt8-review-sequencing.txt` SHA `654d8e30e5a7ec22a8afe307f9dd5602a5b15e9867f6d32f9f46d9c43fa44e44` `APPROVE`; `rp4-attempt8-review-persistence.txt` SHA `4489c53ae85555c45b32485cdff1fa6ceb4af194eecf6ce2668512620d06eebe` `REQUEST_CHANGES`. | The persistence verdict is not rewritten to `APPROVE` or `CLEAN`. |
| Persistence review findings | Non-blocking process-audit debt outside RP4 product-proof scope | Controller pathname/hash not execution-bound; supervisor parent-directory `fsync` omitted after replace; persisted controller packet omits its post-persistence final verdict. These are cache-only orchestration/attestation issues, not Substrate product/runtime defects and not canonical wall-runner defects. | They do not invalidate the focused, authenticated, canonical-wall, differential, authority, or restoration evidence. |
| RP5 — bounded final docs closeout | This exact six-file docs change | Preserves **Docs-on-top -> one ordinary fast-forward publication** and makes no wall rerun, no publication claim, and no parity claim. | At that checkpoint, source publication had not yet occurred, and the exact next step after the reviewed/committed RP5 docs was one ordinary fast-forward source publication. That publication later completed without rewrite. |

R2-3ZP2 is a later runner-contract repair, not a rewrite of the historical RP4 packet above. It
separates live descendant `expected_head` from reviewed `authority_commit_oid`, adds a separate
control-plane `reviewed_authority_commit_oid` that must exactly equal
`authority_commit_oid`, requires the reviewed authority commit to be equal to or an ancestor of
the live head, requires exact authenticated runner/self-test blob continuity across that boundary,
propagates both reviewed-authority fields plus projected repository CWD into Stage B, makes
`reviewed_oid_matches` a verified equality result rather than a caller assertion, and raises the
authenticated self-test gate to the exact 101-method allowlist. This closes
reviewed-authority/descendant drift once the reviewed authority OID is supplied independently by
the trusted control plane, but same-head review provenance remains external to the runner rather
than derivable from caller-selected OIDs alone. The preserved RP4 packet still remains historical
evidence for July 25, 2026, including its then-authenticated `99/99` self-test result on proofed
integration commit `8c46135c861a468dea316cf9fd7d6c6bb15bddac`; that evidence is not retroactively
relabeled as a post-ZP2 proof. The next reviewed authority commit for future canonical walls must
be recomputed from final post-ZP2 bytes and trailers rather than inferred from this ledger entry.

`R2-3ZP3` is an unpublished proof-infrastructure attempt and is permanently deferred from the
blocking R2-3 path. Commits `2cb796ffef68c2b049376984a90ce0382e5f3980`,
`9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`, `50948dbeb921582515a34bb6b7be21c46f29d008`, and
`868994efaf132bb04c6cdd7c82da9433333c94e5` remain diagnostic evidence only. They are not accepted
product source and must not be cherry-picked, pushed, or represented as landed.

The recurring canonical-runner defect is now explicit. Authenticated wall executions can
materialize honest shell results yet still finalize ineligible with `evidence_write_failed` and
`mount_teardown_failed`, because the runner still lacks authenticated Stage-A completion/teardown
proof and hidden backing-path removal proof. R2-3 closeout therefore claims no eligible
canonical-runner provenance. This remains open proof-infrastructure work outside the R2-3
completion gate.

That ordinary fast-forward publication subsequently completed at
`0f1e147fb735791b44a65099a65167cbdc1803af`. The R2-3 suffix then landed as
`R2-3ZT1 -> R2-3ZH1 -> R2-3ZM5 -> fresh native macOS and Windows evidence -> R2-3Z`, with final
source checkpoint `c583c5f293644fab75d8d42bd3bcad63f114d4fe` / tree
`a070f5f5787c27180f13dece9a1c3c3241728fda`.

The accepted direct shell-library proof at that final source is
`1322 discovered / 1274 passed / 48 failed / 0 ignored`, with failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`. Count-only equivalence
remains insufficient: the historical 45-failure set must remain intact, and the only additional
failures may be the separately classified non-R2-3 expectations
`builtins::shim_doctor::report::tests::world_deps_fixture_cannot_establish_runtime_health_or_cross_a`,
`builtins::shim_doctor::report::tests::world_deps_section_forwards_authenticated_a_under_conflicting_ambient_b`,
and `builtins::world_deps::tests::doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation`.
The sole current broad shell-wall entrypoints for this exact source are `make shell-lib-wall` and
`make shell-lib-wall-serial`. The provenance-ineligible authenticated runner's
`1322 discovered / 1277 passed / 45 failed / 0 ignored` result, plus the noncanonical 100- and
76-failure `R2-3ZM5` reruns, remain diagnostic-only environment evidence and never replace this
accepted direct proof.

The refreshed source-bound native receipts are authoritative at this closeout:

- macOS receipt `sha256:3b44f6387070b7aaea4306ae58d7f280b1cee3e163ee59219d4900ce3f53dfaf`,
  artifact `sha256:64a726d45b8bb6724fe43f566903d7b5e5ed6ef16119f14d30cd5a03edcc7e50`,
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-MAC-01`;
- Windows receipt `sha256:4ee942690655c1fac185244438d14e2561df52c306dea7e5428d556b530fd28c`,
  artifact `sha256:2c36a8dae9f3bcfa1c240c62a1e44f93aee0798702ae78a0076d43778b93d46e`,
  `EVIDENCE_CLEAN` for `R2-DIAG-01` and `R2-MAP-WIN-01`.

Both bind the exact source commit/tree/ref, confirm prohibited actions, and leave their platform
checkouts unchanged. Neither claims forwarding activation, provisioning, timeout kill, stop,
PID/artifact deletion, unregister, cleanup, rollback, or convergence authority.

R2-3 therefore closes only PI-009, PI-022, PI-039–PI-046, PI-048–PI-049, PI-052, PI-054–PI-058,
PI-060, PI-068–PI-070, PI-075–PI-076, PI-079, PI-081, PI-090–PI-091, PI-112, PI-115–PI-116, and
PI-118. PI-059 remains harness-only; PI-077 and PI-078 remain byte-frozen fail-closed guards;
PI-050 remains the R2-4 guardrail; PI-080 remains satisfied by earlier R2-2 Linux restart-scope
work and is not reopened here; and every R3 lifecycle/forwarding/provisioning/cleanup/rollback/
convergence action remains open. R2-4 and R3 remain later. Privileged product smoke, broader non-Linux product
proof, direct-member adoption, and seam promotion remain unclaimed.

## Review-process calibration

RP4 exposed a process failure without exposing a product or test regression: a blanket requirement
for four clean reviews allowed findings about agent-created cache-only orchestration to expand the
product-proof acceptance surface. Repeated fix/review attempts then improved bespoke evidence
tooling rather than the selected Substrate outcome. Human disposition correctly preserved the raw
`REQUEST_CHANGES` review while accepting the independently evaluable product proof.

The prospective correction is owned by the development-review contract in `04`:

- `P1`/`P2` block only on demonstrated impact to the selected contract, gate, scope, or completion
  claim;
- one discovery cycle, one consolidated remediation, one closure cycle, and at most two directly
  causal supplemental cycles bound automatic work;
- `CLEAN` is terminal, mechanical-only deltas do not create review cycles, and unrelated or expanded
  blockers stop for authority rather than widening scope; and
- valid non-blocking review/process debt is retained in `06`, separate from this product regression
  ledger.

The three RP4 persistence findings are registered as `RR-RF-0001` through `RR-RF-0003`. Their raw
review files, hashes, and original verdict remain unchanged. This calibration authorizes no
controller/supervisor remediation and changes no RP3/RP4/RP5 proof result.

## A1.1d-5R2-4 terminal evidence ledger

| Evidence / issue | Terminal disposition | Boundary |
|---|---|---|
| Original R2-4 authority | Preserved at `9e7b4b48e92864be6970ad373c35cb8bf18593b0` / tree `71ab2e5675f251f5ac74eed98ea535d5c3aa10f2`; later exact descendants are `316ee5c6cf12c060388c9d9376e0a79537f2094a` and `d5a46fb3a5afbd0e1a92e027d85ae76c3576dc32`. | Earlier blocked receipts remain immutable; no history repair or reinterpretation. |
| Resume-5 Linux evidence | 131-entry manifest verifies; result SHA-256 `35e77a683cfec6589776b974e6c2d511ef14d7a5760bb5ebf157bc71c41ebc28`. | PATH bypass and incomplete public-wrapper limitations remain historical and are superseded only by resume-6. |
| Resume-6 correction/proof | 153-entry manifest verifies; result SHA-256 `0a87fadd0cd22b406b7305c43400a10fe2f40541e4a5dfb29ce8d00b20932547`; legacy receipt SHA-256 `fe3d83727331ac3ae2dd0ee1491687f316534df47e33ff97c1a57a7fe625e620`. | Receipt is plain text, not V1 JSON. Exact later propagation plus independent parent verification is recorded without manufacturing protocol compliance. |
| Normal install/repeat/uninstall | PASS at custom A under hostile B; identical `3c57e0459af3e9e09bef46773e832e57bd3a0e5d14cac5ea51d81395c7bc8cbd`; byte-stable profiles; one A block; public wrapper authenticates A; B unchanged. | Context-selection and propagation only. |
| World/Codex reachability | Bounded earlier manual `pwd`, world-doctor probe, and `codex-cli 0.125.0` reachability adopted. | No authenticated Codex turn, retained worker/task, direct-member, gateway, or session-lifecycle claim. |
| Cache archive mode | Prior archive observation `0640`; exact accepted continuation object `0650`, SHA-256 `4a20a53943a7e6a0c5fa4463d4e47c58dd8e553ecebde455a4107e9906bfb001`, regular/no-symlink, `root:substrate`, bounded ACL, no group/other write; install/sync unchanged and restoration exact. | One local object only; no general relaxation or upstream guarantee. |
| Separate/deferred lanes | Passive health/world-deps diagnostics remain separate; uninstall leftovers and cleanup/rollback/manifest/convergence remain R3-owned. | Neither lane is R2-4 work or completion evidence. |

The full absolute paths, artifact hashes, authority provenance, proof matrix, and exclusions are in
[`review-control/r2-4-closeout-evidence.md`](review-control/r2-4-closeout-evidence.md). R2-4 is
closed only as the bounded R2 propagation join. R3 implementation is `PARKED_BY_USER`; the
just-closed corridor head was the B1/B2.1 joint closeout, B3.1, C1, and the bounded internal
A1.2b packet are complete on the bound Tuesday, August 4, 2026 candidate, the next authority gate
is `AUTHORITY_REQUIRED:R3_RESUME`, and R3 must be revalidated, explicitly reauthorized, resumed,
and completed before A1.3, A1.4, or A1 closeout. The larger architecture and runtime-refactor
program remain open.

## A1.1d-5R3 planned proof and regression ledger

Nothing in this section is executed evidence. It freezes the future proof obligations at planning
source `4ceecd50e20d822dda7cbd8f0e1bef4ccad65d8e` / tree
`8ed5dc7a354b731016a103b68091864b6a09223a`. Planning is complete at
`19c40d41679e843e3e524f64fb9827959849d33e` / `d7f6b84c9efc8ad03d98ad55c4e1a31611b96335` with
planning fingerprint `sha256:8f4cf54640443dbeb82fffbef68fac8d03eeaa6c72cf4e44f645044bc2b210e7`.
R3 implementation is `PARKED_BY_USER`, no implementation task has been dispatched from this plan,
and the just-closed corridor head was
`AUTHORITY_REQUIRED:B1_B2_1_JOINT_CLOSEOUT`; B3.1, C1, and the bounded internal A1.2b packet are
complete on the bound Tuesday, August 4, 2026 candidate, and the next authority gate is
`AUTHORITY_REQUIRED:R3_RESUME`. A fresh authority gate must revalidate this preserved proof
program before R3 is explicitly reauthorized, resumed, and completed before A1.3, A1.4, or A1
closeout. No `cargo test --workspace`
expected-failure inventory is frozen here, and approximate
workspace-failure counts are not authority.

### Source-closure and risk ledger

GitNexus was refreshed to the exact planning base. Query/context/impact still under-resolved shell,
PowerShell, cfg-specific paths, generated projections, and implicit `Drop`, so manual source
closure is binding:

- `ensure_private_substrate_home` is HIGH (16 graph-visible direct callers; the manual production
  chain reaches shell routing and scaffold intake);
- `ShellConfig::from_cli` is HIGH (8 direct/10 total graph-visible) and the edit fence is only its
  shim action block;
- `ShimDeployer::ensure_deployed` has a production chain to `run_shell_with_cli` and must be
  treated at least MEDIUM despite variable graph counts for private helpers;
- `create_ssh_uds_forwarding`, `ForwardingHandle::drop`, and
  `MacLimaBackend::ensure_forwarding` appear LOW or partial in the graph, but cfg reachability,
  implicit destruction, and activation make them manual HIGH-review action boundaries;
- `MacLimaBackend::new_with_mapping` and existing IH/PM validation are frozen; any need to edit
  them stops the macOS packet; and
- every Bash/PowerShell deletion, installer/uninstaller child, generated projection, service/
  platform call, and native action must be manually caller-closed again at its implementation
  base.

The planning-base manual closure specifically found and fenced Linux service enable/stop/socket/
ACL/start actions after the old daemon-reload boundary; the complete `lima-stop.sh` body (which
has no `VM_NAME=` assignment); Windows release/dev creator and destructor ranges, WSL warm stale
PID plus the unconditional live guard and both provisioning health branches; and Unix dev/release
creation through final state writes, the live owner-helper kill tombstone, and release temp-root
creation/registration/extraction/cleanup. Windows closure also begins before the two current log-
directory creations and before release `$tempRoot` creation. The WSL mapping
resolver already requires an exact registered distro machine ID, so the import branch cannot be
made authoritative and is a preserving tombstone. The Lima path also reaches group/membership,
guest binary, private-home, unit/service, layout, and in-guest DNS/toolchain mutations; the latter
are excluded passive-health/build fallback and are tombstoned. It also found that Cargo dist and
dev build/staging omitted all lifecycle executors. The exact
replacement/delivery owners are in `03` and `02`; a later implementation that leaves any legacy
action reachable or any executor unbuilt/unstaged fails its allowlist gate.

Any new HIGH/CRITICAL impact, consumer outside the frozen fence, destructive call, selector, or
authority source is a blocking plan contradiction until explicitly re-authorized.

### Candidate and manifest matrices

| Matrix | Required positive cases | Required preserving negative cases |
|---|---|---|
| Private-home rollback | current attempt created; retained parent; no-follow child open; exact identity rejoin; exact owner/`0700`/ACL; empty; one descriptor-relative removal; clean rerun | `AlreadyExists`, pre-existing, unknown provenance, replaced, nonempty, wrong type/owner/mode/ACL, inaccessible or ambiguous lookup, symlink, descriptor/path mismatch, unsupported identity; bytes/metadata unchanged |
| Manifest parse/join | exact schema/version/independent ID/digest/context/principal/platform/instance/closed role/object/type/disposition/metadata; exact locator/head/current generation and first/repeat read | missing, malformed, duplicate, unknown, tampered, coherent forgery, valid-old replay, partial, unsupported, cross-prefix/principal/platform/instance, locator substitution, unprivileged publisher, parent replacement, role scope escape, wrong object/bytes/target/mode/owner/ACL/security descriptor |
| Protected publisher | direct-terminal exact authorization; verified source receipt plus `ExecutorBuildEvidenceV1`; Unix socketpair or Windows UAC-pipe peer attestation; exhaustive `PublisherBootstrapComponentRoleV1` identities including every creatable parent/container and per-role exact absent-create versus exact pre-existing-dependency disposition; complete `LifecyclePublisherProtectedStateV1` wrapper CAS; null-slot bootstrap-core digest followed by precommitted Ed25519 harness key/external-store descriptor/test-retirement digest and harness-only pre-host reservation of zero Linux or one MAC/WIN guest tuple in final bootstrap and generation one; durable intent/executor/endpoint/key/counter-zero/active/generation-one sequence; fixed Linux `SOCK_SEQPACKET`, macOS XPC, or Windows named-pipe service; one framed request/response; nonce replay join; protected-host P-256-signed one-use guest pairing ticket carrying canonical SPKI; full SPKI hash/challenge/literal pinned through independent host and guest controlling terminals before ticket verification; host-protected one-use guest-mutation grant before the first guest effect; durable host record plus atomic root-only guest intent inside the enumerated state root with confined seed/nonce, signed hello/transcript, later final-key materialization, exact host+guest anchor/consumption join, and null-slot precommitted exhaustive guest-before-host test retirement; exact reserved/ticket-issued unused revocation proving every target's precommitted before-state and zero guest effect, with protected-host-P-256 proof, externally hashed/fsynced proof identity, precommitted-harness-Ed25519 acknowledgement, externally hashed/fsynced acknowledgement identity, and both hashes in protected CAS before pairing-record restoration/removal | missing/wrong confirmation, file/env/script carrier, wrong/expired source receipt, wrong build-evidence source/lock/toolchain/target/hash/file or code identity, staging-path privileged execution, pre-existing publisher identity, absent/pre-existing dependency disposition mismatch, partial/mismatched/omitted/unlisted-parent component, wrong component role/target/dependency, wrong peer PID/SID/uid/image/macOS designated requirement/Windows file identity/hash/DACL/framing, extra/oversize/truncated frame, concurrent/conflicting nonce, same-client coherent rewrite, valid-old anchor, detached prepared digest, protected-wrapper/counter rollback, clear-before-final-anchor, user keystore/DPAPI fallback, caller path/action, batch/unknown role, cross-build/instance; unreserved/extra/reused/omitted guest tuple, second evidence ticket, nonterminal reservation at host teardown, liveness/PID/timeout/pathname unused proof, guest effect without signed consumption grant, pre-existing target treated as absent, wrong unused-proof/acknowledgement key, algorithm, hash, file/parent identity, fsync, reservation/ticket/pairing record, dispatch/scope/nonce, replay, unknown field, self/future digest, missing acknowledgement, early pairing-record removal, final-bootstrap or guest-ticket digest cycle, core/final non-commitment-field mismatch, later-created/substituted retirement authorization, harness key/algorithm/encoding, omitted retirement role, early seed-intent/protected-state removal, or external directory; malformed/substituted SPKI, wrong curve/algorithm/encoding, SPKI/fingerprint or anchor-key mismatch, DER/high-S/malleable signature, request-supplied key, unsigned/wrong/expired/replayed ticket, non-TTY or automated pairing, channel-only authority, non-byte-identical child/channel retry, substituted nonce/transcript/intent/key, named-temp/rename intent publication, effect-visible/identity-record-not-durable component gap, ambiguous restart; zero state change |
| Closed role oracle | every serialized variant in `R3-MANIFEST-01` derives exactly its table domain/target/type/dependencies/actions and rejects every other action | arbitrary absolute/relative path, unknown binary/profile/unit/service/instance/pipe, wildcard, legacy name, alternate transport, wrong domain/object/action, role/manifest target mismatch |
| Publication | temp write, temp fsync, exact destination/generation/head join, atomic rename, parent fsync, head CAS, durable retry | temp symlink/nonregular, parent replacement, destination race, generation mismatch, head rollback/CAS conflict, short write, any sync/rename failure; no deletion authority |
| Action/retry | exact current action finishes or restores; manifest-preallocated receipt ID/name; protected prepared-record/counter; publisher-signed receipt without self/future digest; receipt write/fsync/parent-fsync; verified external hash; append-only `ManagedActionReceiptIndexV1` CAS; head CAS; publisher-anchor CAS; crash/retry at every boundary; committed repeat is a no-op | missing/unplanned/extra/unsigned receipt, filename/hash/file-parent identity mismatch, same-principal substitution, wrong signature key/algorithm/counter/prepared record/action observation, receipt self digest, index/head/anchor revision mismatch or rollback, replacement, ambiguous effect, stale PID, liveness-only join, nonempty state, unowned shared object; preserve and classify |
| Shared claims | first claimant records before-state; compatible second claimant joins; creator-first and creator-last uninstall; last-claim restoration/teardown | conflicting desired state, missing claimant, stale generation, claim removal before consumer release or teardown receipt, one-prefix shared removal, inferred transfer/adoption |
| Legacy collision | none: R3 has no adoption producer or acceptor | every `adopted` value and every unmanifested/pre-R3 collision, even matching bytes, remains unchanged and yields `BLOCKED_SCOPE_EXPANSION` plus manual-adoption handoff |

Candidate kill points are immediately before and after create, first no-follow open, identity/
metadata/ACL validation, cleanup arm, empty proof, removal, and acceptance. Manifest/action kill
points are before and after temp create, write, temp fsync, publication rename, parent fsync,
`ActionStarted`, every destructive action, effect observation, restoration, receipt temp write,
receipt signature, receipt fsync, receipt rename, receipt parent fsync, signature verification,
external receipt hash, receipt-index temp/
fsync/rename/parent-fsync CAS, head CAS, publisher-anchor CAS, and final commit. Every kill point runs exact
retry and verifies the permitted join/finish/current-attempt rollback/preserving-stop branch.

### Lifecycle convergence and preservation matrices

Every platform runs default A and custom A under hostile ambient B, two-prefix A1/A2, and unrelated
sibling sentinels. The matrix includes:

- first install, repeat install, partial failure at every action boundary, exact retry, first and
  repeat uninstall, uninstall after partial failure, uninstall/reinstall, and inverse-order
  two-prefix uninstall;
- exact link and target as separate objects; payload/version/bin/cache/profile file bytes, target,
  mode, owner, ACL/security descriptor, flags, and timestamps where authoritative;
- unit/drop-in bytes, enabled/active state, service identity, socket identity/ACL, runtime/state
  trees, helper/gateway/shim identities, group existence, membership, ACL bridge, linger, process/
  forwarder identity, pipe, VM/distro identity, and platform-control state;
- created and pre-existing-preserved dispositions, plus preserving rejection of every
  schema-reserved `adopted` input with `BLOCKED_SCOPE_EXPANSION`; and
- prohibited wildcard/prefix deletion, ambient-home selection, recursive candidate cleanup,
  follow-links/reparse behavior, broad process kill, stale-PID/timeout authority, unowned VM/WSL
  teardown, and shared-parent cleanup.

Mac cases separately cover guest group/membership, exact private home, all four guest binaries,
unit bytes, service/socket enabled and active state, runtime/state directories, fixed
`/etc/substrate-lima-layout`, A-local `lima_known_hosts` line/file before-state, rendered-unit
current-attempt temp tree, staged tree, and forwarding sockets/process. A missing exact evidence-
built binary
must produce zero guest change; any attempt to build in guest, install a toolchain/package, rewrite
`/etc/resolv.conf`, or restart DNS services is a failing prohibited-action oracle. Mac absent-
instance cases additionally kill before/after stage-one anchor, create, first identity
observation, second identity observation, PM finalization, manifest publication, and stage-one
close; wrong profile/control root, pre-existing instance, identity drift, partial create, or
unfinalizable PM preserves state. Windows cases
begin with an already-registered exact PM-bound distro and prove both unhealthy and healthy guest-
refresh branches; absent distro, download/import, WSL install-tree deletion, and unregister are
preserving scope-expansion oracles. Unix and Windows release cases kill at current-attempt temp-tree
parent/root open, creation, cleanup arm, each checksum/bundle/payload/extracted-child registration
and write, each reverse handle/descriptor-relative removal, empty proof, and root removal; abrupt
death residue is preserved. Windows additionally proves exact `A\forwarder\logs` before-state,
creation/restoration under hostile `LOCALAPPDATA`, and preserving pre-existing, replaced, or
reparse directories for both forwarder entry points. Unix `--kill-live-processes` cases require
`BLOCKED_SCOPE_EXPANSION` before enumeration or signal, preserve the hidden owner helper, and reject
name/PID/executable inference; a separately observed live consumer blocks uninstall.

Success requires an exact pre/post parity manifest for every unrelated or pre-existing object and
no unreceipted managed change. Product uninstall retains only the exact
`Uninstalled` lifecycle capsule/head/executor/receipts named in `04`; repeat uninstall and
reinstall must join it. Disposable publisher proof requires an external descriptor-bound
`PublisherTestRetirementAuthorizationV1` before bootstrap. The harness first creates its key and
descriptor-bound external directory, durably signs the authorization, and commits their exact
identities/digest into bootstrap and the generation-one anchor. The publisher accepts only those
byte-identical precommitted values and signs the exact retirement receipt without an in-record self
digest; the harness fsyncs its canonical bytes, computes the external artifact SHA-256, then signs
and fsyncs `PublisherTestRetirementAcknowledgementV1` binding that hash before reverse-order
component removal. Every kill
point before/after retirement receipt serialization/write/fsync/external hash, acknowledgement
sign/write/fsync/return/verification, publisher acknowledgement record, stop,
endpoint/registration/current-anchor/key/
executor/created-empty-parent removal exact-joins that receipt, and baseline parity precedes the ordinary
evidence receipt. Without that authority, test teardown is forbidden.

### Baseline and product behavior wall

Before a native run, the harness builds the exact platform host executor and, where Lima/WSL is in
scope, the Linux guest executor from the remote-equal source and lock state in an isolated build-
only scope. It records source/lock/toolchain/target/artifact SHA-256/file identity and platform code
identity in `ExecutorBuildEvidenceV1`, removes that scope, and proves build-state absence. It then
captures the clean checkout commit/tree/ref/status/divergence, OS/architecture,
tool versions, current principal, platform mapping, mounted/filesystem/security features, services,
units, sockets/pipes, processes, groups/memberships, ACL/security descriptors, linger, relevant
prefix and platform-control trees, registered VM/distro identities, and unrelated sentinels. The
capture is canonicalized, hashed, and stored before mutation.

An exact-absence publisher-bootstrap subcase additionally requires a disposable dedicated scope,
an absent-component baseline, and an external retained-descriptor retirement directory created by
the evidence harness before product bootstrap. The harness durably publishes the test-retirement
authorization first and treats its signed retirement receipt as the only authority for teardown;
ordinary product lifecycle never exercises retirement. A native host that cannot provide this
isolated scope reports the full platform handoff rather than weakening the bootstrap gate.

Every evidence task must discover product project
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` by exact origin plus target ref and join its freshly bound
dispatch nonce, source task thread/host, evidence task thread/host, return meta thread/host,
evidence ID, and source commit/tree/ref. Those fields are part of the artifact and receipt. After
both validators pass, the task sends the native JSON to the exact return meta task with
`send_message_to_thread` as its final tool action. A missing/mismatched correlation field is invalid
evidence, not a hand-waved routing detail.

The product smoke must preserve:

- filesystem isolation/diff and network behavior;
- policy/configuration/dependency resolution;
- gateway and world-service transport;
- runtime, shim, replay, trace, and diagnostic behavior;
- PTY and non-PTY execution; and
- exact selected-context behavior under hostile ambient B.

It does not import passive health/world-deps remediation, authenticated Codex execution, retained
worker/task/session work, gateway adoption, or direct-member architecture. After failure or
success, execute the recorded restoration plan, invalidate sudo credentials on Linux, prove every
owned test object absent or restored, prove every unrelated/pre-existing sentinel exact, recapture
the baseline, and require canonical digest parity. Static review is never native evidence.

### `R3-NATIVE-LINUX-01`

- **Gates:** `A1.1d-5R3-CLOSEOUT`; also required by `R3-LIFE-01` and
  `A1D5I-ENV-01`.
- **Source checkpoint:** exact landed `A1.1d-5R3-UNIX` commit/tree on
  `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`, with live remote equality,
  required predecessor ancestry, clean checkout/index/untracked set, and `0/0` divergence.
- **Discovery/prerequisites:** dedicated supported Linux host; exact product repository discovered
  by git top-level plus origin/ref/commit/tree checks; systemd, required filesystem/ACL features,
  supported Rust/toolchain, authorized interactive sudo, and read-only discovery plus a restored
  same-filesystem disposable proof that the enumerated `/var/lib/substrate` state root supports
  root `O_TMPFILE`, `linkat(AT_EMPTY_PATH)`, file/parent fsync, and exact unlink; baseline proves no concurrent
  installer/lifecycle process.
- **Allowed actions:** only manifest-bound install/repeat/partial/retry/uninstall/reinstall,
  service/socket/gateway/world product smoke, exact test-created group/membership/ACL/linger and
  synthetic-auth scenarios, the exact harness-owned atomic-intent capability probe, exact
  disposable publisher bootstrap plus externally receipted test retirement, and recorded restoration.
- **Prohibited:** passive-health remediation, unrelated service/account/network changes, broad
  kill, ambient-home or unmanifested deletion, reuse of a nondedicated host, retained sudo
  credential, or restoration by reset/clean.
- **Evidence:** main artifact
  `review-control/r3-native-linux-01-evidence.json`, receipt
  `review-control/r3-native-linux-01-receipt.json`, schema
  `substrate.r3-native-evidence` v1, canonical SHA-256 digests, ordered command/output digests,
  baseline/restoration digests, product behavior results, unrelated sentinels, exact absence of
  service-to-socket propagation, protected world and disposable-publisher
  service-state/coupled-endpoint prepared transactions and receipts, endpoint non-requestability,
  missing/replaced/ambiguous endpoint zero-change,
  activation/observation/receipt kill-retry joins, service stop/restart/restore socket and endpoint
  nonmutation, and terminal `EVIDENCE_CLEAN`, including non-null
  `intent_publication_capability` with probe cleanup/parity.
  The receipt is native `codex.top-level-evidence-receipt.v1`, validated by
  `validate_evidence_receipt.py`; the evidence artifact's gated successor is exactly
  `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-MAC-01`, and receipt source live remote equals its source
  commit/tree/ref.
- **Restoration:** exact unit/drop-in/service/socket/runtime/state/helper/gateway/group/
  membership/ACL/linger/prefix/home parity; `sudo -k` and verification; clean repository and host
lifecycle state. Any mismatch blocks. Unavailable host yields the complete
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` handoff.

### Packet-local provider evidence gates

These gates prevent a later platform packet from starting until the preceding provider has been
published, exercised on its native platform, and closed with evidence-only repository bytes. The
implementation packet first makes one reviewed product/test commit and one normal fast-forward
push. A separate evidence task then discovers a clean checkout at that exact published
commit/tree/ref and requires `source.live_remote == source.commit`. It changes no repository byte,
creates no commit, and validates the native `codex.top-level-evidence-receipt.v1` with
`orchestrate-top-level-tasks/scripts/validate_evidence_receipt.py <receipt>`. That skill validator
has no successor flag. The evidence artifact separately binds the exact gated successor and is
validated with `scripts/ci/validate_r3_native_evidence.py` plus the exact expected evidence ID,
source commit/tree/ref, and gated successor; the receipt's evidence digest must equal the validated
artifact digest through exact receipt field `evidence.artifact_sha256`. Only a later platform-
closeout packet may commit the evidence artifact and receipt.

All three use `substrate.r3-native-evidence` v1, the baseline/action/artifact/restoration/digest
schema in `04`, the same platform discovery and allowed/prohibited clauses as the corresponding
final gate, terminal `EVIDENCE_CLEAN`, and exact checkout/host restoration. A provider gate
exercises only its packet fences plus run-only regressions; it does not satisfy the later post-UNIX
final gate.

| Evidence task | Exact published source | Evidence/receipt paths committed only by closeout | Artifact gated successor | Closeout task-receipt successor |
|---|---|---|---|---|
| `EVIDENCE:R3-LINUX-IMP-01` | landed LINUX commit/tree/ref, live remote equal | `review-control/r3-linux-imp-01-evidence.json`; `review-control/r3-linux-imp-01-receipt.json` | `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX-CLOSEOUT` | `AUTHORITY_REQUIRED:A1.1d-5R3-MAC` |
| `EVIDENCE:R3-MAC-IMP-01` | landed MAC commit/tree/ref, live remote equal | `review-control/r3-mac-imp-01-evidence.json`; `review-control/r3-mac-imp-01-receipt.json` | `AUTHORITY_REQUIRED:A1.1d-5R3-MAC-CLOSEOUT` | `AUTHORITY_REQUIRED:A1.1d-5R3-WIN` |
| `EVIDENCE:R3-WIN-IMP-01` | landed WIN commit/tree/ref, live remote equal | `review-control/r3-win-imp-01-evidence.json`; `review-control/r3-win-imp-01-receipt.json` | `AUTHORITY_REQUIRED:A1.1d-5R3-WIN-CLOSEOUT` | `AUTHORITY_REQUIRED:A1.1d-5R3-UNIX` |

If a platform or privilege is unavailable, the evidence task returns
`BLOCKED_PLATFORM_HANDOFF_REQUIRED` with the published OID/tree/ref, checkout discovery,
prerequisites, allowed/prohibited actions, evidence schema/digests, restoration commands, and
gated successor. An available run that fails or does not restore returns
`BLOCKED_NATIVE_EVIDENCE`. The closeout packet and every later implementation packet are forbidden
in either case.

### `R3-NATIVE-MAC-01`

- **Gates:** `A1.1d-5R3-CLOSEOUT`, `R3-LIFE-01`, and the macOS portion of
  `A1D5I-INSTALL-08`.
- **Source checkpoint:** the same exact landed `UNIX` commit/tree/ref and clean remote-equal
  checkout.
- **Discovery/prerequisites:** native supported macOS/architecture, supported Lima and SSH,
  repository identity checks, exact intended principal, existing or test-created Lima instance
  whose disposition and machine identity are recorded, exact IH/PM, clean platform-control root,
  unrelated instance/state sentinels, and a live human operator with independent host and guest
  controlling TTYs for the full pairing fingerprint/challenge/literal confirmation; read-only
  Lima guest filesystem discovery plus a restored same-filesystem disposable proof requires the
  enumerated guest `/var/lib/substrate` root to support root `O_TMPFILE`,
  `linkat(AT_EMPTY_PATH)`, file/parent fsync, and exact unlink.
- **Allowed actions:** manifest-bound candidate/home parity, Lima stage/unit/socket lifecycle,
  exact owned instance stop/delete only when disposition authorizes it, A-scoped SSH-UDS
  activation/unlink, `StreamLocalBindUnlink`, timeout kill/wait, handle-drop teardown, retry,
  disposable LaunchDaemon/System-Keychain bootstrap, signed one-use guest pairing ticket plus
  human-pinned guest bootstrap, the exact harness-owned guest atomic-intent capability probe,
  both reserved and ticket-issued unused revocation with protected-host proof, externally hashed/
  fsynced proof identity, harness acknowledgement, protected CAS, and exact Keychain pairing-record
  restoration/removal, and externally receipted
  test retirement, product transport, and recorded restoration.
- **Prohibited:** ambient `auto_select`, alternate endpoint/transport/instance/principal/prefix,
  unowned instance destroy, unrelated known-hosts/control-state mutation, or path-existence-only
  unlink; automated/stdin/file/environment pairing confirmation is also prohibited.
- **Evidence:** `review-control/r3-native-mac-01-evidence.json` and
  `r3-native-mac-01-receipt.json`, canonical v1 schema/digests, mapping and instance identities,
  socket/process/action receipts, product outputs, baseline/restoration parity, unrelated
  sentinels, non-null guest `intent_publication_capability` with probe cleanup/parity, unused-
  reservation proof/acknowledgement artifact hashes and Keychain CAS/removal observations, exact
  absence of service-to-socket propagation, protected host-publisher plus guest world/publisher
  service-state/coupled-endpoint prepared transactions and receipts, endpoint non-requestability,
  missing/replaced/ambiguous
  endpoint zero-change, activation/observation/receipt kill-retry joins, service stop/restart/
  restore socket and endpoint nonmutation, terminal `EVIDENCE_CLEAN`. The receipt is native
  `codex.top-level-evidence-receipt.v1`, validated by `validate_evidence_receipt.py`; the evidence
  artifact's gated successor is exactly `AUTHORITY_REQUIRED:EVIDENCE:R3-NATIVE-WIN-01`, and receipt
  source live remote equals the same UNIX source commit/tree/ref.
- **Restoration:** staged/current trees, units, enabled/active state, runtime/host sockets,
  forwarding child, known-hosts, instance/control root, prefix/home, unused or consumed pairing
  record, every reserved guest target, and unrelated instance state exactly restore. Both
  `Reserved` and `TicketIssued` unused branches prove exact baseline parity. Unavailable platform
  yields the full handoff and never a static substitute.

### `R3-NATIVE-WIN-01`

- **Gates:** `A1.1d-5R3-CLOSEOUT`, `R3-WIN-01`,
  `A1D5I-INSTALL-07`, and `A1D5I-INSTALL-10`.
- **Source checkpoint:** the same exact landed `UNIX` commit/tree/ref and clean remote-equal
  checkout.
- **Discovery/prerequisites:** native supported Windows and PowerShell 7; supported WSL; repository
  identity checks; exact current SID, Known Folder observation, registered distro name and
  machine-ID, pipe scope, clean two-prefix fixture, unrelated `.substrate*` and other-distro
  sentinels, and a live human operator with independent host and guest controlling TTYs for the
  full pairing fingerprint/challenge/literal confirmation; read-only WSL guest filesystem
  discovery plus a restored same-filesystem disposable proof requires the enumerated guest
  `/var/lib/substrate` root to support root `O_TMPFILE`, `linkat(AT_EMPTY_PATH)`, file/parent fsync,
  and exact unlink.
- **Allowed actions:** manifest-bound two-prefix install/repeat/partial/retry/uninstall/reinstall;
  profile/shim/bin/version cleanup; exact forwarder child/PID/image/start/pipe timeout and stop;
  exact already-registered PM-bound WSL instance start/stop/restoration and guest service
  lifecycle, including exact `/run/substrate`, group `substrate`, PM-mapped guest membership,
  exact PM-bound `SUBSTRATE_HOME`/host-commitment/mapping service-unit rendering, byte-exact
  root:root `0644` service/socket units with no implicit directory management,
  daemon-reload-before-state ordering, and
  root:`substrate` `0660` `/run/substrate.sock` creation/readback/restoration;
  disposable LocalSystem/CNG/HKLM bootstrap, signed one-use guest pairing ticket plus
  human-pinned guest publisher bootstrap, the exact harness-owned guest atomic-intent capability
  probe, both `Reserved` and `TicketIssued` unused revocation with exact LocalSystem-signed proof,
  externally fsynced proof hash, precommitted-harness acknowledgement and hash, protected HKLM
  reservation CAS, pairing-record restoration/removal, and externally
  receipted test retirement; product transport; restoration. Instance import, install-tree deletion, and
  unregister are never allowed actions.
- **Prohibited:** wildcard/profile sibling deletion, ambient/default prefix or distro selection,
  process-name kill, stale-PID authority, shared-state removal from one prefix alone, any import or
  unregister, unowned terminate, reparse following, unrelated WSL/platform state mutation, or
  automated/stdin/file/environment pairing confirmation.
- **Evidence:** `review-control/r3-native-win-01-evidence.json` and
  `r3-native-win-01-receipt.json`, canonical v1 schema/digests, SID/instance/machine-ID/pipe and
  object receipts, product outputs, baseline/restoration parity, sentinels, non-null guest
  `intent_publication_capability` with probe cleanup/parity, distinct service-template-source,
  PM-render-input, rendered-service, socket-source, and unchanged-installed-socket digests,
  rejection of installed template markers or a rendered/modified socket, hostile-ambient rejection,
  protected LocalSystem-publisher plus WSL world/publisher service-state/coupled-endpoint
  transactions and receipts, endpoint non-requestability,
  missing/replaced/ambiguous endpoint zero-change, activation/observation/receipt kill-retry joins,
  exact absence of service-to-socket propagation, service stop/restart/restore socket and endpoint
  nonmutation, unchanged state/runtime/lifecycle directory
  identities/metadata, unit metadata and daemon-reload/enabled/active observations, both unused
  branch proof/acknowledgement artifact hashes and protected HKLM CAS/removal observations, terminal
  `EVIDENCE_CLEAN`. The receipt is native `codex.top-level-evidence-receipt.v1`, validated by
  `validate_evidence_receipt.py`; the evidence artifact's gated successor is exactly
  `AUTHORITY_REQUIRED:A1.1d-5R3-CLOSEOUT`, and receipt source live remote equals the same UNIX
  source commit/tree/ref.
- **Restoration:** both prefixes, profiles, shims, bin/version/config/log state, forwarder/PID/
  pipe, WSL service/instance, `/run/substrate`, `/run/substrate.sock`, group/membership and socket
  owner/mode/ACL state, exact service/socket unit bytes/metadata and enabled/active state, Known
  Folder control state, both `Reserved` and `TicketIssued` pairing-record before-state and
  baseline parity, unrelated `.substrate*`, and other distros
  exactly match baseline. Missing native authority yields the full platform handoff.

### Final decision rule

`A1.1d-5R3-CLOSEOUT` may close only when all three evidence IDs bind the same landed `UNIX`
checkpoint, validate independently, report exact restoration and `EVIDENCE_CLEAN`, and the
cross-platform regression/review walls are clean with zero unresolved P1-P4. Any missing,
stale, cross-checkpoint, digest-mismatched, partially restored, or statically substituted evidence
leaves the corresponding gate open. R3 closeout does not authorize the later A1.1d/A1 program.

## R3 implementation status append

### `A1.1d-5R3-MANIFEST`

- Status: `LANDED_CLEAN`.
- Newly landed proof surface: canonical/golden manifest encoding, duplicate/unknown/wrong-type
  rejection, receipt/index/head publication joins, bootstrap and pairing-ticket signature checks,
  preserving provider-unavailable channel behavior, the hidden direct-interactive control binary,
  and `validate_r3_native_evidence.py` with focused tests for source/evidence/successor/correlation
  joins.
- Remaining frozen truth: this packet performs no delete/replace/stop/kill/unregister/restore
  action, claims no native evidence, and leaves platform execution and retirement side effects to
  later packets.
- Successor: `AUTHORITY_REQUIRED:A1.1d-5R3-LINUX`.

### `A1.1d-5R3-LINUX`

- Status: `LANDED_CLEAN`.
- Newly landed proof surface: exact Linux executor/client wiring, fixed root-owned publisher
  service/socket units, `world-lifecycle.sh` pre-state snapshot/restore, protected and disposable
  publisher prepared transitions and canonical receipt paths, service/socket non-propagation, guest
  bootstrap transcript verification and negative coverage, and dry-run gateway-proof baseline
  restoration.
- Remaining frozen truth: this packet claims no native evidence, does not widen Unix/macOS/Windows
  ownership, and leaves run-only regression bytes unchanged; the reproduced `install_state_smoke.sh`
  hosted-bundle `world-agent` versus installer `world-service` mismatch is pre-existing at the
  bound base and outside this packet's editable fence.
- Successor: `EVIDENCE:R3-LINUX-IMP-01`.

### `A1.1d-5R3-LINUX-CLOSEOUT`

- Status: `LANDED_CLEAN`.
- Newly landed proof surface: exact repository materialization of the validated
  `review-control/r3-linux-imp-01-evidence.json` artifact and
  `review-control/r3-linux-imp-01-receipt.json` receipt, explicit revalidation of their
  evidence/source/digest joins, and the exact `linux-closeout` review set for the bounded
  docs/evidence closeout.
- Remaining frozen truth: no production, test, fixture, script, dependency, or platform bytes
  changed; the artifact's documented `install_state_smoke.sh` hosted-bundle mismatch remains
  reproduced unchanged at the bound source; and no MAC authority starts here.
- Successor: `COMPLETE` for this authoritative orchestration closeout.

### `A1.1d-5R3-MAC`

- Status: implementation publication scope; no native evidence claim.
- Newly bounded proof surface: typed SSH-UDS exact unlink/timeout/Drop/known-host restoration,
  mapped lifecycle argument/evidence refusal, tombstoned guest DNS/package/toolchain/Cargo
  remediation, exact socket non-propagation, and the non-executing mock executor fixture.
- Remaining evidence-only truth: supported-Lima publisher installation, host/guest executor build
  evidence, code-signing, independent-TTY pairing, lifecycle/restoration exercise, and external
  receipts belong solely to `EVIDENCE:R3-MAC-IMP-01`.
- Successor: `EVIDENCE:R3-MAC-IMP-01`.
