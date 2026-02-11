# Decision Register — workflow-engine

Template standard:
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

### DR-0001 — Workflow spec file format + strictness

**Decision owner(s):** Shell / Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

**Problem / Context**
- The workflow engine needs an operator-editable spec format that is also easy to generate programmatically.
- The spec must be strict (unknown keys rejected) to preserve determinism and avoid silent behavior drift.

**Option A — YAML-only workflow specs (strict)**
- **Pros:** single format; easiest operator authoring; matches existing repo bias toward YAML.
- **Cons:** less convenient for programmatic generation; requires YAML tooling for callers.
- **Cascading implications:** CLI and docs can assume `.yaml`/`.yml` only; fewer parser branches.
- **Risks:** some agent/tooling ecosystems prefer emitting JSON; requires adapters.
- **Unlocks:** minimal initial scope.
- **Quick wins / low-hanging fruit:** simplest implementation.

**Option B — YAML + JSON workflow specs (strict)**
- **Pros:** operator-friendly YAML while allowing programmatic JSON emission; reduces friction for agentic callers.
- **Cons:** two parsers; marginally more testing surface.
- **Cascading implications:** `workflow validate/run` must accept both; docs must specify strictness and extension/format rules.
- **Risks:** drift between formats if tests are weak; mitigated by shared serde structs + golden fixtures.
- **Unlocks:** workflow generation tooling can emit JSON without YAML dependencies.
- **Quick wins / low-hanging fruit:** reuse existing `serde_yaml`/`serde_json` patterns already used in the repo.

**Recommendation**
- **Selected:** Option B — YAML + JSON workflow specs (strict)
- **Rationale (crisp):** preserves operator UX while removing unnecessary friction for programmatic workflow generation.

**Follow-up tasks (explicit)**
- Implement a single `workflow-types` serde model with `deny_unknown_fields` across all structs.
- Add `workflow validate` fixtures in both YAML and JSON that must parse identically.

### DR-0002 — DAG dependency representation in the spec

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

**Problem / Context**
- The workflow spec needs a single, deterministic representation of dependencies that is easy to validate and reason about.

**Option A — Explicit edge list (`edges: [{ from, to }]`)**
- **Pros:** unambiguous graph; easy to validate; keeps node definitions self-contained.
- **Cons:** verbose; ergonomics are slightly worse for hand-edits.
- **Cascading implications:** validation must ensure all nodes participate and graph is connected.
- **Risks:** users introduce redundant edges; mitigated by validation + normalization.
- **Unlocks:** stable internal graph conversion; easy topo scheduling.
- **Quick wins / low-hanging fruit:** aligns with the initial ADR draft structure.

**Option B — Node-local dependencies (`nodes[*].deps: [node_id]`)**
- **Pros:** compact and ergonomic for authors; localizes dependencies to node blocks.
- **Cons:** ambiguity around merge/override behavior and normalization; makes it easier to accidentally encode disconnected nodes.
- **Cascading implications:** validation must derive a canonical edge list anyway; schema is less explicit.
- **Risks:** authors confuse “deps” vs “inputs”; more subtle validation messages.
- **Unlocks:** slightly nicer YAML authoring.
- **Quick wins / low-hanging fruit:** minor reduction in file size.

**Recommendation**
- **Selected:** Option A — Explicit edge list (`edges`)
- **Rationale (crisp):** keeps the contract explicit and validation straightforward; avoids inventing dual representations.

**Follow-up tasks (explicit)**
- Implement strict edge validation (known node ids, acyclic, connected).
- Emit validation errors that include the offending `(from,to)` pair and suggested fix.

### DR-0003 — Failure semantics (fail-fast vs allow_failure)

**Decision owner(s):** Shell / World maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

**Problem / Context**
- Operators need deterministic workflow outcomes while still supporting “best effort” nodes (e.g., optional diagnostics) that should not fail the whole run.

**Option A — Fail-fast always**
- **Pros:** simplest mental model; simplest scheduler behavior.
- **Cons:** cannot express “optional” nodes; encourages moving work outside the workflow runner.
- **Cascading implications:** workflow always terminates on first failure; remaining runnable nodes never run.
- **Risks:** workflows become fragile; operators add ad-hoc wrappers.
- **Unlocks:** minimal implementation.
- **Quick wins / low-hanging fruit:** easy.

**Option B — Default fail-fast + explicit `allow_failure`**
- **Pros:** supports optional nodes deterministically; keeps default safe; still simple for most workflows.
- **Cons:** requires defining “skipped” and “continue” semantics.
- **Cascading implications:**
  - `defaults.fail_fast` default is `true`.
  - Nodes can set `allow_failure: true` to avoid failing the workflow on node failure.
  - If `fail_fast=false`, the scheduler continues running nodes that are not transitively blocked by a failed required dependency.
