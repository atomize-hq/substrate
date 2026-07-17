# Implementation Plan: R8 Sentinel Interpretation Consolidation

Canonical path:
`docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-plan.md`

Status: **R8-IMPLEMENT ACTIVE / R8-1 `ec2c5da7d` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` /
R8-2 `08e0d0e2a` + `5669e1f6e` + `911dd49b` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` /
R8-3 `963a8202` FRESH INDEPENDENT BUILT-IN `default` `CLEAN` / R8-4 DOCS-ONLY AUTHORITY
AMENDMENT `849393029` LANDED / FRESH INDEPENDENT REVIEW PENDING / NO R8-4 SOURCE OR TEST EDITS**.

R8-SPEC and the R8-IMPLEMENT entry gate remain review-clean. R8-1 commit `ec2c5da7d`, R8-2 series
`08e0d0e2a` + `5669e1f6e` + `911dd49b`, and R8-3 commit `963a8202` each received fresh
independent built-in `default` `CLEAN` with no actionable findings. R8-4 is blocked only on
fresh independent review of landed bounded Markdown-only authority amendment `849393029`; no R8-4
source or test edit has started. Recorded R8-4 decisions are
`R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`, and
`R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`. Landed amendment candidate `849393029` is pending
fresh independent review, has no review result, makes no `CLEAN` claim, and starts no code.

## Objective

Make replay and live sentinel paths consume one origin-neutral, fallible typed checkpoint
interpretation while preserving the analyzer as the sole owner of drift, progress, and delegation
semantics. The implementation must centralize the exact v0.2-v0.8 compatibility contract, keep the
public function/method signatures source-compatible, defer the public presentation-shape choice to
its explicit operator gate, and prove that interpretation failures have zero
scheduler-decision, presentation, adjudication, operator-sink, checkpoint-acceptance,
`record_delivery`, and persisted cursor/delivery effects. Existing transport-observation
bookkeeping before `runtime.observe` is preserved and may occur.

## Authoritative Inputs

1. `docs/specs/hybrid-drift-r6-r8-control-pack/03-selective-context-manifests.md`, Manifest G.
2. `docs/specs/r8/MAP.md`.
3. `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-spec.md`.
4. The current Sentinel source and test paths listed below.

The reviewed R8 SPEC controls any disagreement. This plan may packetize its inventory but may not
expand it.

## Verified Current Seams

| Seam | Current source | Current focused proof |
|---|---|---|
| Replay loading and serialized validation | `crates/agent-drift-sentinel/src/input.rs` | `crates/agent-drift-sentinel/tests/replay_input.rs` |
| Live fixture/typed compatibility | `crates/agent-drift-sentinel/src/live_input.rs` | `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`, `tests/live_input.rs`, `tests/live_input_adapter.rs` |
| Live scheduling/state | `crates/agent-drift-sentinel/src/live_runtime.rs` | `crates/agent-drift-sentinel/tests/live_runtime.rs` |
| Real-session delivery/cursors | `crates/agent-drift-sentinel/src/real_session_live.rs` | `crates/agent-drift-sentinel/tests/real_session_live.rs` |
| Replay report and operator presentation | `crates/agent-drift-sentinel/src/operator_surface.rs` | `crates/agent-drift-sentinel/tests/operator_surface.rs`, `tests/live_end_to_end.rs` |
| Adjudication operator summary | `crates/agent-drift-sentinel/src/adjudication.rs` (read-only dependency/call-path evidence) | Existing `tests/adjudication.rs` plus R8-4 parity assertions in edit-manifest tests; no adjudication source/test edit |
| Public request/result facade | `crates/agent-drift-sentinel/src/lib.rs` | compile-time signature assertions added in focused Sentinel tests |

The only new planned path is:

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`, including its in-module
  `#[cfg(test)] mod tests` matrix.

## Non-Negotiable Boundaries

- Do not edit analyzer, compactor, or checkpoint schema code.
- Do not edit `scheduler.rs`, `adjudication.rs`, `cli.rs`, `operator_sink.rs`, or their tests as an
  implementation shortcut.
- Do not change scheduler trigger meanings, cooldown/debounce state, adjudication behavior, policy,
  request-shaping logic, request eligibility, decision semantics, operator sink behavior,
  real-session delivery ordering, checkpoint acceptance, or cursor persistence. Option B may affect
  only the preserved bytes rendered into `operator_summary`, and parity must prove them unchanged.
- Do not add generalized schema parsing. The supported set remains the exact literals v0.2 through
  v0.8.
- Do not add sentinel validation inside analyzer-owned `DelegationContext`. Missing/null v0.8
  delegation and typed/schema/session contradictions are whole-checkpoint failures; analyzer-declared
  mixed/ambiguous topology and partial/opaque visibility remain valid typed input.
- Do not consult a previous checkpoint from another session.
- Do not change the public signatures of `present_checkpoint`,
  `present_checkpoint_with_previous`, `render_replay_report`, or `execute`.
- Keep `checkpoint_interpretation` crate-private: no public module declaration, public item, public
  re-export, or external interpretation integration target.
