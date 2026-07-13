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
| **RG-HOME-01** | Private home creation and world capability parity | **Unresolved product gate; A1.1d-1 through A1.1d-4 are preserved and A1.1d-5 is focused-review clean on Linux** | The A1.1d authority transaction range is preserved through `ca9429e5`, and focused private-home implementation review is clean through `faabed16`, but integrated Linux and native macOS closeout remain open. Unsafe ACL rejection is valid negative security evidence, not positive product smoke. The exact parked-successor integration hold permits A1.1e facade/resolution work without closing this gate or A1.1d. | Prove the trusted parent is descriptor-bound and has the expected owner/type, no other-principal write authority, no foreign ACL grant, and stable physical identity; prove create or `AlreadyExists` convergence followed by the first no-follow open and accepted identity captured only from that descriptor; prove exact owner/`0700`/type/ACL/required-filesystem validation; prove legitimate concurrent creators deterministically converge; prove replacement after descriptor acquisition and before later publication fails closed; prove unsafe other-principal parent permissions, foreign parent/child ACLs, symlink candidates, wrong owner/mode/type, and validation uncertainty fail before descendant or authority writes; prove no path-based operation occurs after binding and existing invalid roots remain byte/metadata unchanged. Do not claim atomic create-and-bind, prevention of malicious same-UID/root substitution before first open, or root as an adversary the user process can defeat. Compare effective policy, world request, network routing, filesystem enforcement plan, allow/deny lists, isolation/host-visibility flags, and policy hashes, then run complete Linux and native macOS product smoke proving filesystem, network, config/dependency, gateway, runtime, shim, replay, trace/diagnostic, PTY/non-PTY, and lifecycle/world-binding parity with no feature removal. | A1.1d-5 |
| **RG-AUTH-01** | Helper/session authority and orphaned helper behavior | **Unresolved architecture; narrow continuity behavior partially resolved** | The A0 inventory in `02-seam-crosswalk.md` traces initial attached-posture birth, home/env/CWD-selected state/config/policy/inventory, helper-plan payload plus mode and parent-role selection, launch/join coordination, exit/retry, startup-prompt and auto-park delivery, process-local startup acceptance, private-stop-ownership mode, public-stop delivery gating, terminal-loss park/stop choice, parking/reconciliation, fork-successor allocation, resumed-turn release, auto-attach verification, endpoint-registration startup failure, stream/process completion, whole-session snapshot arbitration, process-local snapshot-write arbitration, episode-local replacement lineage, and world-binding birth/repair/replacement. The parked-successor diagnostic additionally proves that current `Active`/`ParkedResumable` authority survives with no attached episode and that a reconstructed successor snapshot is not authority. Session/posture logic remains spread across `orchestration_session.rs`, `state_store.rs`, `agents_cmd.rs`, `control.rs`, `async_repl.rs`, and the in-world member registry. | Kill or orphan each episode kind after durable authority exists. Exact session, parked posture, world binding, retained workers, receipts, and obligations remain. Process/helper/prompt state cannot create or regress ownership, episode absence does not erase parked authority, and a stale episode cannot rewrite a newer authority revision. | A0, A1, A2, A3 |
| **RG-AUTH-02** | Process/socket liveness used as durable truth | **Unresolved** | A0 classifies the exact PID/ownership/attachment/heartbeat/socket and ambient-host consumers: constructor-created attached truth, posture/session discovery, helper plan/mode/parent-role gating, participant and parent-session snapshot freshness, process-local snapshot-write arbitration, relative/fallback/re-read authority and workspace paths, effective-config env overrides, UID-selected binding birth/repair/replacement, process-local dispatch and auto-park registries, toolbox cancel's PID-filtered preflight, retained-runtime reuse/exit/replacement, naming-prefix continuity, daemon-memory admission/routing, helper reconciliation, private-stop-ownership-gated startup/parking, public-stop delivery gating, fork/resume PID-zero sentinels, endpoint/startup-prompt state used as durable truth, remote member startup around transport acceptance, fork lineage gated by stop-socket publication, lossy/long-path endpoint identity, last-writer toolbox rebind, hostname-targeted auto-attach, and private-stop fallback. The exact blocker shows `PID=0`, no handle, completed prior prompt, and pending successor prompt are non-authoritative observations. | Table-test every authority use of PID/socket/heartbeat/attached-client/helper/handle/readiness/prompt/ambient-path state and competing process-local writers: availability may classify delivery, but neither presence nor absence may create, delete, regress, rebind, or terminalize durable truth. Episode absence must not erase current parked authority. | A0, A1, A2, A3 |
| **RG-AUTH-03** | Revision-bound host transition intent proof | **Unresolved** | `HiddenOwnerHelperLaunchPlan` currently carries authoritative `Start`/`Attach`/`ResumeOneTurn` mode, identity, lineage, workspace/world binding, descriptor, attach contract, resume handle, and input through arbitrary JSON. Plan load/removal and helper execution do not join a durable authority revision or greenfield-certificate-proven expected absence to a unique replay-safe intent. The parked-successor regression proves stale lifecycle/world-binding rejection is correct: pre-A1.1d `c800436d` and pre-rejection `f73e8a81` passed only while stale whole-session overwrite remained possible; `bdb1796d` and later revisions fail closed. A1.1d-1 through A1.1d-4 remain preserved and A1.1d-5 is focused-review clean on Linux, but integrated and native macOS closeout remain open; no seam promotion is claimed. | First prove A1.1 primitives without claiming production Start: genuinely absent versus initialization-pending, valid-existing, unsupported-pre-A1, and corrupt/unsupported roots; the same physical bootstrap home persisted in marker/root/certificate and used for StateStore/config/policy/inventory, including copied-home rejection; recognized temp cleanup and fail-closed unknown temps across crashes before/after temp fsync and rename; component-by-component no-follow traversal and complete safe enumeration of both pre-A1 authority collections, including ancestor identity/owner/mode/ACL and scan-to-publication replacement; any artifact rejected without parsing/conversion and any unreadable or unsafe collection failed closed. Every pre-A1 read-decide-write transaction must retain the opened physical root, descendants, exact identity, activation observation, and root lock through final `fsync`; a subprocess rename/replacement/rebind after lock acquisition must fail closed with the replacement root untouched and no successful transaction outcome. Post-activation writers reject before mutation and preactivation writers serialize on that same root lock. Exact retry treats reservations, tombstones, intents, issuer indexes, application journals, and object indexes as semantic occupancy and joins only complete matches. Rotation and retirement revalidate the complete reconciled candidate immediately before publication. Crash/restart, contention, temp reconciliation, publication durability, and exact retry converge. Then prove the A1.2 API alone can accept `ExpectedAbsent` only from the verified greenfield certificate, empty pre-A1 collections, and exact absence of the complete namespace-map key; it atomically issues its self-owned reservation and creates initial authority only while applying that intent. A1.3/A1.4 must still prove exclusive real public CLI/REPL/auto-attach adoption. Prove `Attach`/`ResumeOneTurn` from exact current parked or other enumerated authority against exact revision/hash with exact caller/source/target lineage, bootstrap-home/workspace/world binding, descriptor/attach/resume/policy/input commitments, unique intent/request identity, fixed expiry, canonical payload hashes, and retained transport reprojection. Application preserves lifecycle identity and world binding, revision-authorizes successor lineage and immutable input handoff, and never derives eligibility from PID/helper/handle/readiness/prompt state. Exercise every intent/input state and crashes after initialization marker/key/root publication, key publication/root selection/retirement, object publication, issue, plan load/removal, claim, authority commit, input acceptance, `ReleaseEligible` root commit, transport deletion, object-directory fsync, and `Released` root commit with transport present/already absent. Exact retry joins without a second namespace, authority revision, participant allocation, or delivery. Stale revisions, substituted plans, cross-kind refs, mismatched hash/identity/home/binding/descriptor, missing/copied certificate, compatibility-derived absence, pre-A1 artifacts, missing keys, expired intent, superseded claim, and conflicting replay fail closed. A1.3 must prove exclusive real public CLI/helper/REPL parked-turn and explicit-reattach adoption before the open, mandatory `RG-BASE-01` gate can close. The bounded A1.4 auto-attach producer uses the same contract. A1 closes only its scoped clauses of `RG-AUTH-01`/`RG-AUTH-02`; those ledger-wide gates remain unresolved for A2/A3. | A1.2, A1.3, A1.4 |
| **RG-CLOSE-01** | Private stop unreachable versus durable closeout | **Resolved baseline for exact retained stop; generalize without regression** | The debug chain proved exact retained source stop can succeed by detached durable closeout when private owner transport is unreachable; A0 records that bounded recovery and separately records public host stop's active-session delivery gate, where missing/refused transport blocks closeout. Neither current behavior is treated as the target owner model. | Exact valid target + unavailable private stop produces durable closeout when rules allow; live transport remains a fast path; invalid/ambiguous/mismatched target still fails closed; repeated closeout is idempotent. | A0, A2, B4 |
| **RG-EVENT-01** | Runtime event/frame identity, ordering, and exact terminal event | **Partially resolved: B0 producer clauses landed; B2.1 consumer clauses open** | B0 landed the canonical V1 carrier in `common`/`transport-api-types` and producer assignment in ordinary `world-service` task streams plus retained launch/turn streams. Each accepted producer stream has one non-empty UUIDv7 stream ID; all frames share one positive gap-free frame counter; semantic Event/Exit items share one positive event counter and stable UUIDv7 IDs; Exit is the final semantic event and exactly matches its terminal identity. Canonical clone/re-emission preserves bytes. Concurrent stdout/stderr assignment is serialized through enqueue. Typed host decoding forwards identity unchanged, rejects missing/malformed runtime identity, and does not convert Error, EOF, stream exhaustion, or diagnostic non-live state into terminal truth. B2.1 durable replay, duplicate/gap/reorder/conflict rejection, restart observation, and journaling do not exist yet. | Preserve all B0 producer and negative-host gates. B2.1 must make byte-identical replay a no-op, reject conflicting reuse, gaps, reorder, and frames after terminal, and prove restart-safe durable observation without treating EOF/error/process state as completion. | B0, B2.1 |
| **RG-RECEIPT-01** | `run_world_task` blocks until terminal exit | **Unresolved** | `execute_run_world_task_stream` registers a temporary active record, consumes through `ExecuteStreamFrame::Exit`, publishes terminal state, and only then returns; guard drop removes the active record. | After B1 acceptance and B2.1 durable handoff, B2.2 returns `ActiveEphemeralTaskReceiptV1` before terminal exit; exact inspect/cancel works while active. Blocking compatibility during B2.1 is permitted and is not closure. | B2.2 |
| **RG-RECEIPT-02** | `continue_world_worker` blocks until terminal exit | **Unresolved; confirmed design deviation** | `execute_continue_world_worker_stream_for_turn_kind` consumes until `Exit`; `continue_world_worker` returns only afterward. Existing model-visible outcome has no active-run receipt or immutable policy commitment. | After B1 acceptance, B2.1 observation, and C1 event truth, B2.2 returns `ActiveRetainedTurnReceiptV1` before exit; B3.2 preserves clean-turn worker continuity and the host can continue or park. | B2.2, B3.2 |
| **RG-RECEIPT-03** | Active receipt inspection and caller-drop survival | **Unresolved** | Current active task identity is foreground-scoped and removed by guard drop; retained turns have no durable accepted active-run receipt. | B1 preallocates a proposed acceptance-record ID in a typed task/turn request context, but exposes no accepted record until the matching B0 runtime acknowledgement durably creates exact task/active-run acceptance; world-service retains the context for later B3.1 events. When supplied by the host-transition owner, the same context/record carries the immutable equality-only intent/revision/payload/transition-run correlation. B2.1 survives caller drop/restart and exact inspect finds acknowledged work before terminal; B2.2 later exposes early return. | B1, B2.1, B2.2 |
| **RG-RECEIPT-04** | Ephemeral `needs_retained_followup` remains a terminal non-retained result | **Partially implemented; target receipt semantics unproven** | The lifecycle design and current terminal enum preserve `NeedsRetainedFollowup`, but the durable active receipt contract did not state how it closes without becoming retained work. | B2.1 closes the ephemeral receipt with terminal result class `NeedsRetainedFollowup`; it promises no retained participant or continue route, creates no hidden retained worker/conversational obligation, and requires an explicit policy-checked spawn for ongoing work. | B2.1 |
| **RG-SUP-01** | Supervisor idempotent frame/event processing | **Unresolved; B0 identity prerequisite satisfied** | B0 now supplies exact stable stream/frame/event/terminal identity from every in-scope producer path, but foreground stream loops still own observation and no durable dedupe/journal owner exists. B0 deliberately performs no consumer-side duplicate, gap, reorder, or restart handling. | B2.1 makes exact duplicate frames/events no-ops, rejects gaps/reorder/conflicts, prevents stale observer writes, and avoids duplicate receipt/terminal/journal transitions; C1 proves duplicate replay creates no duplicate obligation. | B0, B2.1, C1 |
| **RG-SUP-02** | Supervisor restart and reconciliation | **Unresolved** | No first-class restart-safe supervisor owns non-terminal accepted receipts. | At B1 acceptance, B2.1 establishes one no-gap durable journal path. Startup claims its non-terminal records, resumes B0 observation from the durable cursor through exact producer replay/reconciliation, and never fabricates terminal success or a complete cut when exact terminal truth is missing, the runtime exited without it, or truth is ambiguous. | B2.1 |
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
| **RG-OBS-01** | First-class world-dispatch observability | **Unresolved; B0 carrier clause satisfied** | B0 now exposes exact producer-originated stream/frame/event/terminal identity on ordinary and retained runtime frames, with strict decoding and canonical byte proof. `docs/TRACE.md` still states `run_world_task` and `continue_world_worker` lack first-class internal world-dispatch trace families, and no acceptance, journal, semantic, obligation, cancel, broker, policy, or credential-handoff joins landed in B0. | Remaining clause owners are explicit: B1 acceptance, B2.1 supervisor claim/restart/journal/terminal observation, B3.1 producer semantic normalization plus the typed worker event, C1 materialization/cut, B4 cancel, D2 broker operations, E2 policy commitments, and E3 non-secret credential-handoff state. D3 alone owns final end-to-end integration closure, after every earlier clause is landed, proving those facts join by request/active-run/session/world IDs without secret payloads or secret-derived fingerprints. No earlier packet may claim the whole gate closed. | B0, B1, B2.1, B3.1, C1, B4, D2, E2, E3, D3 |

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
materialization/completeness before A1.2 resumes. This moves no A2/A3 ownership, permits the
foreground to remain blocking until B2.2, and closes none of `RG-EVENT-01`, `RG-RECEIPT-03`,
`RG-SUP-01`, `RG-SUP-02`, `RG-MSG-01`, `RG-OBL-01`, or `RG-OBL-02` by documentation alone. The
crash matrix includes startup evidence,
snapshot publication/orphan staleness, terminal exact retry, and payload deletion/object-directory
`fsync` windows.

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
`0700` and no foreign-principal ACL. The blocked fixture/bootstrap path and its effective/default
ACL ancestry must be captured to determine whether the incompatible ACL was deliberately created,
inherited, produced by normal repository/installation setup, or environmental contamination. If a
normal supported installation path receives that ACL by default, treat it as a product compatibility
decision; do not dismiss it as fixture-only and do not weaken ACL enforcement without an explicit
contract change.

