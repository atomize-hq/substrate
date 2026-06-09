/goal Land Packet `R5-6` from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5-6` only, assuming Packets `R5-0` through `R5-5` are already landed.

Packet authority:

- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`
- `AGENTS.md`

Packet `R5-6` scope only:

- add `v0.6` support to sentinel replay input
- add `v0.6` support to sentinel live input / live compatibility checks
- require `session_progress` for `v0.6`
- preserve `v0.2` through `v0.5` compatibility
- render a compact `Progress:` line after `Archetype:` in operator output
- preserve replay/live parity without changing posture / disposition / severity / scheduler behavior

Out of scope:

- analyzer semantic changes
- scheduler or adjudication changes
- acceptance corpus work
- `R5-7`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any function, method, enum, or exported struct, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `R5-6` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5-7`.

Required verification wall for Packet `R5-6`:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R5-6 only from `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5-0 through R5-5 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R5-6`:
- add `v0.6` support to sentinel replay input
- add `v0.6` support to sentinel live input / live compatibility checks
- require `session_progress` for `v0.6`
- preserve `v0.2` through `v0.5` compatibility
- render a compact `Progress:` line after `Archetype:` in operator output
- preserve replay/live parity without changing posture / disposition / severity / scheduler behavior

Read first:
- `AGENTS.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`

Live-code files to inspect before editing:
- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/live_input.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/replay_input.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R5-6`
- keep this presentation / compatibility only
- do not change scheduler behavior, cooldowns, adjudication, posture, disposition, or severity rules
- preserve `v0.2` through `v0.5` compatibility
- do not start acceptance-corpus work or `R5-7`
- run the required verification wall

Return with: changed files, tests run, open risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R5-6 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5-6` from:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`

Focus:
- correctness of `v0.6` replay/live compatibility and requiredness
- preservation of legacy schema support
- correctness and placement of the `Progress:` operator line
- strict refusal to change scheduler / adjudication / posture behavior

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R5-6 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R5-6 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-spec.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-plan.md`
- `docs/specs/r5/agent-drift-analyzer-session-progress-r5-tasks.md`
- `docs/specs/r5/DESIGN-r5-session-progress-contract.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R5-6` issues
- do not broaden into `R5-7`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:

- implementation commit message example: `feat: add v0.6 sentinel progress support`
- fix commit message example: `fix: address r5-6 review findings`

Your job is done only when Packet `R5-6` is committed and a fresh review subagent reports it review-clean.