- Preserve recorded `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`: `Checkpoint.delegation` alone is
  not a presence carrier, so only the exact final-facade predicate is permitted. Changing the
  upstream analyzer `Checkpoint` schema is outside R8.
- Do not use `unwrap`, panic, default acceptance, error swallowing, or v0.2 fallback to cross the
  fallible v0.3-v0.8 seam.
- Do not edit `real_session_live.rs` in the planned implementation. Existing ordering already calls
  `LiveRuntime::observe` before recording delivery and persisting cursor state. Monitor-closure
  tracking, pending-poll state, and emission-ordinal allocation already occur before that call and
  are preserved/out of scope. R8 proves only that interpretation failure produces no scheduler
  decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted
  cursor/delivery, or checkpoint acceptance; it authorizes no rollback production edit.

Any required expansion is a structured decision and reviewed spec amendment, not an opportunistic
fix.

## Target Architecture

```text
serialized checkpoint
        |
        v
validate_serialized_checkpoint -- exact v0.2-v0.8 required-shape table
        |
        v
typed Checkpoint + previous checkpoint from the same session only
        |
        v
interpret_checkpoint ---------- CheckpointInterpretation / CheckpointContractError
        |                            | cursor, fingerprint, flagged/max score
        |                            | posture/evidence from analyzer facts
        |                            | typed delegation projection
        |
        +--> replay fallible core --> unchanged scheduler --> typed presentation
        |
        +--> live fallible core ----> unchanged scheduler --> typed presentation

public infallible compatibility facades
        --> centralized non-validating compatibility projection
        --> the same typed renderer
```

The core replay and live paths never call the infallible compatibility facades as validation
boundaries. Replay validates/interprets the complete selected set before constructing scheduler or
report state. Live interprets the event before mutating accepted-checkpoint, previous-checkpoint,
scheduler, or cursor-visible runtime state.

## Architecture Decisions

1. **One semantic entry point.** `interpret_checkpoint` receives only the current checkpoint and an
   optional previous checkpoint proven to be from the same session. There is no replay/live flag.
2. **One literal compatibility owner.** The new module owns the supported schema enum, serialized
   required-shape rules, the exact four sentinel-owned typed non-empty fields, analyzer-state
   normalization, bounded v0.2 history, evidence selection, and typed v0.8 delegation projection.
3. **Structured central errors, adapter-owned context.** `CheckpointContractError` owns stable
   schema/checkpoint/field detail. Replay maps it into `InputError`, retaining artifact path/line
   when available, then existing `SentinelError::Input`. Live maps it into `LiveInputError`,
   retaining fixture path/line or event/source detail, then existing `LiveRuntimeError::Input`.
4. **Locked total public facades.** The four public function signatures remain exact. Infallible
   presentation/report APIs use crate-private `project_checkpoint_compatibility`, which returns the
   same `CheckpointInterpretation` consumed by `present_interpretation`. Exact supported literals
   select `CheckpointProjectionProfile::Schema(...)` without validating the typed shape; every other
   typed facade literal selects distinct `CompatibilityOnly` and preserves current total behavior
   without being relabeled v0.2. The helper does not call or catch `interpret_checkpoint` and does
   not validate schema, fields, explicit state, or same-session history. Both entries call one
   private posture/evidence/delegation normalization. Core replay/live remain on the unchanged
   fallible path and can never select `CompatibilityOnly`.
5. **Presentation receives facts; Option B is selected.** Core
   `present_interpretation` may format, truncate, order, label, and apply the existing warning
   policy. It may not classify analyzer state, recognize legacy evidence prefixes, or infer
   delegation. Core interpretation/presentation carries
   `CheckpointInterpretation.delegation: Option<DelegationContext>`. Recorded decision
   `R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` preserves the public shape and confines exactly one
   equality predicate,
   `self.checkpoint.schema_version == "v0.8"`, to the legacy public facade
   `CheckpointPresentation::render_console_block`. Because unchanged `CheckpointPresentation`,
   `ReplayReport`, and `LiveObservation` shapes erase internal optional presence before their later
   render calls, Option B explicitly permits that frozen facade to serve the exact allowed owner
   set `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. Fail-closed
   AST/call-path proof must independently assert both that exact set equality and exact raw direct
   `render_console_block` call-expression count `4`: two calls in
   `ReplayReport::to_console_text`, one in `cli::run_live`, and one in
   `adjudication::shape_request` while constructing `operator_summary`. Any fifth direct call fails,
   even when added inside an allowed owner. Contract validation,
   typed interpretation, posture/evidence/delegation projection, scheduling, presentation
   construction, and adjudication request-shaping/decision logic may not call or reproduce this
   exception. This is a localized compatibility/presentation schema-coupling exception with an
   explicit three-consumer parity-test cost; on the adjudication path, the predicate may affect only
   preserved rendered operator-summary content, and `adjudication.rs` remains unchanged read-only
   evidence. Option B does not claim the internal `Option` reaches final rendering. The operator
   rejected Option A's public-field/source-compatibility cost. No R8-4 source/test symbol is editable
   until the six-file manifest amendment itself is committed and fresh-review-clean.
6. **Crate-private implementation/testing.** `checkpoint_interpretation` and every additive item are
   `pub(crate)` only as needed. R8-1 RED/GREEN is an in-module `#[cfg(test)]` unit-test matrix; public-
   flow integration tests begin with R8-2/R8-3 and exercise the existing public facades.
