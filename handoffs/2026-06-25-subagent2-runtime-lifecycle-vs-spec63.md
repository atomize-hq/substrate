# Runtime lifecycle vs SPEC-63 debug log

Date: 2026-06-25
Repo: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
Mode: read-only debugging/research, except this paper-trail file.

## Objective

Compare current runtime code truth against `DESIGN-world-worker-lifecycle-model.md` and `SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md` to decide whether the original retained-member unregister/delete bug is still present, partially fixed, or replaced by a different runtime lifecycle failure.

## Sections inspected

### Prior/debug authorities
- `handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md`
- `handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md`
- `handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md`

### Design/spec/task authorities
- `llm-last-mile/DESIGN-world-worker-lifecycle-model.md`
- `llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md`
- `llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md`
- `llm-last-mile/TASKS-63.md`

### Current code/tests inspected
- `crates/world-service/src/member_runtime.rs`
- `crates/world-service/tests/member_runtime_retained_lifecycle_v1.rs`
- `crates/world-service/src/service.rs`
- `crates/shell/src/execution/orchestrator_world_dispatch.rs`
- `crates/shell/src/execution/agent_runtime/state_store.rs`
- `crates/shell/src/execution/agent_runtime/session.rs`
- `crates/shell/tests/agent_public_control_surface_v1.rs`

### GitNexus / verification notes
- `gitnexus status` reports index up to date at commit `856e4ce`.
- `gitnexus query/context` CLI calls timed out in this session, so findings below are from live source and targeted grep/read inspection.
- Local host is macOS; Linux-only runtime/public tests are compiled out or unavailable locally. Attempts to run exact Linux test names yielded `0 tests` or timed out while competing for Cargo locks. No green Linux runtime claim is made from this session.

## Current behavior vs SPEC-63 contract

| SPEC-63 / lifecycle contract | Current code truth | Assessment |
| --- | --- | --- |
| Clean bootstrap exit after authoritative retained identity plus resumable session identity is `running -> parked`, not retained-worker destruction. | `member_runtime.rs` now computes `preserve_retained_member` from clean exit plus `active.uaa_session_id().is_some()` and calls `finish_bootstrap(..., true)`, which only `close_bootstrap()` and leaves registry entries intact (`member_runtime.rs:272-290`, `468-482`). | Original unconditional unregister/delete bug is not present in the same form. It appears fixed for successful bootstrap with surfaced session handle. |
| Missing surfaced resumable identity must fail closed instead of pretending parked continuity. | `finish_bootstrap(..., false)` still calls `unregister_member`, and no-session-handle test coverage exists expecting later submit to fail as not retained (`member_runtime.rs:478-482`; `member_runtime_retained_lifecycle_v1.rs:537-588`). | Landed in code/test shape. |
| Later `submit_turn` must locate the parked retained worker, validate exact binding, and resume through surfaced session handle. | `submit_turn` calls `find_submit_target`, `validate_submit_turn_request`, `validate_submit_target_slot`, then requires `uaa_session_id` and builds a run request with `agent_api.session.resume.v1` (`member_runtime.rs:296-324`, `917-936`). | Landed in runtime code shape. |
| Non-zero submitted-turn exit must clean active-turn bookkeeping without deleting retained continuity by default. | Submitted-turn completion calls `manager.unregister_turn(&span_id)`, which clears only `active_turn_span_id`; it does not call `unregister_member` (`member_runtime.rs:485-528`). Regression coverage exists for failed turn then later resume (`member_runtime_retained_lifecycle_v1.rs:504-534`). | Landed in code/test shape. |
| Packet 4 must prove public follow-up over an already-created retained world-member slot, not pure public root start -> turn. | Shell regression now uses `ReadyAndExit`, waits for bootstrap-exited parked member, then public `agent turn` targets it (`agent_public_control_surface_v1.rs:5710-5848`). Comments explicitly preserve the SPEC-30 boundary. | Code/test shape matches the corrected Packet 4 scope. Not locally executed because Linux-only. |
| Ephemeral `run_world_task` remains terminal/non-retained even if backend surfaces continuity metadata. | Shell summary says terminal `run_world_task` may surface continuity metadata "without retained shell state" (`orchestrator_world_dispatch.rs:3852-3859`, test at `13349-13360`). But shell builds `run_world_task` as a `MemberDispatchTransportRequest` with `awm_...` participant and sends it through the same world-service `member_runtime.launch` path as retained workers (`orchestrator_world_dispatch.rs:1059-1118`, `2710-2734`). World-service preservation predicate does not distinguish action/mode/participant family; any clean launch that surfaces `uaa_session_id` is preserved in `active_members` (`member_runtime.rs:272-290`). | Likely new/remaining runtime defect: Packet 2 preservation may be over-broad and can retain ephemeral `run_world_task` launches inside the world-service registry, despite shell contract saying terminal/no retained shell state. |
| Later member allocation must not be blocked by stale/incorrect retained slots. | `register_member` rejects any existing `(orchestration_session_id, world_generation, backend_id)` retained key, regardless of participant prefix/action (`member_runtime.rs:429-454`). If an ephemeral `awm_...` `run_world_task` was preserved, a later `run_world_task`/`spawn_world_worker` on the same slot can fail at world-service launch; shell wraps launch failure as `failed to launch run_world_task over world member dispatch` (`orchestrator_world_dispatch.rs:2888-2894`). | This maps closely to the reported "subsequent attempts can fail with failed to launch run_world_task over world member dispatch" after an apparently completed run. |

