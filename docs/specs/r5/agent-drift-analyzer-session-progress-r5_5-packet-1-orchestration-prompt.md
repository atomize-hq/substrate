/goal Land Packet `R5.5-1` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.5-1` only, assuming Packet `R5.5-0` is already landed.

Packet authority:

- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-session-progress-contract.md
- docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md
- docs/specs/r5/DESIGN-r5-archetype-progress-rules.md

Packet `R5.5-1` scope only:

- add the repeated-signature-plus-overlapping-edit regression for troubleshooting progress
- prevent `FailingScopeEdited` from independently authorizing `advancing` when the same exact or strong-fuzzy failure repeats
- make the direct advancement signal set explicit in the implementation behavior
- add the no-overcorrection guardrails for reduced-failure-count and later-stage-failure advancement

Out of scope:

- objective extraction hardening
- JS/TS verifier-role coverage
- real-rollout corpus expansion
- delegation normalization follow-ons outside the troubleshooting path
- `R5.5-2+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any HIGH or CRITICAL risk warning.
4. The orchestration agent should not implement the packet locally unless the subagent path is unavailable; the default path is delegated implementation.
5. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
6. Commit the landed Packet `R5.5-1` implementation before any review subagent is dispatched.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every fix batch before sending a fresh review subagent back through the review loop.
11. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
12. Run `gitnexus_detect_changes()` before every commit.
13. Do not start Packet `R5.5-2`.

Required verification wall for Packet `R5.5-1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5.5-1 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet `R5.5-0` is already landed

Use the `$incremental-implementation` skill.

You are landing only Packet `R5.5-1`:
- add the repeated-signature-plus-overlapping-edit regression for troubleshooting progress
- prevent `FailingScopeEdited` from independently authorizing `advancing` when the same exact or strong-fuzzy failure repeats
- make the direct advancement signal set explicit in the implementation behavior
- add the no-overcorrection guardrails for reduced-failure-count and later-stage-failure advancement

Read first:
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-session-progress-contract.md
- docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md
- docs/specs/r5/DESIGN-r5-archetype-progress-rules.md

Live-code and doc files to inspect before editing:
- crates/agent-drift-analyzer/src/checkpoint/progress.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R5.5-1`
- run the required verification wall

- do not broaden into adjacent packets or unrelated cleanup

Return with: changed files, tests run, open risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5.5-1 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.5-1` from:
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-session-progress-contract.md
- docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md
- docs/specs/r5/DESIGN-r5-archetype-progress-rules.md

Focus:
- correctness of the direct advancement-signal rule
- correctness of the repeated-signature downgrade path
- proof that the fix does not overcorrect the fewer-failing-tests or later-stage-failure cases
- packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5.5-1 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5.5-1 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-spec.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-plan.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-tasks.md
- docs/specs/r5/agent-drift-analyzer-session-progress-r5_5-planning-input.md
- AGENTS.md
- docs/specs/r5/DESIGN-r5-session-progress-contract.md
- docs/specs/r5/DESIGN-r5-progress-window-and-frontier-model.md
- docs/specs/r5/DESIGN-r5-archetype-progress-rules.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5.5-1` issues
- do not broaden into `R5.5-2+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `feat: harden r5.5 troubleshooting advancement semantics`
- fix commit message example: `fix: address r5.5-1 review findings`

Your job is done only when Packet `R5.5-1` is committed and a fresh review subagent reports it review-clean.
