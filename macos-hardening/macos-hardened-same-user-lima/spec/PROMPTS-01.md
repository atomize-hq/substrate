# PROMPTS-01: Packet Orchestration Prompts For Slice 01

Source spec:
- [`SPEC-01-supported-mode-and-support-taxonomy.md`](./SPEC-01-supported-mode-and-support-taxonomy.md)

Source plan:
- [`PLAN-01.md`](./PLAN-01.md)

Source tasks:
- [`TASKS-01.md`](./TASKS-01.md)

Current branch at prompt authoring time: `HEAD`  
Worker implementation skill:
`/Users/spensermcconnell/.agents/skills/incremental-implementation/SKILL.md`  
Worker review skill:
`/Users/spensermcconnell/.agents/skills/code-review-and-quality/SKILL.md`

These are ready-to-paste prompts for fresh parent sessions. Each prompt is
grounded only in the live Slice `01` spec/plan/tasks stack and the current
feature-local macOS hardening docs.

Because Slice `01` is a docs-and-contract slice, these prompts emphasize:

1. bounded document scope,
2. terminology consistency,
3. explicit deferrals,
4. commit discipline between implementation, review, and fix rounds.

## Packet 1 Prompt

```text
/goal Land Slice 01 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md

Mission:
- Land Packet 1 only: Supported-mode contract freeze.
- Do not start Packet 2.
- Keep the slice bounded to freezing the supported same-user Lima posture and making the Linux non-parity claims impossible to miss.

Before editing:
1. Read SPEC-01, PLAN-01, TASKS-01, the Phase 0 README, milestone 0.1, and DESIGN-supported-mode-and-breakglass-taxonomy.md.
2. Inspect the current feature-local docs in:
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/README.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/README.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md
   - /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-supported-mode-and-breakglass-taxonomy.md
3. Stay within the default execution boundary. Do not widen into top-level repo docs unless a direct contradiction forces it and you call that out explicitly.

Packet 1 scope:
- Task 1.1: Confirm the slice authority stack and freeze the posture language.
- Task 1.2: Make the Linux non-parity claims impossible to miss.

Out of scope:
- Packet 2 or Packet 3 work
- Lima version-floor freeze
- transport, policy, mount, unit, or lifecycle implementation details
- top-level repo-wide macOS docs unless an explicit contradiction forces them into scope

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 1.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 1.1 and Task 1.2.
- After implementation, run the Packet 1 verification commands:
  - `sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-1-target-mode-and-support-contract-sow.md`
  - `rg -n "host-side ownership|privilege boundary|direct guest|normal operator path" macos-hardening/macos-hardened-same-user-lima`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima`
  - `git status --short`
- If implementation is green, commit the Packet 1 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 1 against SPEC-01 / PLAN-01 / TASKS-01 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 1 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 1 checkpoint:
- the supported same-user posture is explicit
- Linux non-parity claims are explicit
- no version-floor or transport-specific claims were frozen by accident
- the touched-doc set stayed within the default execution boundary unless an expansion was explicitly justified

Implementation subagent prompt:
/goal Land Slice 01 Packet 1 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md first. Also read the feature README, the Phase 0 README, milestone 0.1, and DESIGN-supported-mode-and-breakglass-taxonomy.md. Work only on Task 1.1 and Task 1.2. Keep the slice docs-and-contract only. Do not widen into Slice 02 or later semantics. Default to feature-local docs only. Run the Packet 1 verification commands and finish by stating whether Packet 1 is checkpoint-green, what files changed, what verification ran, and whether Packet 2 is unblocked.

Review subagent prompt:
Review the committed Slice 01 Packet 1 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md. Review only Packet 1 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to contract docs. Report findings first with explicit severities. State clearly whether Packet 1 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 01 Packet 1 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md. Fix only the flagged Packet 1 issues without widening scope. Re-run the relevant Packet 1 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 1 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 1 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the touched-doc set stayed within the default execution boundary.
- State whether Packet 2 is unblocked.
- If anything is not green, say explicitly that Packet 2 must not begin.
```

## Packet 2 Prompt

```text
/goal Land Slice 01 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md

