# PROMPTS-61: Packet Orchestration Prompts For Slice 61

Source spec: [SPEC-61-human-public-agent-start-streaming-regression.md](./SPEC-61-human-public-agent-start-streaming-regression.md)  
Source plan: [PLAN-61-human-public-agent-start-streaming-regression.md](./PLAN-61-human-public-agent-start-streaming-regression.md)  
Source tasks: [TASKS-61.md](./TASKS-61.md)  
Related frozen floor:
- [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)
- [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill dependency: `$incremental-implementation`  
Worker review skill dependency: `$code-review-and-quality`  
Workspace root: `/home/spenser/__Active_code/substrate`

Skill availability note:

1. These prompts require a fresh parent session that can use `$incremental-implementation` for implementation/fix workers and `$code-review-and-quality` for review workers.
2. I could not resolve a local installed path for either skill from this session’s visible skill directories, so the fresh parent session should verify that both skills are available before spawning workers.
3. If either required skill is unavailable, the orchestration agent must stop immediately and report the missing dependency instead of silently substituting a different workflow.

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `61` spec/plan/tasks stack. Preserve strict identity validation, preserve the current `--json` public prompt envelope contract, and keep the slice bounded to public prompt event construction and non-JSON rendering.

## Packet 1 Prompt

```text
/goal Land Slice 61 Packet 1 only in /home/spenser/__Active_code/substrate.

Use these source docs as authority:
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md

Mission:
- Land Slice 61 Packet 1 only: Pin The Human Streaming Contract.
- Do not start Packet 2.
- Keep the work bounded to Task 1.1 and Task 1.2 in TASKS-61.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Spawn a fresh GPT-5.4 subagent on high for implementation.
3. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
4. When implementation completes and local verification is green, commit the implementation changes before any review.
5. Spawn a fresh GPT-5.4 subagent on high for review.
6. The review subagent must explicitly use `$code-review-and-quality`.
7. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
8. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
9. After each fix round, rerun the relevant verification, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
10. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-61, PLAN-61, and TASKS-61 first.
2. Inspect the live test surface in:
   - /home/spenser/__Active_code/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
   - /home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/control.rs if Packet 1 needs focused renderer/control-unit coverage
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. If you edit any production Rust symbol, run GitNexus impact analysis before editing and report the blast radius. If Packet 1 remains test-only, say so explicitly.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add plain human `agent start` streaming regression coverage.
- Task 1.2: Extend coverage to the shared non-JSON turn/renderer seam.

Out of scope:
- Packet 2, 3, or 4 work
- shared identity normalization
- non-JSON renderer fallback changes beyond what is strictly required for Packet 1 coverage
- weakening validation
- changing the JSON wire contract

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Land Slice 61 Packet 1 only in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md first. Work only on Task 1.1 and Task 1.2. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any production Rust symbol, run GitNexus impact analysis before editing and report the blast radius; otherwise state explicitly that Packet 1 remained test-only. Add the minimum regression coverage needed so the silent plain-human path reproduces in automation and the success condition requires visible streamed lines before completion. Keep Packets 2 through 4 out of scope. Run `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`, plus any focused control test you add. Final message must state whether Packet 1 is checkpoint-green, what files changed, what verification ran, whether Packet 2 is unblocked, and whether any production-symbol impact analysis was required.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 61 Packet 1 change in /home/spenser/__Active_code/substrate using $code-review-and-quality. Ground the review in /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to a regression-test packet. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 61 Packet 1 review findings in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read the review findings plus /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any production Rust symbol, run GitNexus impact analysis before editing and report the blast radius; otherwise state explicitly that the fix remained test-only. Fix only the flagged Packet 1 issues without widening scope. Re-run `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`, plus any focused control test implicated by the fix. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - any added focused control test command
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit if any non-test code changed.

Packet 1 checkpoint:
- the silent human-path bug reproduces in automation
- the success condition requires visible streamed lines before completion
- the existing NDJSON control-path coverage remains green

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results if any production symbols were edited; otherwise say explicitly that Packet 1 stayed test-only.
- Report GitNexus detect-changes results before each commit if it was required.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 61 Packet 2 only in /home/spenser/__Active_code/substrate.

Use these source docs as authority:
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md

Mission:
- Land Slice 61 Packet 2 only: Normalize Emitted Tuple Clients At The Shared Source.
- Do not start Packet 3.
- Keep the work bounded to Task 2.1 and Task 2.2 in TASKS-61.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Spawn a fresh GPT-5.4 subagent on high for implementation.
3. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
4. When implementation completes and local verification is green, commit the implementation changes before any review.
5. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
6. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
7. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
8. After each fix round, rerun the relevant verification, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
9. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-61, PLAN-61, and TASKS-61 first.
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. Inspect the live code in:
   - /home/spenser/__Active_code/substrate/crates/common/src/identity.rs
   - /home/spenser/__Active_code/substrate/crates/common/src/agent_events.rs
   - /home/spenser/__Active_code/substrate/crates/common/tests/agent_hub_event_envelope_schema.rs
   - /home/spenser/__Active_code/substrate/crates/shell/src/builtins/world_gateway.rs
   - /home/spenser/__Active_code/substrate/crates/shell/tests/world_gateway.rs
4. If GitNexus says the index is stale, run `npx gitnexus analyze`.
5. Before editing any production symbol, run GitNexus impact analysis and report the blast radius.
6. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Add shared client-id normalization for telemetry identity emission.
- Task 2.2: Align shell-side normalization call sites with the shared helper.

Out of scope:
- Packet 3 or 4 work
- non-JSON renderer fallback changes
- new wire-format changes
- weakening validation to accept malformed raw tuple clients

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Land Slice 61 Packet 2 only in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md first. Work only on Task 2.1 and Task 2.2. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. Implement the minimum shared normalization and adjacent test changes needed so runtime-originated public prompt events deserialize cleanly as AgentEvent while strict raw validation remains intact. Reuse or replace the world_gateway-local normalization with the shared helper rather than leaving divergent logic. Keep Packets 3 and 4 out of scope. Run `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`, `cargo test -p substrate-common -- --nocapture`, `cargo test -p shell --test world_gateway -- --nocapture`, and `rg -n "resolve_originating_client|set_pure_agent_telemetry_identity|normalize.*client" crates/common/src crates/shell/src`. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 61 Packet 2 change in /home/spenser/__Active_code/substrate using $code-review-and-quality. Ground the review in /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 61 Packet 2 review findings in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read the review findings plus /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. Fix only the flagged Packet 2 issues without widening scope. Re-run `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`, `cargo test -p substrate-common -- --nocapture`, and `cargo test -p shell --test world_gateway -- --nocapture`. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
  - `cargo test -p substrate-common -- --nocapture`
  - `cargo test -p shell --test world_gateway -- --nocapture`
  - `rg -n "resolve_originating_client|set_pure_agent_telemetry_identity|normalize.*client" crates/common/src crates/shell/src`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit.

Packet 2 checkpoint:
- runtime-originated public prompt events deserialize cleanly as `AgentEvent`
- strict raw validation still rejects malformed tuple clients
- shared normalization logic is not duplicated across separate seams without justification

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 61 Packet 3 only in /home/spenser/__Active_code/substrate.

Use these source docs as authority:
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md

Mission:
- Land Slice 61 Packet 3 only: Harden The Non-JSON Renderer.
- Do not start Packet 4.
- Keep the work bounded to Task 3.1 in TASKS-61.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Spawn a fresh GPT-5.4 subagent on high for implementation.
3. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
4. When implementation completes and local verification is green, commit the implementation changes before any review.
5. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
6. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
7. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
8. After each fix round, rerun the relevant verification, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
9. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-61, PLAN-61, and TASKS-61 first.
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. Inspect the live code in:
   - /home/spenser/__Active_code/substrate/crates/shell/src/execution/agent_runtime/control.rs
   - /home/spenser/__Active_code/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
   - any focused control-unit tests if such a seam already exists
4. If GitNexus says the index is stale, run `npx gitnexus analyze`.
5. Before editing any production symbol, run GitNexus impact analysis and report the blast radius.
6. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Add a human-readable structured-event fallback in `PublicPromptRenderer`.

Out of scope:
- Packet 4 work
- shared identity normalization changes except what is strictly required to integrate the already-landed Packet 2 result
- JSON envelope changes
- summary-line redesign

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Land Slice 61 Packet 3 only in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md first. Work only on Task 3.1. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. Implement the minimum non-JSON renderer hardening needed so structured prompt events never degrade to silent empty output. Keep the preferred path as AgentEvent decode plus format_event_line, keep stderr behavior unchanged, and do not alter the JSON envelope branch. Keep Packet 4 out of scope. Run `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`, plus any focused control test you add. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 61 Packet 3 change in /home/spenser/__Active_code/substrate using $code-review-and-quality. Ground the review in /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 61 Packet 3 review findings in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read the review findings plus /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. Fix only the flagged Packet 3 issues without widening scope. Re-run `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`, plus any focused control test implicated by the fix. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - any added focused control test command
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit.

Packet 3 checkpoint:
- the non-JSON renderer has no silent structured-event path
- JSON rendering remains untouched
- completion summary behavior still matches current human contract

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 61 Packet 4 only in /home/spenser/__Active_code/substrate.

Use these source docs as authority:
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md
- /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md

Mission:
- Land Slice 61 Packet 4 only: Final Validation And Live Repro.
- Keep the work bounded to Task 4.1 and Task 4.2 in TASKS-61.
- Do not reopen Packet 1 through Packet 3 decisions without a concrete failing verification result.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Spawn a fresh GPT-5.4 subagent on high for implementation.
3. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
4. The implementation worker must first run the full Packet 4 validation wall and only make the narrowest follow-up fixes required by failing Packet 4 commands.
5. When implementation or bounded validation-fix work completes and local verification is green, commit the resulting changes before any review.
6. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
7. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
8. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
9. After each fix round, rerun the relevant verification, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
10. Repeat until review-clean.

Commit policy:
- Commit after implementation or bounded validation-fix work before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-61, PLAN-61, and TASKS-61 first.
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. Inspect the Packet 4 validation wall and any files directly implicated by failing commands.
4. If GitNexus says the index is stale, run `npx gitnexus analyze`.
5. If you touch any production Rust symbol while fixing a failing validation command, run GitNexus impact analysis before editing and report the blast radius; otherwise say explicitly that Packet 4 remained validation-only or docs/tests-only.
6. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Run the targeted validation wall.
- Task 4.2: Re-run the live plain-human and JSON repro commands.

Out of scope:
- redesigning earlier packet decisions without a concrete failing verification result
- broad opportunistic cleanup
- any weakening of validation or JSON contract changes

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Land Slice 61 Packet 4 only in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md first. Work only on Task 4.1 and Task 4.2. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any production Rust symbol while fixing a failing validation command, run GitNexus impact analysis before editing and report the blast radius; otherwise say explicitly that Packet 4 remained validation-only or docs/tests-only. Run the full Packet 4 validation wall first. If a failure requires changes, keep the fix bounded strictly to the failing Slice 61 surface and rerun the relevant commands plus any dependent Packet 4 checks. Then rerun the live plain-human and JSON repro commands. Final message must state whether Packet 4 is checkpoint-green, what files or symbols changed, what verification ran, whether bounded follow-up fixes were required, and whether Slice 61 is ready for final closeout.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 61 Packet 4 change in /home/spenser/__Active_code/substrate using $code-review-and-quality. Ground the review in /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 61 Packet 4 review findings in /home/spenser/__Active_code/substrate. Use $incremental-implementation. Re-read the review findings plus /home/spenser/__Active_code/substrate/llm-last-mile/SPEC-61-human-public-agent-start-streaming-regression.md, /home/spenser/__Active_code/substrate/llm-last-mile/PLAN-61-human-public-agent-start-streaming-regression.md, and /home/spenser/__Active_code/substrate/llm-last-mile/TASKS-61.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any production Rust symbol while fixing a flagged issue, run GitNexus impact analysis before editing and report the blast radius; otherwise say explicitly that the fix remained validation-only or docs/tests-only. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- Run the Packet 4 validation wall:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p substrate-common --test agent_hub_event_envelope_schema -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test world_gateway -- --nocapture` if Packet 2 touched gateway normalization
- Re-run the live repro commands:
  - `~/.substrate/bin/substrate agent start --backend cli:codex-host --prompt "Look at the native tools you have available to you within the substrate environment. Not your builtin codex tool_search and other tools, but the tools you have been informed about at runtime."`
  - `~/.substrate/bin/substrate agent start --backend cli:codex-host --prompt "Look at the native tools you have available to you within the substrate environment. Not your builtin codex tool_search and other tools, but the tools you have been informed about at runtime." --json`
- After implementation and after each fix round, run:
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit if any source changes were made.

Packet 4 checkpoint:
- plain human `agent start` visibly streams before completion
- the shared non-JSON renderer path no longer drops structured events silently
- `--json` remains wire-compatible
- strict identity validation still stands

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols if any were touched; otherwise say explicitly that Packet 4 stayed validation-only or docs/tests-only.
- Report GitNexus detect-changes results before each commit if any changes were made.
- State whether Slice 61 is ready for final closeout.
- If anything is not green, say explicitly that Slice 61 must not be closed out.
```
