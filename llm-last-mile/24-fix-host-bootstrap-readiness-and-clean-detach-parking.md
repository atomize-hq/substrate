# SOW: Fix Host Bootstrap Readiness And Clean-Detach Parking

Status: corrective follow-on draft. This SOW narrows and sequences the remaining work after [23-host-orchestrator-durable-session-and-parked-resumable-ownership.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/23-host-orchestrator-durable-session-and-parked-resumable-ownership.md) so the original manual smoke failure from [PLAN-22.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-22.md) is actually fixed in the runtime, not only modeled in persisted state and tests built from synthetic parked sessions.

This slice does not back out the new posture, inbox, or `reattach` semantics. It keeps those features and finishes the missing runtime seams:

- hidden owner-helper startup readiness must stop requiring a continuously attached live owner when a valid resumable host session already exists,
- clean bootstrap/control-stream exit must park a valid host session instead of invalidating it,
- and the public prompt bridge must keep its terminal-delivery guarantee without pretending the deeper startup contract is already durable.

## Objective

Fix the original host smoke-failure path while keeping the new durable-session features.

This slice is done only when all of the following are true:

1. A host `substrate agent start --backend <backend_id> --prompt ...` can succeed even when the prompt-driven backend behaves like `codex exec`: emit a valid session handle, become resumable, then let the bootstrap control stream end cleanly.
2. The resulting orchestration session parks deterministically:
   - `parked_resumable` when the session remains non-terminal, `attached_participant_id == null`, `pending_inbox_count == 0`, and at least one authoritative host participant remains `resume_eligible == true`;
   - `awaiting_attention` when the session remains non-terminal, `attached_participant_id == null`, and `pending_inbox_count > 0`;
   - and never collapses to `invalidated` / `terminal` solely because the bootstrap control stream ended cleanly after continuity was established.
3. `substrate agent turn --session ... --backend ... --prompt ...` and `substrate agent reattach --session ...` continue to work against that legitimately parked session.
4. The new session posture, participant attachment fields, and session-local durable inbox from slice `23` remain intact.
5. The public prompt bridge still guarantees explicit `Completed` or `Failed` after `Accepted`, but the runtime no longer depends on that bridge-level fallback to mask a startup-contract defect.

## Why This Follow-On Exists

Slice `23` landed important scaffolding:

- explicit session posture,
- additive host attachment metadata,
- a canonical session-local durable inbox,
- resumed parked-session tests,
- and a bridge-level synthetic terminal `Failed` when a private prompt owner drops after `Accepted`.

Those are all useful and should remain.

But the original manual failure path still survives because the runtime seams that caused it were not actually changed:

- `run_start(...)` still waits for hidden owner-helper readiness before public prompt submission in [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs:301).
- `wait_for_hidden_owner_helper_readiness(...)` still delegates to `hidden_owner_helper_launch_ready(...)` in [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs:466).
- `hidden_owner_helper_launch_ready(...)` still requires `participant.is_authoritative_live()` and `owner_process_is_alive(participant)` in [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs:924), which is the old attached-live bootstrap model.
- when the attached control stream ends after live ownership, the event-task teardown path still invalidates the orchestrator in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:2608).
- the new parking path currently lives in explicit shutdown handling rather than the hidden-owner-helper bootstrap teardown seam in [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs:4807).

That means the implementation can now model and resume synthetic parked sessions, but it still cannot reliably *create* them through the original host bootstrap path that failed in manual testing.

## Relationship To Slice 23

This SOW is a completion and correction slice for `23`, not a restart.

Keep from `23`:

- explicit `active_attached | parked_resumable | awaiting_attention | terminal` posture,
- additive host participant attachment and `resume_eligible` fields,
- session-local durable inbox under `sessions/<orchestration_session_id>/inbox/`,
- `reattach` as attached-owner recovery only,
- resolved-item retention and later compaction eligibility,
- greenfield session-root state authority,
- and additive inbox correlation compatibility.

Fix after `23`:

- readiness still tied to attached-live ownership,
- invalidation-on-clean-bootstrap-exit still active,
- manual host `start` clean-exit path still failing,
- and test coverage still centered on synthetic parked state rather than the original failing bootstrap seam.

This slice must not remove the new features just because the first implementation missed the root seam.

## Exact Remaining Defect Statement

The original bug is still:

1. Substrate assumes the hidden owner-helper bootstrap run must remain a durable attached-control session until explicit cancel.
2. A `codex exec`-style backend can emit a valid session handle and become resumable, then end the bootstrap control stream cleanly.
3. The runtime still interprets that control-stream end as failure or invalidation instead of valid parked continuity.

The secondary weakness is still:

1. The prompt bridge emits `Accepted` before real prompt completion is durably guaranteed.
2. A later owner drop is rendered as an explicit failure now, which is better than EOF.
3. But the lifecycle contract still needs to stop manufacturing that failure in the first place when the session is actually valid and parked.

