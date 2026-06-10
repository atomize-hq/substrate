# PROMPTS-52: Packet Orchestration Prompts For Slice 52

Source spec: [SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md](./SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md)  
Source plan: [PLAN-52.md](./PLAN-52.md)  
Source tasks: [TASKS-52.md](./TASKS-52.md)  
Source tracker: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `52` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

## Packet 1 Prompt

```text
/goal Land Slice 52 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md

Mission:
- Land Slice 52 Packet 1 only: Shared Adapter Vocabulary And Handle Freeze.
- Do not start Packet 2.
- Keep the slice bounded to one adapter-visible contract surface, exact tool-name freeze, and exact retained versus active-ephemeral handle families.

Before editing:
1. Read SPEC-52, PLAN-52, TASKS-52, and the REMAINING tracker note, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
2. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
3. Run GitNexus impact analysis before editing any production symbol you change.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add the bounded adapter-visible contract module.
- Task 1.2: Freeze exact follow-up handle families.

Out of scope:
- Packet 2, 3, or 4 work
- runtime-family tool registration in `codex` or `claude_code`
- MCP server landing
- public human toolbox CLI widening
- transport redesign or request-envelope widening
- fuzzy handle targeting or mixed handle semantics

Tracker update requirements:
- If implementation or review surfaces new drift, intentional deferrals, validation follow-ups, or sequencing changes, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record items under the correct tracker section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-52 / PLAN-52 / TASKS-52 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the tool vocabulary is frozen to the seven landed dispatch verbs
- exact `task_run_id` and exact `participant_id` handles are frozen
- the adapter contract stays grounded to the live internal request envelope
- no runtime-family registration logic has been introduced

Implementation subagent prompt:
/goal Land Slice 52 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in one new bounded adapter-contract module under crates/shell/src/execution/agent_runtime/, plus crates/shell/src/execution/agent_runtime/mod.rs and crates/shell/src/execution/agent_runtime/dispatch_contract.rs only if bounded helper wiring is required. Keep Packet 2 through Packet 4 work out of scope, and do not land tool registration, MCP, public CLI widening, or transport redesign. Run cargo test -p shell dispatch_contract -- --nocapture and cargo test -p shell orchestrator_world_dispatch -- --nocapture. Update the tracker note if you surface drift, deferrals, or sequencing changes. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 52 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 52 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 52 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md

Mission:
- Land Slice 52 Packet 2 only: Runtime-Owned Injection And Fresh-Allocation Translation.
- Do not start Packet 3.
- Keep the slice bounded to runtime-owned injected fields, fresh-allocation translation, and authoritative follow-up binding resolution.

Before editing:
1. Read SPEC-52, PLAN-52, TASKS-52, and the REMAINING tracker note, then inspect the live code in:
   - the new adapter-contract module landed by Packet 1
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Translate `run_world_task` into the live internal request contract.
- Task 2.2: Translate `spawn_world_worker` into the live internal request contract.
- Task 2.3: Freeze follow-up injection rules against authoritative runtime state.

Out of scope:
- Packet 3 or 4 work
- runtime-family tool registration in `codex` or `claude_code`
- MCP server landing
- public human toolbox CLI widening
- validator-scope narrowing for `idempotency_key`
- transport redesign or request-envelope widening
- making the model author authoritative backend/world-binding truth

Tracker update requirements:
- If implementation or review surfaces new drift, intentional deferrals, validation follow-ups, or sequencing changes, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record items under the correct tracker section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1, Task 2.2, and Task 2.3.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-52 / PLAN-52 / TASKS-52 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- fresh allocation tools inject runtime-owned authority fields
- live `idempotency_key` validation is satisfied
- follow-up tools do not force the model to reconstruct backend/world-binding truth from prose
- the slice still has not widened into tool registration or transport redesign

Implementation subagent prompt:
/goal Land Slice 52 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 2.1, Task 2.2, and Task 2.3. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in the new adapter-contract module, crates/shell/src/execution/orchestrator_world_dispatch.rs, and crates/shell/src/execution/agent_runtime/state_store.rs only if a bounded authoritative helper is required. Keep Packet 3 and Packet 4 work out of scope, and do not land tool registration, MCP, public CLI widening, validator narrowing, or transport redesign. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell orchestrator_world_dispatch -- --nocapture, and cargo test -p shell state_store -- --nocapture. Update the tracker note if you surface drift, deferrals, or sequencing changes. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 52 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 52 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 52 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md

Mission:
- Land Slice 52 Packet 3 only: Receipt Normalization And Exact Follow-Up Resolution.
- Do not start Packet 4.
- Keep the slice bounded to adapter-visible receipt normalization and exact follow-up handle resolution semantics.

Before editing:
1. Read SPEC-52, PLAN-52, TASKS-52, and the REMAINING tracker note, then inspect the live code in:
   - the new adapter-contract module landed by Packets 1 and 2
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Normalize `run_world_task` and `spawn_world_worker` into adapter-visible receipts.
- Task 3.2: Freeze `inspect_world_worker` and `cancel_world_work` exact dual-handle semantics.
- Task 3.3: Freeze retained-only follow-up semantics for `continue_world_worker` and `stop_world_worker`.

Out of scope:
- Packet 4 work
- runtime-family tool registration in `codex` or `claude_code`
- MCP server landing
- public human toolbox CLI widening
- transport redesign or typed-outcome harmonization beyond adapter normalization
- broader autonomy semantics or runtime-family exposure

Tracker update requirements:
- If implementation or review surfaces new drift, intentional deferrals, validation follow-ups, or sequencing changes, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record items under the correct tracker section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1, Task 3.2, and Task 3.3.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-52 / PLAN-52 / TASKS-52 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- fresh-allocation receipts are normalized for the adapter layer
- dual-handle follow-up tools are exact and fail closed
- retained-only follow-up tools remain retained-only
- the slice still has not landed runtime-family tool registration

Implementation subagent prompt:
/goal Land Slice 52 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 3.1, Task 3.2, and Task 3.3. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in the new adapter-contract module, crates/shell/src/execution/orchestrator_world_dispatch.rs, crates/shell/src/execution/agent_runtime/state_store.rs only if a bounded handle-resolution helper is required, and crates/shell/src/execution/agent_runtime/dispatch_contract.rs only if bounded helper types/tests are required. Keep Packet 4 work out of scope, and do not land tool registration, MCP, public CLI widening, transport redesign, or broader autonomy semantics. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell orchestrator_world_dispatch -- --nocapture, and cargo test -p shell state_store -- --nocapture. Update the tracker note if you surface drift, deferrals, or sequencing changes. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 52 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 52 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State whether Packet 4 is unblocked.
- If anything is not green, say explicitly that Packet 4 must not begin.
```

