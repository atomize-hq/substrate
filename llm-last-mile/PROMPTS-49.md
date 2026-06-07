# PROMPTS-49: Packet Orchestration Prompts For Slice 49

Source spec: [SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md](./SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md)  
Source plan: [PLAN-49.md](./PLAN-49.md)  
Source tasks: [TASKS-49.md](./TASKS-49.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `49` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

## Packet 1 Prompt

```text
/goal Land Slice 49 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md

Mission:
- Land Packet 1 only: Host-Targeting Envelope Contract Freeze.
- Do not start Packet 2.
- Keep the slice bounded to widening the canonical obligation record only for bounded host-targeting truth and freezing the wrong-host semantics for the current local-only architecture.

Before editing:
1. Read SPEC-49, PLAN-49, TASKS-49, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - the narrowest existing helper seam under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/ or /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/common/ if one already fits the wrong-host classification work
2. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
3. Run GitNexus impact analysis before editing any production symbol you change.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add the bounded host-targeting fields to the canonical obligation artifact.
- Task 1.2: Freeze wrong-host semantics for the current local-only architecture.

Out of scope:
- Packet 2, 3, or 4 work
- producer widening in orchestrator_world_dispatch.rs
- router attach-path behavior changes in auto_attach.rs
- `SUBSTRATE_HOME/host_inbox/`
- remote ingress materialization or remote federation
- `ingress_source_kind`, `ingress_source_id`, or `ingress_received_at`
- broader `causation_*` envelope widening
- new public CLI, policy, or config surfaces
- workflow-engine or router lifecycle productization

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell obligation -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
- If implementation adds a dedicated helper with direct tests, run those too.
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-49 / PLAN-49 / TASKS-49 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the bounded host-targeting envelope is frozen
- backward compatibility for missing host-targeting metadata is preserved
- wrong-host semantics are explicit
- ingress-ready identity fields remain deferred

Implementation subagent prompt:
/goal Land Slice 49 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/obligation_ledger.rs, crates/shell/src/execution/agent_runtime/state_store.rs, and the narrowest helper seam under crates/shell/src/execution/ or crates/common/ only if strictly required for explicit wrong-host classification semantics. Keep Packet 2 through Packet 4 work out of scope, and do not land producer, router, or `host_inbox` work. Run cargo test -p shell obligation -- --nocapture and cargo test -p shell state_store -- --nocapture, plus any directly adjacent helper tests you add. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 49 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 49 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 49 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md

Mission:
- Land Packet 2 only: Local Producer And Persistence Widening.
- Do not start Packet 3.
- Keep the slice bounded to propagating bounded local host-targeting truth into local obligation producers and preserving authoritative backward-compatible persistence.

Before editing:
1. Read SPEC-49, PLAN-49, TASKS-49, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Stamp bounded local host-targeting truth on locally produced obligations where the runtime knows it.
- Task 2.2: Keep persistence and reload behavior compatible for untargeted legacy/local obligations.

Out of scope:
- Packet 3 or 4 work
- router attach-path logic in auto_attach.rs
- `SUBSTRATE_HOME/host_inbox/`
- remote ingress materialization or remote federation
- `ingress_source_kind`, `ingress_source_id`, or `ingress_received_at`
- broader `causation_*` envelope widening
- new public CLI, policy, or config surfaces
- workflow-engine or router lifecycle productization

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell obligation -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-49 / PLAN-49 / TASKS-49 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- local producers can emit bounded host-targeting truth
- persistence remains authoritative and backward compatible
- no ingress metadata or host-global state path is introduced

Implementation subagent prompt:
/goal Land Slice 49 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md first. Work only on Task 2.1 and Task 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/orchestrator_world_dispatch.rs, crates/shell/src/execution/agent_runtime/state_store.rs, and crates/shell/src/execution/agent_runtime/obligation_ledger.rs. Keep Packet 3 and Packet 4 work out of scope, and do not land router, `host_inbox`, or ingress metadata work. Run cargo test -p shell orchestrator_world_dispatch -- --nocapture, cargo test -p shell state_store -- --nocapture, and cargo test -p shell obligation -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 49 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 49 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 49 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md

Mission:
- Land Packet 3 only: Router Wrong-Host Fail-Closed Evaluation.
- Do not start Packet 4.
- Keep the slice bounded to adding the exact local-host check on the router-owned attach path and settling wrong-host obligations fail closed without mutating review state.

Before editing:
1. Read SPEC-49, PLAN-49, TASKS-49, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - the bounded local-host helper introduced earlier if one exists
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Introduce an exact local-host check on the router-owned attach path.
- Task 3.2: Settle wrong-host obligations fail closed without mutating review state.

Out of scope:
- Packet 4 work
- `SUBSTRATE_HOME/host_inbox/`
- remote ingress materialization or remote federation
- broader ingress metadata or `causation_*` widening
- new public CLI, policy, or config surfaces
- workflow-engine or router lifecycle productization
- producer work beyond the narrowest updates required for Packet 3 tests or outcome logging

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell auto_attach -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
- If implementation adds a dedicated local-host helper with direct tests, run those too.
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-49 / PLAN-49 / TASKS-49 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- wrong-host obligations cannot launch attach
- wrong-host reasons are explanation-ready
- review-state semantics are unchanged
- same-host and untargeted obligations preserve Slice 48 behavior

Implementation subagent prompt:
/goal Land Slice 49 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/auto_attach.rs, crates/shell/src/execution/agent_runtime/state_store.rs, and crates/shell/src/execution/orchestrator_world_dispatch.rs only if Packet 3 outcome logging or tests require it, plus the bounded local-host helper if that seam exists. Keep Packet 4 work out of scope, and do not land `host_inbox`, ingress, or broader federation behavior. Run cargo test -p shell auto_attach -- --nocapture, cargo test -p shell state_store -- --nocapture, and cargo test -p shell orchestrator_world_dispatch -- --nocapture, plus any directly adjacent helper tests you add. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 49 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 49 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 49 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md

Mission:
- Land Slice 49 Packet 4 only: Docs Alignment And Final Validation.
- This packet assumes Packets 1 through 3 are already landed and checkpoint-green.
- Keep the slice bounded to docs/runtime truth and the final validation wall.

Before editing:
1. Read SPEC-49, PLAN-49, TASKS-49, then inspect the live docs and config surfaces in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Align docs with the landed Slice 49 boundary.
- Task 4.2: Run the final validation wall.

Out of scope:
- reopening Packets 1 through 3 unless validation proves a bounded in-scope follow-up is required
- `SUBSTRATE_HOME/host_inbox/`
- remote ingress materialization or remote federation
- broader ingress metadata or `causation_*` widening
- new public CLI, policy, or config surfaces
- workflow-engine or router lifecycle productization

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell auto_attach -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell obligation -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture`
  - `cargo test --workspace -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 docs/validation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-49 / PLAN-49 / TASKS-49 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- the Slice 49 boundary is safely bounded
- docs and runtime truth are honest
- the validation wall is green

Implementation subagent prompt:
/goal Land Slice 49 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md first. Work only on Task 4.1 and Task 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Align docs and runtime truth without widening scope, then run the full Packet 4 validation wall. If a validation failure requires more code or doc changes, keep any follow-up bounded to explicit Slice 49 surfaces only. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test -p shell auto_attach -- --nocapture, cargo test -p shell state_store -- --nocapture, cargo test -p shell obligation -- --nocapture, cargo test -p shell orchestrator_world_dispatch -- --nocapture, and cargo test --workspace -- --nocapture. Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, whether any reopen condition was discovered, and whether Slice 49 is ready for final closeout.

Review subagent prompt:
Review the committed Slice 49 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 49 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-49-internal-family-2-host-targeted-obligation-envelope-and-wrong-host-fail-closed-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-49.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-49.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 49 is fully review-clean.
- If anything is not green, say explicitly that Slice 49 is not ready to close out.
```
