# Spec: Agent Drift Analyzer Sparse Readable Session Fail-Open (R5.75-2)

Status: draft spec created on 2026-06-21 after closing `R5.75-1` (structured-objective anchoring fix,
promoted) and reproducing the `R5.75-2` abort against the live crate. This spec is the implementation
authority for the `R5.75-2` packet defined in `docs/specs/r5/R5_75/MAP.md`.

Authority order for this packet:
`docs/specs/r5/R5_75/MAP.md` (the `R5.75-2` packet) owns landing order, the named smoke sessions, and
the promotion gate; this SPEC/PLAN/TASKS family owns the implementation contract; the live crate
(`crates/agent-drift-analyzer/src/input.rs`) is the ground truth for current behavior.

## Assumptions I'm Making

1. **Fail-open is scoped to the two "sparse-but-readable" conditions**, not a general relaxation of
   `validate_surface`:
   - `truth_artifact_hints` (no path-like hints survived in directive text), and
   - `working_set_hints` / `tool_argument_json` (no parseable tool-call payloads — both are false when
     a readable session simply has zero `tool_call` rows, which is the confirmed `f47b81f39f2495dd`
     case).
   These two fail open **independently** (resolved decision below): each is a distinct sparsity signal,
   so a session missing either *or both* still produces a conservative checkpoint instead of aborting.
   `f47b81f39f2495dd` has paths but no tool calls (only the tool-payload axis is sparse); a conceptual
   ask ("explain how the retry path works") with no paths and no tool calls is also a legitimate
   readable session, so gating on `truth_artifact_hints` would wrongly abort it. Coupling the two (only
   failing open when both are sparse) is rejected because it would leave `f47b81f39f2495dd` aborting.
2. **Everything else stays hard-fail.** `repetition_preserved` and `stable_row_refs` are structural
   integrity (dedupe/ref corruption), and the upstream `InputError` variants
   (`UnsupportedSchemaVersion`, `MissingArtifact`, `DuplicateSourceFile*`, `UnknownSourceFileId`,
   `UnknownTurnIdRef`, `NoSessions`, `UnstableOrdering`, `MissingDedupeRepresentative`) are corruption.
   None of these are relaxed in this packet.
3. **`literal_objective_rows` stays hard-fail in this packet (resolved decision below).** This is the
   *floor* of "readable": with zero directive rows there is no ask to anchor a conservative checkpoint
   to, and "every user/developer/system row dropped in normalization" is ambiguous between a genuinely
   empty session and a degenerate import — so the conservative choice is to not analyze. The canonical
   sparse case (and the repro) retains objective rows above this floor.
4. **The conservative checkpoint reuses existing surfaces — no schema bump.**
   `ProgressStatus::InsufficientEvidence` (`checkpoint/schema.rs`) and `Confidence::Low` already exist,
   and `R5.75-1` already gives honestly-unknown structured objectives via `ObjectiveUnknown` /
   `objective_class=NotTaskStatement`. The fail-open path must lean on these, not introduce new public
   enum variants or bump the `v0.6`/`v0.2` schema versions.
5. **`AnalyzerSurface` is additive and currently validation-only.** It is computed in `validate_surface`,
   returned on `InputBundle.surface`, and **not read by `analyze_loaded_bundle`** today. The fix keeps
   it additive: the sparse case returns `Ok(AnalyzerSurface { working_set_hints: false, … })` instead of
   `Err`, and the analysis path consults the surface (or per-session sparsity) **only if** smoke shows
   the existing pipeline over-claims on a sparse session.
6. **The conservative checkpoint's structured objective anchors to the real readable ask, not pasted
   boilerplate.** On `f47b81f39f2495dd` the real ask is the `steer` row; the 24KB `<skill>` body is
   boilerplate (negative `objective_score`, excluded by the `R5.75-1` anchoring fix). Weak fields stay
   unknown. This is a cross-check that `R5.75-2` and `R5.75-1` compose, not new objective work.
7. **The deeper input-contract redesign is out of scope.** No new normalization, no widening of the
   public bundle schema, no changes to the compactor.

If any of these assumptions drift, update this spec before implementation.

## Objective

Stop the analyzer from hard-aborting an entire readable session bundle when the only thing missing is
parseable tool-call payloads or path hints. Split structurally-invalid (corrupt) bundle failures —
which must still hard-fail — from semantically-sparse-but-readable bundles, and for the sparse case
emit at least one conservative, low-confidence checkpoint instead of returning an error.

Primary users:

1. operators running the analyzer over real/adapted sessions that legitimately have little or no tool
   activity, who currently get nothing instead of a conservative read;
2. the sentinel/replay path, which should receive a conservative checkpoint rather than a pipeline
   abort for sparse readable sessions;
3. the adapted-external robustness corpus (`R5.75-5`), which needs the sparse-readable class to produce
   output it can lock as a fixture.

This packet succeeds when:

