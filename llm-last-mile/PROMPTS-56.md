# PROMPTS-56: Packet Orchestration Prompts For Slice 56

Source spec: [SPEC-56-read-side-and-strict-control-surface-hardening.md](./SPEC-56-read-side-and-strict-control-surface-hardening.md)  
Source plan: [PLAN-56-read-side-and-strict-control-surface-hardening.md](./PLAN-56-read-side-and-strict-control-surface-hardening.md)  
Source tasks: [TASKS-56.md](./TASKS-56.md)  
Source tracker: [REMAINING-overall-scope-2026-06-10.md](./REMAINING-overall-scope-2026-06-10.md)  
Worker implementation skill: `/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill: `/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is grounded only in the live Slice `56` spec/plan/tasks stack and current repo truth. Do not use any `ORCH_PLAN*` file as a reference when running them.

Packet mapping for Slice `56`:

1. **Packet 1** = Task `56.1`
2. **Packet 2** = Task `56.2`
3. **Packet 3** = Tasks `56.3` and `56.4`
4. **Packet 4** = Task `56.5`

## Packet 1 Prompt

```text
/goal Land Slice 56 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md

Mission:
- Land Slice 56 Packet 1 only: freeze the read-side vs strict-control contract and canonical naming.
- Do not start Packet 2.
- Keep the work bounded to Task 56.1 only.

Before editing:
1. Re-read SPEC-56, PLAN-56, TASKS-56, the REMAINING tracker note, and AGENT_ORCHESTRATION_GAP_MATRIX.md.
2. Confirm the packet mapping from PLAN-56:
   - Packet 1 = Task 56.1 only.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. If you end up editing any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius. If Packet 1 remains docs-only, say explicitly that no production-symbol impact analysis was required.
5. Stay strictly within Packet 1 scope.

Packet 1 scope:
- Task 56.1 only: freeze the read-side vs strict-control contract and canonical naming.

Out of scope:
- Packet 2, 3, or 4 work
- degraded status code-path changes
- participant-aware fallback behavior changes
- selector ergonomics like `--current`
- default-agent routing or caller-surface widening
- broad governance cleanup beyond de-canonicalizing legacy-handle semantics in docs/tracker wording

Tracker update requirements:
- If implementation or review surfaces new drift, sequencing changes, or contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 56.1.
- After implementation, run:
  - `rg -n "orchestration_session_id|session_handle_id|active_session_handle_id|fail-closed|degraded|warning" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-56 / PLAN-56 / TASKS-56 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the slice docs explicitly distinguish warning-bearing readable `agent status` from fail-closed control surfaces
- the slice docs explicitly freeze `orchestration_session_id` as the only forward public session handle
- the slice docs explicitly state that `session_handle_id` and `active_session_handle_id` are not forward public contract and, if retained, are compatibility/storage artifacts only
- no updated doc implies selector widening, fuzzy lookup, or control authorization from degraded status

