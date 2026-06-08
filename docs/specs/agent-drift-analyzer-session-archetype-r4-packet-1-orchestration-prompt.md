/goal Land Packet `R4-1` from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R4-1` only.

Packet authority:

- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`
- `AGENTS.md`

Packet `R4-1` scope only:

- add analyzer-owned `SessionArchetype` and `SessionArchetypeLabel` checkpoint schema types
- export `session_archetype` through `agent_drift_analyzer::Checkpoint` with legacy-safe serde for
  `v0.2` through `v0.4`
- widen new analyzer checkpoints to `schema_version = "v0.5"`
- make `v0.5` require `session_archetype` by schema-aware contract without breaking legacy loads

Out of scope:

- deterministic archetype derivation
- analyzer summary rendering
- regression-fixture expansion beyond the minimal contract coverage needed for `R4-1`
- sentinel replay/live compatibility
- replay/live operator presentation
- `R4-2+`

Hard rules:

1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the
   subagent to use the `$incremental-implementation` skill.
3. Before editing any symbol, the implementation subagent must run the required GitNexus impact
   analysis for that symbol and respect any high-risk warning.
4. After the implementation subagent finishes, review its actual diff and verification results
   yourself before proceeding.
5. Commit the landed Packet `R4-1` implementation before any review subagent is dispatched.
6. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
7. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to
   use the `$code-review-and-quality` skill.
8. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt
   must start with `/goal ` and must explicitly instruct the subagent to use the
   `$incremental-implementation` skill.
9. Commit every fix batch before sending a fresh review subagent back through the review loop.
10. Repeat review -> fix -> commit -> fresh review until the packet is review-clean.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R4-2`.

Required implementation verification wall for Packet `R4-1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet R4-1 only from `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

You are landing only Packet `R4-1`:
- add analyzer-owned `SessionArchetype` and `SessionArchetypeLabel` checkpoint schema types
- export `session_archetype` through `agent_drift_analyzer::Checkpoint` with legacy-safe serde for `v0.2` through `v0.4`
- widen new analyzer checkpoints to `schema_version = "v0.5"`
- make `v0.5` require `session_archetype` by schema-aware contract without breaking legacy loads

Read first:
- `AGENTS.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Live-code files to inspect before editing:
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/lib.rs`
- `crates/agent-drift-analyzer/tests/checkpoints.rs`
- `crates/agent-drift-analyzer/tests/end_to_end.rs`

GitNexus requirements:
- before modifying any function, method, enum, or exported struct, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Execution rules:
- stay strictly within Packet `R4-1`
- keep the shared checkpoint DTO legacy-safe at the serde layer
- enforce `session_archetype` requiredness for `v0.5` through schema-aware validation, not by breaking legacy deserialization
- do not start deterministic archetype derivation, summary rendering, sentinel compatibility, or `R4-2+`
- run the required verification wall
- return with: changed files, tests run, any open risks, and the exact commit message you recommend
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet R4-1 implementation in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its spec, plan, and tasks, and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R4-1` from:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Focus:
- correctness of the new checkpoint schema types and `v0.5` cutover
- correctness of legacy-safe serde and version-aware requiredness
- packet-scope adherence
- whether the verification story is sufficient for the schema/export packet

Review the tests first, then the implementation.
List findings by severity.
State clearly whether the packet is review-clean or requires changes.
If changes are required, make them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet R4-1 only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet R4-1 verification commands.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- `docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md`
- `docs/specs/agent-drift-analyzer-session-archetype-r4-tasks.md`

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `R4-1` issues
- do not broaden into Packet `R4-2+`
- run impact analysis before editing affected symbols
- rerun the smallest sufficient verification subset needed to restore confidence
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, tests run, residual risks if any, and the exact commit message you recommend
```

Commit guidance:

- implementation commit message example: `feat: add session archetype checkpoint schema`
- fix commit message example: `fix: address r4-1 review findings`

Your job is done only when Packet `R4-1` is committed and a fresh review subagent reports it
review-clean.
