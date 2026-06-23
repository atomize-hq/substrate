# Retained World Worker Lifecycle Debug Log

## 2026-06-23 14:23:18 EDT — Investigation start

### Scope
- Debugging-only investigation of retained world-worker continuation lifecycle.
- Primary authority: `handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md`.
- Highest-likelihood thread only: retained member lifecycle mismatch around bootstrap completion, `active_members` unregistration, surfaced `uaa_session_id` retention, and `continue_world_worker` resume semantics.

### Initial constraints
- No implementation fix yet unless required for a definitive proof artifact.
- Keep exactly this one paper-trail artifact updated in-place.
- GitNexus-first workflow required; MCP tools unavailable in-session, so CLI fallback will be used and recorded.

### Initial repo state
- CWD: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- `git status --short --branch`: `## feat/internal-host-orchestrator-world-dispatch-bootstrap...origin/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Target artifact path created fresh for this run.

## 2026-06-23 14:23:36 EDT — GitNexus setup / index state

### Observations
- `gitnexus status` reported this repo as **stale**: indexed commit `017aaec`, current commit `43fa73b`.
- Initial CLI `gitnexus query/context` calls failed because multiple repos are indexed; explicit `--repo` selection is required in this environment.
- Per the GitNexus debugging skill, the next step is to refresh the index before trusting query/context output.

### Untrusted output note
- The CLI error text was treated as diagnostic data only. No embedded instructions were followed automatically beyond the repo's pre-existing GitNexus stale-index rule.

## 2026-06-23 14:32:25 EDT — GitNexus-first mapping results

### Commands run
- `gitnexus status` → stale index at `017aaec` vs current `43fa73b`
- `gitnexus analyze . --index-only` → refreshed index successfully
- `gitnexus query --repo /Users/spensermcconnell/__Active_Code/atomize-hq/substrate ... continue_world_worker retained member unregister_member active_members uaa_session_id resume`
- `gitnexus context --repo /Users/spensermcconnell/__Active_Code/atomize-hq/substrate --file crates/world-service/src/member_runtime.rs launch`
- `gitnexus context --repo /Users/spensermcconnell/__Active_Code/atomize-hq/substrate --file crates/world-service/src/member_runtime.rs submit_turn`
- `gitnexus context --repo /Users/spensermcconnell/__Active_Code/atomize-hq/substrate unregister_member`
- `gitnexus cypher --repo /Users/spensermcconnell/__Active_Code/atomize-hq/substrate ...` to enumerate key functions in `member_runtime.rs`

### What GitNexus confirmed
- `crates/world-service/src/member_runtime.rs` contains the exact retention seam functions: `launch`, `submit_turn`, `register_member`, `unregister_member`, `find_submit_target`, `validate_submit_target_slot`, and `build_submitted_turn_run_request`.
- Query results also surfaced shell-side authoritative-routing definitions/tests for `continue_world_worker`, including `resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker` and `public_turn_routes_linux_world_member_follow_up_through_typed_submit_path`.
- GitNexus did **not** directly return a clean end-to-end retained-bootstrap-exit process for the real bug; the most useful output was function localization plus related routing definitions/tests.

### Immediate implication
GitNexus localization aligns with the handoff hypothesis: the likely failure is a seam mismatch between shell-side authoritative retained-worker routing and world-service's in-memory retained-member registry.

## 2026-06-23 14:32:25 EDT — Reproduction/evidence pass

### Targeted tests run
- `cargo test -p shell public_start_persists_detached_session_when_hidden_owner_helper_exits -- --nocapture` → **passed**
- `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture` → **passed**

### Why these two tests matter
1. `public_start_persists_detached_session_when_hidden_owner_helper_exits` proves the broader session model intentionally preserves a durable surfaced session handle across post-bootstrap process exit. The test asserts persisted `internal.uaa_session_id == "thread-test"` and `internal.resume_eligible == true` after the helper exits (`crates/shell/tests/agent_public_control_surface_v1.rs:4588-4683`).
2. `resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker` proves shell-side follow-up routing will continue to target the persisted retained worker record when the authoritative session/world linkage still matches (`crates/shell/src/execution/agent_runtime/state_store.rs:1431-1534`, tested at `8882-8925`).

### Reproduction limit encountered
- I attempted to build a standalone live repro outside the repo using the public `world-service` crate API, but on this macOS target the Linux shared-world helper surface needed for `ensure_session_world` is unavailable from the public API. I did **not** patch repo code just to force a platform-specific repro.
- Because of that platform limit, I reduced the bug using live source + targeted tests that independently prove the two conflicting halves of the contract.

## 2026-06-23 14:32:25 EDT — Source-level narrowing

### World-service lifecycle behavior (suspect seam)
- `MemberRuntimeManager::launch` registers the retained member before the bootstrap stream begins (`crates/world-service/src/member_runtime.rs:187-215`).
- It remembers any surfaced `uaa_session_id` while bootstrap events/completion arrive (`238-263`).
- **Immediately after bootstrap completion frames are emitted, it always calls `manager.unregister_member(&participant_id)`** (`278`).
- `unregister_member` removes both `by_participant_id` and `by_retained_key` entries from `active_members` (`447-455`).

### Continuation path expectations
- `continue_world_worker` resolves an exact retained target, builds a `MemberTurnSubmitRequestV1`, and submits it over the existing member-turn seam (`crates/shell/src/execution/orchestrator_world_dispatch.rs:1189-1229`, `2819-2861`, `3099-3105`).
- `resolve_internal_continue_world_dispatch_target` explicitly treats the persisted retained worker as the authoritative follow-up target when session/world/backend linkage still matches and the participant is considered authoritative-live (`crates/shell/src/execution/agent_runtime/state_store.rs:1431-1534`).

### Why the mismatch is definitive
- `submit_turn` in world-service requires the target retained member to still be present in `active_members` (`member_runtime.rs:284-296`).
- `find_submit_target` fails closed if the participant has already been removed, emitting `member_turn_submit.participant_id <id> is not retained` (`515-541`).
- `validate_submit_target_slot` also fails if the retained slot key is gone, emitting `member_turn_submit retained member is not active ...` (`544-559`).
- Therefore, once bootstrap completion exits and `launch` unregisters the member, the continuation path has no in-memory retained registry entry left to resume against — even if shell-side state still has the persisted `uaa_session_id` and still routes follow-up work to that participant.

## 2026-06-23 14:32:25 EDT — Test gap / smallest convincing explanation

### Existing coverage gap
- The relevant shell/public world follow-up test `public_turn_routes_linux_world_member_follow_up_through_typed_submit_path` proves typed submit routing **only while the member runtime stays live and held open** via `ReadyAndHoldUntilCancel` (`crates/shell/tests/agent_public_control_surface_v1.rs:5625-5848`).
- The world-service member runtime unit test `bootstrap_completion_with_session_handle_emits_registered_then_exit` proves bootstrap can legally emit a registered event and then exit cleanly when a session handle is present (`crates/world-service/src/member_runtime.rs:2213-2256`).
- There is no current joined regression that covers: **registered retained worker → bootstrap exits cleanly → later continue by surfaced session handle**.

### Smallest convincing explanation
1. Real bootstrap can finish cleanly and surface continuity metadata/session handle.
2. Shell-side control semantics intentionally preserve that durable handle and keep the retained participant routable.
3. World-service deletes the retained member registry entry as soon as bootstrap exits.
4. The next `continue_world_worker` call submits to world-service expecting retained continuity, but world-service no longer has any retained member to resume.

## 2026-06-23 14:32:25 EDT — Final suspected definitive bug

### Root cause statement
**Definitive bug:** `crates/world-service/src/member_runtime.rs:MemberRuntimeManager::launch` models retained-member liveness as the lifetime of the bootstrap `codex exec` process and unconditionally calls `unregister_member` when that first exec exits (`278`). That destroys both `active_members.by_participant_id` and `by_retained_key` (`447-455`) before later `continue_world_worker` turns arrive. But shell-side routing still treats the participant as the authoritative retained worker because it preserves the surfaced `uaa_session_id` / resumable linkage in durable participant state (`session.rs:613-619`; `agent_public_control_surface_v1.rs:4656-4683`; `state_store.rs:1431-1534`). The continuation request is therefore routed to a participant that world-service has already forgotten, so the member-turn submit seam fails with a not-retained / missing-retained-slot error instead of resuming by durable session handle.

### Exact symbols/files involved
- `crates/world-service/src/member_runtime.rs`
  - `MemberRuntimeManager::launch`
  - `MemberRuntimeManager::unregister_member`
  - `MemberRuntimeManager::submit_turn`
  - `MemberRuntimeManager::find_submit_target`
  - `MemberRuntimeManager::validate_submit_target_slot`
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
  - `continue_world_worker`
  - `build_continue_world_worker_submit_request`
  - `execute_continue_world_worker_stream_for_turn_kind`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
  - `resolve_internal_continue_world_dispatch_target`
- `crates/shell/src/execution/agent_runtime/session.rs`
  - `AgentRuntimeParticipantRecord::set_uaa_session_id`

### Confidence
- **High** on the root-cause seam mismatch itself.
- Slightly below absolute certainty only because I did not force a Linux-only live world-service repro by editing repo code or fabricating a platform-specific harness inside the repo. The source contract plus targeted tests is still strongly conclusive.
