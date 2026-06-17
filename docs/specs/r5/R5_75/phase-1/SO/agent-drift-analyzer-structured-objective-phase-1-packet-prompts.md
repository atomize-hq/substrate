# Structured Objective Phase 1 Packet Prompts
Status: draft orchestration prompts created on 2026-06-14 and reconciled on 2026-06-17 against the
live `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` ledger after the
grounding follow-on family closed through `SO-G6`.
These prompts intentionally cover the phase-1 packets `SO-1.1` through `SO-6.2`. Already-landed
docs-lock packets `SO-0.*` are excluded, and deferred ask-first packets `SO-X.*` remain excluded
until separately approved. Packet numbering below preserves phase-1 task order for auditability;
however, the live post-grounding restart no longer begins inside this historical sequence. The
active restart now goes through the dedicated `SO-2.3B-refine` packet prompts, and only after that
subfamily lands should the remaining `SO-3.*` / `SO-4.*` prompts below be reused as applicable.
Shared authority for all packets:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
---
## Packet 1 Prompt — Task SO-1.1

````text
/goal Land Packet `SO-1.1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-1.1` only, assuming `SO-0` is already landed and no later `SO-1.2+` packet has started.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-1.1` scope only:
- define shared schema types for `StructuredObjective`, field-level supporting enums, `ObjectiveEvidenceSpan`, and `ObjectiveUnknown`
- keep the work centered on `crates/agent-drift-analyzer/src/checkpoint/schema.rs` and the minimum wiring needed in `crates/agent-drift-analyzer/src/context/objective.rs`
- match the architecture authority's minimum semantic coverage without widening into downstream consumer migration

Primary files for this packet:
- crates/agent-drift-analyzer/src/checkpoint/schema.rs
- crates/agent-drift-analyzer/src/context/objective.rs

Out of scope:
- `ObjectiveSummary` compatibility-field wiring from `SO-1.2`
- section/clause decomposition and structured assembly from `SO-2.*`
- `TaskFrame` or downstream consumer migration
- classifier/model work
- Packet `SO-1.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-1.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-1.1` is review-clean.

Required verification wall for Packet `SO-1.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-1.1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming `SO-0` is already landed and no later `SO-1.2+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-1.1`:
- define shared schema types for `StructuredObjective`, field-level supporting enums, `ObjectiveEvidenceSpan`, and `ObjectiveUnknown`
- keep the work centered on `crates/agent-drift-analyzer/src/checkpoint/schema.rs` and the minimum wiring needed in `crates/agent-drift-analyzer/src/context/objective.rs`
- match the architecture authority's minimum semantic coverage without widening into downstream consumer migration

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/checkpoint/schema.rs
- crates/agent-drift-analyzer/src/context/objective.rs

Files to inspect before editing:
- crates/agent-drift-analyzer/src/checkpoint/schema.rs
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
- stay strictly inside Packet `SO-1.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-1.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-1.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-1.1` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- define shared schema types for `StructuredObjective`, field-level supporting enums, `ObjectiveEvidenceSpan`, and `ObjectiveUnknown`
- keep the work centered on `crates/agent-drift-analyzer/src/checkpoint/schema.rs` and the minimum wiring needed in `crates/agent-drift-analyzer/src/context/objective.rs`
- match the architecture authority's minimum semantic coverage without widening into downstream consumer migration
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-1.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-1.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-1.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-1.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-1.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: add structured objective phase-1 schema types`
- fix commit message example: `fix: address so-1.1 review findings`

Your job is done only when Packet `SO-1.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 2 Prompt — Task SO-1.2

