# Spec: Agent Drift Analyzer Session Archetype R4

## Assumptions I'm Making

1. Live repo truth on `2026-06-08` is the authority: the analyzer outcome-evidence `R1` family,
   acceptance-fixture `R2`, turn-context `R3`, and sentinel trigger-headline `R3.5` packets are
   landed on this worktree; `R3.75-1` through `R3.75-3` are also landed, `R3.75-4` remains the
   remaining delegation-aware analyzer packet, and `R4` session-archetype classification remains
   the next packet family after that boundary fully lands.
2. `R4` is intentionally narrower than `R5+`. Its job is to add explicit checkpoint-local
   archetype state, not archetype-aware progress semantics, drift-scorer retuning, or sentinel
   scheduler changes.
3. The current analyzer already exports enough deterministic evidence to support a conservative
   archetype classifier without new upstream compactor schema:
   - `task_frame` objective, truth artifacts, working set, command families, and verification
     commands
   - `turn_context` activity mix, execution mode, prompt counts, and checkpoint density
   - checkpoint diagnostics such as verification density and task-frame transitions
4. Because the repo treats checkpoint-contract growth as explicit schema events, `R4` should widen
   analyzer checkpoints from `v0.4` to `v0.5` instead of silently adding archetype fields under
   `v0.4`.
5. Sentinel replay/live consumers still need to load legacy `v0.2`, `v0.3`, and `v0.4`
   checkpoints after `R4` lands, so `R4` must preserve those compatibility paths while preferring
   `v0.5` when `session_archetype` is present.
6. Archetype classification must be analyzer-computed and deterministic. `R4` should not invoke a
   model, rely on free-form commentary alone, or depend on prompt-time intent labeling to decide
   whether a checkpoint is troubleshooting, planning, autonomous implementation, or verification /
   closeout.
7. The classifier should be additive and confidence-bearing: it should combine multiple observable
   signals, carry explicit confidence, and expose evidence and counter-evidence rather than hiding
   the decision inside one drift scorer.
8. External research can shape the module boundary and taxonomy, but `R4` should not adopt a broad
   learned failure taxonomy or multi-trajectory evaluation contract as the canonical artifact.

If any of these assumptions are wrong, correct them before `R4-2+` implementation starts.

## Objective

Add analyzer-owned per-checkpoint `session_archetype` state so the hybrid-drift stack can tell
whether a checkpoint belongs to troubleshooting, planning, autonomous implementation, or verification /
closeout before `R5` starts modeling progress within that archetype.

Primary users:

- the maintainer reading analyzer checkpoints or replay/live sentinel output and needing explicit
  session-mode context instead of inferring it from turn shape alone
- the later `R5` and `R6` packet author who needs a stable machine-readable archetype seam before
  adding progress semantics or retuning `dead_end_thrash`

Success means:

- every analyzer checkpoint exported under schema `v0.5` carries structured `session_archetype`
- `session_archetype` is derived from observable evidence already present in analyzer inputs and
  `R3` turn context
- analyzer summary output exposes compact archetype inspection without replacing the existing
  session-shape metrics
- sentinel replay/live consumers accept `v0.5` checkpoints, preserve `v0.2` through `v0.4`
  compatibility, and surface the same compact archetype view without changing posture or scheduler
  behavior
- the packet proves the difference between:
  - troubleshooting that expects verification and failure churn
  - planning / brainstorming that expects directive synthesis more than verification density
  - autonomous implementation that expects sustained write/test loops against a stable objective
  - verification / closeout that expects narrowing scope and proof-oriented verification
- `R4` does not widen into `R5` progress semantics, `R6` scorer cutover, `R7` full
  delegated-session support, or `R8` sentinel
  interpretation consolidation

## Tech Stack

- Language: Rust 2021
- Target crates:
  - `crates/agent-drift-analyzer`
  - `crates/agent-drift-sentinel`
- Existing analyzer checkpoint contract:
  - `v0.4`
- New analyzer checkpoint contract owned by `R4`:
  - `v0.5`
- Existing sentinel compatibility:
  - `v0.2 | v0.3 | v0.4`
