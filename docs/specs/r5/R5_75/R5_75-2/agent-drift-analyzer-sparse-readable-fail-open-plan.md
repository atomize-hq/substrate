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
2. The corruption set stays hard-fail verbatim: `repetition_preserved`, `stable_row_refs`, and every
   upstream `InputError` variant (schema, missing artifact, duplicate ids, unknown refs, ordering,
   dedupe representative, no sessions).
3. `literal_objective_rows` stays hard-fail in this packet (Spec Assumption 3 / Resolved Decision 1).
4. The conservative checkpoint reuses `ProgressStatus::InsufficientEvidence` + `Confidence::Low` +
   the `R5.75-1` `ObjectiveUnknown` machinery. No new public enum, no schema version bump.
5. `AnalyzerSurface` stays additive. It is currently computed and returned but **not consumed** by
   `analyze_loaded_bundle`; whether the analysis path must start reading it is resolved empirically in
   step R5.75-2.1, not assumed.
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
- answer Spec Open Question 1: does `checkpoint_analyses` already emit ≥1 conservative checkpoint for a
  tool-call-free session, or does the analysis over-claim / emit nothing?

### Primary Files

```text
crates/agent-drift-analyzer/src/lib.rs        (analyze_loaded_bundle, read-only)
crates/agent-drift-analyzer/src/checkpoint/   (checkpoint_analyses / scoring, read-only)
```

### Why Before The Split

The size of step R5.75-2.3 depends entirely on this answer. If the pipeline self-conservatizes (no
verifier/edit evidence → `InsufficientEvidence`), the fix is almost entirely in `validate_surface`. If
it over-claims, the surface must be threaded into the analysis. Decide with evidence, not assumption.

### Verification

Local experiment + recorded finding in the TASKS ledger; no committed code from this step.

## R5.75-2.2: Split `validate_surface` Into Corruption Hard-Fail vs Sparse Fail-Open

### Scope

- keep `repetition_preserved` / `stable_row_refs` (and all upstream `InputError` variants) returning
  `Err` exactly as today
- stop returning `Err` for the path-hint and tool-payload conditions; let `validate_surface` return
  `Ok(AnalyzerSurface { … })` with those flags `false`
- keep `literal_objective_rows` hard-fail

### Primary Files

```text
crates/agent-drift-analyzer/src/input.rs
```

### Why Here

This is the single change that turns the abort into a recorded-weakness surface. It is additive: the
`AnalyzerSurface` shape is unchanged; only the early-return is removed for the two sparse conditions.

### Verification

```bash
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## R5.75-2.3: Guarantee The Conservative Checkpoint

### Scope

- if R5.75-2.1 showed self-conservatism: no code change — the de-aborted pipeline already emits a
  low-confidence / `InsufficientEvidence` checkpoint; just assert it
- else: thread `bundle.surface` (or per-session sparsity) into `analyze_loaded_bundle` so a sparse
  session caps to `InsufficientEvidence` / `Confidence::Low`, reusing existing surfaces only

### Primary Files

```text
crates/agent-drift-analyzer/src/lib.rs        (only if needed)
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

- `tests/input_contract.rs`: add **two** sparse-readable-fail-open cases — the tool-payload axis
  (objective rows + path hints + zero parseable tool calls, the `f47b81f39f2495dd` shape) and the
  path-hint axis (objective row + no paths + no tool calls, the conceptual-ask shape) — both return
  `Ok`; keep a corrupt-bundle case that still hard-fails (prove the split, not just the relaxation)
- `tests/checkpoints.rs`: a sparse readable session emits exactly one conservative checkpoint whose
  status is `InsufficientEvidence` and whose structured objective anchors to the real ask with weak
  fields unknown (cross-checks `R5.75-1` composition)

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
caps confidence via the surface. The checkpoints regression locks the conservative outcome.

### Risk: the conservative checkpoint's objective re-pools boilerplate (the `<skill>` body)

Mitigation: `R5.75-1` already excludes negative-`objective_score` rows from the goal; the regression
asserts the objective anchors to the steer with weak fields unknown, so a regression here fails the
build.

### Risk: scope creep into an input-contract redesign

Mitigation: Boundaries forbid schema bumps, `AnalyzerSurface` field additions, and compactor changes;
`literal_objective_rows` relaxation is explicitly ask-first.

## Verification Checkpoints

1. **After R5.75-2.1** — recorded finding: pipeline self-conservatizes or not; R5.75-2.3 sized accordingly.
2. **After R5.75-2.2** — `input_contract` proves sparse → `Ok`, corrupt → existing `Err`.
3. **After R5.75-2.3/2.4** — checkpoints prove one conservative checkpoint with an honest objective.
4. **Packet closeout** — full analyzer wall + sentinel spot-checks green; both smoke sessions behave per
   the SPEC success criteria; MAP status updated.

## Out Of Scope

- relaxing `literal_objective_rows` (ask-first)
- any public bundle/checkpoint schema change or version bump
- adding fields to `AnalyzerSurface`
- `TaskFrame` / `working_set` / `progress` comparability migration (that is `R5.75-6` / later)
- compactor or normalization changes
