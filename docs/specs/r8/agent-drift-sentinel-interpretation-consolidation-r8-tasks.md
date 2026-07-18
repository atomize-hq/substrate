# Tasks: R8 Sentinel Interpretation Consolidation

Canonical path:
`docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-tasks.md`

Status: **R8-IMPLEMENT COMPLETE / R8 FAMILY TERMINALLY COMPLETE / R8-6 RECEIPT `549160ebc` FRESH
INDEPENDENT BUILT-IN `default` `CLEAN` / ALL `57` IMPLEMENTATION CHECKBOXES COMPLETE / TERMINAL
TRANSITION COMMIT `65eac5ab` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` / R8-IMPLEMENT EXIT GATE
REVIEW-CLEAN / CURRENT REVIEW RECEIPT PENDING ITS OWN FRESH INDEPENDENT REVIEW**.

R8-SPEC and R8-IMPLEMENT are `COMPLETE`; the R8 family is terminally complete. R8-1
`ec2c5da7d`, R8-2 `08e0d0e2a` + `5669e1f6e` + `911dd49b`, R8-3 `963a8202f`, R8-5.1
`8ef705d8a`, and R8-5.2 `2616c4651` + `bc64fe962` + `cb1a276b7` + `813e1db17` + `ad6340190`
each received fresh independent built-in `default` `CLEAN` with no actionable findings. The exact
R8-4 implementation/review-fix series
`f3d19687a` + `4027e2e82` + `ae5b45408` + `ba7979b4f` + `348b34038` + `246b2fb72` +
`888553555` + `c7bf1dcff` + `3fdbf4bd5` + `87963d46d` + `4227ae920` + `b05c7843d` +
`c39126c1b` + `3cc2a8aba` + `49a1e7dc8` + `4552b89f6` + `910597cb7` + `c172e252a` +
`293bbf708` + `0692cd1bc` + `00111d66a` received fresh independent built-in `default` `CLEAN` with
no actionable findings. Five-doc R8-6 receipt `549160ebc1d93b26fdbe8203d748dfb1a64787ef`
received fresh independent built-in `default` `CLEAN` with no findings.

At implementation HEAD `ad6340190`, the exact final wall is green: Sentinel tests pass `233` with
zero failures, and workspace tests pass `2,657` with zero failures and `2` ignored. `CTX-R8-01..06`
are `PROVEN`, and all `57` R8 implementation checkboxes/tasks are complete. Recorded decisions are
`R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A`,
`R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A`,
`R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A`,
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`,
`R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`,
`R8-4-HIGH-IMPACT-CENTRAL-PROJECTION-01: A`,
`R8-4-HIGH-IMPACT-COMPATIBILITY-INPUT-01: A`, and
`R8-4-CRITICAL-ZERO-EVIDENCE-LIMIT-01: A`.

Current active phase is `none` and active packet is `none` because the master sequence ends at
R8-IMPLEMENT. **NO NEXT ELIGIBLE PHASE IS DEFINED IN THIS CONTROL PACK.** No executable Prompt 1
selector exists; do not recycle R8-IMPLEMENT or invent a successor phase. Terminal transition commit `65eac5ab2ecf354a731d127014f91da5c87e6e8d` received fresh independent
built-in `default` `CLEAN` with no findings, so the terminal R8 transition and R8-IMPLEMENT exit gate
are review-clean. This Markdown-only review receipt records that already-reviewed terminal transition;
the receipt assigns itself no commit hash or review result, claims no `CLEAN` result for itself, remains
pending its own fresh independent review, and starts no further phase work.

## R8-IMPLEMENT Evidence Receipt

- The complete R8 MAP/SPEC/PLAN/TASKS family through `806e53740` is landed and received fresh independent built-in
  `default` `CLEAN` with no findings.
- `CTX-R8-01..06` are `PROVEN`.
- R8-1 through R8-5.2 are landed and fresh-review-clean. Five-doc R8-6 receipt
  `549160ebc1d93b26fdbe8203d748dfb1a64787ef` received fresh independent built-in `default`
  `CLEAN` with no findings.
- All `57` R8 implementation checkboxes/tasks are complete; R8-IMPLEMENT and the terminal R8 family
  are complete with active phase `none` and active packet `none`.
- Terminal transition commit `65eac5ab2ecf354a731d127014f91da5c87e6e8d` received fresh independent
  built-in `default` `CLEAN` with no findings, so the terminal R8 transition and R8-IMPLEMENT exit gate
  are review-clean.
- This separate review receipt assigns itself no commit hash or review result, remains pending its own
  fresh independent review, and starts no further phase work.

## Recorded Packet-Local Decisions

These replies authorize only the exact named seam and manifest. Any additional HIGH/CRITICAL symbol
or manifest expansion requires a new structured decision.

| Owning task | Stable required reply before symbol edit | Refreshed evidence |
|---|---|---|
| R8-2 | `DECISION R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A` | `load_replay_bundle`: HIGH, 17 direct dependents, 1 affected `execute` process, 1 module. |
| R8-3 | `DECISION R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A` | `verify_live_checkpoint_compatibility`: HIGH, 17 direct dependents, 0 processes, 1 module. |
| R8-3 | `DECISION R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A` | file-disambiguated `LiveRuntime::observe` in `src/live_runtime.rs`: HIGH, 17 direct dependents, 0 processes, 1 module. |
| R8-4 | `DECISION R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A` | `uses_explicit_analyzer_state`: HIGH, 10 impacted, 2 direct, 0 currently mapped processes, 3 modules. |
| R8-4 | `DECISION R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` | Preserve public shapes/output; accept dev-only `syn` 2 (`full`, `visit`) plus exact lockfile cost and one direct literal-v0.8 predicate in `CheckpointPresentation::render_console_block`; exact owner/count and behavior parity proof remains mandatory. |
| R8-4 | `DECISION R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A` | `score_has_historical_evidence`: HIGH, 5 impacted, 2 direct, 0 processes, 3 modules. |
| R8-4 | `DECISION R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A` | `push_evidence_lines`: HIGH, 7 impacted, 3 direct, 1 affected `execute` process, 3 modules. |
| R8-4 | `DECISION R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A` | Expand only the R8-4 exact manifest from five files to six by adding `src/checkpoint_interpretation.rs`; preserve core fallible validation and add the total compatibility-only projection contract. |
| R8-4 | `DECISION R8-4-HIGH-IMPACT-CENTRAL-PROJECTION-01: A` | Authorize only the exact shared central projection seam; no validation fallback, public API expansion, or seventh manifest file. |
| R8-4 | `DECISION R8-4-HIGH-IMPACT-COMPATIBILITY-INPUT-01: A` | Authorize only the total compatibility-input seam used by public facades; validated replay/live cannot select `CompatibilityOnly`. |
| R8-4 | `DECISION R8-4-CRITICAL-ZERO-EVIDENCE-LIMIT-01: A` | Authorize the bounded zero-limit preservation fix: validated replay/live keep flattened-core parity while compatibility facades keep pre-R8 grouped-stop behavior. |

