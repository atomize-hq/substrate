# Plan: Agent Drift Analyzer Sparse Readable Session Fail-Open (R5.75-2)

Status: draft plan created on 2026-06-21 after reproducing the `R5.75-2` abort on the live crate
(`f47b81f39f2495dd` hard-aborts at `validate_surface`) and reading the analyzer pipeline end to end.
This plan is reviewable: it should be possible to read it and say "yes, that approach" or "no, change
X" before any code lands.

## Objective

Split structurally-invalid bundle failures (which keep hard-failing) from semantically-sparse-but-
readable bundles (which fail open with one conservative checkpoint), with the smallest additive change
to `validate_surface` and the analysis path, and no public schema bump.

## Planning Decisions Locked For This Draft

1. The fail-open set is exactly two conditions from `validate_surface`: the path-hint check
   (`truth_artifact_hints`) and the tool-payload pair (`working_set_hints` / `tool_argument_json`).
   They fail open **independently** (resolved 2026-06-21): coupling them would leave the repro aborting
   (its truth axis is not sparse), and gating on `truth_artifact_hints` would wrongly abort a legitimate
   conceptual-ask session (objective row, no paths, no tool calls). Each axis gets its own fail-open
   regression shape.
2. The corruption/integrity set stays hard-fail verbatim — but `repetition_preserved` is **split**
   (resolved 2026-06-21, from the Codex review): only the archival-coverage half (`archival >= compact`)
   stays a hard-fail; the `!dedupe_groups.is_empty()` half is dropped because a no-duplicate bundle
   legitimately has zero dedupe groups (and the conceptual-ask sparse axis may have none). Remaining
   hard-fails: `stable_row_refs`, archival-coverage, and every upstream `InputError` variant (schema,
   missing artifact, duplicate ids, unknown refs, ordering, `MissingDedupeRepresentative`, no sessions).
   Dedupe-ref integrity is unaffected (still caught by `MissingDedupeRepresentative` /
   `validate_dedupe_refs`), so genuine ref/dedupe corruption still hard-fails.
3. `literal_objective_rows` (the directive-row floor) stays hard-fail in this packet (Spec Assumption 3 /
   Resolved Decision 1).
4. The conservative checkpoint's contract is the `ProgressStatus::InsufficientEvidence` status
   specifically (distinct from `Confidence`) + the `R5.75-1` `ObjectiveUnknown` machinery. No new public
   enum, no schema version bump.
5. The conservative decision is **per session**, not bundle-wide (resolved 2026-06-21, from the Codex
   review). `AnalyzerSurface` is one bundle-wide field computed across all compact rows and is currently
   **not consumed** by `analyze_loaded_bundle`. The bundle-wide surface is fine for the abort/no-abort
   gate, but the per-session conservative cap is derived from each session's own rows — never by
   threading the single bundle-wide surface. Whether that cap needs production code at all is resolved
   empirically in step R5.75-2.1.
6. Objective text for the conservative checkpoint comes from the existing extractor (post-`R5.75-1`);
   the fail-open path does not special-case objective assembly.
7. Packet-prompt rule: verify `R5.75-1` is landed before editing; it is (commit `68216bf02`).

## Why This Packet Exists

`load_bundle` → `validate_surface` aborts the entire bundle when tool-call payloads are not parseable,
even for a readable session with real objective rows. Confirmed against `f47b81f39f2495dd`:

- compactor exits 0 and emits 6 compact rows (3 `user_message` incl. a real steer ask, a 24KB `<skill>`
  body, plus status/turn_context rows) — readable, not corrupt;
- analyzer exits 1 with `InsufficientContract { reason: "tool-call argument payloads are not parseable
  enough to infer command families and working-set paths" }` ([input.rs:483]);
- no analyzer output directory is created — zero checkpoints.

Checks 1 (`literal_objective_rows`) and 2 (`truth_artifact_hints`) pass; check 3 fires only because the
session has zero `tool_call` rows. That is the textbook sparse-but-readable case the packet targets.

## Dependency Graph

```text
docs lock (this SPEC/PLAN/TASKS)
  -> empirical probe: does the pipeline self-conservatize once the abort is removed?
  -> validate_surface split (corruption Err vs sparse Ok)
  -> conservative checkpoint guarantee (no-op if probe shows self-conservatism; else thread surface)
  -> input_contract + checkpoints regressions
  -> smoke (f47b81f39f2495dd fail-open, 019eb430 control) + full + sentinel walls

explicitly deferred / out of scope:
  -> relaxing literal_objective_rows
  -> any public schema / AnalyzerSurface field addition
  -> TaskFrame / working_set / progress migration
  -> compactor or normalization changes
```