````text
/goal Land Packet `SO-1.2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-1.2` only, assuming Packet `SO-1.1` is already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-1.2` scope only:
- add `comparison_key` and `structured: Option<StructuredObjective>` to `ObjectiveSummary`
- preserve `text`, `verification_commands`, and `evidence` while keeping the sidecar additive and optional
- update the minimum analyzer-local wiring so current callers still compile without assuming sidecar presence

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/src/context/mod.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- section/clause decomposition and structured extraction logic from `SO-2.*`
- compatibility rendering policy changes beyond the minimum bridge
- downstream consumer migration in `working_set`, `checkpoint/mod.rs`, or `checkpoint/progress.rs`
- Packet `SO-2.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-1.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-1.2` is review-clean.

Required verification wall for Packet `SO-1.2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-1.2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet `SO-1.1` is already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-1.2`:
- add `comparison_key` and `structured: Option<StructuredObjective>` to `ObjectiveSummary`
- preserve `text`, `verification_commands`, and `evidence` while keeping the sidecar additive and optional
- update the minimum analyzer-local wiring so current callers still compile without assuming sidecar presence

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/src/context/mod.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Files to inspect before editing:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/src/context/mod.rs
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
- stay strictly inside Packet `SO-1.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-1.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-1.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-1.2` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- add `comparison_key` and `structured: Option<StructuredObjective>` to `ObjectiveSummary`
- preserve `text`, `verification_commands`, and `evidence` while keeping the sidecar additive and optional
- update the minimum analyzer-local wiring so current callers still compile without assuming sidecar presence
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-1.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-1.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-1.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-1.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-1.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: add objective summary comparison key sidecar bridge`
- fix commit message example: `fix: address so-1.2 review findings`

Your job is done only when Packet `SO-1.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 3 Prompt — Task SO-2.1

````text
/goal Land Packet `SO-2.1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-2.1` only, assuming Packets `SO-1.1` and `SO-1.2` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-2.1` scope only:
- teach objective extraction to distinguish mission/scope, checklist, verification, constraints, deliverables, context, boilerplate, and tooling-instruction sections
- keep the work centered on `crates/agent-drift-analyzer/src/context/objective.rs` plus targeted regressions in `tests/checkpoints.rs`
- stop whole-row flattening from hiding the real mission without widening into full field assembly yet

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- clause-role labeling from `SO-2.2`
- full structured-field assembly from `SO-2.3`
- compatibility rendering from `SO-3.*`
- Packet `SO-2.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-2.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-2.1` is review-clean.

Required verification wall for Packet `SO-2.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-2.1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `SO-1.1` and `SO-1.2` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-2.1`:
- teach objective extraction to distinguish mission/scope, checklist, verification, constraints, deliverables, context, boilerplate, and tooling-instruction sections
- keep the work centered on `crates/agent-drift-analyzer/src/context/objective.rs` plus targeted regressions in `tests/checkpoints.rs`
- stop whole-row flattening from hiding the real mission without widening into full field assembly yet

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

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
- stay strictly inside Packet `SO-2.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-2.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-2.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-2.1` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- teach objective extraction to distinguish mission/scope, checklist, verification, constraints, deliverables, context, boilerplate, and tooling-instruction sections
- keep the work centered on `crates/agent-drift-analyzer/src/context/objective.rs` plus targeted regressions in `tests/checkpoints.rs`
- stop whole-row flattening from hiding the real mission without widening into full field assembly yet
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-2.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-2.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-2.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-2.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-2.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: add structured objective section decomposition`
- fix commit message example: `fix: address so-2.1 review findings`

Your job is done only when Packet `SO-2.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 4 Prompt — Task SO-2.2

````text
/goal Land Packet `SO-2.2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-2.2` only, assuming Packets `SO-1.1` through `SO-2.1` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-2.2` scope only:
- label clause or sentence units with `goal`, `constraint`, `verification`, `context`, or `other_role`
- ground each nontrivial structured field candidate to evidence spans with source kind and section kind
- keep the work centered on deterministic clause-role logic and focused checkpoint regressions

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- full phase-1 field assembly from `SO-2.3`
- compatibility rendering and comparison-key work from `SO-3.*`
- objective-acceptance harness work from `SO-4.*`
- Packet `SO-2.3+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-2.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-2.2` is review-clean.

Required verification wall for Packet `SO-2.2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-2.2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `SO-1.1` through `SO-2.1` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-2.2`:
- label clause or sentence units with `goal`, `constraint`, `verification`, `context`, or `other_role`
- ground each nontrivial structured field candidate to evidence spans with source kind and section kind
- keep the work centered on deterministic clause-role logic and focused checkpoint regressions

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

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
- stay strictly inside Packet `SO-2.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-2.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-2.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-2.2` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- label clause or sentence units with `goal`, `constraint`, `verification`, `context`, or `other_role`
- ground each nontrivial structured field candidate to evidence spans with source kind and section kind
- keep the work centered on deterministic clause-role logic and focused checkpoint regressions
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-2.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-2.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-2.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-2.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-2.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: add structured objective clause roles and grounding`
- fix commit message example: `fix: address so-2.2 review findings`

