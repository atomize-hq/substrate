# Tasks: R8 Sentinel Interpretation Consolidation

Canonical path:
`docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-tasks.md`

Status: **BOUNDED COUNT-LOCK MARKDOWN FIX FOR `24e649de6` `CHANGES_REQUIRED` / CURRENT FIX CLAIMS
NO REVIEW RESULT / ALL TASKS UNSTARTED / `CTX-R8-02` OPEN / REVIEW PENDING /
R8-IMPLEMENT BLOCKED**.

R8-SPEC is the sole active phase and is IN PROGRESS with packet `none`. The R8 MAP/SPEC contract
series `698c766f9` + `f5865fb7` + `95529809` received fresh independent built-in `default` `CLEAN`
with no findings. `CTX-R8-01` is `PROVEN` by the stable R7 analyzer/delegation contract plus that
clean R8 MAP/SPEC freeze. Fresh independent built-in `default` review of the complete family at
`0ed3d8f04` + `cfcf65507` returned `CHANGES_REQUIRED` with five scoped documentation findings.
Bounded docs-only fix `2b9565fb9` landed. Follow-up fix `b04207fb6` then received fresh
independent built-in `default` `CHANGES_REQUIRED` with one scoped conditional-acceptance finding.
Bounded conditional-acceptance fix `b9ce44c6f` then received fresh independent built-in `default`
`CHANGES_REQUIRED` with two scoped documentation findings. Bounded two-finding docs-only fix `904c93d0d` then received fresh independent built-in `default`
`CHANGES_REQUIRED` with one scoped Option B call-path finding. Bounded one-finding Markdown-only fix
series `67c81c6ff` + `24e649de6` then received fresh independent built-in `default`
`CHANGES_REQUIRED` with one scoped Option B raw-direct-call count-lock finding. This current bounded
docs-only fix addresses only that latest finding and claims no review result. All R8 implementation tasks remain unchecked and unstarted. `CTX-R8-02` is `OPEN` / `REVIEW PENDING` and
not proven; `CTX-R8-03` through `CTX-R8-06` remain `BLOCKED`. R8-4 and `CTX-R8-05` remain
decision-blocked by their future structured gates. R8-IMPLEMENT remains blocked/boundary-only, and
no R8 code has started. No phase transition, Prompt 1 eligibility,
implementation authorization, complete-family `CLEAN`, or review result for this progress receipt
is claimed. No checkbox below authorizes
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

## Future Symbol-Edit Decision Gates — Not Requested During R8-SPEC

The refreshed 2026-07-16 upstream graph records the first four gates below as HIGH. The fifth gate
records an unresolved public delegation-presence contract. They are future R8-IMPLEMENT
dependencies, not blockers to completing this docs-only fix. Do not ask the operator now because no
production symbol is being edited. During implementation, exact A/B options, recommendations,
tests, and prompts come from the PLAN decision table; silence is never authorization.

