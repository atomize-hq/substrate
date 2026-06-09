/goal Land Packet `R5-1` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5-1` only, assuming Packet `R5-0` is already landed.

Packet authority:

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`
- `AGENTS.md`

Packet `R5-1` scope only:

- add public `SessionProgress`, `ProgressStatus`, `ProgressDimension`, `ProgressSignal`, `ProgressSignalCode`, `SignalPolarity`, and `SignalStrength` DTOs
- add optional `Checkpoint.session_progress`
- make `v0.6` require `session_progress` while preserving `v0.2` through `v0.5` compatibility
- emit a temporary conservative `session_progress`
- bump analyzer-created checkpoints to `schema_version = "v0.6"`
- re-export the public DTOs from analyzer public surfaces

Out of scope:

- `attempt.rs`
- `diagnostics.rs`
- `progress.rs`
- real progress semantics beyond the temporary conservative placeholder
- sentinel compatibility or operator presentation
- `R5-2+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R5-1` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5-2`.

Required verification wall for Packet `R5-1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5-1 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet R5-0 is already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5-1`:
- add public `SessionProgress`, `ProgressStatus`, `ProgressDimension`, `ProgressSignal`, `ProgressSignalCode`, `SignalPolarity`, and `SignalStrength` DTOs
- add optional `Checkpoint.session_progress`
- make `v0.6` require `session_progress` while preserving `v0.2` through `v0.5` compatibility
- emit a temporary conservative `session_progress`
- bump analyzer-created checkpoints to `schema_version = "v0.6"`
- re-export the public DTOs from analyzer public surfaces

Read first:
- `AGENTS.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/lib.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R5-1`
- keep the checkpoint DTO legacy-safe at the serde layer
- make the first `session_progress` placeholder explicitly conservative
- do not start `attempt.rs`, `diagnostics.rs`, `progress.rs`, sentinel work, or `R5-2+`
- run the required verification wall

Return with: changed files, tests run, open risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5-1 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5-1` from:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`

Focus:
- correctness of the new public DTOs
- correctness of `v0.6` requiredness and legacy-safe compatibility
- correctness and conservatism of the temporary placeholder progress builder
- packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5-1 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5-1 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5-1` issues
- do not broaden into `R5-2+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `feat: add v0.6 session progress checkpoint dto`
- fix commit message example: `fix: address r5-1 review findings`

Your job is done only when Packet `R5-1` is committed and a fresh review subagent reports it review-clean.