- Required sentinel compatibility after `R4`:
  - `v0.2 | v0.3 | v0.4 | v0.5`

No new crate or external dependency is required by default.

## Commands

Formatting gate:

```bash
cargo fmt --all -- --check
```

Analyzer-focused validation:

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer export_bundle -- --nocapture
cargo test -p agent-drift-analyzer end_to_end -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

Sentinel replay/live compatibility validation:

```bash
cargo test -p agent-drift-sentinel replay_input -- --nocapture
cargo test -p agent-drift-sentinel live_checkpoint_compatibility -- --nocapture
cargo test -p agent-drift-sentinel operator_surface -- --nocapture
cargo test -p agent-drift-sentinel live_end_to_end -- --nocapture
```

Optional targeted wall while iterating on the contract:

```bash
cargo test -p agent-drift-analyzer checkpoints_are_deterministic_and_session_scoped -- --nocapture
cargo test -p agent-drift-sentinel replay_input -- --nocapture
```

## Project Structure

```text
crates/agent-drift-analyzer/src/checkpoint/schema.rs
  Owns checkpoint DTOs and the exported schema contract. R4 adds `SessionArchetype` and
  `SessionArchetypeLabel` here and widens the checkpoint schema to `v0.5`.

crates/agent-drift-analyzer/src/checkpoint/mod.rs
  Owns checkpoint analysis and checkpoint construction. R4 should classify session archetype during
  checkpoint assembly using existing task-frame, diagnostics, and turn-context evidence.

crates/agent-drift-analyzer/src/checkpoint/export.rs
  Owns `summary.md`. R4 should render compact archetype inspection here without replacing the
  existing session-shape metrics.

crates/agent-drift-analyzer/tests/checkpoints.rs
  Deterministic checkpoint-contract coverage. R4 should add `session_archetype` assertions here.

crates/agent-drift-analyzer/tests/export_bundle.rs
  Summary-contract coverage. R4 should lock the operator-facing archetype rendering here.

crates/agent-drift-analyzer/tests/end_to_end.rs
  End-to-end artifact stability. R4 should prove reruns preserve `v0.5` checkpoints and summary.

crates/agent-drift-sentinel/src/input.rs
  Replay checkpoint loading and schema-version compatibility.

crates/agent-drift-sentinel/src/live_input.rs
  Live checkpoint compatibility and fixture validation.

crates/agent-drift-sentinel/src/operator_surface.rs
  Replay/live operator presentation. R4 should expose compact archetype inspection here without
  changing posture classification.

crates/agent-drift-sentinel/tests/replay_input.rs
crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs
crates/agent-drift-sentinel/tests/operator_surface.rs
crates/agent-drift-sentinel/tests/live_end_to_end.rs
  Sentinel regression walls that should lock `v0.5` loading and replay/live presentation parity.
```

## Code Style

Planned Rust style for this packet:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SessionArchetypeLabel {
    Troubleshooting,
    Planning,
    AutonomousImplementation,
    VerificationCloseout,
}

