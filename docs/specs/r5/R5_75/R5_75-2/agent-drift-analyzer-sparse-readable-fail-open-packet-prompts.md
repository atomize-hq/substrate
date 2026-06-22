# Sparse Readable Fail-Open (R5.75-2) Packet Prompts

Status: orchestration prompts created on 2026-06-21 against the live
`docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` ledger.

Each prompt below is self-contained: paste one into a fresh session to land exactly one R5.75-2 packet
through a GPT-5.4 subagent implementation -> commit -> review -> fix -> commit loop. Packet `R5.75-2.0`
(docs lock) is already landed (commit `68216bf02`/`92b0a930a`) and is excluded; the Deferred / Ask-First
tasks (`R5.75-2.X.*`) are excluded until separately approved. Packets are ordered by dependency:
`R5.75-2.1` (investigation) sizes `R5.75-2.3`; `R5.75-2.4` locks the behavior `R5.75-2.2`/`R5.75-2.3`
introduce; `R5.75-2.5` is closeout.

Shared authority for all packets:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md (the R5.75-2 packet)
- AGENTS.md

Note: the GitNexus index may be stale; run `npx gitnexus analyze` before impact analysis if a tool
warns. Working directory for every prompt: `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

---
## Packet 1 Prompt — Task R5.75-2.1 (Investigation; no production code lands)

````text
/goal Land Packet `R5.75-2.1` from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` — an investigation packet whose only committed artifact is a recorded finding — using an implementation -> commit -> review -> fix -> commit loop until review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-2.1` only, assuming Packet `R5.75-2.0` (docs lock) is already landed and no later `R5.75-2.2+` packet has started.

Packet authority:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Packet `R5.75-2.1` scope only:
- determine, empirically, what `analyze_loaded_bundle` / `checkpoint_analyses` emits for the adapted session `f47b81f39f2495dd` IF `validate_surface` did not abort: does it already emit >=1 conservative (`InsufficientEvidence` / low-confidence) checkpoint, or does it emit nothing / over-claim?
- this is investigation only: any temporary `validate_surface` relaxation (or a hand-built sparse-bundle unit test) used to observe behavior MUST be reverted; no production code is committed by this packet
- record the finding in the tasks ledger under Task `R5.75-2.1.1`; it resolves Spec Open Question 1 and sizes Packet `R5.75-2.3`

Primary files for this packet:
- (read-only) crates/agent-drift-analyzer/src/lib.rs
- (read-only) crates/agent-drift-analyzer/src/checkpoint/mod.rs
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md (record the finding here)

Out of scope:
- any committed production code change (that is Packets `R5.75-2.2`/`R5.75-2.3`)
- schema changes, `AnalyzerSurface` field additions
- Packets `R5.75-2.2+`

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before modifying any Rust function, method, enum, struct, helper, or other indexed symbol (including any temporary probe edit), the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the subagent finishes, confirm yourself that any temporary `validate_surface` edit was reverted (`git status` clean except the tasks-doc finding) and that the finding is recorded.
6. This packet lands no production code; commit ONLY the recorded finding in the tasks doc. Do not invent an empty code commit.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent flags issues (e.g. leaked probe code, unsound methodology, finding not grounded in an actual run), spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must instruct it to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-2.2` until `R5.75-2.1` is review-clean and the finding is committed.

Required verification for Packet `R5.75-2.1`:
- the finding must be backed by an actual analyzer run on `f47b81f39f2495dd` (compactor bundle already exists under `target/ranga-validation/`), not asserted from reading code alone
- `git status` shows no production-source changes remaining after the probe

Implementation subagent prompt to send:

