# PROMPTS-55: Packet Orchestration Prompts For Slice 55

Source spec: [SPEC-55-broader-caller-surface-contract-freeze.md](./SPEC-55-broader-caller-surface-contract-freeze.md)  
Source plan: [PLAN-55-broader-caller-surface-contract-freeze.md](./PLAN-55-broader-caller-surface-contract-freeze.md)  
Source tasks: [TASKS-55.md](./TASKS-55.md)  
Source tracker: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `55` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

Packet mapping for Slice `55`:

1. **Packet 1** = Task `55.1`
2. **Packet 2** = Tasks `55.2` and `55.3`
3. **Packet 3** = Task `55.4`
4. **Packet 4** = Task `55.5`

## Packet 1 Prompt

```text
/goal Land Slice 55 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md

Mission:
- Land Slice 55 Packet 1 only: freeze the caller-surface decision ledger in repo-facing docs.
- Do not start Packet 2.
- Keep the work bounded to Task 55.1 only.

Before editing:
1. Re-read SPEC-55, PLAN-55, TASKS-55, the REMAINING tracker note, and AGENT_ORCHESTRATION_GAP_MATRIX.md.
2. Confirm the packet mapping from PLAN-55:
   - Packet 1 = Task 55.1 only.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. If you end up editing any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius. If Packet 1 remains docs-only, say explicitly that no production-symbol impact analysis was required.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 55.1 only: freeze the caller-surface decision ledger in repo-facing docs.

Out of scope:
- Packet 2, 3, or 4 work
- docs/USAGE.md wording reconciliation
- world-backed start wording cleanup beyond what is strictly required to record the decision ledger
- regression-floor or help-text changes
- default-agent UX
- broader non-REPL targeting
- public member-root lifecycle or selector UX widening

Tracker update requirements:
- If implementation or review surfaces new drift, sequencing changes, or newly discovered contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 55.1.
- After implementation, run:
  - `rg -n "default-agent routing|default backend|shell wrap|world-root start|standalone member-root" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-55 / PLAN-55 / TASKS-55 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the repo explicitly distinguishes config/runtime default backend selection from implicit default-agent routing
- the repo explicitly lists allowed prompt-taking surfaces, shell-only surfaces, exact selector rules, world-start wording, and deferred follow-ons at the decision-ledger layer
- no updated doc implies fuzzy or implicit prompt routing

Implementation subagent prompt:
/goal Land Slice 55 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md first. Work only on Task 55.1. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 1 is docs-only and no production-symbol impact run was required. Implement only the minimum doc changes needed to freeze the caller-surface decision ledger. Do not start Packet 2. Run the Packet 1 verification grep plus `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 1 is checkpoint-green, what files changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 55 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and the live diff. Review only Packet 1. Review across correctness, readability, architecture, security, and performance as they apply to a contract/doc slice. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 55 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Fix only the flagged Packet 1 issues without widening scope. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained docs-only. Re-run the Packet 1 verification grep, plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols, or explicitly say Packet 1 stayed docs-only.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 55 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md

Mission:
- Land Slice 55 Packet 2 only: reconcile operator-facing wording for prompt-taking vs shell-wrap surfaces and reconcile host-rooted world-backed start wording against deferred member-root lifecycle.
- Do not start Packet 3.
- Keep the work bounded to Tasks 55.2 and 55.3 only.

Before editing:
1. Re-read SPEC-55, PLAN-55, TASKS-55, the REMAINING tracker note, docs/USAGE.md, AGENT_ORCHESTRATION_GAP_MATRIX.md, and the historical authorities:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. If you end up editing any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius. If Packet 2 stays docs-only, say explicitly that no production-symbol impact analysis was required.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 55.2 only: reconcile operator-facing wording for prompt-taking vs shell-wrap surfaces.
- Task 55.3 only: reconcile host-rooted world-backed start wording against deferred member-root lifecycle.

Out of scope:
- Packet 3 or 4 work
- regression-floor test edits unless a doc contradiction absolutely requires adjacent wording changes
- CLI/help-text Rust changes unless a current user-facing surface plainly contradicts the frozen contract
- default-agent UX
- broader non-REPL targeting
- public member-root selector or lifecycle expansion

Tracker update requirements:
- If implementation or review surfaces new drift, sequencing changes, or unresolved contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Tasks 55.2 and 55.3.
- After implementation, run:
  - `rg -n "substrate -c|--command|piped stdin|shell-wrap|shell execution surfaces rather than agent-prompt aliases" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md`
  - `rg -n "host-rooted world|world-backed start|public world-root start|standalone member-root|born_unattached" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-55 / PLAN-55 / TASKS-55 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- docs/USAGE.md clearly freezes shell-wrap-only surfaces versus prompt-taking surfaces