Your job is done only when Packet `SO-2.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 5 Prompt — Task SO-2.3

````text
/goal Land Packet `SO-2.3` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-2.3` only, assuming Packets `SO-1.1` through `SO-2.2` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-2.3` scope only:
- assemble objective class, primary intent, target, constraints, success conditions, deliverables, verification commands, confidence, evidence spans, and unknowns only when evidence supports them
- preserve weak evidence as explicit unknowns instead of guesses
- keep the work analyzer-local to structured extraction and focused checkpoint regressions

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- compatibility rendering policy and comparison-key derivation from `SO-3.*`
- objective-acceptance harness work from `SO-4.*`
- downstream consumer migration
- Packet `SO-3.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-2.3` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-2.3` is review-clean.

Required verification wall for Packet `SO-2.3`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-2.3` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets `SO-1.1` through `SO-2.2` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-2.3`:
- assemble objective class, primary intent, target, constraints, success conditions, deliverables, verification commands, confidence, evidence spans, and unknowns only when evidence supports them
- preserve weak evidence as explicit unknowns instead of guesses
- keep the work analyzer-local to structured extraction and focused checkpoint regressions

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

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
- stay strictly inside Packet `SO-2.3`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-2.3` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-2.3` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-2.3` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- assemble objective class, primary intent, target, constraints, success conditions, deliverables, verification commands, confidence, evidence spans, and unknowns only when evidence supports them
- preserve weak evidence as explicit unknowns instead of guesses
- keep the work analyzer-local to structured extraction and focused checkpoint regressions
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-2.3` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-2.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-2.3` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-2.3` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-2.3` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: assemble structured objective phase-1 fields`
- fix commit message example: `fix: address so-2.3 review findings`

Your job is done only when Packet `SO-2.3` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 6 Prompt — Task SO-3.1

````text
/goal Land Packet `SO-3.1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-3.1` only, assuming the
grounding follow-on family through `SO-G6` plus Packets `SO-4.1` and `SO-4.2` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-3.1` scope only:
- render `ObjectiveSummary.text` as a compatibility view over structured state when the evidence is sufficient
- keep conservative fallback behavior when the sidecar is absent or key fields remain unknown
- prove the display string is a projection rather than the semantic authority

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- comparison-key derivation changes from `SO-3.2`
- objective-acceptance harness and fixtures from `SO-4.*` and `SO-5.*`
- downstream consumer migration
- Packet `SO-3.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-3.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-3.1` is review-clean.

Required verification wall for Packet `SO-3.1`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-3.1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` plus Packets `SO-4.1` and `SO-4.2` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-3.1`:
- render `ObjectiveSummary.text` as a compatibility view over structured state when the evidence is sufficient
- keep conservative fallback behavior when the sidecar is absent or key fields remain unknown
- prove the display string is a projection rather than the semantic authority

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

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
- stay strictly inside Packet `SO-3.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-3.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-3.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-3.1` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- render `ObjectiveSummary.text` as a compatibility view over structured state when the evidence is sufficient
- keep conservative fallback behavior when the sidecar is absent or key fields remain unknown
- prove the display string is a projection rather than the semantic authority
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-3.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-3.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-3.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-3.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-3.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: render objective compatibility text from structured state`
- fix commit message example: `fix: address so-3.1 review findings`

Your job is done only when Packet `SO-3.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 7 Prompt — Task SO-3.2

````text
/goal Land Packet `SO-3.2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-3.2` only, assuming the
grounding follow-on family through `SO-G6` plus Packets `SO-4.1`, `SO-4.2`, and `SO-3.1` are
already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-3.2` scope only:
- derive `comparison_key` from structured semantic state rather than raw pretty text
- add focused regressions proving checklist or boilerplate wording changes do not silently become the truth source
- keep the work centered on objective comparison-bridge logic rather than downstream comparability consumers