## Recommended Landing Sequence

## R5.75-2.0: Docs Lock (This SPEC / PLAN / TASKS)

### Scope

- commit the bounded SPEC/PLAN/TASKS family under `docs/specs/r5/R5_75/R5_75-2/`
- record the corruption-vs-sparse split and the additive-only constraint before code

### Why First

The MAP packet is a landing-order definition, not an implementation contract. The split decision and
the "no schema bump / reuse InsufficientEvidence" boundary must be explicit before editing.

### Verification

Manual review against `docs/specs/r5/R5_75/MAP.md` (the `R5.75-2` packet) and the live `input.rs`.

## R5.75-2.1: Characterize Downstream Behavior Of A De-Aborted Sparse Session

### Scope

- determine, empirically, what `analyze_loaded_bundle` produces for `f47b81f39f2495dd` if
  `validate_surface` did not abort (e.g. a temporary local relax, or a unit test over a hand-built
  sparse bundle) — **investigation only, reverted before the real change**
- answer Spec Open Question 1: does the de-aborted pipeline score the sparse session **conservatively on
  its own**, or does it **over-claim** (troubleshooting/strong-progress)? The pipeline emits ≥1 window
  for any non-empty session (`checkpoint_windows` always returns at least one), so "emits nothing" is not
  a realistic outcome — frame the finding as over-claim vs self-conservative.

### Primary Files

```text
crates/agent-drift-analyzer/src/lib.rs        (analyze_loaded_bundle, read-only)
crates/agent-drift-analyzer/src/checkpoint/   (checkpoint_analyses / scoring, read-only)
```

### Why Before The Split

The size of step R5.75-2.3 depends entirely on this answer. If the pipeline self-conservatizes (no
verifier/edit evidence → `InsufficientEvidence`), the fix is almost entirely in `validate_surface`. If
it over-claims, a **per-session** conservative cap is added to the analysis (derived from each session's
own rows — never the bundle-wide surface). Decide with evidence, not assumption.

### Verification

Local experiment + recorded finding in the TASKS ledger; no committed code from this step.

## R5.75-2.2: Split `validate_surface` Into Corruption Hard-Fail vs Sparse Fail-Open

### Scope

- keep `stable_row_refs` and all upstream `InputError` variants returning `Err` exactly as today
- **split `repetition_preserved`**: keep the archival-coverage half (`archival >= compact`) as a
  hard-fail; drop the `!dedupe_groups.is_empty()` half so a no-duplicate bundle no longer aborts
  (dedupe-ref integrity stays enforced by `MissingDedupeRepresentative` / `validate_dedupe_refs`)
- stop returning `Err` for the path-hint and tool-payload conditions; let `validate_surface` return
  `Ok(AnalyzerSurface { … })` with those flags `false`
- keep `literal_objective_rows` (directive-row floor) hard-fail
- add the minimal `input_contract` proof for the tool-payload axis here (TDD for this slice); the full
  matrix is consolidated in R5.75-2.4

### Primary Files

```text
crates/agent-drift-analyzer/src/input.rs
crates/agent-drift-analyzer/tests/input_contract.rs   (minimal tool-payload-axis proof only)
```

### Why Here

This is the single change that turns the abort into a recorded-weakness surface. It is additive: the
`AnalyzerSurface` shape is unchanged; the early-returns are removed for the two sparse conditions and the
`repetition_preserved` hard-fail is narrowed to the archival-coverage invariant.

### Verification