## Per-Task Rules

Apply these rules to every task below.

1. **One seam, exact manifest.** No unlisted file changes. If the manifest must expand, stop for a
   structured decision/spec amendment.
2. **GitNexus before symbols.** Run upstream impact for every named existing production symbol
   before editing it. New symbols are recorded as new/no pre-existing target. HIGH or CRITICAL
   requires a structured operator decision before editing.
3. **TDD order.** Add/run the smallest failing contract or parity witness first and record the RED;
   implement the smallest change second; run focused and adjacent GREEN proof third. Never commit
   intentional RED.
4. **Protected boundaries.** No analyzer, compactor, schema, scheduler, adjudication, CLI, sink, or
   real-session delivery/cursor behavior edits. `adjudication.rs` is read-only R8-4 call-path
   evidence; on Option B's adjudication path, the predicate may affect only preserved rendered
   `operator_summary` content, with exact
   bytes/full-request parity, never request-shaping logic, eligibility, policy, or decision
   semantics. No public-signature break, generalized version parser, cross-session history,
   delegation inference, or error fallback.
5. **Atomic boundary.** Stage only the exact manifest; run staged GitNexus, cached whitespace check,
   and inspect the full cached diff; then create one task commit. Review fixes are separate commits.
6. **Fresh review.** A fresh independent built-in `default` reviewer must return `CLEAN` for the
   landed task/fix series before the next task starts. No self-approval. External CLI review is
   forbidden for this autonomous series.

Standard pre-commit gate for every task:

```bash
git status --short
git add -- <exact manifest only>
npx gitnexus detect-changes --scope staged -r 97a0-substrate
git diff --cached --check
git diff --cached -- <exact manifest only>
```

## R8-1 — Shared Typed Interpretation And Structured Errors

**Status:** complete; commit `ec2c5da7d` received fresh independent built-in `default` `CLEAN`.

**Description:** Introduce the origin-neutral interpretation module, exact literal schema profile,
serialized validator, typed interpretation, and structured contract errors. No replay/live core is
cut over in this task.

**Dependencies:** Entry Gate only.

**Gate mapping:** `CTX-R8-01` guard (consume unchanged analyzer types); `CTX-R8-02` entry dependency;
`CTX-R8-03` direct foundation; `CTX-R8-04` direct central owner; `CTX-R8-05` guard (facts, not
strings); `CTX-R8-06` guard (no protected-path edits).

**Exact manifest (2 files):**

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` (new)
- `crates/agent-drift-sentinel/src/lib.rs`

**GitNexus impact targets before edits:**

- New `CheckpointSchemaVersion`, `CheckpointInterpretationInput`, `CheckpointInterpretation`,
  `CheckpointContractError`, `validate_serialized_checkpoint`, and `interpret_checkpoint`: no
  pre-existing graph targets; record as new.
- `crates/agent-drift-sentinel/src/lib.rs` receives crate-private module wiring only; no public
  re-export or API expansion is allowed. If implementing the
  wiring requires editing `execute`, run upstream impact for `execute` first and stop because that
  edit belongs to R8-2.

**TDD sequence:**

1. RED: add the in-module `#[cfg(test)] mod tests` matrix that fails because the central schema/
   interpretation contract does not exist. Do not add an external integration test target.
2. GREEN: implement only the types, validator, interpreter, and crate-private module wiring needed
   by the witness.
3. PROOF: expand/run the exact central matrix and focused lint/format commands.

**Acceptance criteria:**

- [x] The supported schema set is exactly v0.2, v0.3, v0.4, v0.5, v0.6, v0.7, v0.8; v0.3-v0.8
  explicit state and version-specific required fields fail closed.
- [x] Sentinel typed non-empty validation is exactly `session_id`, `checkpoint_id`,
  `task_frame.objective`, and `expected_next_step`; no `DelegationContext` field-level validation
  exists.
- [x] Interpretation uses same-session history only, preserves bounded v0.2 behavior, gives
  v0.3-v0.8 explicit analyzer state precedence, and projects analyzer-owned v0.8 delegation without
  raw-event/orchestration inference.
- [x] `CheckpointInterpretation` carries validated schema, checkpoint, cursor, fingerprint,
  flagged/max score inputs, posture, evidence, and typed delegation facts; no scheduler decision,
  adjudication data, or rendered console strings.
- [x] Structured errors identify the schema/checkpoint/field contract gap without panic, unwrap,
  default acceptance, or legacy fallback.