Mission:
- Land Packet 2 only: Support taxonomy freeze and feature-local propagation.
- Do not start Packet 3.
- Keep the slice bounded to freezing the `supported` / `degraded-but-supported` / `breakglass` taxonomy and propagating it through the visible feature-local docs.

Before editing:
1. Read SPEC-01, PLAN-01, TASKS-01, then verify Packet 1 is already landed and checkpoint-green on the current tree.
2. Re-read DESIGN-supported-mode-and-breakglass-taxonomy.md and the current feature README, ROADMAP.md, EXECUTION-RUBRIC.md, and Phase 0 docs.
3. Stay within the default feature-local docs boundary unless a contradiction forces escalation.

Packet 2 scope:
- Task 2.1: Freeze the support taxonomy terms and examples.
- Task 2.2: Align the roadmap and phase-local wording to the taxonomy.

Out of scope:
- Packet 3 work
- version-floor or breakglass-semantic freeze that belongs to Slice 02
- transport, policy, mount, unit, or lifecycle details
- top-level repo docs unless explicitly justified

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 2.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 2.1 and Task 2.2.
- After implementation, run the Packet 2 verification commands:
  - `rg -n "supported|degraded-but-supported|breakglass" macos-hardening/macos-hardened-same-user-lima`
  - `rg -n "supported|degraded-but-supported|breakglass" macos-hardening/macos-hardened-same-user-lima/ROADMAP.md macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima`
  - `git status --short`
- If implementation is green, commit the Packet 2 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 2 against SPEC-01 / PLAN-01 / TASKS-01 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 2 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 2 checkpoint:
- the support-taxonomy terms are consistent
- feature-local execution docs use the same taxonomy
- the slice remains feature-local and docs-first
- the next-slice handoff language still points cleanly to Slice 02

Implementation subagent prompt:
/goal Land Slice 01 Packet 2 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md first. Verify Packet 1 is already green. Work only on Task 2.1 and Task 2.2. Keep the edits feature-local, docs-only, and taxonomy-focused. Run the Packet 2 verification commands and finish by stating whether Packet 2 is checkpoint-green, what files changed, what verification ran, and whether Packet 3 is unblocked.

Review subagent prompt:
Review the committed Slice 01 Packet 2 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md. Review only Packet 2 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to contract docs. Report findings first with explicit severities. State clearly whether Packet 2 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 01 Packet 2 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md. Fix only the flagged Packet 2 issues without widening scope. Re-run the relevant Packet 2 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 2 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 2 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether the taxonomy is consistent across the touched feature-local docs.
- State whether Packet 3 is unblocked.
- If anything is not green, say explicitly that Packet 3 must not begin.
```

## Packet 3 Prompt

```text
/goal Land Slice 01 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate.

Use these source docs as authority:
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md
- /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/EXECUTION-RUBRIC.md

Mission:
- Land Slice 01 Packet 3 only: Validation, explicit deferrals, and next-slice readability.
- Do not reopen Packet 1 or Packet 2 except where final wording cleanup is strictly required.
- Keep the slice bounded to explicit deferrals, coherence review, and leaving the short-prompt planning path clean for Slice 02.

Before editing:
1. Read SPEC-01, PLAN-01, TASKS-01, then verify Packets 1 and 2 are already landed and checkpoint-green on the current tree.
2. Re-read EXECUTION-RUBRIC.md, ROADMAP.md, and the full Slice 01 doc set.
3. Confirm the default next seam is still Slice 02 and that no hidden dependency inversion has appeared.

Packet 3 scope:
- Task 3.1: Validate explicit deferrals to later slices.
- Task 3.2: Final diff and coherence review.

