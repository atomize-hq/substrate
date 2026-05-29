# Hybrid Drift Sentinel v0.1

## Problem Statement

How might we build a cross-repo sentinel that watches long Codex turns, infers the agent's current task frame from observable evidence, and flags likely drift without requiring a known active plan?

## Recommended Direction

Build the sentinel around a `task frame`, not an `active plan`.

The sentinel should assume that many repos have no explicit plan artifact, no reliable file naming convention, and no single authoritative task document. Instead of asking whether the agent is following a plan, it should ask whether the current work trajectory still fits the best available task-frame hypothesis given the user objective, observed working set, and candidate truth artifacts.

This makes the system useful in weakly documented repos and safer in strongly documented repos. Plan docs, SOWs, and handoffs still matter when present, but they become evidence rather than prerequisites. That gives the sentinel an upgrade path from observation to warning to steering without baking false certainty into the core model.

## User Value

This is a painkiller if it can do three things reliably:

- surface wrong-branch drift before a human observer would otherwise notice it
- show why the sentinel thinks drift is happening
- build trust through a semantic timeline that matches human judgment

The first customer is the operator watching a long turn, not the main agent. That means the system should optimize for interpretability before autonomy.

## Feasibility

The technical center is feasible in a staged way:

- v0.1 can run as observation-only using a normalized event stream plus periodic `gpt-5.4-mini` checks
- the deterministic parts are straightforward: capture, normalize, batch, score, and emit checkpoints
- the hardest part is not data collection but task-frame inference quality and drift scoring calibration

The key constraint is that the sentinel should not require full repo-specific integration to be useful. That rules out any design that depends on a known plan file or repo-local conventions as mandatory inputs.

## Differentiation

The differentiator is not just “another summary loop.” The stronger version is:

- evidence-driven rather than transcript-dump-driven
- uncertainty-aware rather than fake-authoritative
- semantic rather than purely textual
- staged for steerability once trust is earned

That is meaningfully different from a plain “summarize the latest blocks every N seconds” design.

## Core Model

### Declared Objective

What the user most recently asked for.

Likely sources:

- latest user prompt for the turn
- optionally root prompt for thread-level context

Confidence:

- high

### Observed Working Set

What the main agent is actually doing.

Likely sources:

- files read
- files edited
- tools used
- command summaries
- search targets
- assistant commentary
- assistant final answer

Confidence:

- high

### Candidate Truth Artifacts

Artifacts that may govern the task if they exist.

Likely sources:

- files explicitly mentioned by the user
- files read early in the turn
- docs repeatedly revisited
- issue/spec/handoff/SOW-like artifacts when found

Confidence:

- medium unless explicitly pinned by the user

### Task Frame Hypothesis

The sentinel's best current explanation of what the agent is trying to do right now.

Example forms:

- gathering repo truth before recommending next steps
- implementing a narrow fix in a known file set
- validating hook behavior before changing logic
- finalizing an answer before sufficient grounding

Confidence:

- variable and always explicit

## v0.1 Architecture Spec

### Goals

- observe long turns with low overhead
- infer a task frame from evidence
- flag likely drift with evidence and confidence
- emit operator-readable semantic checkpoints

### Non-Goals

- autonomous steering
- perfect plan reconstruction
- full causal replay
- repo-specific assumptions as the primary mechanism

### Components

#### 1. Event Capture Layer

Captures canonical turn records from the transcript stream.

For v0.1, the capture should keep:

- user prompt
- assistant commentary
- assistant final answer
- tool call starts and completions when available
- command/tool result summaries when available
- file read / file edit hints when available

#### 2. Normalizer

Maps raw transcript records into a stable event schema.

Responsibilities:

- map transport-specific records into logical event types
- strip low-signal noise
- attach timestamps and turn-local ordering
- produce compact summaries for verbose tool outputs

#### 3. Context Assembler

Builds an initial sentinel context pack deterministically at turn start.

Inputs:

- latest user prompt
- referenced file paths
- cwd / branch if available
- early files read by the agent

Outputs:

- objective
- candidate truth files
- initial observed working set
- initial task-frame hypothesis
- watch priorities

This should not require a background agent by default. A cheap model pass can be added after deterministic assembly to compress or sharpen the context pack when useful.

#### 4. Trigger Scheduler

Decides when the sentinel evaluates the stream.

Use mixed triggers, not time-only polling:

- every 20 seconds during active work
- every 3 completed tool calls
- on meaningful work-area shift
- on repeated failure pattern
- on a no-progress timeout

#### 5. Heuristic Scorer

Computes deterministic suspicion scores before model escalation.