```bash
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.75-2.3: Guarantee The Conservative Checkpoint

### Scope

- if R5.75-2.1 showed self-conservatism: no code change — the de-aborted pipeline already emits an
  `InsufficientEvidence` checkpoint; just assert it
- else: add a **per-session** conservative cap in `analyze_loaded_bundle` so a sparse session's
  checkpoint status caps to `ProgressStatus::InsufficientEvidence`, deriving the sparsity signal from
  that session's own rows (never the single bundle-wide `AnalyzerSurface`), reusing existing surfaces
  only

### Primary Files

```text
crates/agent-drift-analyzer/src/lib.rs        (only if needed; per-session, not bundle.surface)
```

### Why After The Split

The split makes the sparse session analyzable; this step only ensures the result is honest. Keeping it
conditional avoids speculative plumbing.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## R5.75-2.4: Regressions

### Scope

- **complete the matrix** R5.75-2.2/2.3 began (do not re-add their tests): R5.75-2.2 already proved the
  tool-payload axis. Here, add to `tests/input_contract.rs`: the path-hint axis (objective row + no
  paths + no tool calls, the conceptual-ask shape) returns `Ok`; a clean **no-duplicate** bundle (empty
  `dedupe_groups`) returns `Ok` (locks the `repetition_preserved` split); and a corrupt bundle
  (non-unique refs / missing dedupe representative / `archival < compact` / bad schema) still hard-fails
- `tests/checkpoints.rs`: a sparse readable session emits ≥1 conservative checkpoint (the minimized
  `f47b81f39f2495dd` fixture yields exactly one) whose status is `ProgressStatus::InsufficientEvidence`
  and whose structured objective anchors to the real ask with weak fields unknown (cross-checks
  `R5.75-1` composition)

### Primary Files

```text
crates/agent-drift-analyzer/tests/input_contract.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
```

## R5.75-2.5: Smoke And Closeout

### Scope

- adapted smoke: `f47b81f39f2495dd` no longer aborts, emits ≥1 conservative checkpoint
- native control smoke: `019eb430-6f9a-7a03-9a63-cb451b654795` unchanged
- full analyzer wall + touched sentinel spot-checks green
- update the MAP `R5.75-2` promotion status and routing note

### Primary Files

```text
docs/specs/r5/R5_75/MAP.md   (status/routing only at closeout)
```

### Verification

```bash
cargo test -p agent-drift-analyzer -- --nocapture
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
# + the manual smoke pipeline from the SPEC Commands section for both sessions
```

## Risks And Mitigations

### Risk: relaxing the abort lets a genuinely useless bundle through

Mitigation: only the two named sparse conditions fail-open; corruption and `literal_objective_rows`
stay hard-fail. The conservative checkpoint is `InsufficientEvidence`, so a near-empty session yields an
honest "not enough evidence" read, not fabricated progress.

### Risk: the de-aborted pipeline over-claims progress on a sparse session

Mitigation: that is exactly what R5.75-2.1 probes before any code lands; if it over-claims, R5.75-2.3
adds a per-session cap to `ProgressStatus::InsufficientEvidence` (from that session's rows). The
checkpoints regression locks the conservative outcome.

### Risk: a no-duplicate sparse session still aborts at `repetition_preserved`

Mitigation (from the Codex review): the `f47b81f39f2495dd` repro survives only because it contains a
duplicate (`dedupe_groups` non-empty); the conceptual-ask axis may have none. R5.75-2.2 splits
`repetition_preserved` so the `!dedupe_groups.is_empty()` half no longer aborts, and R5.75-2.4 locks a
no-duplicate-bundle `Ok` case. Genuine ref/dedupe corruption still hard-fails via
`MissingDedupeRepresentative`.

### Risk: the conservative checkpoint's objective re-pools boilerplate (the `<skill>` body)

Mitigation: `R5.75-1` already excludes negative-`objective_score` rows from the goal; the regression
asserts the objective anchors to the steer with weak fields unknown, so a regression here fails the
build.

### Risk: scope creep into an input-contract redesign

Mitigation: Boundaries forbid schema bumps, `AnalyzerSurface` field additions, and compactor changes;
`literal_objective_rows` relaxation is explicitly ask-first.

## Verification Checkpoints

1. **After R5.75-2.1** — recorded finding: pipeline self-conservatizes or not; R5.75-2.3 sized accordingly.
2. **After R5.75-2.2** — `input_contract` proves the tool-payload-axis sparse bundle → `Ok`, corrupt →
   existing `Err`, and the `repetition_preserved` split holds (no-duplicate bundle no longer aborts).
3. **After R5.75-2.3/2.4** — checkpoints prove ≥1 conservative checkpoint (fixture yields one) with an
   `InsufficientEvidence` status and an honest objective; both sparsity axes locked in `input_contract`.
4. **Packet closeout** — full analyzer wall + sentinel spot-checks green; both smoke sessions behave per
   the SPEC success criteria; MAP status updated.

## Out Of Scope

- relaxing `literal_objective_rows` (ask-first)
- any public bundle/checkpoint schema change or version bump
- adding fields to `AnalyzerSurface`, or threading the bundle-wide `AnalyzerSurface` into per-session
  scoring (the per-session cap derives from each session's own rows)
- weakening dedupe-ref integrity: `MissingDedupeRepresentative` / `validate_dedupe_refs` stay hard-fail
- `TaskFrame` / `working_set` / `progress` comparability migration (that is `R5.75-6` / later)
- compactor or normalization changes