The corrected A1 V1 identity boundary starts at the first successful no-follow child open beneath
the retained, validated parent. Candidate creation or `AlreadyExists` convergence precedes that
boundary; all later operations remain descriptor-relative and later replacement fails closed.
Neither malicious root nor malicious same-UID substitution before the first open is in scope, and
no portable atomic create-and-bind or privileged-broker claim is part of `RG-HOME-01`.

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
branch. Its resumable post-turn closure is sequencing-blocked on the B0 -> B1 -> B2.1 -> B3.1 ->
C1 corridor. Do not restore, modify, or resume A1.2 before that corridor lands. A1.3 is not
dependency-ready. B0 was the exact next implementation packet and is now landed; B1 is the next
dependency-ready packet. A1 as a whole remains incomplete and non-landable.

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
`RG-OBS-01` are satisfied. B2.1 consumer clauses remain open. B1 is dependency-ready; B2.1,
B3.1, C1, and A1.2 are not. The preserved A1.2 checkpoint remains untouched at
`18bea80b75ad2c59c7b635851b14552e380585f2`. No seam is promoted.

## Baseline behaviors that all tracks preserve

These rows define permanent behavior contracts, not current pass claims. Any row explicitly marked
open remains a blocking regression gate until its named owner and real-path proof close it.