Primary files for this packet:
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- `TaskFrame.objective_key` coexistence or progress comparability migration
- objective-acceptance harness work from `SO-4.*`
- locked acceptance corpus expansion from `SO-5.*`
- Packet `SO-4.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-3.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-3.2` is review-clean.

Required verification wall for Packet `SO-3.2`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-3.2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` plus Packets `SO-4.1`, `SO-4.2`, and `SO-3.1` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-3.2`:
- derive `comparison_key` from structured semantic state rather than raw pretty text
- add focused regressions proving checklist or boilerplate wording changes do not silently become the truth source
- keep the work centered on objective comparison-bridge logic rather than downstream comparability consumers

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/src/context/objective.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

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
- stay strictly inside Packet `SO-3.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-3.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-3.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-3.2` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- derive `comparison_key` from structured semantic state rather than raw pretty text
- add focused regressions proving checklist or boilerplate wording changes do not silently become the truth source
- keep the work centered on objective comparison-bridge logic rather than downstream comparability consumers
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-3.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-3.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-3.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-3.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-3.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: derive structured objective comparison key`
- fix commit message example: `fix: address so-3.2 review findings`

Your job is done only when Packet `SO-3.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 8 Prompt — Task SO-4.1

````text
/goal Land Packet `SO-4.1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-4.1` only, assuming the
grounding follow-on family through `SO-G6` is already landed and no later `SO-4.2+` packet has
started.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-4.1` scope only:
- add `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
- add the deterministic fixture loader/support needed for committed objective-acceptance fixtures
- assert the fixture directory contract for `design-set`, `locked-acceptance`, and `stretch-external`

Primary files for this packet:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/support/mod.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

Out of scope:
- encoding the detailed expected-shape contract from `SO-4.2`
- adding the seed fixtures from `SO-5.*`
- changing full analyzer behavior beyond what the new harness requires
- Packet `SO-4.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-4.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-4.1` is review-clean.

Required verification wall for Packet `SO-4.1`:

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-4.1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` is already landed and no later `SO-4.2+` packet has started.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-4.1`:
- add `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
- add the deterministic fixture loader/support needed for committed objective-acceptance fixtures
- assert the fixture directory contract for `design-set`, `locked-acceptance`, and `stretch-external`

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/tests/support/mod.rs
- crates/agent-drift-analyzer/tests/progress_acceptance.rs
- crates/agent-drift-analyzer/tests/acceptance_fixtures.rs

Files to inspect before editing:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/support/mod.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-4.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-4.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-4.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-4.1` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- add `crates/agent-drift-analyzer/tests/objective_acceptance.rs`
- add the deterministic fixture loader/support needed for committed objective-acceptance fixtures
- assert the fixture directory contract for `design-set`, `locked-acceptance`, and `stretch-external`
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-4.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-4.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-4.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-4.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-4.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add objective acceptance harness scaffold`
- fix commit message example: `fix: address so-4.1 review findings`

Your job is done only when Packet `SO-4.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 9 Prompt — Task SO-4.2

````text
/goal Land Packet `SO-4.2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-4.2` only, assuming the
grounding follow-on family through `SO-G6` and Packet `SO-4.1` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-4.2` scope only:
- teach the harness to validate structured-field expectations, role spans, grounding refs, forbidden promotions, compatibility rendering, and unknown-field correctness from fixture metadata
- keep the work centered on the committed expected-shape contract rather than adding many fixtures yet
- make the acceptance wall depend on structured correctness rather than one exact objective string

Primary files for this packet:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

Out of scope:
- adding WDAP or preserved-target fixture seeds from `SO-5.*`
- changing core analyzer extraction logic unless the harness absolutely requires it
- Packet `SO-5.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-4.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-4.2` is review-clean.

Required verification wall for Packet `SO-4.2`:

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-4.2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` and Packet `SO-4.1` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-4.2`:
- teach the harness to validate structured-field expectations, role spans, grounding refs, forbidden promotions, compatibility rendering, and unknown-field correctness from fixture metadata
- keep the work centered on the committed expected-shape contract rather than adding many fixtures yet
- make the acceptance wall depend on structured correctness rather than one exact objective string

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

Files to inspect before editing:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-4.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-4.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-4.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-4.2` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- teach the harness to validate structured-field expectations, role spans, grounding refs, forbidden promotions, compatibility rendering, and unknown-field correctness from fixture metadata
- keep the work centered on the committed expected-shape contract rather than adding many fixtures yet
- make the acceptance wall depend on structured correctness rather than one exact objective string
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-4.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-4.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-4.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-4.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-4.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: encode objective acceptance contract checks`
- fix commit message example: `fix: address so-4.2 review findings`

Your job is done only when Packet `SO-4.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 10 Prompt — Task SO-5.1

