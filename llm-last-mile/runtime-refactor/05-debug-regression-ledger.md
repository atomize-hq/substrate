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
| **RG-EVENT-01** | Runtime event/frame identity, ordering, and exact terminal event | **Partially resolved: B0 producer and B2.1 durable-consumer cores review-clean; joint closeout open** | B0 landed the canonical V1 carrier in `common`/`transport-api-types` and producer assignment in ordinary `world-service` task streams plus retained launch/turn streams. Each accepted producer stream has one non-empty UUIDv7 stream ID; all frames share one positive gap-free frame counter; semantic Event/Exit items share one positive event counter and stable UUIDv7 IDs; Exit is the final semantic event and exactly matches its terminal identity. Canonical clone/re-emission preserves bytes. Concurrent stdout/stderr assignment is serialized through enqueue. Typed host decoding forwards identity unchanged, rejects missing/malformed runtime identity, and does not convert Error, EOF, stream exhaustion, or diagnostic non-live state into terminal truth. The recovered B2.1 core owns durable consumer journaling/replay acceptance, duplicate/gap/reorder/conflict rejection, and restart observation; the joint production integration closeout remains open, and producer replay retention remains separately bounded process-memory world-service transport state. | Preserve all B0 producer and negative-host gates. The later joint closeout must prove the recovered consumer on every real accepted production path while preserving byte-identical replay no-op, conflict/gap/reorder/post-terminal rejection, and restart-safe durable observation without treating EOF/error/process state as completion. | B0, B2.1 |
| **RG-RECEIPT-01** | `run_world_task` blocks until terminal exit | **Unresolved; B1/B2.1-0 prerequisite review-clean and joint closeout open** | B1/B2.1-0 prepares RunWorldTask from exact HSA, immutable acceptance, and supervisor truth and excludes the legacy active-task registration path; the foreground call still blocks over supervisor truth. | The recovered B1/B2.1 cores and B1/B2.1-0 are review-clean without closing the joint packet. The joint closeout proves every accepted production path enters the durable supervisor without a legacy writer; blocking remains a waiter. Only B2.2 later returns `ActiveEphemeralTaskReceiptV1` before exit. | B1, B2.1, B2.2 |
| **RG-RECEIPT-02** | `continue_world_worker` blocks until terminal exit | **Unresolved; confirmed design deviation** | `execute_continue_world_worker_stream_for_turn_kind` consumes until `Exit`; `continue_world_worker` returns only afterward. Existing model-visible outcome has no active-run receipt or immutable policy commitment. | After B1 acceptance, B2.1 observation, and C1 event truth, B2.2 returns `ActiveRetainedTurnReceiptV1` before exit; B3.2 preserves clean-turn worker continuity and the host can continue or park. | B2.2, B3.2 |
| **RG-RECEIPT-03** | Active receipt inspection and caller-drop survival | **Unresolved; receipt, supervisor, R0, and B1/B2.1-0 prerequisite clauses review-clean; joint closeout open** | Recovered B1 acceptance and B2.1 claims survive real guard, waiter, and caller drop. B1/B2.1-0 routes ephemeral Inspect/Cancel/Wait through exact receipt/supervisor truth and ordinary retained Continue through exact R0+B3.2a target/routability truth without legacy active-task synthesis. | B1-3a/B1-3b preallocate and durably create only the immutable acknowledged acceptance record. B2.1-1 claims that exact record for both work families before any later frame, B2.1-2 supplies inspect/wait compatibility over supervisor truth, and B2.1-3 preserves observation across restart. Caller/waiter/guard drop never deletes accepted or supervised truth. Guard-drop and waiter-drop tests must exercise actual guard/waiter destruction on the production-owned path; calling a receipt or supervisor helper directly is `RegressionMasked`. B1 and B2.1 close jointly; B2.2 later exposes early return. | B1, B2.1, B2.2 |
| **RG-RECEIPT-04** | Ephemeral `needs_retained_followup` remains a terminal non-retained result | **Partially implemented; target receipt semantics unproven** | The lifecycle design and current terminal enum preserve `NeedsRetainedFollowup`, but the durable active receipt contract did not state how it closes without becoming retained work. | B2.1 records `NeedsRetainedFollowup` only in immutable supervisor terminal closeout while leaving the B1 acceptance record unchanged; it promises no retained participant or continue route, creates no hidden retained worker/conversational obligation, and requires an explicit policy-checked spawn for ongoing work. | B2.1 |
| **RG-SUP-01** | Supervisor idempotent frame/event processing | **Unresolved; B2.1 durable core and B1/B2.1-0 route review-clean; joint closeout open** | The recovered B2.1 core claims exact B1 acceptance, durably journals canonical B0 frames/events, makes byte-identical duplicates no-ops, and rejects gaps, reorder, conflicts, stale observers, and post-terminal frames. B1/B2.1-0 routes the named production actions to that owner while foreground callers remain blocking waiters. | B2.1-1 establishes the no-gap claim from exact B1 acceptance; B2.1-2 makes exact duplicate frames/events no-ops, rejects gaps/reorder/conflicts/post-terminal frames, and prevents stale observer writes; C1 later proves duplicate replay creates no duplicate obligation. | B0, B1, B2.1, C1 |
| **RG-SUP-02** | Supervisor restart and reconciliation | **Unresolved; replay/startup core review-clean and joint production proof open** | The recovered supervisor durably owns nonterminal claims and exact cursors, and the bounded startup hook invokes canonical recovery. Surviving process-memory producer replay resumes exact frames; unavailable replay remains unresolved and nonterminal. | B2.1-3 alone discovers every nonterminal claim from durable supervisor state, resumes the exact B0 cursor through exact acceptance-record/stream/cursor producer replay when the process-memory world-service registry survives a host/shell restart, rejects stale observers, and never fabricates terminal success, cancellation, deletion, cursor advance, or a complete cut from EOF, timeout, PID/helper/socket/readiness/process/caller/endpoint state, local error, producer unavailability, or ambiguous/missing terminal truth. World-service restart or unavailable replay leaves the claim durably nonterminal and unresolved. The startup hook invokes only the canonical recovery entry point and distinguishes fatal store/claim corruption from a valid unresolved producer. | B2.1 |
| **RG-CANCEL-01** | Same-turn continue → cancel | **Unresolved** | Current retained cancel resolves worker state, `authoritative_live`, owner PID, `latest_run_id`, and `Running`; the blocking continue returns after that active window is gone. | In one host turn, continue returns accepted active-run ID and immediate cancel targets it. Final receipt is cancelled or pending closeout, never `stale_linkage` merely because the worker/owner parked. | B3.2, B4, E2 |
| **RG-CANCEL-02** | Cancel outcome semantics | **Unresolved** | Current retained cancel terminal enum only expresses `Cancelled`; no-active work is reported through target-not-cancelable/stale-linkage-shaped errors. | Prove all categories from `04`: live cancel, pending closeout, already terminal, no active work, owner unreachable, invalid target, binding mismatch, ambiguity, and policy denial. | B4 |
| **RG-MSG-01** | Typed retained-worker messaging and causation identity | **Unresolved; B0 ordering prerequisite satisfied** | B0 now gives every retained runtime Event exact producer-assigned event/frame identity and ordering, and host decoding cannot repair missing carrier identity. Typed message semantics remain mixed with prompt rendering, stream parsing, foreground delivery, and optional nested payload IDs; live thread/class/attention classification still reads provider JSON after `AgentEvent` construction and may drop an unrecognized shape. B0 did not classify provider payloads or add retained-message meaning. | After B1 acknowledgement and before `AgentEvent` construction, B3.1's named world-service normalizer maps every retained provider wrapper event fail-closed into `NormalizedWorldWorkerEventFacetV1`, with exact thread/class/attention/payload and B1 request/message causation. It then shapes the typed top-level `AgentEvent.worker_event` member on the existing event-frame path, gives C1 exact acceptance-record, worker-to-host target/active-run/thread/class/attention/causation semantics joined to B0 order, and copies any B1 equality-only opaque host-transition correlation unchanged. Every post-ack retained `Event` frame must be typed. Supported non-attention shapes and each attention-driving class have explicit producer tests; an unknown, ambiguous, deferred, or malformed shape fails the stream and C1 cut closed instead of being omitted, left untyped, or downgraded. Host/C1 JSON-pointer parsing of emitted `AgentEvent.data` cannot supply missing identity or semantics. B3.2 completes host-to-worker and broader immediate/durable delivery semantics. | B3.1, B3.2, C1 |
| **RG-OBL-01** | Obligation materialization before terminal exit | **Unresolved** | `continue_world_worker` saves a surfaced event and calls `persist_continue_world_worker_obligation` only after the full stream function returns. | A B3.1 attention event with exact B1 acceptance and B0 stream identity is durable in B2.1 and creates one C1 canonical obligation before the exact B0 terminal event; byte-identical replay creates no duplicate and terminal failure does not erase it. Before classification, C1 verifies that every post-ack retained B2.1 `Event` ref is typed and commits the complete canonical B3.1 event. C1 advances a monotonic ledger revision/materialized watermark, returns Pending before coverage, and Complete only through the exhaustive acceptance-record/stream/terminal-event cut; an untyped or omitted event prevents Complete. | B0, B1, B2.1, B3.1, C1 |
| **RG-OBL-02** | Obligation as canonical truth; inbox/attention as projections | **Partially implemented, unproven** | Obligation records, inbox materialization, pending counts, and attach fields exist, but production timing is terminal-coupled and compatibility rows/counts still share state-store authority. | The closed C1 snapshot query binds exact store/session/participant, B1 acceptance ID/revision and accepted active run, B0 stream and terminal event/cut, and an exhaustive ordered join to every post-ack retained B2.1 `Event` ref plus canonical commitments to its B3.1 source/target/thread/class/attention/causation/payload envelope through the cut even when no attention obligation exists, along with unchanged owner-supplied transition intent/revision/payload commitment and distinct transition run, authority/ledger revisions, disposition, and sorted canonical-record commitments. Any untyped/omitted/substituted event or stale/mismatched scope, correlation, cut, disposition, or record fails closed, and both empty/no-event-or-no-attention and non-empty/attention dispositions are proven. HostSessionAuthority consumes that result unchanged and cannot invent or reinterpret event or transition semantics. Rebuild inbox/auto-attach projections from obligations; host `AwaitingAttention` exactly follows unresolved attention-driving obligations; worker `AttentionPending` remains separate. | C1, C2 |
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
| **RG-OBS-01** | First-class world-dispatch observability | **Unresolved; B0, B1/B2.1 cores, R0, and B1/B2.1-0 prerequisite clauses satisfied** | B0 exposes exact producer-originated stream/frame/event/terminal identity. The recovered B1 receipt and B2.1 supervisor cores supply immutable acceptance plus durable claim/journal/cursor/terminal truth. R0+B3.2a supply exact retained target/routability, and B1/B2.1-0 joins those sources for its named full-dispatch and tool routes without legacy active-task writes; later joint production integration remains open. | Remaining clause owners are explicit: the B1/B2.1 joint production closeout, B3.1 producer semantic normalization plus typed worker event, C1 materialization/cut, B4 cancel, D2 broker operations, E2 policy commitments, and E3 non-secret credential-handoff state. World-service replay transports only exact producer facts, and `run_async_repl` only activates canonical supervisor recovery; neither is observation or lifecycle authority. Full-dispatcher, real guard/waiter-drop, and real tool-to-dispatch assertions are mandatory; direct resolver, transport, receipt, or supervisor substitutions remain `RegressionMasked`. B3.1 cannot start before the joint closeout. D3 alone owns final end-to-end integration closure, proving those facts join by request/active-run/session/world IDs without secret payloads or secret-derived fingerprints. | B0, B1, B2.1, B3.1, C1, B4, D2, E2, E3, D3 |

