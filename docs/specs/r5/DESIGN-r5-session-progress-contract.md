# Design: R5 Session Progress Contract

Status: canonical design authority locked in Packet R5-0 on 2026-06-09.

## Why This Doc Exists

`R4` landed checkpoint-local `session_archetype` under analyzer checkpoint schema `v0.5`. The next
packet must answer a different question: for the current archetype, is the checkpoint showing
progress, stasis, regression, mixed evidence, or insufficient evidence?

The important boundary is that `R5` describes progress. It does not score drift. `R6` will later
consume `session_progress` alongside typed outcome evidence, turn context, and archetype to retune
`dead_end_thrash` and any related scorers.

## Current Repo Reality

The provided crate snapshot has these relevant facts:

1. `crates/agent-drift-analyzer/src/checkpoint/schema.rs` owns the public checkpoint DTOs.
2. `Checkpoint` currently serializes:
   - `schema_version`
   - `session_id`
   - `checkpoint_id`
   - `ordinal`
   - `boundary`
   - optional `turn_context`
   - `diagnostics`
   - `task_frame`
   - optional `session_archetype`
   - `drift_scores`
   - `expected_next_step`
   - `flagged`
3. `schema_requires_session_archetype(...)` currently returns true only for `v0.5`.
4. `build_session_checkpoint_from_analysis_with_ordinal(...)` currently emits `schema_version:
   "v0.5"` and attaches `Some(session_archetype)`.
5. The public `EvidenceRef` is already established as:

```rust
pub struct EvidenceRef {
    pub row: RowRef,
    pub reason: String,
}
```

6. Sentinel replay and live input currently support `v0.2 | v0.3 | v0.4 | v0.5`; both paths enforce
   `turn_context` and `session_archetype` for `v0.5`.
7. Operator rendering already displays compact `Turn context:` and `Archetype:` lines.

The R5 contract must therefore reuse the existing public `EvidenceRef` shape. Do not introduce a
second public evidence-reference type in the first landing. Richer path, symbol, attempt, and
signature anchors should stay internal or debug-only until there is a separate schema reason to make
them public.

## Research-Derived Direction

The R5 research packet points toward one design sentence:

> Progress is not "the agent stayed busy"; progress is an evidence-backed change in the relevant
> frontier for the current archetype.

The concrete design imports are:

1. **DoVer**: treat failure/progress attribution as a hypothesis that must be validated or left
   inconclusive. For R5 this means explicit evidence, counter-evidence, confidence, and
   `insufficient_evidence`; it does not mean live intervention execution.
2. **Lanser-CLI**: canonicalize machine diagnostics and compare deltas instead of relying on raw
   output hashes.
3. **AgentLens**: use trajectory/history-aware intent and archetype context, not command family
   alone.
4. **TRAJEVAL**: represent regression separately from stalling because trajectories can make real
   progress and later destroy it.
5. **Step-level Optimization**: keep a progress/stuck monitor separate from scorer/milestone
   policy.
6. **AgentRx / TrajAudit**: keep outputs auditable and evidence-linked, with noisy trace filtering
   before interpretation.

## Public Checkpoint Contract

R5 should widen analyzer checkpoints from `v0.5` to `v0.6` by adding one optional field to the
shared DTO and making it required only by version-aware validation.

```rust
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct Checkpoint {
    pub schema_version: String,
    pub session_id: String,
    pub checkpoint_id: String,
    pub ordinal: usize,
    pub boundary: CheckpointBoundary,
    #[serde(default)]
    pub turn_context: Option<TurnContext>,
    pub diagnostics: CheckpointDiagnostics,
    pub task_frame: TaskFrame,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_archetype: Option<SessionArchetype>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_progress: Option<SessionProgress>,
    pub drift_scores: Vec<DriftScore>,
    pub expected_next_step: String,
    pub flagged: bool,
}
```

The shared DTO must deserialize legacy artifacts first, then apply schema-specific requiredness
without drifting from the repo's current validation split.

```rust
fn schema_requires_session_archetype(schema_version: &str) -> bool {
    matches!(schema_version, "v0.5" | "v0.6")
}

fn schema_requires_session_progress(schema_version: &str) -> bool {
    schema_version == "v0.6"
}
```

`turn_context` already has explicit `v0.4+` contract checks in sentinel replay/live loaders. R5 can
keep that split or centralize it later, but the docs and implementation must stay consistent about
where `turn_context` requiredness is enforced.

## Public DTOs

