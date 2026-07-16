# Tasks: R8 Sentinel Interpretation Consolidation

Canonical path:
`docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-tasks.md`

Status: **CANDIDATE / AWAITING FRESH INDEPENDENT BUILT-IN `default` REVIEW / ALL TASKS UNSTARTED /
`CTX-R8-02` OPEN / REVIEW PENDING / R8-IMPLEMENT BLOCKED**.

R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. PLAN/TASKS candidate commit `0ed3d8f04` is landed and awaits fresh
independent built-in `default` review; all implementation tasks remain unchecked and unstarted.
`CTX-R8-02` is `OPEN` / `REVIEW PENDING` and not proven; `CTX-R8-03` through `CTX-R8-06` remain
`BLOCKED`. R8-IMPLEMENT remains blocked/boundary-only, and no R8 code has started. No phase
transition, Prompt 1 eligibility, implementation authorization, or complete-family `CLEAN` is
claimed. This progress receipt claims no review result for itself. No checkbox below authorizes
source/test implementation until the complete four-document family is fresh-review-clean and
`CTX-R8-02` is proven.

## Entry Gate — Must Be Satisfied Before R8-1

- [ ] Fresh independent built-in `default` review covers the complete current R8 MAP/SPEC/PLAN/TASKS
  family and returns `CLEAN`.
- [ ] Any review findings are fixed in bounded docs-only commits and freshly re-reviewed.
- [ ] The canonical authority records `CTX-R8-02` as satisfied for implementation entry without
  claiming any implementation gate proven.
- [ ] `git status --short` is clean before the first source/test edit.
- [ ] No R8 implementation has started early.

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
   real-session delivery/cursor behavior edits. No public-signature break, generalized version
   parser, cross-session history, delegation inference, or error fallback.
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

**Status:** unstarted; blocked by the Entry Gate.

**Description:** Introduce the origin-neutral interpretation module, exact literal schema profile,
serialized validator, typed interpretation, and structured contract errors. No replay/live core is
cut over in this task.

**Dependencies:** Entry Gate only.

**Gate mapping:** `CTX-R8-01` guard (consume unchanged analyzer types); `CTX-R8-02` entry dependency;
`CTX-R8-03` direct foundation; `CTX-R8-04` direct central owner; `CTX-R8-05` guard (facts, not
strings); `CTX-R8-06` guard (no protected-path edits).

**Exact manifest (3 files):**

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` (new)
- `crates/agent-drift-sentinel/src/lib.rs`
- `crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs` (new)

**GitNexus impact targets before edits:**

- New `CheckpointSchemaVersion`, `CheckpointInterpretationInput`, `CheckpointInterpretation`,
  `CheckpointContractError`, `validate_serialized_checkpoint`, and `interpret_checkpoint`: no
  pre-existing graph targets; record as new.
- `crates/agent-drift-sentinel/src/lib.rs` receives module/export wiring only. If implementing the
  wiring requires editing `execute`, run upstream impact for `execute` first and stop because that
  edit belongs to R8-2.

**TDD sequence:**

1. RED: add focused tests that fail because the central schema/interpretation contract does not
   exist.
2. GREEN: implement only the types, validator, interpreter, and module/export wiring needed by the
   witness.
3. PROOF: expand/run the exact central matrix and focused lint/format commands.

**Acceptance criteria:**

- [ ] The supported schema set is exactly v0.2, v0.3, v0.4, v0.5, v0.6, v0.7, v0.8; v0.3-v0.8
  explicit state and version-specific required fields fail closed.
- [ ] Sentinel typed non-empty validation is exactly `session_id`, `checkpoint_id`,
  `task_frame.objective`, and `expected_next_step`; no `DelegationContext` field-level validation
  exists.
- [ ] Interpretation uses same-session history only, preserves bounded v0.2 behavior, gives
  v0.3-v0.8 explicit analyzer state precedence, and projects analyzer-owned v0.8 delegation without
  raw-event/orchestration inference.
- [ ] `CheckpointInterpretation` carries validated schema, checkpoint, cursor, fingerprint,
  flagged/max score inputs, posture, evidence, and typed delegation facts; no scheduler decision,
  adjudication data, or rendered console strings.
- [ ] Structured errors identify the schema/checkpoint/field contract gap without panic, unwrap,
  default acceptance, or legacy fallback.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel --test checkpoint_interpretation -- --nocapture
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
git diff --check -- crates/agent-drift-sentinel/src/checkpoint_interpretation.rs crates/agent-drift-sentinel/src/lib.rs crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs
```

**Decision triggers:** Any need to change analyzer/schema types, validate inside `DelegationContext`,
parse generalized versions, use cross-session history, or edit another Sentinel source/test file.

**Atomic commit:** suggested message `feat: centralize sentinel checkpoint interpretation`.

