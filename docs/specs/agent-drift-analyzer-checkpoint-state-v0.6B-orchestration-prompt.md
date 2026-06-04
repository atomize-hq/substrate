/goal Land Packet `v0.6B` from `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `v0.6B` only, assuming Packet `v0.6A` is already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `AGENTS.md`

Packet `v0.6B` scope only:

- cut sentinel posture over to analyzer-exported state with `v0.2` fallback
- prove replay/live parity survives the checkpoint-state cutover
- refresh packet authority notes and bounded live proof guidance

Out of scope:

- Packet `v0.6A` implementation work except narrowly required follow-up adjustments
- typed outcome evidence / `ToolOutput` redesign
- reopening `v0.5A` or `v0.5B`
- scheduler redesign, sink-family redesign, or unrelated cleanup

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results yourself before proceeding.
5. Commit the landed Packet `v0.6B` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not broaden into the typed outcome evidence follow-on.

Required implementation verification wall for Packet `v0.6B`:

```bash
cargo test -p agent-drift-analyzer truth_grounding_gap -- --nocapture
cargo test -p agent-drift-analyzer dead_end_thrash -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel operator_sink -- --nocapture
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel -- --nocapture
```

Required bounded live proof:

```bash
export SESSION_ID="<active-session-id>"
export CODEX_HOME="${CODEX_HOME:-$HOME/.codex}"
export LIVE_STATE_DIR="target/hybrid-drift-live/$SESSION_ID"

sh -c 'cargo run -p agent-drift-sentinel -- --mode live --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --checkpoint-dir "$LIVE_STATE_DIR" & pid=$!; sleep 8; kill "$pid" 2>/dev/null || true; wait "$pid"'
```

Implementation subagent prompt to send:

```text
/goal Implement Packet v0.6B only from `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet v0.6A is already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `v0.6B`:
- cut sentinel posture over to analyzer-exported state with `v0.2` fallback
- prove replay/live parity survives the checkpoint-state cutover
- refresh packet authority notes and bounded live proof guidance

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

Live-code files to inspect before editing:
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/src/operator_sink.rs`
- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/live_input.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/operator_sink.rs`
- `crates/agent-drift-sentinel/tests/replay_input.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
- `docs/specs/hybrid-drift-sentinel-implementation-order.md`
- `docs/internals/testing/hybrid-drift-stack-smoke-guide.md`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `v0.6B`
- assume Packet `v0.6A` is already landed; do not reopen its design unless required for narrow compatibility repair
- prefer explicit analyzer-exported state for `v0.3`, keep `v0.2` fallback isolated
- do not broaden into typed outcome evidence or coordinator redesign
- run the full Packet `v0.6B` verification wall
- run the bounded live proof if an actually growing session is available; if not, state exactly what blocked that proof
- return with: changed files, tests run, proof status, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet v0.6B implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `v0.6B` from:
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`

Focus:
- correctness of sentinel cutover to analyzer-exported state
- correctness and containment of `v0.2` fallback behavior
- replay/live parity and bounded proof story
- architecture fit with analyzer-owned semantics and sentinel presentation-only posture
- strict packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet v0.6B only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet v0.6B verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-spec.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-plan.md`
- `docs/specs/agent-drift-analyzer-checkpoint-state-v0.6-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `v0.6B` issues
- do not broaden into typed outcome evidence or reopen unrelated Packet `v0.5A` / `v0.5B` work
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset, plus any broader Packet `v0.6B` checks needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, proof status, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: land sentinel checkpoint state cutover v0.6b`
- fix commit message example: `fix: address packet v0.6b review findings`

Your job is not done when code lands. Your job is done only when Packet `v0.6B` is committed and a fresh review subagent reports it review-clean.
