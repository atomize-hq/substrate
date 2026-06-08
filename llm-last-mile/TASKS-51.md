# TASKS-51: Internal Family-2 Host-Global Inbox Layering And Local Obligation Materialization Boundary

Source spec: [SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md)  
Source plan: [PLAN-51.md](./PLAN-51.md)  
Source prior slice: [TASKS-50.md](./TASKS-50.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: Packet 4 closeout verified on `2026-06-08`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `51` have been reviewed and accepted as the bounded source of truth before implementation begins.

Current tree note:

1. Packets `1` through `3` are already landed and verified on the live repo before this session.
2. This Packet `4` pass owns only doc alignment plus the validation wall.
3. The checklist below remains the slice ledger; this session marks only Packet `4` tasks directly after the validation wall reran green.

## Execution Packets

This slice should run as four sequential `/incremental-implementation` sessions:

1. Packet 1 freezes the host-inbox contract.
2. Packet 2 lands persistence and local obligation materialization.
3. Packet 3 lands the bounded host-side execution boundary and coexistence proof.
4. Packet 4 aligns docs and runs the validation wall.

Do not collapse multiple packets into one session unless a review pass explicitly approves that compression after Packet 1.

## Packet 1: Host-Inbox Contract Freeze

Session goal:

1. add the bounded host-global inbox artifact contract,
2. freeze the minimum materialization-state vocabulary,
3. keep host-inbox ownership distinct from canonical local obligation ownership.

### Tasks

- [ ] Task 1.1: Add the bounded host-inbox artifact model under `agent_runtime`
  - Acceptance: the runtime has one internal host-inbox record type with exact identity, target-host, ingress, and materialization-state fields suitable for local obligation materialization, and the type does not redefine local review or auto-attach state as host-global truth.
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
  - Files:
    - one new bounded host-inbox module under `crates/shell/src/execution/agent_runtime/`
    - [`crates/shell/src/execution/agent_runtime/mod.rs`](../crates/shell/src/execution/agent_runtime/mod.rs) only if export wiring is needed

- [ ] Task 1.2: Extend the state store with host-inbox path and persistence helpers
  - Acceptance: the state store owns `SUBSTRATE_HOME/host_inbox/` path construction and bounded host-inbox read/write helpers, keeping write ownership centralized and avoiding ad hoc filesystem writes elsewhere.
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - the new host-inbox module

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the host-inbox artifact contract is frozen,
2. host-inbox persistence paths are store-owned,
3. host-inbox is explicitly not the canonical local obligation ledger.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Host-Inbox Persistence And Local Materialization

Session goal:

1. persist host-inbox records under `SUBSTRATE_HOME/host_inbox/`,
2. materialize valid records into exactly one canonical local obligation,
3. make repeated materialization idempotent and invalid cases fail closed.

### Tasks

- [ ] Task 2.1: Materialize a valid host-inbox record into one local obligation
  - Acceptance: a valid host-inbox record with exact local-boundary truth can produce exactly one local obligation under the target orchestration session, and the materialized obligation reuses the landed Slice `49`/`50` host-targeting and ingress/causation envelope rather than inventing a second local schema.
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
  - Files:
    - the new host-inbox module
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs) only if a bounded helper is required

- [ ] Task 2.2: Make host-inbox materialization idempotent
  - Acceptance: rerunning materialization for the same host-inbox record does not create duplicate obligations and instead preserves or reports the already-materialized outcome deterministically.
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - the new host-inbox module
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.3: Fail closed on wrong-host or invalid local-boundary truth
  - Acceptance: wrong-host host-inbox records or records missing required exact local-boundary truth do not create local obligations, and the runtime persists an explanation-ready failed-closed materialization outcome instead.
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
  - Files:
    - the new host-inbox module
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. valid records materialize exactly one local obligation,
2. repeated materialization is idempotent,
3. invalid local-boundary cases fail closed,
4. no remote sync or federation behavior is introduced.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Host-Side Execution Boundary And Router Coexistence

Session goal:

1. introduce one internal host-side materialization entrypoint,
2. prove that existing router-owned attach still runs only on local obligations,
3. keep host-inbox materialization outcomes explanation-ready.

### Tasks

- [ ] Task 3.1: Add one bounded internal host-side host-inbox materialization entrypoint
  - Acceptance: the repo has a non-test internal path that can discover pending host-inbox records for the local host and run local obligation materialization without becoming a public daemon/control surface.
  - Verify:
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if that file owns the entrypoint
  - Files:
    - one host-side execution file under `crates/shell/src/execution/`
    - the new host-inbox module
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 3.2: Prove `host_inbox -> local obligation -> existing router` coexistence
  - Acceptance: once a host-inbox record is materialized, existing review projection and router-owned auto-attach continue to consume only the local obligation ledger; no path consumes host-inbox records directly as router work.
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell host_inbox -- --nocapture`
  - Files:
    - the new host-inbox module
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs) only if a narrow coexistence regression needs coverage or sequencing
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) only if the entrypoint lives there

- [ ] Task 3.3: Emit explanation-ready host-inbox materialization outcomes without widening the slice
  - Acceptance: materialization success, idempotent re-run, and fail-closed outcomes are explanation-ready with exact host/session/obligation joins, but the packet does not widen into remote sync or public trace UX productization.
  - Verify:
    - targeted regression coverage for outcome serialization/logging helpers
    - `cargo test -p shell host_inbox -- --nocapture`
  - Files:
    - the new host-inbox module
    - one host-side execution file under `crates/shell/src/execution/`

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. one internal host-side materialization entrypoint exists,
2. router-owned auto-attach still consumes obligations only,
3. host-inbox materialization outcomes are explanation-ready,
4. no public daemon UX or federation workflow has been introduced.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align docs with the landed Slice `51` boundary,
2. keep broader cross-host delivery and federation work explicitly deferred,
3. run the validation wall.

### Tasks

- [x] Task 4.1: Align docs with the landed Slice `51` host-global inbox boundary
  - Acceptance: docs describe Slice `51` as host-global inbox layering plus exact local obligation materialization only, preserve the rule that local obligations remain canonical local truth, and do not imply remote sync, lease coordination, or public host-inbox UX have landed.
  - Verify:
    - manual diff review
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`docs/TRACE.md`](../docs/TRACE.md) if touched
    - [`llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md`](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md)
    - [`llm-last-mile/PLAN-51.md`](./PLAN-51.md)
    - [`llm-last-mile/TASKS-51.md`](./TASKS-51.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests are green against the bounded Slice `51` file set; this task validates the slice and does not become an open-ended cleanup bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell host_inbox -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if Packet 3 touched that seam
    - `cargo test -p shell auto_attach -- --nocapture` if Packet 3 touched coexistence with router sequencing
    - `cargo test --workspace -- --nocapture`
  - Files:
    - none
  - Stop condition: if a validation command fails because Slice `51` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `51` boundary is still safely bounded,
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
4. the slice has not widened into remote sync, lease coordination, or broader federation scope.

Reopen spec, plan, or tasks only if one of these is true:

1. the host-inbox layer cannot be landed without replacing or duplicating the canonical local obligation ledger,
2. idempotent local materialization cannot be preserved without a broader local obligation redesign,
3. router coexistence requires the router to consume host-inbox records directly,
4. the slice cannot land without immediately introducing broader cross-host sync or federation-delivery machinery.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether the slice stayed bounded to host-global inbox layering plus exact local materialization rather than widening into remote sync, lease coordination, or broader federation scope.
