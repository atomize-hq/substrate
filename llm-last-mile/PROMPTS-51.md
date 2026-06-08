# PROMPTS-51: Packet Orchestration Prompts For Slice 51

Source spec: [SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md](./SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md)  
Source plan: [PLAN-51.md](./PLAN-51.md)  
Source tasks: [TASKS-51.md](./TASKS-51.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `51` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

## Packet 1 Prompt

```text
/goal Land Slice 51 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md

Mission:
- Land Slice 51 Packet 1 only: Host-Inbox Contract Freeze.
- Do not start Packet 2.
- Keep the slice bounded to the host-inbox artifact contract, minimum materialization-state vocabulary, and state-store-owned `SUBSTRATE_HOME/host_inbox/` path/persistence helpers.

Before editing:
1. Read SPEC-51, PLAN-51, TASKS-51, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/mod.rs
   - the narrowest existing seam under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/ where a bounded host-inbox module should live
2. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
3. Run GitNexus impact analysis before editing any production symbol you change.
4. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add the bounded host-inbox artifact model under `agent_runtime`.
- Task 1.2: Extend the state store with host-inbox path and persistence helpers.

Out of scope:
- Packet 2, 3, or 4 work
- local obligation materialization logic
- router sequencing or coexistence changes
- remote sync protocols
- receive-cursor or sync-state coordination
- lease or lock coordination for cross-host delivery
- public host-inbox CLI, daemon UX, review surface, or new public policy/config surfaces
- any change that lets `host_inbox` replace the local obligation ledger as canonical local truth

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
- If implementation adds a narrow export wiring seam with direct tests, run those too.
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-51 / PLAN-51 / TASKS-51 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the host-inbox artifact contract is frozen
- host-inbox persistence paths are store-owned
- host-inbox is explicitly not the canonical local obligation ledger

Implementation subagent prompt:
/goal Land Slice 51 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/state_store.rs, one new bounded host-inbox module under crates/shell/src/execution/agent_runtime/, and crates/shell/src/execution/agent_runtime/mod.rs only if export wiring is required. Keep Packet 2 through Packet 4 work out of scope, and do not land local obligation materialization, router sequencing, remote sync, federation delivery, or public host-inbox UX/policy work. Run cargo test -p shell host_inbox -- --nocapture and cargo test -p shell state_store -- --nocapture, plus any directly adjacent tests you add. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 51 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 51 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 51 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md

Mission:
- Land Slice 51 Packet 2 only: Host-Inbox Persistence And Local Materialization.
- Do not start Packet 3.
- Keep the slice bounded to persisting host-inbox records, materializing valid records into exactly one canonical local obligation, enforcing idempotence, and failing closed on invalid local-boundary truth.

Before editing:
1. Read SPEC-51, PLAN-51, TASKS-51, then inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/obligation_ledger.rs
   - the new bounded host-inbox module landed by Packet 1
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Materialize a valid host-inbox record into one local obligation.
- Task 2.2: Make host-inbox materialization idempotent.
- Task 2.3: Fail closed on wrong-host or invalid local-boundary truth.

Out of scope:
- Packet 3 or 4 work
- host-side execution entrypoint wiring
- router sequencing changes except for narrow coexistence-safe helpers required by materialization
- remote sync protocols
- receive-cursor or sync-state coordination
- lease or lock coordination for cross-host delivery
- public host-inbox CLI, daemon UX, review surface, or new public policy/config surfaces
- any change that lets the router consume host-inbox records directly

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1, Task 2.2, and Task 2.3.
- After implementation, run the Packet 2 verification commands:
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell obligation -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-51 / PLAN-51 / TASKS-51 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- valid records materialize exactly one local obligation
- repeated materialization is idempotent
- invalid local-boundary cases fail closed
- no remote sync or federation behavior is introduced

Implementation subagent prompt:
/goal Land Slice 51 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md first. Work only on Task 2.1, Task 2.2, and Task 2.3. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in the new host-inbox module, crates/shell/src/execution/agent_runtime/state_store.rs, and crates/shell/src/execution/agent_runtime/obligation_ledger.rs only if a bounded materialization helper is required. Keep Packet 3 and Packet 4 work out of scope, and do not land host-side execution entrypoints, router direct host-inbox consumption, remote sync, federation delivery, or public host-inbox UX/policy work. Run cargo test -p shell host_inbox -- --nocapture, cargo test -p shell state_store -- --nocapture, and cargo test -p shell obligation -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 51 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 51 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 51 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md

Mission:
- Land Slice 51 Packet 3 only: Host-Side Execution Boundary And Router Coexistence.
- Do not start Packet 4.
- Keep the slice bounded to one internal host-side materialization entrypoint, proof that existing router-owned attach still consumes obligations only, and explanation-ready host-inbox materialization outcomes.

Before editing:
1. Read SPEC-51, PLAN-51, TASKS-51, then inspect the live code in:
   - the new bounded host-inbox module landed by Packets 1 and 2
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/auto_attach.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/orchestrator_world_dispatch.rs
   - the narrowest host-side execution file under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/ if the entrypoint belongs outside orchestrator_world_dispatch.rs
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Add one bounded internal host-side host-inbox materialization entrypoint.
- Task 3.2: Prove `host_inbox -> local obligation -> existing router` coexistence.
- Task 3.3: Emit explanation-ready host-inbox materialization outcomes without widening the slice.

Out of scope:
- Packet 4 work
- remote sync protocols
- receive-cursor or sync-state coordination
- lease or lock coordination for cross-host delivery
- public host-inbox CLI, daemon UX, review surface, or new public policy/config surfaces
- any change that lets the router consume host-inbox records directly
- broader workflow-engine, federation, or public trace UX productization

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1, Task 3.2, and Task 3.3.
- After implementation, run the Packet 3 verification commands:
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell auto_attach -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if that seam owns the entrypoint or coexistence proof
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-51 / PLAN-51 / TASKS-51 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- one internal host-side materialization entrypoint exists
- router-owned auto-attach still consumes obligations only
- host-inbox materialization outcomes are explanation-ready
- no public daemon UX or federation workflow has been introduced

Implementation subagent prompt:
/goal Land Slice 51 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md first. Work only on Task 3.1, Task 3.2, and Task 3.3. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in the new host-inbox module, crates/shell/src/execution/agent_runtime/state_store.rs, and the narrowest host-side execution seam under crates/shell/src/execution/, plus crates/shell/src/execution/agent_runtime/auto_attach.rs or crates/shell/src/execution/orchestrator_world_dispatch.rs only if coexistence proof or bounded sequencing requires it. Keep Packet 4 work out of scope, and do not land remote sync, federation delivery, or public host-inbox UX/policy work. Run cargo test -p shell host_inbox -- --nocapture, cargo test -p shell auto_attach -- --nocapture, and cargo test -p shell orchestrator_world_dispatch -- --nocapture if you touched that seam. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 51 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 51 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 51 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md

Mission:
- Land Slice 51 Packet 4 only: Docs Alignment And Final Validation.
- This packet assumes Packets 1 through 3 are already landed and checkpoint-green.
- Keep the slice bounded to docs/runtime truth and the final validation wall.

Before editing:
1. Read SPEC-51, PLAN-51, TASKS-51, then inspect the live docs and config surfaces in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/TRACE.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Align docs with the landed Slice 51 host-global inbox boundary.
- Task 4.2: Run the final validation wall.

Out of scope:
- reopening Packets 1 through 3 unless validation proves a bounded in-scope follow-up is required
- remote sync protocols
- receive-cursor or sync-state coordination
- lease or lock coordination for cross-host delivery
- public host-inbox CLI, daemon UX, review surface, or new public policy/config surfaces
- broader federation routing/productization

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run the Packet 4 verification commands:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell host_inbox -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell obligation -- --nocapture`
  - `cargo test -p shell orchestrator_world_dispatch -- --nocapture` if Packet 3 touched that seam
  - `cargo test -p shell auto_attach -- --nocapture` if Packet 3 touched coexistence with router sequencing
  - `cargo test --workspace -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 docs/validation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-51 / PLAN-51 / TASKS-51 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- the Slice 51 boundary is still safely bounded
- docs and runtime truth are honest
- the validation wall is green

Implementation subagent prompt:
/goal Land Slice 51 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md first. Work only on Task 4.1 and Task 4.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Align docs and runtime truth without widening scope, then run the full Packet 4 validation wall. If a validation failure requires more code or doc changes, keep any follow-up bounded to explicit Slice 51 surfaces only. Run cargo fmt --all -- --check, cargo clippy --workspace --all-targets -- -D warnings, cargo test -p shell host_inbox -- --nocapture, cargo test -p shell state_store -- --nocapture, cargo test -p shell obligation -- --nocapture, cargo test -p shell orchestrator_world_dispatch -- --nocapture if Packet 3 touched that seam, cargo test -p shell auto_attach -- --nocapture if Packet 3 touched coexistence with router sequencing, and cargo test --workspace -- --nocapture. Final message must state whether Packet 4 is checkpoint-green, what symbols changed, what verification ran, whether any reopen condition was discovered, and whether Slice 51 is ready for final closeout.

Review subagent prompt:
Review the committed Slice 51 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 51 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-51-internal-family-2-host-global-inbox-layering-and-local-obligation-materialization-boundary.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-51.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-51.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 51 is ready for final closeout.
- If anything is not green, say explicitly that Slice 51 is not ready for final closeout.
```
