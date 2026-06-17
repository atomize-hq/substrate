# SO-2.3B-refine Packet Prompts

Status: draft orchestration prompts created on 2026-06-17 for the `SO-2.3B-refine` packet family.

These prompts treat each open task in
`docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md`
as one sequential packet prompt:

- Packet 1 -> Task `B1.1`
- Packet 2 -> Task `B2.1`
- Packet 3 -> Task `B2.2`
- Packet 4 -> Task `B3.1`
- Packet 5 -> Task `B3.2`
- Packet 6 -> Task `B4.1`
- Packet 7 -> Task `B5.1`
- Packet 8 -> Task `B5.2`

Already-landed docs-lock task `B0.1` is excluded.

Global rules for every packet prompt below:

1. The parent session is the orchestration agent; it should not do packet work locally unless
   delegated execution is unavailable.
2. The orchestration agent must spawn a fresh `GPT-5.4` subagent on `high` for implementation
   first.
3. Every implementation or fix subagent prompt must start with `/goal ` and must explicitly tell
   the subagent to use the `$incremental-implementation` skill.
4. After the implementation work is landed, the orchestration agent must commit before dispatching
   review. If a packet is verification-only and no files changed, do not invent an empty commit;
   instead record honestly that no implementation commit was needed.
5. The orchestration agent must then spawn a fresh `GPT-5.4` subagent on `high` for review.
6. Every review subagent prompt must start with `/goal ` and must explicitly tell the subagent to
   use the `$code-review-and-quality` skill.
7. If the review subagent flags issues, the orchestration agent must spawn a fresh `GPT-5.4`
   `high` fix subagent whose prompt starts with `/goal ` and explicitly uses the
   `$incremental-implementation` skill.
8. Commit every non-empty implementation/fix batch before sending a fresh review subagent back
   through the loop. Do not advance to the next packet until the current packet is committed and a
   fresh review subagent reports it review-clean, or for verification-only packets with no file
   changes, explicitly reports there was nothing to commit.
9. Run `gitnexus_detect_changes()` before every real commit.
10. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the
    implementation/fix subagent must honor the repo GitNexus rule: run impact analysis first and
    report any HIGH or CRITICAL blast radius before proceeding.

Shared authority for all packets:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md
---
## Packet 1 Prompt — Task B1.1

````text
/goal Land Packet `B1.1` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B1.1` only, assuming the docs-lock task `B0.1` is already landed and no later `B2.*+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B1.1` scope only:
- stop `checkpoint_analyses(...)` from overwriting a richer structured objective with `ObjectiveSummary::compatibility(...)`
- preserve the richer `assemble_context(&window).objective` when it already carries `structured`
- treat legacy narrowed summary behavior as fallback-only; if needed for display, it must not drop `structured`, `verification_commands`, `unknowns`, or evidence spans
- do not default to re-running objective extraction over a different row slice unless equivalence and preservation are explicitly proven
- land the focused regression in the same packet proving `analysis.current.context.objective.structured.is_some()` survives narrowing and keeps section/clause grounding

Primary files for this packet:
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- target-evidence tightening from `B2.*` except for the minimum bridge behavior needed to keep this packet honest
- verification-grounding changes from `B3.*`
- compatibility rendering / `comparison_key` derivation from `SO-3.*`
- `objective_acceptance` harness or fixture work from `SO-4.*` / `SO-5.*`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B1.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B1.1` is review-clean.
13. Do not let the implementation miss the actual defect: the bug is bridge overwrite, not “structured extraction cannot produce structure.”

Required verification wall for Packet `B1.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B1.1` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the docs-lock task `B0.1` is already landed and no later `B2.*+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B1.1`:
- stop `checkpoint_analyses(...)` from overwriting a richer structured objective with `ObjectiveSummary::compatibility(...)`
- preserve the richer `assemble_context(&window).objective` when it already carries `structured`
- treat legacy narrowed summary behavior as fallback-only; if needed for display, it must not drop `structured`, `verification_commands`, `unknowns`, or evidence spans
- do not default to re-running objective extraction over a different row slice unless equivalence and preservation are explicitly proven
- land the focused regression in the same packet proving `analysis.current.context.objective.structured.is_some()` survives narrowing and keeps section/clause grounding

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B1.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- treat the richer objective produced by `assemble_context(&window)` as the semantic authority when
  it already has `structured`; do not replace it with compatibility-only state
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B1.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B1.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B1.1` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether the narrowing bridge preserves the richer `assemble_context(&window).objective` when it
  already has `structured`, instead of collapsing back to compatibility-only state
- whether any legacy narrowed text is layered only as display fallback without dropping
  `verification_commands`, `unknowns`, or evidence spans
- whether the packet includes the proving regression in the same packet rather than deferring proof
- whether the regression would fail if checkpoint narrowing regressed back to text-only behavior

Review the new bridge-preservation regression first, then the implementation diff.
List findings by severity.
State clearly whether Packet `B1.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B1.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B1.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B1.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- preserve the richer objective from `assemble_context(&window)` when it already contains
  `structured`; do not reintroduce compatibility-only overwrite or re-extraction-default behavior
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B1.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: preserve structured objective through checkpoint narrowing`
- fix commit message example: `fix: address b1.1 review findings`

Your job is done only when Packet `B1.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 2 Prompt — Task B2.1

