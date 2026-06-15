# TASKS-57: Internal Family-2 Host-Global Ingress Receive-Cursor And Sync-State Coordination

Source spec: [SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md](./SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md)  
Source plan: [PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md](./PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md)  
Source prior slice: [TASKS-51.md](./TASKS-51.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: proposed on `2026-06-12`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `57` have been reviewed and accepted as the bounded source of truth before implementation begins.

Current tree note:

1. Slice `51` already landed the bounded host-global inbox namespace and exact local materialization boundary.
2. Slice `56` already closed the current read-side and strict-control hardening seam.
3. The next honest roadmap gap from the remaining-scope note is therefore above local materialization: source-scoped host-global ingress lifecycle coordination.
4. This slice is intentionally narrower than broader federation or cross-host delivery. It only owns receive-cursor and sync-state coordination for exact local receipt into canonical host-inbox records.

## Execution Packets

This slice should run as four sequential `/incremental-implementation` sessions:

1. Packet 1 freezes the coordination contract.
2. Packet 2 lands durable coordination artifacts and validation.
3. Packet 3 lands exact ingress apply sequencing and replay handling.
4. Packet 4 proves coexistence, aligns docs, and runs the validation wall.

Do not collapse multiple packets into one session unless a review pass explicitly approves that compression after Packet 1.

## Packet 1: Coordination Contract Freeze

Session goal:

1. freeze the source-scoped coordination artifact and source key,
2. freeze the “persist host-inbox record before cursor advance” rule,
3. keep coordination state distinct from canonical host-inbox record truth.

### Tasks

- [ ] Task 1.1: Freeze the bounded source-scoped coordination artifact
  - Acceptance:
    - the slice docs define one exact coordination artifact for receive-cursor or sync-state truth
    - the artifact is clearly above canonical host-inbox record truth rather than a replacement for it
    - the slice docs explicitly state whether same-cursor replay is allowed and under what exact condition
  - Verify:
    - manual doc diff review
  - Files:
    - `llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md`
    - `llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md`
    - `llm-last-mile/TASKS-57.md`

- [ ] Task 1.2: Freeze the layering and non-goal contract
  - Acceptance:
    - the slice docs explicitly keep cross-host delivery, lease/lock coordination, and public operator UX out of scope
    - the slice docs explicitly preserve the landed `host_inbox -> local obligation -> router` responsibility split
  - Verify:
    - `rg -n "lease|lock|federation|public|router|host_inbox" llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md llm-last-mile/TASKS-57.md`
  - Files:
    - Slice `57` doc set only

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. one exact coordination artifact is frozen,
2. one exact cursor-advance rule is frozen,
3. the slice is still clearly narrower than delivery/federation work.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Durable Coordination Artifacts And Validation

Session goal:

1. add store-owned durable coordination artifacts,
2. validate source identity and cursor truth,
3. keep malformed or impossible coordination state fail-closed.

### Tasks

- [ ] Task 2.1: Add the bounded host-inbox coordination artifact model
  - Acceptance:
    - the runtime has one internal coordination record type with exact source key plus receive-cursor or sync-state fields
    - the type does not redefine host-inbox record materialization or local obligation ownership
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/host_inbox.rs`
    - one new bounded coordination module under `crates/shell/src/execution/agent_runtime/`
    - `crates/shell/src/execution/agent_runtime/mod.rs` only if export wiring is needed

- [ ] Task 2.2: Extend the state store with coordination persistence helpers
  - Acceptance:
    - the state store owns coordination artifact paths and bounded read/write helpers
    - invalid or malformed coordination artifacts fail validation
    - no ad hoc filesystem writer is introduced elsewhere
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/state_store.rs`
    - the bounded coordination module

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. durable coordination artifacts exist,
2. validation is exact and fail-closed,
3. coordination persistence is store-owned.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Exact Ingress Apply Sequencing And Replay Truth

Session goal:

1. add one bounded internal ingress apply path,
2. durably persist canonical host-inbox records before advancing source state,
3. make replay idempotent and mismatches fail closed.

### Tasks

- [ ] Task 3.1: Apply a valid new ingress cursor into exactly one canonical host-inbox record
  - Acceptance:
    - a valid ingress candidate with exact source/cursor truth persists exactly one canonical host-inbox record
    - the matching source coordination state advances only after that durable write succeeds
    - the task does not materialize a local obligation as part of ingress receipt
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/state_store.rs`
    - the bounded coordination module
    - one bounded host-side ingress helper under `crates/shell/src/execution/` only if needed

- [ ] Task 3.2: Make same-cursor replay idempotent
  - Acceptance:
    - replaying the same exact cursor and record identity does not create duplicate host-inbox artifacts
    - the runtime reports deterministic “already applied” truth instead of silently rewriting meaning
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/state_store.rs`
    - the bounded coordination module

- [ ] Task 3.3: Fail closed on non-monotonic or mismatched replay
  - Acceptance:
    - replay with an older cursor or mismatched record identity fails closed
    - invalid replay does not advance source coordination state
    - invalid replay does not create duplicate host-inbox records
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - `crates/shell/src/execution/agent_runtime/state_store.rs`
    - the bounded coordination module

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. new ingress applies exactly once,
2. exact replay is idempotent,
3. invalid replay fails closed,
4. cursor advancement never outruns durable local receipt.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Coexistence Proof, Docs Alignment, And Validation Wall

Session goal:

1. prove downstream layering still holds,
2. align docs with the landed Slice `57` boundary,
3. run the validation wall.

### Tasks

- [ ] Task 4.1: Prove coordination-layer coexistence with the landed materialization/router path
  - Acceptance:
    - ingress coordination stops at canonical host-inbox receipt
    - host-inbox materialization still consumes host-inbox records only
    - router-owned auto-attach still consumes local obligations only
  - Verify:
    - `cargo test -p shell host_inbox_materialization -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if coexistence assertions require that file
  - Files:
    - `crates/shell/src/execution/host_inbox_materialization.rs`
    - `crates/shell/src/execution/orchestrator_world_dispatch.rs` only if bounded coexistence assertions require it
    - adjacent tests

- [ ] Task 4.2: Align docs and run the final validation wall
  - Acceptance:
    - docs describe Slice `57` as receive-cursor/sync-state coordination only
    - docs preserve the rule that broader delivery/federation and public UX remain later work
    - formatting, clippy, targeted shell suites, and full workspace tests are green against the bounded file set
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell host_inbox_materialization -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if Packet 4 touched that seam
    - `cargo test --workspace -- --nocapture`
  - Files:
    - files touched by Tasks `4.1` and earlier packets
    - `llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md`
    - `llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md`
    - `llm-last-mile/TASKS-57.md`
    - `AGENT_ORCHESTRATION_GAP_MATRIX.md` only if bounded wording alignment is needed

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the coordination seam stays safely bounded,
2. downstream layering is still honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

1. After Packet 1, review whether the source-key and replay contract are exact enough before writing runtime code.
2. After Packet 2, review whether the durable artifact shape is still clearly above canonical host-inbox records.
3. After Packet 3, review whether the slice stayed out of delivery/federation semantics.
4. Packet 4 should close only after the final validation wall is green.