````text
/goal Land Packet `SO-5.1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-5.1` only, assuming the
grounding follow-on family through `SO-G6` plus Packets `SO-4.1` and `SO-4.2` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-5.1` scope only:
- add the required WDAP kickoff seeds to the locked acceptance set
- encode expected metadata that explicitly forbids subordinate checklist lines from being promoted to `goal`
- keep the work centered on committed fixtures plus the harness updates needed to load and assert them

Primary files for this packet:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**

Out of scope:
- preserved boilerplate-target controls from `SO-5.2`
- concise-goal / review / planning control families from `SO-5.3`
- downstream analyzer migrations
- Packet `SO-5.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-5.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-5.1` is review-clean.

Required verification wall for Packet `SO-5.1`:

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-5.1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` plus Packets `SO-4.1` and `SO-4.2` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-5.1`:
- add the required WDAP kickoff seeds to the locked acceptance set
- encode expected metadata that explicitly forbids subordinate checklist lines from being promoted to `goal`
- keep the work centered on committed fixtures plus the harness updates needed to load and assert them

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

Files to inspect before editing:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-5.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-5.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-5.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-5.1` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- add the required WDAP kickoff seeds to the locked acceptance set
- encode expected metadata that explicitly forbids subordinate checklist lines from being promoted to `goal`
- keep the work centered on committed fixtures plus the harness updates needed to load and assert them
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-5.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-5.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-5.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-5.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-5.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add wdap locked objective acceptance seeds`
- fix commit message example: `fix: address so-5.1 review findings`

Your job is done only when Packet `SO-5.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 11 Prompt — Task SO-5.2

````text
/goal Land Packet `SO-5.2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-5.2` only, assuming the
grounding follow-on family through `SO-G6` plus Packets `SO-4.1`, `SO-4.2`, and `SO-5.1` are
already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-5.2` scope only:
- add locked-acceptance cases where the real task is to analyze or edit `AGENTS.md`, `<skill>`, `Available skills`, or similar instruction surfaces
- prove the structured extractor preserves those targets instead of filtering them away
- keep the work fixture-centered with only the harness updates strictly required to score those controls

Primary files for this packet:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**

Out of scope:
- concise-goal / review / planning control families from `SO-5.3`
- new extraction heuristics beyond what the acceptance cases reveal as necessary
- Packet `SO-5.3+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-5.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-5.2` is review-clean.

Required verification wall for Packet `SO-5.2`:

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-5.2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` plus Packets `SO-4.1`, `SO-4.2`, and `SO-5.1` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-5.2`:
- add locked-acceptance cases where the real task is to analyze or edit `AGENTS.md`, `<skill>`, `Available skills`, or similar instruction surfaces
- prove the structured extractor preserves those targets instead of filtering them away
- keep the work fixture-centered with only the harness updates strictly required to score those controls

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

Files to inspect before editing:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-5.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-5.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-5.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-5.2` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- add locked-acceptance cases where the real task is to analyze or edit `AGENTS.md`, `<skill>`, `Available skills`, or similar instruction surfaces
- prove the structured extractor preserves those targets instead of filtering them away
- keep the work fixture-centered with only the harness updates strictly required to score those controls
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-5.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-5.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-5.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-5.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-5.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add preserved boilerplate objective controls`
- fix commit message example: `fix: address so-5.2 review findings`

Your job is done only when Packet `SO-5.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 12 Prompt — Task SO-5.3

````text
/goal Land Packet `SO-5.3` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-5.3` only, assuming the
grounding follow-on family through `SO-G6` plus Packets `SO-4.1`, `SO-4.2`, `SO-5.1`, and
`SO-5.2` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-5.3` scope only:
- add concise `/goal` controls, review/no-code semantics, and planning/research/docs prompts to the committed objective-acceptance corpus
- prove Phase 1 can distinguish implementation from non-implementation intent without overfitting to WDAP alone
- keep the work centered on design-set and locked-acceptance fixtures plus the harness updates needed to score them

