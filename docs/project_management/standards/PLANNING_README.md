# Planning Standards Quick Start

This file explains how to run a docs-first planning pass that produces an execution-ready Planning Pack with **zero ambiguity** and strict traceability into the triad workflow.

This is the entrypoint for planning/research/documentation work. Execution work uses:
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`

## When to use planning

Run a planning pass when the next body of work requires any of:
- Cross-platform parity (Linux/macOS/Windows).
- A stable user contract (CLI/config/exit codes/path semantics).
- Security or isolation semantics (fail-closed posture, cage constraints).
- Sequencing across multiple tracks/sprints.

## Planning outputs (required)

Planning work must produce a Planning Pack under:
- `docs/project_management/next/<feature>/`

## Planning Pack artifact index (canonical filenames)

This is the quick “did we create everything?” checklist.

Always required:
- `docs/project_management/next/<feature>/plan.md`
- `docs/project_management/next/<feature>/tasks.json`
- `docs/project_management/next/<feature>/session_log.md`
- `docs/project_management/next/<feature>/kickoff_prompts/`
- Specs: `docs/project_management/next/<feature>/*-spec*.md`

Required for decision-heavy or cross-platform work:
- `docs/project_management/next/<feature>/decision_register.md`
- `docs/project_management/next/<feature>/integration_map.md`
- `docs/project_management/next/<feature>/manual_testing_playbook.md`
- `docs/project_management/next/<feature>/smoke/`
  - `docs/project_management/next/<feature>/smoke/linux-smoke.sh`
  - `docs/project_management/next/<feature>/smoke/macos-smoke.sh`
  - `docs/project_management/next/<feature>/smoke/windows-smoke.ps1`

Required before execution triads begin:
- `docs/project_management/next/<feature>/quality_gate_report.md`
  - must contain `RECOMMENDATION: ACCEPT`

Minimum required:
- `plan.md`
- `tasks.json`
- `session_log.md`
- Specs (`*-spec*.md`)
- `kickoff_prompts/*-{code,test,integ}.md` for every task

Decision-heavy or cross-platform work must also include:
- `decision_register.md`
- `integration_map.md`
- `manual_testing_playbook.md`
- `smoke/` scripts:
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`

Before execution triads begin, the Planning Pack must include:
- `quality_gate_report.md` with `RECOMMENDATION: ACCEPT`

## Standards to follow (required reading)

Planning agents must read end-to-end:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md` (when cross-platform / smoke scripts exist)
- `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
- `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
- `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/next/sequencing.json`

## Quality gate (required)

Planning is not “done” until a third-party reviewer:
1) Runs the mechanical checks in `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`.
2) Produces `docs/project_management/next/<feature>/quality_gate_report.md` using `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`.
3) Records `RECOMMENDATION: ACCEPT` or `RECOMMENDATION: FLAG FOR HUMAN REVIEW`.

Execution triads must not begin until the recommendation is `ACCEPT`.

## Kickoff prompt (planning agent)

Copy/paste the following prompt to start a planning pass:

```md
You are the planning/research/documentation agent for <FEATURE>.

Goal:
- Produce an execution-ready Planning Pack under `docs/project_management/next/<feature>/` with zero ambiguity.
- All decisions are final and explicitly recorded; the plan is auditable and strictly compatible with triad execution.

Constraints (non-negotiable):
- Do not write production code.
- Do not leave open questions, TBD/TODO/WIP/TBA, “optional” behavior, or ambiguous contracts.
- Greenfield by default: do not plan migrations/backwards compatibility unless an ADR explicitly mandates it.
- Every architectural decision must be recorded as exactly two options (A/B) with explicit tradeoffs and a single selected option.

Required reading (end-to-end):
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md` (when cross-platform / smoke scripts exist)
- `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/next/sequencing.json`
- All existing planning docs relevant to <FEATURE> (if any).

Required deliverables (must create or update):
1) Planning Pack (minimum):
   - `docs/project_management/next/<feature>/plan.md`
   - `docs/project_management/next/<feature>/tasks.json`
   - `docs/project_management/next/<feature>/session_log.md` (START/END entries only)
   - Specs: `docs/project_management/next/<feature>/*-spec*.md`
   - Kickoff prompts: `docs/project_management/next/<feature>/kickoff_prompts/*-{code,test,integ}.md`
   - Execution gates (recommended; scaffolded by `make planning-new-feature` / `make planning-new-feature-ps`):
     - `docs/project_management/next/<feature>/execution_preflight_report.md`
     - `docs/project_management/next/<feature>/<slice>-closeout_report.md` (e.g., `C0-closeout_report.md`)
   - If you want to use triad execution automation (task runner/finisher + feature cleanup), scaffold with `AUTOMATION=1`:
     - `make planning-new-feature FEATURE=<feature> AUTOMATION=1`
     - `make planning-new-feature-ps FEATURE=<feature> AUTOMATION=1`
2) If decision-heavy or cross-platform:
   - `docs/project_management/next/<feature>/decision_register.md`
   - `docs/project_management/next/<feature>/integration_map.md`
   - `docs/project_management/next/<feature>/manual_testing_playbook.md`
   - `docs/project_management/next/<feature>/smoke/{linux-smoke.sh,macos-smoke.sh,windows-smoke.ps1}`

Required interoperability rules:
- `tasks.json` must match the required fields and workflow described in `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.
- Every task must have a kickoff prompt file and must include the exact rule: `Do not edit planning docs inside the worktree.`
- Integration tasks must include running the feature-local smoke script (if present) and recording results in `session_log.md`.
  - For cross-platform smoke, prefer GitHub Actions + self-hosted runners via `make feature-smoke` (see `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`).
  - If you opt into the platform-fix integration model, set `meta.schema_version: 2` and `meta.platforms_required: [...]` in `tasks.json` and create `X-integ-core`, `X-integ-<platform>`, and `X-integ` tasks per slice (see `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`).
  - If WSL coverage is required, use `meta.wsl_required: true` and `meta.wsl_task_mode: "bundled"|"separate"` (do not add `"wsl"` to `meta.platforms_required`).
  - Preferred smoke dispatch examples:
    - All platforms: `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=all WORKFLOW_REF="feat/<feature>"`
    - Linux + WSL bundled: `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=linux RUN_WSL=1 WORKFLOW_REF="feat/<feature>"`
    - WSL-only (separate WSL task): `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=wsl WORKFLOW_REF="feat/<feature>"`

Sequencing rules:
- Align `docs/project_management/next/sequencing.json` with task dependencies.
- If you introduce a dependency not represented in `sequencing.json`, update `sequencing.json` or remove the dependency and fix the plan/specs.

Output requirements:
- Produce a concise summary of files created/modified.
- Do not claim work is “ready” without a quality gate; the gate is a separate step.
```