## Packet landing status from current code truth

| Packet | Current status | Evidence / caveat |
| --- | --- | --- |
| Packet 2: Preserve retained continuity across clean bootstrap exit | Appears landed for retained bootstrap with surfaced session handle, but likely over-broad. | `finish_bootstrap` now preserves registry slots on clean exit with any `uaa_session_id`; unit/integration test shapes exist. Missing discriminator means ephemeral dispatch can also be preserved. |
| Packet 3: Make parked resume work and keep turn failure non-terminal | Appears landed in world-service code shape. | `submit_turn` resumes by `uaa_session_id`; submitted-turn cleanup uses `unregister_turn` / slot clear, not member deletion. Linux integration tests exist for clean resume and fail-then-resume. |
| Packet 4: Shell/public already-created retained member follow-up | Appears landed in source/test shape, with corrected scope comments. | `public_turn_routes_linux_world_member_follow_up_through_typed_submit_path` now models `ReadyAndExit` and public follow-up after parked member creation. Not locally verified on macOS. |

## Most likely remaining runtime lifecycle defects

1. **Over-broad preservation in `MemberRuntimeManager::launch`:** the runtime decides retained continuity solely from successful exit plus surfaced `uaa_session_id`, not from a retained action/mode/role. Since `run_world_task` uses the same `member_dispatch` path with `awm_...` participant IDs, a successful ephemeral task that surfaces session metadata can be kept in `active_members` and occupy the `(orchestration_session_id, world_generation, backend_id)` retained slot.
2. **Retained-slot poisoning after terminal ephemeral work:** once an `awm_...` ephemeral task is preserved, `register_member` can reject the next allocation for the same session/generation/backend as a duplicate retained member. The shell wraps execute-stream launch errors as `failed to launch run_world_task over world member dispatch`, matching the reported subsequent-attempt failure family.
3. **Host-visible file side effect remains a separate placement/sync issue:** `run_world_task` returning completed does not prove host-visible file materialization. Prior handoff already separated world overlay persistence from host reconciliation; SPEC-63 explicitly excludes workspace sync/reconciliation. The current runtime lifecycle defect can coexist with that symptom but is not itself proof that the file was not written inside the world.

## What is ruled out

- The exact old bug, "clean bootstrap with surfaced session handle always calls `unregister_member`", is ruled out by current `finish_bootstrap(..., true)` code.
- Shell-side exact-target `continue_world_worker` request building is not the primary mismatch: it still builds exact participant/world/backend requests and submits over `/member-turn` (`orchestrator_world_dispatch.rs:2819-2864`, `3101-3105`).
- Raw missing `uaa_session_id` preservation is not the retained-path primary issue: current code remembers session IDs from events and completion and requires them before resume.
- Packet 4 no longer appears to be falsely claiming pure public root start -> turn; the test comments and setup keep it scoped to follow-up over an already-created retained world member.
- Host-visible side effects from `run_world_task` are not resolved by SPEC-63; absence of a host-visible file after "completed" is not, by itself, proof that retained resume is still broken.

## Next 3 decisive checks/fixes

1. **Add a Linux regression for ephemeral `run_world_task` followed by another same-session world dispatch:** fake runtime should surface a session handle and exit 0 on the first `run_world_task`; assert world-service unregisters/does not retain the `awm_...` slot and the second dispatch does not hit duplicate retained slot ownership.
2. **Add an explicit dispatch lifecycle discriminator to the world-service member launch contract:** `MemberDispatchRequestV1` or adjacent transport should distinguish `ephemeral` vs `retained`; `finish_bootstrap` preservation must require retained mode plus clean exit plus surfaced resumable identity.
3. **Capture world-service error body/logs for the reported second-attempt failure:** confirm whether the wrapped `failed to launch run_world_task over world member dispatch` contains duplicate retained member / retained slot owner mismatch mentioning an `awm_...` participant. That would directly confirm slot poisoning.

## Final take

The original retained-member unregister/delete bug is **partially fixed and not present in its old unconditional form**. Packets 2 and 3 appear to have repaired retained parked/resume behavior, and Packet 4 appears source-landed for the narrowed public follow-up proof. The remaining/similar failure is most likely a **new runtime-side lifecycle overcorrection**: world-service now preserves any clean member runtime that surfaces `uaa_session_id`, including ephemeral `run_world_task`, which contradicts shell's terminal/non-retained contract and can poison the retained slot for later same-session dispatch. Treat host-visible file absence as an adjacent overlay/reconciliation issue unless logs prove the runtime never executed in the world.