Primary files for this packet:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**

Out of scope:
- full phase-1 validation and closeout from `SO-6.*`
- future stretch-external expansion beyond what this packet needs
- downstream migration work
- Packet `SO-6.1+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-5.3` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-5.3` is review-clean.

Required verification wall for Packet `SO-5.3`:

```bash
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-5.3` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6` plus Packets `SO-4.1`, `SO-4.2`, `SO-5.1`, and `SO-5.2` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-5.3`:
- add concise `/goal` controls, review/no-code semantics, and planning/research/docs prompts to the committed objective-acceptance corpus
- prove Phase 1 can distinguish implementation from non-implementation intent without overfitting to WDAP alone
- keep the work centered on design-set and locked-acceptance fixtures plus the harness updates needed to score them

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/README.md

Files to inspect before editing:
- crates/agent-drift-analyzer/tests/objective_acceptance.rs
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/design-set/**
- crates/agent-drift-analyzer/tests/fixtures/objective_acceptance/locked-acceptance/**

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-5.3`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-5.3` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-5.3` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-5.3` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- add concise `/goal` controls, review/no-code semantics, and planning/research/docs prompts to the committed objective-acceptance corpus
- prove Phase 1 can distinguish implementation from non-implementation intent without overfitting to WDAP alone
- keep the work centered on design-set and locked-acceptance fixtures plus the harness updates needed to score them
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-5.3` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-5.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-5.3` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-5.3` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-5.3` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: add structured objective control families`
- fix commit message example: `fix: address so-5.3 review findings`

Your job is done only when Packet `SO-5.3` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 13 Prompt — Task SO-6.1

````text
/goal Land Packet `SO-6.1` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-6.1` only, assuming the
grounding follow-on family through `SO-G6`, Packets `SO-4.1` and `SO-4.2`, Packets `SO-3.1` and
`SO-3.2`, and Packets `SO-5.1` through `SO-5.3` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-6.1` scope only:
- run the full phase-1 validation wall: fmt, clippy, focused checkpoints, objective acceptance, and the full analyzer suite
- if validation exposes a real packet-scoped defect in the already-landed structured-objective work, fix only what is required to make the phase-1 validation wall honest and green
- do not invent empty commits if validation passes with no code or doc changes

Primary files for this packet:
- no file edits required by default; only touch packet-scoped defects revealed by validation

Out of scope:
- capturing deferred follow-on seams in docs from `SO-6.2`
- new feature work outside the validation wall
- TaskFrame/downstream/classifier follow-ons
- Packet `SO-6.2`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-6.1` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-6.1` is review-clean.

