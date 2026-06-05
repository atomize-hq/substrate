Status: synthesis
Scope: effort+exec
Authority: non-canonical
Artifact-boundary impact: none

# HF Papers Modern Reference Addendum

This note records additional reference candidates gathered with `hf papers search` to deepen the current `effort` and `exec` research stack.

The selection policy here is:

1. prioritize newer material when it is directly relevant to repository-level planning, graph reasoning, orchestration, or verified runtime execution
2. keep older literature when it contributes durable concepts that still map cleanly onto the code-intelligence architecture
3. avoid treating novelty alone as authority

This note does not replace the earlier raw captures or synthesis notes.
It is an addendum for modern reference coverage.

---

## 1. Why this matters

The unsettled layer in the current stack is still `effort`:

- how `LiftEffortSignalV1` should carry dependency and risk structure
- how `WorkGraphV1` should encode conflicts, dependencies, and ownership
- how `LanePlanV1` and `HandoffPacketV1` should stay deterministic without collapsing into vague natural-language planning

The newer `hf papers` hits are useful because they add stronger modern coverage for:

- repository planning graphs
- repository dependency understanding
- graph-based repository reasoning
- deterministic workflow/orchestration graphs
- contract-grounded verified execution

Those are better modern complements to the older workflow-verification and merge-conflict literature already captured in this repo.

---

## 2. Highest-priority new references for `effort`

These are the strongest modern candidates for the planning layer.

### `2509.16198`