`RG-AUTH-03` A1.2 proof additionally requires Start/Attach transport to remain retained until
closed startup evidence binds exact store/session/intent/claim/claimant-attempt/run/application/authority/
participant identity, the exact target-participant or launch-claimant protocol actor, and durably
records the matching acceptance or definitive terminal reason. Timeout,
EOF, helper/PID/socket/handle/readiness loss, and ambiguity remain Pending. Exact retries after
`ReleaseEligible`, payload deletion, and `Released` join through terminal proof without requiring
deleted transport bytes; an unexpected Released copy is removed and its exact object directory is
`fsync`ed before success. Every Resume terminal outcome references one immutable, exact-scope
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
claims no native macOS or Windows proof. R2 implementation is ordered
R2-1 -> R2-2 -> R2-3 -> R2-4; R3 follows and remains unimplemented.

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
| **R2-UDEV-01** | Unix dev install, uninstall, `dev-shim-bootstrap.sh`, and successful install-sensitive standalone CLI modes each construct exactly one equal A/H/R context from the declared/installed-witness/account-default source before explicit-context home bootstrap; parse/help/version exits mutate no scaffold; hidden argv carrier alone selects internal-child validation and the current account+UID must equal its committed principal; custom A/bin dev symlink self-derives A without an outer override; direct repo and zero/multiple witnesses fail; conflicting ambient B/dev-prefix cannot retarget bootstrap, children, preexec, or uninstall selection; a forged-principal carrier fails before mutation. |
| **R2-UREL-01** | Unix release wrapper, direct installer/uninstaller, and installed child distinguish constructor/child modes and preserve the same context across install/uninstall; Unix release A/bin witness self-derives A; every child binds the committed principal to current Unix identity/sudo origin and cannot reinterpret prefix; repeat install preserves commitment; no removal/convergence claim. |
| **R2-SHIM-01** | Installer-managed, automatic CLI, standalone declared/self-derived deploy/remove/status/doctor/repair, and physical shim paths receive one complete context or verified mapping projection; a bare physical shim recovers exactly one no-follow invocation witness without PATH precedence, resolves the canonical current Unix account+UID or Windows account+SID through its exact OS observation surface, and rejects zero/multiple witnesses or a forged principal before dispatch; repair target derives from the committed principal; telemetry and manager-hint manifest/overlay consume custom A under ambient B, with no compiled-repo or manifest-environment fallback in normal product mode; shell and physical-shim trace output is exactly `A/trace.jsonl`, policy-commit metadata reads only A, unbound trace initialization fails, missing Git metadata does not fall back, and the A/B proof observes zero filesystem access under B; legacy H/R/carriers are consistency checks. Replacement/migration/recursive removal actions and trace rotation/retention semantics remain byte-frozen in R2. |
| **R2-GEN-01** | Dev/release `env.sh`, manager, Bash preexec, helper, generated `A/manager_hooks.yaml`, configuration, version, install-state, dependency, service, intended-principal PATH, and Lima known-hosts projections are prefix-relative or self-derived, encode/project the same context where consumed, and never treat a conflicting ambient B, repository path, or generated value as selection authority. |
| **R2-RUNTIME-01** | Dependency add/remove/current/list/sync, world-deps global/current inner leaves, runtime-family/Codex provisioning, config/policy proof producers and consumers, and gateway sync/status/restart leaves receive the exact context; member-dispatch/gateway Codex paths and synthetic-auth creation target the committed principal's account-database home. Rollback and synthetic-auth deletion remain R3. Runtime, policy, gateway, and provider semantics do not change. |
| **R2-LINUX-01** | Unix account+UID and context survive every release, dev-install, and provision sudo boundary: context-aware Substrate helpers validate the full argv carrier; arbitrary tools receive only exact context-derived argv after parent revalidation and no preserved environment; the ACL bridge receives no context and rejects every tuple except its exact three fixed mode/target/`substrate` combinations while preserving group-member enumeration. Linux unit environment carries H=A, R=A, commitment, and intended-principal projection; socket/drop-in and `ReadWritePaths` derive from A; only the fixed same-attempt Linux socket-restart unlink is exercised; focused/static proof passes in R2-2 and dedicated-host service/world/Codex product proof passes in R2-4. |
| **R2-DIAG-01** | Trace, shim, repair, world, host, health, world-deps, config/policy proof, gateway, Lima/WSL, and pipe diagnostics report selected host commitment, verified platform mapping/transport, and host platform-control root where applicable; direct mode constructs from declared/principal/OS Known Folder inputs, internal mode validates hidden argv and matching scrubbed projections, and no default-home/guest/pipe/`LIMA_HOME`/`LOCALAPPDATA` reconstruction is represented as authority. Normal live trace and policy-commit production use the entry-bound A projections; `SHIM_TRACE_LOG` is explicit diagnostic/test input only and cannot satisfy product proof. |
| **R2-MAP-MAC-01** | IH plus the VM selector and host account-database-derived `/.lima` control root performs only Stage-1 declared-instance realization; parents scrub/overwrite child `HOME`/`LIMA_HOME`, direct internal mismatch rejects, and PM is finalized from running-guest machine ID/account+UID+home before R2 guest projection. R2 fixes the future V1 SSH-UDS target from `A/sock/agent.sock` to `/run/substrate.sock`; backend auto-selection, `vsock-proxy`, TCP, ambient endpoints, and ambient Lima store cannot replace it. The validated R2 path stops before forwarder launch with an explicit R3 prerequisite, so socket unlink, `StreamLocalBindUnlink`, timeout kill/wait, handle-drop teardown, retry, and convergence remain exclusively R3 and are not exercised as R2 proof. Host commitment also binds the guest unit and future A-scoped known-hosts projection; shell/shim/replay factory callers are explicit; no path/principal equality. Static proof is labeled static and a native pre-existing-Lima no-forwarder A/B mapping-only run is assigned separately; macOS product transport remains pending R3. |
| **R2-MAP-WIN-01** | Windows account+SID host context survives dev/release/uninstall and `-NoAutoSource`; the release A/bin copy self-derives A under ambient B and every internal carrier is bound to the current account+SID, with forged-principal rejection. It binds one WSL instance, WSL-native account+UID+home, one normalized public-selected/default pipe, and an OS Known Folder control root. A canonical SID+registered-distro+machine-ID+pipe digest scopes the shared PID root; config/logs are under A and all paths are explicit, so `LOCALAPPDATA`/`USERPROFILE` cannot select them. Warm/forwarder/backend/status/doctor plus shell/shim/replay consume the same mapping. The forwarder validates PM/current token before `wsl -d`; its WSL child receives only PM-derived distro, normal-product guest target, and commitment despite conflicting ambient config/target/`WSLENV`. The fail-closed WSL guards remain byte-identical; creation, replacement, timeout kill, stop, PID deletion, and convergence remain R3; static proof is labeled static and native existing-instance A/B mapping-only proof is assigned without claiming provisioning. |
| **R3-LIFE-01** | R3-only candidate, rollback, manifest, managed-system cleanup, account-state restoration, crash-window, uninstall/reinstall, shim/payload/bin/cache/helper/unit/socket/platform-staging/forwarder unlink/drop/timeout/synthetic-auth cleanup, and unrelated-state preservation matrix. Referencing this gate from R2 transports a target only. |
| **R3-WIN-01** | R3-only native Windows two-prefix cleanup matrix: no wildcard removal; exact per-prefix versus shared state; version/bin/profile replacement, timeout rollback, uninstall order, WSL/forwarder/shim cleanup, partial install, and unrelated-state preservation. |