- [x] `checkpoint_interpretation` and every additive item are crate-private (`pub(crate)` only as
  needed), with no public re-export/API expansion; its matrix is in-module `#[cfg(test)]` only.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel checkpoint_interpretation::tests --lib -- --nocapture
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
git diff --check -- crates/agent-drift-sentinel/src/checkpoint_interpretation.rs crates/agent-drift-sentinel/src/lib.rs
```

**Decision triggers:** Any need to change analyzer/schema types, validate inside `DelegationContext`,
parse generalized versions, use cross-session history, or edit another Sentinel source/test file.

**Atomic commit:** suggested message `feat: centralize sentinel checkpoint interpretation`.

**Review gate:** fresh built-in `default` review of the exact landed R8-1 series must be `CLEAN`.

## R8-2 — Replay Core Migration With Locked Public Facades

**Status:** complete; series `08e0d0e2a` + `5669e1f6e` + `911dd49b` received fresh independent built-in `default` `CLEAN`.

**Description:** Delegate replay raw validation to the central validator, add the internal fallible
report core, interpret the complete selected set before constructing scheduler/report state, and
route `execute` through the fallible core without changing public signatures or replay sorting/
cursor behavior.

**Dependencies:** R8-1 landed and fresh-review-clean; exact operator reply
`DECISION R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A`. A `B` reply requires respec/defer; no reply means
`load_replay_bundle` remains uneditable.

**Gate mapping:** `CTX-R8-01` guard; `CTX-R8-02` entry dependency; `CTX-R8-03` direct replay half;
`CTX-R8-04` direct replay delegation; `CTX-R8-05` direct typed-render input; `CTX-R8-06` direct
zero-effect and unchanged scheduling/adjudication proof.

**Exact manifest (5 files):**

- `crates/agent-drift-sentinel/src/input.rs`
- `crates/agent-drift-sentinel/src/lib.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/replay_input.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`

**GitNexus impact targets before edits:**

- `validate_checkpoint_contract` in `src/input.rs`
- `require_non_null_field` in `src/input.rs`
- `validate_drift_score_state_contract` in `src/input.rs`
- `read_checkpoint_jsonl_file` in `src/input.rs`
- `load_replay_bundle` in `src/input.rs`
- `render_replay_report` in `src/operator_surface.rs`
- `execute` in `src/lib.rs`
- New `try_render_replay_report` and any error-adapter helper: record as new/no pre-existing target.

Run impact again for any additional existing function before editing it. Refreshed upstream impact
for `load_replay_bundle` is HIGH (17 direct dependents, one affected `execute` process, one module),
so the stable decision above is mandatory regardless of later routine refreshes. Any additional
HIGH/CRITICAL result requires its own named operator decision.

**TDD sequence:**

1. RED: add replay failure-chain, zero-report/effect, complete-set-before-scheduler, and exact public
   signature witnesses.
2. GREEN: centralize raw validation/error mapping, add the smallest fallible report core, and switch
   `execute` to it.
3. PROOF: run replay/operator focused tests plus source-order inspection.

**Acceptance criteria:**

- [x] Serialized replay gaps retain artifact path/line; typed interpretation gaps retain
  checkpoint/schema/field detail; both flow through `InputError` and existing
  `SentinelError::Input`.
- [x] All selected checkpoints are validated/interpreted before `ReplayScheduler` or report vectors
  are constructed; failure returns no report, decision, presentation, adjudication request, or next
  cursor.
- [x] After complete-set interpretation, replay scheduling consumes only typed interpretation inputs
  and rendering calls internal `present_interpretation`, not a public compatibility facade.
- [x] Sorting, mixed-version rejection, cursor filtering, report grouping, and successful scheduler
  outputs remain behavior-compatible.
- [x] Exact compile-time function-pointer assertions lock
  `execute: fn(&SentinelRequest) -> Result<SentinelResult, SentinelError>` and
  `render_replay_report`'s current signature.
- [x] `execute` calls `try_render_replay_report`, not the infallible `render_replay_report` facade;
  no bridge uses panic, unwrap, swallowed error, or v0.2 fallback.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel --test replay_input -- --nocapture
cargo test -p agent-drift-sentinel --test operator_surface -- --nocapture
rg -n 'validate_serialized_checkpoint|interpret_checkpoint|try_render_replay_report|ReplayScheduler::new|scheduler\.observe|present_interpretation' crates/agent-drift-sentinel/src/{input.rs,operator_surface.rs,lib.rs}
! rg -n 'operator_surface::render_replay_report' crates/agent-drift-sentinel/src/lib.rs
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

Manual inspection must establish complete-set interpretation before scheduler/report creation; line
presence alone is not proof.

**Decision triggers:** Any public signature change; any need to edit scheduler/adjudication/CLI;
loss of artifact path/line; inability to keep the facade infallible without fallback; any change to
sorting, selection, cursor semantics, or report grouping.

**Atomic commit:** suggested message `refactor: route sentinel replay through typed interpretation`.

**Review gate:** fresh built-in `default` review of the exact landed R8-2 series must be `CLEAN`.

## R8-3 — Live Input And Runtime Migration With Zero-Effect Errors

**Status:** complete; commit `963a8202f` received fresh independent built-in `default` `CLEAN`.

**Description:** Delegate live fixture/typed compatibility to the same central contract and make
`LiveRuntime::observe` interpret before accepted-checkpoint, previous-by-session, scheduler, or
cursor-visible mutation.

**Dependencies:** R8-2 landed and fresh-review-clean; exact replies
`DECISION R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A` and
`DECISION R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A`. A `B` reply stops the owning seam for respec/defer;
silence on either ID is not authorization.

**Gate mapping:** `CTX-R8-01` guard; `CTX-R8-02` entry dependency; `CTX-R8-03` direct live half;
`CTX-R8-04` direct compatibility delegation; `CTX-R8-05` direct typed presentation input;
`CTX-R8-06` direct no-state-change and unchanged trigger/scheduler proof.

**Exact manifest (5 files):**

- `crates/agent-drift-sentinel/src/live_input.rs`
- `crates/agent-drift-sentinel/src/live_runtime.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
- `crates/agent-drift-sentinel/tests/live_input.rs`
- `crates/agent-drift-sentinel/tests/live_runtime.rs`

**GitNexus impact targets before edits:**

- `validate_live_fixture_contract` in `src/live_input.rs`
- `require_non_null_fixture_field` in `src/live_input.rs`
- `schema_requires_turn_context` in `src/live_input.rs`
- `schema_requires_session_archetype` in `src/live_input.rs`
- `schema_requires_session_progress` in `src/live_input.rs`
- `validate_fixture_drift_score_state_contract` in `src/live_input.rs`
- `compatibility_gap` in `src/live_input.rs`
- `verify_live_checkpoint_compatibility` in `src/live_input.rs`
- file-disambiguated `LiveRuntime::observe` (`observe` in
  `crates/agent-drift-sentinel/src/live_runtime.rs`)
- Any compatibility/error adapter helper introduced here: new/no pre-existing target.

Refreshed upstream reports mark `verify_live_checkpoint_compatibility` and file-disambiguated
`LiveRuntime::observe` HIGH with 17 direct dependents each. The former is bound to
`R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01`; the latter is bound to
`R8-3-HIGH-IMPACT-LIVE-RUNTIME-01`. Neither edit is permitted until its separate operator reply is
`A`; do not collapse the two symbol gates.

