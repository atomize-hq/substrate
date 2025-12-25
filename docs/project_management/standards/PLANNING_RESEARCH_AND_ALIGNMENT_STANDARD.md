# Planning / Research / Documentation Standard (Docs-First)

This document is the sister standard to `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.

Use this standard when the next unit of work is **planning/research/documentation** (ADRs/specs/tasks/UX contracts),
not execution (production code/tests/integration). The goal is to produce an implementation-ready plan with **zero
ambiguity**, explicit tradeoffs, and cross-sprint alignment.

---

## 1) When To Use This Standard

Use a docs-first planning pass when one or more are true:
- Multiple triads/sprints interact (e.g., config + policy + platform backends + installer).
- Cross-platform parity is required (Linux/macOS/Windows).
- You need a stable user-facing contract (CLI, config files, exit codes, filesystem semantics).
- You are introducing a new configuration surface (file path, schema, precedence rules).
- Security/isolation is part of the requirement (fail-closed, privilege posture, cage constraints).
- The repo already contains partial plans and you must align/sequence them before execution.

If you already have crisp specs and only need to implement them, use the execution standard:
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

---

## 2) Core Principles (Non-Negotiable)

- **Docs are the single source of truth.** Execution reconciles to specs; specs are not retroactively “explained” by code.
- **Zero ambiguity.** Any behavior-level statement must be singular and testable.
- **Every architectural decision is explicit.** Do not leave open questions, TBDs, or “optional” behavior contracts.
- **Exactly two options per decision.** Every decision must compare two viable solutions and pick one.
- **Cross-sprint alignment is required.** Plans must reconcile with adjacent queued work and the repository’s sequencing spine.
- **Greenfield by default.** Do not plan migrations/backwards-compat unless an ADR explicitly mandates it.

---

## 3) Required Outputs (The “Planning Pack”)

When you run a docs-first planning pass for a feature/track, you must produce a Planning Pack under:
`docs/project_management/next/<feature>/`

### 3.1 Always required (minimum)

1) `plan.md`
- A runbook: scope boundaries, triad overview, invariants, and operator UX expectations.

2) `tasks.json`
- Triad tasks (code/test/integration) with explicit dependencies, references, and acceptance criteria.

3) `session_log.md`
- Append-only START/END entries for planning and (later) execution sessions.

4) Specs (`<slice>-spec*.md`)
- One or more spec slices with: scope, exact behavior, error handling, platform rules, acceptance criteria, out-of-scope.

5) Kickoff prompts (`kickoff_prompts/<task>-{code,test,integ}.md`)
- Each prompt must clearly bound role responsibilities and required commands.

### 3.2 Required for “decision-heavy” or “cross-platform” work

Add these files when the work introduces new user contracts, config, or platform behaviors:

1) `decision_register.md`
- A single source of truth for all architectural decisions.

2) `integration_map.md`
- An end-to-end map of affected components and how this work composes with adjacent tracks (config, policy, platform).

3) `manual_testing_playbook.md`
- A human-run checklist with explicit commands and expected exit codes/output. No “verify it works” steps.

### 3.3 Required for “multi-track alignment” work

If the scope spans multiple tracks (or you are reconciling a backlog), produce:

1) `docs/project_management/next/final_alignment_report.md`
- An auditable report of what was reviewed, what was changed, and what remains (must be “0 unresolved misalignments”).

2) Update `docs/project_management/next/sequencing.json` if needed
- Sequencing is the execution spine; if tasks and sequencing disagree, fix one and record it.

---

## 4) Decision Register Standard (Exact Format)

Every decision must be recorded as a Decision Register entry. Each entry must:
- present **exactly two** viable solutions (Option A / Option B),
- include pros/cons/implications/risks/unlocks/quick wins for both,
- end with one selected option and a crisp rationale,
- list explicit follow-up tasks (no hand-waving).

### 4.1 Required template

```md
### DR-XXXX — <Decision Title>

**Decision owner(s):** <role/team>  
**Date:** <YYYY-MM-DD>  
**Status:** Accepted | Superseded  
**Related docs:** <links>

**Problem / Context**
- <what is being decided and why now?>

**Option A — <name>**
- **Pros:** …
- **Cons:** …
- **Cascading implications:** …
- **Risks:** …
- **Unlocks:** …
- **Quick wins / low-hanging fruit:** …

**Option B — <name>**
- **Pros:** …
- **Cons:** …
- **Cascading implications:** …
- **Risks:** …
- **Unlocks:** …
- **Quick wins / low-hanging fruit:** …

**Recommendation**
- **Selected:** Option <A|B> — <name>
- **Rationale (crisp):** <why this tradeoff wins>