fn classify_session_archetype(turn_context: &TurnContext) -> SessionArchetypeLabel {
    if matches!(turn_context.execution_mode, TurnExecutionMode::VerificationHeavy) {
        return SessionArchetypeLabel::Troubleshooting;
    }

    SessionArchetypeLabel::Planning
}
```

Conventions:

- keep new checkpoint types in `schema.rs` with the existing serde and enum-derive pattern
- use snake_case function names and `snake_case` serialized enum labels
- keep helpers deterministic and conservative; ambiguous cases should lower confidence instead of
  inventing a new label
- keep presentation-only formatting in sentinel/operator-surface code, not analyzer scoring code

## Testing Strategy

- Framework: existing Rust unit and integration tests via `cargo test`
- Analyzer contract tests:
  - `crates/agent-drift-analyzer/tests/checkpoints.rs`
  - `crates/agent-drift-analyzer/tests/end_to_end.rs`
- Analyzer summary rendering tests:
  - `crates/agent-drift-analyzer/tests/export_bundle.rs`
- Sentinel compatibility and presentation tests:
  - `crates/agent-drift-sentinel/tests/replay_input.rs`
  - `crates/agent-drift-sentinel/tests/live_checkpoint_compatibility.rs`
  - `crates/agent-drift-sentinel/tests/operator_surface.rs`
  - `crates/agent-drift-sentinel/tests/live_end_to_end.rs`
- Coverage expectation for this packet:
  - deterministic cases for the four initial archetypes
  - legacy schema fallback coverage for `v0.4`
  - replay/live parity coverage for `v0.5` presentation

Two-layer expectation:

- `R4` should use a lower deterministic intent-evidence layer as classifier input, similar in shape
  to AgentLens's context-sensitive phase labeling.
- That lower layer should summarize behavior such as exploration-like, implementation-like,
  verification-like, and orchestration/bookkeeping activity from trajectory evidence rather than
  from prompt wording alone.
- The exported `session_archetype` is session type at the current checkpoint, derived from the
  cumulative session prefix up to that checkpoint. Turn-local evidence is an input, not the final
  scope of the label.

## Boundaries

- Always:
  - keep `R4` limited to checkpoint-local session-archetype classification
  - widen the checkpoint schema explicitly from `v0.4` to `v0.5`
  - preserve `v0.2` through `v0.4` sentinel compatibility while adding `v0.5`
  - keep classification deterministic, evidence-backed, and auditable
  - run the focused analyzer and sentinel validation commands before claiming the packet landed
- Ask first:
  - changing the initial four-archetype taxonomy
  - adding dependencies or model-based classification
  - widening scope into `R5`, `R6`, or scheduler-policy changes
  - changing compactor schema or analyzer input artifacts upstream of this packet
- Never:
  - silently add `session_archetype` under `v0.4`
  - couple archetype labels to drift severity or scheduler triggers in this packet
  - replace deterministic evidence with prompt-only or commentary-only classification
  - treat review-draft docs as proof that implementation is complete

## Success Criteria

- Analyzer checkpoints emitted by `R4` use `schema_version = "v0.5"` and serialize required
  `session_archetype` state on every checkpoint.
- `session_archetype` exposes one of the four initial labels plus explicit confidence and evidence.
- the four initial labels are `troubleshooting`, `planning`, `autonomous_implementation`, and
  `verification_closeout`
- Analyzer summary output renders compact archetype inspection without removing existing
  turn-context or calibration reporting.
- Sentinel replay/live loaders accept `v0.5` while preserving `v0.2`, `v0.3`, and `v0.4`.
- Replay and live operator surfaces render the same archetype view for matching checkpoints.
- The packet lands without introducing `R5` progress semantics or `R6` scorer changes.
- kickoff-prompt signals remain bounded priors and do not override contradictory behavioral
  evidence

## Open Questions

- Should `planning` and `brainstorming` remain one combined initial archetype, or do you want that
  split deferred but called out explicitly in the docs?
- Should explicit kickoff-prompt priors ship in the first `R4` implementation packet, or should
  the first code landing stay fully behavior-only and defer prompt priors to a narrow follow-on?

## Packet Boundary

This packet is **checkpoint-local session-archetype classification only**.

In scope:

- define one analyzer-owned `SessionArchetype` contract for every checkpoint
- define one explicit `SessionArchetypeLabel` enum with the initial labels:
  - `troubleshooting`
  - `planning`
  - `autonomous_implementation`
  - `verification_closeout`
- attach confidence plus evidence / counter-evidence to the archetype decision
- widen analyzer checkpoint export from schema `v0.4` to `v0.5`
- derive archetype deterministically from existing task-frame, turn-context, diagnostics, and
  command-observation signals
- allow kickoff prompt shape, explicit skill calls, and orchestration-prompt markers only as
  bounded prior evidence; they may influence confidence but must not override contradictory
  behavioral evidence
- render compact archetype information in analyzer `summary.md`
- preserve replay/live loading by extending sentinel compatibility from
  `v0.2 | v0.3 | v0.4` to `v0.2 | v0.3 | v0.4 | v0.5`
- surface compact archetype output in replay/live operator presentation without changing posture,
  severity, or trigger logic
- add focused analyzer and sentinel tests that lock the `R4` contract

Out of scope:

- progress semantics such as advancing failure frontier, narrowing candidate sets, or
  verification-wall movement
- any `dead_end_thrash`, `wrong_plan_branch`, or `truth_grounding_gap` scoring changes
- scheduler-policy changes, warning-threshold changes, or live-runtime coordinator redesign
- broader failure-taxonomy expansion beyond the four initial session archetypes
- learned classifiers, prompt-based labeling, or external model inference
- multi-trajectory comparison, rollout-tail adjudication redesign, or replay-time interventions
- sentinel interpretation consolidation beyond the narrow compatibility/presentation work needed to
  expose `v0.5` archetype state

## Checkpoint Contract

`R4` should add one new analyzer-owned field to `Checkpoint`:

```text
session_archetype: SessionArchetype
```

`SessionArchetype` should carry:

- `label: SessionArchetypeLabel`
- `confidence: Confidence`
- `supporting_evidence: Vec<EvidenceRef>`
- `counter_evidence: Vec<EvidenceRef>`

Interpretation rules:

- the label is the analyzer's best current session-type classification for the checkpoint
- confidence reflects how strongly the observable evidence supports that label
- supporting/counter evidence keeps the classifier auditable and reusable by `R5` and `R6`
- ambiguous cases should degrade confidence instead of inventing a new archetype

State-scope rule:

- `session_archetype` is session-prefix state evaluated at a checkpoint
- it may use current-turn evidence, but it should not collapse to turn archetype alone
- later checkpoints may legitimately carry a different session type if the session mode shifts

`R4` should treat `session_archetype` as required for `v0.5` checkpoints and absent for older
schemas.

## Classification Rules

`R4` should classify from observable signals, not from commentary tone alone.

Preferred evidence sources:

- kickoff prompt shape, explicit skills mentioned, and orchestration-prompt markers
- kickoff objective shape and expected-next-step language already extracted into analyzer state
- truth-artifact density and working-set concentration
- turn-context execution mode, activity mix, checkpoint density, and prompt cadence
- write/test cadence from command-family and verification observations
- diagnostics such as task-frame transitions and verification density

Conservative first-pass guidance:

- `troubleshooting`
  - repeated verification or diagnostic commands are expected
  - failure-oriented evidence can be present without automatically implying drift
  - edits often cluster near the currently failing scope
- `planning`
  - directive and synthesis activity dominate
  - verification density is lower
  - working set may still be broad while the objective sharpens
- `autonomous_implementation`
  - write/test cadence is steady inside one stable turn or task frame
  - turn context tends toward `autonomous` or `mixed`
  - truth artifacts and working set stay concentrated on the active implementation scope
- `verification_closeout`
  - verification and proof gathering dominate over exploratory edits
  - scope narrows rather than expands
  - expected-next-step language points toward validation, review response, or completion proof

Conservative bias:

- use multiple signals before assigning high confidence
- bias ambiguous cases to `low` or `medium` confidence rather than overclaiming
- do not let one keyword in a prompt override contradictory turn-context or command evidence

Kickoff-prompt rule:

- explicit kickoff cues such as large orchestration prompts or named skill invocations can seed an
  early low-confidence prior before enough behavioral evidence accumulates
- once command, file, and verification evidence accumulates, observed behavior should dominate the
  classification

## Interpretation Of External Patterns

### AgentLens

Adopt:

- make work mode explicit instead of hiding it inside one scorer
- use trajectory structure and tool behavior as context for interpretation

Reject for `R4`:

- per-action intent streams as the canonical artifact
- lucky/solid/ideal task-quality tiers
- multi-trajectory PTA references

`R4` interpretation:

- use the paper as support for a lower deterministic intent-evidence layer
- do not reuse AgentLens's four phase labels as the final `R4` session-archetype output
- use trajectory-history-aware evidence as input to checkpoint-level session-type judgment

### Failure-Taxonomy Literature

Adopt:

- separate session-mode classification from drift scoring
- keep the taxonomy modular so later packets can compose archetype with progress and failure state

Reject for `R4`:

- a wide learned ontology of failure modes
- broad phase labels that replace the current drift classes in one packet

`R4` interpretation:

- session archetype is a small explicit state seam, not a replacement for the current drift-score
  contract
