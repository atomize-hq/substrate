# TASKS-30.25: Sanctioned Manual Host Attach For Born-Unattached World Start

Source spec: [SPEC-30_25.md](./SPEC-30_25.md)  
Source plan: [PLAN-30_25.md](./PLAN-30_25.md)  
Adjacent landed slice inputs: [TASKS-30.md](./TASKS-30.md), [30-public-world-scoped-agent-start-and-capability-flags.md](./30-public-world-scoped-agent-start-and-capability-flags.md)  
Deferred follow-on: [31-lazy-host-attach-for-host-rooted-world-start.md](./31-lazy-host-attach-for-host-rooted-world-start.md)  
Phase: `TASKS`  
Execution model: parked; do not implement until the slice is explicitly reactivated  
Status: parked on 2026-05-27 after slice-30 was realigned to a host-first default path

## Parking Note

These tasks are not ready for implementation.

They depend on `born_unattached` being the normal public world-start path. The updated slice-30 direction no longer treats that as the thin-slice happy path, so this task packet remains historical planning context only unless a later slice deliberately revives an explicit unattached public start mode.

## Execution Packets

This slice should be implemented as four separate `/incremental-implementation` sessions.

- Packet 1 implements Phase 1 only.
- Packet 2 implements Phase 2 only.
- Packet 3 implements Phase 3 only.
- Packet 4 implements Phase 4 only.

Do not start a later packet until the prior packet's checkpoint is green.

## Packet 1: Manual Attach Contract And Eligibility

Session goal:

1. extend the public `reattach` contract to eligible `born_unattached` sessions,
2. preserve existing parked-host recovery semantics,
3. pin fail-closed rejection behavior for ineligible sessions.

### Tasks

- [ ] Task 1.1: Widen public `reattach` eligibility to include sanctioned manual attach for eligible `born_unattached` sessions
  - Acceptance: `substrate agent reattach --session <orchestration_session_id>` remains session-scoped and non-prompt-taking; it can target an eligible `born_unattached` session without allocating a new orchestration session; existing parked/detached host-session recovery behavior remains unchanged.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

- [ ] Task 1.2: Pin fail-closed reattach rejection taxonomy for ineligible `born_unattached` and detached sessions
  - Acceptance: ineligible targets still fail closed with explicit reasons; `reattach` does not silently broaden into prompt-taking follow-up, world-to-world continuity, or a new-session allocation path; tests assert stable rejection classifiers and messages.
  - Verify: `cargo test -p shell reattach -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. `reattach` is explicitly allowed for eligible `born_unattached` sessions,
2. existing parked-host `reattach` behavior stays green,
3. ineligible sessions still fail closed with explicit reasons.

Do not start Packet 2 until Packet 1 verification is green.

## Packet 2: Explicit Manual Attach Planning From Persisted Truth

Session goal:

1. plan manual attach from persisted `HostAttachContract` truth,
2. choose continuity attach versus fresh attach explicitly,
3. keep both modes non-prompt-taking and auditable.

### Tasks

- [ ] Task 2.1: Build one sanctioned attach planner that chooses continuity attach versus fresh attach from persisted attach truth
  - Acceptance: the runtime selects continuity attach when a valid persisted continuity selector exists and fresh attach when continuity does not exist but the persisted baseline is complete; attach-mode choice is explicit and testable; the planner does not recover semantics from stale participant snapshots when durable attach truth is present.
  - Verify: `cargo test -p shell attach_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/dispatch_contract.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)

- [ ] Task 2.2: Execute fresh attach for `born_unattached` sessions without synthetic prompt submission
  - Acceptance: an eligible `born_unattached` session can successfully attach a real host execution client through `reattach`; the attach path reuses the persisted `HostAttachContract`; no hidden bootstrap prompt, inbox-consumption prompt, or synthetic warm-up turn is submitted.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agents_cmd.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
    - [`crates/shell/src/execution/agent_runtime/control.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. manual attach planning explicitly chooses continuity or fresh attach,
2. `born_unattached` sessions can attach through the sanctioned path,
3. neither mode injects hidden prompt-bearing behavior.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Truthful Post-Attach And Failure Lifecycle State

Session goal:

1. transition successful manual attach to truthful attached state,
2. preserve non-theatrical failure outcomes,
3. keep parked and attention-needed semantics distinct from born-unattached attach flows.

### Tasks

- [ ] Task 3.1: Persist truthful `active_attached` state after successful manual attach
  - Acceptance: successful `reattach` for a `born_unattached` session keeps the same orchestration session id and transitions that session to truthful `active_attached` state with authoritative attached-participant linkage; later `turn` and `stop` continue to operate on that same durable session.
  - Verify: `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs)

- [ ] Task 3.2: Preserve truthful failure and status-projection semantics for manual attach
  - Acceptance: failed manual attach leaves the session non-terminal and does not misreport `parked_resumable`, `awaiting_attention`, or `active_attached` when that state was not actually achieved; status JSON and projection rules remain explicit for `born_unattached`, `active_attached`, `parked_resumable`, and `awaiting_attention`.
  - Verify: `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/tests/agent_successor_contract_ahcsitc0.rs`](/home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. successful manual attach results in truthful `active_attached` state for the same session,
2. failed attach outcomes stay non-theatrical and non-terminal,
3. parked and attention-needed host lifecycle semantics remain intact.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Docs, Deferred-Scope Alignment, And Validation Wall

Session goal:

1. update operator/docs wording to reflect the shipped manual attach bridge,
2. keep automatic attach policy explicitly deferred to slice 31,
3. run the final validation wall.

### Tasks

- [ ] Task 4.1: Update operator and slice docs to describe sanctioned manual attach for `born_unattached`
  - Acceptance: docs stop claiming that `born_unattached` has no sanctioned host-attach surface; `reattach` is documented as the manual-only path for this bridge slice; wording distinguishes the new `30.25` bridge contract from the broader automatic-attach work still deferred to slice 31.
  - Verify: manual diff review plus `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`docs/USAGE.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/docs/USAGE.md)
    - [`llm-last-mile/README.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/README.md)
    - [`llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md`](/home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/31-lazy-host-attach-for-host-rooted-world-start.md)

- [ ] Task 4.2: Run the final validation wall for the full bridge slice
  - Acceptance: targeted shell suites, focused reattach/attach-contract coverage, formatting, clippy, and full workspace tests pass; any Linux manual evidence needed for the bridge slice is captured before closeout.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
    - `cargo test -p shell reattach -- --nocapture`
    - `cargo test -p shell attach_contract -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - No planned source edits; this is the validation gate after the implementation tasks above.

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. docs describe the shipped manual attach bridge truthfully,
2. slice 31 remains limited to automatic-trigger and broader taxonomy questions,
3. the full validation wall passes.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.

## Notes For Implementation

- Packet 1 is the contract-widening packet. Do not leak attach-runtime behavior changes into it beyond what is needed to pin eligibility and rejection semantics.
- Packet 2 is the highest-risk runtime packet. Keep it focused on manual attach planning and execution from persisted truth.
- Packet 3 should stay focused on truthful lifecycle transitions and status projection. If it expands into docs or policy questions, stop and defer that work to Packet 4 or slice 31.
- Packet 4 is the integration packet. This is where stale “no sanctioned host-attach surface” wording should be retired and where deferred automatic-attach scope must remain explicit.