Responsibilities:

- detect branch divergence
- detect truth-grounding gaps
- detect dead-end repetition
- decide whether to invoke the model

#### 6. Sentinel Reasoner

Runs `gpt-5.4-mini` on medium when the heuristic layer requests adjudication or when a periodic checkpoint is due.

Inputs:

- sentinel context pack
- sliding event window
- compact running episode summary
- current heuristic scores

Outputs:

- current task frame
- confidence
- semantic phase
- drift status
- evidence list
- expected next step

#### 7. Timeline Store

Persists semantic checkpoints and transitions.

For v0.1, it can be append-only JSONL with one record per checkpoint.

#### 8. Operator Surface

Displays the latest checkpoint and warnings to the human observer.

In v0.1 this can be a simple local log or sidecar file; no interrupt behavior is required.

## Sentinel Context Pack

```json
{
  "objective": "string",
  "success_hint": "string | null",
  "workspace_root": "string | null",
  "candidate_truth_files": [
    {
      "path": "string",
      "reason": "user_mentioned | early_read | repeatedly_read | inferred_governing_artifact",
      "confidence": "high | medium | low"
    }
  ],
  "observed_working_set": {
    "files_read": ["string"],
    "files_edited": ["string"],
    "tools_used": ["string"],
    "commands": ["string"]
  },
  "task_frame_hypothesis": "string",
  "task_frame_confidence": "high | medium | low",
  "drift_watch_priorities": [
    "wrong_plan_branch",
    "ignoring_repo_truth",
    "dead_end_thrash"
  ]
}
```

## Exact Normalized Event Schema

The normalized event stream should be cross-repo and transport-agnostic.

### Shared Envelope

```json
{
  "event_id": "string",
  "turn_id": "string",
  "timestamp": "RFC3339 string",
  "seq": 0,
  "type": "string",
  "payload": {}
}
```

### Event Types

#### `user_prompt`

```json
{
  "text": "string",
  "source_files": ["string"]
}
```

#### `assistant_commentary`

```json
{
  "text": "string"
}
```

#### `assistant_final`

```json
{
  "text": "string"
}
```

#### `tool_call_started`

```json
{
  "tool_name": "string",
  "summary": "string | null"
}
```

#### `tool_call_completed`

```json
{
  "tool_name": "string",
  "success": true,
  "summary": "string",
  "artifacts": ["string"]
}
```

#### `command_summary`

```json
{
  "command_family": "read | search | test | build | edit | git | other",
  "summary": "string",
  "exit_code": 0
}
```

#### `file_read`

```json
{
  "path": "string",
  "kind": "doc | code | config | other"
}
```

#### `file_edit`

```json
{
  "path": "string",
  "kind": "doc | code | config | other"
}
```

#### `search_summary`

```json
{
  "query": "string",
  "scope": "repo | docs | web | other",
  "summary": "string"
}
```

#### `checkpoint`

```json
{
  "task_frame": "string",
  "task_frame_confidence": "high | medium | low",
  "phase": "orientation | truth_gathering | implementation | validation | stuck | branch_shift | recovery | finalization",
  "risk_level": "none | low | medium | high",
  "drift_flags": ["string"],
  "expected_next_step": "string",
  "evidence": ["string"]
}
```

### Normalization Rules

- `response_item.message` with `role = user` becomes `user_prompt`
- `response_item.message` with `role = assistant` and `phase = commentary` becomes `assistant_commentary`
- `response_item.message` with `role = assistant` and `phase = final_answer` becomes `assistant_final`
- raw tool records should be compressed into started/completed summaries rather than copied verbatim
- file events should be emitted only when the path is known
- repeated identical commentary lines should not produce duplicate normalized events

## Task-Frame Inference Model

The sentinel should infer a task frame from evidence, not from a required plan.

### Inputs to Inference

- latest user prompt
- early commentary
- current working set
- candidate truth files
- recent tool trajectory

### Output

```json
{
  "task_frame": "string",
  "confidence": "high | medium | low",
  "supporting_evidence": ["string"],
  "counter_evidence": ["string"]
}
```

### Guidance

- prefer literal objective language from the user prompt over freeform assistant phrasing
- treat repo docs as evidence, not authority, unless explicitly pinned
- lower confidence when multiple plausible frames fit the evidence
- explicitly record branch shifts when the best frame changes

## Drift Scoring Model

The three v0.1 drift classes should use deterministic scoring first, then model adjudication when thresholds are crossed or ambiguity is high.

### Score Shape

Each drift class gets:

- `raw_score` from 0-100
- `confidence` from low/medium/high
- `evidence` list
- optional `model_adjudicated` boolean