**TDD sequence:**

1. RED: add raw/typed error-chain witnesses and before/after runtime snapshot assertions for every
   relevant state surface.
2. GREEN: delegate to central validation/interpretation and reorder only the live core necessary to
   interpret before mutation/scheduling.
3. PROOF: run compatibility, event-sequence, runtime, and unchanged fixture-adapter targets.

**Acceptance criteria:**

- [x] Raw fixture contract errors retain path/line and typed event errors retain checkpoint/source
  detail through `CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`.
- [x] `LiveCheckpointCompatibility` and `verify_live_checkpoint_compatibility` keep their public
  supported-input behavior while delegating schema/semantic ownership to the central module.
- [x] On interpretation failure, latest/previous checkpoints, per-session history, scheduler state,
  processed-event count, last trigger, compatibility/cursor view, and presentation remain unchanged.
- [x] Successful checkpoint-ready and synthetic-trigger behavior stays compatible, including the
  distinction between repeated-failure trigger and analyzer posture.
- [x] No edit occurs in `real_session_live.rs`; no scheduler, adjudication, sink, delivery, or
  persistence behavior changes.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_input -- --nocapture
cargo test -p agent-drift-sentinel --test live_runtime -- --nocapture
cargo test -p agent-drift-sentinel --test live_input_adapter -- --nocapture
rg -n 'interpret_checkpoint|latest_checkpoint_by_session\.insert|scheduler\.observe|present_interpretation|processed_events' crates/agent-drift-sentinel/src/live_runtime.rs
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

Manual inspection must establish interpretation before every named mutation/call.

**Decision triggers:** HIGH/CRITICAL impact without authorization; any need to change
`validate_live_event_sequence`, scheduler semantics, synthetic cursor behavior, public compatibility
shape, or any unlisted file; any request to infer delegation or accept malformed explicit state.

**Atomic commit:** suggested message `refactor: route sentinel live through typed interpretation`.

**Review gate:** fresh built-in `default` review of the exact landed R8-3 series must be `CLEAN`.

## R8-4 — Operator Presentation Migration And Public Signature Lock

**Status:** complete; the exact chronological implementation/review-fix series
`f3d19687a` + `4027e2e82` + `ae5b45408` + `ba7979b4f` + `348b34038` + `246b2fb72` +
`888553555` + `c7bf1dcff` + `3fdbf4bd5` + `87963d46d` + `4227ae920` + `b05c7843d` +
`c39126c1b` + `3cc2a8aba` + `49a1e7dc8` + `4552b89f6` + `910597cb7` + `c172e252a` +
`293bbf708` + `0692cd1bc` + `00111d66a` received fresh independent built-in `default` `CLEAN`
with no actionable findings at `00111d66a`.

**Description:** Remove the remaining version/analyzer/evidence decisions before final rendering,
route compatibility construction through centralized non-validating projection plus the typed
renderer, and preserve exact public signatures/current total behavior, including unsupported typed
facade input. Under recorded Option B,
the frozen legacy facade and its sole schema-presence predicate may serve exact allowed owner set
`{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` after the unchanged
public shapes erase internal presence. Its raw direct `render_console_block` call-expression count
must equal exactly `4`: two replay, one live, and one adjudication. Any fifth direct call fails even
inside an allowed owner.

**Dependencies:** R8-3 commit `963a8202f` landed and fresh-review-clean; exact replies
`DECISION R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
`DECISION R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
`DECISION R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
`DECISION R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`,
`DECISION R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`,
`DECISION R8-4-HIGH-IMPACT-CENTRAL-PROJECTION-01: A`,
`DECISION R8-4-HIGH-IMPACT-COMPATIBILITY-INPUT-01: A`, and
`DECISION R8-4-CRITICAL-ZERO-EVIDENCE-LIMIT-01: A`.

**Gate mapping:** `CTX-R8-01` guard; `CTX-R8-02` entry dependency; `CTX-R8-03` direct shared renderer;
`CTX-R8-04` direct removal of duplicate compatibility; `CTX-R8-05` direct proof;
`CTX-R8-06` guard through unchanged decision/policy inputs.

**Exact manifest (6 files):**

- `crates/agent-drift-sentinel/Cargo.toml`
- `Cargo.lock`
- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

**Read-only dependency/call-path evidence outside the six-file edit manifest:**

- `crates/agent-drift-sentinel/src/adjudication.rs` — preserve `shape_request` unchanged; edit-
  manifest tests lock exact `operator_summary` bytes, full request equality, request eligibility,
  and every non-summary field. Do not edit this file or adjudication policy/request-shaping logic.

Recorded Option B requires the dev-only
`syn = { version = "2", features = ["full", "visit"] }` addition. Live `Cargo.lock` already
contains transitive `syn` `2.0.117`, but no manifest exposes it; the expected lock delta adds `syn`
only to the Sentinel package dependency list and retains the locked package/version. Any resolver
expansion is a stop. The recorded presence reply accepts this cost. No runtime dependency or
unrelated lockfile update is allowed. Re-run `live_checkpoint_compatibility` without editing it.

**Central compatibility projection contract:** Add crate-private
`CheckpointCompatibilityProjectionInput`,
`CheckpointProjectionProfile::{Schema(CheckpointSchemaVersion), CompatibilityOnly}`, and total
`project_checkpoint_compatibility(...) -> CheckpointInterpretation` in
`checkpoint_interpretation.rs`. `interpret_checkpoint` keeps the unchanged fallible validation and
error contract; after validation it and the total adapter call one private
posture/evidence/delegation/cursor/fingerprint/flagged-score projection. Exact supported typed facade
literals select `Schema(...)` without typed validation. Every other facade literal selects explicit
`CompatibilityOnly`, preserves current legacy-compatible total behavior, and is never relabeled as
v0.2. The adapter uses the optional previous checkpoint exactly as the current total facade does,
performs no schema/field/explicit-state/same-session validation, does not call or catch
`interpret_checkpoint`, and is never called by core replay/live. Both returned interpretations feed
the same private renderer through distinct adapters: validated core uses `present_interpretation`,
and total facades use `present_compatibility_interpretation`. Their flattened-core and grouped-stop
evidence-limit policies remain distinct. No public API changes.

**GitNexus impact targets before edits:**

- `present_checkpoint` in `src/operator_surface.rs`
- `present_checkpoint_with_previous` in `src/operator_surface.rs`
- `render_replay_report` in `src/operator_surface.rs`
- `CheckpointPresentation::render_console_block` in `src/operator_surface.rs`
- `format_delegation_summary`, `format_delegation_topology`, and
  `format_child_work_visibility` in `src/operator_surface.rs`
- `classify_checkpoint_posture`, `uses_explicit_analyzer_state`,
  `classify_checkpoint_posture_from_state`, `classify_checkpoint_posture_legacy`,
  `checkpoint_had_active_class`, `score_has_historical_evidence`, `historical_reason_prefixes`,
  `collect_evidence_lines`, `collect_state_backed_evidence_lines`,
  `collect_legacy_evidence_lines`, `push_evidence_lines`, `classify_checkpoint`,
  `warning_fingerprint`, and `max_flagged_score` before delegating/removing their semantic ownership
- `CheckpointInterpretation`, `CheckpointSchemaVersion::from_literal`, `interpret_checkpoint`,
  `checkpoint_posture`, and `checkpoint_evidence` in `src/checkpoint_interpretation.rs`
- New `CheckpointCompatibilityProjectionInput`, `CheckpointProjectionProfile`,
  `project_checkpoint_compatibility`, and any new private shared-projection helper: new/no
  pre-existing target.

Refreshed upstream impact for `uses_explicit_analyzer_state` is HIGH (10 impacted symbols, two
direct dependents, zero currently mapped processes, three modules), authorized by
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`. `score_has_historical_evidence` is HIGH (five impacted,
two direct, zero processes, three modules), authorized by
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`. `push_evidence_lines` is HIGH (seven impacted, three
direct, one affected `execute` process, three modules), authorized by
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`. Before code, run impact for every listed existing central
item and any other existing symbol actually edited. Any additional HIGH/CRITICAL result needs its
own decision.