7. **No protected-boundary edits.** Scheduler, adjudication, sinks, and real-session delivery/cursor
   code remain unchanged. Tests preserve pre-observe transport bookkeeping and establish the
   narrower post-interpretation zero-effect boundary.

## Doubt Claims And Disproof Evidence

These are hypotheses to disprove during implementation and fresh review, not completion claims.

| Claim | Why it matters | Required disproof attempt |
|---|---|---|
| One typed interpretation can preserve v0.2 and v0.3-v0.8 behavior without moving semantics out of the analyzer. | A wrong normalization creates a second analyzer. | Exact version matrix, explicit-state precedence, same-session history, typed delegation acceptance, and parent-orchestration negative witnesses. |
| Fallible core paths can coexist with locked infallible facades without fallback or panic. | Error swallowing would make malformed checkpoints appear valid. | Compile-time signature locks, structured error-chain tests, source-order inspection, and searches excluding `unwrap`/panic/fallback bridges. |
| Live contract failures can be rejected before scheduler/presentation/acceptance/delivery effects, while preserving existing pre-observe transport bookkeeping. | A post-interpretation delivery/cursor effect breaks restart safety, but claiming rollback of monitor/poll/ordinal bookkeeping would exceed R8. | Before/after runtime snapshots plus static real-session ordering prove no scheduler decision, presentation, adjudication, operator sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance after failure; monitor-closure, pending-poll, and emission-ordinal observations are permitted. |
| Core operator construction can remain typed and predicate-free while the selected public boundary preserves compatibility. | Schema/state inference before final rendering would retain the duplication R8 exists to remove; pretending the unchanged public shape carries delegation presence would lose information. | The explicit presence-representation decision; fail-closed AST/call-path proof of exact allowed owner-set equality `{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}` plus exact raw direct `render_console_block` call-expression count `4` with any fifth call rejected; and separate behavior proof for exact facade/core output, final replay/live rendering, and unchanged adjudication `operator_summary` bytes/full requests. |

Fresh-context built-in `default` review is required after every packet before the next cutover.
Cross-model review is skipped for this autonomous plan because external CLI use is explicitly
forbidden; no external CLI review may be substituted silently.

## Dependency Order

```text
fresh-review-clean MAP/SPEC/PLAN/TASKS + reconciled CTX-R8-02 authority
        |
        v
R8-1 shared typed interpretation and structured errors
        |
        v
R8-2 replay fallible core and typed replay rendering
        |
        v
R8-3 live adapter/runtime cutover with pre-mutation failure
        |
        v
R8-4 operator facade consolidation and signature lock
        |
        v
R8-5.1 parity/matrix proof
        |
        v
R8-5.2 protected-boundary zero-effect proof
        |
        v
R8-6 final wall and canonical family status checkpoint
```

The order is sequential. R8-2 and R8-3 share the central seam; R8-4 removes the remaining facade
duplication only after both core paths are using typed interpretation. R8-5.2 is not an independent
branch: it may start only after the complete R8-5.1 commit/fix series has landed and received fresh
independent `CLEAN` review everywhere. The packets never run concurrently.

## Universal Packet Protocol

Every R8 implementation packet follows the same bounded cycle:

1. Confirm the worktree is clean and the preceding packet is fresh-review-clean.
2. Confirm the exact packet manifest. No file outside it may change.
3. Run GitNexus upstream impact for every existing symbol named by the packet before editing it.
   New symbols have no pre-existing graph target; record that explicitly rather than pretending an
   impact result exists.
4. If any impact result is HIGH or CRITICAL, stop and obtain a structured operator decision before
   the first production-symbol edit. The recorded decisions below cover only their exact named
   seams. Any additional HIGH/CRITICAL existing symbol requires a new stable ID and operator reply.
5. Use TDD in this exact order: write and run the smallest failing contract/parity witness; record
   the RED; make the smallest production change; run the focused GREEN proof and adjacent
   regression target. Do not commit the intentional RED state.
6. Run formatting and focused clippy/tests. Inspect static ordering/ownership checks named by the
   packet.
7. Stage only the packet manifest, then run:

   ```bash
   npx gitnexus detect-changes --scope staged -r 97a0-substrate
   git diff --cached --check
   git diff --cached -- <exact packet manifest>
   ```

8. Commit one atomic packet. A review fix is a separate bounded follow-up commit and may not be
   hidden in the next packet.
9. Invoke fresh independent built-in `default` adversarial review. Reconcile every finding; do not
   start the next packet until the landed packet/fix series is `CLEAN`.

## R8-1 — Shared Typed Interpretation And Error Contract

Create the origin-neutral module and prove the exact central compatibility matrix before any core
adapter cutover.

