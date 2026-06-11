/goal Land Packet `R5.5-3` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.5-3` only, assuming Packets `R5.5-0` through `R5.5-2` are already landed.

Packet authority:

- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-command-attempt-and-diagnostic-signature.md

Packet `R5.5-3` scope only:

- expand top-level command-role coverage for the agreed JS/TS verifier matrix
- expand attempt-role coverage for the same JS/TS verifier matrix
- include `npm`, `pnpm`, `yarn`, `vitest`, `npx vitest`, and low-cost `bun test` coverage where the current classifier shape makes that straightforward
- prove the improved classifier changes checkpoint-level progress evidence rather than only low-level enums

Out of scope:

- new troubleshooting semantics beyond `R5.5-1`
- objective extraction logic beyond `R5.5-2`
- real-rollout corpus expansion
- delegation normalization work
- `R5.5-4+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any HIGH or CRITICAL risk warning.
4. The orchestration agent should not implement the packet locally unless the subagent path is unavailable; the default path is delegated implementation.
5. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
6. Commit the landed Packet `R5.5-3` implementation before any review subagent is dispatched.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every fix batch before sending a fresh review subagent back through the review loop.
11. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
12. Run `gitnexus_detect_changes()` before every commit.
13. Do not start Packet `R5.5-4`.

Required verification wall for Packet `R5.5-3`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5.5-3 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `R5.5-0` through `R5.5-2` are already landed

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.5-3`:
- expand top-level command-role coverage for the agreed JS/TS verifier matrix
- expand attempt-role coverage for the same JS/TS verifier matrix
- include `npm`, `pnpm`, `yarn`, `vitest`, `npx vitest`, and low-cost `bun test` coverage where the current classifier shape makes that straightforward
- prove the improved classifier changes checkpoint-level progress evidence rather than only low-level enums

Read first:
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-command-attempt-and-diagnostic-signature.md

Live-code and doc files to inspect before editing:
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/checkpoint/attempt.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R5.5-3`
- run the required verification wall

- do not broaden into adjacent packets or unrelated cleanup

Return with: changed files, tests run, open risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5.5-3 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.5-3` from:
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-command-attempt-and-diagnostic-signature.md

Focus:
- correctness of the JS/TS command-role matrix
- correctness of the JS/TS attempt-role matrix
- proof that checkpoint-level progress benefits from the richer verifier classification
- packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5.5-3 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5.5-3 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-command-attempt-and-diagnostic-signature.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.5-3` issues
- do not broaden into `R5.5-4+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `feat: harden r5.5 js verifier classification`
- fix commit message example: `fix: address r5.5-3 review findings`

Your job is done only when Packet `R5.5-3` is committed and a fresh review subagent reports it review-clean.
