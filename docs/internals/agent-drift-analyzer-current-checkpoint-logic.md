# Agent Drift Analyzer Current Checkpoint Logic

This document explains how the current `agent-drift-analyzer` checkpoint pipeline works today,
including the `v0.4` checkpoint-analysis seam, which parts still depend on cumulative checkpoint
windows, and how analyzer output feeds the sentinel.

It is a code-reading aid for the current implementation, not a target-state design doc.

## Code Map

- `crates/agent-drift-analyzer/src/lib.rs`
- `crates/agent-drift-analyzer/src/context/mod.rs`
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`
- `crates/agent-drift-analyzer/src/scoring/truth_grounding_gap.rs`
- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`

## Resolution 1: Stack-Level Data Flow

```mermaid
flowchart TD
    A["Compactor bundle input"] --> B["Analyzer loads InputBundle"]
    B --> C["For each BundleSession"]
    C --> D["Build checkpoint windows"]
    D --> E["Build CheckpointAnalysis values"]
    E --> F["For each checkpoint analysis"]
    F --> G["Score drift classes"]
    G --> H["Build Checkpoint"]
    H --> I["Export checkpoints.jsonl"]
    H --> J["Aggregate summary.md metrics"]
    I --> K["Sentinel replay or live ingestion"]
    J --> K
```

## Resolution 2: Per-Session Analyzer Flow

```mermaid
flowchart TD
    A["BundleSession"] --> B["checkpoint_windows(session)"]
    B --> C["Window 1"]
    B --> D["Window 2"]
    B --> E["Window N"]
    C --> F["assemble_context(window)"]
    D --> G["assemble_context(window)"]
    E --> H["assemble_context(window)"]
    F --> I["infer_task_frame(window_context)"]
    G --> J["infer_task_frame(window_context)"]
    H --> K["infer_task_frame(window_context)"]
    I --> L["CheckpointSlice"]
    J --> M["CheckpointSlice"]
    K --> N["CheckpointSlice"]
    L --> O["interval_slice + repetition_slice + task_frame_delta + recovery_state"]
    M --> P["interval_slice + repetition_slice + task_frame_delta + recovery_state"]
    N --> Q["interval_slice + repetition_slice + task_frame_delta + recovery_state"]
    O --> R["CheckpointAnalysis 1"]
    P --> S["CheckpointAnalysis 2"]
    Q --> T["CheckpointAnalysis N"]
    R --> U["score_session(analysis, prior_truth_grounding_gap)"]
    S --> V["score_session(analysis, prior_truth_grounding_gap)"]
    T --> W["score_session(analysis, prior_truth_grounding_gap)"]
    U --> X["build_session_checkpoint_from_analysis(...)"]
    V --> Y["build_session_checkpoint_from_analysis(...)"]
    W --> Z["build_session_checkpoint_from_analysis(...)"]
```

## Resolution 3: Bundle And Context Surfaces

This is the object-model view of the analyzer input bundle, per-session bundle, and derived context
object.

```mermaid
flowchart TD
    A["Compactor artifact directory"] --> B["load_bundle(input_dir)"]
    B --> C["InputBundle"]
    C --> D["manifest"]
    C --> E["archival_rows"]
    C --> F["compact_rows"]
    C --> G["dedupe_groups"]
    C --> H["sessions"]
    C --> I["unscoped_archival_rows"]
    C --> J["unscoped_compact_rows"]
    C --> K["surface contract flags"]
    H --> L["BundleSession"]
    L --> M["session_id"]
    L --> N["archival_rows for one session"]
    L --> O["compact_rows for one session"]
    L --> P["assemble_context(session)"]
    P --> Q["ContextPack"]
    Q --> R["objective"]
    Q --> S["truth_artifacts"]
    Q --> T["working_set_paths"]
    Q --> U["tools"]
    Q --> V["command_families"]
    Q --> W["command_observations"]
    Q --> X["supporting_evidence"]
```

## Resolution 4: How Checkpoint Windows Are Built

The key current behavior is that each checkpoint window is a cumulative prefix, not an isolated
interval.

```mermaid
flowchart TD
    A["Full session compact_rows"] --> B["Find checkpoint_end_indices"]
    B --> C["Phase boundaries"]
    B --> D["64-row max chunk boundaries"]
    C --> E["Normalized end indices"]
    D --> E
    E --> F["For each end_index"]
    F --> G["compact_rows = rows[0..=end_index]"]
    G --> H["Find end_row"]
    H --> I["Compute end_key"]
    I --> J["archival_rows = archival rows with row_key <= end_key"]
    J --> K["Emit BundleSession window"]
    K --> L["Window is full prefix through this checkpoint"]
```

