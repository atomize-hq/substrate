/goal Land Packet `R3.5-3` from `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R3.5-3` only, assuming Packets `R3.5-1` and `R3.5-2` are already landed.

Packet authority:

- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-plan.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md`
- `AGENTS.md`

Packet `R3.5-3` scope only:

- preserve synthetic repeated-failure labeling
- keep recovered and historical-only checkpoint posture rendering intact after the headline cutover
- lock replay/live parity coverage for ordinary checkpoint presentation and true fast-path events

Out of scope:

- analyzer schema, scorer, or posture changes
- scheduler-policy or warning-threshold changes
- broader replay/live interpretation consolidation
- `R4+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any HIGH or CRITICAL risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R3.5-3` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start `R4`.

Required implementation verification wall for Packet `R3.5-3`:

```bash
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel live_runtime -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R3.5-3 only from `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R3.5-1 and R3.5-2 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R3.5-3`:
- preserve synthetic repeated-failure labeling
- keep recovered and historical-only checkpoint posture rendering intact after the headline cutover
- lock replay/live parity coverage for ordinary checkpoint presentation and true fast-path events

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-plan.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-sentinel/src/live_runtime.rs`
- `crates/agent-drift-sentinel/tests/operator_sink.rs`
- `crates/agent-drift-sentinel/tests/live_runtime.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R3.5-3`
- preserve the distinction between ordinary checkpoint presentation and synthetic scheduler fast-path events
- do not broaden into analyzer semantics or `R4+`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R3.5-3 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R3.5-3` from:
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-plan.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md`

Focus:
- preservation of synthetic repeated-failure labeling
- correctness of replay/live proof coverage after the headline cutover
- preservation of recovered and historical-only posture rendering
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R3.5-3 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R3.5-3 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-spec.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-plan.md`
- `docs/specs/agent-drift-sentinel-trigger-headline-canonicalization-r3_5-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R3.5-3` issues
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- do not broaden into `R4+`
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `test: preserve r3.5 scheduler fast-path labeling`
- fix commit message example: `fix: address r3.5-3 review findings`

Your job is done only when Packet `R3.5-3` is committed and a fresh review subagent reports it review-clean.