**Review gate:** fresh built-in `default` review of the exact landed R8-1 series must be `CLEAN`.

## R8-2 — Replay Core Migration With Locked Public Facades

**Status:** unstarted; blocked by R8-1 review-clean.

**Description:** Delegate replay raw validation to the central validator, add the internal fallible
report core, interpret the complete selected set before constructing scheduler/report state, and
route `execute` through the fallible core without changing public signatures or replay sorting/
cursor behavior.

**Dependencies:** R8-1 landed and fresh-review-clean.

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
- `read_checkpoint_jsonl_file` in `src/input.rs`
- `load_replay_bundle` in `src/input.rs`
- `render_replay_report` in `src/operator_surface.rs`
- `execute` in `src/lib.rs`
- New `try_render_replay_report` and any error-adapter helper: record as new/no pre-existing target.

Run impact again for any additional existing function before editing it. Any HIGH/CRITICAL result
requires a named operator decision.

**TDD sequence:**

1. RED: add replay failure-chain, zero-report/effect, complete-set-before-scheduler, and exact public
   signature witnesses.
2. GREEN: centralize raw validation/error mapping, add the smallest fallible report core, and switch
   `execute` to it.
3. PROOF: run replay/operator focused tests plus source-order inspection.

**Acceptance criteria:**

- [ ] Serialized replay gaps retain artifact path/line; typed interpretation gaps retain
  checkpoint/schema/field detail; both flow through `InputError` and existing
  `SentinelError::Input`.
- [ ] All selected checkpoints are validated/interpreted before `ReplayScheduler` or report vectors
  are constructed; failure returns no report, decision, presentation, adjudication request, or next
  cursor.
- [ ] After complete-set interpretation, replay scheduling consumes only typed interpretation inputs
  and rendering calls internal `present_interpretation`, not a public compatibility facade.
- [ ] Sorting, mixed-version rejection, cursor filtering, report grouping, and successful scheduler
  outputs remain behavior-compatible.
- [ ] Exact compile-time function-pointer assertions lock
  `execute: fn(&SentinelRequest) -> Result<SentinelResult, SentinelError>` and
  `render_replay_report`'s current signature.
- [ ] `execute` calls `try_render_replay_report`, not the infallible `render_replay_report` facade;
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

**Status:** unstarted; blocked by R8-2 review-clean and a required HIGH-impact decision if refreshed
GitNexus remains HIGH.

**Description:** Delegate live fixture/typed compatibility to the same central contract and make
`LiveRuntime::observe` interpret before accepted-checkpoint, previous-by-session, scheduler, or
cursor-visible mutation.

**Dependencies:** R8-2 landed and fresh-review-clean; structured HIGH-risk decision satisfied.

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
- `verify_live_checkpoint_compatibility` in `src/live_input.rs`
- `LiveRuntime::observe` in `src/live_runtime.rs`
- Any compatibility/error adapter helper introduced here: new/no pre-existing target.

Current planning-time impact reports mark `verify_live_checkpoint_compatibility` and
`LiveRuntime::observe` HIGH with 17 direct test dependents each. Refresh the graph and emit
`DECISION REQUIRED R8-3-HIGH-IMPACT-LIVE-CUTOVER-01: A|B` before either edit if that risk remains.

**TDD sequence:**

1. RED: add raw/typed error-chain witnesses and before/after runtime snapshot assertions for every
   relevant state surface.
2. GREEN: delegate to central validation/interpretation and reorder only the live core necessary to
   interpret before mutation/scheduling.
3. PROOF: run compatibility, event-sequence, runtime, and unchanged fixture-adapter targets.

**Acceptance criteria:**

- [ ] Raw fixture contract errors retain path/line and typed event errors retain checkpoint/source
  detail through `CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`.
- [ ] `LiveCheckpointCompatibility` and `verify_live_checkpoint_compatibility` keep their public
  supported-input behavior while delegating schema/semantic ownership to the central module.
- [ ] On interpretation failure, latest/previous checkpoints, per-session history, scheduler state,
  processed-event count, last trigger, compatibility/cursor view, and presentation remain unchanged.
- [ ] Successful checkpoint-ready and synthetic-trigger behavior stays compatible, including the
  distinction between repeated-failure trigger and analyzer posture.
- [ ] No edit occurs in `real_session_live.rs`; no scheduler, adjudication, sink, delivery, or
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

**Status:** unstarted; blocked by R8-3 review-clean.

**Description:** Remove the remaining version/analyzer/evidence decisions from operator
presentation, route all compatibility facades through centralized non-validating projection plus
the typed renderer, and preserve exact public signatures/current supported behavior.

**Dependencies:** R8-3 landed and fresh-review-clean.

