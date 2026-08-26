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
| **RG-AUTH-03** | Revision-bound host transition intent proof | **Unresolved** | `HiddenOwnerHelperLaunchPlan` currently carries authoritative `Start`/`Attach`/`ResumeOneTurn` mode, identity, lineage, workspace/world binding, descriptor, attach contract, resume handle, and input through arbitrary JSON. Plan load/removal and helper execution do not join a durable authority revision or greenfield-certificate-proven expected absence to a unique replay-safe intent. The parked-successor regression proves stale lifecycle/world-binding rejection is correct: pre-A1.1d `c800436d` and pre-rejection `f73e8a81` passed only while stale whole-session overwrite remained possible; `bdb1796d` and later revisions fail closed. A1.1d-1 through A1.1d-4 remain preserved and A1.1d-5 is focused-review clean on Linux, but integrated and native macOS closeout remain open; no seam promotion is claimed. | First prove A1.1 primitives without claiming production Start: genuinely absent versus initialization-pending, valid-existing, unsupported-pre-A1, and corrupt/unsupported roots; the same physical bootstrap home persisted in marker/root/certificate and used for StateStore/config/policy/inventory, including copied-home rejection; recognized temp cleanup and fail-closed unknown temps across crashes before/after temp fsync and rename; component-by-component no-follow traversal and complete safe enumeration of both pre-A1 authority collections, including ancestor identity/owner/mode/ACL and scan-to-publication replacement; any artifact rejected without parsing/conversion and any unreadable or unsafe collection failed closed. Every pre-A1 read-decide-write transaction must retain the opened physical root, descendants, exact identity, activation observation, and root lock through final `fsync`; a subprocess rename/replacement/rebind after lock acquisition must fail closed with the replacement root untouched and no successful transaction outcome. Post-activation writers reject before mutation and preactivation writers serialize on that same root lock. Exact retry treats reservations, tombstones, intents, issuer indexes, application journals, and object indexes as semantic occupancy and joins only complete matches. Rotation and retirement revalidate the complete reconciled candidate immediately before publication. Crash/restart, contention, temp reconciliation, publication durability, and exact retry converge. Then prove the A1.2 API alone can accept `ExpectedAbsent` only from the verified greenfield certificate, empty pre-A1 collections, and exact absence of the complete namespace-map key; it atomically issues its self-owned reservation and creates initial authority only while applying that intent. The held A1.3-P0 and A1.3 records remain preserved as non-executable historical fences. A1.3-P1 must atomically land the new authority-managed public Start/turn/reattach transport path, exact public CLI/helper/REPL authority consumption and actor-event resolution, and only the strictly mechanical projection of already HSA-authorized correlation through the exact retained-turn `orchestrator_world_dispatch.rs` acceptance-context construction/projection seam and into the existing B1 acceptance-context field, while preserving legacy helper behavior for compatibility callers and leaving auto-attach unchanged. A1.4 then proves bounded auto-attach producer adoption. Prove `Attach`/`ResumeOneTurn` from exact current parked or other enumerated authority against exact revision/hash with exact caller/source/target lineage, bootstrap-home/workspace/world binding, descriptor/attach/resume/policy/input commitments, unique intent/request identity, fixed expiry, canonical payload hashes, and retained transport reprojection. Application preserves lifecycle identity and world binding, revision-authorizes successor lineage and immutable input handoff, and never derives eligibility from PID/helper/handle/readiness/prompt state. Exercise every intent/input state and crashes after initialization marker/key/root publication, key publication/root selection/retirement, object publication, issue, plan load/removal, claim, authority commit, input acceptance, `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit with transport present/already absent. Exact retry joins without a second namespace, authority revision, participant allocation, or delivery. Stale revisions, substituted plans, cross-kind refs, mismatched hash/identity/home/binding/descriptor, missing/copied certificate, compatibility-derived absence, pre-A1 artifacts, missing keys, expired intent, superseded claim, and conflicting replay fail closed. A1.4 closes only its scoped clauses of `RG-AUTH-01`/`RG-AUTH-02`; those ledger-wide gates remain unresolved for A2/A3. | A1.2, A1.3-P1, A1.4 |
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

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a0-closeout-evidence`](a1-2-earlier-histories/evidence-regression.md#a0-closeout-evidence).

## A1.1d mandatory-review state

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11d-mandatory-review-state`](a1-2-earlier-histories/evidence-regression.md#a11d-mandatory-review-state).

## A1.1d-5I installer/bootstrap audit record

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11d-5i-installerbootstrap-audit-record`](a1-2-earlier-histories/evidence-regression.md#a11d-5i-installerbootstrap-audit-record).

## A1.1e explicit-home policy proof requirement

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a11e-explicit-home-policy-proof-requirement`](a1-2-earlier-histories/evidence-regression.md#a11e-explicit-home-policy-proof-requirement).

## A1.2a, A1.2a-WB, and A1.2a-S recorded result

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12a-a12a-wb-and-a12a-s-recorded-result`](a1-2-earlier-histories/evidence-regression.md#a12a-a12a-wb-and-a12a-s-recorded-result).

## B0 closeout evidence

Canonical content: [`b1-b2-1/evidence-regression.md#b0-closeout-evidence`](b1-b2-1/evidence-regression.md#b0-closeout-evidence).

## B1 production-path sequencing blocker and disposition

Canonical content: [`b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition`](b1-b2-1/evidence-regression.md#b1-production-path-sequencing-blocker-and-disposition).

## B1/B2.1 `RegressionMasked` stop and ownership disposition

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-regressionmasked-stop-and-ownership-disposition`](b1-b2-1/evidence-regression.md#b1b21-regressionmasked-stop-and-ownership-disposition).

## B1/B2.1-R0 recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-r0-recorded-result`](b1-b2-1/evidence-regression.md#b1b21-r0-recorded-result).

## B3.2a and B3.2a-WA recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b32a-and-b32a-wa-recorded-result`](b1-b2-1/evidence-regression.md#b32a-and-b32a-wa-recorded-result).

## B1/B2.1 core recovery and B1/B2.1-0 recorded result

Canonical content: [`b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result`](b1-b2-1/evidence-regression.md#b1b21-core-recovery-and-b1b21-0-recorded-result).

## B3.1 recorded result

Canonical content: [`b3-1-c1/evidence-regression.md#b31-recorded-result`](b3-1-c1/evidence-regression.md#b31-recorded-result).

## C1 recorded result

Canonical content: [`b3-1-c1/evidence-regression.md#c1-recorded-result`](b3-1-c1/evidence-regression.md#c1-recorded-result).

## A1.2b recorded result

Canonical content: [`a1-2-earlier-histories/evidence-regression.md#a12b-recorded-result`](a1-2-earlier-histories/evidence-regression.md#a12b-recorded-result).

## Baseline behaviors that all tracks preserve

These rows define permanent behavior contracts, not current pass claims. Any row explicitly marked
open remains a blocking regression gate until its named owner and real-path proof close it.

| Gate ID | Baseline | Required proof |
|---|---|---|
| **RG-BASE-01** | Public world-scoped start → turn/reattach → stop; **open, blocking, and not waived** | A1.2 must supply exact parked-successor `Attach`/`ResumeOneTurn` application. The held A1.3-P0 and A1.3 records remain preserved but cannot be dispatched. A1.3-P1 must atomically land the new authority-managed public Start/turn/reattach transport path, exact startup/post-turn actor-event resolution on the real public CLI/helper/REPL path, and only the strictly mechanical projection of already HSA-authorized correlation through the exact retained-turn `orchestrator_world_dispatch.rs` acceptance-context construction/projection seam and into the existing B1 acceptance-context field, without legacy authority writes, readiness/process inference, auto-attach drift, B1 semantic ownership changes, or C1/B3.1 semantic changes. Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. Stale lifecycle/world-binding overwrite remains rejected. Prove unsafe other-principal authority fails closed separately, then run the positive smoke against an owner-only private bootstrap home with exact type/`0700`, descriptor identity/replacement safety, qualified access/default `NoData` under authoritative mode bits, and no effective other-principal authority; `NoData` is not physical-absence proof. This gate is neither permanently expected to fail nor successful until that real-path wall is green. |
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

### R2-2 historical failed integration closeout and remaining-seam correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction`](a1.1d-5r2-2f/evidence-regression.md#r2-2-historical-failed-integration-closeout-and-remaining-seam-correction).

## A1.1d-5R2-2F0-HC empirical closure record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-hc-empirical-closure-record`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-hc-empirical-closure-record).

### Environment-inventory correction evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#environment-inventory-correction-evidence`](a1.1d-5r2-2f/evidence-regression.md#environment-inventory-correction-evidence).

### Preflight and preservation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#preflight-and-preservation`](a1.1d-5r2-2f/evidence-regression.md#preflight-and-preservation).

### Forced overlap matrix

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#forced-overlap-matrix`](a1.1d-5r2-2f/evidence-regression.md#forced-overlap-matrix).

### Bounded clean-code broad evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#bounded-clean-code-broad-evidence`](a1.1d-5r2-2f/evidence-regression.md#bounded-clean-code-broad-evidence).

## A1.1d-5R2-2F0 historical parallel artifact recovery and authority correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-historical-parallel-artifact-recovery-and-authority-correction`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f0-historical-parallel-artifact-recovery-and-authority-correction).

### Bounded recovery result

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#bounded-recovery-result`](a1.1d-5r2-2f/evidence-regression.md#bounded-recovery-result).

### Corrected semantic and concurrency authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#corrected-semantic-and-concurrency-authority`](a1.1d-5r2-2f/evidence-regression.md#corrected-semantic-and-concurrency-authority).

### Canonical candidate preservation, proof, review, and stop boundary

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#canonical-candidate-preservation-proof-review-and-stop-boundary`](a1.1d-5r2-2f/evidence-regression.md#canonical-candidate-preservation-proof-review-and-stop-boundary).

## A1.1d-5R2-2F readiness-boundary evidence ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-readiness-boundary-evidence-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-readiness-boundary-evidence-ledger).

### Reviewer finding and correction disposition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#reviewer-finding-and-correction-disposition`](a1.1d-5r2-2f/evidence-regression.md#reviewer-finding-and-correction-disposition).

## A1.1d-5R2-2F5-PD evidence and decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f5-pd-evidence-and-decision-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f5-pd-evidence-and-decision-ledger).

### Verified starting state and retained evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#verified-starting-state-and-retained-evidence`](a1.1d-5r2-2f/evidence-regression.md#verified-starting-state-and-retained-evidence).

### Exact blocked-candidate preservation

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#exact-blocked-candidate-preservation`](a1.1d-5r2-2f/evidence-regression.md#exact-blocked-candidate-preservation).

### Control-pack contradiction and source decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#control-pack-contradiction-and-source-decision`](a1.1d-5r2-2f/evidence-regression.md#control-pack-contradiction-and-source-decision).

### Security decision

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#security-decision`](a1.1d-5r2-2f/evidence-regression.md#security-decision).

## A1.1d-5R2-2F final evidence and decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger`](a1.1d-5r2-2f/evidence-regression.md#a11d-5r2-2f-final-evidence-and-decision-ledger).

### Verified source and preservation identities

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#verified-source-and-preservation-identities`](a1.1d-5r2-2f/evidence-regression.md#verified-source-and-preservation-identities).

### Blocked-donor hunk disposition

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#blocked-donor-hunk-disposition`](a1.1d-5r2-2f/evidence-regression.md#blocked-donor-hunk-disposition).

### Final runtime behavior and proof

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#final-runtime-behavior-and-proof`](a1.1d-5r2-2f/evidence-regression.md#final-runtime-behavior-and-proof).

### Final decision and next node

Compatibility anchor only; canonical content: [`a1.1d-5r2-2f/evidence-regression.md#final-decision-and-next-node`](a1.1d-5r2-2f/evidence-regression.md#final-decision-and-next-node).
## Renewed R2-2 publication decision ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-r2-2-publication-decision-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-r2-2-publication-decision-ledger).

## A1.1d-5R2-2-B1 broad-wall invocation evidence correction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#a11d-5r2-2-b1-broad-wall-invocation-evidence-correction`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#a11d-5r2-2-b1-broad-wall-invocation-evidence-correction).

### Verified B0 evidence and causal reconstruction

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#verified-b0-evidence-and-causal-reconstruction`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#verified-b0-evidence-and-causal-reconstruction).

### Baseline and future classification authority

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#baseline-and-future-classification-authority`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#baseline-and-future-classification-authority).

## Closeout-remediation planning ledger (historical RP0 checkpoint)

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#closeout-remediation-planning-ledger-historical-rp0-checkpoint).

### Immutable planning checkpoint

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#immutable-planning-checkpoint`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#immutable-planning-checkpoint).

### Renewed-closeout blocker record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-closeout-blocker-record`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#renewed-closeout-blocker-record).

### R1 source-closure evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#r1-source-closure-evidence`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#r1-source-closure-evidence).

### P1 failed-harness evidence

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-failed-harness-evidence`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-failed-harness-evidence).

### P1 source-closure and decision record

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-source-closure-and-decision-record`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#p1-source-closure-and-decision-record).

## RP3/RP4/RP5 closeout ledger

Compatibility anchor only; canonical content: [`a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger`](a1.1d-5r2-2-renewed-closeout/evidence-regression.md#rp3rp4rp5-closeout-ledger).
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

Canonical content: [`a1.1d-5r2-4/evidence-regression.md#a11d-5r2-4-terminal-evidence-ledger`](a1.1d-5r2-4/evidence-regression.md#a11d-5r2-4-terminal-evidence-ledger).

## A1.1d-5R3 planned proof and regression ledger (archived for active scheduling)

Canonical content: [`a1.1d-5r3/evidence-regression.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling`](a1.1d-5r3/evidence-regression.md#a11d-5r3-planned-proof-and-regression-ledger-archived-for-active-scheduling).

### Source-closure and risk ledger

Canonical content: [`a1.1d-5r3/evidence-regression.md#source-closure-and-risk-ledger`](a1.1d-5r3/evidence-regression.md#source-closure-and-risk-ledger).

### Candidate and manifest matrices

Canonical content: [`a1.1d-5r3/evidence-regression.md#candidate-and-manifest-matrices`](a1.1d-5r3/evidence-regression.md#candidate-and-manifest-matrices).

### Lifecycle convergence and preservation matrices

Canonical content: [`a1.1d-5r3/evidence-regression.md#lifecycle-convergence-and-preservation-matrices`](a1.1d-5r3/evidence-regression.md#lifecycle-convergence-and-preservation-matrices).

### Baseline and product behavior wall

Canonical content: [`a1.1d-5r3/evidence-regression.md#baseline-and-product-behavior-wall`](a1.1d-5r3/evidence-regression.md#baseline-and-product-behavior-wall).

### `R3-NATIVE-LINUX-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-linux-01`](a1.1d-5r3/evidence-regression.md#r3-native-linux-01).

### Packet-local provider evidence gates

Canonical content: [`a1.1d-5r3/evidence-regression.md#packet-local-provider-evidence-gates`](a1.1d-5r3/evidence-regression.md#packet-local-provider-evidence-gates).

### `R3-NATIVE-MAC-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-mac-01`](a1.1d-5r3/evidence-regression.md#r3-native-mac-01).

### `R3-NATIVE-WIN-01`

Canonical content: [`a1.1d-5r3/evidence-regression.md#r3-native-win-01`](a1.1d-5r3/evidence-regression.md#r3-native-win-01).

### Final decision rule

Canonical content: [`a1.1d-5r3/evidence-regression.md#final-decision-rule`](a1.1d-5r3/evidence-regression.md#final-decision-rule).
## R3 implementation status append

Canonical content: [`a1.1d-5r3/evidence-status.md#r3-implementation-status-append`](a1.1d-5r3/evidence-status.md#r3-implementation-status-append).

### `A1.1d-5R3-MANIFEST`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-manifest`](a1.1d-5r3/evidence-status.md#a11d-5r3-manifest).

### `A1.1d-5R3-LINUX`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-linux`](a1.1d-5r3/evidence-status.md#a11d-5r3-linux).

### `A1.1d-5R3-LINUX-CLOSEOUT`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-linux-closeout`](a1.1d-5r3/evidence-status.md#a11d-5r3-linux-closeout).

### `A1.1d-5R3-MAC`

Canonical content: [`a1.1d-5r3/evidence-status.md#a11d-5r3-mac`](a1.1d-5r3/evidence-status.md#a11d-5r3-mac).
## A1.1d-5R3-MAC attempt-4 remediation status (2026-08-06)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06`](r3-mac-evidence-recovery/evidence-regression.md#a11d-5r3-mac-attempt-4-remediation-status-2026-08-06).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression status (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-status-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-PLAN regression-command and status correction (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-plan-regression-command-and-status-correction-2026-08-07).
## AUX-R3-MAC-EVIDENCE-RECOVERY-R3 recovery-current validator invocation (2026-08-07)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-evidence-recovery-r3-recovery-current-validator-invocation-2026-08-07).

## AUX-R3-MAC-SYSTEM-KEYCHAIN-SOFTWARE-SIGNER-CORRECTION regression ledger (2026-08-10)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10`](r3-mac-evidence-recovery/evidence-regression.md#aux-r3-mac-system-keychain-software-signer-correction-regression-ledger-2026-08-10).
## R3 macOS retirement/orphan planning and G2/G3 stop ledger (2026-08-13)

Canonical content: [`r3-mac-evidence-recovery/evidence-regression.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13`](r3-mac-evidence-recovery/evidence-regression.md#r3-macos-retirementorphan-planning-and-g2g3-stop-ledger-2026-08-13).
## Current cross-lane regression ledger (2026-08-20; controlling)

Canonical content: [`macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling`](macos-dev-parity/evidence-regression.md#current-cross-lane-regression-ledger-2026-08-20-controlling).