## Packet 4 Prompt

```text
/goal Land Slice 52 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md

Mission:
- Land Slice 52 Packet 4 only: Repo-Local Docs, Comments, And Final Validation.
- This packet assumes Packets 1 through 3 are already landed and checkpoint-green.
- Keep the slice bounded to repo-local docs/comments alignment and the final validation wall.

Before editing:
1. Read SPEC-52, PLAN-52, TASKS-52, and the REMAINING tracker note, then inspect the live docs and bounded code-comment surfaces in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md
   - the bounded Slice 52 code comments only if they need truth-alignment edits
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Align repo-local docs and bounded code comments with Slice 52.
- Task 4.2: Run the final validation wall.

Out of scope:
- reopening Packets 1 through 3 unless validation proves a bounded in-scope follow-up is required
- runtime-family tool registration in `codex` or `claude_code`
- MCP server landing
- public human toolbox CLI widening
- transport redesign
- implying that runtime-family landing has already happened

Tracker update requirements:
- If implementation or review surfaces new drift, intentional deferrals, validation follow-ups, or sequencing changes, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record items under the correct tracker section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - `cargo test -p shell async_repl -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test --workspace -- --nocapture`
- Also do the required manual diff review for the docs/comment truth-alignment work.
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 docs/validation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-52 / PLAN-52 / TASKS-52 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- docs/comments and runtime truth are aligned
- the validation wall is green
- Slice 52 still reads honestly as a contract-freeze slice

Implementation subagent prompt:
/goal Land Slice 52 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 4.1 and Task 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Align repo-local docs/comments with the frozen Slice 52 contract without implying live tool registration, MCP, or public CLI landing. If a validation failure requires follow-up changes, keep them bounded to explicit Slice 52 surfaces only; if validation proves runtime-family registration, public CLI work, or transport redesign is needed, stop and record that as a follow-on slice instead of widening this one. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell orchestrator_world_dispatch -- --nocapture, cargo test -p shell async_repl -- --nocapture, cargo test -p shell state_store -- --nocapture, and cargo test --workspace -- --nocapture. Also perform the required manual diff review. Update the tracker note if you surface drift, deferrals, or sequencing changes. Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, whether any reopen condition was discovered, and whether Slice 52 is ready for final closeout.

Review subagent prompt:
Review the committed Slice 52 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 52 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-52-internal-runtime-owned-host-orchestrator-tool-adapter-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-52.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-52.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands and redo the manual diff review if docs/comments changed. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State whether Slice 52 is fully review-clean.
- If anything is not green, say explicitly that Slice 52 is not ready to close out.
```
