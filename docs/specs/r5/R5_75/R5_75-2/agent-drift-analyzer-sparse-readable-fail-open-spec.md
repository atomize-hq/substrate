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
   - `truth_artifact_hints` (no path-like hints survived in **any compact-row text** — the live check
     scans all compact rows, not only directive rows), and
   - `working_set_hints` / `tool_argument_json` (no parseable tool-call payloads — both are false when
     a readable session simply has zero `tool_call` rows, which is the confirmed `f47b81f39f2495dd`
     case).
   These two fail open **independently** (resolved decision below): each is a distinct sparsity signal,
   so a session missing either *or both* still produces a conservative checkpoint instead of aborting.
   `f47b81f39f2495dd` has paths but no tool calls (only the tool-payload axis is sparse); a conceptual
   ask ("explain how the retry path works") with no paths and no tool calls is also a legitimate
   readable session, so gating on `truth_artifact_hints` would wrongly abort it. Coupling the two (only
   failing open when both are sparse) is rejected because it would leave `f47b81f39f2495dd` aborting.
2. **Corruption / integrity stays hard-fail — but `repetition_preserved` is split (resolved decision
   below).** The live check is `archival_rows.len() >= compact_rows.len() && !dedupe_groups.is_empty()`.
   Only the first half is an integrity invariant (compaction cannot emit more rows than the source) and
   stays hard-fail; the `!dedupe_groups.is_empty()` half is **not** corruption — a bundle with no
   duplicate rows legitimately yields zero dedupe groups (and our own conceptual-ask sparse axis may
   have none), so it must not abort. Dedupe-*reference* integrity is already enforced separately by
   `MissingDedupeRepresentative` / `validate_dedupe_refs`, so dropping the emptiness coupling does not
   weaken corruption detection. The remaining hard-fails are: `stable_row_refs`, the archival-coverage
   invariant (`archival >= compact`), and the upstream `InputError` variants (`UnsupportedSchemaVersion`,
   `MissingArtifact`, `DuplicateSourceFile*`, `UnknownSourceFileId`, `UnknownTurnIdRef`, `NoSessions`,
   `UnstableOrdering`, `MissingDedupeRepresentative`).
3. **`literal_objective_rows` stays hard-fail in this packet (resolved decision below).** This is the
   *directive-row floor* of "readable": the live check is "at least one non-empty user/developer/system
   row" (it does not prove a real *objective* survived, only a directive row). With zero directive rows
   there is no ask to anchor a conservative checkpoint to, and "every directive row dropped in
   normalization" is ambiguous between a genuinely empty session and a degenerate import — so the
   conservative choice is to not analyze. The canonical sparse case (and the repro) retains directive
   rows above this floor.
4. **The conservative checkpoint reuses existing surfaces — no schema bump.** The contract is the
   `ProgressStatus::InsufficientEvidence` status specifically (a distinct field from `Confidence`; a
   low-confidence `Mixed`/`Stalled` result is **not** the same contract). `InsufficientEvidence`
   (`checkpoint/schema.rs`) already exists, and `R5.75-1` already gives honestly-unknown structured
   objectives via `ObjectiveUnknown` / `objective_class=NotTaskStatement`. The fail-open path must lean
   on these, not introduce new public enum variants or bump the `v0.6`/`v0.2` schema versions.
5. **Per-session sparsity, not the bundle-wide surface.** `AnalyzerSurface` is computed in
   `validate_surface` **across all compact rows**, returned as a single bundle-wide `InputBundle.surface`
   field, and **not read by `analyze_loaded_bundle`** today. The bundle-wide surface is fine for the
   abort/no-abort *gate*, but the per-session conservative-checkpoint decision must be derived from
   **that session's own rows** in the analysis loop — never by threading the one bundle-wide
   `AnalyzerSurface`, which in a multi-session bundle would cap the wrong session or let one session's
   tool activity mask another's sparsity. The change stays additive (no `AnalyzerSurface` field added,
   no schema bump).
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
emit at least one conservative `InsufficientEvidence` checkpoint instead of returning an error.

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
2. the emitted checkpoint is conservative — its progress status is `ProgressStatus::InsufficientEvidence`
   (the status field specifically, not merely a low `Confidence`), never a fabricated strong-progress or
   troubleshooting posture;
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
// Corruption / integrity → hard-fail: structural invariants the analyzer cannot trust.
// NOTE the repetition_preserved split: only the archival-coverage half is an invariant.
// `!dedupe_groups.is_empty()` is NOT corruption (a no-duplicate bundle legitimately has none),
// so it is dropped from the hard-fail. Dedupe-ref integrity is still caught by
// MissingDedupeRepresentative / validate_dedupe_refs upstream.
if archival_rows.len() < compact_rows.len() {
    return Err(InputError::InsufficientContract {
        reason: "archival rows do not cover the compacted view".to_string(),
    });
}
if !stable_row_refs {
    return Err(InputError::InsufficientContract {
        reason: "row references are not unique and stable".to_string(),
    });
}