Recommended first public surface:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SessionProgress {
    pub status: ProgressStatus,
    pub dimension: ProgressDimension,
    pub confidence: Confidence,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<ProgressSignal>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supporting_evidence: Vec<EvidenceRef>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub counter_evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    Advancing,
    Mixed,
    Stalled,
    Regressing,
    InsufficientEvidence,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressDimension {
    TroubleshootingFrontier,
    PlanningConvergence,
    ImplementationVerificationWall,
    VerificationCloseoutNarrowing,
    ParentVisibleOrchestration,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProgressSignal {
    pub code: ProgressSignalCode,
    pub polarity: SignalPolarity,
    pub strength: SignalStrength,
    pub summary: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub after: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evidence: Vec<EvidenceRef>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalPolarity {
    Positive,
    Negative,
    Mixed,
    Limiting,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SignalStrength {
    Weak,
    Moderate,
    Strong,
}
```

Initial `ProgressSignalCode` set:

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum ProgressSignalCode {
    FailureFrontierAdvanced,
    FailureSignatureRepeated,
    FailureCountReduced,
    FailureCountIncreased,
    FailingScopeEdited,
    FailingScopeUnchanged,
    VerificationClean,
    VerificationScopeBroadened,
    VerificationScopeNarrowed,
    PlanArtifactCreated,
    PlanArtifactRefined,
    CandidateSetNarrowed,
    CandidateSetExpanded,
    WorkingSetConcentrated,
    WorkingSetDiffused,
    ResidualScopeShrank,
    ResidualScopeReopened,
    PreviouslyCleanScopeBroken,
    DelegationVisibilityLimited,
    TargetNotExercised,
}
```

Keep this list intentionally low-cardinality. R6 needs stable typed inputs, not arbitrary prose.
The `summary` field provides the human-readable explanation.

## Version Compatibility Contract

R5 must preserve the compatibility discipline already used by R4:

| Schema | Required fields beyond base checkpoint |
|---|---|
| `v0.2` | none of `turn_context`, `session_archetype`, `session_progress` |
| `v0.3` | analyzer-owned `DriftScore.state` still expected by sentinel contract checks |
| `v0.4` | `turn_context` |
| `v0.5` | `turn_context`, `session_archetype` |
| `v0.6` | `turn_context`, `session_archetype`, `session_progress` |

Sentinel replay/live supported schema arrays must become:

```rust
const SUPPORTED_ANALYZER_CHECKPOINT_SCHEMAS: &[&str] =
    &["v0.2", "v0.3", "v0.4", "v0.5", "v0.6"];
```

Both replay and live validation should require `checkpoint.session_progress` for `v0.6`, while
legacy fixtures remain loadable.

## Relationship To Existing Analyzer Fields

R5 should consume, but not redefine, existing analyzer context:

1. `turn_context` provides prompt cadence, rows since turn start, checkpoint density, execution
   mode, and activity mix.
2. `session_archetype` provides the current mode; R5 chooses the corresponding progress dimension.
3. `task_frame` provides objective, truth artifacts, working set, command families, and verification
   commands.
4. `diagnostics` provides command counts and interval verification counts.
5. `repetition` and `recovery` slices already expose repeated verification/failure and clean
   verification interval signals.
6. The internal `DelegationContext` from R3.75 must cap progress claims when child work is opaque.

R5 should not change `classify_outcome_evidence(...)`, `tool_output_is_unambiguous_failure(...)`,
or drift-score semantics. Diagnostic parsing for progress can be richer than R1 outcome evidence,
but scorer behavior remains unchanged until R6.

## Evidence Rules

Public `SessionProgress` should carry both signal-local evidence and summary evidence.

Rules:

1. Every non-`insufficient_evidence` status should include at least one `supporting_evidence` item.
2. `stalled` and `regressing` should also include counter-evidence when available.
3. Opaque delegated-parent cases should include `DelegationVisibilityLimited` and a counter-evidence
   item.
4. Evidence lists should be deduped using the existing `(source_file, event_index, row_ordinal,
   reason)` style.
5. Public evidence reasons should describe the observation, not causal certainty. Prefer:
   - "same diagnostic signature repeated with no overlapping failing-scope edit"
   - "compile failure advanced to focused assertion failure"
   - "child-opaque delegation limited progress certainty"
   Avoid:
   - "this edit fixed the bug"
   - "the child agent failed"

## Confidence Rules

```text
High:
  at least two direct machine-observed signals,
  no major contradiction,
  no opaque delegation boundary.

Medium:
  one direct machine-observed signal plus contextual support,
  or multiple contextual signals with limited contradiction.

Low:
  contextual signals only,
  fuzzy diagnostic matching,
  sparse evidence,
  or partial delegation visibility.

InsufficientEvidence:
  no comparable attempt,
  verifier target not exercised,
  or opaque delegation prevents honest archetype-native progress claim.
```

## Operator Rendering

R5 should add one compact line after the existing `Archetype:` line:

```text
- Progress: status=advancing dimension=troubleshooting_frontier confidence=high support[compile failure advanced to focused test failure; edits overlapped failing scope] counter[none]
```

Keep detailed attempt timelines in analyzer summary/debug artifacts, not the live warning block.

## Non-Goals

R5 does not:

1. retune `dead_end_thrash` or any scorer,
2. change scheduler/adjudication policy,
3. infer child implementation progress when delegation is opaque,
4. add LLM/learned progress classification,
5. require language-server integration,
6. require upstream compactor schema changes,
7. publish diagnostic signatures in the public checkpoint schema unless a later packet explicitly
   decides to do so.

## Locked Decisions After Packet R5-0

1. The first public `ProgressSignalCode` set is the full initial enum listed in this contract,
   including `VerificationScopeBroadened` and `VerificationScopeNarrowed`; later packets may leave
   individual codes temporarily unused, but should not narrow or rename the public enum mid-family.
2. Any `progress_debug.jsonl` artifact remains optional and debug-only throughout R5. It is not
   part of the public checkpoint contract and is never required for packet or family review-clean
   status.
3. The first semantic acceptance wall uses a dedicated R5 progress corpus
   (`progress_acceptance.rs` plus `tests/fixtures/progress_acceptance/**`) rather than widening the
   frozen R2 acceptance wall by default.