Implementation subagent prompt:
/goal Land Slice 56 Packet 1 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/AGENT_ORCHESTRATION_GAP_MATRIX.md first. Work only on Task 56.1. If GitNexus says the index is stale, run `npx gitnexus analyze` first. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that Packet 1 is docs-only and no production-symbol impact run was required. Implement only the minimum doc/tracker wording changes needed to freeze the read-side vs strict-control contract and canonical naming. Do not start Packet 2. Run the Packet 1 verification grep plus `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 1 is checkpoint-green, what files changed, what verification ran, whether Packet 2 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 56 Packet 1 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and the live diff. Review only Packet 1. Review across correctness, readability, architecture, security, and performance as they apply to a contract/doc slice. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 56 Packet 1 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Fix only the flagged Packet 1 issues without widening scope. If you touch any Rust production symbol, run GitNexus impact analysis before editing it and report the blast radius; otherwise say explicitly that the fix remained docs-only. Re-run the Packet 1 verification grep, plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 56 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs

Mission:
- Land Slice 56 Packet 2 only: normalize degraded `agent status` rendering without weakening strict control helpers.
- Do not start Packet 3.
- Keep the work bounded to Task 56.2 only.

Before editing:
1. Re-read SPEC-56, PLAN-56, TASKS-56, the REMAINING tracker note, and inspect the live code/tests in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs
2. Verify Packet 1 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 2 scope.

Packet 2 scope:
- Task 56.2 only: normalize degraded `agent status` rendering without weakening strict control helpers.

Out of scope:
- Packet 3 or 4 work
- participant-aware fallback identity/suppression cleanup beyond what is strictly necessary to keep Packet 2 coherent
- legacy-handle de-canonicalization beyond what is strictly required to avoid contradiction
- selector ergonomics like `--current`
- caller-surface widening, default-agent routing, or toolbox mutation

Tracker update requirements:
- If implementation or review surfaces new drift, sequencing changes, or unresolved contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 56.2.
- After implementation, run:
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-56 / PLAN-56 / TASKS-56 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- `substrate agent status --json` returns warning-bearing output for the targeted torn/degraded cases rather than aborting the whole read surface
- the degraded warnings explain the specific missing/ambiguous condition instead of silently dropping truth
- strict control-plane helpers remain fail-closed and are not relaxed to share status permissiveness
- no touched code turns degraded status into a control selector source

Implementation subagent prompt:
/goal Land Slice 56 Packet 2 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/state_store.rs, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs first. Work only on Task 56.2. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement only the minimum code/test changes needed to make degraded `agent status` rendering consistent for the targeted torn/degraded cases without weakening strict control helpers. Do not start Packet 3. Run `cargo test -p shell state_store -- --nocapture` and `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`, then `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 2 is checkpoint-green, what symbols and files changed, what verification ran, whether Packet 3 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 56 Packet 2 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and the live diff. Review only Packet 2. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 56 Packet 2 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 2 issues without widening scope. Re-run the Packet 2 verification tests plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 56 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs

Mission:
- Land Slice 56 Packet 3 only: finish participant-aware fallback/suppression/correlation and de-canonicalize legacy handle naming while preserving bounded compatibility if needed.
- Do not start Packet 4.
- Keep the work bounded to Tasks 56.3 and 56.4 only.

Before editing:
1. Re-read SPEC-56, PLAN-56, TASKS-56, the REMAINING tracker note, and inspect the live code/tests in:
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs
   - /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs
2. Verify Packet 2 is already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 3 scope.

Packet 3 scope:
- Task 56.3 only: finish participant-aware fallback/suppression/correlation and keep coarse fallback honest.
- Task 56.4 only: de-canonicalize legacy handle naming while preserving bounded compatibility if needed.

Out of scope:
- Packet 4 work
- broad storage/schema/governance migration
- selector ergonomics like `--current`
- default-agent routing or caller-surface widening
- Family-2 or platform-parity work

Tracker update requirements:
- If implementation or review surfaces new drift, sequencing changes, or contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Tasks 56.3 and 56.4.
- After implementation, run:
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `cargo test -p shell session -- --nocapture`
  - `rg -n "session_handle_id|active_session_handle_id" /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-56 / PLAN-56 / TASKS-56 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- sibling participants remain distinct when `participant_id` / `parent_participant_id` evidence exists
- participant-aware suppression only suppresses matching fallback rows rather than unrelated siblings
- participant-less fallback emits explicit warnings and remains coarse rather than pretending to be exact
- forward-facing status/control/docs/comments treat `orchestration_session_id` as canonical and no longer describe `session_handle_id` / `active_session_handle_id` as meaningful public handles
- if legacy handle names remain in storage or serde aliases, they are explicitly marked as compatibility-only temporary artifacts

Implementation subagent prompt:
/goal Land Slice 56 Packet 3 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agents_cmd.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/session.rs, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/src/execution/agent_runtime/orchestration_session.rs, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/shell/tests/agent_successor_contract_ahcsitc0.rs first. Work only on Tasks 56.3 and 56.4. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement only the minimum code/test/doc/comment changes needed to finish participant-aware fallback behavior and de-canonicalize legacy handle naming without forcing a broad storage/schema migration. Do not start Packet 4. Run `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`, `cargo test -p shell session -- --nocapture`, the legacy-handle grep, then `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 3 is checkpoint-green, what symbols and files changed, what verification ran, whether Packet 4 is unblocked, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 56 Packet 3 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and the live diff. Review only Packet 3. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 56 Packet 3 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 3 issues without widening scope. Re-run the Packet 3 verification commands plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

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
/goal Land Slice 56 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md