- touched docs consistently say host-rooted world-backed start exists
- touched docs consistently say standalone member-root public start/continuity does not
- touched docs do not conflate world-backed host-rooted start with public member-root lifecycle

Implementation subagent prompt:
/goal Land Slice 55 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/USAGE.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/20-public-non-interactive-agent-caller-surface.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-30-public-world-scoped-agent-start-and-capability-flags.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-30.md first. Work only on Tasks 55.2 and 55.3. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 2 remained docs-only. Implement only the minimum doc changes needed to align shell-wrap versus prompt-taking wording and host-rooted world-backed start versus deferred member-root lifecycle wording. Do not start Packet 3. Run the two Packet 2 verification greps plus `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 2 is checkpoint-green, what files changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 55 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and the live diff. Review only Packet 2. Review across correctness, readability, architecture, security, and performance as they apply to a contract/doc slice. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 55 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Fix only the flagged Packet 2 issues without widening scope. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained docs-only. Re-run the Packet 2 verification greps, plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols, or explicitly say Packet 2 stayed docs-only.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 55 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs

Mission:
- Land Slice 55 Packet 3 only: preserve the exact public follow-up selector contract and shell-wrap regression floor.
- Do not start Packet 4.
- Keep the work bounded to Task 55.4 only.

Before editing:
1. Re-read SPEC-55, PLAN-55, TASKS-55, the REMAINING tracker note, and inspect the live code/tests in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_public_control_surface_v1.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/repl_world_first_routing_v1.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/cli.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 55.4 only: preserve the exact public follow-up selector contract and shell-wrap regression floor.

Out of scope:
- Packet 4 work
- broad doc cleanup outside what is necessary to keep tests/help text consistent
- default-agent UX
- broader non-REPL targeting
- public member-root lifecycle or selector widening
- runtime redesign unrelated to the regression/help-text contract

Tracker update requirements:
- If implementation or review surfaces new drift, sequencing changes, or contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 55.4.
- After implementation, run:
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-55 / PLAN-55 / TASKS-55 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- public follow-up still requires exact `(--session <orchestration_session_id>, --backend <backend_id>)`
- noncanonical public selectors remain rejected
- shell-wrap regression coverage still proves `substrate -c` is not an agent prompt surface
- no touched test/help text implies implicit backend inference or public member-root selectors

Implementation subagent prompt:
/goal Land Slice 55 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 55.4. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement only the minimum tests/help-text/adjacent runtime changes needed to preserve the exact selector contract and shell-wrap regression floor. Do not start Packet 4. Run `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture` and `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`, then `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 3 is checkpoint-green, what symbols and files changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 55 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and the live diff. Review only Packet 3. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 55 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the Packet 3 verification tests plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 55 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md

Mission:
- Land Slice 55 Packet 4 only: close the validation wall and document what remains deferred.
- This packet assumes Packets 1-3 are already landed and checkpoint-green.
- Keep the work bounded to Task 55.5 only.

Before editing:
1. Re-read SPEC-55, PLAN-55, TASKS-55, the REMAINING tracker note, and the final live diffs from Packets 1-3.
2. Verify Packets 1-3 are already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 55.5 only: close the Slice 55 validation wall and document what remains deferred.

Out of scope:
- reopening Packet 1-3 design decisions
- widening runtime behavior beyond what Packets 1-3 already landed
- default-agent UX
- broader non-REPL targeting
- public member-root lifecycle or selector widening

Tracker update requirements:
- If validation or review surfaces new drift, sequencing changes, or contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 55.5.
- After implementation, run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell --test agent_public_control_surface_v1 -- --nocapture`
  - `cargo test -p shell --test repl_world_first_routing_v1 -- --nocapture`
  - `rg -n "default-agent UX|broader non-REPL targeting|public member-root lifecycle|deferred" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-55 / PLAN-55 / TASKS-55 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- fmt/clippy and targeted caller-surface regression tests are green
- final spec/plan/tasks/docs wording is internally consistent
- Slice 55 clearly states what remains deferred
- the slice remained a contract-freeze/productization slice rather than a runtime redesign

Implementation subagent prompt:
/goal Land Slice 55 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 55.5. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement only the minimum closeout/doc/test/help-text adjustments needed to make the Slice 55 validation wall pass and the deferred follow-ons explicit. Do not reopen Packets 1-3 decisions or widen runtime scope. Run the Packet 4 verification commands, then `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 4 is checkpoint-green, what symbols and files changed, what verification ran, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 55 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and the live diff. Review only Packet 4. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 55 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-55-broader-caller-surface-contract-freeze.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-55.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the Packet 4 verification commands plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State explicitly whether Slice 55 is review-clean and ready for human approval.
- If anything is not green, say explicitly that Slice 55 is not ready to close.
```