Required verification wall for Packet `SO-6.1`:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test objective_acceptance -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-6.1` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6`, Packets `SO-4.1` and `SO-4.2`, Packets `SO-3.1` and `SO-3.2`, and Packets `SO-5.1` through `SO-5.3` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-6.1`:
- run the full phase-1 validation wall: fmt, clippy, focused checkpoints, objective acceptance, and the full analyzer suite
- if validation exposes a real packet-scoped defect in the already-landed structured-objective work, fix only what is required to make the phase-1 validation wall honest and green
- do not invent empty commits if validation passes with no code or doc changes

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md

Files to inspect before editing:
- no file edits required by default; only touch packet-scoped defects revealed by validation

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-6.1`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-6.1` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-6.1` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-6.1` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- run the full phase-1 validation wall: fmt, clippy, focused checkpoints, objective acceptance, and the full analyzer suite
- if validation exposes a real packet-scoped defect in the already-landed structured-objective work, fix only what is required to make the phase-1 validation wall honest and green
- do not invent empty commits if validation passes with no code or doc changes
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-6.1` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-6.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-6.1` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-6.1` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-6.1` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `fix: resolve structured objective validation wall defects`
- fix commit message example: `fix: address so-6.1 review findings`

Your job is done only when Packet `SO-6.1` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 14 Prompt — Task SO-6.2

````text
/goal Land Packet `SO-6.2` from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until the packet is review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `SO-6.2` only, assuming the
grounding follow-on family through `SO-G6`, Packets `SO-4.1` and `SO-4.2`, Packets `SO-3.1` and
`SO-3.2`, Packets `SO-5.1` through `SO-5.3`, and Packet `SO-6.1` are already landed.

Packet authority:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Packet `SO-6.2` scope only:
- record any remaining TaskFrame coexistence, working-set migration, checkpoint predicate migration, progress comparability migration, or classifier work as explicit follow-on debt
- keep the work docs-only in the phase-1 plan/tasks artifacts
- make the closeout story honest instead of leaving implicit TODOs or half-wired code comments

Primary files for this packet:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md

Out of scope:
- implementing any deferred follow-on seam
- rewriting root landing-order authority beyond the phase-1 docs
- touching analyzer code unless a doc statement would otherwise be false
- deferred `SO-X.*` work

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before editing any Rust function, method, enum, struct, helper, or other indexed symbol, the implementation subagent must honor the repo GitNexus rule: run impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the implementation subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `SO-6.2` implementation before dispatching review. If the packet is verification-only and no files changed, do not invent an empty commit; instead record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start the next packet until `SO-6.2` is review-clean.

Required verification wall for Packet `SO-6.2`:

```bash
Manual review of the landed phase-1 docs and code diff
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `SO-6.2` only from `docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming the grounding follow-on family through `SO-G6`, Packets `SO-4.1` and `SO-4.2`, Packets `SO-3.1` and `SO-3.2`, Packets `SO-5.1` through `SO-5.3`, and Packet `SO-6.1` are already landed.

Use the `$incremental-implementation` skill.

You are landing only Packet `SO-6.2`:
- record any remaining TaskFrame coexistence, working-set migration, checkpoint predicate migration, progress comparability migration, or classifier work as explicit follow-on debt
- keep the work docs-only in the phase-1 plan/tasks artifacts
- make the closeout story honest instead of leaving implicit TODOs or half-wired code comments

Authoritative docs to read first:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md

Files to inspect before editing:
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md

GitNexus requirements:
- before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, run impact analysis on the symbol first
- report any HIGH or CRITICAL blast radius before proceeding
- run `gitnexus_detect_changes()` before handing back for commit

Precondition check:
- Confirm the prior packet tasks named in this prompt are landed in code and tests before editing.
- If a prerequisite is missing, stop and report the missing prerequisite instead of compensating
  inside this packet.

Execution rules:
- stay strictly inside Packet `SO-6.2`
- use `$incremental-implementation` to keep the work slice-sized and verification-backed
- do not broaden into later packets or deferred follow-on work
- if this packet is validation-first or docs-only, keep any fixes tightly packet-scoped
- run the required Packet `SO-6.2` verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `SO-6.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `SO-6.2` from:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Focus:
- record any remaining TaskFrame coexistence, working-set migration, checkpoint predicate migration, progress comparability migration, or classifier work as explicit follow-on debt
- keep the work docs-only in the phase-1 plan/tasks artifacts
- make the closeout story honest instead of leaving implicit TODOs or half-wired code comments
- whether the packet stayed scoped to its declared files and acceptance criteria
- whether the verification story is sufficient and honest for this packet
- whether any regressions, hidden scope expansion, or contract mismatches remain

Review the tests or verification story first, then the implementation/doc diff.
List findings by severity.
State clearly whether Packet `SO-6.2` is review-clean or requires changes.
If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `SO-6.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the relevant Packet `SO-6.2` verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/DESIGN-r5-structured-objective-architecture.md
- docs/specs/r5/DESIGN-r5-structured-objective-evaluation-and-annotation.md
- docs/specs/r5/DESIGN-r5-structured-objective-migration-and-integration.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-spec.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-plan.md
- docs/specs/r5/agent-drift-analyzer-structured-objective-phase-1-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet `SO-6.2` issues
- keep the work packet-scoped
- do not broaden into later packets or deferred `SO-X.*` work
- run impact analysis before editing affected Rust symbols
- rerun the required Packet `SO-6.2` verification wall
- run `gitnexus_detect_changes()` before handing back for commit

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `docs: capture structured objective phase-1 follow-on seams`
- fix commit message example: `docs: address so-6.2 review findings`

Your job is done only when Packet `SO-6.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