R2-0 cross-document checkpoint rules are: inventory counts and owner/packet columns must agree;
every R2 packet has an exact production/test allowlist; R2-4 has no production allowlist; every R3
action above stays exclusive to R3; host and Lima/WSL paths/principals are never equated;
`InstallBootstrapContextV1` commitment framing is canonical and unambiguous; and the serial graph is
acyclic. The exact next packet after a clean R2-0 commit is **A1.1d-5R2-1 — Host context
construction and Unix dev propagation**.

**Frozen planning validation record:** the inventory contains exactly 118 unique, contiguous rows
and dispositions: 19 owned by R2-1, 41 by R2-2, 35 by R2-3, one by R2-4, and 22 by R3. Each row
has exactly one approved edge class and one packet owner; the class totals are 16
`ContextConstruction`, 26 `ExplicitHostPropagation`, five `PrivilegeBoundaryPropagation`, 17
`PlatformMapping`, 13 `GeneratedProjectionConsumption`, 15 `DiagnosticProjection`, 22
`R3CleanupOnly`, and four `OutOfScope`. The frozen DAG is exactly
R1 -> R2-0 -> R2-1 -> R2-2 -> R2-3 -> R2-4 -> R3. Mechanical validation covers row-ID,
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
The final authority review then found two trace-specific ambient reads that the broad CLI/shim rows
had not made independently reviewable. PI-117 now assigns the common shell/global-trace carrier to
R2-1, and PI-118 assigns the physical-shim binding/reuse calls to R2-3. A normal entry binds exact
`A/trace.jsonl` plus policy Git directory A after IH/principal validation; unbound
`init_trace(None)` errors, missing policy metadata returns none without fallback, and
`SHIM_TRACE_LOG` remains explicit diagnostic/test input only. The R2-1 allowlist therefore adds
only `trace/src/{context.rs,util.rs}` and the named shell trace call sites; R2-3 adds only shim
`logger.rs` and `exec/policy.rs` beside its already-allowed shim entry. Trace
writer/rotation/retention and policy semantics remain frozen. GitNexus reported LOW risk for
`evaluate_policy`/`get_policy_git_hash` and MEDIUM direct test fan-out for
`TraceContext::init_trace`; no runtime edit occurred in R2-0.
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