1. the adapted repro `f47b81f39f2495dd` no longer aborts: `analyze_bundle` returns `Ok` and the
   analyzer emits at least one checkpoint;
2. the emitted checkpoint is conservative — `ProgressStatus::InsufficientEvidence` (or equivalently
   low-confidence), never a fabricated strong-progress or troubleshooting posture;
3. the checkpoint's `structured_objective` is honest: anchored to the real readable ask where one
   exists (the steer on `f47b81f39f2495dd`), with weak fields carrying `ObjectiveUnknown`s;
4. genuinely corrupt bundles still hard-fail with the existing `InputError` variants;
5. the native control session `019eb430-6f9a-7a03-9a63-cb451b654795` still produces normal output after
   the contract change;
6. no public schema version is bumped and `AnalyzerSurface` changes stay additive.

## Tech Stack

- Language: Rust 2021
- Primary crate: `agent-drift-analyzer`
- Live code seams for this packet:
  - `crates/agent-drift-analyzer/src/input.rs` (`validate_surface`, `AnalyzerSurface`, `load_bundle`,
    `InputError`)
  - `crates/agent-drift-analyzer/src/lib.rs` (`analyze_loaded_bundle`) — only if the surface must be
    threaded to cap confidence
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/input_contract.rs` (the existing contract-error wall)
- Live docs seam: this SPEC/PLAN/TASKS family plus the `R5.75-2` packet in `docs/specs/r5/R5_75/MAP.md`.

No compactor change, no new dependency, no `TaskFrame`/`working_set`/`progress` migration, and no public
schema version bump belong in this packet.

## Commands

Focused checkpoint + input-contract regressions:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer --test input_contract -- --nocapture
```

Full analyzer wall for closeout:

```bash
cargo test -p agent-drift-analyzer -- --nocapture
```

Touched-surface sentinel spot-checks (the exported checkpoint set changes for sparse sessions):

```bash
cargo test -p agent-drift-sentinel warning_policy -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Repro the failure (current behavior, for grounding):

```bash
export CODEX_HOME="$(pwd)/target/ranga-validation/codex-home"
export SESSION_ID="f47b81f39f2495dd"
export SMOKE_ROOT="target/r5_75-smoke/R5.75-2/$SESSION_ID"
rm -rf "$SMOKE_ROOT"
cargo run -p agent-session-compactor -- --codex-home "$CODEX_HOME" --session-id "$SESSION_ID" --output-dir "$SMOKE_ROOT/compactor"
cargo run -p agent-drift-analyzer -- --input-dir "$SMOKE_ROOT/compactor" --output-dir "$SMOKE_ROOT/analyzer"
```

## Project Structure

```text
docs/specs/r5/R5_75/MAP.md
  Landing-order authority: the R5.75-2 packet (problem, required change, smoke sessions, promotion gate).

docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-spec.md
docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-plan.md
docs/specs/r5/R5_75/R5_75-2/agent-drift-analyzer-sparse-readable-fail-open-tasks.md
  This packet's implementation authority.

crates/agent-drift-analyzer/src/input.rs
  validate_surface (the 5 contract checks + the InsufficientContract aborts), AnalyzerSurface, load_bundle.
  The fail-open split lands here.

crates/agent-drift-analyzer/src/lib.rs
  analyze_loaded_bundle: the analysis loop. Touch only if the sparse surface must cap checkpoint confidence.

crates/agent-drift-analyzer/tests/input_contract.rs
  Existing contract-error wall: must still prove corrupt bundles hard-fail, and now prove sparse-readable fail-opens.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Conservative-checkpoint regression: a sparse readable session emits one InsufficientEvidence checkpoint with an honest objective.
```

## Code Style

Prefer turning a sparse-but-readable signal into an explicit `Ok` surface flag over an early `Err`, and
keep corruption checks returning `Err` exactly as before. The split should read as two clearly-labeled
buckets, not a softened single check.

```rust
// Corruption → hard-fail (unchanged): structural integrity the analyzer cannot trust.
if !repetition_preserved {
    return Err(InputError::InsufficientContract {
        reason: "archival rows do not preserve repetition beyond the compacted view".to_string(),
    });
}
if !stable_row_refs {
    return Err(InputError::InsufficientContract {
        reason: "row references are not unique and stable".to_string(),
    });
}

