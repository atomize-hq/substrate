# PROMPTS-58: Packet Orchestration Prompts For Slice 58

Source spec: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Source plan: [PLAN-58-placement-aware-agent-inventory-and-selector-contract.md](./PLAN-58-placement-aware-agent-inventory-and-selector-contract.md)  
Source tasks: [TASKS-58.md](./TASKS-58.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `58` spec/plan/tasks stack and current repo truth. Preserve the landed Slice `59` runtime/bootstrap contract throughout.

## Packet 1 Prompt

```text
/goal Land Slice 58 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md

Mission:
- Land Slice 58 Packet 1 only: Placement-Aware Schema And Projection.
- Do not start Packet 2.
- Keep the work bounded to `version: 2` placement-aware inventory parsing and projection.

Before editing:
1. Read SPEC-58, PLAN-58, TASKS-58, and the landed Slice 59 spec first.
2. Inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agents_validate.rs
   - the narrowest adjacent inventory tests under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests
3. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Add typed `config.placements` schema for logical agents.
- Task 1.2: Project placement-aware logical agents into realized placement rows.

Out of scope:
- Packet 2, 3, or 4 work
- exact selector cutover
- policy/doc/runtime-control exact-id migration
- guest-runtime/bootstrap redesign
- weakening or reopening landed Slice 59 runtime truth

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run:
  - `cargo test -p shell agents_validate -- --nocapture`
  - `cargo test -p shell agent_inventory -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-58 / PLAN-58 / TASKS-58 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- placement-aware inventory parses
- projection emits exact realized rows
- no selector cutover has happened yet
- the distinction between logical id and realized id is explicit in the projection layer

Implementation subagent prompt:
/goal Land Slice 58 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md, and the landed Slice 59 spec first. Work only on Task 1.1 and Task 1.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_inventory.rs and the narrowest adjacent inventory-validation tests. Keep Packet 2 through Packet 4 work out of scope. Do not cut over exact selectors, migrate policy/docs, or reopen Slice 59 runtime semantics. Run cargo test -p shell agents_validate -- --nocapture and cargo test -p shell agent_inventory -- --nocapture. Final message must state whether Packet 1 is checkpoint-green, what symbols changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 58 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 58 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 58 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md

Mission:
- Land Slice 58 Packet 2 only: Placement-Qualified Exact Selector Derivation.
- Do not start Packet 3.
- Keep the work bounded to placement-qualified realized ids, exact backend ids, fail-closed selection, and derived human-facing labels.

Before editing:
1. Read SPEC-58, PLAN-58, TASKS-58, and the landed Slice 59 spec first.
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. Inspect the live code in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_inventory.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/dispatch_contract.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/validator.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs
   - the narrowest adjacent tests under /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
6. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Derive placement-qualified realized agent ids and exact backend ids.
- Task 2.2: Keep multi-placement selection fail-closed and labels read-only.

Out of scope:
- Packet 3 or 4 work
- inventory file migration
- docs/policy/smoke-helper exact-id migration
- backend-id grammar changes
- logical-agent shorthand selectors
- weakening or reopening landed Slice 59 host-vs-world runtime separation

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run:
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-58 / PLAN-58 / TASKS-58 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- placement-qualified exact ids are live for placement-aware entries
- labels are clearly separated from selectors
- host/world cannot silently cross-match
- backend-id grammar and policy model remain unchanged

Implementation subagent prompt:
/goal Land Slice 58 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md, and the landed Slice 59 spec first. Work only on Task 2.1 and Task 2.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement the minimum code and tests needed in crates/shell/src/execution/agent_inventory.rs, crates/shell/src/execution/agent_runtime/dispatch_contract.rs, crates/shell/src/execution/agent_runtime/validator.rs, crates/shell/src/execution/agents_cmd.rs, and the narrowest adjacent Packet 2 test seams. Keep Packet 3 and Packet 4 work out of scope. Do not migrate inventory files, docs, or policy examples yet. Do not change backend-id grammar. Run cargo test -p shell dispatch_contract -- --nocapture, cargo test -p shell agent_runtime::validator -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, and cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 58 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 58 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 58 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md

Mission:
- Land Slice 58 Packet 3 only: Inventory, Policy, Fixture, And Doc Migration.
- Do not start Packet 4.
- Keep the work bounded to placement-aware inventory file conversion plus exact-id migration across authoritative forward surfaces.

Before editing:
1. Read SPEC-58, PLAN-58, TASKS-58, and the landed Slice 59 spec first.
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. Inspect the live code and artifacts in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/claude_code.yaml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/claude_code_world.yaml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex.yaml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/config/agents/codex_world.yaml
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/CONFIGURATION.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/policy_model.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/repl/async_repl.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-fresh-install-gateway-smoke.sh
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/substrate/dev-fresh-install-gateway-smoke-claude-code.sh
4. Build an explicit inventory of authoritative forward surfaces that still pin legacy `cli:*_world` ids before editing.
5. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
6. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
7. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Convert split inventory to placement-aware logical-agent files.
- Task 3.2: Migrate exact backend ids in docs, examples, policy samples, and fixtures.

Out of scope:
- Packet 4 work
- backend-id grammar changes
- new selector ergonomics
- reopening Slice 59 runtime/install/provisioning semantics
- unrelated cleanup outside authoritative forward surfaces

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run:
  - `cargo test -p shell agent_inventory -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - a scoped grep/manual review of authoritative forward surfaces that previously pinned `cli:*_world` ids
- If implementation is green, run `git diff --stat` and `git status --short`.
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-58 / PLAN-58 / TASKS-58 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- the repo has one forward logical-agent file per multi-placement agent
- docs and fixtures use the current exact ids consistently
- old and new exact ids are not both presented as equivalent forward truth
- any temporary compatibility posture is explicit rather than implied

Implementation subagent prompt:
/goal Land Slice 58 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md, and the landed Slice 59 spec first. Work only on Task 3.1 and Task 3.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Convert the split inventory files to the placement-aware forward model and migrate authoritative policy/docs/runtime-control/smoke-helper exact-id surfaces to placement-qualified ids without widening scope. Preserve landed Slice 59 runtime truth, remediation, and installer semantics while exact ids move. Run cargo test -p shell agent_inventory -- --nocapture, cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture, cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture, cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture, plus a scoped grep/manual review of authoritative forward surfaces that previously pinned legacy `cli:*_world` ids. Final message must state whether Packet 3 is checkpoint-green, what symbols/files changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 58 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 58 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 58 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md

Mission:
- Land Slice 58 Packet 4 only: Final Validation Wall.
- Do not reopen earlier packets unless validation proves a concrete issue.
- Keep the work bounded to proving the Packet 1-3 cutover is coherent and that landed Slice 59 runtime truth remains intact.

Before editing:
1. Read SPEC-58, PLAN-58, TASKS-58, and the landed Slice 59 spec first.
2. Verify Packet 3 is already landed and checkpoint-green on the current tree.
3. Inspect the final validation seams and any remaining migration-sensitive exact-id surfaces before running the wall.
4. If GitNexus indicates the index is stale, run `npx gitnexus analyze`.
5. If validation exposes a concrete defect that requires a fix, run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
6. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 4.1: Run the final validation wall.
- Only if the validation wall finds a concrete Packet 1-3 issue, fix that exact issue and rerun the wall.

Out of scope:
- new feature work
- selector ergonomics beyond Slice 58
- backend-id grammar changes
- reopening Slice 59 runtime/provisioning semantics except where validation proves a regression caused by the Slice 58 cutover

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to execute Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must start with the validation wall before making changes.
- Run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell agents_validate -- --nocapture`
  - `cargo test -p shell agent_inventory -- --nocapture`
  - `cargo test -p shell dispatch_contract -- --nocapture`
  - `cargo test -p shell agent_runtime::validator -- --nocapture`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
- If all checks pass with no code changes, report Packet 4 green and do not invent an empty commit.
- If validation exposes a concrete issue that requires changes, fix only that issue, rerun the relevant wall commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes before committing, and commit the fix before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- If Packet 4 required fixes, the review subagent must review only those Packet 4 fix changes against SPEC-58 / PLAN-58 / TASKS-58 and the live diff.
- If the review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant validation commands, run `git diff --stat` and `git status --short`, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.
- If Packet 4 was validation-only with zero code changes, state explicitly that no review/fix commit loop was needed because no diff was produced.

Commit policy:
- If Packet 4 is validation-only and produces no changes, do not create an empty commit.
- If Packet 4 requires fixes, commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- placement-aware inventory is the forward contract
- exact selectors remain fail-closed and placement-qualified
- labels remain human-facing only
- landed Slice 59 runtime truth, remediation, and installer semantics remain intact after the exact-id cutover

Implementation subagent prompt:
/goal Run Slice 58 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md, and the landed Slice 59 spec first. Start by running the full Packet 4 validation wall. Do not make changes unless the validation wall reveals a concrete Packet 1-3 defect. If a fix is required, keep it minimal and strictly within Packet 4 scope, run GitNexus impact analysis before editing any production symbol, and rerun the relevant wall commands after the fix. Final message must state whether Packet 4 is checkpoint-green, what verification ran, whether any code changes were necessary, whether the Slice 58 cutover preserved landed Slice 59 semantics, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 58 Packet 4 fix change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality, but only if Packet 4 required a code fix. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Review only the Packet 4 fix diff. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 58 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-58-placement-aware-agent-inventory-and-selector-contract.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-58-placement-aware-agent-inventory-and-selector-contract.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-58.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 validation-wall issues without widening scope. Re-run the relevant Packet 4 validation commands. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- If code changes were required, report GitNexus impact-analysis results for edited production symbols and GitNexus detect-changes results before each commit.
- If no code changes were required, state explicitly that Packet 4 was validation-only and no commit/review/fix loop was needed.
- State whether Slice 58 is closed and whether any reopen condition remains.
```
