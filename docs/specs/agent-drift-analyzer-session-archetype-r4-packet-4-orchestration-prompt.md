/goal Land Packet `R4-4` from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R4-4` only, assuming Packets
`R4-1` through `R4-3` are already landed.

Packet authority:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `AGENTS.md`

Packet `R4-4` scope only:

- extend sentinel replay input and live compatibility to `v0.5`
- preserve `v0.2` through `v0.4` fallback behavior
- make contract validation explicit about field requiredness by schema version
- update any explicit analyzer-state helper that still hard-codes only `v0.3 | v0.4`

Out of scope:

- analyzer schema/export changes
- archetype derivation logic
- analyzer summary rendering
- replay/live presentation formatting beyond what is required to recognize `v0.5` as
  analyzer-state-backed
- `R4-5+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the
   subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact
   analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results
   yourself before proceeding.
5. Commit the landed Packet `R4-4` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to
   use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt
   must start with `/goal ` and must explicitly instruct the subagent to use the
   `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R4-5`.

Required implementation verification wall for Packet `R4-4`:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R4-4 only from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R4-1 through R4-3 are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `R4-4`:
- extend sentinel replay input and live compatibility to `v0.5`
- preserve `v0.2` through `v0.4` fallback behavior
- make contract validation explicit about field requiredness by schema version
- update any explicit analyzer-state helper that still hard-codes only `v0.3 | v0.4`

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/live_input.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/replay_input.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R4-4`
- preserve legacy-schema behavior while adding `v0.5`
- keep requiredness logic explicit by schema version
- do not start replay/live presentation work, analyzer logic, or `R4-5+`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R4-4 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R4-4` from:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Focus:
- correctness of replay/live `v0.5` compatibility
- preservation of legacy-schema fallback behavior
- explicitness and safety of schema-version requiredness checks
- packet-scope adherence

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R4-4 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R4-4 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R4-4` issues
- do not broaden into Packet `R4-5+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: accept v0.5 archetype checkpoints in sentinel`
- fix commit message example: `fix: address r4-4 review findings`

Your job is done only when Packet `R4-4` is committed and a fresh review subagent reports it
review-clean.
