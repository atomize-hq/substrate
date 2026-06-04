# TASKS-42: Internal Retained Host Clarification Response Bootstrap

Source spec: [SPEC-42-internal-retained-host-clarification-response-bootstrap.md](./SPEC-42-internal-retained-host-clarification-response-bootstrap.md)  
Source plan: [PLAN-42.md](./PLAN-42.md)  
Source validation note: [NOTE-37-family-1-ordering-after-cancel-closeout.md](./NOTE-37-family-1-ordering-after-cancel-closeout.md)  
Phase: `TASKS`  
Execution model: four separate `/incremental-implementation` sessions  
Status: Packet 4 complete on `2026-06-04`; docs truth is aligned and the final validation wall is green
Landed posture note: typed host `clarification_response` now has deny-by-default policy gating, exact unresolved `FollowUpRequired` binding, deterministic prompt rendering on `continue_world_worker`, and post-delivery closeout, while broader host response/control classes, active-ephemeral exact task-identity widening, and Family-2 router execution remain deferred.
Validation note: final validation did not expose any in-scope stabilization follow-up, and no broader host response/control work, active-ephemeral identity widening, transport redesign, or Family-2 execution work was reopened.

## Execution Packets

This slice should be implemented as four sequential `/incremental-implementation` sessions.

- Packet 1 freezes the typed clarification-response contract and minimal policy surface.
- Packet 2 lands follow-up-obligation consumer semantics.
- Packet 3 lands live delivery ordering and deterministic rendering over the existing member-turn seam.
- Packet 4 aligns docs truth and runs the final validation wall.

Do not start a later packet until the prior packet checkpoint is green.

## Packet 1: Typed Clarification Response Contract And Policy Surface

Session goal:

1. accept typed `clarification_response` host payloads on `continue_world_worker`,
2. add the minimum deny-by-default policy key needed for those payloads,
3. keep broader host-response/control classes deferred.

### Tasks

- [x] Task 1.1: Widen the `continue_world_worker` payload contract for typed clarification responses
  - Acceptance: internal `continue_world_worker` requests can carry a typed `clarification_response` payload bound to exact follow-up causation plus non-empty clarification text; generic prompt-based continue remains valid; `progress_ack`, `control_directive`, `control_ack`, and `fork_command` remain deferred.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 1.2: Add minimal deny-by-default policy parsing and denial coverage for typed host clarification responses
  - Acceptance: policy/config truth can explicitly allow or deny typed `clarification_response` delivery; denied responses fail with stable explanation-ready errors; existing `agents.world_dispatch` action/mode/backend/boundary checks remain intact.
  - Verify:
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p substrate-broker -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/policy_model.rs`](../crates/shell/src/execution/policy_model.rs)
    - [`crates/broker/src/policy.rs`](../crates/broker/src/policy.rs)
    - [`crates/broker/src/effective_policy.rs`](../crates/broker/src/effective_policy.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the typed `clarification_response` host payload is frozen,
2. the new policy key is deny-by-default,
3. broader host-response/control classes still remain deferred,
4. policy denials are explanation-ready and do not weaken the existing `agents.world_dispatch` steering floor.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Follow-Up Obligation Consumer Semantics

Session goal:

1. bind typed clarification responses to exact unresolved `FollowUpRequired` obligations,
2. keep resolved-at semantics explicit and durable,
3. fail closed on ambiguous or stale obligation targeting.

### Tasks

- [x] Task 2.1: Freeze exact follow-up-obligation lookup and closeout semantics
  - Acceptance: typed clarification responses must bind to exact unresolved `FollowUpRequired` obligations; cross-session, wrong-kind, already-resolved, or missing obligations fail closed; no obligation is mutated before delivery succeeds.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs) if validation helpers need widening

- [x] Task 2.2: Freeze clarification-response resolution mapping
  - Acceptance: successful clarification responses resolve the matching follow-up obligation; resolved timestamps and compatibility projections stay deterministic and reviewable; delivery failure leaves the obligation pending.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/agent_runtime/obligation_ledger.rs`](../crates/shell/src/execution/agent_runtime/obligation_ledger.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. every typed clarification response has one authoritative follow-up-obligation consumer path,
2. resolve semantics stay explicit and durable,
3. unresolved obligations remain unchanged until live delivery succeeds.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Continue Routing, Deterministic Rendering, And Delivery Ordering

Session goal:

1. deliver typed clarification responses through the live `continue_world_worker` path,
2. reuse the existing member-turn prompt seam honestly with deterministic rendering,
3. prove that obligation closeout happens only after successful delivery.

### Tasks

- [x] Task 3.1: Render typed clarification responses deterministically onto the existing retained member-turn seam
  - Acceptance: typed clarification responses generate canonical prompt text from the typed payload rather than caller-authored ad hoc prompt assembly; live `continue_world_worker` submits that rendered prompt to the exact retained worker.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](../crates/shell/src/execution/agent_runtime/dispatch_contract.rs)

- [x] Task 3.2: Prove delivery ordering and follow-up-obligation closeout semantics
  - Acceptance: successful typed clarification-response delivery resolves the matching obligation exactly once; delivery failure leaves the obligation unresolved; public status/control regressions stay green.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Expected files touched:
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - targeted shell tests adjacent to the touched implementation files

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. live continue handling can deliver the typed clarification-response slice,
2. delivery failures leave obligations unresolved,
3. successful delivery performs deterministic one-time follow-up-obligation closeout,
4. the slice still does not widen into generic control directives or transport-schema redesign.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs Alignment And Final Validation

Session goal:

1. align repo-local docs with the frozen Slice `42` scope,
2. keep broader host-response/control widening, active-ephemeral identity, and Family-2 work explicit,
3. run the final validation wall.

### Tasks

- [x] Task 4.1: Align planning/config truth without widening the slice
  - Acceptance: repo-local docs describe Slice `42` as typed host clarification-response bootstrap over the existing `continue_world_worker` seam, not transport redesign, generic control-directive delivery, or Family-2 router execution.
  - Verify:
    - manual diff review
  - Expected files touched:
    - [`docs/CONFIGURATION.md`](../docs/CONFIGURATION.md)
    - [`llm-last-mile/SPEC-42-internal-retained-host-clarification-response-bootstrap.md`](./SPEC-42-internal-retained-host-clarification-response-bootstrap.md)
    - [`llm-last-mile/PLAN-42.md`](./PLAN-42.md)
    - [`llm-last-mile/TASKS-42.md`](./TASKS-42.md)

- [x] Task 4.2: Run the final validation wall
  - Acceptance: formatting, clippy, targeted shell suites, broker tests, and full workspace tests are green; no unintended widening into generic control directives, transport-schema redesign, active-ephemeral identity, or Family-2 execution appears.
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
    - final validation may require bounded follow-up fixes inside already-touched Slice `42` runtime surfaces, but must not reopen broader host-response/control widening, active-ephemeral identity, or Family-2 router execution

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. the Slice `42` surface is safely bounded,
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
4. the slice has not widened into generic control directives, transport-schema redesign, active-ephemeral identity, or Family-2 execution work.

Reopen spec, plan, or tasks only if one of these is true:

1. honest typed clarification-response delivery requires transport-api widening first,
2. follow-up-obligation closeout cannot stay exact and fail-closed on the current local ledger model,
3. deterministic prompt rendering proves unable to carry the typed clarification-response contract honestly,
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