```text
/goal Investigate Packet `R5.75-2.1` only from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packet `R5.75-2.0` is landed and no later packet has started.

Use the `$incremental-implementation` skill.

This is an investigation packet — no production code is committed. Your deliverable is a recorded finding.

Do this:
- read crates/agent-drift-analyzer/src/input.rs (validate_surface), src/lib.rs (analyze_loaded_bundle), and src/checkpoint/mod.rs
- temporarily make validate_surface NOT abort on the sparse conditions (truth_artifact_hints and working_set_hints/tool_argument_json), OR build a hand-crafted sparse bundle unit test, to observe downstream behavior
- run the analyzer against the existing adapted bundle for session f47b81f39f2495dd (compactor output under target/ranga-validation/, or recompact per the SPEC Commands section)
- observe: does analyze_loaded_bundle / checkpoint_analyses emit >=1 checkpoint, and is it conservative (ProgressStatus::InsufficientEvidence / low-confidence), or does it over-claim / emit nothing?
- REVERT the temporary relaxation/experiment fully (git status must show no production-source diff)
- record the finding in docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md under Task R5.75-2.1.1, stating clearly whether Packet R5.75-2.3 will be assertion-only or must thread bundle.surface into analyze_loaded_bundle

GitNexus: run impact analysis before any temporary symbol edit; run gitnexus_detect_changes() before handing back.

Precondition check: confirm Packet R5.75-2.0 is landed; if a prerequisite is missing, stop and report it instead of compensating here.

Return with: the finding (self-conservatizes vs over-claims vs emits-nothing), the exact run/output you observed, confirmation the experiment was reverted, and the exact commit message you recommend for the tasks-doc finding.
```

Review subagent prompt to send after the finding commit:

```text
/goal Review the already-recorded Packet `R5.75-2.1` finding in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is sound and safe to keep.

Use the `$code-review-and-quality` skill.

Review only Packet `R5.75-2.1` against:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Focus:
- whether any temporary probe code leaked into production source (git history/working tree must be clean of it)
- whether the finding is grounded in an actual analyzer run on f47b81f39f2495dd, not inferred from reading code
- whether the finding correctly sizes Packet R5.75-2.3 (assertion-only vs must-thread-surface)
- whether the packet stayed investigation-only and in scope

State clearly whether Packet R5.75-2.1 is review-clean or requires changes. If changes are required, keep them concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-2.1` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet R5.75-2.1 issues (e.g. revert leaked probe code, re-run the probe, correct the recorded finding)
- keep it investigation-only; commit no production code
- run gitnexus_detect_changes() before handing back

Return with: exact fixes made, the re-run evidence, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- finding commit message example: `docs: record R5.75-2.1 sparse fail-open probe finding`
- fix commit message example: `fix: address R5.75-2.1 review findings`

Your job is done only when Packet `R5.75-2.1` is review-clean and the finding has been committed.
````

---
## Packet 2 Prompt — Task R5.75-2.2 (Split validate_surface)

````text
/goal Land Packet `R5.75-2.2` from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-2.2` only, assuming Packets `R5.75-2.0` and `R5.75-2.1` are already landed.

Packet authority:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Packet `R5.75-2.2` scope only:
- in `crates/agent-drift-analyzer/src/input.rs::validate_surface`, stop returning `Err` for the two sparse-but-readable conditions — the path-hint check (`truth_artifact_hints`) and the tool-payload pair (`working_set_hints` / `tool_argument_json`) — each independently, neither gated on the other; return `Ok(AnalyzerSurface { … })` with those flags `false`
- keep `repetition_preserved`, `stable_row_refs`, `literal_objective_rows`, and every upstream `InputError` variant returning their existing errors and messages verbatim (corruption + the readable floor stay hard-fail)
- keep `AnalyzerSurface` shape unchanged: no new fields, no public schema/version bump
- per `$incremental-implementation`, add the minimal `tests/input_contract.rs` case proving the split for at least the tool-payload axis (the `f47b81f39f2495dd` shape returns `Ok`); the full two-axis + corruption locked set is consolidated in Packet `R5.75-2.4`

Primary files for this packet:
- crates/agent-drift-analyzer/src/input.rs
- crates/agent-drift-analyzer/tests/input_contract.rs (minimal proof of the split)

Out of scope:
- conservative-checkpoint confidence plumbing (Packet `R5.75-2.3`)
- the full locked regression set incl. the conceptual-ask axis and the checkpoint objective-anchor test (Packet `R5.75-2.4`)
- relaxing `literal_objective_rows` (Deferred / Ask-First)
- any schema/`AnalyzerSurface` field change, compactor change, downstream migration

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before modifying any Rust function, method, enum, struct, helper, or other indexed symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding (`validate_surface` is on the `load_bundle` path — expect callers).
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `R5.75-2.2` implementation before dispatching review.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must instruct it to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-2.3` until `R5.75-2.2` is review-clean.

