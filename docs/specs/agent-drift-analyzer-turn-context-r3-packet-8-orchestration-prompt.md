/goal Land Packet `R3-8` from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3-8` only, assuming Packets `R3-1` through `R3-7` are already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`
- `AGENTS.md`

Packet `R3-8` scope only:

- surface compact turn-context inspection in replay/live operator presentation
- keep replay/live turn-context presentation aligned
- keep posture, severity, trigger, scheduler, and warning-policy behavior unchanged

Out of scope:

- compatibility changes already owned by Packet `R3-7`
- archetype, progress, or scorer changes
- any broader sentinel redesign

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3-8` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not broaden beyond Packet `R3-8`.

Required implementation verification wall for Packet `R3-8`:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3-8 only from `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R3-1 through R3-7 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3-8`:
- surface compact turn-context inspection in replay/live operator presentation
- keep replay/live turn-context presentation aligned
- keep posture, severity, trigger, scheduler, and warning-policy behavior unchanged

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3-8`
- keep turn-context presentation compact and readable
- do not alter posture classification, trigger selection, scheduler policy, or warning thresholds
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3-8 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3-8` from:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Focus:
- correctness of replay/live turn-context presentation
- preservation of posture and warning behavior
- parity between replay and live surfaces
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3-8 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3-8 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-turn-context-r3-spec.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-plan.md`
- `docs/specs/agent-drift-analyzer-turn-context-r3-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3-8` issues
- do not broaden beyond Packet `R3-8`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: surface turn context in sentinel output`
- fix commit message example: `fix: address r3-8 review findings`

Your job is done only when Packet `R3-8` is committed and a fresh review subagent reports it review-clean.
