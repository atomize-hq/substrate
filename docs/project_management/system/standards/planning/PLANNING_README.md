# Planning Standards Quick Start

This file explains how to run a docs-first planning pass that produces an execution-ready Planning Pack with **zero ambiguity** and strict traceability into the triad workflow.

This is the entrypoint for planning/research/documentation work. Execution work uses:
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (preferred when using triad automation; agents start inside worktrees)

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
- `docs/project_management/next/<feature>/spec_manifest.md`
- `docs/project_management/next/<feature>/kickoff_prompts/`
- Specs: `docs/project_management/next/<feature>/*-spec*.md`

Required for decision-heavy or cross-platform work:
- `docs/project_management/next/<feature>/decision_register.md`
- `docs/project_management/next/<feature>/impact_map.md`
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
- `spec_manifest.md`
- Specs (`*-spec*.md`)
- `kickoff_prompts/*-{code,test,integ}.md` for every task

Decision-heavy or cross-platform work must also include:
- `decision_register.md`
- `impact_map.md`
- `manual_testing_playbook.md`
- `smoke/` scripts:
  - `smoke/linux-smoke.sh`
  - `smoke/macos-smoke.sh`
  - `smoke/windows-smoke.ps1`

Before execution triads begin, the Planning Pack must include:
- `quality_gate_report.md` with `RECOMMENDATION: ACCEPT`

## Standards to follow (required reading)

Planning agents must read end-to-end:
- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md` (when cross-platform / smoke scripts exist)
- `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
- `docs/project_management/system/prompts/planning/quality_gate_reviewer.md`
- `docs/project_management/system/templates/planning_pack/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- `docs/project_management/next/sequencing.json`

## Quality gate (required)

Planning is not “done” until a third-party reviewer:
1) Runs the mechanical checks in `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`.
2) Produces `docs/project_management/next/<feature>/quality_gate_report.md` using `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`.
3) Records `RECOMMENDATION: ACCEPT` or `RECOMMENDATION: FLAG FOR HUMAN REVIEW`.

Execution triads must not begin until the recommendation is `ACCEPT`.

If the recommendation is `FLAG FOR HUMAN REVIEW`, remediate the Planning Pack (docs-only) using:
- `docs/project_management/system/prompts/planning/quality_gate_remediation.md`

## Kickoff prompt (planning agent)

Before using this kickoff prompt, ensure `spec_manifest.md` exists for the feature directory:
- Run `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md` against the drafted ADR(s).
- Update the ADR `Related Docs` list to include `spec_manifest.md` and the selected `*-spec*.md` files.
Then create the impact map:
- Run `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md` to produce `impact_map.md` (replaces legacy `integration_map.md`).

Copy/paste the following prompt to start a planning pass:

Canonical prompt: docs/project_management/system/prompts/planning/planning_kickoff_prompt.md