**Gate mapping:** `CTX-R8-01` guard; `CTX-R8-02` entry dependency; `CTX-R8-03` direct shared renderer;
`CTX-R8-04` direct removal of duplicate compatibility; `CTX-R8-05` direct proof;
`CTX-R8-06` guard through unchanged decision/policy inputs.

**Exact manifest (5 files):**

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

**GitNexus impact targets before edits:**

- `present_checkpoint` in `src/operator_surface.rs`
- `present_checkpoint_with_previous` in `src/operator_surface.rs`
- `render_replay_report` in `src/operator_surface.rs`
- `classify_checkpoint_posture`, `uses_explicit_analyzer_state`,
  `classify_checkpoint_posture_from_state`, `classify_checkpoint_posture_legacy`,
  `collect_evidence_lines`, and `warning_fingerprint` before delegating/removing their semantic
  ownership
- New `present_interpretation` or compatibility-projection helpers: new/no pre-existing target.

**TDD sequence:**

1. RED: add exact function-pointer signature assertions, facade behavior fixtures, and a static/
   behavioral witness that typed rendering owns no version/state inference.
2. GREEN: move/delegate only the remaining semantic projection to the central module and render the
   typed facts.
3. PROOF: run operator, compatibility, and replay/live presentation parity targets plus negative
   source searches.

**Acceptance criteria:**

- [ ] Exact public signatures of `present_checkpoint`, `present_checkpoint_with_previous`, and
  `render_replay_report` compile unchanged; R8-2's `execute` signature assertion remains green.
- [ ] Public facades preserve current supported-input output and delegate to centralized
  non-validating projection plus `present_interpretation`; neither core path calls a facade as a
  validation boundary.
- [ ] `operator_surface.rs` contains no schema-version table/string checks, `DriftState`
  classification, legacy evidence-prefix recognition, or delegation inference.
- [ ] Presentation only formats/truncates/orders typed facts, labels trigger separately from
  posture, and applies existing decision/warning policy.
- [ ] Analyzer-owned mixed/ambiguous topology and partial/opaque visibility are rendered as typed
  facts without revalidation.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel --test operator_surface -- --nocapture
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_end_to_end -- --nocapture
! rg -n 'schema_version|DriftState|historical_reason_prefixes|uses_explicit_analyzer_state|classify_checkpoint_posture' crates/agent-drift-sentinel/src/operator_surface.rs
rg -n 'present_interpretation|format_delegation_summary' crates/agent-drift-sentinel/src/operator_surface.rs
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

**Decision triggers:** Any public signature/source-compatibility break; any need for a facade to
validate, panic, unwrap, or fall back; any analyzer/delegation inference remaining in presentation;
any scheduler/adjudication/sink edit.

**Atomic commit:** suggested message `refactor: render sentinel checkpoints from typed facts`.

**Review gate:** fresh built-in `default` review of the exact landed R8-4 series must be `CLEAN`.

## R8-5.1 — Exact v0.2-v0.8 Replay/Live Parity Matrix

**Status:** unstarted; blocked by R8-4 review-clean.

**Description:** Add the test-only final compatibility/parity matrix. This task proves the migrated
cores and facades; it does not authorize production corrections outside an owning earlier packet.

**Dependencies:** R8-4 landed and fresh-review-clean.

**Gate mapping:** `CTX-R8-01` direct analyzer-fact witness; `CTX-R8-02` entry dependency;
`CTX-R8-03` direct parity proof; `CTX-R8-04` direct exact-version proof; `CTX-R8-05` direct rendering
proof; `CTX-R8-06` direct unchanged trigger/output proof.

**Exact manifest (4 test files):**

- `crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs`
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

- [ ] The matrix names v0.2 and each of v0.3, v0.4, v0.5, v0.6, v0.7, and v0.8 explicitly; only
  those literal versions are accepted.
- [ ] Matching replay/live inputs agree on schema profile, cursor, fingerprint, flagged/max-score
  inputs, diagnostics, headline, context/archetype/progress, posture, evidence, delegation,
  session-local history, and trigger/posture separation.
- [ ] v0.2 legacy behavior stays bounded; v0.3-v0.8 explicit state wins and malformed explicit
  fields fail closed without v0.2 inference.
- [ ] The exact four typed non-empty fields and v0.8 whole-delegation rule are equivalent across raw
  replay, raw live fixture, typed replay, and typed live paths.
- [ ] Parent orchestration never becomes child implementation/progress/drift evidence.

**Exact verification:**

