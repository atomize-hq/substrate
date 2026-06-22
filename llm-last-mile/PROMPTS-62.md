# PROMPTS-62: Packet Orchestration Prompts For Slice 62

Source spec: [SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md](./SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md)  
Source plan: [PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md](./PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md)  
Source tasks: [TASKS-62.md](./TASKS-62.md)  
Related architectural inputs:
- [DESIGN-agent-facing-config-projection-framework.md](./DESIGN-agent-facing-config-projection-framework.md)
- [DESIGN-codex-world-home-auth-and-config-mapping.md](./DESIGN-codex-world-home-auth-and-config-mapping.md)
- [DESIGN-workspace-scoped-adapter-overlay-model.md](./DESIGN-workspace-scoped-adapter-overlay-model.md)  
Related handoffs:
- [handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md](../handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md)
- [handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md](../handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md)  
Current branch at prompt authoring time: `feat/internal-host-orchestrator-world-dispatch-bootstrap`  
Worker implementation skill dependency: `$incremental-implementation`  
Worker review skill dependency: `$code-review-and-quality`  
Resolved implementation skill path: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Resolved review skill path: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`  
Workspace root: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`

Skill availability note:

1. These prompts require a fresh parent session that can use `$incremental-implementation` for implementation/fix workers and `$code-review-and-quality` for review workers.
2. The skill docs were explicitly provided while authoring this file at:
   - `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`
   - `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`
3. The fresh parent session should still verify that both skills are available before spawning workers.
4. If either required skill is unavailable, the orchestration agent must stop immediately and report the missing dependency instead of silently substituting a different workflow.

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded in the live Slice `62` spec/plan/tasks stack and preserves the packet boundary: this slice is a bounded compatibility-bridge repair for the direct `cli:codex-world` member path, not a generic Codex projection framework and not future MCP/app-runtime/apps-connectors/hooks/rules/skills/plugin/custom-agent/workspace-overlay work.

## Packet 1 Prompt

