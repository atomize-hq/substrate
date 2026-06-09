/goal Land Packet `R5-0` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5-0` only.

Packet authority:

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`
- `docs/specs/r5/DESIGN-r5-command-attempt-and-diagnostic-signature.md`
- `docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md`
- `docs/specs/r5/DESIGN-r5-archetype-progress-rules.md`
- `docs/specs/r5/DESIGN-r5-delegation-progress-guardrails.md`
- `docs/specs/r5/DESIGN-r5-validation-and-rollout-protocol.md`
- `AGENTS.md`

Packet `R5-0` scope only:

- finalize the canonical R5 DESIGN docs under `docs/specs/r5/`
- finalize the R5 spec / plan / tasks / fixtures docs under `docs/specs/r5/`
- lock the packet split, non-goals, verification story, and fixture-manifest direction

Out of scope:

- any Rust code changes
- any test-fixture or harness creation
- any analyzer or sentinel implementation
- `R5-1+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. This packet is docs-only. Do not allow the implementation or fix subagents to touch Rust code, tests, or fixture directories.
4. After the implementation subagent finishes, review its actual doc diff yourself before proceeding.
5. Commit the landed Packet `R5-0` doc changes before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Do not start Packet `R5-1`.

Required verification wall for Packet `R5-0`:

```bash
# Manual doc review only.
```

Implementation subagent prompt to send:

```text
/goal Finalize Packet R5-0 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5-0`:
- finalize the canonical R5 DESIGN docs under `docs/specs/r5/`
- finalize the R5 spec / plan / tasks / fixtures docs under `docs/specs/r5/`
- lock the packet split, non-goals, verification story, and fixture-manifest direction

Read first:
- `AGENTS.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- all six `docs/specs/r5/DESIGN-r5-*.md` docs

Execution rules:
- stay strictly within Packet `R5-0`
- make docs consistent with each other and current repo reality
- do not touch Rust code, tests, or fixture directories
- do not start `R5-1+`

Return with: changed files, any unresolved doc tensions, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5-0 doc implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, tasks, fixture manifest, and DESIGN docs, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5-0` from:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- all six `docs/specs/r5/DESIGN-r5-*.md` docs

Focus:
- internal consistency across the R5 doc stack
- repo-truth alignment
- packet-scope clarity
- whether the docs are safe to hand to an implementation agent

List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5-0 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- all six `docs/specs/r5/DESIGN-r5-*.md` docs

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5-0` doc issues
- do not touch Rust code, tests, or fixture directories
- do not broaden into `R5-1+`

Return with: exact fixes made and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `docs: lock r5 progress packet plan`
- fix commit message example: `fix: address r5-0 review findings`

Your job is done only when Packet `R5-0` is committed and a fresh review subagent reports it review-clean.
