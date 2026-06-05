Status: synthesis
Scope: effort+exec
Authority: non-canonical
Artifact-boundary impact: possible

# Effort And Exec Architecture Synthesis

This note consolidates the strongest architecture-relevant patterns from:

- [planner-safe-parallel-lanes.md](../../raw/2026-06-04/planner-safe-parallel-lanes.md)
- [artifact-shapes-ownership-freeze.md](../../raw/2026-06-04/artifact-shapes-ownership-freeze.md)
- [merge-conflict-prediction.md](../../raw/2026-06-04/merge-conflict-prediction.md)
- [deterministic-planning-uncertainty.md](../../raw/2026-06-04/deterministic-planning-uncertainty.md)
- [validation-model-lane-plan-soundness.md](../../raw/2026-06-04/validation-model-lane-plan-soundness.md)
- [hf-papers-modern-reference-addendum.md](./hf-papers-modern-reference-addendum.md)

It is not a canonical architecture doc.
Its purpose is to sharpen the next design and implementation decisions for `crates/effort` and `crates/exec`.

---

## 1. Confidence tiers

### High-confidence and directly relevant

These patterns map cleanly onto the current code-intelligence architecture:

1. Parallel planning should distinguish textual, structural, and semantic conflict classes.
2. Safe concurrency requires an explicit dependency graph plus an explicit conflict graph.
3. Planning should be conservative under uncertainty and incomplete signals.
4. Ownership, forbidden surfaces, and frozen surfaces must be first-class planning outputs.
5. Validation should be layered: static plan validation first, runtime evidence second, merged-tree truth last.
6. Blocked states should be explicit artifacts, not inferred only from logs or operator memory.
7. Runtime should shift from optimistic parallelism to fail-closed handling once real conflict or instability is observed.

### Medium-confidence and useful with adaptation

These patterns are useful, but should be imported as analogies rather than copied literally:

1. Solver-backed scheduling is promising for later conflict-minimal planning, but not required for MVP.
2. Continuous speculative merge/build/test verification is a strong validation layer, but likely belongs after static planner MVP.
3. Visual ownership/freeze metaphors are useful for operator UX and reports, not as the primary contract model.

### Low-confidence or indirect

These sources contributed useful framing but are not direct product templates:

1. The artifact-shapes note is mostly visualization literature; it validates the need for ownership/freeze surfaces more than it validates exact schema shape.
2. The lane-plan soundness note drifted into database query-plan validation; its best use here is the compile-time proof plus runtime revalidation analogy, not its concrete mechanisms.

---

## 2. Core synthesis

The strongest combined lesson is:

> `effort` should produce a conservative, deterministic concurrency plan from explicit dependency, conflict, ownership, and confidence signals, and `exec` should treat that plan as immutable runtime intent while collecting evidence, enforcing freezes, and collapsing risky optimism when the world disagrees.

This lines up well with the current architecture split:

- `effort` defines the graph and partitions lanes.
- `exec` materializes worktrees, records runtime truth, and closes or blocks gates based on evidence.

The research strengthens that split in four ways:

1. `effort` should not just emit a graph; it should emit a graph plus a risk model.
2. `effort` should not infer safe parallelism from touched files alone; it needs conflict classes and confidence-aware rules.
3. `exec` should not just materialize branches; it should preserve blocked-condition evidence and degrade gracefully from optimistic to constrained execution.
4. Final acceptance should remain merged-tree truth even if all lane-local checks look green.

---

## 3. What this implies for `effort`

### 3.1 Planner inputs need stronger risk structure

`LiftEffortSignalV1` should carry enough information for conservative planning, not just generic score data.

Recommended minimum content:

- touched files, symbols, components, and public boundaries
- confidence per touched surface or finding family
- conflict-risk flags
- missing-input flags
- candidate frozen surfaces
- candidate ownership hints
- test topology coverage hints
- evidence refs backing each major claim

The key rule is:

> low-confidence or missing signals should reduce parallelism, not merely lower a score.

### 3.2 The planner needs two graphs, not one