```text
/goal Orchestrate and land TASKS-62 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code/docs yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md

Mission:
- Land Packet 1 only: Pin The Direct Compatibility Bridge Boundary.
- Do not start Packet 2.
- Keep the work bounded to Task 1.1 and Task 1.2 in TASKS-62.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-62, PLAN-62, and TASKS-62 before dispatching any worker.
3. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
4. Spawn a fresh GPT-5.4 subagent on high for implementation.
5. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
6. When implementation completes and Packet 1 verification is green, run any parent-side rechecks you need, run GitNexus detect-changes before commit if any production code changed, and commit the implementation changes before review.
7. Spawn a fresh GPT-5.4 subagent on high for review.
8. The review subagent must explicitly use `$code-review-and-quality`.
9. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
10. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each fix round, rerun the relevant verification, run GitNexus detect-changes before commit if any production code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
12. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-62, PLAN-62, and TASKS-62 first.
2. Inspect the live code and test surface in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs
   - adjacent world-service tests around `prepare_codex_runtime_env`
   - nearby shell tests for member-dispatch builder behavior
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 1.1: Expand world-service bootstrap coverage for bounded config materialization.
- Task 1.2: Keep exact-backend seed-home gating pinned in shell tests.

Out of scope:
- Packet 2 or Packet 3 work
- rendering any new bootstrap config behavior
- replay of `~/.codex/*.config.toml` profile overlays
- replay of repo `.codex/config.toml` project config
- MCP/app-runtime/apps-connectors/hooks/rules/skills/plugins/custom-agent/workspace-overlay work
- weakening exact-backend host-read policy

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-62 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md first. Work only on Task 1.1 and Task 1.2. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Add the minimum coverage needed to prove the direct member bridge contract is bounded auth-plus-config behavior rather than auth alone, while keeping exact-backend allowlist truth pinned. Do not widen into Packet 2 or Packet 3. Run `cargo test -p world-service prepare_codex_runtime_env -- --nocapture` and `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`. Do not commit. Final message must state whether Packet 1 is checkpoint-green, what files changed, what GitNexus impact results were found, what verification ran, whether Packet 2 is unblocked, and whether any out-of-scope widening pressure was discovered.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 62 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md. Review only Packet 1 and the live diff. Focus on correctness of the test pinning, exact-backend gating preservation, packet-boundary discipline, and whether broader projection work was smuggled in. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 62 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 1 issues without widening scope. Re-run `cargo test -p world-service prepare_codex_runtime_env -- --nocapture` and `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
  - `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit if any non-test code changed.

Packet 1 checkpoint:
- world-service tests pin that the bridge includes bounded config behavior rather than auth alone
- shell tests still pin exact-backend allowlist truth
- the slice boundary is tight enough that Packet 2 cannot hide broader config projection inside the same seam

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
/goal Orchestrate and land TASKS-62 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code/docs yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md

Mission:
- Land Packet 2 only: Land Bounded Bootstrap Config Materialization And Fail-Closed Diagnostics.
- Do not start Packet 3.
- Keep the work bounded to Task 2.1, Task 2.2, and Task 2.3 in TASKS-62.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-62, PLAN-62, and TASKS-62 before dispatching any worker.
3. Verify Packet 1 is already landed and checkpoint-green on the current tree.
4. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
5. Spawn a fresh GPT-5.4 subagent on high for implementation.
6. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
7. When implementation completes and Packet 2 verification is green, run any parent-side rechecks you need, run GitNexus detect-changes before commit, and commit the implementation changes before review.
8. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
9. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
10. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each fix round, rerun the relevant verification, run GitNexus detect-changes before commit, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
12. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-62, PLAN-62, and TASKS-62 first.
2. Inspect the live code and test surface in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/Cargo.toml
   - adjacent world-service tests around `prepare_codex_runtime_env`
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If GitNexus reports HIGH or CRITICAL risk, stop and report before editing.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 2.1: Derive and render the narrow non-secret Codex startup subset.
- Task 2.2: Keep the bridge internal and bounded.
- Task 2.3: Add explanation-ready fail-closed diagnostics for missing bounded startup truth.

Out of scope:
- Packet 3 work
- replay of `~/.codex/*.config.toml` profile overlays
- replay of repo `.codex/config.toml` project config
- copying broader Codex home/config state
- generic projection framework work
- MCP/app-runtime/apps-connectors/hooks/rules/skills/plugins/custom-agent/workspace-overlay work
- plugin-bundled MCP servers, plugin-bundled hooks, managed requirements/allowlists, or workspace-shared plugin state

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-62 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md first. Work only on Task 2.1, Task 2.2, and Task 2.3. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Read only from the already policy-gated host seed-home source, derive only the smallest non-secret user-level startup subset required for truthful startup on the diagnosed profile, render that subset into isolated CODEX_HOME/config.toml, preserve auth materialization behavior, strip SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME before child spawn, and fail closed with a direct explanation if the bounded startup subset cannot be derived. Do not widen into Packet 3 or generic projection work. Run `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`, `rg -n "config\\.toml|model|provider|base_url|profile" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`, and `rg -n "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME|mcp|skills|plugin|workspace \\.codex|app-runtime|hooks|rules|agents|requirements" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`. Do not commit. Final message must state whether Packet 2 is checkpoint-green, what symbols changed, what GitNexus impact results were found, what verification ran, whether Packet 3 is unblocked, and whether any widening or ambiguity remains.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 62 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md. Review only Packet 2 and the live diff. Focus on correctness of bounded config derivation/materialization, fail-closed diagnostics, compatibility-only posture, and prevention of profile/project replay or broader capability projection. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 62 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 2 issues without widening scope. Re-run `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`, `rg -n "config\\.toml|model|provider|base_url|profile" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`, and `rg -n "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME|mcp|skills|plugin|workspace \\.codex|app-runtime|hooks|rules|agents|requirements" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
  - `rg -n "config\\.toml|model|provider|base_url|profile" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`
  - `rg -n "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME|mcp|skills|plugin|workspace \\.codex|app-runtime|hooks|rules|agents|requirements" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-service/src/member_runtime.rs`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit.

Packet 2 checkpoint:
- isolated direct-member homes receive bounded startup config instead of depending on Codex defaults
- profile/project config replay and broader config/state domains are still absent from the bridge
- missing bounded startup truth fails closed with a specific explanation
- the implementation still reads as a transitional direct-member seam rather than a generic projection framework

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
/goal Orchestrate and land TASKS-62 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

You are the parent/orchestration session. Stay orchestration-only: do not implement, review, or fix code/docs yourself.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md

Mission:
- Land Packet 3 only: Docs, Validation Wall, And Live Smoke Proof.
- Keep the work bounded to Task 3.1, Task 3.2, and Task 3.3 in TASKS-62.
- Do not widen into future capability work.

Required orchestration loop:
1. Verify the fresh parent session has both required skills available: `$incremental-implementation` and `$code-review-and-quality`.
2. Read SPEC-62, PLAN-62, and TASKS-62 before dispatching any worker.
3. Verify Packet 2 is already landed and checkpoint-green on the current tree.
4. Inspect `git status --short` and preserve unrelated dirt; stage and commit only packet-relevant files.
5. Spawn a fresh GPT-5.4 subagent on high for implementation.
6. The implementation subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
7. When implementation completes and Packet 3 verification is green, run any parent-side rechecks you need, run GitNexus detect-changes before commit if any production code changed, and commit the implementation changes before review.
8. Spawn a fresh GPT-5.4 subagent on high for review using `$code-review-and-quality`.
9. If the review subagent flags issues, spawn a new fresh GPT-5.4 subagent on high to fix only those findings.
10. The fix subagent prompt must begin with `/goal ` and must explicitly instruct the worker to use `$incremental-implementation`.
11. After each fix round, rerun the relevant verification, run GitNexus detect-changes before commit if any production code changed, commit the fixes, and then rerun a fresh GPT-5.4 high review subagent.
12. Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Use conventional commit style.
- Do not amend unless absolutely required.

Before editing:
1. Read SPEC-62, PLAN-62, and TASKS-62 first.
2. Inspect the live docs and proof surfaces in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md
   - the current installed runtime/redeploy path
   - the exact June 21, 2026 public bootstrap smoke command from the user transcript/handoffs
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius. If Packet 3 remains docs-plus-verification only, say so explicitly.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 3.1: Update direct-member operator/developer docs with bounded bridge posture.
- Task 3.2: Run the targeted validation wall.
- Task 3.3: Rebuild/redeploy and rerun the live smoke floor plus the June 21 public bootstrap smoke.

Out of scope:
- new config projection features
- Packet 4 or later work
- future MCP/app-runtime/apps-connectors/hooks/rules/skills/plugins/custom-agent/workspace-overlay work
- turning the compatibility bridge into the steady-state architecture

Implementation worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Implementation subagent prompt:
  /goal Implement TASKS-62 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md first. Work only on Task 3.1, Task 3.2, and Task 3.3. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Update docs so they accurately describe direct `cli:codex-world` member bootstrap as a bounded transitional compatibility bridge that includes narrow non-secret startup config in addition to auth seeding, and make sure the wording does not imply this is the final gateway-front-door architecture or a generic future-capability base. Then run `rg -n "cli:codex-world|CODEX_HOME|compatibility bridge|gateway" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md`, `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`, `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`, `cargo test -p shell c3_internal_toolbox_run_world_task_fast_completion_still_streams_registered_task_run_id_before_terminal_result -- --nocapture`, rebuild/redeploy the installed runtime, run `~/.substrate/bin/substrate agent start --backend cli:codex-world --scope world --prompt 'Reply with WORLD READY only.' --json`, and rerun the exact June 21, 2026 public bootstrap smoke command from the transcript/handoffs and confirm it creates `from_the_world_worker.md`. Do not commit. Final message must state whether Packet 3 is checkpoint-green, what files changed, what verification ran, what smoke evidence was captured, whether any production-symbol impact analysis was required, and whether the slice still reads as transitional rather than a widened capability bridge.

Review worker requirements:
- Spawn a fresh GPT-5.4 subagent on high.
- Review subagent prompt:
  Review the committed Slice 62 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md, and the June 21 handoffs. Review only Packet 3 and the live diff plus verification evidence. Focus on docs truthfulness, validation-wall completeness, rebuilt-runtime smoke proof, and preventing wording that encourages future capability widening. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix worker requirements:
- If review finds issues, spawn a fresh GPT-5.4 subagent on high.
- Fix subagent prompt:
  /goal Address only the required Slice 62 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-62-world-codex-direct-member-bootstrap-compatibility-bridge-completion.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-62.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-135249-world-agent-run-world-task-auth-seeding.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/handoffs/2026-06-21-154318-world-agent-run-world-task-model-config.md. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Before editing any production Rust symbol, run GitNexus impact analysis and report the blast radius; if GitNexus reports HIGH or CRITICAL risk, stop and report before editing. Fix only the flagged Packet 3 issues without widening scope. Re-run the verification and smoke commands implicated by the fix. Do not commit. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Verification and commit requirements for the parent session:
- After implementation and after each fix round, run:
  - `rg -n "cli:codex-world|CODEX_HOME|compatibility bridge|gateway" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md`
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p world-service prepare_codex_runtime_env -- --nocapture`
  - `cargo test -p shell codex_member_dispatch_injects_internal_seed_home_when_backend_is_allowlisted -- --nocapture`
  - `cargo test -p shell c3_internal_toolbox_run_world_task_fast_completion_still_streams_registered_task_run_id_before_terminal_result -- --nocapture`
  - `~/.substrate/bin/substrate agent start --backend cli:codex-world --scope world --prompt 'Reply with WORLD READY only.' --json`
  - the exact June 21, 2026 public bootstrap smoke command from the transcript/handoffs
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before each commit if any non-test code changed.

Packet 3 checkpoint:
- repo truth describes the direct member seam as a bounded compatibility bridge
- targeted validation is green
- the rebuilt installed runtime fixes the real smoke
- nothing in docs or implementation encourages future capability work to piggyback on this bridge

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results if any production symbols were edited; otherwise say explicitly that Packet 3 remained docs/verification-only.
- Report GitNexus detect-changes results before each commit if it was required.
- Report exact live-smoke evidence, including whether `from_the_world_worker.md` was created.
- If anything is not green, say explicitly that the slice is not closed out.
```