The exact next packet is **A1.1d-5R2-1 — Host context construction and Unix dev propagation**,
followed by R2-2, R2-3, R2-4, and the audit-proven **R3**. After the implementation packets are
independently review-clean, rerun the complete Linux
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
review-clean through `6436289f`, `c519024b`, `de727091`, `717579b0`, and `83101dcb`. Their joint
production integration closeout has not begun, B3.1 and A1.3 are not dependency-ready, and A1 as a
whole remains incomplete and non-landable.

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
neither B1 nor B2.1 is production-complete before their joint closeout. The A1.2a-WB, A1.2a-S,
B1/B2.1-R0, B3.2a, and B3.2a-WA blockers are satisfied. The joint closeout has not begun; B3.1,
C1, and A1.2b are not ready. A1.2a, A1.2a-WB, and A1.2a-S are landed; the preserved broad A1.2
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
**not begun**. B3.1 dependency-ready: **no**. Seam promotions: **none**. Within the B corridor, the
next architectural packet remains the B1/B2.1 joint production integration closeout. The
repository's exact next packet is A1.1d-5R2-1 — Host context construction and Unix dev
propagation, followed by R2-2, R2-3, R2-4, and R3; only the joint closeout's Linux
product-smoke portion waits for those remediations and their required smoke, and its receipt and
supervisor semantics are not reopened.

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
