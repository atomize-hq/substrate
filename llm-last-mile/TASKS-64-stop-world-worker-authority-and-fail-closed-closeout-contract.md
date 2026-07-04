# TASKS-64: Stop World Worker Authority And Fail-Closed Closeout Contract

Source spec: [SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md](./SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md)  
Source plan: [PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md](./PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md)  
Phase: `TASKS`  
Execution model: five sequential implementation packets  
Status: draft for review

## Phase Gate

These tasks assume:

1. orchestration-session truth is the durable authority root for stop,
2. the currently attached host participant is only the current sanctioned execution client and may be replaced through sanctioned recovery,
3. the private stop transport remains delivery-only and must not be treated as lifecycle truth,
4. one bounded authority-recovery attempt plus exactly one bounded exact-target re-attempt is required when `SPEC-64` says sanctioned recovery is available,
5. caller-visible stop success requires both authoritative stopped closeout and terminal proof for that same stop episode,
6. caller-visible non-success outcomes keep the existing error/result shape in this slice and must instead preserve stable textual distinctions for `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed`,
7. Linux-first remains rollout posture and smoke-validation scope only; non-Linux remains fail-closed unless a later higher-authority slice broadens support.

Do not begin implementation until the spec and plan are accepted.

If implementation evidence forces a new public CLI contract, a second stop transport, a new typed stop-outcome taxonomy, or broader attach/router redesign, stop and update the spec/plan before continuing.

## Execution Packets

This slice should be implemented as five sequential packets:

1. pin the stop-authority, transport, recovery, and caller-visible-proof seam in tests,
2. rebind stop dispatch to current authoritative session, owner, and retained-worker truth,
3. implement required bounded recovery and exactly one exact-target re-attempt,
4. gate caller-visible success on authoritative closeout plus terminal proof while preserving stable textual failure distinctions,
5. run the final Linux-first validation wall and align slice-local artifacts.

Do not begin a later packet until the prior packet checkpoint is green.

## Packet 1: Pin The Stop Contract Seam In Tests

Session goal:

1. reproduce the current authority-versus-transport drift in automation,
2. pin the required recovery behavior before implementation changes,
3. pin the caller-visible proof rule and the stable textual distinction floor for non-success outcomes.

### Tasks

- [x] Task: Add focused regression coverage for stale attached-host owner rejection
  - Acceptance: automated coverage proves that a stop attempt bound to a stale or disproven attached-host owner path is rejected before it can count as valid delivery, and the failure remains explanation-ready rather than collapsing into generic transport error handling.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) only if file-local tests are the narrowest honest place to pin owner-rejection behavior

- [x] Task: Add focused regression coverage proving dead or disappearing private stop delivery does not imply stop success
  - Acceptance: automated coverage proves that a dead, dropped, or disappearing private stop delivery path does not by itself count as durable stop success, and the worker is not treated as `stopped` merely because delivery failed mid-episode.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
  - Files:
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs) only if an adjacent unit harness is the narrowest honest place to pin runtime-side non-success proof behavior

- [x] Task: Add focused regression coverage for missing-terminal-proof fail-closed caller behavior
  - Acceptance: automated coverage proves that when a stop episode cannot return terminal proof for that same episode, the caller-visible result fails closed even if later read-side state observation could differ.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)

- [x] Task: Add harness coverage for the required bounded recovery-and-one-retry path
  - Acceptance: automated coverage proves that when `SPEC-64` recovery conditions hold, exactly one bounded recovery attempt and one exact-target re-attempt are expected, and the harness is narrow enough that later implementation can satisfy it without reopening transport or attach scope.
  - Verify:
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs) only if file-local tests are needed
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs) only if file-local tests are needed

- [x] Task: Pin the stable textual distinction floor for caller-visible non-success outcomes
  - Acceptance: automated coverage proves that caller-visible non-success outputs remain stably distinguishable using the existing shape for `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed`, without requiring a new typed outcome family.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 1 Checkpoint

Packet 1 is complete only when:

1. the negative stop seam is reproducible in automation,
2. the required recovery path has a reviewable harness,
3. caller-visible missing-terminal-proof failure is pinned,
4. stale attached-host owner rejection is pinned independently from other failure modes,
5. dead/disappearing private stop delivery is pinned independently from caller-visible proof gaps,
6. the stable textual distinction floor for non-success outcomes is pinned before behavior changes begin.

Do not start Packet 2 until Packet 1 is reviewed and green.

## Packet 2: Rebind Stop Dispatch To Authoritative Session And Worker Truth

Session goal:

1. bind stop dispatch to durable orchestration-session authority,
2. reject stale attached-host owner paths before delivery,
3. preserve exact retained-worker identity and lineage checks.

### Tasks

- [x] Task: Resolve current authoritative attached-host ownership at stop-dispatch time
  - Acceptance: stop dispatch resolves the current sanctioned owner path from persisted/session truth instead of trusting stale attached-host snapshots or transport discovery alone, and stale/disproven owner paths fail closed with explanation-ready output.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/orchestration_session.rs`](../crates/shell/src/execution/agent_runtime/orchestration_session.rs)
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)

- [x] Task: Keep retained-worker exact-target resolution distinct from owner rebinding
  - Acceptance: the exact retained-worker tuple remains mandatory across stop dispatch and recovery, and a valid owner rebinding never silently broadens, guesses, or mutates retained-worker identity, world binding, or lineage.
  - Verify:
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/state_store.rs`](../crates/shell/src/execution/agent_runtime/state_store.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

### Packet 2 Checkpoint

Packet 2 is complete only when:

1. stale attached-host snapshots are rejected before delivery,
2. stop dispatch resolves against durable session truth,
3. exact retained-worker targeting remains fail-closed and distinct from owner rebinding,
4. disproven worker binding or lineage does not convert into successful stop.

Do not start Packet 3 until Packet 2 verification is green.

## Packet 3: Implement Required Recovery And One Exact-Target Re-Attempt

Session goal:

1. make recovery an explicit bounded stop-episode rule,
2. ensure recovery uses existing sanctioned authority truth without widening scope,
3. cap the stop episode to one recovery attempt and one exact-target re-attempt.

### Tasks

- [x] Task: Implement one bounded sanctioned recovery attempt for qualifying stop-delivery failures
  - Acceptance: when the stop episode has no terminal proof yet, exact-target authority still holds, and the failure is stale/missing/refused current-owner delivery rather than disproven worker identity, the implementation performs one bounded sanctioned recovery attempt; otherwise it fails closed immediately without hidden fallback behavior.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

- [x] Task: Enforce exactly one bounded exact-target stop re-attempt after successful recovery
  - Acceptance: after successful recovery, the stop episode performs exactly one exact-target re-attempt through the recovered owner path; no second retry, fuzzy retargeting, or transport-family widening occurs, and failed recovery still returns fail-closed non-success output.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell dispatch_contract -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)

### Packet 3 Checkpoint

Packet 3 is complete only when:

1. one bounded recovery attempt occurs when required,
2. at most one exact-target re-attempt occurs after recovery,
3. failed or unavailable recovery paths fail closed without hidden fallback behavior,
4. the slice has not widened into general attach or router redesign.

Do not start Packet 4 until Packet 3 verification is green.

## Packet 4: Gate Caller-Visible Success And Preserve Stable Non-Success Distinctions

Session goal:

1. require both authoritative stopped closeout and terminal proof for caller-visible success,
2. keep proof gaps fail-closed,
3. preserve the existing caller-visible shape while stabilizing the required non-success distinctions.

### Tasks