// Sparse-but-readable → fail open: record the weakness on the surface, do not abort.
// working_set_hints / tool_argument_json / truth_artifact_hints may all be false here, and
// dedupe_groups may be empty; the per-session analysis stays conservative (InsufficientEvidence)
// rather than the bundle aborting.
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
- Pin the conservative contract to the `ProgressStatus::InsufficientEvidence` status (a distinct field
  from `Confidence`); do not invent a new conservative enum.
- Keep the remaining corruption hard-fails (`stable_row_refs`, archival-coverage, dedupe-ref integrity,
  schema/artifact/ordering) verbatim so the `input_contract` wall stays meaningful; only the
  `dedupe_groups`-emptiness half of `repetition_preserved` stops aborting.
- Derive the conservative decision per session (from that session's rows), not from the bundle-wide
  `AnalyzerSurface`.
- Let the existing objective extraction (post-`R5.75-1`) produce the conservative checkpoint's objective;
  do not special-case objective text in the fail-open path.

## Testing Strategy

1. **Input-contract split regressions** (`tests/input_contract.rs`) — pin each sparsity axis independently:
   - the tool-payload axis: a readable bundle with objective rows + path hints but **zero parseable
     tool-call payloads** (the `f47b81f39f2495dd` shape) returns `Ok` instead of `InsufficientContract`;
   - the path-hint axis: a readable bundle with an objective row but **no path hints and no tool calls**
     (the conceptual-ask shape) also returns `Ok`;
   - a corrupt bundle (non-unique/unstable row refs, a dedupe-audit entry referencing a missing archival
     row, bad schema, `archival < compact`) still returns the exact existing `InputError` variant;
   - a clean **no-duplicate** bundle (objective rows present, `dedupe_groups` empty) returns `Ok` — proof
     the `repetition_preserved` split landed and the dedupe-emptiness half no longer aborts.

2. **Conservative-checkpoint regression** (`tests/checkpoints.rs`)
   - a sparse readable session emits at least one checkpoint;
   - its progress status is `ProgressStatus::InsufficientEvidence` (the status field, not just a low
     `Confidence`; no troubleshooting/strong-progress escalation);
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
  - keep the corruption/integrity checks (`stable_row_refs`, the archival-coverage invariant,
    dedupe-ref integrity via `MissingDedupeRepresentative`, schema/artifact/ordering) hard-failing with
    their existing `InputError` variants and messages;
  - keep `AnalyzerSurface` and the public bundle schema additive (no version bump);
  - emit a conservative checkpoint via existing insufficient-evidence surfaces, decided per session;
  - run the focused checkpoint + input-contract walls before calling the packet complete.

- **Ask first:**
  - relaxing `literal_objective_rows` to fail-open (currently hard-fail per Assumption 3);
  - whether the per-session conservative cap needs production code in `analyze_loaded_bundle` at all
    (only if the R5.75-2.1 probe shows the de-aborted pipeline over-claims on a sparse session);
  - any change to the conservative checkpoint's exported shape beyond setting the conservative status.

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
2. That checkpoint's status is `ProgressStatus::InsufficientEvidence` (the status field, not merely a low
   `Confidence`) with a structured objective anchored to the real ask and weak fields unknown.
3. Corrupt bundles still hard-fail with their existing `InputError` variants (proven by retained
   `input_contract` cases), while a clean no-duplicate bundle (empty `dedupe_groups`) returns `Ok`.
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
3. **Resolved (2026-06-21, from the Codex review): split `repetition_preserved`.** The live check
   `archival_rows.len() >= compact_rows.len() && !dedupe_groups.is_empty()` couples a real integrity
   invariant with a non-invariant. Keep the archival-coverage half (`archival >= compact`) as a
   hard-fail; drop `!dedupe_groups.is_empty()` from the hard-fail because a bundle with no duplicate rows
   legitimately produces zero dedupe groups (and the conceptual-ask sparse axis may have none, so leaving
   it would re-abort that axis at this check). Dedupe-*reference* integrity is unaffected — it is still
   enforced by `MissingDedupeRepresentative` / `validate_dedupe_refs` — so genuine ref/dedupe corruption
   still hard-fails.
4. **Resolved (2026-06-21, from the Codex review): the conservative decision is per session, not
   bundle-wide.** `AnalyzerSurface` is one bundle-wide field computed across all compact rows; the
   per-session conservative-checkpoint cap is derived from each session's own rows in the analysis loop.
   Threading the single bundle-wide surface would cap the wrong session or mask sparsity in a
   multi-session bundle.

## Open Questions

1. Once `load_bundle` stops aborting, does the existing analysis pipeline score a tool-call-free session
   **conservatively on its own**, or does it **over-claim** (e.g. troubleshooting/strong-progress)? The
   pipeline is known to emit ≥1 checkpoint for any non-empty session — `checkpoint_windows` returns at
   least one window — so "emits nothing" is not a realistic outcome; the open question is over-claim vs
   self-conservative. (Resolve empirically in PLAN step R5.75-2.1 before writing the per-session
   conservative cap.)