## Resolution 5: What Goes Into a Checkpoint

```mermaid
flowchart TD
    A["Window BundleSession"] --> B["checkpoint_boundary"]
    A --> C["checkpoint_diagnostics"]
    A --> D["expected_next_step"]
    A --> E["drift_scores from score_session"]
    A --> F["window TaskFrame"]
    B --> G["CheckpointBoundary start and end RowRef"]
    C --> H["Diagnostics"]
    D --> I["First verification command or fallback text"]
    E --> J["flagged = any score.flagged"]
    F --> K["TaskFrame snapshot for this window"]
    G --> L["Checkpoint"]
    H --> L
    I --> L
    J --> L
    K --> L
```

## Resolution 6: Diagnostics Now Route Through `CheckpointAnalysis`

The exported diagnostics are now computed from the checkpoint-analysis seam. The windowing model is
still cumulative, but diagnostics read explicit interval and recovery fields rather than
recomputing a prior window ad hoc inside the checkpoint builder.

```mermaid
flowchart TD
    A["CheckpointAnalysis"] --> B["analysis.interval.command_observations"]
    A --> C["analysis.recovery.interval_verification_command_count"]
    A --> D["analysis.task_frame_delta.task_frame_transitioned"]
    A --> E["analysis.task_frame_delta.working_set_changed"]
    F["drift_scores"] --> G["evidence_item_count"]
    B --> H["interval_command_count"]
    C --> I["interval_verification_command_count"]
    D --> J["task_frame_transitioned"]
    E --> K["working_set_changed"]
    G --> L["CheckpointDiagnostics"]
    H --> L
    I --> L
    J --> L
    K --> L
```

## Resolution 7: Task-Frame Assembly

```mermaid
flowchart TD
    A["BundleSession rows"] --> B["extract_objective"]
    A --> C["collect_truth_artifacts"]
    A --> D["collect_command_observations"]
    C --> E["collect_working_set_paths"]
    D --> E
    D --> F["collect_tools"]
    D --> G["unique_command_families"]
    B --> H["collect_supporting_evidence"]
    C --> H
    E --> H
    F --> H
    B --> I["ContextPack"]
    C --> I
    D --> I
    E --> I
    F --> I
    G --> I
    H --> I
    I --> J["infer_task_frame"]
    J --> K["objective"]
    J --> L["truth_artifacts"]
    J --> M["working_set_paths"]
    J --> N["tools"]
    J --> O["command_families"]
    J --> P["verification_commands"]
    J --> Q["supporting_evidence"]
    J --> R["counter_evidence"]
    J --> S["confidence"]
```

## Resolution 8: Scoring Orchestration

```mermaid
flowchart TD
    A["score_session(analysis, previous_truth_grounding_gap)"] --> B["score_wrong_plan_branch"]
    A --> C["score_truth_grounding_gap"]
    A --> D["score_dead_end_thrash"]
    B --> E["DriftScore wrong_plan_branch"]
    C --> F["DriftScore truth_grounding_gap"]
    D --> G["DriftScore dead_end_thrash"]
    E --> H["Sort by drift class order"]
    F --> H
    G --> H
    H --> I["Checkpoint.drift_scores"]
```

## Resolution 9: `wrong_plan_branch`

Current behavior: expected scope still comes from the current checkpoint task frame, but the
scorer only inspects command observations from the explicit interval slice.

```mermaid
flowchart TD
    A["analysis.current.task_frame"] --> B["Start expected scope from task_frame.truth_artifacts"]
    A --> C["Add current.context.working_set_paths where source is not observed_command"]
    B --> D["Dedup expected scope paths"]
    C --> D
    A --> E["Iterate analysis.interval.command_observations"]
    E --> F{"command has paths and is write_like or verification_like"}
    F -->|No| G["Skip command"]
    F -->|Yes| H["Check whether all command paths match expected scope"]
    H -->|Yes| I["In-scope"]
    H -->|No| J["Append command evidence to out_of_scope"]
    J --> K["Raw score by out_of_scope count"]
    K --> L["0 -> 0"]
    K --> M["1 -> 60"]
    K --> N["2 -> 80"]
    K --> O["3+ -> 100"]
    L --> P["Flagged if raw_score >= 60"]
    M --> P
    N --> P
    O --> P
```

## Resolution 10: `truth_grounding_gap`

Current behavior: this scorer treats truth-grounding as a hybrid signal. It computes active state
from the latest interval and carries forward prior flagged evidence as explicit historical context.

