Status: temporary
Scope: lift+effort research carry-forward
Authority: non-canonical
Artifact-boundary impact: none

# Temporary Lift/Effort Research Ledger

This file is a temporary, repo-root carry-forward document for the current `lift` / `effort` research thread.

It preserves:

- the current repo-local research context
- the five Sci-Bot research answers and their cited papers
- the Hugging Face paper search results that added new ideas
- the archived `system` Work Lift v1 precedent
- the concrete synthesis for a more deterministic next-generation `lift` / `effort` design

It is intended to let a fresh session resume the research without replaying the whole thread.

---

## 1. Current Repo Context

### Canonical authority docs

- [docs/code-intelligence-program.md](./docs/code-intelligence-program.md)
- [docs/code-intelligence-contracts-and-gates.md](./docs/code-intelligence-contracts-and-gates.md)
- [docs/code-intelligence-workstream-orchestration.md](./docs/code-intelligence-workstream-orchestration.md)

### Non-canonical research docs already in this repo

- [docs/code-intelligence-research/README.md](./docs/code-intelligence-research/README.md)
- [docs/code-intelligence-research/overlays/effort-exec/architecture-synthesis.md](./docs/code-intelligence-research/overlays/effort-exec/architecture-synthesis.md)
- [docs/code-intelligence-research/overlays/effort-exec/parallel-planning-and-conflict.md](./docs/code-intelligence-research/overlays/effort-exec/parallel-planning-and-conflict.md)
- [docs/code-intelligence-research/overlays/effort-exec/ownership-freeze-and-boundaries.md](./docs/code-intelligence-research/overlays/effort-exec/ownership-freeze-and-boundaries.md)
- [docs/code-intelligence-research/overlays/effort-exec/runtime-validation-and-fail-closed.md](./docs/code-intelligence-research/overlays/effort-exec/runtime-validation-and-fail-closed.md)
- [docs/code-intelligence-research/overlays/effort-exec/hf-papers-modern-reference-addendum.md](./docs/code-intelligence-research/overlays/effort-exec/hf-papers-modern-reference-addendum.md)
- [docs/code-intelligence-research/crates/effort/implications.md](./docs/code-intelligence-research/crates/effort/implications.md)
- [docs/code-intelligence-research/crates/effort/open-questions.md](./docs/code-intelligence-research/crates/effort/open-questions.md)
- [docs/code-intelligence-research/crates/exec/implications.md](./docs/code-intelligence-research/crates/exec/implications.md)

### Latest handoff loaded in this session

- [.codex/handoffs/2026-06-04-150628-code-intelligence-research-organization.md](./.codex/handoffs/2026-06-04-150628-code-intelligence-research-organization.md)

### Resume point from the handoff

- The active unresolved area is still the `LiftEffortSignalV1` handoff surface.
- The highest-leverage next research target remains: what upstream Lift outputs are required for deterministic `WorkGraphV1` / `LanePlanV1` construction.
- The repo-local discipline remains:
  - top-level docs are canonical
  - `docs/code-intelligence-research/` is non-canonical
  - raw evidence, overlays, and crate notes should stay separate
  - provisional artifact families should not be promoted prematurely

---

## 2. High-Level Design Conclusions Reached In This Session

The main direction is now reasonably clear:

1. `lift` should emit a typed, deterministic repository-intelligence artifact, not a prose-heavy handoff.
2. `effort` should plan from explicit dependency, conflict, ownership, freeze, confidence, and missing-signal structure.
3. AI-authored judgment should be fallback and annotation, not the primary planning substrate.
4. A scalar score can still exist for coarse triage, but it should be secondary to explicit graph structure, confidence, missing inputs, and split/block triggers.
5. Runtime (`exec`) should remain fail-closed and evidence-driven when plan assumptions break.

Short version:

> Deterministic extraction should own the planner substrate whenever possible; AI should propose, summarize, classify, and explain where deterministic extraction cannot fully resolve the shape.

---

## 3. Sci-Bot Research Answers

The user ran five pointed research questions through Sci-Bot and provided the public links. The page content was extracted from the embedded `renderSharedPage(...)` payloads in each public page.

### 3.1 Deterministic software task planning inputs

Source:

