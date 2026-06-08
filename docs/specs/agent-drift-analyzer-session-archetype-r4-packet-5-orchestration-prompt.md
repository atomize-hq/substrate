/goal Land Packet `R4-5` from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R4-5` only, assuming Packets
`R4-1` through `R4-4` are already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `AGENTS.md`

Packet `R4-5` scope only:

- surface compact archetype inspection in replay/live operator presentation
- keep replay and live presentation aligned for matching checkpoints
- preserve existing posture, trigger labeling, diagnostics rendering, and turn-context output

Out of scope:

- analyzer schema/export changes
- archetype derivation logic
- analyzer summary rendering
- sentinel compatibility logic beyond the presentation seam already unlocked by `R4-4`
- posture/scoring/trigger behavior changes
- `R5+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the
   subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact
   analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results
   yourself before proceeding.
5. Commit the landed Packet `R4-5` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to
   use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt
   must start with `/goal ` and must explicitly instruct the subagent to use the
   `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start any `R5` work.

Required implementation verification wall for Packet `R4-5`:

```bash
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R4-5 only from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R4-1 through R4-4 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R4-5`:
- surface compact archetype inspection in replay/live operator presentation
- keep replay and live presentation aligned for matching checkpoints
- preserve existing posture, trigger labeling, diagnostics rendering, and turn-context output

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R4-5`
- keep the new archetype output compact and presentation-only
- preserve existing posture, trigger-label, diagnostic, and turn-context semantics
- keep replay/live wording aligned unless a spec/task difference requires otherwise
- do not introduce `R5` progress/scoring/warning-policy work
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R4-5 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R4-5` from:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Focus:
- correctness and clarity of replay/live operator presentation
- parity between replay and live surfaces for matching checkpoints
- preservation of posture, trigger labeling, diagnostics, and turn-context output
- packet-scope adherence and refusal to introduce scorer/policy changes

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R4-5 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R4-5 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R4-5` issues
- do not broaden into `R5+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: render r4 archetype operator output`
- fix commit message example: `fix: address r4-5 review findings`

Your job is done only when Packet `R4-5` is committed and a fresh review subagent reports it
review-clean.