````text
/goal Land Packet `B2.1` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B2.1` only, assuming Packet `B1.1` is already landed and no later `B2.2+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B2.1` scope only:
- separate grounded goal selection from explicit target extraction
- leave `target == None` plus `ObjectiveUnknown { field_name: "target", ... }` when no explicit target evidence exists
- land the vague-target regression in the same packet so the unknown behavior is proven before review

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- explicit-target preservation matrix from `B2.2` beyond the minimum needed to keep this packet safe
- verification-grounding work from `B3.*`
- compatibility rendering / `comparison_key` work from `SO-3.*`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B2.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B2.1` is review-clean.

Required verification wall for Packet `B2.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B2.1` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet `B1.1` is already landed and no later `B2.2+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B2.1`:
- separate grounded goal selection from explicit target extraction
- leave `target == None` plus `ObjectiveUnknown { field_name: "target", ... }` when no explicit target evidence exists
- land the vague-target regression in the same packet so the unknown behavior is proven before review

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B2.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B2.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B2.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B2.1` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether vague review/analyze/fix prompts now keep `target` unknown instead of fabricating a conceptual topic
- whether the packet includes the vague-target proving regression in the same packet
- whether the packet stayed scoped to target honesty rather than widening into compatibility or verifier work

Review the new vague-target regression first, then the implementation diff.
List findings by severity.
State clearly whether Packet `B2.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B2.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B2.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B2.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B2.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: stop structured objective target fabrication`
- fix commit message example: `fix: address b2.1 review findings`

Your job is done only when Packet `B2.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 3 Prompt — Task B2.2

````text
/goal Land Packet `B2.2` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B2.2` only, assuming Packets `B1.1` and `B2.1` are already landed and no later `B3.*+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B2.2` scope only:
- preserve explicit file/directory, instruction-surface, spec/doc, test/verifier, crate/package, and workspace-ref targets after the unknown gate tightening
- land the explicit-target preservation regression matrix in the same packet
- treat this packet as preservation work, not as a reopening of vague-target fabrication behavior

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- new target classes not already implied by the phase-1 docs
- verification-grounding changes from `B3.*`
- compatibility rendering / `comparison_key` work from `SO-3.*`
- `objective_acceptance` harness or fixture work

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B2.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B2.2` is review-clean.

Required verification wall for Packet `B2.2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B2.2` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `B1.1` and `B2.1` are already landed and no later `B3.*+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B2.2`:
- preserve explicit file/directory, instruction-surface, spec/doc, test/verifier, crate/package, and workspace-ref targets after the unknown gate tightening
- land the explicit-target preservation regression matrix in the same packet
- treat this packet as preservation work, not as a reopening of vague-target fabrication behavior

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B2.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B2.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B2.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B2.2` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether explicit concrete targets still resolve correctly after the unknown gate was tightened
- whether the explicit-target preservation matrix is present in the same packet
- whether the packet preserves instruction-surface and workspace-ref style targets without reopening vague target fabrication

Review the explicit-target preservation matrix first, then the implementation diff.
List findings by severity.
State clearly whether Packet `B2.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B2.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B2.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B2.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B2.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: preserve explicit structured objective targets`
- fix commit message example: `fix: address b2.2 review findings`

Your job is done only when Packet `B2.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 4 Prompt — Task B3.1

````text
/goal Land Packet `B3.1` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B3.1` only, assuming Packets `B1.1` through `B2.2` are already landed and no later `B3.2+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B3.1` scope only:
- make command-like verifier clauses discoverable without a dedicated `Verification` heading
- allow bullet-only or inline verifier clauses in `Mission`, `Scope`, or `UnknownSection` contexts to produce grounded verification evidence
- land the bullet-only / inline verifier-role regression in the same packet

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- clause-grounded preference vs candidate-row fallback from `B3.2` beyond the minimum needed for this packet
- target-extraction changes from `B2.*`
- compatibility rendering / `comparison_key` work from `SO-3.*`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B3.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B3.1` is review-clean.