**TDD sequence:**

1. RED: add the central supported/unsupported total-projection matrix, exact function-pointer
   signature assertions, facade behavior fixtures, and the `syn` AST ownership policy. Recorded
   Option B compile-locks the unchanged public shapes and proves exactly one direct field use and
   literal-v0.8 predicate in `CheckpointPresentation::render_console_block`, with zero in every
   other item. The fail-closed production call-owner witness
   independently asserts exact owner-set equality
   `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` and exact raw direct
   `render_console_block` call-expression count `4`, partitioned as two replay, one live, and one
   adjudication. Any fifth direct call fails even inside an allowed owner. Separate behavior witnesses exercise replay output, final
   `LiveObservation.presentation.render_console_block` output corresponding to `cli::run_live`, and
   `adjudication::shape_request` `operator_summary`/full-request parity; the AST witness does not
   claim runtime data flow or behavior.
2. GREEN: move/delegate only the remaining semantic projection to the central module and render the
   typed facts.
3. PROOF: run operator, compatibility, and replay/live presentation parity targets plus negative
   source searches.

**Acceptance criteria:**

- [x] Exact public signatures of `present_checkpoint`, `present_checkpoint_with_previous`, and
  `CheckpointPresentation::render_console_block` and `render_replay_report` compile unchanged;
  R8-2's `execute` signature assertion remains green.
