# Agent Drift Analyzer Current Checkpoint Logic

This document explains how the current `agent-drift-analyzer` checkpoint pipeline works today,
including where the analyzer is interval-aware, where it is still cumulative, and how its output
feeds the sentinel.

It is a code-reading aid for the current implementation, not a target-state design doc.

## Code Map

- `crates/agent-drift-analyzer/src/lib.rs`
- `crates/agent-drift-analyzer/src/context/mod.rs`
- `crates/agent-drift-analyzer/src/inference/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/mod.rs`
- `crates/agent-drift-analyzer/src/checkpoint/schema.rs`
- `crates/agent-drift-analyzer/src/checkpoint/export.rs`
- `crates/agent-drift-analyzer/src/scoring/wrong_plan_branch.rs`
- `crates/agent-drift-analyzer/src/scoring/ignoring_repo_truth.rs`
- `crates/agent-drift-analyzer/src/scoring/dead_end_thrash.rs`
- `crates/agent-drift-sentinel/src/operator_surface.rs`

## Resolution 1: Stack-Level Data Flow

```mermaid
flowchart TD
    A["Compactor bundle input"] --> B["Analyzer loads InputBundle"]
    B --> C["For each BundleSession"]
    C --> D["Assemble full-session ContextPack"]
    D --> E["Infer full-session TaskFrame"]
    C --> F["Build checkpoint windows"]
    F --> G["For each checkpoint window"]
    G --> H["Assemble window ContextPack"]
    H --> I["Infer window TaskFrame"]
    I --> J["Score drift classes"]
    J --> K["Build Checkpoint"]
    K --> L["Export checkpoints.jsonl"]
    K --> M["Aggregate summary.md metrics"]
    L --> N["Sentinel replay or live ingestion"]
    M --> N
```

## Resolution 2: Per-Session Analyzer Flow

```mermaid
flowchart TD
    A["BundleSession"] --> B["assemble_context(session)"]
    B --> C["ContextPack"]
    C --> D["infer_task_frame(context)"]
    D --> E["Session-level TaskFrame"]
    A --> F["checkpoint_windows(session)"]
    F --> G["Window 1"]
    F --> H["Window 2"]
    F --> I["Window N"]
    G --> J["assemble_context(window)"]
    H --> K["assemble_context(window)"]
    I --> L["assemble_context(window)"]
    J --> M["infer_task_frame(window_context)"]
    K --> N["infer_task_frame(window_context)"]
    L --> O["infer_task_frame(window_context)"]
    M --> P["score_session(window, window_context, window_task_frame)"]
    N --> Q["score_session(window, window_context, window_task_frame)"]
    O --> R["score_session(window, window_context, window_task_frame)"]
    P --> S["build_session_checkpoint(window, ordinal, window_task_frame, scores)"]
    Q --> T["build_session_checkpoint(window, ordinal, window_task_frame, scores)"]
    R --> U["build_session_checkpoint(window, ordinal, window_task_frame, scores)"]
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

## Resolution 6: Diagnostics Are Interval-Aware

The checkpoint diagnostics are the one part of the current checkpoint builder that already compares
the current prefix window against the previous checkpoint window.

```mermaid
flowchart TD
    A["build_session_checkpoint(window, ordinal, ...)"] --> B["checkpoint_diagnostics(window, ordinal, ...)"]
    B --> C{"ordinal > 1"}
    C -->|No| D["Use all compact_rows in this window as interval_rows"]
    C -->|Yes| E["previous_checkpoint_window(window, ordinal)"]
    E --> F["Recompute checkpoint_windows on the current prefix"]
    F --> G["Pick prior prefix window"]
    G --> H["previous_context = assemble_context(previous_window)"]
    H --> I["previous_task_frame = infer_task_frame(previous_context)"]
    I --> J["interval_start = previous_window.compact_rows.len()"]
    J --> K["interval_rows = current_window.compact_rows[interval_start..]"]
    D --> L["collect_command_observations(interval_rows)"]
    K --> L
    L --> M["interval_command_count"]
    L --> N["interval_verification_command_count"]
    I --> O["task_frame_transitioned"]
    I --> P["working_set_changed"]
    Q["drift_scores"] --> R["evidence_item_count"]
    M --> S["CheckpointDiagnostics"]
    N --> S
    O --> S
    P --> S
    R --> S
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
    A["score_session(window, context, task_frame)"] --> B["score_wrong_plan_branch"]
    A --> C["score_ignoring_repo_truth"]
    A --> D["score_dead_end_thrash"]
    B --> E["DriftScore wrong_plan_branch"]
    C --> F["DriftScore ignoring_repo_truth"]
    D --> G["DriftScore dead_end_thrash"]
    E --> H["Sort by drift class order"]
    F --> H
    G --> H
    H --> I["Checkpoint.drift_scores"]