| Owning task | Stable required reply before symbol edit | Refreshed evidence |
|---|---|---|
| R8-2 | `DECISION R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A` | `load_replay_bundle`: HIGH, 17 direct dependents, 1 affected `execute` process, 1 module. |
| R8-3 | `DECISION R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A` | `verify_live_checkpoint_compatibility`: HIGH, 17 direct dependents, 0 processes, 1 module. |
| R8-3 | `DECISION R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A` | file-disambiguated `LiveRuntime::observe` in `src/live_runtime.rs`: HIGH, 17 direct dependents, 0 processes, 1 module. |
| R8-4 | `DECISION R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A` | `uses_explicit_analyzer_state`: HIGH, 10 impacted, 2 direct, 1 affected `execute` process, 3 modules. |
| R8-4 | `DECISION R8-4-PRESENTATION-DELEGATION-PRESENCE-01: A` **or** `: B` | `Checkpoint.delegation` is non-optional after pre-v0.8 absence deserializes to `DelegationContext::default()`; public `CheckpointPresentation` has no optional presence carrier, and unchanged `ReplayReport`/`LiveObservation` erase that internal presence before final rendering. Option A accepts the public-field/source-compatibility cost, requires zero presentation `schema_version` uses/predicates, and preserves the same public output. Option B accepts one direct literal-v0.8 predicate in `CheckpointPresentation::render_console_block` plus localized compatibility/presentation coupling/parity cost and permits that facade to serve exact allowed owner set `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. Fail-closed AST/call-path proof independently asserts that set equality and exact raw direct `render_console_block` call-expression count `4` — two replay, one live, and one adjudication — and fails on any fifth call even inside an allowed owner. Validation/interpretation/projection/scheduling/construction and adjudication policy/request-shaping or decision logic contain zero R8-4 schema-presence predicates; on the adjudication path, the predicate may affect only preserved rendered operator-summary content. Both accept dev-only `syn` 2 (`full`, `visit`) plus exact lockfile cost for AST/call-path proof. `adjudication.rs` is read-only evidence, not an edit-manifest file. PLAN recommends B without resolving it. Upstream `Checkpoint` changes are out of scope. |

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

**Status:** unstarted; blocked by the Entry Gate.

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
- [ ] `checkpoint_interpretation` and every additive item are crate-private (`pub(crate)` only as
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

**Status:** unstarted; blocked by R8-1 review-clean and
`R8-2-HIGH-IMPACT-REPLAY-LOADER-01` operator reply `A`.

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

**Status:** unstarted; blocked by R8-2 review-clean and separate operator replies `A` for
`R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01` and `R8-3-HIGH-IMPACT-LIVE-RUNTIME-01`.

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

**Status:** unstarted; blocked by R8-3 review-clean and
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01` operator reply `A`, plus an explicit A/B choice for
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01`.

**Description:** Remove the remaining version/analyzer/evidence decisions before final rendering,
route compatibility construction through centralized non-validating projection plus the typed
renderer, and preserve exact public signatures/current supported behavior. Under Option B only,
the frozen legacy facade and its sole schema-presence predicate may serve exact allowed owner set
`{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` after the unchanged
public shapes erase internal presence. Its raw direct `render_console_block` call-expression count
must equal exactly `4`: two replay, one live, and one adjudication. Any fifth direct call fails even
inside an allowed owner.

**Dependencies:** R8-3 landed and fresh-review-clean; exact operator reply
`DECISION R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`; and exact operator reply
`DECISION R8-4-PRESENTATION-DELEGATION-PRESENCE-01: A|B`. A `B` reply to the HIGH-impact gate
requires respec/defer; the presence reply selects the PLAN's A or B implementation/test branch.
No reply to either gate means every R8-4 symbol remains uneditable.

**Gate mapping:** `CTX-R8-01` guard; `CTX-R8-02` entry dependency; `CTX-R8-03` direct shared renderer;
`CTX-R8-04` direct removal of duplicate compatibility; `CTX-R8-05` direct proof;
`CTX-R8-06` guard through unchanged decision/policy inputs.

**Exact manifest (5 files):**

- `crates/agent-drift-sentinel/Cargo.toml`
- `Cargo.lock`
- `crates/agent-drift-sentinel/src/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/operator_surface.rs`
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`

**Read-only dependency/call-path evidence outside the five-file edit manifest:**

- `crates/agent-drift-sentinel/src/adjudication.rs` — preserve `shape_request` unchanged; edit-
  manifest tests lock exact `operator_summary` bytes, full request equality, request eligibility,
  and every non-summary field. Do not edit this file or adjudication policy/request-shaping logic.

Both branches require the same future dev-only
`syn = { version = "2", features = ["full", "visit"] }` addition. Live `Cargo.lock` already
contains transitive `syn` `2.0.117`, but no manifest exposes it; the expected lock delta adds `syn`
only to the Sentinel package dependency list and retains the locked package/version. Any resolver
expansion is a stop. The presence reply must accept this cost for A or B; it is not authorized now.
No runtime dependency or unrelated lockfile update is allowed. Re-run `live_checkpoint_compatibility` without editing it.

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
- New `present_interpretation` or compatibility-projection helpers: new/no pre-existing target.

Refreshed upstream impact for `uses_explicit_analyzer_state` is HIGH (10 impacted symbols, two
direct dependents, one affected `execute` process, three modules). It is bound to
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01`; no edit/removal is permitted before the operator reply is `A`.

**TDD sequence:**

1. RED: add exact function-pointer signature assertions, facade behavior fixtures, and the `syn`
   AST ownership policy. Apply only the selected presence branch: Option A directly constructs the
   new optional public field and proves `None`/`Some` plus zero `schema_version` uses/predicates;
   Option B compile-locks the unchanged public shapes and proves exactly one direct field use and
   literal-v0.8 predicate in `CheckpointPresentation::render_console_block`, with zero in every
   other item. Both lock the existing signature. The fail-closed production call-owner witness
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

- [ ] Exact public signatures of `present_checkpoint`, `present_checkpoint_with_previous`, and
  `CheckpointPresentation::render_console_block` and `render_replay_report` compile unchanged;
  R8-2's `execute` signature assertion remains green.
- [ ] Public facades preserve current supported-input output and delegate to centralized
  non-validating projection plus `present_interpretation`; neither core path calls a facade as a
  validation boundary. Option B explicitly allows the frozen legacy facade and its sole predicate
  to serve exact allowed owner set
  `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. No other production
  owner may call it, and exact raw direct `render_console_block` call-expression count must remain
  `4` — two replay, one live, and one adjudication. A fifth direct call fails even inside an allowed
  owner.
- [ ] The recorded presence decision selects exactly one mutually exclusive schema acceptance
  branch: Option A proves zero `schema_version` field/path uses and predicates in all of
  `operator_surface.rs`; Option B proves exactly one direct field use and one predicate
  `self.checkpoint.schema_version == "v0.8"` in the named legacy public facade and zero in every
  other item. Neither branch permits a supported-version table, `DriftState` classification, legacy
  evidence-prefix recognition, or earlier delegation inference.
- [ ] Presentation only formats/truncates/orders typed facts, labels trigger separately from
  posture, and applies existing decision/warning policy.
- [ ] Core delegation projection and presentation construction follow
  `CheckpointInterpretation.delegation: Option<DelegationContext>` exactly. The selected public
  representation is proven: Option A's explicit optional field has the authorized public-field
  cost and exact `None`/`Some` construction/output tests with no predicate and the same public
  output; or Option B leaves the public shape/output unchanged and uses its sole named predicate
  only at final legacy rendering. With Option B,
  unchanged `CheckpointPresentation`, `ReplayReport`, and `LiveObservation` erase internal optional
  presence before that final call; no claim says otherwise. Contract validation, typed
  interpretation, posture/evidence/delegation projection, scheduling, construction,
  `present_interpretation`, all other formatting helpers, and adjudication policy/request-shaping or
  decision logic contain no R8-4 presence predicate. On the adjudication path, the predicate may affect only preserved
  rendered operator-summary content; `adjudication.rs` remains unedited.
- [ ] v0.2-v0.7 absence/default and v0.8 presence cases pass in `operator_surface` and
  `live_checkpoint_compatibility`. Option A proves its public carrier survives to output. Option B
  proves legacy-facade v0.2-v0.7/v0.8 absence/presence output and edit-manifest tests separately
  prove `ReplayReport::to_console_text`, final `LiveObservation.presentation.render_console_block`
  output corresponding to `cli::run_live`, and `adjudication::shape_request` traverse the legacy
  facade. Exact pre/post assertions lock unchanged `operator_summary` bytes and full
  `AdjudicationRequest` equality, including request eligibility and every non-summary field.
  Behavior/parity tests, not the AST test, prove output and call-path behavior.
- [ ] Analyzer-owned mixed/ambiguous topology and partial/opaque visibility are rendered as typed
  facts without revalidation.

**Exact verification:**

```bash
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
unparsed macro, unclassified occurrence, or unowned form fails. Option A expects zero uses and
predicates. Option B expects exactly one direct field use and one predicate in the whole file, both
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
validate, panic, unwrap, or fall back; any analyzer/delegation inference remaining in presentation;
any scheduler/adjudication/sink edit; any attempt to improvise the delegation-presence carrier or
change upstream analyzer `Checkpoint` outside the recorded A/B reply.

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

**Status:** unstarted; blocked by R8-5.1 review-clean.

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

- [ ] Fixture path/line and event/source error detail survive the central error mapping.
- [ ] The runtime negative witness changes no accepted-checkpoint/runtime snapshot field, scheduler
  decision/state, processed-event count, or presentation. Real-session source ordering permits
  monitor-closure tracking, pending-poll bookkeeping, and emission-ordinal allocation before the
  error, but proves no adjudication/operator-sink input, `record_delivery`, persisted cursor/
  delivery, or checkpoint acceptance occurs after interpretation fails.
- [ ] Append-only fixture ordering, cursor mismatch/regression errors, sparse startup, restart,
  verified closure, per-session freshness, and delivery order remain unchanged.
- [ ] Successful delivery still occurs only after successful `LiveRuntime::observe`; no
  `real_session_live.rs` production edit is required.
- [ ] No test asserts rollback of pre-observe transport bookkeeping; no rollback production edit is
  authorized.
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

**Atomic commit:** suggested message `test: prove sentinel interpretation failure boundary`.

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
  Option A has zero presentation schema uses/predicates, or Option B has only its exact direct-field/
  one-predicate `CheckpointPresentation::render_console_block` legacy exception, exact allowed owner
  set `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`, and exact raw
  direct call-expression count `4` — two replay, one live, and one adjudication — with any fifth call
  failing even inside an allowed owner. Validation, interpretation, projection, scheduling, construction,
  and adjudication policy/request-shaping or decision logic own no such predicate under either
  branch; Option B behavior proof locks unchanged `operator_summary` bytes/full requests, and
  Option A has no predicate while preserving the same public output.
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
- [ ] `R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A` recorded; R8-2 fresh-review-clean.
- [ ] `R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01: A` and
  `R8-3-HIGH-IMPACT-LIVE-RUNTIME-01: A` recorded; R8-3 fresh-review-clean.
- [ ] `R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A` recorded; R8-4 fresh-review-clean.
- [ ] `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: A|B` recorded before R8-4 edits, with the selected
  public-compatibility cost, dev-only `syn`/lockfile cost, AST ownership policy, exact Option B owner
  set plus raw direct call count `4` lock, and separate final behavior/output/adjudication-request
  parity proof fresh-review-clean.
- [ ] R8-5.1 fresh-review-clean.
- [ ] R8-5.2 fresh-review-clean.
- [ ] No production or test file outside exact packet manifests changed.
- [ ] No analyzer, compactor, schema, scheduler, adjudication, CLI, sink, or real-session delivery/
  cursor behavior edit landed.
- [ ] `adjudication.rs` remained read-only R8-4 dependency/call-path evidence; Option B changed no
  adjudication policy/request-shaping logic or decision semantics and preserved exact
  `operator_summary` bytes/full request behavior; Option A used no presentation predicate and
  preserved the same public output.
- [ ] Exact v0.2-v0.8 matrix and replay/live parity pass.
- [ ] Public facade signatures and supported behavior remain locked.
- [ ] After interpretation failure there is no scheduler decision, presentation, adjudication,
  operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance;
  existing pre-observe monitor-closure/pending-poll/emission-ordinal bookkeeping is permitted.
- [ ] Full Sentinel and workspace wall pass with current counts.
- [ ] R8-6 canonical receipt fresh-review-clean.
- [ ] `CTX-R8-01..06` status reconciled only from reviewed evidence.
- [ ] Separate mirror/phase transition reviewed before any R8 completion or next-phase claim.

All boxes are intentionally unchecked. `CTX-R8-01` is proven only by the stable R7 contract plus the
clean R8 MAP/SPEC freeze. This candidate TASKS artifact does not claim `CTX-R8-02..06`, R8
implementation, or the full R8 family is clean or complete.