- title: `RPG: A Repository Planning Graph for Unified and Scalable Codebase Generation`
- link: [arXiv 2509.16198](https://arxiv.org/abs/2509.16198)
- why it matters:
  - explicit repository planning graph
  - persistent representation spanning proposal and implementation stages
  - direct inspiration for `WorkGraphV1` and graph-backed planning artifacts

### `2503.06689`

- title: `DependEval: Benchmarking LLMs for Repository Dependency Understanding`
- link: [arXiv 2503.06689](https://arxiv.org/abs/2503.06689)
- why it matters:
  - reinforces that repository dependency understanding is a distinct capability
  - useful when justifying Lift-to-effort dependency surfaces and test inputs

### `2602.02084`

- title: `Closing the Loop: Universal Repository Representation with RPG-Encoder`
- link: [Hugging Face 2602.02084](https://huggingface.co/papers/2602.02084)
- why it matters:
  - understanding and generation loop over one repository representation
  - useful precedent for a higher-fidelity repository substrate that can serve both localization and planning

### `2505.16901`

- title: `Code Graph Model (CGM): A Graph-Integrated Large Language Model for Repository-Level Software Engineering Tasks`
- link: [arXiv 2505.16901](https://arxiv.org/abs/2505.16901)
- why it matters:
  - graph-structured repository reasoning
  - explicit structural dependency integration
  - useful for thinking about richer Lift graph outputs without deep crate coupling

### `2605.03117`

- title: `ARISE: A Repository-level Graph Representation and Toolset for Agentic Fault Localization and Program Repair`
- link: [arXiv 2605.03117](https://arxiv.org/abs/2605.03117)
- why it matters:
  - multi-granularity repository graph
  - statement-level and data-flow-aware reasoning
  - useful for future higher-fidelity Lift signals and conflict reasoning

### `2602.13921`

- title: `GREPO: A Benchmark for Graph Neural Networks on Repository-Level Bug Localization`
- link: [arXiv 2602.13921](https://arxiv.org/abs/2602.13921)
- why it matters:
  - evidence that repository-wide dependency graphs materially help localization
  - supports graph-first thinking for planning inputs beyond file lists

### `2602.02934`

- title: `Beyond Blame: Rethinking SZZ with Knowledge Graph Search`
- link: [arXiv 2602.02934](https://arxiv.org/abs/2602.02934)
- why it matters:
  - temporal and structural graph search over repository history
  - useful for future causal and historical Lift signals instead of shallow blame-only logic

### `2309.12499`

- title: `CodePlan: Repository-level Coding using LLMs and Planning`
- link: [Hugging Face 2309.12499](https://huggingface.co/papers/2309.12499)
- why it matters:
  - incremental dependency analysis
  - change may-impact analysis
  - adaptive multi-step planning
  - useful bridge between coarse lift-style signals and richer planner inputs

### `2406.01422`

- title: `How to Understand Whole Software Repository?`
- link: [Hugging Face 2406.01422](https://huggingface.co/papers/2406.01422)
- why it matters:
  - repository knowledge graph plus structured exploration
  - useful for shaping richer lifted repository summaries before planning

### `2512.22469`

- title: `GraphLocator: Graph-guided Causal Reasoning for Issue Localization`
- link: [Hugging Face 2512.22469](https://huggingface.co/papers/2512.22469)
- why it matters:
  - explicit causal issue graphs
  - useful precedent for one-to-many issue decomposition and downstream work-graph shaping

---

## 3. Highest-priority new references for `exec`

These are the strongest modern candidates for runtime materialization, verified execution, and state handling.

### `2601.04688`

- title: `ToolGate: Contract-Grounded and Verified Tool Execution for LLMs`
- link: [arXiv 2601.04688](https://arxiv.org/abs/2601.04688)
- why it matters:
  - precondition/postcondition-gated execution
  - verified state evolution
  - strongest modern parallel to the code-intelligence contracts-and-gates model

### `2605.13848`

- title: `GraphBit: A Graph-based Agentic Framework for Non-Linear Agent Orchestration`
- link: [arXiv 2605.13848](https://arxiv.org/abs/2605.13848)
- why it matters:
  - deterministic DAG orchestration
  - engine-governed routing and state transitions
  - reproducibility and auditability
  - strong conceptual match for `exec`

### `2605.06110`

- title: `On Time, Within Budget: Constraint-Driven Online Resource Allocation for Agentic Workflows`
- link: [arXiv 2605.06110](https://arxiv.org/abs/2605.06110)
- why it matters:
  - explicit budget and deadline constraints over dependency-structured workflows
  - useful for later runtime allocation and constrained execution research

### `2603.22386`

- title: `From Static Templates to Dynamic Runtime Graphs: A Survey of Workflow Optimization for LLM Agents`
- link: [arXiv 2603.22386](https://arxiv.org/abs/2603.22386)
- why it matters:
  - useful vocabulary for distinguishing reusable workflow templates, realized graphs, and execution traces
  - maps well to plan artifacts versus runtime artifacts versus checkpoints

---

## 4. Additional useful references

These are adjacent but still worth keeping in the active reference pool.

### `2508.15204`

- title: `R-ConstraintBench: Evaluating LLMs on NP-Complete Scheduling`
- link: [arXiv 2508.15204](https://arxiv.org/abs/2508.15204)
- why it matters:
  - good reminder that interacting constraints, not graph depth alone, often cause the real failures
  - relevant to lane partitioning and conservative parallel-window generation

### `2502.11620`

- title: `Assessing Correctness in LLM-Based Code Generation via Uncertainty Estimation`
- link: [arXiv 2502.11620](https://arxiv.org/abs/2502.11620)
- why it matters:
  - strong support for abstention and uncertainty-aware gating
  - useful when deciding how low-confidence signals should constrain planning and closeout

### `2604.03551`

- title: `AgenticFlict: A Large-Scale Dataset of Merge Conflicts in AI Coding Agent Pull Requests on GitHub`
- link: [Hugging Face 2604.03551](https://huggingface.co/papers/2604.03551)
- why it matters:
  - merge conflicts are frequent in agentic PR flows
  - practical precedent for conservative conflict modeling around parallel work

### `2410.14684`

- title: `RepoGraph: Enhancing AI Software Engineering with Repository-level Code Graph`
- link: [Hugging Face 2410.14684](https://huggingface.co/papers/2410.14684)
- why it matters:
  - adjacent repository graph framing
  - useful additional support for graph-first repository reasoning

### `2412.00573`

- title: `Opus: A Large Work Model for Complex Workflow Generation`
- link: [arXiv 2412.00573](https://arxiv.org/abs/2412.00573)
- why it matters:
  - DAG-based workflow generation plus graph optimization
  - more workflow-generic than software-repo-specific, but still useful as planning vocabulary

### `2411.05451`

- title: `WorkflowLLM: Enhancing Workflow Orchestration Capability of Large Language Models`
- link: [arXiv 2411.05451](https://arxiv.org/abs/2411.05451)
- why it matters:
  - useful background on workflow orchestration benchmarks and hierarchical thought
  - weaker direct fit than repository-planning papers, but still useful context

### `2504.04578`

- title: `Hierarchical Planning for Complex Tasks with Knowledge Graph-RAG and Symbolic Verification`
- link: [arXiv 2504.04578](https://arxiv.org/abs/2504.04578)
- why it matters:
  - reinforces hierarchical decomposition plus symbolic verification
  - good supporting reference for staged plan validation

### `2410.10762`

- title: `AFlow: Automating Agentic Workflow Generation`
- link: [arXiv 2410.10762](https://arxiv.org/abs/2410.10762)
- why it matters:
  - search over code-represented workflows
  - useful as adjacent workflow-optimization framing

### `2506.09003`

- title: `SWE-Flow: Synthesizing Software Engineering Data in a Test-Driven Manner`
- link: [arXiv 2506.09003](https://arxiv.org/abs/2506.09003)
- why it matters:
  - runtime dependency graph for development scheduling
  - test-driven incremental development schedule
  - useful for thinking about validation-wall and test-topology driven planning

---

## 5. Older literature to keep, and why

Prioritizing newer work does not mean dropping the older references.

The older material in the existing raw captures still carries high-value concepts:

- merge-conflict prediction and speculative merging
- dependency-aware checkpoints
- blocked-state evidence
- summary-based validation
- final merged-tree truth

Those are durable concepts even when the newer work provides stronger modern vocabulary or better repository-level graph framing.

The clean rule is:

- use newer material to guide graph structure, repo-level reasoning, and modern execution vocabulary
- use older material to preserve foundational concepts that still map cleanly to the architecture

---

## 6. Recommended prioritization for future citation

If only a small number of modern papers are cited in the next planning pass, prioritize these first:

### For `effort`

1. `2509.16198` — `RPG`
2. `2503.06689` — `DependEval`
3. `2602.02084` — `RPG-Encoder`
4. `2505.16901` — `CGM`
5. `2605.03117` — `ARISE`
6. `2602.13921` — `GREPO`
7. `2602.02934` — `Beyond Blame`

### For `exec`

1. `2601.04688` — `ToolGate`
2. `2605.13848` — `GraphBit`
3. `2603.22386` — workflow optimization survey
4. `2605.06110` — constraint-driven online resource allocation

### For cautionary constraint framing

1. `2508.15204` — `R-ConstraintBench`
2. `2502.11620` — uncertainty estimation for correctness gating

---

## 7. Integration guidance for this repo

The cleanest next use of this addendum is:

- cite it from the `effort-exec` overlay
- use it as the modern-paper source when refining `effort` artifact design
- avoid promoting paper-specific jargon directly into canonical artifact names

This note is intentionally a reference ledger, not a contract surface.