Exact manifest (2 files):

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` (new);
- `crates/agent-drift-sentinel/src/lib.rs`.

The RED witness must cover the exact v0.2-v0.8 schema set; v0.3-v0.8 explicit state; v0.4 turn
context; v0.5 archetype; v0.6-v0.8 progress; v0.8 non-null typed delegation; exactly the four
sentinel-owned non-empty fields; same-session history; v0.2 bounded legacy behavior; explicit-state
precedence; structured errors; valid mixed/ambiguous and partial/opaque analyzer projections; and a
negative parent-orchestration witness. All RED/GREEN cases live under `#[cfg(test)] mod tests`
inside the new module. The smallest implementation adds the schema profile,
serialized validator, typed input/result, error types, interpretation, and minimal crate-private
module wiring. The module/items stay crate-private with no public re-export or API expansion. No
replay/live production path changes in this packet.

Focused proof:

```bash
cargo test -p agent-drift-sentinel checkpoint_interpretation::tests --lib -- --nocapture
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

## R8-2 — Replay Fallible Core And Typed Replay Rendering

Route replay through complete-set validation/interpretation before scheduler/report construction,
add the internal fallible report core, and keep public replay/report signatures exact.

Status/dependency gate: **UNSTARTED / BLOCKED** until R8-1 is landed and fresh-review-clean **and**
the operator replies `DECISION R8-2-HIGH-IMPACT-REPLAY-LOADER-01: A`. A `B` reply stops R8-2 for
respec; silence is not authorization.

Exact manifest (5 files):

- `crates/agent-drift-sentinel/src/input.rs`;
- `crates/agent-drift-sentinel/src/lib.rs`;
- `crates/agent-drift-sentinel/src/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/replay_input.rs`;
- `crates/agent-drift-sentinel/tests/operator_surface.rs`.

Replay raw validation delegates to the central validator, and typed errors map deterministically to
`InputError` with artifact path/line when present. `execute` uses the fallible report core and
preserves `CheckpointContractError -> InputError -> SentinelError::Input`. A failure produces no
report, scheduler decision, presentation, adjudication request, or next cursor. Sorting, mixed-
version failure, selection, and cursor behavior remain unchanged. The public `render_replay_report`
facade remains infallible and source-compatible but is not called by `execute`.

Complete existing-symbol impact inventory before edits:

- `input.rs`: `load_replay_bundle`, `read_checkpoint_jsonl_file`,
  `validate_checkpoint_contract`, `require_non_null_field`, and
  `validate_drift_score_state_contract`;
- `lib.rs`: `execute`;
- `operator_surface.rs`: `render_replay_report`;
- additive `try_render_replay_report` and error-adapter helpers: new/no pre-existing graph target.

Refreshed upstream impact for `load_replay_bundle` is HIGH: 17 direct dependents, one affected
`execute` process, and one affected module. That result is bound to
`R8-2-HIGH-IMPACT-REPLAY-LOADER-01`; no edit to the loader is allowed before the operator reply.

Focused proof:

```bash
cargo test -p agent-drift-sentinel --test replay_input -- --nocapture
cargo test -p agent-drift-sentinel --test operator_surface -- --nocapture
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

## R8-3 — Live Adapter And Runtime Cutover

Delegate live serialized/typed compatibility to the central contract and make
`LiveRuntime::observe` interpret checkpoint-ready events before any state or scheduler mutation.

Status/dependency gate: **UNSTARTED / BLOCKED** until R8-2 is landed and fresh-review-clean **and**
the operator separately replies `A` to both
`R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01` and
`R8-3-HIGH-IMPACT-LIVE-RUNTIME-01`. A `B` reply stops the owning seam for respec; silence on either
ID is not authorization.

Exact manifest (5 files):

- `crates/agent-drift-sentinel/src/live_input.rs`;
- `crates/agent-drift-sentinel/src/live_runtime.rs`;
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`;
- `crates/agent-drift-sentinel/tests/live_input.rs`;
- `crates/agent-drift-sentinel/tests/live_runtime.rs`.

`LiveCheckpointCompatibility` and `verify_live_checkpoint_compatibility` remain behavior-compatible
for supported typed checkpoints. Raw fixture errors retain path/line; typed errors retain
checkpoint/source detail; the runtime returns
`CheckpointContractError -> LiveInputError -> LiveRuntimeError::Input`. Before/after snapshot tests
must prove no accepted checkpoint, previous-by-session update, scheduler change, processed-event
increment, presentation, or cursor-visible change on error. Synthetic trigger behavior and the
separation between repeated-failure trigger and analyzer posture remain unchanged.

Complete existing-symbol impact inventory before edits:

- `live_input.rs`: `validate_live_fixture_contract`, `require_non_null_fixture_field`,
  `schema_requires_turn_context`, `schema_requires_session_archetype`,
  `schema_requires_session_progress`, `validate_fixture_drift_score_state_contract`,
  `compatibility_gap`, and `verify_live_checkpoint_compatibility`;
- `live_runtime.rs`: file-disambiguated `LiveRuntime::observe` (`observe` in
  `crates/agent-drift-sentinel/src/live_runtime.rs`);
- additive compatibility/error-adapter helpers: new/no pre-existing graph target.

Refreshed upstream impacts are separately HIGH: `verify_live_checkpoint_compatibility` has 17
direct dependents, and file-disambiguated `LiveRuntime::observe` has 17 direct dependents. They are
bound to the two stable R8-3 decision IDs above; neither symbol may be edited before its reply.

Focused proof:

```bash
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_input -- --nocapture
cargo test -p agent-drift-sentinel --test live_runtime -- --nocapture
cargo test -p agent-drift-sentinel --test live_input_adapter -- --nocapture
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

