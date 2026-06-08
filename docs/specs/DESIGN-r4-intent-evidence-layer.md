# Design: R4 Intent-Evidence Layer

Status: draft design input. This document defines the lower deterministic intent-evidence seam that
should sit underneath `R4` session-archetype classification. It is not the public checkpoint
contract and it is not an implementation-ready packet by itself. It freezes the missing analyzer
mechanism so later `R4-2+` work does not collapse into prompt heuristics or scorer drift.

## Why This Doc Exists

The `R4` spec/plan/tasks set already freezes the public goal:

1. export `session_archetype` on analyzer checkpoint schema `v0.5`,
2. classify checkpoints as `troubleshooting`, `planning`, `autonomous_implementation`, or
   `verification_closeout`,
3. keep the packet additive and deterministic.

What is still not frozen is the lower layer that turns observed session behavior into reusable
intent evidence before final archetype aggregation.

Without that lower seam:

1. `R4` risks becoming prompt-keyword classification,
2. archetype rules will drift into one-off special cases inside the checkpoint builder,
3. later `R5` / `R6` work will have no auditable intermediate state between raw telemetry and the
   final session label.

This design fills that exact gap.

## Relationship To Existing Decisions

This design must compose with:

1. [docs/specs/agent-drift-analyzer-session-archetype-r4-spec.md](./agent-drift-analyzer-session-archetype-r4-spec.md):
   `R4` is checkpoint-scoped session-archetype classification only.
2. [docs/specs/agent-drift-analyzer-session-archetype-r4-plan.md](./agent-drift-analyzer-session-archetype-r4-plan.md):
   `R4-2a` should be the lower intent-evidence seam.
3. [docs/internals/agent-drift-analyzer-current-checkpoint-logic.md](../internals/agent-drift-analyzer-current-checkpoint-logic.md):
   current checkpoint assembly already has the task-frame, command-observation, and diagnostics
   surfaces this layer should read.
4. [docs/specs/agent-drift-analyzer-turn-context-r3-spec.md](./agent-drift-analyzer-turn-context-r3-spec.md):
   `R3` exported turn-local state but deliberately stopped short of session meaning.
5. AgentLens as summarized in
   [.codex/handoffs/2026-06-07-220133-hybrid-drift-r4-session-archetype.md](../../.codex/handoffs/2026-06-07-220133-hybrid-drift-r4-session-archetype.md):
   useful as mechanism inspiration, not as public taxonomy authority.

## Problem Statement

How might the analyzer derive a small, deterministic, auditable intent-evidence profile from the
session prefix at each checkpoint so that:

1. final `session_archetype` classification is based on observed behavior rather than prompt tone,
2. read/write/test behavior can be interpreted with context instead of one-tool-one-meaning rules,
3. the final archetype decision can expose evidence and counter-evidence without embedding all
   logic directly in one top-level classifier?

## Frozen Direction

The direction frozen by this design is:

1. introduce one analyzer-local lower layer between current checkpoint analysis inputs and the
   public `session_archetype` decision,
2. keep that layer deterministic and rule-based,
3. model behavior as a small fixed set of intent-evidence buckets rather than as final public
   archetype labels,
4. let tool or command families provide default hints only,
5. let context-sensitive overrides reinterpret those hints using file role, edit history,
   verification behavior, and recent trajectory shape,
6. keep this layer analyzer-local in the first `R4` landing; `v0.5` must expose
   `session_archetype`, not a new public per-action or per-bucket stream.

## Non-Goals

This design does not:

1. expose AgentLens-style per-action labels as a public checkpoint artifact,
2. replace `task_frame`, `turn_context`, or existing drift scores,
3. define `R5` progress semantics,
4. define sentinel operator copy,
5. justify a learned classifier or prompt-only classifier.

## Conceptual Output

The lower seam should produce one analyzer-local profile conceptually equivalent to:

```text
IntentEvidenceProfile
- exploration_like: EvidenceBucket
- implementation_like: EvidenceBucket
- verification_like: EvidenceBucket
- orchestration_like: EvidenceBucket

EvidenceBucket
- strength: none | weak | moderate | strong
- evidence: Vec<EvidenceRef>
- counter_evidence: Vec<EvidenceRef>
```

Interpretation rules:

1. these are not the public `R4` labels,
2. a checkpoint may legitimately show mixed evidence across multiple buckets,
3. ambiguity should stay visible here rather than being hidden until the final archetype label,
4. the profile is derived from the session prefix up to the current checkpoint, with recent
   interval evidence allowed to move bucket strength materially.

## Evidence Families

The lower layer should read from four evidence families.

### 1. Command and tool defaults

Default hints may come from command family or tool identity, for example:

1. search, listing, or file-inspection commands bias toward `exploration_like`,
2. source-edit commands bias toward `implementation_like`,
3. test, lint, build, replay, diff-check, or doctor commands bias toward `verification_like`,
4. explicit orchestration or bookkeeping tools bias toward `orchestration_like`.

These are defaults, not final meanings.

### 2. File-role and working-set context

The same command may mean different things depending on the touched scope.

Examples:

1. editing a source file after a stable implementation objective strengthens
   `implementation_like`,
2. editing only test fixtures or golden outputs after source edits have tapered can strengthen
   `verification_like`,
3. broad file inspection across many unrelated paths should strengthen `exploration_like` even if
   some verification commands appear.

### 3. Session-history and recency context

The layer should read current interval behavior against recent checkpoint history.

Examples:

1. repeated failing verification around one active scope can coexist with high
   `verification_like` and rising `exploration_like`,
2. sustained write plus local verification cadence on a concentrated working set should bias toward
   `implementation_like`,
3. short orchestration bursts early in the session should not keep dominating after sustained build
   or edit activity begins.

### 4. Task-frame and objective context

The layer may use existing analyzer state such as:

1. objective wording already normalized into task-frame state,
2. truth-artifact density,
3. working-set concentration,
4. turn-context execution mode and activity mix,
5. checkpoint diagnostics such as task-frame transitions and verification density.

This context should reinterpret behavior, not replace it.

## Context-Sensitive Rules

The first rule set should stay intentionally conservative.

### Rule 1: File-inspection commands stay exploration-like by default

Commands such as `rg`, `grep`, `cat`, `sed`, `find`, `ls`, and `git log` should stay
`exploration_like` unless paired with stronger nearby evidence that the session is in a more
specific mode.

This preserves the conservative AgentLens lesson already called out in the handoff.

### Rule 2: Source edits plus local verification imply implementation-like work

When the recent prefix shows:

1. concentrated source-file edits,
2. stable objective language,
3. local verification against the same scope,

the profile should strengthen `implementation_like` even if exploratory inspection still appears.

### Rule 3: Proof-oriented verification can outweigh residual implementation

When recent checkpoints show:

1. verification commands dominating,
2. scope narrowing rather than expanding,
3. little or no new source editing,

the profile should strengthen `verification_like` even if the session implemented code earlier.

### Rule 4: Troubleshooting evidence is not a separate lower bucket

The lower seam should not create a dedicated `troubleshooting_like` bucket.

Instead, troubleshooting should emerge later from a specific mixture:

1. high `verification_like`,
2. meaningful `exploration_like`,
3. failure-oriented or diagnostic repetition on a constrained scope,
4. limited stable implementation evidence.

This keeps the lower seam reusable and narrower than the public `R4` taxonomy.

### Rule 5: Orchestration evidence must stay bounded

Large kickoff prompts, explicit skills, or orchestration markers can strengthen
`orchestration_like`, but they must not overpower contradictory behavior once the session begins
editing, testing, or verifying.

## Recommended Implementation Shape

The cleanest first implementation shape is:

1. add one small analyzer-local module under checkpoint assembly,
2. build the profile from existing task-frame, turn-context, diagnostics, and command-observation
   surfaces,
3. return a single `IntentEvidenceProfile` object to the public archetype aggregator,
4. avoid adding new upstream compactor fields in this packet,
5. keep evidence references aligned with existing analyzer evidence items so later summary and test
   output can reuse them.

## Open Questions

1. Should the analyzer persist the lower profile only as an internal helper, or also expose it in
   debug/test-only output for easier review?
2. Should `orchestration_like` include only explicit skill/orchestration markers, or also resume
   and handoff-heavy administrative turns?
3. Should test-file edits after source edits count as `implementation_like`, `verification_like`,
   or a mixture by default?