Required verification wall for Packet `R5.75-2.2`:

```bash
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-2.2` only from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5.75-2.0 and R5.75-2.1 are landed.

Use the `$incremental-implementation` skill.

You are landing only Packet R5.75-2.2 — the validate_surface corruption-vs-sparse split:
- stop returning Err for truth_artifact_hints and for the working_set_hints/tool_argument_json pair, each independently; return Ok(AnalyzerSurface { … }) with those flags false
- keep repetition_preserved, stable_row_refs, literal_objective_rows, and all upstream InputError variants failing exactly as today (verbatim messages)
- AnalyzerSurface shape is unchanged: no new fields, no schema/version bump
- add the minimal tests/input_contract.rs case proving the tool-payload-axis sparse bundle now returns Ok (the f47b81f39f2495dd shape: objective rows + path hints + zero parseable tool calls)

Read first:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md (Assumptions, Code Style, Boundaries)
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md (Task R5.75-2.2.1)
- crates/agent-drift-analyzer/src/input.rs (validate_surface)
- AGENTS.md

GitNexus: run impact analysis on validate_surface and any edited symbol before editing; report HIGH/CRITICAL blast radius; run gitnexus_detect_changes() before handing back.

Precondition check: confirm Packets R5.75-2.0 and R5.75-2.1 are landed in code/tests; if a prerequisite is missing, stop and report it instead of compensating here.

Execution rules:
- stay strictly inside Packet R5.75-2.2; do not add the conceptual-ask axis test or the conservative-checkpoint test (those are R5.75-2.4) and do not touch confidence plumbing (R5.75-2.3)
- keep the work slice-sized and verification-backed
- run the required verification wall

Return with: changed files, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-2.2` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` against its authority docs and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet R5.75-2.2 from:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Focus:
- the two sparse conditions (truth_artifact_hints; working_set_hints/tool_argument_json) now fail open independently, and the repro shape returns Ok
- corruption checks (repetition_preserved, stable_row_refs) and literal_objective_rows still hard-fail with their exact existing InputError variants/messages
- AnalyzerSurface shape and the public schema are unchanged (no version bump, no new fields)
- whether the packet stayed scoped to validate_surface + a minimal input_contract proof, with no leakage into R5.75-2.3/R5.75-2.4 scope
- whether the verification story is sufficient and honest

Review the tests/verification story first, then the implementation diff. List findings by severity. State clearly whether Packet R5.75-2.2 is review-clean or requires changes; keep any required changes concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-2.2` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet R5.75-2.2 verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet R5.75-2.2 issues; keep the work packet-scoped
- do not broaden into R5.75-2.3/R5.75-2.4 or deferred work
- run impact analysis before editing affected Rust symbols
- rerun: cargo test -p agent-drift-analyzer --test input_contract -- --nocapture and cargo test -p agent-drift-analyzer -- --nocapture
- run gitnexus_detect_changes() before handing back

Return with: exact fixes made, verification commands run, residual risks if any, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: split validate_surface into corruption hard-fail vs sparse fail-open (R5.75-2.2)`
- fix commit message example: `fix: address R5.75-2.2 review findings`

Your job is done only when Packet `R5.75-2.2` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 3 Prompt — Task R5.75-2.3 (Guarantee the conservative checkpoint)

