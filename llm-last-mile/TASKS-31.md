# TASKS-31: Lazy Host Attach For Host-Rooted World Start

Source SOW: [31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)  
Source spec: [SPEC-31-lazy-host-attach-for-host-rooted-world-start.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)  
Source plan: [PLAN-31.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-31.md)  
Source lifecycle design: [DESIGN-world-worker-lifecycle-model.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: draft for review on 2026-05-29

## Execution Packets

This slice should be implemented as four separate sessions. Do not start the next packet until the current packet checkpoint is green.

- Packet 1 establishes the detached taxonomy and canonical obligation-ledger floor.
- Packet 2 makes obligations authoritative for review/status projection.
- Packet 3 adds router-owned automatic attach trigger and coalescing.
- Packet 4 lands attach execution, interop, docs, and the full validation wall.

## Packet 1: Detached Taxonomy And Obligation-Ledger Floor

Session goal:

1. freeze truthful detached host-session taxonomy,
2. add canonical obligation-ledger record shape and validation,
3. keep current inbox persistence as compatibility projection only.

### Tasks

- [ ] Task 1.1: Freeze specialized detached posture truth without reopening slice 30
  - Acceptance: runtime and status projection distinguish never-attached-yet, `parked_resumable`, and `awaiting_attention`; slice-30 attached world-start happy path remains unchanged; legacy `born_unattached` projection stays truthful if retained.
  - Verify: `cargo test -p shell orchestration_session -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

- [ ] Task 1.2: Introduce canonical obligation-ledger records and validators
  - Acceptance: a new durable obligation record shape exists with explicit classes, pending/resolved state, orchestration-session ownership, and attach-review projection inputs; invalid records fail at persistence boundaries; no separate canonical queue shape is introduced.
  - Verify: `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/mod.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. detached posture distinctions are pinned by tests,
2. a canonical obligation-ledger artifact exists and validates,
3. no slice-30 attached-path behavior regresses.

## Packet 2: Obligation Persistence And Projection

Session goal:

1. persist detached host-side follow-up work as obligations,
2. derive review/inbox and detached status from obligation truth,
3. preserve fail-closed world follow-up.

### Tasks

- [ ] Task 2.1: Persist eligible detached host-side work as obligations
  - Acceptance: eligible unresolved host-side work such as follow-up, approval, blocked, and fork-request states is written as obligation records under the orchestration session; worker events influence host posture only through obligation persistence; no synthetic prompt reconstruction is introduced.
  - Verify: `cargo test -p shell control -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Derive review/inbox projection and detached posture from obligations
  - Acceptance: unresolved attention-driving obligations project to readable review/inbox state and to truthful detached posture; resolving obligations updates the same canonical truth; detached or born-unattached world follow-up still fails closed until attach succeeds.
  - Verify: `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. canonical obligation persistence exists for the eligible classes,
2. review/status projection derives from obligations rather than ad hoc detached heuristics,
3. detached world follow-up still fails closed.

## Packet 3: Router-Owned Automatic Attach Trigger

Session goal:

1. evaluate eligible unresolved obligations for automatic attach,
2. coalesce automatic attach per orchestration session,
3. persist attach-episode claim/outcome truth.

### Tasks

- [ ] Task 3.1: Implement session-scoped attach-trigger evaluation and claim state
  - Acceptance: unresolved eligible obligations become attach candidates through router-owned evaluation; at most one obligation may hold the active attach claim for a session at one time; attach decisions persist explanation-ready outcomes.
  - Verify: `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 3.2: Prevent duplicate launches and preserve manual-reattach interop
  - Acceptance: sibling obligations in one session do not launch duplicate attach work; successful manual `reattach` may satisfy or supersede an automatic attach attempt; no duplicate backend launch occurs after the same session is already reattached.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. automatic attach is triggered from obligations rather than trace reconstruction,
2. one session produces at most one active attach episode,
3. manual `reattach` and automatic attach do not duplicate work.

## Packet 4: Attach Execution, Interop, Docs, And Validation

Session goal:

1. execute continuity-first and fresh-fallback attach through persisted truth,
2. preserve one coherent authority model across manual and automatic attach,
3. keep detached world follow-up fail-closed until ownership returns,
4. align docs and pass the validation wall.

### Tasks

- [ ] Task 4.1: Route automatic attach through persisted-truth continuity-first and fresh-fallback execution
  - Acceptance: automatic attach resolves through persisted attach truth, prefers continuity when valid, falls back to fresh attach only when continuity is unavailable but persisted truth remains valid, and fails closed before backend launch when attach truth is missing, drifted, or disallowed.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

- [ ] Task 4.2: Preserve fail-closed world follow-up and operator-visible recovery guidance
  - Acceptance: detached or never-attached-yet world follow-up remains non-routable until sanctioned host attach succeeds; operator-facing errors continue to direct users through `reattach`; status and toolbox surfaces stay coherent under the new obligation-ledger truth.
  - Verify: `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

- [ ] Task 4.3: Align planning docs and run the full validation wall
  - Acceptance: slice-31 docs match the landed specialized-path contract, including obligation-ledger authority, router-owned attach, continuity-first/fresh-fallback execution, and fail-closed world follow-up; format, lint, targeted suites, and full workspace tests pass; Linux manual smoke evidence is recorded.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - [`llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-31-lazy-host-attach-for-host-rooted-world-start.md)
    - [`llm-last-mile/PLAN-31.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-31.md)
    - [`llm-last-mile/TASKS-31.md`](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-31.md)
    - `llm-last-mile/CLOSEOUT-31-*.md`

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. automatic attach executes through the persisted attach contract with continuity-first/fresh-fallback behavior,
2. manual and automatic attach share one coherent durable authority model,
3. detached world follow-up stays fail-closed until attach succeeds,
4. docs and the full validation wall are green.