The workstream doc already points toward dependency and conflict classes.
The research says this should be made explicit in implementation early.

Recommended planner internals:

- dependency graph:
  - `must_precede`
  - `producer_before_consumer`
  - `contract_before_implementation`
  - `implementation_before_tests`
  - `freeze_before_parallel_window`
- conflict graph:
  - `hard_same_file`
  - `hard_generated_or_lockfile`
  - `hard_migration`
  - `hard_frozen_surface`
  - `medium_same_contract_surface`
  - `medium_same_schema_or_config`
  - `medium_public_api_producer_consumer`
  - `soft_same_component`
  - `soft_same_boundary`

This should not be treated as bookkeeping.
It is the core reasoning surface for deterministic lane partitioning.

### 3.3 Lane generation should be staged

The research supports a staged planner rather than one monolithic heuristic.

Recommended MVP sequence inside `effort`:

1. Normalize task brief, context packet, and Lift signals into candidate surfaces.
2. Build work nodes.
3. Build hard dependency edges.
4. Build hard conflict edges.
5. Classify medium and soft conflicts.
6. Mark low-confidence and missing-signal regions as non-parallelizable by default.
7. Partition only the remaining safe regions into lanes.
8. Emit ownership, forbidden-surface, and freeze metadata inside lane outputs.

This gives you a deterministic planner that is conservative first and smarter later.

### 3.4 Ownership and freeze should become real contract content

The visualization-heavy research is still useful here because it reinforces one important point:

> ownership, forbidden surfaces, and freeze boundaries are not optional annotations; they are coordination primitives.

For MVP, they do not need standalone top-level artifacts yet.
They can live inside `LanePlanV1`, `HandoffPacketV1`, and `WorktreeIntentV1`.

Recommended fields or substructures:

- `owned_surfaces`
- `forbidden_surfaces`
- `frozen_surfaces`
- `shared_read_context`
- `required_predecessors`
- `blocked_if_touched`

That is likely enough for the first planner.
If the same shapes keep recurring across multiple orchestration patterns, then promote `OwnershipMapV1` or `FreezeMapV1` later.

### 3.5 Conflict prediction should be multi-tier

The merge-conflict research suggests a layered approach:

1. cheap prefilter:
   - same file
   - same method or symbol
   - same generated file or lockfile
   - same migration surface
2. stronger structural checks:
   - producer-consumer boundary overlap
   - shared contract surface
   - shared schema/config surface
3. later expensive checks:
   - speculative merge
   - build/test validation
   - semantic interference checks

This matches the likely product path:

- MVP planner uses deterministic static signals.
- Later `exec` or validation layers can add speculative verification.

---

## 4. What this implies for `exec`

### 4.1 `exec` should enforce plan immutability

The planner research and the current architecture both point to the same rule:

> `exec` may refuse, block, or serialize, but it should not silently rewrite the work graph.

If runtime evidence invalidates the plan, the correct result is:

- blocked condition
- explicit evidence
- checkpointed state
- optional resume or replan request

Not hidden mutation of the plan artifact.

### 4.2 Runtime needs explicit blocked-condition artifacts

This is one of the highest-signal patterns across the orchestration research.

Recommended blocked-condition families:

- frozen-surface violation
- source-lock mismatch
- worktree materialization failure
- unmet predecessor gate
- overlapping ownership detected at runtime
- speculative validation failure
- merged-tree validation wall failure

Each blocked result should preserve:

- triggering claim or invariant
- triggering evidence
- affected lanes
- allowed next actions
- whether replan is required

### 4.3 Runtime should fail closed after real instability

The deterministic-planning research supports a two-phase runtime stance:

1. optimistic mode while the plan remains consistent with evidence
2. fail-closed mode after concrete instability is observed

For this system, that likely means:

- if a lane hits a hard gate failure, speculative parallel progress on related lanes should tighten
- if a frozen or forbidden surface is touched, affected lanes should block immediately
- if merged validation indicates hidden coupling, the runtime should preserve the failure cluster for replan rather than continue broad parallelization