## Scope

In scope:

- change hidden owner-helper startup readiness so a valid parked-resumable host session counts as ready enough for `agent start` to continue,
- replace invalidation with parking on the exact clean bootstrap/control-stream-end seam when host session continuity is valid,
- preserve invalidation only for truly broken or unrecoverable startup cases,
- align startup completion handling with the new parked-session contract,
- add a true regression test for the manual smoke path,
- and tighten docs/tests around the distinction between:
  - successful parked bootstrap,
  - real startup failure,
  - and post-`Accepted` terminal failure.

Out of scope:

- new public verbs or selector changes,
- world-root broadening or detached-world recovery broadening,
- inbox product UX such as listing or browsing inbox items,
- changing the durable inbox outer envelope again,
- changing compaction policy,
- or backing out the new posture/participant/inbox fields from `23`.

## Required Runtime Contract

### 1. Hidden owner-helper readiness must accept valid parked host continuity

For host orchestrator startup, readiness must no longer mean only:

- session is `Active`,
- exact authoritative participant is selected,
- participant is authoritative-live,
- owner PID is alive,
- internal session id exists when required.

It must also allow the new valid detached host posture when all of the following are true:

- the orchestration session remains non-terminal,
- `active_session_handle_id` still points at the authoritative host participant selected for this orchestration session,
- `attached_participant_id == null`,
- the persisted session posture is already normalized to either:
  - `parked_resumable` when `pending_inbox_count == 0`, or
  - `awaiting_attention` when `pending_inbox_count > 0`,
- the authoritative host participant remains `resume_eligible == true`,
- the authoritative host participant has `attached_client_present == false`,
- a valid `uaa_session_id` exists when required by the backend resume path,
- and no participant or session lifecycle field marks the session invalidated, stopped, failed, or otherwise terminal.

This is the runtime seam that lets a `codex exec`-style bootstrap succeed without requiring a permanently attached owner process.

The minimum persisted resumability proof for this slice is therefore:

- session `state` is non-terminal,
- session `posture` is `parked_resumable` or `awaiting_attention`,
- session `attached_participant_id == null`,
- session `active_session_handle_id` still matches the authoritative host participant,
- session `pending_inbox_count` agrees with the normalized posture,
- participant `resume_eligible == true`,
- participant `attached_client_present == false`,
- participant `uaa_session_id` is populated when the backend resume path requires it.

Anything weaker than that is not enough to treat clean bootstrap exit as resumable continuity.

### 2. Clean bootstrap control-stream end must park, not invalidate

When the hidden owner-helper bootstrap path has already established valid host session continuity and the control stream ends cleanly:

- the participant must release attachment diagnostics,
- the session must remain `state == Active`,
- the session posture must normalize deterministically:
  - `parked_resumable` when `pending_inbox_count == 0`,
  - `awaiting_attention` when `pending_inbox_count > 0`,
- and the participant must remain `resume_eligible == true`.

This path must not:

- transition the participant or session to `Invalidated`,
- mark the session `terminal`,
- or emit startup failure solely because the owner process is no longer attached after continuity was established.

### 3. Invalidations remain for truly broken startup cases

The runtime must still fail closed when:

- ownership was never established,
- the session handle or `uaa_session_id` required for resume was never surfaced,
- `active_session_handle_id` does not match the authoritative host participant,
- the authoritative host participant record is missing, terminal, or not host-routable,
- `resume_eligible == false`,
- `attached_participant_id != null` after clean detach normalization should already have happened,
- `pending_inbox_count` and `posture` disagree with the persisted invariants from ADR-0047 and `PLAN.md`,
- the session `state` is `Invalidated`, `Stopped`, `Failed`, or otherwise terminal,
- the selected target is world-only or detached-world follow-up would be required,
- or the control stream ends with concrete evidence that the session is not safely resumable under the persisted contract.

This slice is not “always park on stream end.” It is “park only when the new durable-session contract says the host session is already valid.”

### 4. The public prompt bridge keeps explicit terminal delivery

The post-`Accepted` bridge rule from `23` remains:

- once `Accepted` is emitted, the request must terminate with `Completed` or `Failed`.

But the runtime should now hit the `Failed` path less often for clean host bootstrap behavior because more valid clean-detach cases are classified as parked continuity rather than owner death.

### 5. `turn` and `reattach` semantics stay as already corrected

This slice must preserve:

- `turn` as prompt-taking resume against exact `(orchestration_session_id, backend_id)`,
- `reattach` as attached-owner recovery only,
- and detached-world follow-up as fail closed.

## Concrete Work Breakdown

### 1. Rework hidden owner-helper readiness classification

Update the startup/readiness path across:

- [agents_cmd.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs)
- [control.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/control.rs)
- [state_store.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs)

Required change:

- `hidden_owner_helper_launch_ready(...)` must stop treating attached-live ownership as the only successful bootstrap readiness state.
- It must accept a valid parked/resume-eligible host session when the authoritative linkage and required internal session id are already in place.