````text
/goal Land Packet `R5.75-2.3` from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-2.3` only, assuming Packets `R5.75-2.0`, `R5.75-2.1`, and `R5.75-2.2` are already landed. FIRST read the Packet `R5.75-2.1` finding recorded in the tasks doc — it determines whether this packet is assertion-only or must thread the surface.

Packet authority:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Packet `R5.75-2.3` scope only:
- guarantee a sparse readable session emits >=1 checkpoint with `ProgressStatus::InsufficientEvidence` (or equivalently low-confidence), never a troubleshooting/strong-progress posture
- if the Packet `R5.75-2.1` finding showed the de-aborted pipeline already self-conservatizes: this packet is assertion-only (no source change; the checkpoints regression in `R5.75-2.4` locks it) — confirm and record that
- else: thread `bundle.surface` (or per-session sparsity) into `crates/agent-drift-analyzer/src/lib.rs::analyze_loaded_bundle` to cap a sparse session to `InsufficientEvidence` / `Confidence::Low`, reusing EXISTING surfaces only
- objective text for the conservative checkpoint comes from the existing extractor (post-`R5.75-1`); do not special-case objective assembly here

Primary files for this packet:
- crates/agent-drift-analyzer/src/lib.rs (only if the R5.75-2.1 finding showed over-claiming)
- crates/agent-drift-analyzer/tests/checkpoints.rs (minimal assertion of the conservative outcome)

Out of scope:
- any public schema/`AnalyzerSurface` field change or version bump
- the full locked regression set (Packet `R5.75-2.4`)
- objective-extraction changes (rely on R5.75-1)
- `TaskFrame`/`working_set`/`progress` migration (that is `R5.75-6`/later)

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before modifying any indexed Rust symbol, the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `R5.75-2.3` work before dispatching review. If the R5.75-2.1 finding made this packet assertion-only and no source/test changed, do not invent an empty commit; record that no implementation commit was needed.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must instruct it to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-2.4` until `R5.75-2.3` is review-clean.

Required verification wall for Packet `R5.75-2.3`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-2.3` only from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5.75-2.0/2.1/2.2 are landed.

Use the `$incremental-implementation` skill.

FIRST read the Packet R5.75-2.1 finding in the tasks doc:
- if it says the de-aborted pipeline already emits a conservative InsufficientEvidence checkpoint, this packet is assertion-only: add nothing to src; confirm the conservative outcome (a checkpoints assertion is fine) and record that no implementation commit was needed
- if it says the pipeline over-claims or emits nothing, thread bundle.surface (or per-session sparsity) into crates/agent-drift-analyzer/src/lib.rs::analyze_loaded_bundle so a sparse session caps to ProgressStatus::InsufficientEvidence / Confidence::Low using EXISTING surfaces only — no new enum, no schema bump

Constraints:
- objective text comes from the existing extractor (post-R5.75-1); do not special-case objective assembly
- do not change AnalyzerSurface fields or the public schema

Read first:
- the Packet R5.75-2.1 finding (tasks doc)
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- crates/agent-drift-analyzer/src/lib.rs (analyze_loaded_bundle), crates/agent-drift-analyzer/src/checkpoint/mod.rs
- AGENTS.md

GitNexus: run impact analysis before editing any symbol; report HIGH/CRITICAL; run gitnexus_detect_changes() before handing back.

Precondition check: confirm Packets R5.75-2.0/2.1/2.2 are landed; if a prerequisite is missing, stop and report it instead of compensating here.

Run: cargo test -p agent-drift-analyzer checkpoints -- --nocapture and cargo test -p agent-drift-analyzer -- --nocapture

Return with: whether this was assertion-only or required threading, changed files, verification run, residual risks, and the exact commit message you recommend (or that no implementation commit was needed).
```

Review subagent prompt to send after the implementation commit (or after the verification run if no implementation commit was needed):

```text
/goal Review the already-landed Packet `R5.75-2.3` result in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether it is ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet R5.75-2.3 from:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Focus:
- a sparse readable session emits exactly one conservative checkpoint (InsufficientEvidence / low-confidence), never troubleshooting/strong-progress
- if surface threading was used, it reuses existing surfaces only — no new enum, no schema/version bump, no AnalyzerSurface field change
- the decision (assertion-only vs threaded) matches the recorded R5.75-2.1 finding
- no objective-extraction special-casing crept in
- verification story is sufficient and honest

Review the tests/verification story first, then the diff. List findings by severity. State clearly whether Packet R5.75-2.3 is review-clean or requires changes; keep required changes concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-2.3` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet R5.75-2.3 verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet R5.75-2.3 issues; keep it packet-scoped and reuse existing surfaces only
- do not broaden into R5.75-2.4 or deferred work
- run impact analysis before editing affected Rust symbols
- rerun: cargo test -p agent-drift-analyzer checkpoints -- --nocapture and cargo test -p agent-drift-analyzer -- --nocapture
- run gitnexus_detect_changes() before handing back