// Sparse-but-readable → fail open: record the weakness on the surface, do not abort.
// working_set_hints / tool_argument_json / truth_artifact_hints may all be false here; the
// downstream analysis stays conservative (InsufficientEvidence) rather than the bundle aborting.
Ok(AnalyzerSurface {
    literal_objective_rows,
    truth_artifact_hints,
    working_set_hints,
    repetition_preserved,
    stable_row_refs,
    tool_argument_json,
})
```

Conventions for this packet:

- Keep `AnalyzerSurface` additive: same fields, same types; the sparse case just stops being an `Err`.
- Reuse `ProgressStatus::InsufficientEvidence` / `Confidence::Low`; do not invent a new conservative enum.
- Keep corruption hard-fails verbatim so the `input_contract` wall stays meaningful.
- Let the existing objective extraction (post-`R5.75-1`) produce the conservative checkpoint's objective;
  do not special-case objective text in the fail-open path.

## Testing Strategy

1. **Input-contract split regressions** (`tests/input_contract.rs`) — pin each sparsity axis independently:
   - the tool-payload axis: a readable bundle with objective rows + path hints but **zero parseable
     tool-call payloads** (the `f47b81f39f2495dd` shape) returns `Ok` instead of `InsufficientContract`;
   - the path-hint axis: a readable bundle with an objective row but **no path hints and no tool calls**
     (the conceptual-ask shape) also returns `Ok`;
   - a corrupt bundle (unstable/duplicate row refs, broken dedupe, bad schema) still returns the exact
     existing `InputError` variant.

2. **Conservative-checkpoint regression** (`tests/checkpoints.rs`)
   - a sparse readable session emits at least one checkpoint;
   - its progress status is `InsufficientEvidence` (no troubleshooting/strong-progress escalation);
   - its `structured_objective` anchors to the real ask with weak fields unknown.

3. **Native control**
   - `019eb430` still produces normal output (manual smoke), proving the relaxed contract did not change
     ordinary runs.

4. **Full analyzer + touched sentinel walls**
   - `cargo test -p agent-drift-analyzer -- --nocapture`
   - `cargo test -p agent-drift-sentinel warning_policy live_end_to_end -- --nocapture`

## Boundaries

- **Always:**
  - verify `R5.75-1` is landed in live code/tests before editing (it is, as of commit `68216bf02`); if a
    named prerequisite were missing, stop and report rather than compensating here;
  - keep corruption checks (`repetition_preserved`, `stable_row_refs`, schema/artifact/dedupe/ordering)
    hard-failing with their existing `InputError` variants and messages;
  - keep `AnalyzerSurface` and the public bundle schema additive (no version bump);
  - emit a conservative checkpoint via existing insufficient-evidence surfaces;
  - run the focused checkpoint + input-contract walls before calling the packet complete.

- **Ask first:**
  - relaxing `literal_objective_rows` to fail-open (currently hard-fail per Assumption 3);
  - threading `AnalyzerSurface` into `analyze_loaded_bundle` to cap confidence (only if smoke shows the
    pipeline over-claims on a sparse session);
  - any change to the conservative checkpoint's exported shape beyond setting status/confidence.

- **Never:**
  - widen or version-bump the public bundle/checkpoint schema to accommodate the fail-open;
  - weaken a corruption check to make a malformed bundle pass;
  - fabricate strong progress, a concrete target, or a non-empty success/deliverable set for a sparse
    session;
  - touch the compactor or introduce a new normalization path.

## Success Criteria

1. `load_bundle` returns `Ok` for the sparse-but-readable class (no parseable tool-call payloads and/or
   no path hints, objective rows present); `analyze_bundle` on `f47b81f39f2495dd` returns `Ok` and emits
   ≥1 checkpoint.
2. That checkpoint is `ProgressStatus::InsufficientEvidence` / low-confidence with a structured objective
   anchored to the real ask and weak fields unknown.
3. Corrupt bundles still hard-fail with their existing `InputError` variants (proven by retained
   `input_contract` cases).
4. The native control `019eb430` is unchanged.
5. `cargo test -p agent-drift-analyzer -- --nocapture` and the touched sentinel spot-checks are green;
   no schema version bumped.

## Resolved Decisions

1. **Resolved (2026-06-21): `literal_objective_rows == false` stays hard-fail.** The opposing case —
   that "no objective rows" is sparsity not corruption, the pipeline already degrades gracefully
   ("No objective row available"), and an anchorless conservative checkpoint is still honest — was
   considered and declined: zero directive rows is the floor below "readable" (no anchor) and is
   ambiguous with a degenerate import, so the conservative choice is to not analyze. Revisit only if a
   real readable-but-objectiveless repro appears (Deferred / Ask-First in TASKS).
2. **Resolved (2026-06-21): `truth_artifact_hints` and the tool-payload pair fail open independently.**
   Coupling (only failing open when both axes are sparse) is rejected because it would leave the
   `f47b81f39f2495dd` repro aborting (its truth axis is not sparse), and keeping `truth_artifact_hints`
   as a gate would wrongly abort a legitimate conceptual-ask session (objective row, no paths, no tool
   calls). Both axes get an independent fail-open regression shape.

## Open Questions

1. Does the existing analysis pipeline already produce a conservative checkpoint for a tool-call-free
   session once `load_bundle` stops aborting, or must `analyze_loaded_bundle` read `bundle.surface` to
   cap confidence? (Resolve empirically in PLAN step R5.75-2.1 before writing the conservative-checkpoint
   code.)