- [Sci-Bot: What input fields are required for deterministic software task planning from repository analysis?](https://sci-bot.ru/what-input-fields-are-required-aae0)

Main takeaways:

- Deterministic planning requires multiple input categories, not just source code structure.
- The answer grouped useful inputs into:
  - project sizing fields
  - technical/environment fields
  - team/organization fields
  - issue report fields
  - repository history fields
  - low-level developer activity fields
  - architecture-evolution fields
- The strongest reusable idea for this repo is not the old effort-estimation literature by itself, but the broader point that planning quality depends on structured upstream inputs across repo, task, and architecture layers.

Referenced papers:

- [Selecting best predictors from large software repositories for highly accurate software effort estimation](https://doi.org/10.1002/smr.2271)
- [Estimating Story Points from Issue Reports](https://doi.org/10.1145/2972958.2972959)
- [A Retrospective Study of Software Analytics Projects: In-Depth Interviews with Practitioners](https://doi.org/10.1109/ms.2013.93)
- [Recommendation system to enhance planning of software development using r](https://doi.org/10.1145/2593822.2593831)
- [Mining Software Repositories to Assist Developers and Support Managers](https://doi.org/10.1109/icsm.2006.38)
- [An analysis of developers' tasks using low-level, automatically collected data](https://doi.org/10.1145/1287624.1287715)
- [An approach to software development effort estimation using machine learning](https://doi.org/10.1109/iccp.2017.8117004)
- [Software evolutionary architecture: Automated planning for functional changes](https://doi.org/10.1016/j.scico.2023.102978)
- [Evaluating Pred(p) and standardized accuracy criteria in software development effort estimation](https://doi.org/10.1002/smr.1925)

### 3.2 Representing dependency type, confidence, and blocking risk

Source:

- [Sci-Bot: How should dependency type, confidence, and blocking risk be represented in an agent planning graph?](https://sci-bot.ru/how-should-dependency-type-confidence-b76b)

Main takeaways:

- Dependency type should not be flattened into one generic edge type.
- Different dependency families imply different graph constructs:
  - causal dependencies
  - resource dependencies
  - interaction dependencies
  - precedence dependencies
- Confidence should be attached explicitly to actions/chance nodes, e.g. through ranks, CPT-like structures, or reward/risk values.
- Blocking risk should be represented via conflict patterns, synchronization constraints, or decomposition depth, rather than treated as an informal afterthought.

Most reusable repo-local implication:

- `WorkGraphV1` or its planner internals should distinguish dependency edges and conflict/blocking structure explicitly, and `LiftEffortSignalV1` should carry enough upstream confidence/missing-signal data for that graph to be conservative under uncertainty.

Referenced papers:

- [Representing and planning with interacting actions and privacy](https://doi.org/10.1016/j.artint.2019.103200)
- [Explaining interdependent action delays in multiagent plans execution](https://doi.org/10.1007/s10458-015-9298-0)
- [A Graph-Based Multi-Agent Planning Algorithm with QoS Guarantees](https://doi.org/10.1109/iat.2007.87)
- [Rationale in planning: causality, dependencies, and decisions](https://doi.org/10.1017/s026988899800201x)
- [A Collaborative Multiagent Framework Based on Online Risk-Aware Planning and Decision-Making](https://doi.org/10.1109/ictai.2016.0015)
- [Multiagent Conflict Resolution Planning](https://doi.org/10.1109/smc.2013.57)
- [Conflict estimation of abstract plans for multiagent systems](https://doi.org/10.1145/1329125.1329279)
- [A privacy-preserving model for multi-agent propositional planning](https://doi.org/10.1080/0952813x.2018.1456786)

### 3.3 Inputs that predict safe parallelization vs merge/coordination conflicts

Source:

- [Sci-Bot: What planner inputs best predict safe parallelization versus merge or coordination conflicts?](https://sci-bot.ru/what-planner-inputs-best-predict-a8f7)

Main takeaways:

- Shared-resource footprints are the strongest conflict predictor.
- Explicit concurrency constraints are more meaningful than broad optimism about independent work.
- Cross-plan causal links predict coupling and coordination cost.
- Operator mergeability and conflict patterns matter as structural signals.
- Safe parallelization should be derived from explicit resource/dependency/conflict reasoning, not from shallow “different files touched” heuristics.

Most reusable repo-local implication:

- The planner needs both dependency and conflict structure, and low-confidence shared surfaces should default toward serialization rather than optimistic parallelism.

Referenced papers:

- [Solving Multiagent Planning Problems with Concurrent Conditional Effects](https://doi.org/10.1609/aaai.v33i01.33017594)
- [Theory and algorithms for plan merging](https://doi.org/10.1016/0004-3702(92)90016-q)
- [Conflict solving into the multi-agent distributed planning](https://doi.org/10.1109/icsmc.1998.728083)
- [Predicting possible conflicts in hierarchical planning for multi-agent systems](https://doi.org/10.1145/1082473.1082597)
- [An efficient algorithm for multiagent plan coordination](https://doi.org/10.1145/1082473.1082599)
- [How to solve deadlock situations within the plan-merging paradigm for multi-robot cooperation](https://doi.org/10.1109/iros.1997.656573)
- [Parallel Performance Problems on Shared-Memory Multicore Systems: Taxonomy and Observation](https://doi.org/10.1109/tse.2016.2519346)
- [A satisficing multiagent plan coordinating algorithm for dynamic domains](https://doi.org/10.1145/375735.376032)
- [An extension of the plan-merging paradigm for multi-robot coordination](https://doi.org/10.1109/robot.2001.933066)

### 3.4 Encoding historical execution data without nondeterminism

Source:

- [Sci-Bot: How should historical execution data be encoded so a planner can improve decisions without becoming nondeterministic?](https://sci-bot.ru/how-should-historical-execution-data-ac8b)

Main takeaways:

- Raw history should be turned into deterministic planner artifacts, not stochastic online behavior.
- Strong encoding families include:
  - fragility costs / upgraded cost models
  - deterministic plan/case retrieval
  - learned deterministic control rules
  - macro-operators and nogoods
  - CSP-learned action models
  - deterministic planner-performance feature vectors

Most reusable repo-local implication:

- historical execution evidence should become costs, constraints, features, causal edges, or explicit planner hints
- it should not directly make planning nondeterministic

Referenced papers:

- [Integrating Planning, Execution, and Learning to Improve Plan Execution](https://doi.org/10.1111/j.1467-8640.2012.00447.x)
- [Case-based planning](https://doi.org/10.1017/s0269888906000592)
- [Identifying and Exploiting Features for Effective Plan Retrieval in Case-Based Planning](https://doi.org/10.3233/fi-2016-1447)
- [Knowledge Transfer between Automated Planners](https://doi.org/10.1609/aimag.v32i2.2334)
- [Using Data Mining to Enhance Automated Planning and Scheduling](https://doi.org/10.1109/cidm.2007.368881)
- [Learning temporal action models from multiple plans: A constraint satisfaction approach](https://doi.org/10.1016/j.engappai.2021.104590)
- [Learning from planner performance](https://doi.org/10.1016/j.artint.2008.11.009)
- [Extending BDI plan selection to incorporate learning from experience](https://doi.org/10.1016/j.robot.2010.05.008)

### 3.5 Artifact schema bridging code intelligence into lane plans and handoff packets

Source:

- [Sci-Bot: What artifact schema best bridges code intelligence outputs into lane-based execution plans and handoff packets?](https://sci-bot.ru/what-artifact-schema-best-bridges-b849)

Main takeaways:

- The strongest schema pattern is explicit-external, not implicit/internal.
- A multi-layer representation is better than one flat artifact:
  - code-intelligence representation layer
  - architectural / lane-plan layer
  - handoff / packet / message layer
- Source-based mappings and artifact-centric process models are strong bridging techniques.

Most reusable repo-local implication:

- `LiftEffortSignalV1` should likely sit as a deterministic bridge artifact between lifted repo intelligence and planner/runtime artifacts.
- The system should preserve clear layer boundaries rather than collapsing analysis, plan, and handoff into one ambiguous document shape.

Referenced papers:

- [Where's the schema? A taxonomy of patterns for software exchange](https://doi.org/10.1109/wpc.2002.1021320)
- [Issues in Integrating Schemas for Reverse Engineering](https://doi.org/10.1016/j.entcs.2004.01.002)
- [Analyzing Inaccurate Artifact Usages in a Workflow Schema](https://doi.org/10.1109/compsac.2006.113)
- [An intermediate representation for integrating reverse engineering analyses](https://doi.org/10.1109/wcre.1998.723194)
- [An XML-based framework for language neutral program representation and generic analysis](https://doi.org/10.1109/cmpsac.2004.1342654)
- [Linking Analysis and Transformation Tools with Source-Based Mappings](https://doi.org/10.1109/scam.2006.18)
- [A Programmable Analysis and Transformation Framework for Reverse Engineering](https://doi.org/10.1016/j.entcs.2004.01.005)
- [Dataflow plan execution for software agents](https://doi.org/10.1145/336595.337087)
- [Tool Integration Models](https://doi.org/10.1109/apsec.2013.70)
- [Package-oriented programming of engineering tools](https://doi.org/10.1145/253228.253501)
- [Tools cooperation in an integration environment by message-passing mechanism](https://doi.org/10.1109/cmpsac.1994.342763)
- [Towards an Integration System for Artifact-centric Processes](https://doi.org/10.1145/2926693.2929904)
- [Bridging the Concrete and Logical Domains for Software Architecture Reconstruction](https://doi.org/10.1109/wicsa.2005.18)
- [Integration Workbench: Integrating Schema Integration Tools](https://doi.org/10.1109/icdew.2006.69)
- [Integration and Analysis of Design Artefacts in Embedded Software Development](https://doi.org/10.1109/compsacw.2012.94)

---

## 4. Hugging Face Paper Search Results

These came from `hf papers search` runs targeted at the unresolved `lift` / `effort` questions.

### 4.1 Highest-value modern additions

These were the most useful additions beyond the earlier repo-local note set:

- [RPG: A Repository Planning Graph for Unified and Scalable Codebase Generation](https://arxiv.org/abs/2509.16198)
  - strongest direct inspiration for repository planning graphs
  - reinforces graph-backed plan artifacts over prose

- [Closing the Loop: Universal Repository Representation with RPG-Encoder](https://huggingface.co/papers/2602.02084)
  - useful for the “understanding ↔ generation” closed loop
  - suggests a higher-fidelity repository representation that can serve both localization and planning

- [Code Graph Model (CGM): A Graph-Integrated Large Language Model for Repository-Level Software Engineering Tasks](https://arxiv.org/abs/2505.16901)
  - graph-structured repository reasoning
  - useful for richer Lift outputs without tight coupling to one runtime

- [DependEval: Benchmarking LLMs for Repository Dependency Understanding](https://arxiv.org/abs/2503.06689)
  - dependency understanding is a distinct capability
  - supports making dependency extraction first-class in Lift

- [ARISE: A Repository-level Graph Representation and Toolset for Agentic Fault Localization and Program Repair](https://arxiv.org/abs/2605.03117)
  - multi-granularity repository graph
  - suggests future richer conflict and localization signals

- [GREPO: A Benchmark for Graph Neural Networks on Repository-Level Bug Localization](https://arxiv.org/abs/2602.13921)
  - supports graph-first repo reasoning

- [Beyond Blame: Rethinking SZZ with Knowledge Graph Search](https://arxiv.org/abs/2602.02934)
  - strongest history/causal structure precedent found in this session
  - relevant to history signals in `LiftEffortSignalV1`

- [CodePlan: Repository-level Coding using LLMs and Planning](https://huggingface.co/papers/2309.12499)
  - incremental dependency analysis
  - change may-impact analysis
  - adaptive multi-step planning
  - useful as a bridge between old coarse Work Lift style signals and richer planner inputs

- [How to Understand Whole Software Repository?](https://huggingface.co/papers/2406.01422)
  - repository knowledge graph plus structured exploration
  - useful for shaping lifted repo summaries before planning

- [Assessing Correctness in LLM-Based Code Generation via Uncertainty Estimation](https://arxiv.org/abs/2502.11620)
  - strongest modern support for abstention / uncertainty-aware gating
  - useful for low-confidence handling

- [AgenticFlict: A Large-Scale Dataset of Merge Conflicts in AI Coding Agent Pull Requests on GitHub](https://huggingface.co/papers/2604.03551)
  - merge conflicts are frequent in agentic PRs
  - strong practical precedent for conservative conflict modeling

- [GraphLocator: Graph-guided Causal Reasoning for Issue Localization](https://huggingface.co/papers/2512.22469)
  - good precedent for explicit causal issue graphs
  - useful for one-to-many issue decomposition and downstream work graph shaping

### 4.2 Runtime / orchestration additions

- [ToolGate: Contract-Grounded and Verified Tool Execution for LLMs](https://arxiv.org/abs/2601.04688)
  - strongest runtime-guardrail parallel to contracts-and-gates

- [GraphBit: A Graph-based Agentic Framework for Non-Linear Agent Orchestration](https://arxiv.org/abs/2605.13848)
  - deterministic DAG orchestration
  - runtime-state and auditability language maps well to `exec`

- [On Time, Within Budget: Constraint-Driven Online Resource Allocation for Agentic Workflows](https://arxiv.org/abs/2605.06110)
  - more relevant for later runtime scheduling than early MVP planning

- [From Static Templates to Dynamic Runtime Graphs: A Survey of Workflow Optimization for LLM Agents](https://arxiv.org/abs/2603.22386)
  - useful vocabulary for plan artifacts versus realized runtime graphs

### 4.3 Constraint / cautionary papers

- [R-ConstraintBench: Evaluating LLMs on NP-Complete Scheduling](https://arxiv.org/abs/2508.15204)
  - useful warning that interacting constraints dominate difficulty

- [RepoGraph: Enhancing AI Software Engineering with Repository-level Code Graph](https://huggingface.co/papers/2410.14684)
  - useful adjacent repository graph framing

### 4.4 Strongest session-level HF conclusion

The modern paper set materially strengthens the case that:

- repository-level planning should be graph-backed
- dependency understanding should be first-class
- conflict and causal/history structure should be explicit
- runtime verification and contract-grounded execution should remain separate but linked

---

## 5. Archived Work Lift v1 Precedent

The user asked for practical precedent from the archived repo:

- `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system`

This section preserves what was gathered from that tree.

### 5.1 Where the archived implementation lived

Primary implementation:

- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift.py)

Key supporting files:

- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_model.v1.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_model.v1.json)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_vector.schema.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_vector.schema.json)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_RUBRIC.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_RUBRIC.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_report.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_report.py)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_strict_check.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_strict_check.py)

### 5.2 How the archived score worked

The archived implementation was deterministic once the vector existed.

Input surface:

- `touch.create_files`
- `touch.edit_files`
- `touch.delete_files`
- `touch.deprecate_files`
- `touch.crates_touched`
- `touch.boundary_crossings`
- `contract.cli_flags`
- `contract.config_keys`
- `contract.exit_codes`
- `contract.file_formats`
- `contract.behavior_deltas`
- `qa.new_test_files`
- `qa.new_test_cases`
- `docs.new_docs_files`
- `ops.new_smoke_steps`
- `ops.ci_changes`
- `risk.cross_platform`
- `risk.security_sensitive`
- `risk.concurrency_or_ordering`
- `risk.migration_or_backfill`
- `risk.unknowns_high`

Scoring structure:

- additive weighted base
- multiplied by boolean risk multipliers
- plus unknowns addend
- `lift_score = ceil(score_unrounded)`
- `estimated_slices = max(1, ceil(lift_score / 12))`

Exact v1 weights and multipliers from `work_lift_model.v1.json`:

- touch:
  - `create_files = 3`
  - `edit_files = 2`
  - `delete_files = 1`
  - `deprecate_files = 1`
  - `crates_touched = 4`
  - `boundary_crossings = 3`
- contract:
  - `cli_flags = 3`
  - `config_keys = 3`
  - `exit_codes = 4`
  - `file_formats = 5`
  - `behavior_deltas_blowup = 10 * max(0, behavior_deltas - 1)`
- QA:
  - `new_test_files = 2`
  - `new_test_cases = 1`
- docs:
  - `new_docs_files = 2`
- ops:
  - `new_smoke_steps = 3`
  - `ci_changes = 3`
- risk multipliers:
  - `cross_platform = 1.15`
  - `security_sensitive = 1.20`
  - `concurrency_or_ordering = 1.15`
  - `migration_or_backfill = 1.25`
- unknowns add:
  - `unknowns_high_multiplier = 2`

### 5.3 Signals and data-flow paths in the archived version

Three upstream signal paths were confirmed:

1. Intake / ADR markdown:
   - authored fenced JSON Lift Vector block
2. Planning pack / impact map:
   - touch-set derivation from `validate_impact_map.py --emit-json`
3. Git diff calibration:
   - `git diff --name-status -M`

Important limitation:

- the archived version did not use rich repo history, issue history, execution history, or semantic dependency graphs as primary planner inputs
- it mostly depended on AI-authored or human-authored coarse vector fields, plus shallow deterministic file-touch derivation

### 5.4 Confidence and missing-input posture in the archived version

This is one of the strongest precedents worth preserving.

Rules:

- missing or `null` numeric inputs scored as `0`
- but they were appended to `missing_inputs`
- and any non-empty `missing_inputs` dropped `confidence` to `low`
- prefix-derived touch sets also forced `confidence = low`

This means the archived version already had a good deterministic posture:

- keep scoring deterministic
- make uncertainty explicit
- do not hide low-confidence inputs behind a clean-looking scalar

### 5.5 Triggers in the archived version

Configured v1 triggers:

- `split_required:behavior_deltas>1`
- `likely_split:crates_touched>2`
- `likely_split:touch_files_sum>12`
- `likely_split:contract_surface_sum>4`
- `likely_split:lift_score>24`
- `split_required:estimated_slices>3`

Discovery-flow threshold in prompt guidance:

- `lift_score > 60` was treated as a strong split signal in some user-facing prompt guidance

Implementation quirk:

- the config declared both `adr_candidate` and `workstream` trigger families
- but `pm_lift.py` v1 only evaluated the `adr_candidate` rules in code

### 5.6 Best lessons from the archived version

Keep:

- strict input schema
- deterministic scoring
- explicit missing-input tracking
- explicit confidence degradation
- explicit split triggers
- advisory-first posture

Do not carry forward unchanged:

- heavy reliance on AI-authored coarse counts as primary substrate
- shallow file-touch proxies as the core planner model
- scalar-first interpretation when richer graph structure is possible

---

## 6. Consolidated Synthesis For New Lift/Effort Design

### 6.1 Deterministic by default

The strongest practical rule from this session is:

> deterministic extraction should own as much of the planner substrate as possible

That means the new version should prefer repo-truth-derived facts over AI-authored raw inputs whenever possible.

### 6.2 AI should become fallback and annotation

AI still has value, but in a narrower role:

- semantic clustering
- intent labeling
- behavior-delta summarization
- explanation for humans
- hypothesis generation where repo-truth is incomplete

AI should not be the default source of primary planning counts if deterministic extraction can derive the same class of signal.

### 6.3 Proposed three-layer shape

#### Layer 1: deterministic lifted facts

Examples:

- touched files, symbols, crates, tests, docs, configs
- public contract deltas
- dependency graph edges
- conflict candidates
- source mappings
- causal/history evidence
- confidence and missing-signal flags

#### Layer 2: planner-ready structures

Examples:

- work nodes
- typed dependency edges
- typed conflict edges
- ownership / freeze candidates
- split/block triggers
- lane candidates

#### Layer 3: runtime / handoff structures

Examples:

- lane plans
- handoff packets
- blocked-condition artifacts
- validation gates
- runtime evidence attachments

### 6.4 Dependency and conflict must be separated

This is now one of the clearest architecture decisions from the research:

- a single “risk score” is not enough
- safe parallelism requires:
  - dependency structure
  - conflict structure
  - confidence / missing-signal structure

### 6.5 History should bias deterministic planning, not randomize it

History should become:

- costs
- constraints
- causal edges
- confidence adjustments
- retrieval features
- blocked-state priors

History should not directly make the planner nondeterministic.

### 6.6 Score should become secondary

A scalar score can remain useful for rough triage.

But the new design should prioritize:

- explicit dependency edges
- explicit conflict edges
- missing-input inventory
- per-surface confidence
- split/block triggers
- ownership/freeze/forbidden-surface summaries

In other words:

> score is a summary, not the planner substrate

---

## 7. Practical Mapping: Old Work Lift v1 To New Lift/Effort World

### Keep

- touch counts
- contract surface deltas
- QA/docs/ops counts
- explicit missing-input reporting
- explicit confidence
- trigger-based interpretation

### Upgrade

- replace shallow touch-only reasoning with graph-backed repo reasoning
- replace one scalar conflict intuition with explicit dependency + conflict edges
- replace “unknowns_high” as a generic count with more structured missing-signal / confidence annotations
- enrich history from optional diff calibration into richer causal/history evidence

### Replace

- AI-authored coarse raw counts where deterministic extraction is possible
- scalar-only planning signals
- file-list-only parallelization heuristics

---

## 8. Recommended Next Session Starting Point

The next useful step is not more broad research.

The next useful step is to draft concrete repo-local artifact fields:

1. `LiftEffortSignalV1`
2. `WorkGraphV1`
3. `LanePlanV1`
4. optionally the `HandoffPacketV1` subset that must be planner-owned

Best way to start that draft:

- take archived Work Lift v1 inputs
- classify each field as:
  - keep
  - upgrade
  - replace
- then add the new graph-native fields demanded by the current research:
  - dependency edges
  - conflict edges
  - source mappings
  - causal/history signals
  - per-field/per-surface confidence
  - ownership/freeze candidates

---

## 9. Source Index For Fresh Sessions

### Current repo-local docs

- [docs/code-intelligence-program.md](./docs/code-intelligence-program.md)
- [docs/code-intelligence-contracts-and-gates.md](./docs/code-intelligence-contracts-and-gates.md)
- [docs/code-intelligence-workstream-orchestration.md](./docs/code-intelligence-workstream-orchestration.md)
- [docs/code-intelligence-research/README.md](./docs/code-intelligence-research/README.md)
- [docs/code-intelligence-research/overlays/effort-exec/architecture-synthesis.md](./docs/code-intelligence-research/overlays/effort-exec/architecture-synthesis.md)
- [docs/code-intelligence-research/overlays/effort-exec/hf-papers-modern-reference-addendum.md](./docs/code-intelligence-research/overlays/effort-exec/hf-papers-modern-reference-addendum.md)
- [docs/code-intelligence-research/crates/effort/implications.md](./docs/code-intelligence-research/crates/effort/implications.md)
- [docs/code-intelligence-research/crates/effort/open-questions.md](./docs/code-intelligence-research/crates/effort/open-questions.md)
- [.codex/handoffs/2026-06-04-150628-code-intelligence-research-organization.md](./.codex/handoffs/2026-06-04-150628-code-intelligence-research-organization.md)

### Sci-Bot pages

- [What input fields are required for deterministic software task planning from repository analysis?](https://sci-bot.ru/what-input-fields-are-required-aae0)
- [How should dependency type, confidence, and blocking risk be represented in an agent planning graph?](https://sci-bot.ru/how-should-dependency-type-confidence-b76b)
- [What planner inputs best predict safe parallelization versus merge or coordination conflicts?](https://sci-bot.ru/what-planner-inputs-best-predict-a8f7)
- [How should historical execution data be encoded so a planner can improve decisions without becoming nondeterministic?](https://sci-bot.ru/how-should-historical-execution-data-ac8b)
- [What artifact schema best bridges code intelligence outputs into lane-based execution plans and handoff packets?](https://sci-bot.ru/what-artifact-schema-best-bridges-b849)

### Archived Work Lift v1 implementation

- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift.py)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_model.v1.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_model.v1.json)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_vector.schema.json](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/schemas/work_lift_vector.schema.json)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_RUBRIC.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_RUBRIC.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/standards/shared/WORK_LIFT_MODEL_V1_GOLDENS.md)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_report.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_report.py)
- [/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_strict_check.py](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/archive/project_management/_archived/system/scripts/planning/pm_lift_strict_check.py)

### Strongest modern papers to revisit first

- [RPG](https://arxiv.org/abs/2509.16198)
- [RPG-Encoder](https://huggingface.co/papers/2602.02084)
- [CGM](https://arxiv.org/abs/2505.16901)
- [DependEval](https://arxiv.org/abs/2503.06689)
- [ARISE](https://arxiv.org/abs/2605.03117)
- [GREPO](https://arxiv.org/abs/2602.13921)
- [Beyond Blame](https://arxiv.org/abs/2602.02934)
- [CodePlan](https://huggingface.co/papers/2309.12499)
- [How to Understand Whole Software Repository?](https://huggingface.co/papers/2406.01422)
- [Assessing Correctness in LLM-Based Code Generation via Uncertainty Estimation](https://arxiv.org/abs/2502.11620)
- [AgenticFlict](https://huggingface.co/papers/2604.03551)
- [GraphLocator](https://huggingface.co/papers/2512.22469)
- [ToolGate](https://arxiv.org/abs/2601.04688)
- [GraphBit](https://arxiv.org/abs/2605.13848)
- [On Time, Within Budget](https://arxiv.org/abs/2605.06110)
- [Workflow Optimization Survey](https://arxiv.org/abs/2603.22386)

