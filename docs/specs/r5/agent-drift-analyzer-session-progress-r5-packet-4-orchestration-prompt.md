/goal Land Packet `R5-4` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5-4` only, assuming Packets `R5-0` through `R5-3` are already landed.

Packet authority:

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md`
- `docs/specs/r5/DESIGN-r5-archetype-progress-rules.md`
- `docs/specs/r5/DESIGN-r5-delegation-progress-guardrails.md`
- `AGENTS.md`

Packet `R5-4` scope only:

- add internal `checkpoint/progress.rs`
- derive `SessionProgress` from R4 archetype plus attempts/signatures/repetition/recovery/task-frame delta/delegation
- implement troubleshooting frontier first
- implement conservative planning / implementation / closeout progress rules
- apply delegation caps and `ParentVisibleOrchestration` fallback
- replace the temporary `R5-1` placeholder progress builder

Out of scope:

- analyzer summary formatting
- sentinel compatibility
- acceptance corpus work
- scorer retuning
- `R5-5+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R5-4` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5-5`.

Required verification wall for Packet `R5-4`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5-4 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5-0 through R5-3 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5-4`:
- add internal `checkpoint/progress.rs`
- derive `SessionProgress` from R4 archetype plus attempts/signatures/repetition/recovery/task-frame delta/delegation
- implement troubleshooting frontier first
- implement conservative planning / implementation / closeout progress rules
- apply delegation caps and `ParentVisibleOrchestration` fallback
- replace the temporary `R5-1` placeholder progress builder

Read first:
- `AGENTS.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md`
- `docs/specs/r5/DESIGN-r5-archetype-progress-rules.md`
- `docs/specs/r5/DESIGN-r5-delegation-progress-guardrails.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/attempt.rs`
- `crates/agent-drift-analyzer/src/checkpoint/diagnostics.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R5-4`
- troubleshooting frontier is first priority, but planning / implementation / closeout rules must still remain conservative and evidence-backed
- non-`insufficient_evidence` statuses must carry supporting evidence
- opaque delegated-parent cases must never claim child progress
- do not retune drift scorers
- do not start summary work, sentinel work, acceptance-corpus work, or `R5-5+`
- run the required verification wall

Return with: changed files, tests run, open risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5-4 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, tasks, fixtures, and DESIGN docs, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5-4` from:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md`
- `docs/specs/r5/DESIGN-r5-archetype-progress-rules.md`
- `docs/specs/r5/DESIGN-r5-delegation-progress-guardrails.md`

Focus:
- correctness of dimension selection and packet-local progress aggregation
- correctness of troubleshooting frontier movement and regression detection
- conservatism of planning / implementation / closeout rules
- delegation caps and evidence limiting
- strict refusal to retune scorers or start `R5-5+`

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5-4 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5-4 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-fixtures.md`
- `docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md`
- `docs/specs/r5/DESIGN-r5-archetype-progress-rules.md`
- `docs/specs/r5/DESIGN-r5-delegation-progress-guardrails.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5-4` issues
- do not broaden into `R5-5+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `feat: add r5 session progress engine`
- fix commit message example: `fix: address r5-4 review findings`

Your job is done only when Packet `R5-4` is committed and a fresh review subagent reports it review-clean.
