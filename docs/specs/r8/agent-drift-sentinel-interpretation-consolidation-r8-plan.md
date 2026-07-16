# Implementation Plan: R8 Sentinel Interpretation Consolidation

Canonical path:
`docs/specs/r8/agent-drift-sentinel-interpretation-consolidation-r8-plan.md`

Status: **CANDIDATE / AWAITING FRESH INDEPENDENT BUILT-IN `default` REVIEW / `CTX-R8-02`
UNPROVEN / R8-IMPLEMENT BLOCKED**.

The MAP/SPEC contract series `698c766f9` + `f5865fb7` + `95529809` is reported by the current
session handoff as fresh independent built-in `default` `CLEAN`. This PLAN and its companion TASKS
are new candidate artifacts and make no review-clean or implementation-ready claim. The checked-in
MAP/SPEC and control-pack mirrors still contain their pre-review status wording; reconciling those
mirrors is outside this two-file authoring boundary. `CTX-R8-02` therefore remains unproven and no
R8 source or test edit may start until the complete MAP/SPEC/PLAN/TASKS family is freshly reviewed
and the authority status is reconciled.

## Objective

Make replay and live sentinel paths consume one origin-neutral, fallible typed checkpoint
interpretation while preserving the analyzer as the sole owner of drift, progress, and delegation
semantics. The implementation must centralize the exact v0.2-v0.8 compatibility contract, keep the
public sentinel facades source-compatible, and prove that interpretation failures have zero
scheduler, presentation, adjudication, sink, checkpoint-acceptance, delivery, and cursor effects.

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
| Public request/result facade | `crates/agent-drift-sentinel/src/lib.rs` | compile-time signature assertions added in focused Sentinel tests |

The new planned paths are exactly:

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`;
- `crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs`.

## Non-Negotiable Boundaries

- Do not edit analyzer, compactor, or checkpoint schema code.
- Do not edit `scheduler.rs`, `adjudication.rs`, `cli.rs`, `operator_sink.rs`, or their tests as an
  implementation shortcut.
- Do not change scheduler trigger meanings, cooldown/debounce state, adjudication behavior, operator
  sink behavior, real-session delivery ordering, checkpoint acceptance, or cursor persistence.
- Do not add generalized schema parsing. The supported set remains the exact literals v0.2 through
  v0.8.
- Do not add sentinel validation inside analyzer-owned `DelegationContext`. Missing/null v0.8
  delegation and typed/schema/session contradictions are whole-checkpoint failures; analyzer-declared
  mixed/ambiguous topology and partial/opaque visibility remain valid typed input.
- Do not consult a previous checkpoint from another session.
- Do not change the public signatures of `present_checkpoint`,
  `present_checkpoint_with_previous`, `render_replay_report`, or `execute`.
- Do not use `unwrap`, panic, default acceptance, error swallowing, or v0.2 fallback to cross the
  fallible v0.3-v0.8 seam.
- Do not edit `real_session_live.rs` in the planned implementation. Existing ordering already calls
  `LiveRuntime::observe` before recording delivery and persisting cursor state; R8 proves that
  boundary with tests instead of redesigning it.

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
4. **Locked public facades.** The four public function signatures remain exact. Infallible
   presentation/report APIs use centralized non-validating projection for already-typed supported
   inputs; they neither validate nor silently recover from a contract failure.
5. **Presentation receives facts.** `present_interpretation` may format, truncate, order, label, and
   apply the existing warning policy. It may not inspect schema strings, classify analyzer state,
   recognize legacy evidence prefixes, or infer delegation.
6. **No protected-boundary edits.** Scheduler, adjudication, sinks, and real-session delivery/cursor
   code remain unchanged. Tests establish equivalence and zero-effect failure behavior.

## Doubt Claims And Disproof Evidence

These are hypotheses to disprove during implementation and fresh review, not completion claims.

| Claim | Why it matters | Required disproof attempt |
|---|---|---|
| One typed interpretation can preserve v0.2 and v0.3-v0.8 behavior without moving semantics out of the analyzer. | A wrong normalization creates a second analyzer. | Exact version matrix, explicit-state precedence, same-session history, typed delegation acceptance, and parent-orchestration negative witnesses. |
| Fallible core paths can coexist with locked infallible facades without fallback or panic. | Error swallowing would make malformed checkpoints appear valid. | Compile-time signature locks, structured error-chain tests, source-order inspection, and searches excluding `unwrap`/panic/fallback bridges. |
| Live contract failures can be rejected before any runtime or delivery state changes. | A cursor or delivery side effect on failure makes replay/live non-equivalent and breaks restart safety. | Before/after runtime snapshots plus persisted real-session state and sink/adjudication negative witnesses. |
| Operator presentation can become a pure typed renderer. | Schema/state inference in presentation would retain the duplication R8 exists to remove. | Static source checks plus replay/live rendering parity for posture, evidence, context, progress, and delegation. |

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
        +--------------------+
        v                    v
R8-5.1 parity/matrix proof   R8-5.2 protected-boundary zero-effect proof
        \                    /
         +--------+---------+
                  v
R8-6 final wall and canonical family status checkpoint
```

The order is sequential. R8-2 and R8-3 share the central seam; R8-4 removes the remaining facade
duplication only after both core paths are using typed interpretation. R8-5.1 and R8-5.2 are
logically independent proof packets but must not run concurrently if they touch the same test
workspace or review ledger.