### 2. Rewrite the bootstrap event/completion teardown seam

Update the hidden owner-helper lifecycle teardown in:

- [async_repl.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs)

Required change:

- the event-task branch that currently invalidates on `shell-owned orchestrator control stream ended before completion observation` must instead route through a “park if continuity already established” decision for host orchestrators.
- the completion-side startup failure path must align with that same decision so event/completion race ordering cannot reintroduce terminal invalidation for resumable host sessions.

### 3. Preserve explicit shutdown parking but do not rely on it as the only parking seam

The explicit shutdown parking path added in `23` stays. This slice extends the same semantics to the hidden-owner-helper bootstrap lifecycle seam instead of leaving the manual smoke path on old invalidation logic.

### 4. Align status/projection logic with the real startup contract

Any status-session completeness or readiness projection that still assumes attached-live owner truth as the only valid active host continuity must be updated to respect the new parked bootstrap outcomes.

Required projection rule:

- posture is recomputed from authoritative session + participant + pending-inbox truth, not command-local heuristics;
- `attached_participant_id != null` normalizes to `active_attached`;
- `attached_participant_id == null` plus `pending_inbox_count > 0` normalizes to `awaiting_attention`;
- `attached_participant_id == null` plus `pending_inbox_count == 0` plus at least one authoritative host participant with `resume_eligible == true` normalizes to `parked_resumable`;
- terminal lifecycle state normalizes to `terminal`.

This slice should not leave any read path free to infer parked versus attention-needed versus terminal from `owner_process_is_alive(...)` alone.

### 5. Add the missing regression tests

Required tests:

- a true `agent start` / hidden-owner-helper bootstrap regression proving that:
  - a fake host backend can emit a valid session handle,
  - then end the bootstrap control stream cleanly,
  - and the resulting session persists as:
    - `parked_resumable` when `pending_inbox_count == 0`,
    - `awaiting_attention` when detached pending inbox work exists,
    - and never `invalidated` solely because the bootstrap stream ended cleanly.
- a follow-up test proving that exact public `turn` succeeds against the session created by that bootstrap path.
- a follow-up test proving `reattach` restores attached ownership for that same session.
- a guard test proving that genuinely broken startup without resumability still fails closed.

Synthetic prewritten parked-session tests from `23` remain useful, but they are not sufficient on their own.

## Acceptance Criteria

This slice is done only when all of the following are true:

1. The original manual smoke path that motivated this work succeeds:
   - `substrate agent start --backend <host_backend_id> --prompt ...` can create a valid host session even if the bootstrap control stream ends cleanly after session establishment.
2. That session persists with the exact normalized posture required by ADR-0047 and `PLAN.md`:
   - `parked_resumable` when detached and `pending_inbox_count == 0`,
   - `awaiting_attention` when detached and `pending_inbox_count > 0`,
   - never `invalidated` or `terminal` solely because bootstrap ended cleanly.
3. `substrate agent turn --session ... --backend ... --prompt ...` succeeds against that exact parked session.
4. `substrate agent reattach --session ...` succeeds against that exact parked session and restores attached host ownership without submitting a prompt.
5. Truly broken startup without resumability still fails closed as `runtime_start_failed`.
6. Detached-world follow-up remains fail closed.
7. The prompt bridge still guarantees explicit terminal delivery after `Accepted`.
8. The durable posture, participant, and inbox features from `23` remain intact.

## Testing Expectations

Required coverage:

- startup readiness classification for hidden owner-helper launch
- bootstrap event-stream teardown classification
- bootstrap completion race alignment
- posture normalization from authoritative session + participant + pending-inbox truth
- public host `start` with a clean-exit fake backend
- public host `turn` after parked bootstrap
- public host `reattach` after parked bootstrap
- fail-closed startup when resumability prerequisites are absent
- existing parked-session synthetic tests as non-regression support

Recommended commands:

```bash
cargo test -p shell async_repl -- --nocapture
cargo test -p shell agent_runtime::state_store -- --nocapture
cargo test -p shell agent_runtime::control -- --nocapture
cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture
```

Manual validation should re-run the real CLI host flow, not only harness tests:

```bash
substrate agent start --backend <host_backend_id> --prompt "hello" --json
substrate agent turn --session <orchestration_session_id> --backend <host_backend_id> --prompt "next" --json
substrate agent reattach --session <orchestration_session_id> --json
```

The manual result is green only when the original failure mode is gone:

- no `runtime_start_failed` on clean bootstrap continuity,
- no `shell-owned orchestrator control stream ended before completion observation` terminal collapse for a resumable session,
- detached clean-exit with no pending inbox normalizes to `parked_resumable`,
- detached clean-exit with pending inbox work normalizes to `awaiting_attention`,
- and no need to rely on synthetic fixtures to claim the bootstrap path works.
