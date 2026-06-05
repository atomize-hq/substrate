# TASKS-45: Internal Retained Host Fork Command Bootstrap

Source spec: [SPEC-45-internal-retained-host-fork-command-bootstrap.md](./SPEC-45-internal-retained-host-fork-command-bootstrap.md)  
Source plan: [PLAN-45.md](./PLAN-45.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packets `1`-`3` are landed on the current tree; Packet `4` remains open on `2026-06-05`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `45` have been reviewed and accepted as the bounded source of truth before implementation begins.

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 froze the typed host `fork_command` contract and the deny-by-default policy surface on the live tree.
- Packet 2 landed deterministic rendering and exact-source bootstrap preparation on the live tree.
- Packet 3 landed routed allocation reuse plus outcome-honesty and rollback proof on the live tree.
- Packet 4 aligns docs/config truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Typed Fork Command Contract And Policy Surface

Session goal:

1. add a typed host `fork_command` payload on `continue_world_worker`,
2. gate it behind a dedicated deny-by-default policy key,
3. keep optional `progress_ack` and broader autonomy deferred.

### Tasks

- [x] Task 1.1: Add the typed host `fork_command` payload contract
  - Acceptance: `continue_world_worker` accepts the new typed host `fork_command` payload only with the required exact child-work intent fields plus optional bounded metadata; generic prompt-based continue remains valid.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 1.2: Add the dedicated deny-by-default host fork-command policy gate
  - Acceptance: `agents.world_dispatch.fork.commands_allowed` parses, defaults to deny, and yields explanation-ready denials for disallowed typed host fork commands.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. typed host `fork_command` is a valid `continue_world_worker` payload,
2. the new host fork-command gate is deny-by-default,
3. explanation-ready denials exist for disallowed typed host fork commands,
4. optional `progress_ack` and broader autonomy still remain deferred.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Deterministic Rendering And Exact-Source Bootstrap Preparation

Session goal:

1. render typed host fork commands onto the existing member-turn seam deterministically,
2. preserve exact retained source-worker truth before reuse,
3. fail closed before any child-allocation side effect when source truth is wrong.

### Tasks

- [x] Task 2.1: Render typed host `fork_command` deterministically over the retained member-turn seam
  - Acceptance: accepted typed host fork commands render through the existing transport prompt seam in a deterministic implementation-owned format rather than caller-authored ad hoc prompt assembly.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - targeted tests adjacent to the touched implementation

- [x] Task 2.2: Prepare exact-source fork bootstrap reuse without child side effects
  - Acceptance: invalidated, terminal, cross-session, or world-binding-drifted sources fail closed before any child allocation; exact retained source-worker targeting remains mandatory.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - targeted tests adjacent to the touched implementation files

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. accepted typed host fork commands render deterministically,
2. exact retained source-worker targeting remains mandatory,
3. invalid source or boundary drift fails closed before any child side effect.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Routed Allocation Reuse And Outcome Honesty

Session goal:

1. route accepted typed host fork commands through the already-landed fork bootstrap path,
2. keep source-to-child lineage explicit,
3. keep summaries honest and rollback behaviour fail closed.

### Tasks

- [x] Task 3.1: Reuse the landed fork bootstrap path for accepted typed host fork commands
  - Acceptance: allowed typed host fork commands allocate one retained child through the existing exact-source fork path and preserve explicit source-to-child lineage.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [x] Task 3.2: Preserve honest summaries and fail-closed rollback behaviour
  - Acceptance: successful summaries distinguish typed command delivery from child allocation truth; failure paths are explanation-ready and leave no partial child-registration side effects.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. allowed typed host fork commands allocate one retained child through the existing fork path,
2. source-to-child lineage remains explicit,
3. summaries remain honest about delivery versus allocation,
4. rollback and denial paths fail closed without widening into worker auto-fork, optional `progress_ack`, or Family-2 work.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `45` scope,
2. keep worker auto-fork, optional `progress_ack`, active-ephemeral identity, and Family-2 work explicit,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align planning and config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `45` as typed host `fork_command` bootstrap over the existing `continue_world_worker` seam plus reuse of the landed fork bootstrap path, not generalized control transport, worker auto-fork, or Family-2 router execution.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md`](./SPEC-45-internal-retained-host-fork-command-bootstrap.md)
    - [`llm-last-mile/PLAN-45.md`](./PLAN-45.md)
    - [`llm-last-mile/TASKS-45.md`](./TASKS-45.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker policy coverage, and full workspace tests are green against the already-landed Slice `45` file set; this task validates the bounded implementation and does not serve as an open-ended fix bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - none
  - Stop condition: if any validation command fails because Slice `45` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `45` surface is safely bounded,
2. config/docs truth is honest,
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
4. the slice has not widened into worker auto-fork, optional `progress_ack`, transport-schema redesign, active-ephemeral identity, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest typed host fork-command handling requires a broader host/worker transport envelope first,
2. reuse of the landed fork bootstrap path proves wrong and a distinct allocator would be required,
3. accepting typed host fork commands requires a different policy boundary than the dedicated `fork.commands_allowed` gate,
4. verification proves the planned order is wrong.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet's verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether any condition to reopen spec, plan, or tasks was discovered,
5. the GitNexus impact-analysis results for each production symbol edited in that packet, including any `HIGH` or `CRITICAL` warnings reviewed before editing,
6. any remaining risks, deferred follow-ups, or assumptions the next packet must keep.

If a packet is not fully green, the final message must say explicitly that the next packet should not begin yet.
