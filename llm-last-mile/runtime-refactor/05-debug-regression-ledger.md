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
| **RG-AUTH-01** | Helper/session authority and orphaned helper behavior | **Unresolved architecture; narrow continuity behavior partially resolved** | Session/posture logic is still spread across `orchestration_session.rs`, `state_store.rs`, `agents_cmd.rs`, `control.rs`, and `async_repl.rs`; helper exit and owner/attached-client observations still participate in decisions. | Kill or orphan each episode kind after durable authority exists. Exact session, world binding, retained workers, receipts, and obligations remain. A stale episode cannot rewrite a newer authority revision. | A0, A1, A2, A3 |
| **RG-AUTH-02** | Process/socket liveness used as durable truth | **Unresolved** | `state_store.rs` still combines `is_authoritative_live()` with owner-PID checks in routing/status paths; private transports remain entangled with lifecycle paths. | Table-test every authority use of PID/socket/heartbeat/attached-client state: it may classify delivery availability but cannot create/delete/rebind/terminalize durable truth. | A0, A1, A2, A3 |
| **RG-CLOSE-01** | Private stop unreachable versus durable closeout | **Resolved baseline for exact retained stop; generalize without regression** | The debug chain proved exact retained source stop can succeed by detached durable closeout when private owner transport is unreachable; fork-child stop can use live private transport. | Exact valid target + unavailable private stop produces durable closeout when rules allow; live transport remains a fast path; invalid/ambiguous/mismatched target still fails closed; repeated closeout is idempotent. | A0, A2, B4 |
| **RG-RECEIPT-01** | `run_world_task` blocks until terminal exit | **Unresolved** | `execute_run_world_task_stream` registers a temporary active record, consumes through `ExecuteStreamFrame::Exit`, publishes terminal state, and only then returns; guard drop removes the active record. | Foreground returns `ActiveEphemeralTaskReceiptV1` after durable acceptance and supervisor handoff, before terminal exit; exact inspect/cancel works while active. | B2 |
| **RG-RECEIPT-02** | `continue_world_worker` blocks until terminal exit | **Unresolved; confirmed design deviation** | `execute_continue_world_worker_stream_for_turn_kind` consumes until `Exit`; `continue_world_worker` returns only afterward. Existing model-visible outcome has no active-run receipt or immutable policy commitment. | Continue returns `ActiveRetainedTurnReceiptV1` before exit; the host can perform same-session work or park; supervisor owns remaining stream; clean turn closeout preserves worker continuity. | B2, B3 |
| **RG-RECEIPT-03** | Active receipt inspection and caller-drop survival | **Unresolved** | Current active task identity is foreground-scoped and removed by guard drop; retained turns have no durable accepted active-run receipt. | An accepted receipt is durably persisted from valid runtime acknowledgement, survives caller drop/restart, and can be inspected by exact task/active-run ID before terminal exit. | B1, B2 |
| **RG-RECEIPT-04** | Ephemeral `needs_retained_followup` remains a terminal non-retained result | **Partially implemented; target receipt semantics unproven** | The lifecycle design and current terminal enum preserve `NeedsRetainedFollowup`, but the durable active receipt contract did not state how it closes without becoming retained work. | Ephemeral receipt closes with terminal result class `NeedsRetainedFollowup`; it promises no retained participant or continue route, creates no hidden retained worker/conversational obligation, and requires an explicit policy-checked spawn for ongoing work. | B1 |
| **RG-SUP-01** | Supervisor idempotent frame/event processing | **Unresolved** | Foreground stream loops currently own terminal observation; no durable dedupe owner exists. | Duplicate frames/events do not duplicate receipt transitions, terminal closeout, persisted worker events, or obligations; stale observers cannot overwrite newer state. | B2, C1 |
| **RG-SUP-02** | Supervisor restart and reconciliation | **Unresolved** | No first-class restart-safe supervisor owns non-terminal accepted receipts. | Startup claims non-terminal receipts, resumes observation or reconciles exact runtime truth, and never fabricates terminal success when truth is missing or ambiguous. | B2 |
| **RG-CANCEL-01** | Same-turn continue → cancel | **Unresolved** | Current retained cancel resolves worker state, `authoritative_live`, owner PID, `latest_run_id`, and `Running`; the blocking continue returns after that active window is gone. | In one host turn, continue returns accepted active-run ID and immediate cancel targets it. Final receipt is cancelled or pending closeout, never `stale_linkage` merely because the worker/owner parked. | B3, B4, E2 |
| **RG-CANCEL-02** | Cancel outcome semantics | **Unresolved** | Current retained cancel terminal enum only expresses `Cancelled`; no-active work is reported through target-not-cancelable/stale-linkage-shaped errors. | Prove all categories from `04`: live cancel, pending closeout, already terminal, no active work, owner unreachable, invalid target, binding mismatch, ambiguity, and policy denial. | B4 |
| **RG-MSG-01** | Typed retained-worker messaging and causation identity | **Unresolved; useful footholds only** | Typed payloads/events exist but remain mixed with prompt rendering, stream parsing, and foreground delivery. | Host-to-worker messages and worker-to-host events preserve exact target, thread, request/message/event causation, event class, ordering, and attention semantics across immediate and durable channels. | B3, C1 |
| **RG-OBL-01** | Obligation materialization before terminal exit | **Unresolved** | `continue_world_worker` saves a surfaced event and calls `persist_continue_world_worker_obligation` only after the full stream function returns. | Attention-driving event is durable and creates one canonical obligation before terminal `Exit`; duplicate event/frame replay creates no duplicate; terminal failure does not erase it. | B2, C1 |
| **RG-OBL-02** | Obligation as canonical truth; inbox/attention as projections | **Partially implemented, unproven** | Obligation records, inbox materialization, pending counts, and attach fields exist, but production timing is terminal-coupled and compatibility rows/counts still share state-store authority. | Rebuild inbox/auto-attach projections from obligations; host `AwaitingAttention` exactly follows unresolved attention-driving obligations; worker `AttentionPending` remains separate. | C1, C2 |
| **RG-ATTACH-01** | Auto-attach claim and router trigger | **Partially implemented, unproven** | Eligibility, claim, settle, wrong-host checks, and helper launch exist; trigger is spawned after terminal-coupled obligation creation and has not been proven in the target event flow. | Multiple eligible obligations coalesce to one session claim; wrong-host/policy denial fail closed; restart/retry is idempotent; manual reattach and auto-attach do not create duplicate owners. | C2, C3 |
| **RG-ATTACH-02** | Router must restore host ownership only | **Unresolved proof gate** | Current code launches a hidden owner helper, but no end-to-end proof establishes the permanent non-responsibility boundary. | Instrument the router entrypoint and assert it cannot submit prompts, approve, answer, fork, continue, cancel, or stop worker work; it records attach outcome and stops. | C3 |
| **RG-UAA-01** | Codex/UAA guest runtime realizability | **Partially resolved; keep as baseline** | World Codex config and validator require `/var/lib/substrate/world-deps/bin/codex`; the old host-NVM exit-127 diagnosis is historical for current placement-aware config. | World Codex rejects host-local binary paths and succeeds only with guest-visible runtime deps; host and world envelope kinds remain distinct. | D1 |
| **RG-CONFIG-01** | Per-worker projection identity and no sibling aliasing | **Unresolved; defensive scaffolding only** | Isolated Codex home/config exists, but projection authority and per-retained-worker ownership are not proven on the real runtime path. | Sibling retained workers cannot share or alias projected runtime config/home state unless an explicit policy and projection identity permit it. | D1, E3 |
| **RG-CONFIG-02** | Runtime-native files are projection, not authority | **Unresolved; defensive scaffolding only** | Runtime homes and workspace config files can exist independently of canonical projection truth; current bounded Codex auth/config seeding is a compatibility bridge. | `.codex`, `CODEX_HOME`, `config.toml`, `.mcp.json`, and equivalent native files can be rebuilt as non-secret projections from Substrate truth. Credentials enter only through launch-time gateway handoff; copied auth/config is named compatibility only; narrowed hints never replace broker/world-service enforcement. | E3 |
| **RG-CONFIG-03** | In-world gateway secure-FD credential handoff | **Unresolved; compatibility bridge only** | Current isolated `CODEX_HOME` copies host auth material and a bounded host-derived Codex config subset, reducing ambient leakage but not proving host credential authority -> secure FD -> in-world gateway flow. | A credential-requiring world UAA starts in contract-correct mode only when host authority delivers credentials once through a secure FD to the exact in-world Substrate gateway. The gateway consumes/closes it; the UAA and children never inherit the FD or raw secret; no host credential file is copied into world-visible runtime homes; traces contain non-secret handoff refs/states and redacted diagnostics only. Explicit compatibility copy mode is named/logged/non-promotable. | D1, E3, D3 |
| **RG-UAA-02** | Codex/UAA world command caging and side-effect mediation | **Unresolved** | Initial runtime is launched in world context and existing world-service commands have cage/fs/network enforcement, but Codex/UAA shell/edit/write/tool side effects are not forced through a per-operation broker; credential access is still bridged through copied runtime-home material. | For shell, patch/edit, direct write, write-capable MCP/tool, process spawn, and network: broker under receipt snapshot or fail closed. Credential-dependent provider access uses the in-world gateway session from secure handoff, never copied secrets. `cwd`, env, initial placement, or isolated home alone cannot pass. | D1, D2, E1, E3 |
| **RG-UAA-03** | `external_sandbox` means Substrate-owned mediation | **Unresolved; current behavior is unsafe as architectural proof** | `gateway/adapter_runtime.rs` defaults Codex to `agent_api.exec.external_sandbox.v1=true` and allows external sandbox execution, but no `WorldCommandExecutionBroker` proves Substrate owns every side effect and no secure gateway handoff proves Substrate owns credential bootstrap. | With external sandbox enabled, removing/interrupting broker capability or required gateway credential handoff makes world-UAA startup/side effects fail closed. With both active, every side effect joins to the accepted policy hash and every credentialed session joins to non-secret handoff evidence. | D2, D3, E3 |
| **RG-POLICY-01** | Dispatch-scoped `world_fs` narrowing carrier | **Unresolved** | Policy has `allow_capability_narrowing`; inventory overlays and boolean capability overrides exist; no request-scoped restricted `PolicyPatch.world_fs` reaches task/turn receipts. | Gate=false rejects patch; gate=true accepts only restricted world_fs narrowing; resulting hash/ref/reason is persisted on receipt/manifest. | E1, E2 |
| **RG-POLICY-02** | Narrowed policy enforcement and path containment | **Unresolved** | Existing inventory overlay subset check is equality-based; existing `PolicySnapshotV3` and world-service enforcement are useful primitives but not wired to dispatch-scoped narrowing. | Base `.` or `src` may narrow to `src/parser.rs`; file-to-directory/root, absolute paths, `..`, symlink escape, denial removal, disabled-write enablement, and weakened cage/fail-closed/enforcement are rejected. Allowed file succeeds and sibling file fails at runtime. | D2, E1, E4 |
| **RG-POLICY-03** | Immutable active snapshots and retained worker caps | **Unresolved** | Current task/continue outcomes do not commit immutable dispatch policy snapshots; retained manifests do not hold the target worker-cap contract. | Active run retains accepted hash across parent changes; future turn recomputes parent ∧ cap ∧ turn patch; parent broadening never broadens worker; parent narrowing narrows or invalidates next turn; fork inherits cap. | E2 |
| **RG-SYNC-01** | Host-visible write sync semantics | **Unresolved/open debug bucket** | Debug evidence warns that task completion does not prove a host-visible file; current docs/runtime distinguish host-visible overlay behavior from full isolation/reconciliation. | Matrix proves: host-visible allowed write visibility, host-visible denied write, full-isolation non-visibility before reconciliation, explicit reconciliation result, retained-turn behavior, and narrowed allowlist behavior. | E4 |
| **RG-WORKER-EXIT-01** | Non-zero Codex/UAA worker-turn exit semantics | **Unresolved/open debug bucket** | `codex exited non-zero` remains separate from repaired routing/stop seams; current blocking outcome can conflate runtime failure with authority/liveness loss. | Non-zero turn closes active receipt as durable `Failed` with exit diagnostics and policy/session/world joins; retained worker/session authority is preserved or explicitly invalidated for a stated lifecycle reason; no false success/attention loss. | B3, D3 |
| **RG-OBS-01** | First-class world-dispatch observability | **Unresolved** | `docs/TRACE.md` states `run_world_task` and `continue_world_worker` lack first-class internal world-dispatch trace families. | Accepted receipt, supervisor claim/restart, event/obligation, cancel, broker operation, policy hash, non-secret credential-handoff state, and terminal closeout are joinable by request/active-run/session/world IDs without logging secret payloads or secret-derived fingerprints. | B2, D3 |