Return with: exact fixes made, verification run, residual risks, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `feat: emit conservative checkpoint for sparse readable sessions (R5.75-2.3)`
- fix commit message example: `fix: address R5.75-2.3 review findings`

Your job is done only when Packet `R5.75-2.3` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 4 Prompt — Task R5.75-2.4 (Lock the regressions)

````text
/goal Land Packet `R5.75-2.4` from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` using a packet-scoped implementation -> commit -> review -> fix -> commit loop until review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-2.4` only, assuming Packets `R5.75-2.0` through `R5.75-2.3` are already landed.

Packet authority:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Packet `R5.75-2.4` scope only (tests only; no production code change):
- `tests/input_contract.rs`: lock all three cases — (a) tool-payload axis (objective rows + path hints + zero parseable tool calls, the `f47b81f39f2495dd` shape) returns `Ok`; (a') path-hint axis (objective row + no paths + no tool calls, the conceptual-ask shape) returns `Ok`; (b) a corrupt bundle (unstable/duplicate row refs or broken dedupe) still returns the exact existing `InputError` variant. Each sparsity axis pinned independently.
- `tests/checkpoints.rs`: a sparse readable session (steer ask + pasted `<skill>` body + no tool calls — the minimized `f47b81f39f2495dd` shape) emits exactly one `InsufficientEvidence` checkpoint whose `structured_objective` anchors to the steer ask, with `success_conditions`/`deliverables`/`target` unknown, never anchored to the `<skill>` body (cross-checks the `R5.75-1` anchoring fix)

Primary files for this packet:
- crates/agent-drift-analyzer/tests/input_contract.rs
- crates/agent-drift-analyzer/tests/checkpoints.rs

Out of scope:
- any production source change (those are `R5.75-2.2`/`R5.75-2.3`) — if a test cannot pass, stop and report it as a defect rather than editing production code here
- schema changes
- packets `R5.75-2.5`+

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. Before modifying any indexed Rust symbol (test helpers count), the subagent must run GitNexus impact analysis first and report any HIGH or CRITICAL blast radius before proceeding.
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the subagent finishes, inspect its diff and verification results yourself.
6. Commit the landed Packet `R5.75-2.4` tests before dispatching review.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must instruct it to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. Do not start Packet `R5.75-2.5` until `R5.75-2.4` is review-clean.

Required verification wall for Packet `R5.75-2.4`:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Implementation subagent prompt to send:

```text
/goal Implement Packet `R5.75-2.4` only from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5.75-2.0 through R5.75-2.3 are landed.

Use the `$incremental-implementation` skill.

You are landing only Packet R5.75-2.4 — locking regressions, tests only, no production source change:
- tests/input_contract.rs: (a) tool-payload axis sparse bundle (objective rows + path hints + zero parseable tool calls) returns Ok; (a') path-hint axis sparse bundle (objective row + no paths + no tool calls) returns Ok; (b) a corrupt bundle (unstable/duplicate row refs or broken dedupe) still returns the exact existing InputError variant. Pin each sparsity axis independently.
- tests/checkpoints.rs: a sparse readable session (steer ask + pasted <skill> body + no tool calls, the minimized f47b81f39f2495dd shape) emits exactly one InsufficientEvidence checkpoint whose structured_objective anchors to the steer ask with success_conditions/deliverables/target unknown — never the <skill> body

Read first:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md (Testing Strategy, Success Criteria)
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md (Task R5.75-2.4.1/4.2)
- existing crates/agent-drift-analyzer/tests/input_contract.rs and tests/checkpoints.rs for fixture/helper conventions
- AGENTS.md

If a test cannot pass against current production code, STOP and report it as a defect in R5.75-2.2/2.3 rather than editing production source here.

GitNexus: run impact analysis before editing any indexed test helper; run gitnexus_detect_changes() before handing back.

Precondition check: confirm Packets R5.75-2.0 through R5.75-2.3 are landed; if a prerequisite is missing, stop and report it.

Run the full verification wall above.

Return with: changed test files, verification run, residual risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the implementation commit:

```text
/goal Review the already-landed Packet `R5.75-2.4` regressions in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether they are ready to keep.

Use the `$code-review-and-quality` skill.

Review only Packet R5.75-2.4 from:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Focus:
- both sparsity axes are pinned independently (tool-payload AND path-hint), plus the corruption hard-fail case — the split is provable, not just the relaxation
- the checkpoint test actually asserts InsufficientEvidence AND objective-anchor-to-steer with weak fields unknown (would fail if the objective re-pooled the <skill> body)
- tests-only: no production source changed
- the tests are honest (would fail if the behavior regressed) and not tautological

Review the test design first. List findings by severity. State clearly whether Packet R5.75-2.4 is review-clean or requires changes; keep required changes concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-2.4` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, then rerun the Packet R5.75-2.4 verification wall.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet R5.75-2.4 test issues; tests-only, no production source change
- run impact analysis before editing affected indexed test helpers
- rerun the full verification wall
- run gitnexus_detect_changes() before handing back

Return with: exact fixes made, verification run, residual risks, and the exact commit message you recommend.
```

Commit guidance:
- implementation commit message example: `test: lock sparse fail-open + conservative-checkpoint regressions (R5.75-2.4)`
- fix commit message example: `fix: address R5.75-2.4 review findings`

Your job is done only when Packet `R5.75-2.4` is review-clean and every non-empty implementation/fix batch has been committed.
````

---
## Packet 5 Prompt — Task R5.75-2.5 (Smoke and closeout)

````text
/goal Land Packet `R5.75-2.5` from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` — the smoke + closeout packet — using a validation -> commit -> review -> fix -> commit loop until review-clean.

You are the orchestration agent. Stay strictly scoped to Packet `R5.75-2.5` only, assuming Packets `R5.75-2.0` through `R5.75-2.4` are already landed.

Packet authority:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Packet `R5.75-2.5` scope only (validation + MAP status; no production source change):
- adapted smoke: run compactor -> analyzer -> sentinel for session `f47b81f39f2495dd` and confirm it no longer aborts and emits >=1 conservative checkpoint (inspect `summary.md` objective line + `checkpoints.jsonl`)
- native control smoke: run the same pipeline for `019eb430-6f9a-7a03-9a63-cb451b654795` and confirm normal output is unchanged
- run the full analyzer wall + the touched sentinel spot-checks
- update the `R5.75-2` packet in `docs/specs/r5/R5_75/MAP.md`: promotion status + routing note pointing to `R5.75-3` as the next active seam (mirror the prior promotion style; do not over-edit)

Primary files for this packet:
- docs/specs/r5/R5_75/MAP.md (status/routing only)
- (smoke outputs only, not committed) target/r5_75-smoke/R5.75-2/

Out of scope:
- any production source change (if smoke reveals a defect, STOP and report it as an R5.75-2.2/2.3/2.4 regression rather than fixing here)
- starting `R5.75-3`
- the deeper `R5.75-6` objective-faithfulness work

Hard rules:
1. Spawn a fresh implementation subagent first. Use `GPT-5.4` on `high`.
2. The implementation subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$incremental-implementation` skill.
3. This packet runs the analyzer/sentinel and edits only the MAP; no Rust symbols are modified, so GitNexus impact analysis is not expected — but if any source edit becomes necessary, STOP and report instead (it belongs in an earlier packet).
4. The orchestration agent must not implement locally unless the subagent path is unavailable.
5. After the subagent finishes, inspect the smoke evidence and the MAP diff yourself.
6. Commit the MAP status/routing update before dispatching review. The smoke run itself produces no committed source.
7. Then spawn a fresh review subagent. Use `GPT-5.4` on `high`.
8. The review subagent prompt must start with `/goal ` and must explicitly instruct the subagent to use the `$code-review-and-quality` skill.
9. If the review subagent finds issues, spawn a fresh fix subagent on `GPT-5.4` `high`. Its prompt must start with `/goal ` and must instruct it to use the `$incremental-implementation` skill.
10. Commit every non-empty fix batch before sending a fresh review subagent back through the review loop.
11. Run `gitnexus_detect_changes()` before every commit.
12. R5.75-2 is complete only when this packet is review-clean; do not begin `R5.75-3`.

Required verification wall for Packet `R5.75-2.5`:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Plus the manual smoke pipeline from the SPEC Commands section for both sessions.

Implementation subagent prompt to send:

```text
/goal Run and close out Packet `R5.75-2.5` only from `docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md` in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`, assuming Packets R5.75-2.0 through R5.75-2.4 are landed.

Use the `$incremental-implementation` skill.

You are landing only Packet R5.75-2.5 — smoke + closeout (validation + MAP status; no production source change):
- adapted smoke: compactor -> analyzer -> sentinel for f47b81f39f2495dd; confirm no abort and >=1 conservative checkpoint; inspect summary.md (objective line conservative/unknown) and checkpoints.jsonl
- native control smoke: same pipeline for 019eb430-6f9a-7a03-9a63-cb451b654795; confirm unchanged normal output
- run the full analyzer wall + sentinel spot-checks (warning_policy, live_end_to_end)
- update the R5.75-2 packet in docs/specs/r5/R5_75/MAP.md: promotion status + routing note to R5.75-3 next, matching the existing promotion style

If any smoke/wall fails, STOP and report it as a defect in R5.75-2.2/2.3/2.4 — do NOT edit production source in this packet.

Read first:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md (Commands, Success Criteria)
- docs/specs/r5/R5_75/MAP.md (the R5.75-2 packet + how R5.75-1 recorded its promotion)
- AGENTS.md

Run gitnexus_detect_changes() before handing back.

Precondition check: confirm Packets R5.75-2.0 through R5.75-2.4 are landed; if a prerequisite is missing, stop and report it.

Return with: the smoke outcomes for both sessions, the wall results, the MAP diff, residual risks, and the exact commit message you recommend.
```

Review subagent prompt to send after the MAP commit:

```text
/goal Review the already-landed Packet `R5.75-2.5` closeout in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate` and determine whether R5.75-2 is honestly complete.

Use the `$code-review-and-quality` skill.

Review only Packet R5.75-2.5 from:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md (Success Criteria)
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Focus:
- the smoke evidence genuinely shows f47b81f39f2495dd no longer aborts and emits a conservative checkpoint, and 019eb430 is unchanged
- the full analyzer wall and sentinel spot-checks are green
- the MAP R5.75-2 status/routing update is honest, scoped to status only, and correctly points to R5.75-3 next (no over-editing, no premature R6/R5.75-6 claims)
- no production source was changed in this closeout packet

List findings by severity. State clearly whether Packet R5.75-2.5 (and thus R5.75-2) is review-clean or requires changes; keep required changes concrete and packet-scoped.
```

Fix subagent prompt template to use if review flags issues:

```text
/goal Address the review findings for already-landed Packet `R5.75-2.5` only in `/Users/spensermcconnell/.codex/worktrees/97a0/substrate`.

Use the `$incremental-implementation` skill.

Authoritative packet docs:
- docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
- docs/specs/r5/R5_75/MAP.md
- AGENTS.md

Review findings to fix:
- [PASTE REVIEW FINDINGS HERE]

Rules:
- fix only the flagged Packet R5.75-2.5 issues (MAP wording/scope or re-running smoke/walls); no production source change — escalate any real defect to the owning earlier packet
- run gitnexus_detect_changes() before handing back

Return with: exact fixes made, re-run evidence, residual risks, and the exact commit message you recommend.
```

Commit guidance:
- closeout commit message example: `docs(r5.75-2): promote sparse readable fail-open; route to R5.75-3`
- fix commit message example: `fix: address R5.75-2.5 review findings`

Your job is done only when Packet `R5.75-2.5` is review-clean, R5.75-2 is recorded as promoted in the MAP, and every non-empty batch has been committed.
````
