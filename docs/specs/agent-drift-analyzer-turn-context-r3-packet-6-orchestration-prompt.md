/goal Land Packet `R3-6` from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3-6` only, assuming Packets `R3-1` through `R3-5` are already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
- `AGENTS.md`

Packet `R3-6` scope only:

- add analyzer regression coverage for long-turn and many-short-turn shapes
- prove reruns preserve identical `turn_context`
- keep checkpoint artifacts session-scoped and ordinal-stable

Out of scope:

- new analyzer semantics
- sentinel compatibility or presentation
- `R3-7+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3-6` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R3-7`.

Required implementation verification wall for Packet `R3-6`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3-6 only from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R3-1 through R3-5 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3-6`:
- add analyzer regression coverage for long-turn and many-short-turn shapes
- prove reruns preserve identical `turn_context`
- keep checkpoint artifacts session-scoped and ordinal-stable

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`
- `crates/agent-drift-analyzer/tests/support/mod.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3-6`
- prefer deterministic fixtures and assertions over broad new harness work
- do not broaden into sentinel or `R3-7+`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3-6 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3-6` from:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Focus:
- adequacy of long-turn vs many-short-turn coverage
- determinism of the new assertions
- preservation of session-local and ordinal-stable artifact behavior
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3-6 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3-6 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3-6` issues
- do not broaden into Packet `R3-7+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `test: add r3 turn-context regression coverage`
- fix commit message example: `fix: address r3-6 review findings`

Your job is done only when Packet `R3-6` is committed and a fresh review subagent reports it review-clean.
