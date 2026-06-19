# PROMPTS-60: Packet Orchestration Prompts For Slice 60

Source spec: [SPEC-60-post-placement-aware-compatibility-retirement.md](./SPEC-60-post-placement-aware-compatibility-retirement.md)  
Source plan: [PLAN-60-post-placement-aware-compatibility-retirement.md](./PLAN-60-post-placement-aware-compatibility-retirement.md)  
Source tasks: [TASKS-60.md](./TASKS-60.md)  
Related frozen floor:
- [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)
- [TASKS-59.md](./TASKS-59.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/home/azureuser/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/home/azureuser/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `60` spec/plan/tasks stack and the frozen Slice `59` runtime-truth floor. Preserve Slice `59` runtime-realizability, world-deps, and installer semantics throughout. Do not use older placeholder-only Slice `60` artifacts as authority.

## Packet 1 Prompt

```text
/goal Land Slice 60 Packet 1 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 60 Packet 1 only: Freeze The Retirement Boundary.
- Do not start Packet 2.
- Keep the work bounded to defining the forward-truth grep wall, historical allowlist, and any minimal clarifying comments required to make those boundaries explicit.

Before editing:
1. Read SPEC-60, PLAN-60, TASKS-60, and the frozen Slice 59 spec/task closeout first.
2. Inspect the live docs in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md
   - /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md
   - /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md
   - any narrowly adjacent docs/comments that must explicitly mark a retained old-id reference as historical
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. If you end up editing any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise state explicitly that Packet 1 remained docs-only and no production-symbol impact run was required.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Freeze the forward-surface grep wall and historical allowlist.

Out of scope:
- Packet 2, 3, 4, or 5 work
- effective-inventory bridge retirement
- split-entry selector retirement in runtime/control surfaces
- forward docs/script/test migration beyond what is strictly required to freeze the boundary
- reopening Slice 59 runtime semantics

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1.
- After implementation, run:
  - `rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" docs config crates/shell scripts -g '!target'`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-60 / PLAN-60 / TASKS-60 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- forward truth versus historical evidence is explicit
- the grep wall is defined
- reopen conditions are stated if hidden live dependencies appear

Implementation subagent prompt:
/goal Land Slice 60 Packet 1 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 1.1. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 1 remained docs-only. Implement only the minimum doc/comment changes needed to freeze the forward-truth grep wall and historical allowlist. Keep Packets 2 through 5 out of scope, and do not begin compatibility-bridge retirement, selector retirement, or broad forward-surface migration. Run the Packet 1 grep verification plus `git diff --stat` and `git status --short`. Final message must state whether Packet 1 is checkpoint-green, what files changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 60 Packet 1 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md, plus the frozen Slice 59 floor as context only. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to a contract/doc slice. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 60 Packet 1 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained docs-only. Fix only the flagged Packet 1 issues without widening scope. Re-run the Packet 1 grep verification plus `git diff --stat` and `git status --short`. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results if any production symbols were edited; otherwise say explicitly that Packet 1 stayed docs-only.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 60 Packet 2 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 60 Packet 2 only: Retire The Effective-Inventory Compatibility Bridge.
- Do not start Packet 3.
- Keep the work bounded to retiring the Slice 58 Packet 1.5 compatibility materialization path from forward live behavior while preserving placement-aware realized identities and the Slice 59 runtime floor.

Before editing:
1. Read SPEC-60, PLAN-60, TASKS-60, and the frozen Slice 59 spec/task closeout first.
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. Inspect the live code in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
6. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Remove v2-to-split-entry effective-inventory materialization from forward live behavior.
- Task 2.2: Prove public control surfaces no longer rely on the bridge.

Out of scope:
- Packet 3, 4, or 5 work
- split-entry selector retirement in validator/dispatch/runtime surfaces
- broad doc/script/example migration
- reopening Slice 59 runtime semantics
- adding a new compatibility bridge

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run:
  - `cargo test -p shell agent_inventory -- --nocapture`
  - `cargo test -p shell agents_validate -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-60 / PLAN-60 / TASKS-60 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- placement-aware effective inventory no longer depends on split-entry materialization
- live control surfaces still function with realized placement identities
- no new compatibility branch was introduced

Implementation subagent prompt:
/goal Land Slice 60 Packet 2 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 2.1 and Task 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_inventory.rs and the narrowest adjacent control-surface tests. Keep Packets 3 through 5 out of scope. Do not begin split-entry selector retirement, broad doc migration, or any Slice 59 runtime-semantic changes. Run cargo test -p shell agent_inventory -- --nocapture, cargo test -p shell agents_validate -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, and cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 60 Packet 2 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 60 Packet 2 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 60 Packet 3 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 60 Packet 3 only: Retire Split-Entry Exact Selector Compatibility.
- Do not start Packet 4.
- Keep the work bounded to retiring split-entry exact selectors from forward runtime/control/policy behavior while preserving placement-qualified exact ids and the Slice 59 runtime floor.

Before editing:
1. Read SPEC-60, PLAN-60, TASKS-60, and the frozen Slice 59 spec/task closeout first.
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. Inspect the live code in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs
   - /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs
   - the narrowest adjacent tests under /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
6. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Reject retired split-entry exact backend ids on runtime selection paths.
- Task 3.2: Retire forward policy/example/REPL dependence on split-entry exact ids.

Out of scope:
- Packet 4 or 5 work
- broad docs/scripts migration beyond the exact policy/example/REPL seams required here
- changing backend-id grammar
- any Slice 59 runtime-semantic change
- reintroducing alias fallback

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-60 / PLAN-60 / TASKS-60 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- split-entry exact ids are no longer forward-valid selectors
- placement-qualified exact ids are the only green path
- runtime truth from Slice 59 remains untouched

Implementation subagent prompt:
/goal Land Slice 60 Packet 3 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_runtime/dispatch_contract.rs, crates/shell/src/execution/agent_runtime/validator.rs, crates/shell/src/execution/policy_model.rs, crates/shell/src/repl/async_repl.rs, and the narrowest adjacent runtime/control tests. Keep Packets 4 and 5 out of scope. Do not widen into broad docs/scripts migration or any Slice 59 runtime-semantic reopen. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell agent_runtime::validator -- --nocapture, cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture, and cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture. Final message must state whether Packet 3 is checkpoint-green, what symbols changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 60 Packet 3 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 60 Packet 3 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 60 Packet 4 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 60 Packet 4 only: Migrate Forward Docs, Scripts, And Fixtures.
- Do not start Packet 5.
- Keep the work bounded to forward docs/scripts/examples/fixtures and explicit retirement tests. Do not reopen runtime semantics or reintroduce compatibility.

Before editing:
1. Read SPEC-60, PLAN-60, TASKS-60, and the frozen Slice 59 spec/task closeout first.
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. Inspect the live forward surfaces in:
   - /home/azureuser/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /home/azureuser/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh
   - /home/azureuser/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh
   - relevant forward-facing fixtures under /home/azureuser/__Active_Code/atomize-hq/substrate/crates/shell/tests
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 4 remained docs/tests/scripts-only.
6. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Normalize authoritative docs and active scripts to placement-qualified ids.
- Task 4.2: Normalize forward-facing test fixtures while preserving explicit negative retirement tests.

Out of scope:
- Packet 5 validation-only closeout work
- reopening Packet 2 or Packet 3 decisions
- broad historical `llm-last-mile/` provenance rewrites
- any Slice 59 runtime-semantic change

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 4.1 and Task 4.2.
- After implementation, run:
  - `rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" docs scripts -g '!target'`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-60 / PLAN-60 / TASKS-60 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- forward docs/scripts/examples are placement-aware only
- remaining legacy-name hits are explicit history or explicit negative tests
- old and new ids are no longer presented as coequal current truth

Implementation subagent prompt:
/goal Land Slice 60 Packet 4 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 4.1 and Task 4.2. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 4 remained docs/tests/scripts-only. Implement only the minimum doc/script/test-fixture changes needed to make forward current truth placement-aware only while preserving explicit negative retirement tests. Keep Packet 5 out of scope. Do not rewrite broad historical provenance or reopen Slice 59 runtime semantics. Run the Packet 4 grep verification and the three Packet 4 shell tests, plus `git diff --stat` and `git status --short`. Final message must state whether Packet 4 is checkpoint-green, what files changed, what verification ran, whether Packet 5 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 60 Packet 4 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. Review only Packet 4 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to docs/scripts/tests retirement work. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 60 Packet 4 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained docs/tests/scripts-only. Fix only the flagged Packet 4 issues without widening scope. Re-run the relevant Packet 4 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results if any production symbols were edited; otherwise say explicitly that Packet 4 stayed docs/tests/scripts-only.
- Report GitNexus detect-changes results before each commit.
- State whether Packet 5 is unblocked.
- If anything is not green, say explicitly that Packet 5 must not begin.
```

## Packet 5 Prompt

```text
/goal Land Slice 60 Packet 5 only in /home/azureuser/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md
- /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md

Mission:
- Land Slice 60 Packet 5 only: Final Validation Wall.
- Keep the work bounded to running the validation wall and making only the narrowest follow-up fixes required by failing Packet 5 commands.
- Do not reopen Packet 1 through Packet 4 decisions without an explicit reopen condition.

Before editing:
1. Read SPEC-60, PLAN-60, TASKS-60, and the frozen Slice 59 spec/task closeout first.
2. Verify Packet 4 is already landed and checkpoint-green on the current tree.
3. Inspect the live Packet 5 validation wall in TASKS-60 and any files directly implicated by failing commands.
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. If you touch any Rust production symbol while fixing a validation failure, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 5 remained validation-only or docs/tests/scripts-only.
6. Stay strictly within Packet 5 scope.

Packet 5 scope:
- Task 5.1: Run the final validation wall and forward-truth grep proof.

Out of scope:
- redesigning earlier packet decisions without a concrete reopen condition
- broad opportunistic cleanup
- any Slice 59 runtime-semantic change unless a failing command proves one is required and the slice is formally reopened

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 5.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 5.1.
- Run the Packet 5 validation wall:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell agents_validate -- --nocapture`
  - `cargo test -p shell agent_inventory -- --nocapture`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `rg -n "\bcodex_world\b|\bclaude_code_world\b|cli:(codex|claude_code)_world\b" docs config crates/shell scripts -g '!target'`
- If the wall is already green, still run `git diff --stat` and `git status --short`, run GitNexus detect-changes before committing if any changes were made, and commit the Packet 5 validation/fix work before review.
- If a validation failure requires changes, keep the fixes bounded to the failing surfaces only, then rerun only the relevant commands plus any dependent Packet 5 wall commands affected by the fix.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 5 against SPEC-60 / PLAN-60 / TASKS-60 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 5 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again if any changes remain, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation or bounded validation-fix work before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 5 checkpoint:
- the Slice 58 compatibility bridge is retired from forward live behavior
- placement-qualified exact ids are the only forward selectors
- authoritative forward truth is placement-aware only
- Slice 59 runtime semantics remain green

Implementation subagent prompt:
/goal Land Slice 60 Packet 5 only in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-59.md first. Work only on Task 5.1. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol while fixing a failing validation command, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 5 remained validation-only or docs/tests/scripts-only. Run the full Packet 5 validation wall first. If a failure requires changes, keep the fix bounded strictly to the failing Slice 60 surface and rerun the relevant commands plus any dependent Packet 5 checks. Do not reopen earlier packet decisions unless you hit an explicit reopen condition. Final message must state whether Packet 5 is checkpoint-green, what symbols or files changed, what verification ran, whether bounded follow-up fixes were required, and whether Slice 60 is ready for final closeout.

Review subagent prompt:
Review the committed Slice 60 Packet 5 change in /home/azureuser/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. Review only Packet 5 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 5 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 60 Packet 5 review findings in /home/azureuser/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-60-post-placement-aware-compatibility-retirement.md, /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-60-post-placement-aware-compatibility-retirement.md, and /home/azureuser/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-60.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol while fixing a flagged issue, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained validation-only or docs/tests/scripts-only. Fix only the flagged Packet 5 issues without widening scope. Re-run the relevant Packet 5 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 5 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 5 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols if any were touched; otherwise say explicitly that Packet 5 stayed validation-only or docs/tests/scripts-only.
- Report GitNexus detect-changes results before each commit.
- State whether Slice 60 is ready for final closeout.
- If anything is not green, say explicitly that Slice 60 must not be closed out.
```
