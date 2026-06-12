# PROMPTS-57: Packet Orchestration Prompts For Slice 57

Source spec: [SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md](./SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md)  
Source plan: [PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md](./PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md)  
Source tasks: [TASKS-57.md](./TASKS-57.md)  
Source tracker: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Source gap matrix: [AGENT_ORCHESTRATION_GAP_MATRIX.md](../AGENT_ORCHESTRATION_GAP_MATRIX.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/home/azureuser/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/home/azureuser/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `57` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

Packet mapping for Slice `57`:

1. **Packet 1** = Tasks `1.1` and `1.2`
2. **Packet 2** = Tasks `2.1` and `2.2`
3. **Packet 3** = Tasks `3.1`, `3.2`, and `3.3`
4. **Packet 4** = Tasks `4.1` and `4.2`

## Packet 1 Prompt

```text
/goal Land Slice 57 Packet 1 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md

Mission:
- Land Slice 57 Packet 1 only: Coordination Contract Freeze.
- Do not start Packet 2.
- Keep the work bounded to Tasks 1.1 and 1.2 only.

Before doing any implementation work:
1. Re-read SPEC-57, PLAN-57, TASKS-57, the REMAINING tracker note, and AGENT_ORCHESTRATION_GAP_MATRIX.md.
2. Confirm the packet mapping from TASKS-57:
   - Packet 1 = Tasks 1.1 and 1.2 only.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. If you end up editing any production Rust symbol, run GitNexus impact analysis before editing it and report the blast radius. If Packet 1 stays docs-only, say explicitly that no production-symbol impact analysis was required.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Freeze the bounded source-scoped coordination artifact.
- Task 1.2: Freeze the layering and non-goal contract.

Out of scope:
- Packet 2, 3, or 4 work
- any runtime coordination artifact implementation
- any state-store persistence helpers
- any ingress apply sequencing
- any replay handling implementation
- any host-inbox materialization or router behavior changes
- any cross-host delivery, lease/lock coordination, federation, or public operator UX widening

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Tasks 1.1 and 1.2.
- After implementation, run:
  - `rg -n "cursor|sync-state|host_inbox|federation|lease|lock|public|router" /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-57 / PLAN-57 / TASKS-57 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 subagent on high to fix them.
- The fix subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- one exact coordination artifact is frozen
- one exact cursor-advance rule is frozen
- the slice is still clearly narrower than delivery/federation work

Implementation subagent prompt:
/goal Land Slice 57 Packet 1 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md first. Work only on Tasks 1.1 and 1.2. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 1 stayed docs-only. Implement only the minimum doc changes needed to freeze the source-scoped coordination artifact, the cursor-advance rule, and the non-goal/layering contract. Do not start Packet 2. Run the Packet 1 verification grep plus `git diff --stat` and `git status --short`. Final message must state whether Packet 1 is checkpoint-green, what files changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 57 Packet 1 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md, and the live diff. Review only Packet 1. Review across correctness, readability, architecture, security, and performance as they apply to a contract/doc slice. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 57 Packet 1 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Fix only the flagged Packet 1 issues without widening scope. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained docs-only. Re-run the Packet 1 verification grep plus `git diff --stat` and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols, or explicitly say Packet 1 stayed docs-only.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 57 Packet 2 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/host_inbox.rs
- /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs

Mission:
- Land Slice 57 Packet 2 only: Durable Coordination Artifacts And Validation.
- Do not start Packet 3.
- Keep the work bounded to Tasks 2.1 and 2.2 only.

Before editing:
1. Re-read SPEC-57, PLAN-57, TASKS-57, then inspect the live code in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/host_inbox.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - the narrowest bounded coordination module location under /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Add the bounded host-inbox coordination artifact model.
- Task 2.2: Extend the state store with coordination persistence helpers.

Out of scope:
- Packet 3 or 4 work
- ingress apply sequencing
- same-cursor replay/idempotence behavior
- non-monotonic replay fail-closed logic
- any host-side ingress helper
- any host-inbox materialization or router behavior changes
- any cross-host delivery, lease/lock coordination, federation, or public operator UX widening

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Tasks 2.1 and 2.2.
- After implementation, run:
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-57 / PLAN-57 / TASKS-57 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 subagent on high to fix them.
- The fix subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- durable coordination artifacts exist
- validation is exact and fail-closed
- coordination persistence is store-owned

Implementation subagent prompt:
/goal Land Slice 57 Packet 2 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md first. Work only on Tasks 2.1 and 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/host_inbox.rs, crates/shell/src/execution/agent_runtime/state_store.rs, one new bounded coordination module under crates/shell/src/execution/agent_runtime/, and crates/shell/src/execution/agent_runtime/mod.rs only if export wiring is required. Keep Packet 3 and Packet 4 work out of scope, and do not land ingress apply sequencing, replay handling, host-side ingress helpers, materialization/router changes, or broader federation/public UX work. Run `cargo test -p shell host_inbox -- --nocapture` and `cargo test -p shell state_store -- --nocapture`, then `git diff --stat` and `git status --short`. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 57 Packet 2 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 57 Packet 2 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run `cargo test -p shell host_inbox -- --nocapture`, `cargo test -p shell state_store -- --nocapture`, plus `git diff --stat` and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 57 Packet 3 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/host_inbox.rs
- /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs

Mission:
- Land Slice 57 Packet 3 only: Exact Ingress Apply Sequencing And Replay Truth.
- Do not start Packet 4.
- Keep the work bounded to Tasks 3.1, 3.2, and 3.3 only.

Before editing:
1. Re-read SPEC-57, PLAN-57, TASKS-57, then inspect the live code in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - the bounded coordination module landed by Packet 2
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/host_inbox.rs
   - the narrowest host-side seam under /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/ if an ingress apply helper is required
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Apply a valid new ingress cursor into exactly one canonical host-inbox record.
- Task 3.2: Make same-cursor replay idempotent.
- Task 3.3: Fail closed on non-monotonic or mismatched replay.

Out of scope:
- Packet 4 work
- downstream host-inbox materialization or router coexistence proof except what is strictly necessary to keep Packet 3 coherent
- broader host_inbox -> obligation sequencing redesign
- cross-host delivery, lease/lock coordination, federation, or public operator UX widening

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Tasks 3.1, 3.2, and 3.3.
- After implementation, run:
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-57 / PLAN-57 / TASKS-57 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 subagent on high to fix them.
- The fix subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- new ingress applies exactly once
- exact replay is idempotent
- invalid replay fails closed
- cursor advancement never outruns durable local receipt

Implementation subagent prompt:
/goal Land Slice 57 Packet 3 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md first. Work only on Tasks 3.1, 3.2, and 3.3. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/state_store.rs, the bounded coordination module, crates/shell/src/execution/agent_runtime/host_inbox.rs, and one bounded host-side ingress helper under crates/shell/src/execution/ only if needed. Keep Packet 4 and any broader materialization/router/federation/public UX work out of scope. Run `cargo test -p shell host_inbox -- --nocapture` and `cargo test -p shell state_store -- --nocapture`, then `git diff --stat` and `git status --short`. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 57 Packet 3 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 57 Packet 3 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run `cargo test -p shell host_inbox -- --nocapture`, `cargo test -p shell state_store -- --nocapture`, plus `git diff --stat` and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 57 Packet 4 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/host_inbox_materialization.rs
- /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs

Mission:
- Land Slice 57 Packet 4 only: Coexistence Proof, Docs Alignment, And Validation Wall.
- This is the final packet for Slice 57.
- Keep the work bounded to Tasks 4.1 and 4.2 only.

Before editing:
1. Re-read SPEC-57, PLAN-57, TASKS-57, then inspect the live code/docs in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/host_inbox_materialization.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Prove coordination-layer coexistence with the landed materialization/router path.
- Task 4.2: Align docs and run the final validation wall.

Out of scope:
- any new coordination artifact semantics beyond what earlier packets already landed
- any cross-host delivery, lease/lock coordination, federation, or public operator UX widening
- any unrelated cleanup outside the bounded Slice 57 file set

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Tasks 4.1 and 4.2.
- After implementation, run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell host_inbox_materialization -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if Packet 4 touched that seam
  - `cargo test --workspace -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-57 / PLAN-57 / TASKS-57 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 subagent on high to fix them.
- The fix subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- the coordination seam stays safely bounded
- downstream layering is still honest
- the validation wall is green

Implementation subagent prompt:
/goal Land Slice 57 Packet 4 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md, /home/azureuser/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md, /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/host_inbox_materialization.rs, and /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs first. Work only on Tasks 4.1 and 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement only the minimum code/docs/test changes needed to prove coexistence, align docs, and close the validation wall. Do not widen scope into new delivery/federation/public UX work. Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p shell host_inbox -- --nocapture`, `cargo test -p shell state_store -- --nocapture`, `cargo test -p shell host_inbox_materialization -- --nocapture`, `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if you touched that seam, `cargo test --workspace -- --nocapture`, then `git diff --stat` and `git status --short`. Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, whether Slice 57 is closed, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 57 Packet 4 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 57 Packet 4 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-57-internal-family-2-host-global-ingress-receive-cursor-and-sync-state-coordination.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-57.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands plus `git diff --stat` and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 57 is closed.
- If anything is not green, say explicitly that Slice 57 is not closed.
```