## R8-4 — Presentation-Only Operator Facades

Remove remaining schema/state/evidence inference from `operator_surface.rs`, route public facades
through centralized non-validating projection and `present_interpretation`, and lock exact public
signatures plus current total behavior, including unsupported typed facade input.

Status/dependency gate: **UNSTARTED / BLOCKED ONLY ON FRESH INDEPENDENT REVIEW OF LANDED DOCS-ONLY
AMENDMENT `849393029`**. R8-3 commit `963a8202` is fresh independent built-in `default` `CLEAN`.
The operator recorded `R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`,
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B`,
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`,
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`, and
`R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01: A`. No R8-4 source/test edit has started. The amendment
candidate `849393029` is pending fresh independent review, has no review result, and makes no
`CLEAN` claim.

Exact manifest (6 files):

- `crates/agent-drift-sentinel/Cargo.toml`;
- `Cargo.lock`;
- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`;
- `crates/agent-drift-sentinel/src/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`.

Read-only dependency/call-path evidence outside that six-file edit manifest:

- `crates/agent-drift-sentinel/src/adjudication.rs` — `shape_request` currently constructs
  `operator_summary` from `CheckpointPresentation::render_console_block(None)`. R8-4 must not edit
  this file or any adjudication policy/request-shaping logic; tests in the edit manifest lock its
  existing bytes and full request behavior.

Recorded Option B adds exactly one dev-only parser dependency,
`syn = { version = "2", features = ["full", "visit"] }`, in the Sentinel crate and the matching
`Cargo.lock` dependency-edge change. Live `Cargo.lock` already contains transitive `syn` `2.0.117`,
but no workspace/crate manifest exposes a Rust AST parser. The expected lock impact is adding `syn`
to the `agent-drift-sentinel` package dependency list while retaining the locked package/version;
any resolver expansion requires a stop. Decision
`R8-4-PRESENTATION-DELEGATION-PRESENCE-01: B` accepts this compile-time/lockfile cost. No runtime
dependency or unrelated lockfile update is permitted. R8-4 re-runs, but does not edit, the already-owned
`live_checkpoint_compatibility` proof.

Compile-time function-pointer assertions lock `present_checkpoint`,
`present_checkpoint_with_previous`, `CheckpointPresentation::render_console_block`, and
`render_replay_report`; R8-2 locks `execute`. The typed renderer may format delegation and other
analyzer-owned facts but may not validate or infer them. Core delegation projection and
presentation construction are controlled by
`CheckpointInterpretation.delegation: Option<DelegationContext>`. That optional value cannot be
reconstructed from `CheckpointPresentation.checkpoint.delegation` alone because pre-v0.8 absence
deserializes to `DelegationContext::default()` and the public presentation struct has no presence
carrier. Recorded Option B preserves the unchanged public shape plus exactly one equality predicate,
`self.checkpoint.schema_version == "v0.8"`, inside the named legacy public facade
`CheckpointPresentation::render_console_block`. Under Option B, constructing the unchanged
`CheckpointPresentation` and then placing it in unchanged `ReplayReport` or `LiveObservation`
erases the internal `Option` before later rendering. The frozen facade therefore may serve the
exact allowed owner set `{ReplayReport::to_console_text, cli::run_live,
adjudication::shape_request}`, with exact raw direct `render_console_block` call-expression count
`4`: two replay, one live, and one adjudication. Any fifth direct call fails even within one of the
three allowed owners. The sole predicate reconstructs the optional delegation argument at the final
presentation boundary. Contract validation, typed interpretation,
posture/evidence/delegation projection, scheduling, presentation/report/observation construction,
and adjudication policy/request-shaping or decision logic contain no R8-4 presence predicate.
Option B is an explicit localized compatibility/presentation coupling exception with parity cost,
not a claim that internal optional presence survives to final rendering and not authorization to
edit `adjudication.rs`. On the adjudication path, its predicate may affect only preserved rendered
operator-summary content; exact parity must show unchanged `operator_summary` bytes and full
requests. The operator rejected Option A's public-field/source-compatibility cost. Changing the
upstream analyzer `Checkpoint` schema is not an R8-4 option.

The amended central seam is exact and implementable. Add crate-private
`CheckpointProjectionProfile::{Schema(CheckpointSchemaVersion), CompatibilityOnly}` plus total
`project_checkpoint_compatibility(CheckpointCompatibilityProjectionInput<'_>) ->
CheckpointInterpretation`. `interpret_checkpoint` validates exactly as it does now, then both
entries call one private projection routine that owns posture, evidence, delegation, cursor,
fingerprint, flagged, and maximum-score normalization. Exact supported typed facade literals select
`Schema(...)` without validating their typed shape. Every other typed facade literal selects
`CompatibilityOnly`, applies the existing legacy-compatible facade projection, and is never stored
or reported as `CheckpointSchemaVersion::V0_2`. The helper treats its supplied optional previous
checkpoint exactly as the current total facade does; it performs no schema/field/explicit-state/
same-session validation, does not call/catch `interpret_checkpoint`, and cannot be called by core
replay/live. `present_checkpoint`, `present_checkpoint_with_previous`, and `render_replay_report`
delegate only to that total projection and `present_interpretation`; the review-clean `input.rs`,
`live_input.rs`, and `live_runtime.rs` call sites do not change.

