# PROMPTS-63: Packet Orchestration Prompts For Slice 63

Source spec: [SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md](./SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md)  
Source plan: [PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md](./PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md)  
Source tasks: [TASKS-63.md](./TASKS-63.md)  
Related lifecycle authority:
- [DESIGN-world-worker-lifecycle-model.md](./DESIGN-world-worker-lifecycle-model.md)  
Related architectural inputs:
- [DESIGN-host-orchestrator-world-dispatch-contract.md](./DESIGN-host-orchestrator-world-dispatch-contract.md)
- [DESIGN-retained-world-worker-messaging-and-steering-contract.md](./DESIGN-retained-world-worker-messaging-and-steering-contract.md)
- [DESIGN-host-orchestrator-tool-invocation-surface.md](./DESIGN-host-orchestrator-tool-invocation-surface.md)
- [DESIGN-internal-toolbox-transport-and-session-binding.md](./DESIGN-internal-toolbox-transport-and-session-binding.md)
- [docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md](../docs/adr/implemented/ADR-0047-host-orchestrator-durable-session-and-parked-resumable-ownership.md)  
Related handoffs:
- [handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md](../handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md)
- [handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md](../handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md)
- [handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md](../handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill dependency: `$incremental-implementation`  
Worker review skill dependency: `$code-review-and-quality`  
Resolved implementation skill path: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Resolved review skill path: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`  
Workspace root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`

Skill availability note:

1. These prompts require a fresh parent session that can use `$incremental-implementation` for implementation/fix workers and `$code-review-and-quality` for review workers.
2. The fresh parent session should verify both skills are available before spawning workers.
3. If either required skill alias is unavailable, the parent session must load the skill doc from the resolved path above and preserve the same workflow explicitly; do not silently omit the skill discipline.
4. If a required skill path is unavailable, stop immediately and report the missing dependency instead of substituting another workflow.

GitNexus tooling note:

1. These prompts assume the parent session can run `gitnexus_detect_changes()` before commits when non-test code changed.
2. If that exact tool surface is unavailable in the parent session, stop and report the missing GitNexus capability instead of silently skipping the scope verification step.

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded in the live Slice `63` spec/plan/tasks stack and preserves the packet boundary: this slice is a bounded retained-world-worker parked/resume lifecycle repair, not `SPEC-62` bootstrap compatibility work, not config/auth projection, not workspace sync, not router/inbox redesign, and not unrelated worker behavior.

## Packet 1 Prompt

```text
/goal Orchestrate and land TASKS-63 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-63.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md

Mission:
- Land Packet 1 only: Pin The Lifecycle Seam In World-Service.
- Keep the work bounded to Task 1.1 in TASKS-63.
- Do not start Packet 2.

Required orchestration loop:
1. Verify the parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md before dispatching any worker.
3. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
4. Spawn a fresh GPT-5.4 subagent on high for implementation.
5. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
6. When the implementation subagent completes and Packet 1 verification is green, rerun the packet verification in the parent session, run `gitnexus_detect_changes()` before commit if any non-test code changed, and commit the implementation changes before review.
7. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
8. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
9. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
10. After each fix round, rerun the relevant verification, run `gitnexus_detect_changes()` before commit if any non-test code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
11. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Inspect the live code and test surface in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/tests/
2. If GitNexus says the index is stale, run `npx gitnexus analyze`.
3. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add focused `world-service` lifecycle regression coverage for bootstrap-exit parked resume.

Locked defaults to preserve:
- retained continuity and process continuity separate once resumable identity is surfaced
- no new persisted parked field in this packet
- prove the lifecycle seam in `world-service` first

Out of scope:
- Packet 2 or later packets
- changing runtime lifecycle behavior beyond what is required to pin the failing seam in tests
- `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-63 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs first. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Work only on Task 1.1. Add the minimum honest world-service lifecycle regression coverage that proves: retained registration happens, resumable session identity can be surfaced, bootstrap can exit cleanly, the worker remains resumable for later submit-turn, and the inverse no-session-handle case fails closed. Keep the test honest about bootstrap exit; do not rely only on held-open runtime behavior. Run `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture` and `cargo test -p world-service member_runtime -- --nocapture`. Do not commit. Final message must state whether Packet 1 is checkpoint-green, what files changed, what GitNexus impact results were found, what verification ran, and whether Packet 2 is unblocked.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  /goal Review the committed Slice 63 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. Review only Packet 1 and the live diff. Focus on correctness of the lifecycle repro, honesty of the positive and inverse cases, packet-boundary discipline, and whether broader runtime redesign or later-packet behavior was smuggled in. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 63 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 1 issues without widening scope. Re-run `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture` and `cargo test -p world-service member_runtime -- --nocapture`. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture`
  - `cargo test -p world-service member_runtime -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run `gitnexus_detect_changes()` before each commit if any non-test code changed.

Packet 1 checkpoint:
- the active lifecycle seam reproduces in automation
- the positive parked-resume success condition is explicit
- the no-session-handle inverse case is pinned
- the proof does not depend on bootstrap process liveness being artificially held open

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results if any production symbols were edited; otherwise say explicitly that the packet remained test-only.
- Report GitNexus detect-changes results before each commit if it was required.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Orchestrate and land TASKS-63 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-63.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md

Mission:
- Land Packet 2 only: Preserve Retained Continuity Across Clean Bootstrap Exit.
- Keep the work bounded to Task 2.1 and Task 2.2 in TASKS-63.
- Do not start Packet 3.

Required orchestration loop:
1. Verify the parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md before dispatching any worker.
3. Verify Packet 1 is already landed and checkpoint-green on the current tree.
4. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
5. Spawn a fresh GPT-5.4 subagent on high for implementation.
6. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
7. When the implementation subagent completes and Packet 2 verification is green, rerun the packet verification in the parent session, run `gitnexus_detect_changes()` before commit if any non-test code changed, and commit the implementation changes before review.
8. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
9. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
10. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each fix round, rerun the relevant verification, run `gitnexus_detect_changes()` before commit if any non-test code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
12. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Inspect the live code and test surface in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs
2. If GitNexus says the index is stale, run `npx gitnexus analyze`.
3. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.
4. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Separate retained registry ownership from bootstrap-process cleanup.
- Task 2.2: Keep parked truth inferred rather than adding a persisted parked field.

Locked defaults to preserve:
- retained continuity independent of process continuity once resumable identity exists
- smallest acceptable change is preserving the existing retained registry entry in place
- a minimal internal split is acceptable only if preserving the entry in place becomes too tangled
- do not add a new persisted parked field unless implementation proves it is required; if so, stop and update the spec/plan instead of silently widening

Out of scope:
- Packet 3 or later packets
- changing submit-turn semantics beyond what is required to preserve retained continuity across bootstrap exit
- `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-63 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs first. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Work only on Task 2.1 and Task 2.2. Make retained continuity survive clean bootstrap exit once authoritative retained identity plus surfaced resumable session identity exist. Prefer preserving the existing retained registry entry in place. If a minimal internal split between retained membership and active bootstrap/turn bookkeeping is required, keep it minimal and local. Keep parked truth inferred rather than adding a new persisted parked field; if you discover that a persisted field is required, stop and report instead of widening silently. Run `cargo test -p world-service member_runtime -- --nocapture` and `rg -n "unregister_member|remember_uaa_session_id|launcher_dir|bootstrap|parked|resume|active_turn_span_id|uaa_session_id" crates/world-service/src/member_runtime.rs`. Do not commit. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what GitNexus impact results were found, what verification ran, and whether Packet 3 is unblocked.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  /goal Review the committed Slice 63 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. Review only Packet 2 and the live diff. Focus on correctness of retained continuity preservation, cleanup-vs-closeout separation, packet-boundary discipline, and whether a persisted parked field or broader lifecycle redesign was smuggled in. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 63 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 2 issues without widening scope. Re-run `cargo test -p world-service member_runtime -- --nocapture` and `rg -n "unregister_member|remember_uaa_session_id|launcher_dir|bootstrap|parked|resume|active_turn_span_id|uaa_session_id" crates/world-service/src/member_runtime.rs`. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p world-service member_runtime -- --nocapture`
  - `rg -n "unregister_member|remember_uaa_session_id|launcher_dir|bootstrap|parked|resume|active_turn_span_id|uaa_session_id" crates/world-service/src/member_runtime.rs`
  - `git diff --stat`
  - `git status --short`
- Run `gitnexus_detect_changes()` before each commit if any non-test code changed.

Packet 2 checkpoint:
- retained continuity survives clean bootstrap exit
- no duplicate retained-slot or stale-owner drift is introduced
- bootstrap resource cleanup no longer implies retained closeout
- no new persisted parked field has been added without explicit review

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit if it was required.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Orchestrate and land TASKS-63 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-63.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md

Mission:
- Land Packet 3 only: Make Parked Resume Work And Keep Turn Failure Non-Terminal By Default.
- Keep the work bounded to Task 3.1 and Task 3.2 in TASKS-63.
- Do not start Packet 4.

Required orchestration loop:
1. Verify the parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md before dispatching any worker.
3. Verify Packet 2 is already landed and checkpoint-green on the current tree.
4. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
5. Spawn a fresh GPT-5.4 subagent on high for implementation.
6. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
7. When the implementation subagent completes and Packet 3 verification is green, rerun the packet verification in the parent session, run `gitnexus_detect_changes()` before commit if any non-test code changed, and commit the implementation changes before review.
8. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
9. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
10. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each fix round, rerun the relevant verification, run `gitnexus_detect_changes()` before commit if any non-test code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
12. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Inspect the live code and test surface in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/tests/
2. If GitNexus says the index is stale, run `npx gitnexus analyze`.
3. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.
4. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Make `submit_turn` resume parked retained workers through preserved retained membership.
- Task 3.2: Keep non-zero submitted-turn exit from automatically killing retained continuity.

Locked defaults to preserve:
- submitted-turn resume works via surfaced `uaa_session_id`
- exact `participant_id`, `orchestration_session_id`, `orchestrator_participant_id`, `backend_id`, `world_id`, and `world_generation` validation remain fail-closed
- after resumable identity exists, non-zero submitted-turn exit must not automatically destroy retained continuity unless explicit stop, invalidation, or equivalent terminal closeout occurs

Out of scope:
- Packet 4 or later packets
- shell-side redesign unless a narrow integration adjustment is actually required
- `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-63 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs first. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Work only on Task 3.1 and Task 3.2. Ensure `submit_turn` can locate and resume parked retained workers through preserved retained membership using surfaced `uaa_session_id`, while keeping exact identity and binding validation fail-closed. Also ensure non-zero submitted-turn exit cleans up active-turn bookkeeping but does not automatically unregister/delete the retained worker unless explicit terminal closeout semantics occur. Run `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`, `cargo test -p world-service member_runtime -- --nocapture`, and `rg -n "submit_turn|unregister_turn|clear_reserved_turn_slot|unregister_member" crates/world-service/src/member_runtime.rs`. Do not commit. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what GitNexus impact results were found, what verification ran, and whether Packet 4 is unblocked.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  /goal Review the committed Slice 63 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. Review only Packet 3 and the live diff. Focus on correctness of parked resume, exact-binding validation preservation, separation of turn failure from worker death, and packet-boundary discipline. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 63 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 3 issues without widening scope. Re-run `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`, `cargo test -p world-service member_runtime -- --nocapture`, and `rg -n "submit_turn|unregister_turn|clear_reserved_turn_slot|unregister_member" crates/world-service/src/member_runtime.rs`. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`
  - `cargo test -p world-service member_runtime -- --nocapture`
  - `rg -n "submit_turn|unregister_turn|clear_reserved_turn_slot|unregister_member" crates/world-service/src/member_runtime.rs`
  - `git diff --stat`
  - `git status --short`
- Run `gitnexus_detect_changes()` before each commit if any non-test code changed.

Packet 3 checkpoint:
- later submit-turn succeeds after clean bootstrap exit
- exact-binding mismatch still fails closed
- missing surfaced resume handle still fails directly
- non-zero submitted-turn exit no longer implies retained-worker deletion
- explicit terminal closeout still removes resumability

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit if it was required.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Orchestrate and land TASKS-63 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-63.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-retained-world-worker-lifecycle-debug.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-23-design-lineage-audit-retained-worker-resume.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-22-211455-spec-62-world-worker-retained-debug.md

Mission:
- Land Packet 4 only: Keep One Shell/Public Regression For Exact-Target Routing Truth.
- Keep the work bounded to Task 4.1 in TASKS-63.
- Do not start Packet 5.

Required orchestration loop:
1. Verify the parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md before dispatching any worker.
3. Verify Packet 3 is already landed and checkpoint-green on the current tree.
4. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
5. Spawn a fresh GPT-5.4 subagent on high for implementation.
6. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
7. When the implementation subagent completes and Packet 4 verification is green, rerun the packet verification in the parent session, run `gitnexus_detect_changes()` before commit if any non-test code changed, and commit the implementation changes before review.
8. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
9. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
10. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each fix round, rerun the relevant verification, run `gitnexus_detect_changes()` before commit if any non-test code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
12. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Inspect the live code and test surface in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
2. If GitNexus says the index is stale, run `npx gitnexus analyze`.
3. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.
4. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Extend one Linux public regression across the bootstrap-exits-then-resume seam.

Locked defaults to preserve:
- keep one shell/public regression only
- avoid widening into shell-side lifecycle redesign
- exact-target routing truth must stay aligned with repaired `world-service` lifecycle semantics

Out of scope:
- Packet 5 or later packets except stating whether Packet 5 is unblocked
- broader shell lifecycle redesign
- `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-63 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs first. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Work only on Task 4.1. Extend exactly one Linux public regression so `continue_world_worker` proves exact-target routing and success after the retained worker has exited bootstrap and entered parked/resumable posture; do not rely solely on held-open member runtime behavior. Keep shell-side changes minimal and only if a narrow routing mismatch is uncovered. Run `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture` and `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`. Do not commit. Final message must state whether Packet 4 is checkpoint-green, what files changed, what GitNexus impact results were found, what verification ran, and whether Packet 5 is unblocked.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  /goal Review the committed Slice 63 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. Review only Packet 4 and the live diff. Focus on correctness of the public regression, exact-target routing truth, minimal shell-side scope, and whether broader shell lifecycle redesign was smuggled in. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 63 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus SPEC-63, PLAN-63, TASKS-63, DESIGN-world-worker-lifecycle-model.md, and the listed handoffs. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 4 issues without widening scope. Re-run `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture` and `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture`
  - `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run `gitnexus_detect_changes()` before each commit if any non-test code changed.

Packet 4 checkpoint:
- the public regression proves exact-target routing across parked handoff
- shell-side exact-target validation remains unchanged
- no broader shell lifecycle redesign has been introduced

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit if it was required.
- State whether Packet 5 is unblocked.
- If anything is not green, say explicitly that Packet 5 must not begin.
```

## Packet 5 Prompt

```text
/goal Orchestrate and land TASKS-63 Packet 5 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-63-retained-world-worker-parked-resume-session-handle-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-63.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/DESIGN-world-worker-lifecycle-model.md

Mission:
- Land Packet 5 only: Final Validation Wall.
- Keep the work bounded to Task 5.1 in TASKS-63.
- This is a validation-only packet unless the verification wall uncovers issues.

Required orchestration loop:
1. Verify the parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md before dispatching any worker.
3. Verify Packet 4 is already landed and checkpoint-green on the current tree.
4. Inspect `git status --short` and preserve unrelated dirt.
5. Spawn a fresh GPT-5.4 subagent on high for the validation run.
6. The validation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`, but this packet is validation-only: no proactive code changes unless the parent explicitly chooses to open a fix loop after a failing command.
7. When the validation subagent completes, assess whether Packet 5 is already green. If yes, do not create an empty commit.
8. If Packet 5 is not green because validation found real issues, spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality` to review the failed state and triage whether the issue belongs to Slice 63 scope.
9. If the review confirms the issue is an in-scope Slice 63 defect, spawn a fresh GPT-5.4 subagent on high to fix only those in-scope findings.
10. Any fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each in-scope fix round, rerun the relevant verification, run `gitnexus_detect_changes()` before commit if any non-test code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent until review-clean and validation-green.
12. If validation fails only because of out-of-scope or pre-existing issues, stop and report instead of widening this slice.

Commit policy:
- Do not create an empty commit for a green validation-only packet.
- If an in-scope fix round is required, commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Stay strictly within Packet 5 scope.
2. If GitNexus says the index is stale and an in-scope code fix becomes necessary, run `npx gitnexus analyze`.
3. Before editing any production Rust symbol in a fix round, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.

Packet 5 scope:
- Task 5.1: Run the targeted validation wall for the retained parked/resume seam.

Out of scope:
- speculative cleanup after a green validation wall
- widening the slice to absorb unrelated failures
- `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior

Validation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Validation subagent prompt:
  /goal Execute TASKS-63 Packet 5 validation only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md first. Do not make code changes unless the parent session later sends you back for a fix round. Run only the Packet 5 validation wall and report results precisely: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture`, `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`, `cargo test -p world-service member_runtime -- --nocapture`, `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture`, and `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`. Final message must state whether Packet 5 is green, list exact command outcomes, identify any failures as in-scope vs out-of-scope candidates, and state whether a fix loop is necessary.

Review worker requirements if validation fails:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  /goal Review the failed Slice 63 Packet 5 validation state in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md. Review only whether the failing verification belongs to Slice 63 scope and whether a fix is required before closeout. Report findings first with explicit severities. State clearly whether the failure is in-scope for Slice 63 or should be reported without widening scope.

Fix worker requirements if review confirms an in-scope issue:
- Spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required in-scope Slice 63 Packet 5 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus SPEC-63, PLAN-63, TASKS-63, and DESIGN-world-worker-lifecycle-model.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged in-scope Slice 63 issues without widening scope. Re-run the exact failing Packet 5 verification commands plus any dependent checks needed to prove green. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 5 is green, and whether another review round is required.

Verification and commit requirements for the parent session:
- Always run or confirm the full Packet 5 wall:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p world-service bootstrap_completion_with_session_handle_emits_registered_then_exit -- --nocapture`
  - `cargo test -p world-service find_submit_target_rejects_participant_id_drift_for_retained_slot -- --nocapture`
  - `cargo test -p world-service member_runtime -- --nocapture`
  - `cargo test -p shell resolve_internal_continue_world_dispatch_target_returns_exact_retained_worker -- --nocapture`
  - `cargo test -p shell public_turn_routes_linux_world_member_follow_up_through_typed_submit_path -- --nocapture`
- If an in-scope fix round occurs and any non-test code changed, run `gitnexus_detect_changes()` before commit.
- If the packet stays validation-only and green with no diff, do not create an empty commit; report that explicitly.

Packet 5 checkpoint:
- the world-service lifecycle seam is proven green
- the shell/public exact-target proof is green
- the slice did not widen into `SPEC-62` bootstrap compatibility, config/auth projection, workspace sync, router/inbox redesign, or unrelated worker behavior

Final response requirements:
- State whether Packet 5 is green.
- List exact verification commands run and whether they passed.
- If no code changed, say explicitly that no commit was created because Packet 5 was validation-only.
- If an in-scope fix round occurred, report GitNexus impact-analysis results for edited production symbols and `gitnexus_detect_changes()` results before each commit.
- If anything remains red, state explicitly whether it is in-scope blocked work or out-of-scope follow-up.
```