```mermaid
flowchart TD
    A["CheckpointAnalysis"] --> B["truth_paths = current.task_frame.truth_artifacts"]
    A --> C["Find first action index from interval write_like or verification_like commands"]
    A --> D["Iterate interval.command_observations"]
    D --> E["Does command touch a truth path"]
    E -->|Yes| F["Was it a read before first action"]
    F -->|Yes| G["Collect truth_reads evidence"]
    F -->|No| H["No special evidence added"]
    E -->|No| I["If write_like or verification_like, collect acting_without_truth evidence"]
    A --> J["Import historical evidence from prior TruthGroundingGap score"]
    G --> J
    I --> J
    J --> K["No truth paths -> 0"]
    J --> L["Active gap -> 80 and flagged"]
    J --> M["Historical or grounded current interval -> 20"]
    J --> N["Otherwise -> 0"]
    K --> O["Flagged if raw_score >= 60"]
    L --> O
    M --> P["Not flagged"]
    N --> P
```

## Resolution 11: `dead_end_thrash`

Current behavior: this scorer still reads repetition-preserving archival history from the current
checkpoint prefix, but active flagging is gated by explicit recovery state from the latest
interval. A latest interval only counts as clean recovery when it includes verification-like work,
does not add new repeated-loop evidence, and does not keep issuing out-of-scope write or
verification commands.

```mermaid
flowchart TD
    A["analysis.repetition"] --> B["repeated_verification_loops"]
    A --> C["repeated_failure_loops"]
    A --> D["analysis.recovery.recovered_from_thrash"]
    B --> E["Raw score contribution: 40 per command loop"]
    C --> F["Raw score contribution: 30 per failure loop"]
    E --> G["active_raw_score = min 100"]
    F --> G
    D --> H{"Recovered from thrash?"}
    H -->|No| I["Use active raw score and current loop evidence"]
    H -->|Yes| J["Downgrade to historical score 20 and historical evidence"]
```

## Resolution 12: Exported Summary Metrics

`summary.md` is generated from emitted checkpoints. It aggregates checkpoint-level results rather
than recomputing drift on its own.

```mermaid
flowchart TD
    A["All emitted Checkpoint values"] --> B["summarize_checkpoint_diagnostics"]
    B --> C["checkpoint_count"]
    B --> D["flagged_checkpoint_count"]
    B --> E["drift_class_flagged_counts"]
    B --> F["task_frame_transition_count"]
    B --> G["working_set_change_count"]
    B --> H["total_evidence_item_count"]
    B --> I["total_interval_command_count"]
    B --> J["total_interval_verification_command_count"]
    C --> K["render_summary"]
    D --> K
    E --> K
    F --> K
    G --> K
    H --> K
    I --> K
    J --> K
    K --> L["summary.md"]
```

## Resolution 13: Sentinel Consumption

The sentinel does not rescore drift. It consumes checkpoint output, runs scheduler policy, and then
classifies each checkpoint as either `Visible` or `Silent`.

```mermaid
flowchart TD
    A["Analyzer checkpoints.jsonl"] --> B["Sentinel loads Checkpoint values"]
    B --> C["ReplayScheduler.observe"]
    C --> D["EvaluationDecision"]
    B --> E["present_checkpoint"]
    D --> E
    E --> F{"checkpoint.flagged"}
    F -->|No| G["Silent"]
    F -->|Yes| H{"max flagged score >= minimum_visible_score"}
    H -->|No| I["Silent"]
    H -->|Yes| J{"scheduler allowed visible warning"}
    J -->|No| K["Silent"]
    J -->|Yes| L["Visible"]
```

## Current Semantics To Keep In Mind

- Checkpoint windows are still cumulative prefixes.
- `CheckpointAnalysis` is now the internal seam that carries current, previous, interval,
  repetition, task-frame-delta, and recovery semantics for each checkpoint.
- Checkpoint diagnostics come from that seam's interval and recovery fields.
- `wrong_plan_branch` scores only the latest interval, while expected scope still comes from the
  current checkpoint task frame.
- `truth_grounding_gap` is a hybrid signal: the active flag comes from the latest interval, while
  earlier grounding gaps are preserved as historical evidence.
- `dead_end_thrash` still derives loop history from repetition-preserving archival evidence, but
  the active flag clears after a clean verification interval via explicit recovery semantics.
- `summary.md` is a rollup of emitted checkpoint state, not an independently recomputed notion of
  current recovery.
- The sentinel currently exposes only `Visible` versus `Silent`; it does not have a first-class
  recovered or historical-only presentation state.