The compatibility facades may not contain supported-version tables, analyzer-state classification,
legacy evidence-prefix recognition, panic/unwrap, error swallowing, or failure fallback. The single
Option B equality is a presence-only legacy exception, not validation or version-table ownership.

Complete existing-symbol impact inventory before edits/removals:

- public/facade methods and functions: `CheckpointPresentation::render_console_block`,
  `render_replay_report`, `present_checkpoint`, `present_checkpoint_with_previous`,
  `classify_checkpoint`, and `warning_fingerprint`;
- delegation formatting touched by the render behavior lock: `format_delegation_summary`,
  `format_delegation_topology`, and `format_child_work_visibility`;
- posture/evidence helpers moved or removed from `operator_surface.rs`:
  `classify_checkpoint_posture`, `uses_explicit_analyzer_state`,
  `classify_checkpoint_posture_from_state`, `classify_checkpoint_posture_legacy`,
  `checkpoint_had_active_class`, `score_has_historical_evidence`,
  `historical_reason_prefixes`, `collect_evidence_lines`,
  `collect_state_backed_evidence_lines`, `collect_legacy_evidence_lines`,
  `push_evidence_lines`, and `max_flagged_score`;
- existing central items that the amended implementation expects to edit:
  `CheckpointInterpretation`, `CheckpointSchemaVersion::from_literal`, `interpret_checkpoint`,
  `checkpoint_posture`, and `checkpoint_evidence` in `checkpoint_interpretation.rs`;
- additive `CheckpointCompatibilityProjectionInput`, `CheckpointProjectionProfile`,
  `project_checkpoint_compatibility`, and any new private shared-projection adapter: new/no
  pre-existing graph target.

Refreshed upstream impact for `uses_explicit_analyzer_state` is HIGH: 10 impacted symbols, two
direct dependents, zero currently mapped processes, and three affected modules. The operator
recorded `R8-4-HIGH-IMPACT-EXPLICIT-STATE-01: A`. Refreshed upstream impact for
`score_has_historical_evidence` is HIGH: five impacted symbols, two direct dependents, zero affected
processes, and three modules; the operator recorded
`R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01: A`. Refreshed upstream impact for
`push_evidence_lines` is HIGH: seven impacted symbols, three direct dependents, one affected
`execute` process, and three modules; the operator recorded
`R8-4-HIGH-IMPACT-EVIDENCE-LINES-01: A`. These decisions authorize only the exact R8-4 move/removal
contract. Before code, run upstream impact for every listed existing central item and every other
existing symbol actually edited; any additional HIGH/CRITICAL result requires its own decision.

Focused proof:

```bash
cargo test -p agent-drift-sentinel checkpoint_interpretation::tests --lib -- --nocapture
cargo test -p agent-drift-sentinel --test operator_surface -- --nocapture
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_end_to_end -- --nocapture
cargo test -p agent-drift-sentinel --test adjudication -- --nocapture
! rg -n 'DriftState|historical_reason_prefixes|uses_explicit_analyzer_state|classify_checkpoint_posture' crates/agent-drift-sentinel/src/operator_surface.rs
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

`tests/operator_surface.rs` must provide the exact schema-check/call-owner proof; the removed line-
oriented regex and token/delimiter counter are not acceptance evidence. Parse
`src/operator_surface.rs` plus the Sentinel production source set (including read-only `cli.rs` and
`adjudication.rs`) with `syn::parse_file` and run a fail-closed `syn::visit::Visit` ownership/data-
dependency pass. It must
visit every direct `schema_version` field/path occurrence, propagate local aliases/assignments, and
record every enclosing predicate/control form: `==`, `!=`, boolean nesting, `if`/`while`, `match`
scrutinees and guards, parsed `matches!`, method calls, struct field/pattern uses, constants, and aliases. Each occurrence and
predicate is attributed to its owning Rust item; any unparsed macro, unclassified occurrence, or
unowned form fails the test. Recorded Option B asserts exactly one direct field use and one predicate in the whole
file, both owned by `CheckpointPresentation::render_console_block`, zero in every other item, with
the allowed predicate structurally equal to direct `self.checkpoint.schema_version == "v0.8"` using
that literal. The fail-closed production call-path inventory also attributes every direct facade
call to its owning item and asserts exact allowed owner-set equality
`{ReplayReport::to_console_text, cli::run_live, adjudication::shape_request}`. Independently, it asserts exact raw direct
`render_console_block` call-expression count `4`, partitioned as two in
`ReplayReport::to_console_text`, one in `cli::run_live`, and one in
`adjudication::shape_request`. Any fifth direct call fails even inside an allowed owner; any new or
unclassified production call owner also fails. The AST test proves syntax, item ownership, call ownership, and conditional/count policy
only; it must not claim that it proves runtime data flow or behavior.

Central in-module tests separately prove that valid typed v0.2-v0.8 inputs produce equal
renderer-consumed facts through `interpret_checkpoint` and `project_checkpoint_compatibility`;
unsupported typed facade input selects `CompatibilityOnly` rather than `Schema(V0_2)` and preserves
the existing total legacy-compatible projection; typed shapes currently accepted by public facades
remain total; and core unsupported-schema, field-gap, explicit-state, and cross-session failures stay
unchanged. Source inspection must prove the total helper does not call/catch `interpret_checkpoint`.

Behavior proof is separate and locked to recorded Option B. v0.2-v0.7 and v0.8 legacy-facade output must match the intended absence/presence
output, and edit-manifest tests must exercise `ReplayReport::to_console_text`, final
`LiveObservation.presentation.render_console_block` output corresponding to `cli::run_live`, and
`adjudication::shape_request` `operator_summary` construction. Pre/post assertions lock exact
`operator_summary` bytes and full `AdjudicationRequest` equality, including request eligibility and
every non-summary field; only the facade output may flow through existing truncation into the summary; every other
request field remains unchanged. Those tests separately prove output/call-path parity and that no earlier validation,
interpretation, projection, scheduling, construction, adjudication request-shaping logic, or
decision-semantic stage uses the Option B exception; the AST test alone proves none of those runtime
facts.

## R8-5.1 — Replay/Live Parity And Compatibility Matrix

Add the focused, test-only parity wall after both cores and the operator facades are review-clean.

Exact manifest (4 files):

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` (`#[cfg(test)]` only);
- `crates/agent-drift-sentinel/tests/replay_input.rs`;
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`;
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`.

