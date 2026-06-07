# TASKS-50: Internal Family-2 Ingress-Ready Obligation Identity And Causation Envelope

Source spec: [SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md](./SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md)  
Source plan: [PLAN-50.md](./PLAN-50.md)  
Source prior slice: [TASKS-49.md](./TASKS-49.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: drafted on `2026-06-07`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `50` have been reviewed and accepted as the bounded source of truth before implementation begins.

## Execution Packets

This slice should run as four sequential `/incremental-implementation` sessions:

1. Packet 1 freezes the six-field ingress-ready envelope contract and backward-compatible persistence boundary.
2. Packet 2 widens local producers and persistence to preserve exact local ingress and request-causation truth.
3. Packet 3 either threads exact event/message causation through the current producer seam or explicitly freezes those fields as absent when exact truth is unavailable.
4. Packet 4 aligns docs and runs the validation wall.

Do not collapse these packets unless a later validation pass proves one is empty on the live tree.

## Packet 1: Ingress-Ready Envelope Contract Freeze

Session goal:

1. widen the canonical obligation record only for the six deferred ingress-ready fields,
2. freeze backward-compatible semantics when those fields are absent,
3. define the no-synthesis rule for exact event/message causation.

### Tasks

- [ ] Task 1.1: Add the six deferred ingress-ready fields to the canonical obligation artifact
  - Acceptance: the canonical obligation record preserves room for `ingress_source_kind`, `ingress_source_id`, `ingress_received_at`, `causation_event_id`, `causation_message_id`, and `causation_request_id`, round-trips them when present, and does not widen into `host_inbox`, remote ingress materialization, or broader federation state.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 1.2: Freeze exactness and no-synthesis semantics for canonical causation fields
  - Acceptance: the slice makes it explicit that `causation_event_id` and `causation_message_id` are optional exact-identity fields, `thread_id` is not silently reinterpreted as message identity, and missing exact truth remains absent rather than synthetic.
  - Verify:
    - targeted unit coverage for the chosen validation/classification helper behavior
    - `cargo test -p shell obligation -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - one new or existing bounded helper file under `crates/shell/src/execution/` or `crates/common/` only if strictly required

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the six-field ingress-ready envelope is frozen,
2. backward compatibility for missing fields is preserved,
3. exactness and no-synthesis rules are explicit,
4. `host_inbox` and remote ingress remain deferred.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Local Producer And Persistence Widening

Session goal:

1. stamp exact local ingress truth into canonical obligation fields where the current producer already knows it,
2. stamp exact local request causation into canonical obligation fields where the current producer already knows it,
3. keep legacy/local obligations compatible when the new fields are absent.

### Tasks

- [ ] Task 2.1: Stamp exact local ingress truth on locally produced obligations where the runtime already knows it
  - Acceptance: local Family-2 obligation producers preserve exact session/backend/world truth and add bounded ingress metadata such as `ingress_source_kind`, `ingress_source_id`, and `ingress_received_at` only when the current local runtime can state them exactly, without widening into host-global ingress or cross-host routing.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Stamp exact local request causation where the current producer already knows it
  - Acceptance: locally produced obligations preserve exact request/run identity in `causation_request_id` when the current producer already has it, while preserving current behavior for obligations that do not yet have exact event/message identity.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)

- [ ] Task 2.3: Keep persistence and reload behavior compatible for obligations with no new ingress-ready metadata
  - Acceptance: obligations that do not carry the new ingress or causation fields continue to round-trip and remain usable by the landed Slice `49` paths, while new records remain explanation-ready after reload.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. local producers can emit exact ingress truth when known,
2. local producers can emit exact request-causation truth when known,
3. persistence remains authoritative and backward compatible,
4. no host-global ingress or remote delivery path is introduced.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Exact Event/Message Causation Threading Or Explicit Absence Freeze

Session goal:

1. preserve exact event/message causation only when the current producer seam surfaces it directly,
2. keep those fields absent when no exact truth exists,
3. prevent `thread_id` or payload-shape guesses from becoming synthetic canonical causation ids.

### Tasks

- [ ] Task 3.1: Surface exact worker event/message causation only if the current producer seam already has it
  - Acceptance: if the retained-worker event seam already surfaces one exact event or message identifier, the persisted obligation canonicalizes it into `causation_event_id` or `causation_message_id` without reopening broader messaging design; otherwise the slice leaves the corresponding field absent.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture` only if this task widens the event contract
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs) only if narrowly required

- [ ] Task 3.2: Prove the no-synthesis boundary for missing exact event/message truth
  - Acceptance: missing exact event/message identifiers remain `None`, `thread_id` remains thread identity only, and no synthetic placeholder ids are persisted into canonical obligation fields.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. exact event/message causation is preserved only when surfaced directly,
2. missing exact event/message truth remains absent,
3. `thread_id` is not promoted to canonical message identity,
4. no broader retained-worker messaging redesign was required.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align docs with the ingress-ready identity-envelope widening,
2. keep `host_inbox` and remote ingress materialization explicitly deferred,
3. run the validation wall.

### Tasks

- [ ] Task 4.1: Align docs with the landed Slice `50` boundary
  - Acceptance: docs describe Slice `50` as ingress-ready local identity-envelope widening only, and do not imply `host_inbox`, remote ingress, or broader federation support has landed.
  - Verify:
    - manual diff review
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`docs/TRACE.md`](../docs/TRACE.md) if touched
    - [`llm-last-mile/SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md`](./SPEC-50-internal-family-2-ingress-ready-obligation-identity-and-causation-envelope.md)
    - [`llm-last-mile/PLAN-50.md`](./PLAN-50.md)
    - [`llm-last-mile/TASKS-50.md`](./TASKS-50.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests are green against the bounded Slice `50` file set; this task validates the slice and does not become an open-ended cleanup bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture` if Packet 3 touched that seam
    - `cargo test --workspace -- --nocapture`
  - Files:
    - none
  - Stop condition: if a validation command fails because Slice `50` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `50` boundary is safely bounded,
2. docs and runtime truth are honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh `spec-driven-development` restart.

Proceed directly to the next packet only when:

1. the current packet's verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into `host_inbox`, remote ingress materialization, or broader federation scope.

Reopen spec, plan, or tasks only if one of these is true:

1. the six-field envelope cannot be landed without immediately adding host-global inbox or remote delivery behavior,
2. exact event/message identity cannot be preserved honestly without a broader retained-worker messaging redesign,
3. persistence compatibility for already-persisted obligations cannot be preserved without a larger obligation-ledger redesign,
4. router or review behavior changes become necessary just to preserve the new canonical fields.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether the slice stayed bounded to ingress-ready local identity-envelope widening rather than widening into `host_inbox`, remote ingress materialization, or broader federation scope.