Required verification wall for Packet `B3.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B3.1` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `B1.1` through `B2.2` are already landed and no later `B3.2+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B3.1`:
- make command-like verifier clauses discoverable without a dedicated `Verification` heading
- allow bullet-only or inline verifier clauses in `Mission`, `Scope`, or `UnknownSection` contexts to produce grounded verification evidence
- land the bullet-only / inline verifier-role regression in the same packet

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B3.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B3.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B3.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B3.1` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether unheaded verifier clauses can now produce grounded verification-role evidence
- whether the packet includes the bullet-only / inline verifier-role proving regression in the same packet
- whether the packet stayed focused on role discoverability rather than broader extraction changes

Review the new bullet-only / inline verifier-role regression first, then the implementation diff.
List findings by severity.
State clearly whether Packet `B3.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B3.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B3.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B3.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B3.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: ground unheaded verification clauses`
- fix commit message example: `fix: address b3.1 review findings`

Your job is done only when Packet `B3.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 5 Prompt — Task B3.2

````text
/goal Land Packet `B3.2` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B3.2` only, assuming Packets `B1.1` through `B3.1` are already landed and no later `B4.*+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B3.2` scope only:
- prefer clause-grounded verification extraction when a verification-bearing clause exists
- keep whole-candidate fallback only as a conservative backup path
- land the clause-grounded extraction regression in the same packet so the preference is proven when the behavior lands

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- compatibility rendering / `comparison_key` work from `SO-3.*`
- `objective_acceptance` harness or fixture work
- downstream consumer migration

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B3.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B3.2` is review-clean.

Required verification wall for Packet `B3.2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B3.2` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `B1.1` through `B3.1` are already landed and no later `B4.*+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B3.2`:
- prefer clause-grounded verification extraction when a verification-bearing clause exists
- keep whole-candidate fallback only as a conservative backup path
- land the clause-grounded extraction regression in the same packet so the preference is proven when the behavior lands

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B3.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B3.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B3.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B3.2` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether `verification_commands` now come from role-grounded clauses when such clauses exist
- whether the packet includes the clause-grounded proving regression in the same packet
- whether whole-row fallback remains conservative and secondary rather than the hidden authority

Review the clause-grounded extraction regression first, then the implementation diff.
List findings by severity.
State clearly whether Packet `B3.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B3.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B3.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B3.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B3.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: prefer clause-grounded verification extraction`
- fix commit message example: `fix: address b3.2 review findings`

Your job is done only when Packet `B3.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 6 Prompt — Task B4.1

````text
/goal Land Packet `B4.1` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B4.1` only, assuming Packets `B1.1` through `B3.2` are already landed and no later `B5.*+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B4.1` scope only:
- audit the remaining proof surface after B1-B3 land
- add only any missing combined-case or residual regressions needed to make the packet family fully reviewable and durable
- keep the packet centered on `crates/agent-drift-analyzer/tests/checkpoints.rs` unless a tiny packet-scoped fix is required to make the final audit honest

Primary files for this packet:
- crates/agent-drift-analyzer/tests/checkpoints.rs
- minimal packet-scoped code files only if required to make the final combined-case audit honest

Out of scope:
- re-establishing the core proof load for B1-B3
- compatibility rendering / `comparison_key` work from `SO-3.*`
- `objective_acceptance` harness or fixture work from `SO-4.*` / `SO-5.*`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B4.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B4.1` is review-clean.

Required verification wall for Packet `B4.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B4.1` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `B1.1` through `B3.2` are already landed and no later `B5.*+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B4.1`:
- audit the remaining proof surface after B1-B3 land
- add only any missing combined-case or residual regressions needed to make the packet family fully reviewable and durable
- keep the packet centered on `crates/agent-drift-analyzer/tests/checkpoints.rs` unless a tiny packet-scoped fix is required to make the final audit honest

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- crates/agent-drift-analyzer/tests/checkpoints.rs
- crates/agent-drift-analyzer/src/checkpoint/mod.rs
- crates/agent-drift-analyzer/src/context/objective.rs

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B4.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B4.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B4.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B4.1` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether the packet audits final coverage instead of carrying the core proof load for earlier behavior packets
- whether any added regressions are truly missing combined-case coverage rather than deferred core proof
- whether the packet stayed regression-audit-focused and reviewable

Review the final combined-case audit regressions first, then any minimal supporting code changes.
List findings by severity.
State clearly whether Packet `B4.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B4.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B4.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B4.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B4.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add final so-2.3b combined-case regressions`
- fix commit message example: `fix: address b4.1 review findings`

Your job is done only when Packet `B4.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 7 Prompt — Task B5.1