The matrix names v0.2 and each version v0.3 through v0.8 individually. Matching replay/live inputs
must agree on cursor, warning fingerprint, flagged/max score inputs, diagnostics, headlines, turn
context, archetype, progress, posture, evidence, delegation projection, session locality, and
trigger/posture separation. v0.3-v0.8 malformed explicit state must fail closed and never run v0.2
inference. This is a test-only proof packet: a RED caused by production behavior returns to the
smallest owning earlier packet and fresh review; it does not authorize production edits here.

Focused proof:

```bash
cargo test -p agent-drift-sentinel checkpoint_interpretation::tests --lib -- --nocapture
cargo test -p agent-drift-sentinel --test replay_input -- --nocapture
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_end_to_end -- --nocapture
```

## R8-5.2 — Zero-Effect And Protected-Boundary Proof

Prove adapter, runtime, and real-session error effects without editing protected production code.

Exact manifest (4 test files):

- `crates/agent-drift-sentinel/tests/live_input.rs`;
- `crates/agent-drift-sentinel/tests/live_input_adapter.rs`;
- `crates/agent-drift-sentinel/tests/live_runtime.rs`;
- `crates/agent-drift-sentinel/tests/real_session_live.rs`.

Tests preserve append-only fixture ordering, source/cursor errors, sparse startup, verified closure,
restart behavior, per-session cursor freshness, and delivery order. Existing pre-observe transport
bookkeeping may occur: monitor-closure tracking, pending-poll state, and emission-ordinal allocation.
After interpretation fails there must be no scheduler decision, presentation, adjudication, operator
sink emission, `record_delivery`, persisted cursor/delivery, or checkpoint acceptance. A need to
modify `real_session_live.rs`, scheduler, adjudication, CLI, or sinks is a stop-and-respec trigger;
no rollback production edit is authorized.

Focused proof:

```bash
cargo test -p agent-drift-sentinel --test live_input -- --nocapture
cargo test -p agent-drift-sentinel --test live_input_adapter -- --nocapture
cargo test -p agent-drift-sentinel --test live_runtime -- --nocapture
cargo test -p agent-drift-sentinel --test real_session_live -- --nocapture
```

## R8-6 — Final Wall And Canonical Family Status Checkpoint

Run the family wall only after R8-1 through R8-5.2 and all review-fix series are fresh-review-clean.
Record exact counts and receipts; do not infer a phase transition from green commands alone.

Verification wall:

```bash
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
cargo test -p agent-drift-sentinel -- --nocapture
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --nocapture
git diff --check
```

Then inspect source ordering and facade ownership:

```bash
rg -n 'interpret_checkpoint|try_render_replay_report|scheduler\.observe|present_interpretation' crates/agent-drift-sentinel/src/{operator_surface.rs,live_runtime.rs}
! rg -n 'operator_surface::render_replay_report|present_checkpoint_with_previous' crates/agent-drift-sentinel/src/{lib.rs,live_runtime.rs}
rg -n 'unwrap\(|expect\(|panic!|v0\.2' crates/agent-drift-sentinel/src/{checkpoint_interpretation.rs,input.rs,live_input.rs,live_runtime.rs,operator_surface.rs}
```

The second command must confirm that `execute` and `LiveRuntime::observe` do not call compatibility
facades; the third is manually classified so existing unrelated `expect` uses are not silently
treated as proof or opportunistically changed.

Exact status-receipt manifest (5 docs):

- `docs/specs/r8/MAP.md`;
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-spec.md`;
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-plan.md`;
- `docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-tasks.md`;
- `docs/specs/hybrid-drift-r6-r8-control-pack/05-proof-decision-regression-ledger.md`.