```

## Resolution 9: `wrong_plan_branch`

Current behavior: this scorer reads the current window context, but that context was assembled from
the cumulative prefix window.

```mermaid
flowchart TD
    A["window ContextPack"] --> B["Start expected scope from task_frame.truth_artifacts"]
    A --> C["Add context.working_set_paths where source is not observed_command"]
    B --> D["Dedup expected scope paths"]
    C --> D
    A --> E["Iterate context.command_observations"]
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

## Resolution 10: `ignoring_repo_truth`

Current behavior: this scorer also reads the cumulative prefix command stream and asks whether the
session acted before reading likely truth artifacts.

```mermaid
flowchart TD
    A["window ContextPack"] --> B["truth_paths = task_frame.truth_artifacts"]
    A --> C["Find first action index from write_like or verification_like commands"]
    A --> D["Iterate command_observations"]
    D --> E["Does command touch a truth path"]
    E -->|Yes| F["Was it a read before first action"]
    F -->|Yes| G["Collect truth_reads evidence"]
    F -->|No| H["No special evidence added"]
    E -->|No| I["If write_like or verification_like, collect acting_without_truth evidence"]
    B --> J["Compute raw score"]
    G --> J
    I --> J
    J --> K["No truth paths -> 0"]
    J --> L["No truth_reads and actions_without_truth -> 80"]
    J --> M["No truth_reads only -> 60"]
    J --> N["Otherwise -> 20"]
    K --> O["Flagged if raw_score >= 60"]
    L --> O
    M --> O
    N --> O
```

## Resolution 11: `dead_end_thrash`

Current behavior: this scorer intentionally reads the repetition-preserving archival surface, but it
does so from the cumulative archival prefix associated with the current checkpoint.

```mermaid
flowchart TD
    A["window BundleSession"] --> B["collect_command_observations(window.archival_rows)"]
    B --> C["Filter verification_like commands"]
    C --> D["Group evidence by raw_command"]
    A --> E["Iterate archival error and tool-output rows"]
    E --> F["Filter non-empty rows"]
    F --> G["Group evidence by text_hash_hex"]
    D --> H["command_loops = groups with count >= 3"]
    G --> I["failure_loops = groups with count >= 2"]
    H --> J["Raw score contribution: 40 per command loop"]
    I --> K["Raw score contribution: 30 per failure loop"]
    J --> L["raw_score = min 100"]
    K --> L
    L --> M["Flagged if raw_score >= 60"]
    H --> N["Evidence is all loop evidence"]
    I --> N
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

- Checkpoint windows are cumulative prefixes.
- Checkpoint diagnostics compare the current prefix against the previous checkpoint and are
  interval-aware.
- `wrong_plan_branch` and `ignoring_repo_truth` currently score against cumulative command
  observations because their input context is built from the cumulative prefix.
- `dead_end_thrash` currently scores against cumulative archival history because it scans the
  checkpoint's archival prefix.
- `summary.md` is a rollup of emitted checkpoint state, not an independently recomputed notion of
  current recovery.
- The sentinel currently exposes only `Visible` versus `Silent`; it does not have a first-class
  recovered or historical-only presentation state.