````text
/goal Land Packet `B5.1` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B5.1` only, assuming Packets `B1.1` through `B4.1` are already landed and no later `B5.2+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B5.1` scope only:
- run the packet verification wall for the fully landed `SO-2.3B-refine` packet family
- treat this as verification-first work: no implementation changes are needed unless the verification wall exposes a packet-scoped defect
- if defects are exposed, fix only those packet-scoped defects and rerun the required commands

Primary files for this packet:
- no source files are required up front; touch code only if the verification wall exposes a packet-scoped defect

Out of scope:
- new feature work
- compatibility rendering / `comparison_key` work from `SO-3.*`
- `objective_acceptance` harness or fixture work
- broad cleanup outside defects exposed by the packet verification wall

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B5.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B5.1` is review-clean.

Required verification wall for Packet `B5.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B5.1` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `B1.1` through `B4.1` are already landed and no later `B5.2+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `B5.1`:
- run the packet verification wall for the fully landed `SO-2.3B-refine` packet family
- treat this as verification-first work: no implementation changes are needed unless the verification wall exposes a packet-scoped defect
- if defects are exposed, fix only those packet-scoped defects and rerun the required commands

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B5.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B5.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B5.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B5.1` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether the packet verification wall was run honestly and completely
- whether any fixes stayed strictly packet-scoped to defects exposed by the wall
- whether the final verification story is sufficient before handoff to B5.2 / SO-3

Review the verification evidence first, then inspect any implementation diff only if fixes were needed.
List findings by severity.
State clearly whether Packet `B5.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B5.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B5.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B5.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B5.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: resolve so-2.3b refine verification wall defects`
- fix commit message example: `fix: address b5.1 review findings`

Your job is done only when Packet `B5.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---

## Packet 8 Prompt — Task B5.2

````text
/goal Land Packet `B5.2` from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `B5.2` only, assuming Packets `B1.1` through `B5.1` are already landed and the packet family is checkpoint-green.

Packet authority:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Packet `B5.2` scope only:
- capture the next-packet handoff honestly so `SO-3.1` / `SO-3.2` are clearly next and `SO-4` / `SO-5` remain blocked on `SO-3`
- keep this packet docs-only unless a tiny routing-note update outside the packet docs is strictly required
- preserve auditability instead of silently rewriting packet history

Primary files for this packet:
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- docs/specs/r5/R5_75/MAP.md (only if a narrow routing-note touch is strictly required)

Out of scope:
- starting `SO-3` implementation work
- starting `SO-4` harness work
- widening into unrelated phase-1 doc reconciliation
- code changes unrelated to packet closeout

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `B5.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `B5.2` is review-clean.

Required verification wall for Packet `B5.2`:

```bash
manual review of the landed packet docs and any narrow routing-note update
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `B5.2` only from `docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `B1.1` through `B5.1` are already landed and the packet family is checkpoint-green.

Use the `$incremental-implementation` skill.

You are landing only Packet `B5.2`:
- capture the next-packet handoff honestly so `SO-3.1` / `SO-3.2` are clearly next and `SO-4` / `SO-5` remain blocked on `SO-3`
- keep this packet docs-only unless a tiny routing-note update outside the packet docs is strictly required
- preserve auditability instead of silently rewriting packet history

Authoritative docs to read first:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Files to inspect before editing:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `B5.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- if this packet is verification-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `B5.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `B5.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `B5.2` from:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Focus:
- whether the handoff states clearly that `SO-3.1` / `SO-3.2` are next
- whether `SO-4` and `SO-5` remain explicitly blocked on `SO-3`
- whether the closeout wording is honest and narrowly scoped rather than silently rewriting history

Review the docs diff and closeout honesty first; there is no code path to inspect unless the packet widened improperly.
List findings by severity.
State clearly whether Packet `B5.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `B5.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `B5.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/MAP.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-architecture.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/design-arch/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/R5_75/phase-1/SO/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-spec.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-plan.md
- docs/specs/r5/R5_75/phase-1/SO/SO-2.3B-refine/SO-2.3B-refine-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `B5.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-3.*` / `SO-4.*` / `SO-5.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `B5.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: capture so-3 handoff after so-2.3b refine`
- fix commit message example: `docs: address b5.2 review findings`

Your job is done only when Packet `B5.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