That receipt remains candidate until fresh independent review. Any wider root/control-pack mirror or
phase-transition reconciliation is a separate docs-only packet after the receipt is clean; it is not
an implementation task and must not be smuggled into R8-6.

## Gate Coverage

Legend: **D** = direct proof, **G** = preserved guardrail, **E** = entry/review dependency.

| Task | `CTX-R8-01` | `CTX-R8-02` | `CTX-R8-03` | `CTX-R8-04` | `CTX-R8-05` | `CTX-R8-06` |
|---|---|---|---|---|---|---|
| R8-1 | G | E | D | D | G | G |
| R8-2 | G | E | D | D | D | D |
| R8-3 | G | E | D | D | D | D |
| R8-4 | G | E | D | D | D | G |
| R8-5.1 | D | E | D | D | D | D |
| R8-5.2 | G | E | D | D | G | D |
| R8-6 | D | D only after fresh review | D | D | D | D |

`CTX-R8-01` and the entry dependency `CTX-R8-02` remain proven. R8-1, R8-2, and R8-3 are landed and
fresh-review-clean. R8-4's `CTX-R8-05` direct-proof cell remains unproven until implementation and
proof complete; recorded Option B cannot be claimed from `Checkpoint.delegation` alone.

## Recorded Decisions And Future Triggers

| Stable ID | Recorded result | Evidence and exact boundary |
|---|---|---|
| `R8-2-HIGH-IMPACT-REPLAY-LOADER-01` | `A` | Authorized only the exact R8-2 loader cutover; landed series `08e0d0e2a` + `5669e1f6e` + `911dd49b` is fresh independent built-in `default` `CLEAN`. |
| `R8-3-HIGH-IMPACT-LIVE-COMPATIBILITY-01` | `A` | Authorized only the exact compatibility delegation; R8-3 commit `963a8202` is fresh independent built-in `default` `CLEAN`. |
| `R8-3-HIGH-IMPACT-LIVE-RUNTIME-01` | `A` | Authorized only the exact pre-mutation runtime cutover; R8-3 commit `963a8202` is fresh independent built-in `default` `CLEAN`. |
| `R8-4-HIGH-IMPACT-EXPLICIT-STATE-01` | `A` | `uses_explicit_analyzer_state`: HIGH, 10 impacted, 2 direct, 0 currently mapped processes, 3 modules. Authorizes only its R8-4 move/removal under the locked proof. |
| `R8-4-PRESENTATION-DELEGATION-PRESENCE-01` | `B` | Preserve public shapes/output; accept dev-only `syn`/lockfile cost and exactly one final-facade literal-v0.8 predicate plus exact owner/count and behavior parity proof. |
| `R8-4-HIGH-IMPACT-HISTORICAL-EVIDENCE-01` | `A` | `score_has_historical_evidence`: HIGH, 5 impacted, 2 direct, 0 processes, 3 modules. Authorizes only the central semantic-ownership move/removal. |
| `R8-4-HIGH-IMPACT-EVIDENCE-LINES-01` | `A` | `push_evidence_lines`: HIGH, 7 impacted, 3 direct, 1 affected `execute` process, 3 modules. Authorizes only the typed-evidence cutover. |
| `R8-4-COMPATIBILITY-PROJECTION-MANIFEST-01` | `A` | Expands only R8-4's exact manifest from five files to six by adding `src/checkpoint_interpretation.rs`; requires the total central compatibility profile/projection contract in this amendment. |

No table row authorizes an additional HIGH/CRITICAL symbol, a seventh R8-4 file, or code before this
docs amendment is committed and fresh-review-clean. Any such need requires a new structured
`DECISION REQUIRED` gate.

Stop and emit a concrete `DECISION REQUIRED <ID>: A|B` prompt when any of these occurs:

1. GitNexus reports any additional HIGH or CRITICAL production symbol not covered by the exact
   table above. Never combine multiple symbol gates into one ID.
2. A packet needs more than its exact manifest; for R8-4, any seventh file is a new gap.
3. A public signature or current total facade behavior cannot be preserved without panic, fallback, or
   error swallowing.
4. A test suggests changing analyzer, compactor, schema, scheduler, adjudication, CLI, sink, or
   real-session delivery/cursor behavior.
5. Sentinel would need to validate fields inside `DelegationContext`, infer child state from parent
   orchestration, or consult cross-session history.
6. A v0.3-v0.8 failure appears to require v0.2 inference or generalized version parsing.
7. The final wall exposes an unrelated pre-existing failure; classify and report it instead of
   widening R8.

## Completion Condition

R8 is not complete merely because R8-IMPLEMENT is active at entry, code compiles, or a focused suite
is green. Completion requires every
packet and fix series fresh-review-clean, the full wall recorded with exact counts, the canonical
status receipt fresh-review-clean, `CTX-R8-01..06` reconciled against the control ledger, and a
separate authorized phase/status transition. R8-1 through R8-3 are landed and fresh-review-clean;
R8-4 remains unstarted until this amendment is committed and fresh-review-clean. This plan claims
no R8-4 or family completion.