## Resolved baselines that all tracks preserve

| Gate ID | Baseline | Required proof |
|---|---|---|
| **RG-BASE-01** | Public world-scoped start → reattach → stop | Exact session and world binding survive; stop reaches durable terminal truth even when transport posture changes. |
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

### S2 — Receipt, supervisor, obligation, cancel

1. Continue an exact retained worker that emits an attention event and remains active.
2. Receive accepted receipt before terminal exit.
3. Observe obligation before terminal exit.
4. Cancel by `active_run_id` in the same host turn.
5. Restart the supervisor during closeout and prove idempotent final state.

Covers: `RG-RECEIPT-02`, `RG-RECEIPT-03`, `RG-SUP-01`, `RG-SUP-02`, `RG-CANCEL-01`, `RG-OBL-01`, `RG-OBS-01`.

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

### S5 — One-time in-world gateway credential bootstrap

1. Resolve a credential source on the host without copying its payload into a world-visible path.
2. Deliver it once through a secure FD to the exact in-world Substrate gateway for the accepted world generation.
3. Prove the gateway consumes and closes the descriptor, the UAA child and descendants cannot inherit it, and no secret-bearing runtime-native file is created.
4. Start credentialed provider traffic through the gateway and join the session to non-secret handoff state.
5. Exercise duplicate consume, wrong gateway/world generation, expiry, and unavailable handoff; require fail-closed behavior.
6. Exercise the explicitly named compatibility copy mode separately and prove it is logged, has retirement metadata, and cannot satisfy contract-promotion evidence.
7. Inspect logs/traces/manifests for secret payloads, secret-bearing paths, and reusable secret-derived hashes; none may appear.

Covers: `RG-CONFIG-02`, `RG-CONFIG-03`, `RG-UAA-02`, `RG-UAA-03`, `RG-OBS-01`.

## Closeout rule

An implementation PR may mark a ledger row resolved only when:

1. its owning crosswalk seam has the correct owner and call path for that behavior;
2. the named permanent gate passes on the real path;
3. adjacent resolved baselines remain green; and
4. the evidence distinguishes durable success from transport/process success.