## Universal Packet Protocol

Every R8 implementation packet follows the same bounded cycle:

1. Confirm the worktree is clean and the preceding packet is fresh-review-clean.
2. Confirm the exact packet manifest. No file outside it may change.
3. Run GitNexus upstream impact for every existing symbol named by the packet before editing it.
   New symbols have no pre-existing graph target; record that explicitly rather than pretending an
   impact result exists.
4. If any impact result is HIGH or CRITICAL, stop and obtain a structured operator decision before
   the first production-symbol edit. The current graph already reports HIGH for
   `verify_live_checkpoint_compatibility` and `LiveRuntime::observe` (17 direct test dependents
   each), so R8-3 requires such a decision unless a refreshed graph lowers the risk.
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

Exact manifest (3 files):

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs` (new);
- `crates/agent-drift-sentinel/src/lib.rs`;
- `crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs` (new).

The RED witness must cover the exact v0.2-v0.8 schema set; v0.3-v0.8 explicit state; v0.4 turn
context; v0.5 archetype; v0.6-v0.8 progress; v0.8 non-null typed delegation; exactly the four
sentinel-owned non-empty fields; same-session history; v0.2 bounded legacy behavior; explicit-state
precedence; structured errors; valid mixed/ambiguous and partial/opaque analyzer projections; and a
negative parent-orchestration witness. The smallest implementation adds the schema profile,
serialized validator, typed input/result, error types, interpretation, and minimal module/export
wiring. No replay/live production path changes in this packet.

Focused proof:

```bash
cargo test -p agent-drift-sentinel --test checkpoint_interpretation -- --nocapture
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

## R8-2 — Replay Fallible Core And Typed Replay Rendering

Route replay through complete-set validation/interpretation before scheduler/report construction,
add the internal fallible report core, and keep public replay/report signatures exact.

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
signatures and supported-input behavior.

Exact manifest (5 files):

- `crates/agent-drift-sentinel/src/checkpoint_interpretation.rs`;
- `crates/agent-drift-sentinel/src/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/operator_surface.rs`;
- `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`;
- `crates/agent-drift-sentinel/tests/live_end_to_end.rs`.

Compile-time function-pointer assertions lock `present_checkpoint`,
`present_checkpoint_with_previous`, and `render_replay_report`; R8-2 locks `execute`. The typed
renderer may format delegation and other analyzer-owned facts but may not validate or infer them.
The compatibility facades may not contain supported-version tables, analyzer-state classification,
legacy evidence-prefix recognition, panic/unwrap, or failure fallback.

Focused proof:

```bash
cargo test -p agent-drift-sentinel --test operator_surface -- --nocapture
cargo test -p agent-drift-sentinel --test live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel --test live_end_to_end -- --nocapture
! rg -n 'schema_version|DriftState|historical_reason_prefixes|uses_explicit_analyzer_state|classify_checkpoint_posture' crates/agent-drift-sentinel/src/operator_surface.rs
cargo fmt --all -- --check
cargo clippy -p agent-drift-sentinel --all-targets -- -D warnings
```

## R8-5.1 — Replay/Live Parity And Compatibility Matrix

Add the focused, test-only parity wall after both cores and the operator facades are review-clean.

Exact manifest (4 files):

- `crates/agent-drift-sentinel/tests/checkpoint_interpretation.rs`;
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
cargo test -p agent-drift-sentinel --test checkpoint_interpretation -- --nocapture
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
restart behavior, per-session cursor freshness, and delivery order. A contract failure must produce
no adjudication, sink emission, delivery record, persisted cursor, checkpoint acceptance, or
scheduler/runtime state change. A need to modify `real_session_live.rs`, scheduler, adjudication,
CLI, or sinks is a stop-and-respec trigger.

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

No row is currently proven by this candidate plan.

## Structured Decision Triggers

Stop and emit a concrete `DECISION REQUIRED <ID>: A|B` prompt when any of these occurs:

1. GitNexus reports HIGH or CRITICAL for a production symbol. R8-3 already expects
   `R8-3-HIGH-IMPACT-LIVE-CUTOVER-01` unless refreshed evidence changes the risk.
2. A packet needs more than its exact manifest or more than five files to preserve the contract.
3. A public signature or supported-input behavior cannot be preserved without panic, fallback, or
   error swallowing.
4. A test suggests changing analyzer, compactor, schema, scheduler, adjudication, CLI, sink, or
   real-session delivery/cursor behavior.
5. Sentinel would need to validate fields inside `DelegationContext`, infer child state from parent
   orchestration, or consult cross-session history.
6. A v0.3-v0.8 failure appears to require v0.2 inference or generalized version parsing.
7. The final wall exposes an unrelated pre-existing failure; classify and report it instead of
   widening R8.

## Completion Condition

R8 is not complete when code compiles or the focused suite is green. Completion requires every
packet and fix series fresh-review-clean, the full wall recorded with exact counts, the canonical
status receipt fresh-review-clean, `CTX-R8-01..06` reconciled against the control ledger, and a
separate authorized phase/status transition. Until then, R8-IMPLEMENT remains blocked or active but
incomplete according to the latest reviewed authority; this candidate plan claims neither state
transition nor family completion.
