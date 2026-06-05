# TASKS-46: Internal Retained Host Progress Ack Bootstrap

Source spec: [SPEC-46-internal-retained-host-progress-ack-bootstrap.md](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md)  
Source plan: [PLAN-46.md](./PLAN-46.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: drafted on `2026-06-05`

## Phase Gate

These tasks assume the `SPECIFY` and `PLAN` artifacts for Slice `46` have been reviewed and accepted as the bounded source of truth before implementation begins.

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the typed host `progress_ack` contract and the deny-by-default policy surface.
- Packet 2 lands deterministic rendering and delivery-only summary semantics.
- Packet 3 proves no durable side effects and keeps existing progress/control/fork behavior bounded.
- Packet 4 aligns docs/config truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Typed Progress Ack Contract And Policy Surface

Session goal:

1. add a typed host `progress_ack` payload on `continue_world_worker`,
2. gate it behind a dedicated deny-by-default policy key,
3. keep the slice delivery-only and non-durable.

### Tasks

- [ ] Task 1.1: Add the typed host `progress_ack` payload contract
  - Acceptance: `continue_world_worker` accepts the new typed host `progress_ack` payload with exact retained-worker targeting plus optional non-empty `thread_id`; generic prompt-based continue remains valid.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.2: Add the dedicated deny-by-default host progress-ack policy gate
  - Acceptance: `agents.world_dispatch.control.progress_acks_allowed` parses, defaults to deny, and yields explanation-ready denials for disallowed typed host `progress_ack`.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. typed host `progress_ack` is a valid `continue_world_worker` payload,
2. the new host progress-ack gate is deny-by-default,
3. explanation-ready denials exist for disallowed typed host `progress_ack`,
4. the slice still remains delivery-only and non-durable.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Deterministic Rendering And Delivery-Only Summary Semantics

Session goal:

1. render typed host progress acknowledgements onto the existing member-turn seam deterministically,
2. preserve exact retained-worker targeting for live delivery,
3. keep summaries honest that acknowledgement only confirms seen progress.

### Tasks

- [ ] Task 2.1: Render typed host `progress_ack` deterministically over the retained member-turn seam
  - Acceptance: accepted typed host progress acknowledgements render through the existing transport prompt seam in a deterministic implementation-owned format rather than caller-authored ad hoc prompt assembly.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - targeted tests adjacent to the touched implementation

- [ ] Task 2.2: Keep live delivery summaries strictly acknowledgement-only
  - Acceptance: successful typed host `progress_ack` delivery returns a summary that acknowledges seen progress only and does not imply completion, pause, control acknowledgement, or fork handling.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. accepted typed host `progress_ack` messages render deterministically,
2. exact retained-worker targeting remains mandatory,
3. successful summaries stay explicit that acknowledgement is delivery-only.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Regression Coverage And No-Durable-Side-Effect Proof

Session goal:

1. prove typed host `progress_ack` creates no durable obligation side effects,
2. prove it does not imply completion, pause, or fork handling,
3. keep `progress_update`, approval, control, and fork behavior green and bounded.

### Tasks

- [ ] Task 3.1: Prove typed host `progress_ack` stays non-durable
  - Acceptance: successful typed host `progress_ack` delivery does not create, close, or mutate durable obligations and does not change attention posture by itself.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [ ] Task 3.2: Preserve bounded semantics for existing progress/control/fork flows
  - Acceptance: `progress_update` remains non-attention-driving by default; typed host `progress_ack` does not widen into control directives, control acknowledgements, fork commands, or active-ephemeral behavior.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. allowed typed host `progress_ack` delivery leaves no durable obligation side effects,
2. `progress_update` remains non-attention-driving by default,
3. summaries remain honest about acknowledgement-only delivery,
4. active-ephemeral identity and Family-2 work remain out of scope.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `46` scope,
2. keep active-ephemeral identity and Family-2 work explicit,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Align planning and config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `46` as typed host `progress_ack` bootstrap over the existing `continue_world_worker` seam, not generalized control transport, durable progress review, or Family-2 router execution.
  - Verify:
    - manual diff review
  - Files:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-46-internal-retained-host-progress-ack-bootstrap.md`](./SPEC-46-internal-retained-host-progress-ack-bootstrap.md)
    - [`llm-last-mile/PLAN-46.md`](./PLAN-46.md)
    - [`llm-last-mile/TASKS-46.md`](./TASKS-46.md)

- [ ] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker policy coverage, and full workspace tests are green against the already-landed Slice `46` file set; this task validates the bounded implementation and does not serve as an open-ended fix bucket.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - none
  - Stop condition: if any validation command fails because Slice `46` needs more code or doc changes, stop and add an explicit follow-up task against the concrete failing files instead of treating this validation step as implicit cleanup.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `46` surface is safely bounded,
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
4. the slice has not widened into durable progress review, active-ephemeral identity, transport-schema redesign, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest typed host progress acknowledgement requires a broader typed host-message or transport envelope first,
2. the narrow delivery-only semantics prove too weak and a durable causation model would be required,
3. accepting typed host `progress_ack` requires a different policy boundary than the dedicated `control.progress_acks_allowed` gate,
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