This is the runtime analogue of "first failure versus subsequent failures" in CI research.

### 4.4 Checkpoints should align to risk concentration, not fixed intervals

The existing orchestration note already leans this direction, and the research reinforces it.

Recommended early checkpoint anchors:

- source lock acquisition
- post-materialization
- pre-lane handoff
- pre-merge
- post-merge
- validation wall closeout

That is better than uniform periodic checkpoints because these are the places where irreversible state or coordination risk concentrates.

### 4.5 Speculative validation belongs in `exec`, but later

The strongest merge-conflict literature points toward background speculative merge/build/test checking as the most accurate predictor.

That is valuable, but it is not required to land the first `exec` MVP.

Recommended sequencing:

1. MVP `exec`:
   - source lock enforcement
   - worktree materialization
   - branch/runtime truth
   - checkpointing
   - blocked-condition recording
2. later `exec` validation layer:
   - speculative merge
   - compile/build validation
   - targeted test validation
   - stronger semantic conflict detection

This preserves the current rollout discipline.

---

## 5. Validation model for the program

The best synthesis from the five research captures is a three-layer validation model.

### Layer 1: static plan soundness in `effort`

Before runtime materialization, prove that:

- dependency ordering is valid
- hard conflicts are not parallelized
- each lane has explicit owned and forbidden surfaces
- frozen surfaces are respected
- low-confidence regions were not parallelized silently

This is not formal proof in the theorem-prover sense.
But it is a structural soundness check over deterministic artifacts.

### Layer 2: runtime evidence in `exec`

During materialization and execution, verify that:

- source lock remains valid
- worktrees and branches match intent
- lanes do not violate ownership or freeze constraints
- required gates and predecessors are satisfied
- optional speculative validation does not expose contradiction

### Layer 3: final merged-tree validation wall

At closeout, verify that:

- lane-local success did not hide cross-lane breakage
- merged-tree contracts and gates pass
- blocked conditions are absent or explicitly waived
- final evidence is complete enough to close the run

This is the most important architectural consequence of the research:

> planner-local and lane-local correctness are necessary, but merged-tree correctness is the only acceptance authority.

---

## 6. Recommended near-term design moves

### Move 1

Keep `OwnershipMapV1`, `FreezeMapV1`, and `ValidationWallV1` provisional as top-level artifacts, but embed their semantics now inside lane and handoff outputs.

### Move 2

Make confidence and missing-signal handling explicit in `LiftEffortSignalV1` and `EffortRequestV1`.

### Move 3

Implement `effort` partitioning as:

- hard conflict elimination
- dependency-respecting clustering
- conservative lane extraction

Not as generic balancing or throughput optimization.

### Move 4

Implement `exec` blocked-condition recording and checkpointing before attempting advanced speculative validation.

### Move 5

Treat speculative merge/build/test validation as the most promising later accuracy upgrade for `exec`.

---

## 7. What not to overfit from this research

1. Do not turn the visualization literature into a schema design mandate.
   The durable lesson is "make ownership and freeze explicit," not "use these exact shapes."

2. Do not import database query-plan mechanisms literally.
   The durable lesson is "static structural validation plus runtime revalidation," not "adopt chase proofs or morsel scheduling as-is."

3. Do not over-parallelize because a global score looks good.
   The research consistently favors fail-closed handling of uncertainty, missing evidence, and proven conflict surfaces.

4. Do not let runtime silently compensate for weak planning.
   If the plan is wrong, capture that as evidence and block or replan.

---

## 8. Bottom line

The next implementation-worthy interpretation of this research is:

- `effort` should become a deterministic, confidence-aware concurrency planner with explicit dependency, conflict, ownership, and freeze reasoning.
- `exec` should become an evidence-preserving runtime materializer that enforces source lock, records blocked conditions, and treats merged-tree validation as final truth.

If those two constraints hold, later upgrades such as solver-backed scheduling, speculative merge checking, and richer operator UX can be added without changing the core architecture.