**Follow-up tasks (explicit)**
- <concrete tasks/spec edits/tests/scripts>
```

### 4.2 Prohibited patterns
- No “Option C”.
- No “maybe”, “TBD”, “open question”.
- No “A and B” selection; pick one.
- No “or” in behavior statements outside the Option A/Option B comparison.

---

## 5) Integration Map Standard

The integration map must be end-to-end and must answer:
- What components change (CLI, policy, world backends, installer scripts, schemas)?
- What inputs exist (config files, env vars, manifests)?
- What derived state exists (selection, routing, cage mode, backend identity)?
- What actions happen and where (status vs sync vs provision vs replay)?
- What cross-track prerequisites exist and where they live in sequencing?

Minimum sections:
- Scope / non-scope
- End-to-end data flow (inputs → derived state → actions)
- Component map (what changes where)
- Composition with adjacent tracks (explicit dependencies; no circulars)
- Sequencing alignment (final; no “placeholder” language)

---

## 6) Manual Testing Playbook Standard

If the work affects UX or provisioning, the playbook must:
- be runnable by a human without additional context,
- include explicit commands and **expected exit codes/output**,
- cover success + failure modes,
- include cross-platform sections when applicable (Linux/macOS/WSL).

Prohibited:
- “Verify it works”
- “Should”
- “Depends on … once it lands”

Encouraged:
- Using temporary workspaces/homes to avoid polluting a real environment.
- Including `jq -e` assertions for JSON outputs.

---

## 7) Sequencing and Dependency Alignment

You must reconcile three sources of truth:
1) `docs/project_management/next/sequencing.json` (macro-level order)
2) each triad `tasks.json` (micro-level dependencies)
3) the specs (prerequisites and invariants)

Rules:
- If sequencing.json says “X happens before Y”, tasks must not allow Y to start before X’s integration task.
- If tasks create a dependency not reflected in sequencing.json, either:
  - update sequencing.json, or
  - remove the dependency and adjust specs accordingly.
- Record the final aligned outcome in the alignment report or in the integration map.

---

## 8) Lint-Like Rubric (Enforceable Text Rules)

You must scan the scoped planning docs and remove violations. These are enforcement rules for *planning outputs*.

### 8.1 Hard bans (never allowed)

Forbidden anywhere in scoped planning docs (except in a historical quote inside an alignment report where you are
explicitly describing a removed violation):
- `TBD`, `TODO`, `WIP`, `TBA`
- `open question`
- `etc.`, `and so on`

### 8.2 Ambiguity bans (not allowed in behavior/contracts)

The following words must not appear in behavioral contracts (CLI semantics, configs, acceptance criteria, playbooks):
- `should` (replace with `must` or specify the exact behavior)
- `could`, `might`, `maybe`
- `optional` / `optionally`
- behavior-level “A or B” statements

Allowed exception:
- `optional` and “Option A/Option B” language is allowed only inside the explicit two-option decision comparisons in
  `decision_register.md`, and must still end with a single selected option.

### 8.3 Drift bans (names, paths, commands)

Forbidden:
- two different filenames for the same concept without explicit precedence rules,
- inconsistent command spelling/flags in different docs,
- inconsistent exit codes across specs/playbooks.

### 8.4 Testability bans

Forbidden in acceptance criteria and playbooks:
- “verify it works”
- “ensure it behaves correctly”
- any check without a runnable command and a concrete expected output/exit code.

### 8.5 Suggested grep/rg checks

Run these against the scoped directories:

```bash
rg -n "\\b(TBD|TODO|WIP|TBA)\\b" docs/project_management/next/<feature>
rg -n "open question" docs/project_management/next/<feature>
rg -n "\\betc\\.\\b|and so on" docs/project_management/next/<feature>
rg -n "\\b(should|could|might|maybe|optionally)\\b" docs/project_management/next/<feature>
```

For JSON validity:

```bash
jq . docs/project_management/next/sequencing.json >/dev/null
jq . docs/project_management/next/<feature>/tasks.json >/dev/null
```

---

## 9) Standard Prompt Templates

These templates are intended to be pasted into agent kickoff prompts.

### 9.1 Research / Planning Prompt Template (feature-local)

```md
You are the maintainer and program lead for <FEATURE>. Your job is to produce an implementation-ready Planning Pack
with zero ambiguity and full cross-track alignment.

Constraints:
- No production code.
- No TBD/optional/open questions; every decision is final and recorded.
- Greenfield: do not plan migrations/back-compat unless explicitly required by an ADR.

Required reading:
- <list ADRs/specs/standards/sequencing.json>

Deliverables (must create files):
- plan.md, tasks.json, session_log.md
- specs: <list>
- kickoff_prompts: <list>
- decision_register.md, integration_map.md, manual_testing_playbook.md (required for UX/provisioning work)

Decision rule:
- Every architectural decision must be recorded as exactly two options with pros/cons/implications/risks/unlocks/quick wins,
  followed by a single selected option and rationale.
```

### 9.2 Final Alignment Pass Prompt Template (multi-track)

```md
You are performing a final “zero-misalignment” pass across the queued planning docs for:
- <list directories>

Do not write production code. Fix docs only.
Enforce the Planning Rubric (lint-like rules). Output must contain 0 unresolved misalignments.

Deliverables:
- docs/project_management/next/final_alignment_report.md
- missing manual_testing_playbook.md files for any triads that affect UX/provisioning
- updated tasks.json dependencies and/or sequencing.json as required (final, no placeholders)
```

---

## 10) “Ready For Implementation” Checklist (Planning Sign-Off)

The Planning Pack is implementation-ready only when all are true:
- Specs define exact behavior, defaults, error handling, exit codes, and out-of-scope boundaries.
- Decision register exists and covers all major decisions (two options each, one selection).
- Integration map exists and explicitly resolves cross-track dependencies and sequencing.
- Manual testing playbook exists (if UX/provisioning) with runnable commands and expected results.
- `tasks.json` has code/test/integration tasks with dependencies aligned to `sequencing.json`.
- Kickoff prompts exist for every task and reference the correct specs and required commands.
- Rubric checks pass (no banned ambiguity language in behavioral contracts).

