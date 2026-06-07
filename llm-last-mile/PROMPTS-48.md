# PROMPTS-48: Packet Orchestration Prompts For Slice 48

Source spec: [SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md](./SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md)  
Source plan: [PLAN-48.md](./PLAN-48.md)  
Source tasks: [TASKS-48.md](./TASKS-48.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `48` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

## Packet 1 Prompt

```text
/goal Land Slice 48 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md

Mission:
- Land Packet 1 only: Policy And Attach-Semantic Contract Freeze.
- Do not start Packet 2.
- Keep the slice bounded to making router-owned local session auto-attach explicitly deny-by-default and freezing the current local attach-state semantics without schema rename churn.

Before editing:
1. Read SPEC-48, PLAN-48, TASKS-48, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/policy.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/broker/src/effective_policy.rs
2. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
3. Run GitNexus impact analysis before editing any production symbol you change.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Enforce the router-owned auto-attach policy gate.
- Task 1.2: Freeze current local attach-state semantics without renaming the artifact.

Out of scope:
- Packet 2, 3, or 4 work
- adding a production-owned router execution path
- host-global inbox or remote ingress work
- review-state resolution changes
- worker continuation, approval, clarification, or fork execution from the router path
- cross-host targeting fields
- public router or daemon UX

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell auto_attach -- --nocapture`
  - `cargo test -p shell policy_model -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p substrate-broker -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-48 / PLAN-48 / TASKS-48 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- router-owned auto-attach is deny-by-default
- `workflow.router.enabled` is required
- the relevant existing world-dispatch obligation/fork gates are the explicit per-kind allow/deny inputs
- current local attach-state semantics are frozen without rename churn
- missing or disallowed policy truth fails closed before attach execution begins

Implementation subagent prompt:
/goal Land Slice 48 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/auto_attach.rs, crates/shell/src/execution/agent_runtime/obligation_ledger.rs, crates/shell/src/execution/agent_runtime/state_store.rs, crates/shell/src/execution/policy_model.rs, crates/broker/src/policy.rs, and crates/broker/src/effective_policy.rs. Keep Packet 2 through Packet 4 work out of scope. Run cargo test -p shell auto_attach -- --nocapture, cargo test -p shell policy_model -- --nocapture, cargo test -p shell state_store -- --nocapture, and cargo test -p substrate-broker -- --nocapture. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 48 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 48 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 48 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md

Mission:
- Land Packet 2 only: Production-Owned Router Execution Path.
- Do not start Packet 3.
- Keep the slice bounded to adding the first real non-test host-side router-owned execution path and reusing the existing local auto-attach executor through that path.

Before editing:
1. Read SPEC-48, PLAN-48, TASKS-48, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Introduce the host-side router-owned execution entrypoint.
- Task 2.2: Keep attach mode selection and boundary checks exact on the new path.

Out of scope:
- Packet 3 or 4 work
- broader workflow engine or reusable daemon/service work
- public router or daemon CLI management
- world-service-owned attach logic
- host-global inbox or remote ingress work
- review-state resolution changes
- worker continuation, approval, clarification, or fork execution from the router path

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell auto_attach -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-48 / PLAN-48 / TASKS-48 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- a real host-side non-test call site exists
- router-owned execution stays local-only and exact-session scoped
- continuity-first and fail-closed fresh attach semantics remain intact
- no world-service-owned or public CLI-owned attach execution path is introduced

Implementation subagent prompt:
/goal Land Slice 48 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md first. Work only on Task 2.1 and Task 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/auto_attach.rs, crates/shell/src/execution/agent_runtime/state_store.rs, and the narrowest host-side execution file under crates/shell/src/execution/ required to add a production-owned router execution path. Keep Packet 3 and Packet 4 work out of scope. Run cargo test -p shell auto_attach -- --nocapture and cargo test -p shell state_store -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 48 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 48 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 48 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md

Mission:
- Land Packet 3 only: Settlement, Manual Reattach Convergence, And Observability.
- Do not start Packet 4.
- Keep the slice bounded to proving one attach episode per orchestration session, converging manual `reattach` with router-owned attach, and making router-owned outcomes explanation-ready.

Before editing:
1. Read SPEC-48, PLAN-48, TASKS-48, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - the host-side router execution file added for Packet 2
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Prove session-scoped settlement and manual-reattach convergence.
- Task 3.2: Emit explanation-ready router-owned outcomes without widening the slice.

Out of scope:
- Packet 4 work
- broader workflow router engine work
- host-global inbox or remote ingress work
- public router daemon lifecycle UX
- review-state resolution changes
- worker continuation, approval, clarification, or fork execution from the router path
- cross-host targeting fields

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell auto_attach -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-48 / PLAN-48 / TASKS-48 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- one attach episode per orchestration session is proven
- manual `reattach` and router-owned attach converge without duplicate launch
- success does not resolve review state
- router-owned outcomes are explanation-ready and bounded

Implementation subagent prompt:
/goal Land Slice 48 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/auto_attach.rs, crates/shell/src/execution/agent_runtime/state_store.rs, the host-side router execution file added in Packet 2, and docs/TRACE.md only if a real trace publication lands. Keep Packet 4 work out of scope. Run cargo test -p shell state_store -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, and cargo test -p shell auto_attach -- --nocapture. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 48 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 48 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 48 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md

Mission:
- Land Slice 48 Packet 4 only: Docs Alignment And Final Validation.
- This packet assumes Packets 1 through 3 are already landed and checkpoint-green.
- Keep the slice bounded to docs/policy truth and the final validation wall.

Before editing:
1. Read SPEC-48, PLAN-48, TASKS-48, then inspect the live docs and config surfaces in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/reference/policy/contract.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Align policy and orchestration docs with the landed Slice 48 boundary.
- Task 4.2: Run the final validation wall.

Out of scope:
- reopening Packets 1 through 3 unless validation proves a bounded in-scope follow-up is required
- broader workflow engine or public router daemon work
- host-global inbox or remote ingress work
- review-state resolution changes
- worker continuation, approval, clarification, or fork execution from the router path
- broader host-targeting identity envelope work

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell auto_attach -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell policy_model -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p substrate-broker -- --nocapture`
  - `cargo test --workspace -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 docs/validation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-48 / PLAN-48 / TASKS-48 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- the Slice 48 boundary is safely bounded
- docs and policy truth are honest
- the validation wall is green

Implementation subagent prompt:
/goal Land Slice 48 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md first. Work only on Task 4.1 and Task 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Align docs and policy truth without widening scope, then run the full Packet 4 validation wall. If a validation failure requires more code or doc changes, keep any follow-up bounded to explicit Slice 48 surfaces only. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test -p shell auto_attach -- --nocapture, cargo test -p shell state_store -- --nocapture, cargo test -p shell policy_model -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, cargo test -p substrate-broker -- --nocapture, and cargo test --workspace -- --nocapture. Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, whether any reopen condition was discovered, and whether Slice 48 is ready for final closeout.

Review subagent prompt:
Review the committed Slice 48 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 48 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-48-internal-family-2-router-owned-session-auto-attach-execution-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-48.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-48.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 48 is fully review-clean.
- If anything is not green, say explicitly that Slice 48 is not ready to close out.
```