| Gate ID | Baseline | Required proof |
|---|---|---|
| **RG-BASE-01** | Public world-scoped start → turn/reattach → stop; **open, blocking, and not waived** | A1.2 must supply exact parked-successor `Attach`/`ResumeOneTurn` application and A1.3 must adopt it on the real public CLI/helper/REPL path. Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. Stale lifecycle/world-binding overwrite remains rejected. Prove unsafe foreign-principal ACL state fails closed separately, then run the positive smoke against an owner-only private bootstrap home with mode `0700` and no foreign ACL. This gate is neither permanently expected to fail nor successful until that real-path wall is green. |
| **RG-BASE-02** | REPL first-dispatch `run_world_task` binding repair | Correct binding succeeds; stale/mismatched world generation fails closed; no generic binding synthesis on unrelated surfaces. |
| **RG-BASE-03** | Parked host ordinary-command and continuity parity | Unprefixed `ls`/`pwd` remain usable; policy-required `cd ../` cage denial remains; later targeted host turn reuses session/UAA continuity; public CLI parity stays green. |
| **RG-BASE-04** | Retained spawn/fork/exact continue/exact stop plus ambiguity close | Exact source and child handles route correctly; backend-only follow-up with multiple retained workers fails closed; source detached stop and child live-transport stop remain valid. |

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

1. Continue an exact retained worker and prove the runtime emits one B0 stream with monotonic frame
   and event sequences plus an exact terminal event ID/sequence.
2. Persist the B1 accepted retained-turn identity from a pre-terminal acknowledgement, hand the
   stream to B2.1, and prove the foreground may remain a blocking waiter without retaining
   observation ownership.
3. Normalize one supported provider attention shape before `AgentEvent` construction, emit one
   B3.1 event with exact target/run/thread/class/attention/causation identity, and prove B2.1
   journals it durably and C1 materializes one obligation before the terminal event. Prove an
   ambiguous or malformed attention-driving provider shape fails before emission rather than
   becoming progress or no-attention, and prove host JSON-pointer parsing cannot repair it. For
   every post-ack retained `Event` frame, prove either the typed member reaches the journal/cut or
   the stream and Complete cut fail closed; no event is silently dropped or left untyped.
4. Replay duplicate frames/events, restart the supervisor during closeout, and prove the journal,
   receipt, obligation, ledger revision, watermark, and terminal state remain idempotent.
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
