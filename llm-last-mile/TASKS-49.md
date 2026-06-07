# TASKS-49: Internal Family-2 Host-Targeted Obligation Envelope And Wrong-Host Fail-Closed Boundary

Source spec: [SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md](./SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md)  
Source plan: [PLAN-49.md](./PLAN-49.md)  
Source prior slice: [TASKS-48.md](./TASKS-48.md)  
Phase: `TASKS`  
Execution model: four sequential `/incremental-implementation` sessions  
Status: drafted on `2026-06-07`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `49` have been reviewed and accepted as the bounded source of truth before implementation begins.

## Packet 1: Host-Targeting Envelope Contract Freeze

Session goal:

1. widen the obligation record only for bounded host-targeting truth,
2. freeze backward-compatible semantics when host-targeting fields are absent,
3. keep ingress metadata and host-global inbox work out of scope.

### Tasks

- [ ] Task 1.1: Add the bounded host-targeting fields to the canonical obligation artifact
  - Acceptance: the canonical obligation record preserves room for `origin_host_id` and `target_host_id`, round-trips them when present, and does not widen into `ingress_source_*`, `host_inbox`, or broader federation state.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 1.2: Freeze wrong-host semantics for the current local-only architecture
  - Acceptance: the spec-aligned runtime contract makes it explicit that a locally persisted obligation targeted at another host is a fail-closed local condition in the current architecture, while obligations with no `target_host_id` preserve current local-only behavior.
  - Verify:
    - targeted unit coverage for the chosen validation/classification helper
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - one new or existing bounded helper file under `crates/shell/src/execution/` or `crates/common/`

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the bounded host-targeting envelope is frozen,
2. backward compatibility for missing host-targeting metadata is preserved,
3. wrong-host semantics are explicit,
4. ingress-ready identity fields remain deferred.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Local Producer And Persistence Widening

Session goal:

1. propagate bounded local host-targeting truth into local obligation producers,
2. keep the state-store round-trip authoritative,
3. avoid reopening broader producer or ingress architecture.

### Tasks

- [ ] Task 2.1: Stamp bounded local host-targeting truth on locally produced obligations where the runtime knows it
  - Acceptance: local Family-2 obligation producers preserve exact session/backend/world truth and add bounded host-targeting metadata when the local runtime can state it exactly, without widening into cross-host ingress or global routing.
  - Verify:
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [ ] Task 2.2: Keep persistence and reload behavior compatible for untargeted legacy/local obligations
  - Acceptance: obligations with no `origin_host_id` or `target_host_id` continue to round-trip and remain usable by the landed Slice `48` paths, while new targeted records remain explanation-ready after reload.
  - Verify:
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. local producers can emit bounded host-targeting truth,
2. persistence remains authoritative and backward compatible,
3. no ingress metadata or host-global state path is introduced.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Router Wrong-Host Fail-Closed Evaluation

Session goal:

1. evaluate explicit host targeting before router-owned attach claim or launch,
2. fail closed for foreign-targeted local obligations,
3. preserve landed Slice `48` behavior for same-host and untargeted obligations.

### Tasks

- [ ] Task 3.1: Introduce an exact local-host check on the router-owned attach path
  - Acceptance: the router-owned auto-attach path compares explicit `target_host_id` against exact local host identity before attach launch and does not attempt attach when the obligation targets another host.
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - targeted unit coverage for the chosen local-host helper
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - one new or existing bounded helper file under `crates/shell/src/execution/` or `crates/common/`

- [ ] Task 3.2: Settle wrong-host obligations fail closed without mutating review state
  - Acceptance: foreign-targeted local obligations produce explanation-ready fail-closed outcomes, do not launch attach, do not silently reroute, and do not resolve or dismiss review state.
  - Verify:
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/auto_attach.rs`](../crates/shell/src/execution/agent_runtime/auto_attach.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) if outcome logging or producer-side tests require updates

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. wrong-host obligations cannot launch attach,
2. wrong-host reasons are explanation-ready,
3. review-state semantics are unchanged,
4. same-host and untargeted obligations preserve Slice `48` behavior.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align docs with the bounded host-targeting and wrong-host behavior,
2. keep `host_inbox` and ingress-ready identity work explicitly deferred,
3. run the validation wall.

### Tasks

- [ ] Task 4.1: Align docs with the landed Slice `49` boundary
  - Acceptance: docs describe Slice `49` as bounded host-targeting envelope plus wrong-host fail-closed behavior only, and do not imply `host_inbox`, remote ingress, or broader federation support has landed.
  - Verify:
    - manual diff review
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`docs/TRACE.md`](../docs/TRACE.md) if touched
    - [`llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md`](./SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md)
    - [`llm-last-mile/PLAN-49.md`](./PLAN-49.md)
    - [`llm-last-mile/TASKS-49.md`](./TASKS-49.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, and full workspace tests are green against the bounded Slice `49` file set; this task validates the slice and does not become an open-ended cleanup bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell auto_attach -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell obligation -- --nocapture`
    - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - none
  - Stop condition: if a validation command fails because Slice `49` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `49` boundary is safely bounded,
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
4. the slice has not widened into `host_inbox`, remote ingress, broader federation, or workflow-engine scope.

Reopen spec, plan, or tasks only if one of these is true:

1. bounded host-targeting cannot be landed without immediately adding ingress metadata or host-global inbox state,
2. local host identity cannot be defined without a broader public configuration or inventory contract,
3. wrong-host fail-closed behavior proves incompatible with the landed Slice `48` attach path unless broader router behavior changes,
4. backward compatibility for already-persisted obligations cannot be preserved without a larger obligation-ledger redesign.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether the slice stayed bounded to host-targeting envelope and wrong-host fail-closed behavior rather than widening into ingress or federation scope.