Mission:
- Land Slice 56 Packet 4 only: close the Slice 56 validation wall and document any temporary compatibility posture explicitly.
- This packet assumes Packets 1-3 are already landed and checkpoint-green.
- Keep the work bounded to Task 56.5 only.

Before editing:
1. Re-read SPEC-56, PLAN-56, TASKS-56, the REMAINING tracker note, and the final live diffs from Packets 1-3.
2. Verify Packets 1-3 are already landed and checkpoint-green on the current tree.
3. If GitNexus says the index is stale, run `npx gitnexus analyze`.
4. Run GitNexus impact analysis before editing any production symbol you change and report the blast radius.
5. Stay strictly within Packet 4 scope.

Packet 4 scope:
- Task 56.5 only: close the Slice 56 validation wall and document any temporary compatibility posture explicitly.

Out of scope:
- reopening Packet 1-3 design decisions
- selector ergonomics like `--current`
- broad governance/storage migration
- caller-surface widening or default-agent routing
- Family-2 or platform-parity work

Tracker update requirements:
- If validation or review surfaces new drift, sequencing changes, or contradictions in the current remaining-scope story, update /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md before the relevant commit.
- Record tracker items in the correct section instead of leaving them only in chat output.

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 4.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 56.5.
- After implementation, run:
  - `cargo fmt --all -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p shell --test agent_successor_contract_ahcsitc0 -- --nocapture`
  - `cargo test -p shell state_store -- --nocapture`
  - `cargo test -p shell session -- --nocapture`
  - `git diff --stat`
  - `git status --short`
- Run GitNexus detect-changes before committing.
- Commit the Packet 4 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 4 against SPEC-56 / PLAN-56 / TASKS-56 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 4 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, update the tracker if needed, run GitNexus detect-changes again, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 4 checkpoint:
- fmt/clippy and targeted status/control/naming regression coverage are green
- the final spec/plan/tasks/docs wording is internally consistent
- the slice clearly states whether `session_handle_id` / `active_session_handle_id` remain temporarily and in what limited compatibility role
- the slice remained a bounded status/control hardening slice rather than selector ergonomics or broad governance cleanup

Implementation subagent prompt:
/goal Land Slice 56 Packet 4 only in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md first. Work only on Task 56.5. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Implement only the minimum closeout/doc/test/help-text adjustments needed to make the Slice 56 validation wall pass and document any temporary legacy-handle compatibility posture explicitly. Do not reopen Packets 1-3 decisions or widen scope. Run the Packet 4 verification commands, then `git diff --stat` and `git status --short`. Update the tracker note if you surface drift or sequencing changes. Final message must state whether Packet 4 is checkpoint-green, what symbols and files changed, what verification ran, and whether any reopen condition was discovered.

Review subagent prompt:
Review the committed Slice 56 Packet 4 change in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and the live diff. Review only Packet 4. Review across correctness, readability, architecture, security, and performance. Report findings first with explicit severities. State clearly whether Packet 4 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 56 Packet 4 review findings in /Users/spensermcconnell/__Active_Code/atomize-hq/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/SPEC-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/PLAN-56-read-side-and-strict-control-surface-hardening.md, /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/TASKS-56.md, and /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/llm-last-mile/REMAINING-overall-scope-2026-06-10.md. Before editing any production symbol, run GitNexus impact analysis and report the blast radius. If GitNexus says the index is stale, run `npx gitnexus analyze` first. Fix only the flagged Packet 4 issues without widening scope. Re-run the Packet 4 verification commands plus `git diff --stat` and `git status --short`. Update the tracker note if the fixes surface new drift or deferrals. Final message must state which findings were fixed, what verification ran, whether Packet 4 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 4 is checkpoint-green.
- List exact verification commands run and whether they passed.
- Report GitNexus impact-analysis results for edited production symbols.
- Report GitNexus detect-changes results before each commit.
- State whether the tracker note was updated and why.
- State explicitly whether Slice 56 is review-clean and ready for human approval.
- If anything is not green, say explicitly that Slice 56 is not ready to close.
```
