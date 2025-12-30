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
- **Execution workflow stays strict.** This standard must never weaken or contradict `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.

---

## 3) Required Outputs (The “Planning Pack”)

When you run a docs-first planning pass for a feature/track, you must produce a Planning Pack under:
`docs/project_management/next/<feature>/`

### 3.0 Supporting workflow docs (required references)

Planning work must use these standards:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
- `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`

### 3.1 Always required (minimum)

1) `plan.md`
- A runbook: scope boundaries, triad overview, invariants, and operator UX expectations.
  - Include a short triad sizing plan: each slice should represent one behavior delta and avoid “grab bag” scope (see `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`).

2) `tasks.json`
- Triad tasks (code/test/integration) with explicit dependencies, references, and acceptance criteria.

3) `session_log.md`
- Append-only START/END entries for planning and (later) execution sessions.

4) Specs (`<slice>-spec*.md`)
- One or more spec slices with: scope, exact behavior, error handling, platform rules, acceptance criteria, out-of-scope.

5) Kickoff prompts (`kickoff_prompts/<task>-{code,test,integ}.md`)
- Each prompt must clearly bound role responsibilities and required commands.

### 3.1.1 Strict task-triad interoperability (required)

Planning outputs must be directly executable under the triad workflow. Therefore:
- `tasks.json` must conform to the required task shape defined in `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md` (including required fields, checklists, and conventions).
- Docs discipline must match the triad standard:
  - docs/tasks/session logs are edited only on the orchestration branch,
  - docs/tasks/session logs are never edited from inside worktrees.

### 3.1.2 Cross-platform parity planning (required when cross-platform)

If the work requires cross-platform parity (Linux/macOS/Windows and optionally WSL):
- The planning pack must explicitly state the required platform set and the parity guarantees (in ADR/specs/contract).
- The planning pack must choose an integration task model:
  - validation-only, or
  - platform-fix when needed (recommended for any feature that could plausibly diverge by platform).
- If using platform-fix when needed, encode it mechanically in `tasks.json`:
  - `meta.schema_version: 2`
  - `meta.platforms_required: ["linux","macos","windows"]`
  - If WSL coverage is required, do not add `"wsl"` to `meta.platforms_required`; instead use `meta.wsl_required: true` and `meta.wsl_task_mode: "bundled"|"separate"`.
  - per slice: `X-integ-core`, `X-integ-<platform>`, and `X-integ` (final)
  - include platform smoke scripts under `smoke/` and reference them in integration tasks/end checklists.

### 3.2 Required for “decision-heavy” or “cross-platform” work

Add these files when the work introduces new user contracts, config, or platform behaviors:

1) `decision_register.md`
- A single source of truth for all architectural decisions.

2) `integration_map.md`
- An end-to-end map of affected components and how this work composes with adjacent tracks (config, policy, platform).

3) `manual_testing_playbook.md`
- A human-run checklist with explicit commands and expected exit codes/output. No “verify it works” steps.

4) Playbook automation scripts (required alongside the manual playbook)
- Every `manual_testing_playbook.md` must be paired with runnable smoke script(s) so agents can execute validation automatically and humans can re-run without typing every command.
- Scripts must live inside the feature directory they validate (not in the repo root `scripts/` tree):
  - Directory: `docs/project_management/next/<feature>/smoke/`
  - Linux: `docs/project_management/next/<feature>/smoke/linux-smoke.sh`
  - macOS: `docs/project_management/next/<feature>/smoke/macos-smoke.sh`
  - Windows: `docs/project_management/next/<feature>/smoke/windows-smoke.ps1`
- The manual playbook must reference the scripts and describe how to run them, plus how to manually run subsections.
- Triad integration tasks must run the relevant smoke scripts (where applicable) and record results in `session_log.md`.

### 3.3 Required for “multi-track alignment” work

If the scope spans multiple tracks (or you are reconciling a backlog), produce:

1) `docs/project_management/next/final_alignment_report.md`
- An auditable report of what was reviewed, what was changed, and what remains (must be “0 unresolved misalignments”).

2) Update `docs/project_management/next/sequencing.json` if needed
- Sequencing is the execution spine; if tasks and sequencing disagree, fix one and record it.

---

### 3.4 Quality gate artifact (required before execution triads begin)

Before any execution triad begins, a third-party quality gate reviewer must review the Planning Pack and produce an auditable report.

Required output:
- `docs/project_management/next/<feature>/quality_gate_report.md`
  - must follow `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
  - must include evidence that `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md` was run

Gating rule:
- Execution triads must not begin unless the quality gate recommendation is `ACCEPT`.

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

### 4.3 Required traceability to execution tasks

Every Decision Register entry must be executable, not just readable. Therefore:
- Every decision must have follow-up tasks that map to concrete triad task IDs in `tasks.json`.
- Every triad task that implements a decision must list that decision in its `references` (by file path and DR id), e.g.:
  - `docs/project_management/next/<feature>/decision_register.md (DR-00XX)`
- The integration task for the slice must confirm (in its END session log entry) that all referenced DR items are implemented and tested.

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

### 6.1 Automation pairing (required)

To make validation repeatable and auditable:
- Every manual playbook must have one or more corresponding smoke scripts (see section 3.2).
- Smoke scripts must:
  - use temp workspaces/homes by default (no destructive behavior),
  - accept env overrides (so CI or humans can point them at a specific prefix/version),
  - exit non-zero on failure and print actionable diagnostics,
  - be runnable without interactive prompts where possible.
- The manual playbook must state:
  - the exact command(s) to run the script(s),
  - what success looks like (exit code and key expected output),
  - how to run sections manually for debugging.
  - if cross-platform validation is required, how to run smoke scripts via GitHub Actions on self-hosted runners (preferred):
    - `scripts/ci/dispatch_feature_smoke.sh --feature-dir "$FEATURE_DIR" --runner-kind self-hosted --platform all --run-wsl --cleanup`

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

Scope note:
- Apply this rubric to feature planning outputs (ADRs/specs/plan/tasks/prompts/playbooks/reports) under `docs/project_management/next/…`, not to standards docs under `docs/project_management/standards/…`.

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

### 9.3 Planning Quality Gate Prompt (third-party reviewer)

See `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`.

---

## 10) “Ready For Implementation” Checklist (Planning Sign-Off)

The Planning Pack is implementation-ready only when all are true:
- Specs define exact behavior, defaults, error handling, exit codes, and out-of-scope boundaries.
- Decision register exists and covers all major decisions (two options each, one selection).
- Integration map exists and explicitly resolves cross-track dependencies and sequencing.
- Manual testing playbook exists (if UX/provisioning) with runnable commands and expected results.
- Smoke scripts exist (if UX/provisioning) for each required platform, and are referenced by the manual playbook.
- `tasks.json` has code/test/integration tasks with dependencies aligned to `sequencing.json`.
- Kickoff prompts exist for every task and reference the correct specs and required commands.
- Rubric checks pass (no banned ambiguity language in behavioral contracts).