- [x] Task: Require authoritative stopped closeout plus terminal proof for caller-visible success
  - Acceptance: a stop episode reports caller-visible success only when authoritative retained-worker stopped closeout is proven and terminal proof returns for that same episode; a proof gap still fails closed from the caller’s perspective, and later read-side observation does not retroactively convert the earlier attempt into success.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/world-service/src/member_runtime.rs`](../crates/world-service/src/member_runtime.rs) only if narrow runtime proof wiring is needed

- [x] Task: Standardize caller-visible non-success wording using the existing shape
  - Acceptance: this slice does not introduce a new typed stop-outcome taxonomy; instead, caller-visible non-success outputs keep the existing shape and stably distinguish `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed` through reviewable wording that tests can assert.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - Files:
    - [`crates/shell/src/execution/agent_runtime/control.rs`](../crates/shell/src/execution/agent_runtime/control.rs)
    - [`crates/shell/src/execution/orchestrator_world_dispatch.rs`](../crates/shell/src/execution/orchestrator_world_dispatch.rs)
    - [`crates/shell/tests/repl_world_first_routing_v1.rs`](../crates/shell/tests/repl_world_first_routing_v1.rs)
    - [`crates/shell/tests/agent_public_control_surface_v1.rs`](../crates/shell/tests/agent_public_control_surface_v1.rs)

### Packet 4 Checkpoint

Packet 4 is complete only when:

1. caller-visible success requires both authoritative closeout and terminal proof,
2. missing terminal proof yields fail-closed caller-visible outcome,
3. later observed `stopped` state does not retroactively convert the earlier attempt to success,
4. the required non-success failure classes are stably distinguishable using the existing shape,
5. no new typed stop-outcome taxonomy has been introduced in this slice.

Do not start Packet 5 until Packet 4 verification is green.

## Packet 5: Final Validation Wall And Linux-First Proof

Session goal:

1. prove the corrected contract end to end on the current supported rollout path,
2. keep Linux-first as rollout posture only,
3. confirm adjacent slices were not silently reopened.

### Tasks

- [x] Task: Run the targeted validation wall for the stop contract correction
  - Acceptance: formatting, lint, targeted shell/world-service suites, and final workspace validation are green after the stop contract repair, with no evidence that the slice widened into public CLI redesign, general attach/router redesign, or a typed outcome-taxonomy change.
  - Verify:
    - `cargo fmt --all -- --check`
    - `cargo clippy --workspace --all-targets -- -D warnings`
    - `cargo test -p shell dispatch_contract -- --nocapture`
    - `cargo test -p shell state_store -- --nocapture`
    - `cargo test -p shell policy_model -- --nocapture`
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
    - `cargo test --workspace -- --nocapture`
  - Files:
    - no planned source edits; validation only

- [x] Task: Capture Linux-first positive and negative proof for the corrected stop episode
  - Acceptance: the post-implementation proof wall includes one Linux-first positive path where sanctioned recovery succeeds and one exact-target re-attempt reaches terminal stopped proof, plus one Linux-first negative path where delivery fails and durable closeout cannot be proven, producing a fail-closed caller-visible outcome; non-Linux remains explicitly fail-closed rather than semantically divergent.
  - Verify:
    - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
    - `cargo test -p world-service member_runtime -- --nocapture`
  - Files:
    - no planned source edits; validation and smoke evidence only

- [x] Task: Align slice-local artifacts and capture final proof for review
  - Acceptance: the slice-local planning/closeout artifacts accurately reflect the implemented stop contract and its validation proof: `SPEC-64`, `PLAN-64`, and `TASKS-64` remain consistent with the landed behavior; any slice-local closeout or review note added during implementation names the Linux-first positive and negative proof paths actually exercised; and no artifact text implies public stop redesign, transport-defined success, non-Linux semantic parity, or a new typed stop-outcome taxonomy in this slice.
  - Verify:
    - `rg -n "typed stop-outcome taxonomy|public stop|transport.*success|Linux-first|fail-closed|recovery_failed|missing_transport|refused_transport|stale_authority" llm-last-mile/SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md llm-last-mile/PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md llm-last-mile/TASKS-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`
    - manual review that any newly added slice-local closeout or implementation note cites the exact positive and negative proof commands/results captured by this packet
  - Files:
    - [`llm-last-mile/SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`](./SPEC-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md) only if wording must be updated to match landed truth
    - [`llm-last-mile/PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`](./PLAN-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md) only if wording must be updated to match landed truth
    - [`llm-last-mile/TASKS-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md`](./TASKS-64-stop-world-worker-authority-and-fail-closed-closeout-contract.md) only if wording must be updated to match landed truth
    - one slice-local closeout or implementation note in `llm-last-mile/` only if implementation adds one as the review evidence carrier
  - Evidence:
    - [`llm-last-mile/NOTE-64-stop-world-worker-authority-and-fail-closed-closeout-proof.md`](./NOTE-64-stop-world-worker-authority-and-fail-closed-closeout-proof.md)

### Packet 5 Checkpoint

Packet 5 is complete only when:

1. the corrected stop-authority seam is green end to end,
2. Linux-first positive and negative proof paths are both captured,
3. slice-local artifacts and any closeout note are aligned with the landed stop contract and captured proof,
4. non-Linux remains honest and fail-closed,
5. the slice did not widen into public stop redesign, `cancel_world_work`, general attach/router redesign, a second stop transport, or a new typed stop-outcome taxonomy.

## Cross-Packet Dependency Order

1. Packet 1 blocks Packet 2.
2. Packet 2 blocks Packet 3.
3. Packet 3 blocks Packet 4.
4. Packet 4 blocks Packet 5.

## Inter-Packet Review Rules

After each packet:

1. confirm its checkpoint is satisfied,
2. confirm orchestration-session truth remains the durable stop authority root,
3. confirm private stop delivery is not being treated as lifecycle truth,
4. confirm recovery remains bounded to one attempt plus one exact-target re-attempt,
5. confirm caller-visible non-success outcomes still use the existing shape with stable textual distinctions,
6. confirm the slice remains bounded to the stop-world-worker contract correction.