- [x] Public facades preserve current total output, including unsupported typed facade input, and
  delegate to `project_checkpoint_compatibility` plus `present_compatibility_interpretation`;
  validated core uses `present_interpretation`, both adapters share one private renderer with
  distinct evidence-limit policies, and neither core path calls a facade as a validation boundary.
  Recorded Option B allows the frozen legacy facade and its
  sole predicate to serve exact allowed owner set
  `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. No other production
  owner may call it, and exact raw direct `render_console_block` call-expression count must remain
  `4` — two replay, one live, and one adjudication. A fifth direct call fails even inside an allowed
  owner.
- [x] Recorded Option B proves exactly one direct field use and one predicate
  `self.checkpoint.schema_version == "v0.8"` in the named legacy public facade and zero in every
  other item. It permits no supported-version table, `DriftState` classification, legacy
  evidence-prefix recognition, or earlier delegation inference.
- [x] Valid typed v0.2-v0.8 inputs produce equal renderer-consumed facts through validated and total
  central projection. Unsupported typed facade input selects `CompatibilityOnly`, not
  `Schema(V0_2)`, while preserving the existing legacy-compatible total output. Core
  `interpret_checkpoint` still rejects unsupported schema, required-field/explicit-state gaps, and
  cross-session history; the total helper does not call/catch it.
- [x] Presentation only formats/truncates/orders typed facts, labels trigger separately from
  posture, and applies existing decision/warning policy.
- [x] Core delegation projection and presentation construction follow
  `CheckpointInterpretation.delegation: Option<DelegationContext>` exactly. Recorded Option B leaves
  the public shape/output unchanged and uses its sole named predicate only at final legacy rendering.
  Unchanged `CheckpointPresentation`, `ReplayReport`, and `LiveObservation` erase internal optional
  presence before that final call; no claim says otherwise. Contract validation, typed
  interpretation, posture/evidence/delegation projection, scheduling, construction,
  `present_interpretation`, all other formatting helpers, and adjudication policy/request-shaping or
  decision logic contain no R8-4 presence predicate. On the adjudication path, the predicate may affect only preserved
  rendered operator-summary content; `adjudication.rs` remains unedited.
- [x] v0.2-v0.7 absence/default and v0.8 presence cases pass in `operator_surface` and
  `live_checkpoint_compatibility`. Recorded Option B proves legacy-facade v0.2-v0.7/v0.8
  absence/presence output, and edit-manifest tests separately
  prove `ReplayReport::to_console_text`, final `LiveObservation.presentation.render_console_block`
  output corresponding to `cli::run_live`, and `adjudication::shape_request` traverse the legacy
  facade. Exact pre/post assertions lock unchanged `operator_summary` bytes and full
  `AdjudicationRequest` equality, including request eligibility and every non-summary field.
  Behavior/parity tests, not the AST test, prove output and call-path behavior.
- [x] Analyzer-owned mixed/ambiguous topology and partial/opaque visibility are rendered as typed
  facts without revalidation.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel checkpoint_interpretation::tests --lib -- --nocapture
cargo test -p agent-drift-sentinel --test operator_surface -- --nocapture
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel --test adjudication -- --nocapture
! rg -n 'DriftState|historical_reason_prefixes|uses_explicit_analyzer_state|classify_checkpoint_posture' crates/agent-drift-sentinel/src/operator_surface.rs
rg -n 'present_interpretation|format_delegation_summary' crates/agent-drift-sentinel/src/operator_surface.rs
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

The `operator_surface` target must parse `src/operator_surface.rs` plus the Sentinel production
source set (including read-only `cli.rs` and `adjudication.rs`) with `syn::parse_file` and apply a
fail-closed `syn::visit::Visit` ownership/data-dependency/call-owner pass; delimiter/token counters
and the removed line-oriented regex are not acceptance evidence. Visit every direct `schema_version`
field/path occurrence, propagate local aliases/assignments, inspect all enclosing predicate/control
forms (`==`, `!=`, boolean nesting, `if`/`while`, `match` scrutinees/guards, parsed `matches!`,
method calls, struct field/pattern uses, constants, and aliases), and attribute each use/predicate to its owning Rust item. Any
unparsed macro, unclassified occurrence, or unowned form fails. Recorded Option B expects exactly
one direct field use and one predicate in the whole file, both
in `CheckpointPresentation::render_console_block`, every other item zero, and direct equality to
literal `"v0.8"`. The production facade-call owner set must equal exactly
`{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. Independently, raw
direct `render_console_block` call-expression count must equal exactly `4`, partitioned as two in
`ReplayReport::to_console_text`, one in `cli::run_live`, and one in
`adjudication::shape_request`; any fifth direct call fails even within an allowed owner. AST proof covers syntax/ownership/call-owner/count policy
only; separate behavior tests prove the same output, the three actual paths, and unchanged
`operator_summary` bytes/full adjudication requests.

**Decision triggers:** Any public signature/source-compatibility break not explicitly authorized by
the recorded presence decision; any need for a facade to
validate, panic, unwrap, swallow an error, or fall back; inability to preserve unsupported typed
facade behavior without relabeling it v0.2; any analyzer/delegation inference remaining in presentation;
any scheduler/adjudication/sink edit; any attempt to improvise the delegation-presence carrier or
change upstream analyzer `Checkpoint` outside recorded Option B.

**Atomic commit:** suggested message `refactor: render sentinel checkpoints from typed facts`.

**Review gate:** fresh built-in `default` review of the exact landed R8-4 series must be `CLEAN`.

## R8-5.1 — Exact v0.2-v0.8 Replay/Live Parity Matrix

**Status:** complete; commit `8ef705d8a` received fresh independent built-in `default` `CLEAN`.

**Description:** Add the test-only final compatibility/parity matrix. This task proves the migrated
cores and facades; it does not authorize production corrections outside an owning earlier packet.

**Dependencies:** R8-4 landed and fresh-review-clean.

**Gate mapping:** `CTX-R8-01` direct analyzer-fact witness; `CTX-R8-02` entry dependency;
`CTX-R8-03` direct parity proof; `CTX-R8-04` direct exact-version proof; `CTX-R8-05` direct rendering
proof; `CTX-R8-06` direct unchanged trigger/output proof.

**Exact manifest (4 files; test-only edits):**

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` (`#[cfg(test)]` only)
- `crates/agent-drift-sentinel/tests/replay_input.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

**GitNexus impact targets before edits:** Test-only task; no production symbol edits. If a RED
requires production correction, stop and reopen the smallest owning reviewed packet with fresh
impact analysis and a new exact manifest.

**TDD/proof sequence:**

1. Add the smallest missing parity case and run it against the review-clean implementation.
2. If RED, classify it as contract mismatch and return to the owning packet; do not patch production
   under R8-5.1.
3. Expand only after GREEN to the full named matrix, then run all four targets.

**Acceptance criteria:**

- [x] The matrix names v0.2 and each of v0.3, v0.4, v0.5, v0.6, v0.7, and v0.8 explicitly; only
  those literal versions are accepted.
- [x] Matching replay/live inputs agree on schema profile, cursor, fingerprint, flagged/max-score
  inputs, diagnostics, headline, context/archetype/progress, posture, evidence, delegation,
  session-local history, and trigger/posture separation.
- [x] v0.2 legacy behavior stays bounded; v0.3-v0.8 explicit state wins and malformed explicit
  fields fail closed without v0.2 inference.
- [x] The exact four typed non-empty fields and v0.8 whole-delegation rule are equivalent across raw
  replay, raw live fixture, typed replay, and typed live paths.
- [x] Parent orchestration never becomes child implementation/progress/drift evidence.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel checkpoint_interpretation::tests --lib -- --nocapture
cargo test -p agent-drift-sentinel --test replay_input -- --nocapture
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_end_to_end -- --nocapture
cargo fmt --all -- --check
```

**Decision triggers:** Any production RED; matrix ambiguity between v0.6/v0.7; any requested
generalized parsing, field-level delegation validation, or origin-specific semantic switch.

**Atomic commit:** suggested message `test: prove sentinel replay and live interpretation parity`.

**Review gate:** fresh built-in `default` review of the exact landed R8-5.1 series must be `CLEAN`.

## R8-5.2 — Zero-Effect Failure And Protected Delivery/Cursor Proof

**Status:** complete; series `2616c4651` + `bc64fe962` + `cb1a276b7` + `813e1db17` +
`ad6340190` received fresh independent built-in `default` `CLEAN`.

**Description:** Prove the narrow protected boundary after interpretation failure and that existing
append-only delivery and per-session cursor behavior remains unchanged, without production edits.
Pre-observe monitor-closure tracking, pending-poll state, and emission-ordinal allocation are
preserved/out of scope and may occur.

**Dependencies:** R8-5.1 landed and fresh-review-clean.

**Gate mapping:** `CTX-R8-01` guard; `CTX-R8-02` entry dependency; `CTX-R8-03` direct shared-failure
proof; `CTX-R8-04` direct error-equivalence proof; `CTX-R8-05` guard (no presentation on failure);
`CTX-R8-06` direct protected-boundary proof.

**Exact manifest (4 test files):**

- `crates/agent-drift-sentinel/tests/live_input.rs`
- `crates/agent-drift-sentinel/tests/live_input_adapter.rs`
- `crates/agent-drift-sentinel/tests/live_runtime.rs`
- `crates/agent-drift-sentinel/tests/real_session_live.rs`

**GitNexus impact targets before edits:** Test-only task; no production symbol edits. A need to edit
`real_session_live.rs`, `scheduler.rs`, `adjudication.rs`, `operator_sink.rs`, or any other
production file is a mandatory stop-and-respec condition.

**TDD/proof sequence:**

1. Add a failing malformed-checkpoint runtime snapshot witness plus a static real-session ordering
   witness that permits existing monitor-closure/pending-poll/emission-ordinal bookkeeping, then
   locates successful `runtime.observe` before `record_delivery` and persistence.
2. If the review-clean runtime is RED, return to R8-3; if production source ordering has drifted,
   stop for a reviewed contract decision rather than changing protected production code here.
3. Once GREEN, add adjacent append-only/restart/verified-closure/cursor invariance assertions and run
   all four targets. The runtime negative witness plus the real-session source-order witness jointly
   prove that an interpretation error cannot reach delivery or persistence.

**Acceptance criteria:**

- [x] Fixture path/line and event/source error detail survive the central error mapping.
- [x] The runtime negative witness changes no accepted-checkpoint/runtime snapshot field, scheduler
  decision/state, processed-event count, or presentation. Real-session source ordering permits
  monitor-closure tracking, pending-poll bookkeeping, and emission-ordinal allocation before the
  error, but proves no adjudication/operator-sink input, `record_delivery`, persisted cursor/
  delivery, or checkpoint acceptance occurs after interpretation fails.
- [x] Append-only fixture ordering, cursor mismatch/regression errors, sparse startup, restart,
  verified closure, per-session freshness, and delivery order remain unchanged.
- [x] Successful delivery still occurs only after successful `LiveRuntime::observe`; no
  `real_session_live.rs` production edit is required.
- [x] No test asserts rollback of pre-observe transport bookkeeping; no rollback production edit is
  authorized.
- [x] No test treats parent orchestration as proof of child implementation/progress/completion.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel --test live_input -- --nocapture
cargo test -p agent-drift-sentinel --test live_input_adapter -- --nocapture
cargo test -p agent-drift-sentinel --test live_runtime -- --nocapture
cargo test -p agent-drift-sentinel --test real_session_live -- --nocapture
cargo fmt --all -- --check
```

**Decision triggers:** Any production edit; any observed delivery/cursor behavior change; any need to
change scheduler, adjudication, sink, real-session persistence, closure validation, or sparse-startup
logic.

**Atomic commit:** suggested message `test: prove sentinel interpretation failure boundary`.

**Review gate:** fresh built-in `default` review of the exact landed R8-5.2 series must be `CLEAN`.

## R8-6 — Final Family Wall And Canonical Status Receipt

**Status:** complete; the exact five-doc canonical receipt
`549160ebc1d93b26fdbe8203d748dfb1a64787ef` received fresh independent built-in `default` `CLEAN`
with no findings. Terminal transition commit `65eac5ab2ecf354a731d127014f91da5c87e6e8d` also received
fresh independent built-in `default` `CLEAN` with no findings. The separate review receipt remains
pending its own fresh independent review and claims no result for itself.

**Entry/index receipt:** the worktree entered clean at
`ad63401902e1a5eb2e86b72ab6eb2cdd78048d8e`; GitNexus reported indexed commit/current commit
`ad63401` and `up-to-date`; every required packet head is an ancestor; and this packet changes only
the five exact docs below.

**Description:** Run the complete proof wall, record exact counts/receipts in the canonical R8
family and control ledger, and keep phase transition/mirror reconciliation separate.

**Dependencies:** All implementation/proof packets landed and fresh-review-clean.

**Gate mapping:** `CTX-R8-01` final unchanged-analyzer evidence; `CTX-R8-02` final family review
evidence satisfied by receipt `549160ebc`; `CTX-R8-03` final shared-seam wall; `CTX-R8-04` final
matrix wall; `CTX-R8-05` final ownership/static wall; `CTX-R8-06` final protected-boundary wall.

**Exact manifest (5 docs only):**

- `docs/specs/r8/MAP.md`
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-spec.md`
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-plan.md`
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-tasks.md`
- `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`

**GitNexus impact targets before edits:** Docs-only; no production symbol edits.

**Acceptance criteria:**

- [x] Focused targets, full Sentinel, workspace fmt/clippy/tests, staged GitNexus, and cached-diff
  inspection are green with exact observed counts recorded (never copied from an older receipt).
- [x] Static inspection proves core replay/live ordering and absence of facade-as-validation calls;
  recorded Option B has only its exact direct-field/
  one-predicate `CheckpointPresentation::render_console_block` legacy exception, exact allowed owner
  set `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`, and exact raw
  direct call-expression count `4` — two replay, one live, and one adjudication — with any fifth call
  failing even inside an allowed owner. Validation, interpretation, projection, scheduling, construction,
  and adjudication policy/request-shaping or decision logic own no such predicate. Option B behavior
  proof locks unchanged `operator_summary` bytes/full requests.
- [x] Receipts name every task/fix commit and fresh review result, plus every structured decision.
- [x] The receipt does not claim its own review result, phase transition, or family completion before
  fresh independent review.
- [x] Wider root/control-pack mirror reconciliation is explicitly deferred to a separate docs-only
  transition packet.

**Exact verification wall:**

```bash
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
cargo test -p agent-drift-sentinel -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
git diff --check
rg -n 'interpret_checkpoint|try_render_replay_report|scheduler\.observe|present_interpretation' crates/agent-drift-sentinel/src/{operator_surface.rs,live_runtime.rs}
! rg -n 'operator_surface::render_replay_report|present_checkpoint_with_previous' crates/agent-drift-sentinel/src/{lib.rs,live_runtime.rs}
rg -n 'unwrap\(|expect\(|panic!|v0\.2' crates/agent-drift-sentinel/src/{checkpoint_interpretation.rs,input.rs,live_input.rs,live_runtime.rs,operator_surface.rs}
```

The final `rg` output is manually classified; it does not authorize unrelated cleanup.

**Observed R8-6 results at implementation HEAD `ad6340190`:**

| Exact command | Result | Observed duration / count |
|---|---|---|
| `cargo fmt --all -- --check` | PASS | `5.821s`; no output |
| `cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings` | PASS | `1.141s`; Cargo finished in `0.97s` |
| `cargo test -p agent-drift-sentinel -- --nocapture` | PASS | `5.706s`; `233` passed, `0` failed, `0` ignored across `16` result blocks |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS | `5.176s`; Cargo finished in `4.91s` |
| `cargo test --workspace -- --nocapture` | PASS | `754.165s`; `2,657` passed, `0` failed, `2` ignored across `224` result blocks |
| `git diff --check` | PASS | `0.132s`; no output |
| ordering/ownership `rg` | PASS | `0.134s`; `13` matching lines, manually classified below |
| negated core-facade `rg` | PASS | `0.110s`; zero matches |
| panic/fallback/v0.2 `rg` | PASS after manual classification | `0.289s`; `33` matching lines |

Manual classification:

- Validated replay completes every fallible `interpret_checkpoint` call before its scheduler loop;
  then `scheduler.observe` precedes `present_interpretation`. Live delegates through
  `interpret_live_checkpoint` before accepted-state mutation; then `scheduler.observe` precedes
  `present_interpretation`. The separately matched compatibility replay facade is total by contract
  and is not the validated core.
- The negated query's zero matches confirm that `execute` and `LiveRuntime::observe` do not call
  compatibility facades as validation boundaries.
- Of the `33` final-query lines, the only three production matches are the exact supported-schema
  description, `CheckpointSchemaVersion::from_literal`'s v0.2 arm, and the total compatibility
  profile's v0.2 arm. All `16` `expect(` matches, the sole `panic!`, and all remaining v0.2
  fixtures/assertions are inside the `#[cfg(test)]` module; there is no exact `unwrap(` match and no
  match in `input.rs`, `live_input.rs`, `live_runtime.rs`, or `operator_surface.rs`.
- `CheckpointSchemaVersion::from_literal` stayed read-only: its current `698` bytes are identical
  to `ec2c5da7d` with SHA-256
  `4886d6b36e75dcc2df71d5616618e175ee7a6886b1812f85370191435200cbe5`.
- Zero-limit parity is surface-specific and locked: validated replay/live share flattened-core
  behavior for v0.2-v0.8; compatibility facades preserve pre-R8 grouped-stop behavior for both the
  empty-first-group and nonempty-first-group zero-limit cases.

**Decision triggers:** Any wall failure outside R8; any missing receipt/review evidence; any request
to update more than the five canonical docs in this packet; any attempt to mark `CTX-R8-02` or the
full family proven before the receipt itself is fresh-review-clean.

**Atomic commit:** `docs: record R8 implementation proof wall`.

**Review gate:** satisfied — fresh independent built-in `default` review of exact receipt
`549160ebc1d93b26fdbe8203d748dfb1a64787ef` returned `CLEAN` with no findings. This authorizes the
separate docs-only terminal mirror/phase-transition commit. That transition landed at
`65eac5ab2ecf354a731d127014f91da5c87e6e8d` and received its own fresh independent built-in
`default` `CLEAN` review with no findings; this current review receipt remains pending its own fresh
independent review.

## Final Wall

- [x] R8-1 fresh-review-clean.
- [x] `R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A` recorded; R8-2 fresh-review-clean.
- [x] `R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A` and
  `R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A` recorded; R8-3 fresh-review-clean.
- [x] `R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
  `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
  `R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
  `R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`,
  `R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`,
  `R8-4-HIGH-IMPACT-CENTRAL-PROJECTION-01: A`,
  `R8-4-HIGH-IMPACT-COMPATIBILITY-INPUT-01: A`, and
  `R8-4-CRITICAL-ZERO-EVIDENCE-LIMIT-01: A` recorded before their exact R8-4 edits.
- [x] R8-4 implementation/fix series fresh-review-clean, including Option B owner/count proof,
  compatibility-profile parity, and final behavior/output/adjudication-request parity.
- [x] R8-5.1 fresh-review-clean.
- [x] R8-5.2 fresh-review-clean.
- [x] No production or test file outside exact packet manifests changed.
- [x] No analyzer, compactor, schema, scheduler, adjudication, CLI, sink, or real-session delivery/
  cursor behavior edit landed.
- [x] `adjudication.rs` remained read-only R8-4 dependency/call-path evidence; Option B changed no
  adjudication policy/request-shaping logic or decision semantics and preserved exact
  `operator_summary` bytes/full request behavior.
- [x] Exact v0.2-v0.8 matrix and replay/live parity pass.
- [x] Public facade signatures and current total behavior, including unsupported typed facade input,
  remain locked.
- [x] After interpretation failure there is no scheduler decision, presentation, adjudication,
  operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance;
  existing pre-observe monitor-closure/pending-poll/emission-ordinal bookkeeping is permitted.
- [x] Full Sentinel and workspace wall pass with current counts.
- [x] R8-6 canonical receipt `549160ebc` fresh independent built-in `default` `CLEAN`.
- [x] `CTX-R8-01..06` reconciled as `PROVEN` from review-clean packet/receipt proof.
- [x] Fresh independent review of terminal mirror/phase-transition commit
  `65eac5ab2ecf354a731d127014f91da5c87e6e8d`; the fresh built-in `default` reviewer returned `CLEAN`
  with no findings, so the terminal R8 transition and R8-IMPLEMENT exit gate are review-clean.
- [ ] Fresh independent review of this Markdown-only review receipt; until then the receipt records no
  hash or review result for itself and starts no further phase work.

All `57` implementation checkboxes/tasks and the R8-6 receipt-review gate are complete. R8-IMPLEMENT
and the terminal R8 family are complete with active phase `none` and active packet `none`. **NO NEXT
ELIGIBLE PHASE IS DEFINED IN THIS CONTROL PACK.** The remaining unchecked item is review of this
Markdown-only review receipt, not implementation, the terminal transition, or a successor-phase task.
