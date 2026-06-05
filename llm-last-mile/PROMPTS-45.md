# PROMPTS-45: Packet Orchestration Prompts For Slice 45

Source spec: [SPEC-45-internal-retained-host-fork-command-bootstrap.md](./SPEC-45-internal-retained-host-fork-command-bootstrap.md)  
Source plan: [PLAN-45.md](./PLAN-45.md)  
Source tasks: [TASKS-45.md](./TASKS-45.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `45` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

## Packet 1 Prompt

```text
/goal Land Slice 45 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md

Mission:
- Land Packet 1 only: Typed Fork Command Contract And Policy Surface.
- Do not start Packet 2.
- Keep the slice bounded to adding the typed host `fork_command` payload on `continue_world_worker` and the dedicated deny-by-default host fork-command policy gate.

Before editing:
1. Read SPEC-45, PLAN-45, TASKS-45, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/policy.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/effective_policy.rs
2. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
3. Run GitNexus impact analysis before editing any production symbol you change.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add the typed host `fork_command` payload contract.
- Task 1.2: Add the dedicated deny-by-default host fork-command policy gate.

Out of scope:
- Packet 2, 3, or 4 work
- deterministic rendering changes beyond what is strictly required to admit the Packet 1 payload shape
- routed child allocation reuse
- `progress_ack`
- worker auto-fork
- public fork-command UX or CLI changes
- active-ephemeral identity work
- transport schema widening in `transport-api-types` or `world-service`
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell policy_model -- --nocapture`
  - `cargo test -p substrate-broker -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-45 / PLAN-45 / TASKS-45 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- typed host `fork_command` is a valid `continue_world_worker` payload
- the new host fork-command gate is deny-by-default
- explanation-ready denials exist for disallowed typed host fork commands
- optional `progress_ack` and broader autonomy remain deferred

Implementation subagent prompt:
/goal Land Slice 45 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/dispatch_contract.rs, crates/shell/src/execution/policy_model.rs, crates/broker/src/policy.rs, and crates/broker/src/effective_policy.rs. Keep Packet 2 through Packet 4 work out of scope. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell policy_model -- --nocapture, and cargo test -p substrate-broker -- --nocapture. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 45 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 45 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 45 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md

Mission:
- Land Packet 2 only: Deterministic Rendering And Exact-Source Bootstrap Preparation.
- Do not start Packet 3.
- Keep the slice bounded to deterministic typed host `fork_command` rendering and exact-source bootstrap preparation without child-allocation side effects.

Before editing:
1. Read SPEC-45, PLAN-45, TASKS-45, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Render typed host `fork_command` deterministically over the retained member-turn seam.
- Task 2.2: Prepare exact-source fork bootstrap reuse without child side effects.

Out of scope:
- Packet 3 or 4 work
- new child allocation behavior
- second child-allocation path
- public fork-command UX or CLI changes
- `progress_ack`
- worker auto-fork
- active-ephemeral identity work
- transport schema widening in `transport-api-types` or `world-service`
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-45 / PLAN-45 / TASKS-45 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- accepted typed host fork commands render deterministically
- exact retained source-worker targeting remains mandatory
- invalid source or boundary drift fails closed before any child side effect

Implementation subagent prompt:
/goal Land Slice 45 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md first. Work only on Task 2.1 and Task 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/dispatch_contract.rs, crates/shell/src/execution/orchestrator_world_dispatch.rs, and crates/shell/src/execution/agent_runtime/state_store.rs. Preserve Packet 1 behavior and keep child allocation side effects out of scope. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture, and cargo test -p shell state_store -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 45 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 45 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 45 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md

Mission:
- Land Packet 3 only: Routed Allocation Reuse And Outcome Honesty.
- Do not start Packet 4.
- Keep the slice bounded to routing accepted typed host `fork_command` through the existing fork bootstrap path, preserving explicit lineage, and keeping delivery-versus-allocation outcomes honest.

Before editing:
1. Read SPEC-45, PLAN-45, TASKS-45, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Reuse the landed fork bootstrap path for accepted typed host fork commands.
- Task 3.2: Preserve honest summaries and fail-closed rollback behavior.

Out of scope:
- Packet 4 work
- any second child-allocation path
- broad transport redesign
- public fork-command UX or CLI changes
- `progress_ack`
- worker auto-fork
- active-ephemeral identity work
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-45 / PLAN-45 / TASKS-45 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- allowed typed host fork commands allocate one retained child through the existing fork path
- source-to-child lineage remains explicit
- summaries remain honest about delivery versus allocation
- rollback and denial paths fail closed without widening into worker auto-fork, optional `progress_ack`, or Family 2 work

Implementation subagent prompt:
/goal Land Slice 45 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/orchestrator_world_dispatch.rs and directly adjacent shell tests. Reuse the existing fork bootstrap path, preserve explicit lineage, and keep delivery versus allocation summaries truthful. Run cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture and cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 45 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 45 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 45 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md

Mission:
- Land Packet 4 only: Docs Alignment And Final Validation.
- Do not widen beyond the Packet 4 surface.
- Keep the slice bounded to aligning config/docs truth and running the final validation wall for already-landed Slice 45 behavior.

Before editing:
1. Read SPEC-45, PLAN-45, TASKS-45, then inspect the live docs and config in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Align planning and config truth without widening the slice.
- Task 4.2: Run the final validation wall.

Out of scope:
- reopening earlier packets except for concrete validation-proven Slice 45 fixes
- open-ended cleanup
- public fork-command UX or CLI changes
- `progress_ack`
- worker auto-fork
- active-ephemeral identity work
- transport redesign
- Family 2 work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell policy_model -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p substrate-broker -- --nocapture`
  - `cargo test --workspace -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 implementation work before review.
- Treat validation as a gate, not as an implicit cleanup bucket. If validation fails, fix only the concrete failing Slice 45 files proven necessary by the failing commands.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-45 / PLAN-45 / TASKS-45 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- the Slice 45 surface is safely bounded
- config/docs truth is honest
- the validation wall is green

Implementation subagent prompt:
/goal Land Slice 45 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md first. Work only on Task 4.1 and Task 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Start by aligning docs/CONFIGURATION.md and the Slice 45 planning docs to already-landed Packet 1 through Packet 3 truth without widening scope. Then run the full Packet 4 validation wall. If validation fails, fix only the concrete failing Slice 45 files proven necessary by the failing commands; do not treat validation as open-ended cleanup. Final message must state whether Packet 4 is checkpoint-green, what files changed, what verification ran, whether bounded follow-up fixes were required, and whether Slice 45 is ready for final closeout.

Review subagent prompt:
Review the committed Slice 45 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 45 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-45-internal-retained-host-fork-command-bootstrap.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-45.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-45.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 45 is ready for final closeout.
- If anything is not green, say explicitly that Slice 45 is not ready for final closeout.
```
