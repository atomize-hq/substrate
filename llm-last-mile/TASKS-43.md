# TASKS-43: Internal Retained Host Control Directive Bootstrap

Source spec: [SPEC-43-internal-retained-host-control-directive-bootstrap.md](./SPEC-43-internal-retained-host-control-directive-bootstrap.md)  
Source plan: [PLAN-43.md](./PLAN-43.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packet `4` complete on `2026-06-04`

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the typed control-directive contract and minimal policy surface.
- Packet 2 lands the bounded directive taxonomy, fail-closed metadata-label `directive_text` surface, and deterministic rendering on the shared live `continue_world_worker` seam.
- Packet 3 lands delivery-outcome honesty and no-ack-implication regression proof for that already-landed live path.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Typed Control Directive Contract And Policy Surface

Session goal:

1. accept typed `control_directive` host payloads on `continue_world_worker`,
2. add the minimum deny-by-default policy key needed for those payloads,
3. keep broader host control/ack/fork classes deferred.

### Tasks

- [ ] Task 1.1: Widen the `continue_world_worker` payload contract for typed control directives
  - Acceptance: internal `continue_world_worker` requests can carry a typed `control_directive` payload with a bounded directive kind plus optional fail-closed `directive_text` metadata label detail; generic prompt-based continue remains valid; `control_ack`, `fork_command`, and `progress_ack` remain deferred.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 1.2: Add minimal deny-by-default policy parsing and denial coverage for typed host control directives
  - Acceptance: policy/config truth can explicitly allow or deny typed `control_directive` delivery; denied directives fail with stable explanation-ready errors; existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the typed `control_directive` host payload is frozen,
2. the new policy key is deny-by-default,
3. broader host control/ack/fork classes still remain deferred,
4. policy denials are explanation-ready and do not weaken the existing `agents.world_dispatch` steering floor.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Directive Taxonomy And Deterministic Rendering

Session goal:

1. freeze the bounded first directive-kind set,
2. render typed control directives deterministically onto the existing retained member-turn seam,
3. keep rendering implementation-owned and reviewable.

### Tasks

- [ ] Task 2.1: Freeze the bounded first control-directive taxonomy
  - Acceptance: the initial typed control-directive slice accepts only the explicitly allowed directive kinds; unsupported kinds fail closed; optional `directive_text` accepts only the recognized bounded metadata labels for the selected directive kind; the slice does not invent a broad arbitrary control language.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [ ] Task 2.2: Render typed control directives deterministically onto the existing retained member-turn seam
  - Acceptance: typed control directives generate canonical prompt text from the typed payload rather than caller-authored ad hoc prompt assembly; the shared live `continue_world_worker` path uses that deterministic renderer when submitting to the exact retained worker.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. the first directive-kind surface is explicitly bounded,
2. rendered prompt content is deterministic from the typed payload,
3. the shared live `continue_world_worker` path consumes that deterministic renderer for exact retained-worker submission,
4. normal prompt-based continue remains unchanged.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Delivery-Outcome Honesty And No-Ack-Implication Regression Proof

Session goal:

1. prove the already-landed typed control-directive delivery path through live `continue_world_worker`,
2. keep delivery outcomes honest about the absence of typed `control_ack`,
3. prove existing approval and clarification behavior does not regress.

### Tasks

- [ ] Task 3.1: Prove the already-landed live `continue_world_worker` delivery path for typed control directives
  - Acceptance: the already-landed live retained-worker path submits the typed control-directive prompt to the exact retained worker and reports truthful delivery outcomes.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

- [ ] Task 3.2: Prove no-ack implication and non-regression behavior on the shared host-response path
  - Acceptance: successful typed control-directive delivery does not imply a landed `control_ack`; delivery failures return explanation-ready errors; approval/clarification host-response behavior and public status/control projection remain green.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. the already-landed live continue handling remains able to deliver the typed control-directive slice,
2. outcome summaries stay honest about delivery versus acknowledgement,
3. existing approval/clarification and public projection behavior remains green.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `43` scope,
2. keep `control_ack`, `fork_command`, `progress_ack`, active-ephemeral identity, and Family-2 work explicit,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `43` as typed host control-directive bootstrap over the existing `continue_world_worker` seam, not generalized control transport, public control-surface widening, or Family-2 router execution.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-43-internal-retained-host-control-directive-bootstrap.md`](./SPEC-43-internal-retained-host-control-directive-bootstrap.md)
    - [`llm-last-mile/PLAN-43.md`](./PLAN-43.md)
    - [`llm-last-mile/TASKS-43.md`](./TASKS-43.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker tests, and full workspace tests are green; no unintended widening into `control_ack`, `fork_command`, transport-schema redesign, active-ephemeral identity, or Family-2 execution appears.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Expected files touched:
    - final validation may require bounded follow-up fixes inside already-touched Slice `43` runtime surfaces, but must not reopen broader host control/ack/fork widening, active-ephemeral identity, or Family-2 router execution

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `43` surface is safely bounded,
2. config/docs truth is honest,
3. the validation wall is green.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Inter-Packet Review Rules

After completing a packet, treat the next step as a packet checkpoint review, not a fresh `spec-driven-development` restart.

Proceed directly to the next packet only when:

1. the current packet’s verification steps are green,
2. the current packet checkpoint is satisfied,
3. the source spec and plan still match the intended landed contract,
4. the slice has not widened into `control_ack`, `fork_command`, transport-schema redesign, active-ephemeral identity, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest typed control-directive delivery requires transport-api widening first,
2. the bounded directive taxonomy proves too small or too large to stay reviewable as one slice,
3. deterministic prompt rendering proves unable to carry the typed control-directive contract honestly,
4. verification proves the planned order is wrong.

If none of those conditions are met, continue packet-to-packet without re-specifying.

## Packet Session Final Message Requirements

Every packet implementation session should end with a final completion message that surfaces all of the following:

1. whether the packet’s verification commands passed or which ones did not,
2. whether the packet checkpoint is green,
3. whether the next packet is unblocked,
4. whether any condition to reopen spec, plan, or tasks was discovered,
5. the GitNexus impact-analysis results for each production symbol edited in that packet, including any `HIGH` or `CRITICAL` warnings reviewed before editing,
6. any remaining risks, deferred follow-ups, or assumptions the next packet must keep.

If a packet is not fully green, the final message must say explicitly that the next packet should not begin yet.
