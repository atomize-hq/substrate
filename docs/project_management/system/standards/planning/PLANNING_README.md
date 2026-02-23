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

## Planning outputs

Planning work must produce a Planning Pack under:

- `docs/project_management/packs/active/<feature>/`

## Workstreams, Work Items, and dependencies

Planning Packs can optionally track **cross-pack coordination** and **external dependencies** at the `tasks.json` `meta` level:

- `meta.workstream_id` (optional): umbrella workstream for this pack (e.g., `WS-202602-dancing_monkey`; may be `null`)
- `meta.work_item_refs` (optional): related work item IDs (e.g., `WI-202602-provisioning_otter`)
- `meta.depends_on` / `meta.blocks` (optional): external dependencies and blockers
  - `adrs`: ADR refs (project convention; typically `ADR-XXXX`)
  - `work_items`: work item IDs (`WI-YYYYMM-codename`)
  - `workstreams`: workstream IDs (`WS-YYYYMM-codename`)
  - `packs`: repo-relative pack root paths (e.g., `docs/project_management/packs/active/foo-bar`)

Canonical registries:

- Workstreams: `docs/project_management/workstreams/`
- Work items: `docs/project_management/work_items/`

Validation behavior:

- Legacy packs (`meta.slice_spec_version` missing or `< 2`): these fields are optional and only validated when present (format + uniqueness).
- Strict packs (`meta.slice_spec_version >= 2`): referenced IDs/paths must resolve to on-disk records under the registries above.

## Automation vs manual packs (`AUTOMATION=1`)

`AUTOMATION=1` enables the **automation (triad-runner) planning pack mode**. It does **not** run any agents or start work automatically; it only changes what the planning scaffold generates and what the validators expect. In automation mode, `tasks.json` is scaffolded as an **automation pack** (schema v3/v4), slice triad tasks include triad-friendly metadata (branches/merge behavior/required checks), and the pack’s checklists assume you will execute work via the triad commands (`make triad-task-start*`, `make triad-task-finish`, `make triad-orch-ensure`, etc.). Actual execution only happens when you run those triad commands (and **`LAUNCH_CODEX=1`** is what launches Codex during task start); without that, the automation pack still sets up worktrees/metadata but you drive Codex manually.

## Planning Pack artifact index (canonical filenames)

This is the quick “did we create everything?” checklist.

Always required:

- `docs/project_management/packs/active/<feature>/plan.md`
- `docs/project_management/packs/active/<feature>/tasks.json`
- `docs/project_management/packs/active/<feature>/session_log.md`
- `docs/project_management/packs/active/<feature>/spec_manifest.md`
- Kickoff prompts must exist for every task (as referenced by `tasks.json` `kickoff_prompt`):
  - Slice task prompts: `docs/project_management/packs/active/<feature>/slices/<SLICE_ID>/kickoff_prompts/`
  - Feature/ops prompts: `docs/project_management/packs/active/<feature>/kickoff_prompts/`
- Specs: the exact docs listed in `spec_manifest.md` (including slice specs under `slices/<SLICE_ID>/<SLICE_ID>-spec.md`).

Required for decision-heavy or cross-platform work:

- `docs/project_management/packs/active/<feature>/decision_register.md`
- `docs/project_management/packs/active/<feature>/impact_map.md`
- `docs/project_management/packs/active/<feature>/manual_testing_playbook.md`
- `docs/project_management/packs/active/<feature>/smoke/`
  - `docs/project_management/packs/active/<feature>/smoke/linux-smoke.sh`
  - `docs/project_management/packs/active/<feature>/smoke/macos-smoke.sh`
  - `docs/project_management/packs/active/<feature>/smoke/windows-smoke.ps1`

Required before execution triads begin:

- `docs/project_management/packs/active/<feature>/quality_gate_report.md`
  - must contain `RECOMMENDATION: ACCEPT`

## Standards to follow (required reading)

Planning agents must read end-to-end:

- `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_WORK_LIFT_ADVISORY.md`
- `docs/project_management/system/standards/ci/PLATFORM_INTEGRATION_AND_CI.md` (when cross-platform / smoke scripts exist)
- `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`
- `docs/project_management/system/prompts/planning/quality_gate_reviewer.md`
- `docs/project_management/system/templates/planning_pack/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- `docs/project_management/packs/sequencing.json`

## Quality gate (required)

Planning is not “done” until a third-party reviewer:

1. Runs the mechanical checks in `docs/project_management/system/standards/planning/PLANNING_LINT_CHECKLIST.md`.
2. Produces `docs/project_management/packs/active/<feature>/quality_gate_report.md` using `docs/project_management/system/templates/planning_pack/PLANNING_GATE_REPORT_TEMPLATE.md`.
3. Records `RECOMMENDATION: ACCEPT` or `RECOMMENDATION: FLAG FOR HUMAN REVIEW`.

Execution triads must not begin until the recommendation is `ACCEPT`.

If the recommendation is `FLAG FOR HUMAN REVIEW`, remediate the Planning Pack (docs-only) using:

- `docs/project_management/system/prompts/planning/quality_gate_remediation.md`

## Kickoff prompt (planning agent)

Before using the planning kickoff prompt, ensure `spec_manifest.md` exists for the feature directory:

- Run `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md` against the drafted ADR(s).
- Update the ADR `Related Docs` list to include `spec_manifest.md` and the selected spec files.

Then create the impact map:

- Run `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md` to produce `impact_map.md` (replaces legacy `integration_map.md`).

Canonical planning kickoff prompt:

- `docs/project_management/system/prompts/planning/planning_kickoff_prompt.md`