- **Risks:** operators misunderstand why a node did not run; mitigated by explicit node status (`skipped` with a reason).
- **Unlocks:** richer workflows without external scripting.
- **Quick wins / low-hanging fruit:** still implementable without a durable store.

**Recommendation**
- **Selected:** Option B — Default fail-fast + explicit `allow_failure`
- **Rationale (crisp):** keeps the safe default but enables common “best effort” workflows without escaping to ad-hoc orchestration.

**Follow-up tasks (explicit)**
- Define node status enum: `success|failed|denied|skipped`.
- Implement deterministic skip propagation: if any required dependency fails/denies, dependent nodes are `skipped`.
- Document that policy deny is always workflow-fatal (exit `4`) regardless of `allow_failure`.

### DR-0004 — Output wiring + reference model

**Decision owner(s):** Shell maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`

**Problem / Context**
- Workflow nodes need a deterministic way to consume prior node outputs and workflow inputs without embedding a full templating language.

**Option A — Typed JSON outputs + JSON Pointer / expression language**
- **Pros:** powerful and extensible; rich transformations possible.
- **Cons:** higher complexity; more opportunities for ambiguity; larger validation surface.
- **Cascading implications:** requires designing and supporting an expression language with stable semantics and error reporting.
- **Risks:** scope creep; user confusion; hard-to-test edge cases.
- **Unlocks:** future conditional routing and complex dataflow.
- **Quick wins / low-hanging fruit:** none; front-loads complexity.

**Option B — String templating with a small, fixed reference set**
- **Pros:** deterministic; small contract; easy to validate; sufficient for the initial DAG runner.
- **Cons:** limited transformations; users may need helper nodes for complex reshaping.
- **Cascading implications:** templating applies only to string fields; non-string values must be surfaced by executors as strings (or serialized deterministically).
- **Risks:** users want richer transformations; mitigated by adding explicit “transform” node kinds later.
- **Unlocks:** stable v1 without committing to an expression language.
- **Quick wins / low-hanging fruit:** implementable with a tiny interpolation engine + validation.

**Recommendation**
- **Selected:** Option B — String templating with a small, fixed reference set
- **Rationale (crisp):** maximizes determinism and keeps v1 scope bounded while still enabling basic dataflow.

**Follow-up tasks (explicit)**
- Define reference forms:
  - `inputs.<name>`
  - `nodes.<node_id>.<output_key>`
- Restrict interpolation to string fields; invalid references are validation errors (exit `2`).
- Implement `outputs` top-level mapping as stable named references to node outputs.

### DR-0005 — Trace representation (workflow spans + linkage)

**Decision owner(s):** Trace maintainers  
**Date:** 2026-02-11  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0021-substrate-workflow-engine.md`, `docs/project_management/adrs/draft/ADR-0028-in-world-process-execution-tracing-parity.md`

**Problem / Context**
- A workflow run must be observable and replayable as a coherent unit, not as an unstructured collection of unrelated spans.

**Option A — No dedicated workflow spans (only underlying command spans)**
- **Pros:** no schema work; minimal implementation effort.
- **Cons:** cannot represent workflow DAG and node outcomes; workflows become invisible at the “run” level.
- **Cascading implications:** replay/inspection tooling cannot reconstruct workflow structure reliably.
- **Risks:** loss of auditability for workflow orchestration decisions.
- **Unlocks:** none.
- **Quick wins / low-hanging fruit:** minimal.

**Option B — Dedicated workflow root + node spans, with explicit linkage**
- **Pros:** end-to-end observability; enables replay hooks; supports router triggers (ADR-0029); supports deterministic “node outcome” reporting.
- **Cons:** requires schema additions (additive) and careful redaction.
- **Cascading implications:** must extend the trace schema (additive-only) to include workflow correlation fields and node status.
- **Risks:** field proliferation; mitigated by defining a small required set and reusing existing span/link patterns.
- **Unlocks:** reliable “when node X completes, trigger Y” behavior and workflow-level replay tools.
- **Quick wins / low-hanging fruit:** enables phase-7 composition to remain inside Substrate’s trace model.

**Recommendation**
- **Selected:** Option B — Dedicated workflow root + node spans, with explicit linkage
- **Rationale (crisp):** workflows are a first-class orchestration surface and must be first-class in trace for auditability and downstream routing.

**Follow-up tasks (explicit)**
- Add `workflow_run` root span and `workflow_node` child spans in `crates/trace`.
- Ensure node spans link to underlying execution spans via parent/`graph_edges` without duplicating sensitive arguments.
- Add a Phase 8 additive-only update to the trace ADR/docs for the new workflow correlation fields.