```bash
cargo test -p agent-drift-sentinel --test checkpoint_interpretation -- --nocapture
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

**Status:** unstarted; blocked by R8-5.1 review-clean.

**Description:** Prove contract failure has no adapter/runtime/real-session side effects and that
existing append-only delivery and per-session cursor behavior remains unchanged, without production
edits.

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
   witness that locates successful `runtime.observe` before `record_delivery` and persistence.
2. If the review-clean runtime is RED, return to R8-3; if production source ordering has drifted,
   stop for a reviewed contract decision rather than changing protected production code here.
3. Once GREEN, add adjacent append-only/restart/verified-closure/cursor invariance assertions and run
   all four targets. The runtime negative witness plus the real-session source-order witness jointly
   prove that an interpretation error cannot reach delivery or persistence.

**Acceptance criteria:**

- [ ] Fixture path/line and event/source error detail survive the central error mapping.
- [ ] The runtime negative witness changes no snapshot field, scheduler decision/state,
  processed-event count, presentation, or accepted checkpoint; real-session source ordering proves
  the error returns before any delivery record or persisted cursor, so no adjudication/sink input is
  produced.
- [ ] Append-only fixture ordering, cursor mismatch/regression errors, sparse startup, restart,
  verified closure, per-session freshness, and delivery order remain unchanged.
- [ ] Successful delivery still occurs only after successful `LiveRuntime::observe`; no
  `real_session_live.rs` production edit is required.
- [ ] No test treats parent orchestration as proof of child implementation/progress/completion.

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

**Atomic commit:** suggested message `test: prove sentinel interpretation failure is effect free`.

**Review gate:** fresh built-in `default` review of the exact landed R8-5.2 series must be `CLEAN`.

## R8-6 — Final Family Wall And Canonical Status Receipt

**Status:** unstarted; blocked by R8-1..R8-5.2 and every fix series fresh-review-clean.

**Description:** Run the complete proof wall, record exact counts/receipts in the canonical R8
family and control ledger, and keep phase transition/mirror reconciliation separate.

**Dependencies:** All implementation/proof packets landed and fresh-review-clean.

**Gate mapping:** `CTX-R8-01` final unchanged-analyzer evidence; `CTX-R8-02` final family review
evidence only after receipt review; `CTX-R8-03` final shared-seam wall; `CTX-R8-04` final matrix wall;
`CTX-R8-05` final ownership/static wall; `CTX-R8-06` final protected-boundary wall.

**Exact manifest (5 docs only):**

- `docs/specs/r8/MAP.md`
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-spec.md`
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-plan.md`
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-tasks.md`
- `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`

**GitNexus impact targets before edits:** Docs-only; no production symbol edits.

**Acceptance criteria:**

- [ ] Focused targets, full Sentinel, workspace fmt/clippy/tests, staged GitNexus, and cached-diff
  inspection are green with exact observed counts recorded (never copied from an older receipt).
- [ ] Static inspection proves core replay/live ordering and absence of facade-as-validation calls;
  operator presentation owns no schema/state/delegation inference.
- [ ] Receipts name every task/fix commit and fresh review result, plus every structured decision.
- [ ] The receipt does not claim its own review result, phase transition, or family completion before
  fresh independent review.
- [ ] Wider root/control-pack mirror reconciliation is explicitly deferred to a separate docs-only
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

**Decision triggers:** Any wall failure outside R8; any missing receipt/review evidence; any request
to update more than the five canonical docs in this packet; any attempt to mark `CTX-R8-02` or the
full family proven before the receipt itself is fresh-review-clean.

**Atomic commit:** suggested message `docs: record R8 sentinel interpretation proof`.

**Review gate:** fresh built-in `default` review of the exact R8-6 receipt must be `CLEAN`. Only then
may a separate docs-only mirror/phase transition packet be proposed.

## Final Unchecked Wall

- [ ] R8-1 fresh-review-clean.
- [ ] R8-2 fresh-review-clean.
- [ ] R8-3 HIGH-impact decision recorded if required; R8-3 fresh-review-clean.
- [ ] R8-4 fresh-review-clean.
- [ ] R8-5.1 fresh-review-clean.
- [ ] R8-5.2 fresh-review-clean.
- [ ] No production or test file outside exact packet manifests changed.
- [ ] No analyzer, compactor, schema, scheduler, adjudication, CLI, sink, or real-session delivery/
  cursor behavior edit landed.
- [ ] Exact v0.2-v0.8 matrix and replay/live parity pass.
- [ ] Public facade signatures and supported behavior remain locked.
- [ ] Failure produces zero scheduler/presentation/adjudication/sink/acceptance/delivery/cursor
  effects.
- [ ] Full Sentinel and workspace wall pass with current counts.
- [ ] R8-6 canonical receipt fresh-review-clean.
- [ ] `CTX-R8-01..06` status reconciled only from reviewed evidence.
- [ ] Separate mirror/phase transition reviewed before any R8 completion or next-phase claim.

All boxes are intentionally unchecked. `CTX-R8-01` is proven only by the stable R7 contract plus the
clean R8 MAP/SPEC freeze. This candidate TASKS artifact does not claim `CTX-R8-02..06`, R8
implementation, or the full R8 family is clean or complete.