Out of scope:
- any new support-taxonomy rewrite unless needed for final coherence
- version-floor freeze work
- transport, mount, unit, lifecycle, or implementation-bearing semantics
- top-level repo docs unless explicitly justified

Execution requirements:
- Spawn a fresh GPT-5.4 subagent on high to implement Packet 3.
- The implementation subagent prompt must begin with `/goal ` and must instruct the subagent to use `$incremental-implementation`.
- The implementation subagent must work only on Task 3.1 and Task 3.2.
- After implementation, run the Packet 3 verification commands:
  - `rg -n "Slice \`02\`|version floor|transport|policy|mount|unit|lifecycle" macos-hardening/macos-hardened-same-user-lima`
  - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima`
  - `git status --short`
  - manual coherence review of SPEC-01 / PLAN-01 / TASKS-01 / EXECUTION-RUBRIC.md / ROADMAP.md
- If implementation is green, commit the Packet 3 implementation work before review.

Review requirements:
- Spawn a fresh GPT-5.4 subagent on high using `$code-review-and-quality`.
- The review subagent must review only Packet 3 against SPEC-01 / PLAN-01 / TASKS-01 and the live diff.
- If review finds issues, spawn a fresh GPT-5.4 high fix subagent whose prompt begins with `/goal ` and uses `$incremental-implementation`.
- The fix subagent must stay limited to the review findings and Packet 3 scope.
- After fixes, rerun the relevant verification commands, run `git diff --stat` and `git status --short`, commit the fixes, and then rerun a fresh GPT-5.4 high `$code-review-and-quality` review.
- Repeat until review-clean.

Commit policy:
- Commit after implementation before review.
- Commit after each fix round before re-review.
- Do not amend unless absolutely required.

Packet 3 checkpoint:
- Slice 01 stayed bounded
- Slice 02 remains legible as the next seam
- the feature-local planning stack is coherent enough for a short future prompt to continue the sequence
- PROMPTS-01.md is consistent with the packet boundaries and review/fix flow

Implementation subagent prompt:
/goal Land Slice 01 Packet 3 only in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md first. Verify Packets 1 and 2 are already green. Work only on Task 3.1 and Task 3.2. Keep the slice bounded to explicit deferrals, coherence, and next-slice readability. Re-run the Packet 3 verification commands and finish by stating whether Packet 3 is checkpoint-green, what files changed, what verification ran, and whether Slice 02 is unblocked.

Review subagent prompt:
Review the committed Slice 01 Packet 3 change in /Users/spensermcconnell/.codex/worktrees/ff74/substrate using $code-review-and-quality. Ground the review in /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md. Review only Packet 3 and the live diff. Review across correctness, readability, architecture, security, and performance as they apply to contract docs and planning artifacts. Report findings first with explicit severities. State clearly whether Packet 3 is review-clean or requires changes.

Fix subagent prompt:
/goal Address only the required Slice 01 Packet 3 review findings in /Users/spensermcconnell/.codex/worktrees/ff74/substrate. Use $incremental-implementation. Re-read the review findings plus /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/SPEC-01-supported-mode-and-support-taxonomy.md, /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/PLAN-01.md, and /Users/spensermcconnell/.codex/worktrees/ff74/substrate/macos-hardening/macos-hardened-same-user-lima/spec/TASKS-01.md. Fix only the flagged Packet 3 issues without widening scope. Re-run the relevant Packet 3 verification commands. Final message must state which findings were fixed, what verification ran, whether Packet 3 is checkpoint-green, and whether another review round is required.

Final response requirements:
- State whether Packet 3 is checkpoint-green.
- List exact verification commands run and whether they passed.
- State whether Slice 02 is clearly unblocked as the next seam.
- State whether PROMPTS-01.md remains consistent with the landed packet boundaries.
- If anything is not green, say explicitly that Slice 02 planning must not begin.
```