### 1. Wrong Plan Branch

Definition:

The current work trajectory no longer matches the most plausible task frame.

Signals:

- files/tools move into a different subsystem without grounding
- commentary intent shifts away from the declared objective
- recent actions support a different subgoal than the user's request
- high-importance files mentioned by the user are not part of the working set while unrelated files dominate activity

Suggested raw scoring:

- +20 for an unexplained work-area shift
- +20 for repeated activity in an unrelated subsystem
- +15 for commentary that states a new goal not grounded in the prompt or truth artifacts
- +10 for ignoring user-mentioned files while touching unrelated areas
- +15 if the task-frame hypothesis changes twice within a short window

Escalate to model when:

- raw score >= 40
- or task-frame confidence drops to low during active work

### 2. Ignoring Repo Truth

Definition:

The agent is making decisions or edits without sufficient grounding in available evidence.

Signals:

- makes claims before reading likely governing artifacts
- edits implementation before consulting obvious truth anchors
- stops consulting truth artifacts after early ambiguity
- final answer confidence rises while grounding evidence stays thin

Suggested raw scoring:

- +20 when a user-mentioned file has not been read before related conclusions are made
- +15 when edits begin without any relevant file-read events in the same work area
- +15 when commentary includes strong assertions without supporting evidence in the recent window
- +10 when the working set excludes candidate truth files despite repeated ambiguity
- +10 when the final answer arrives without a validation or truth-gathering phase

Escalate to model when:

- raw score >= 35
- or there is a finalization phase with thin grounding

### 3. Dead-End Thrash

Definition:

The agent is stuck in a low-yield loop without reframing.

Signals:

- repeated command families with no state change
- repeated searches/reads with no narrowing
- multiple retries against the same obstacle without a new approach
- commentary keeps restating the same blocker without new evidence

Suggested raw scoring:

- +15 after 3 repeated command-family events with no meaningful change
- +15 after repeated searches for similar queries in the same scope
- +20 when the same tool/action fails repeatedly without strategy shift
- +10 when commentary restates the same obstacle twice without new evidence
- +10 when elapsed active time passes a threshold with no file edits, no narrowed scope, and no task-frame refinement

Escalate to model when:

- raw score >= 40
- or the same blocker persists across two scheduled checkpoints

## Checkpoint Output Schema

Every sentinel evaluation should produce one semantic checkpoint.

```json
{
  "timestamp": "RFC3339 string",
  "turn_id": "string",
  "task_frame": "string",
  "task_frame_confidence": "high | medium | low",
  "phase": "orientation | truth_gathering | implementation | validation | stuck | branch_shift | recovery | finalization",
  "risk_level": "none | low | medium | high",
  "drift": {
    "wrong_plan_branch": {
      "score": 0,
      "confidence": "low | medium | high",
      "flagged": false
    },
    "ignoring_repo_truth": {
      "score": 0,
      "confidence": "low | medium | high",
      "flagged": false
    },
    "dead_end_thrash": {
      "score": 0,
      "confidence": "low | medium | high",
      "flagged": false
    }
  },
  "evidence": ["string"],
  "expected_next_step": "string",
  "operator_message": "string"
}
```

## MVP Scope

Build:

- capture and normalize the canonical turn stream
- deterministic context assembler
- mixed-trigger scheduler
- heuristic drift scoring for the three classes
- `gpt-5.4-mini` adjudication path
- semantic checkpoints persisted to disk

Do not build yet:

- auto-steering
- hard dependency on plan docs
- full replay-grade event graphs
- agent-to-agent intervention loops
- broad anomaly classes beyond the three prioritized drifts

## Not Doing

- Requiring an active plan artifact
- Treating inferred docs as authoritative by default
- Using raw reasoning blocks as the primary substrate
- Silently steering the main agent in early versions
- Optimizing for all repos by hard-coding conventions from this repo

## Key Assumptions to Validate

- [ ] A task frame can be inferred reliably enough from prompt + working set + commentary
- [ ] The normalized event stream is sufficient without full reasoning capture
- [ ] Wrong-branch drift can be surfaced early enough to be operationally useful
- [ ] Operators will trust evidence-backed checkpoints before they trust warnings
- [ ] Cross-repo utility holds even when no explicit plan docs exist

## Open Questions

- What is the minimum event set needed before task-frame inference quality becomes stable?
- Should the model be called only on threshold crossings, or also on a low-frequency heartbeat?
- What is the right threshold for visible warnings versus silent checkpoints?
- How should candidate truth files decay in importance when the working set evolves?
