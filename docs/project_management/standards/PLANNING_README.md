# Planning Standards Quick Start

This file explains how to run a docs-first planning pass that produces an execution-ready Planning Pack with **zero ambiguity** and strict traceability into the triad workflow.

This is the entrypoint for planning/research/documentation work. Execution work uses:
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (preferred when using triad automation; agents start inside worktrees)

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
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md` (when cross-platform / smoke scripts exist)
- `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
- `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
- `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
- `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`
- `docs/project_management/next/sequencing.json`

## Quality gate (required)

Planning is not “done” until a third-party reviewer:
1) Runs the mechanical checks in `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`.
2) Produces `docs/project_management/next/<feature>/quality_gate_report.md` using `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`.
3) Records `RECOMMENDATION: ACCEPT` or `RECOMMENDATION: FLAG FOR HUMAN REVIEW`.

Execution triads must not begin until the recommendation is `ACCEPT`.

If the recommendation is `FLAG FOR HUMAN REVIEW`, remediate the Planning Pack (docs-only) using:
- `docs/project_management/standards/PLANNING_QUALITY_GATE_REMEDIATION_PROMPT.md`

## Kickoff prompt (planning agent)

Before using this kickoff prompt, ensure `spec_manifest.md` exists for the feature directory:
- Run `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md` against the drafted ADR(s).
- Update the ADR `Related Docs` list to include `spec_manifest.md` and the selected `*-spec*.md` files.
Then create the impact map:
- Run `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md` to produce `impact_map.md` (replaces legacy `integration_map.md`).

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
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md` (cross-platform automation packs)
- `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
- `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (automation/worktree execution)
- `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md` (when cross-platform / smoke scripts exist)
- `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
- `docs/project_management/next/sequencing.json`
- `docs/project_management/next/<feature>/spec_manifest.md`
- `docs/project_management/next/<feature>/impact_map.md`
- All existing planning docs relevant to <FEATURE> (if any).

Required deliverables (must create or update):
1) Planning Pack (minimum):
   - `docs/project_management/next/<feature>/plan.md`
   - `docs/project_management/next/<feature>/spec_manifest.md` (authoritative spec list + surface ownership)
   - `docs/project_management/next/<feature>/impact_map.md` (touch set + cascading implications + cross-queue conflicts; replaces legacy `integration_map.md`)
   - `docs/project_management/next/<feature>/ci_checkpoint_plan.md` (cross-platform automation packs only; bounded CI checkpoints between groups of triads)
   - `docs/project_management/next/<feature>/tasks.json`
   - `docs/project_management/next/<feature>/session_log.md` (START/END entries only)
   - Specs: the exact spec docs listed in `spec_manifest.md` (no extras, no missing docs)
   - Kickoff prompts: `docs/project_management/next/<feature>/kickoff_prompts/*-{code,test,integ}.md`
	   - Execution gates (recommended; scaffolded by `make planning-new-feature` / `make planning-new-feature-ps`):
	     - `docs/project_management/next/<feature>/execution_preflight_report.md`
	     - `docs/project_management/next/<feature>/<SLICE_ID>-closeout_report.md` (e.g., `WCU0-closeout_report.md`)
		   - If you want to use triad execution automation (task runner/finisher + feature cleanup), scaffold with `AUTOMATION=1`:
		     - `make planning-new-feature FEATURE=<feature> AUTOMATION=1`
		     - `make planning-new-feature-ps FEATURE=<feature> AUTOMATION=1`
	   - For cross-platform packs, set `CROSS_PLATFORM=1` and optionally split scopes (P3-008):
	     - `make planning-new-feature FEATURE=<feature> CROSS_PLATFORM=1 AUTOMATION=1 BEHAVIOR_PLATFORMS=linux CI_PARITY_PLATFORMS=linux,macos,windows`
	2) If decision-heavy or cross-platform:
	   - `docs/project_management/next/<feature>/decision_register.md`
	   - `docs/project_management/next/<feature>/impact_map.md`
	   - `docs/project_management/next/<feature>/manual_testing_playbook.md`
   - `docs/project_management/next/<feature>/smoke/{linux-smoke.sh,macos-smoke.sh,windows-smoke.ps1}`

Required interoperability rules:
- `tasks.json` must match the required fields and workflow described in `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`.
- `spec_manifest.md` is authoritative for which spec documents must exist. If you discover a missing surface, update `spec_manifest.md` first, then create/update the required spec doc (do not proceed with implied/undocumented surfaces).
- `impact_map.md` is required (non-negotiable); it is the authoritative touch set + cascade/contradiction analysis (legacy: `integration_map.md` is deprecated).
- Planning Pack consistency is required:
  - Cross-compare all Planning Pack docs (ADR/spec_manifest/impact_map/specs/contract/tasks/playbook/smoke/kickoffs) to ensure names, defaults, precedence, schemas, exit codes, and behavior statements match exactly.
  - If a conflict is found, resolve it by updating the authoritative doc for that surface (do not “paper over” inconsistencies by duplicating contract text).
- Every task must have a kickoff prompt file and must include the exact rule: `Do not edit planning docs inside the worktree.`
- For cross-platform automation packs (schema v3+ + `meta.automation.enabled=true` + `meta.cross_platform=true`):
  - `ci_checkpoint_plan.md` is required and authoritative for CI cadence.
  - Cross-platform CI dispatch (compile parity / Feature Smoke / CI Testing) must occur only at the bounded checkpoints defined by `ci_checkpoint_plan.md` (default checkpoint size bounds: min=2 triads, max=4 triads unless explicitly justified).
  - `ci_checkpoint_plan.md` must be wired into `tasks.json` via checkpoint tasks (see `docs/project_management/standards/PLANNING_CI_CHECKPOINT_STANDARD.md`).
- Integration tasks must include the required validation gates and record results in `session_log.md`:
  - **Behavior platforms**: run the feature-local smoke script via CI (`make feature-smoke`) when `FEATURE_DIR/smoke/` exists.
  - **CI parity platforms**: run cross-platform compile parity (and CI Testing when required by the slice/workflow); smoke is not required for CI parity-only platforms.
  - For cross-platform smoke, prefer GitHub Actions + self-hosted runners via `make feature-smoke` (see `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`).
  - Before dispatching CI/smoke, run the advisory CI audit and follow its recommendation:
    - `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "<orch-branch>" --ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl"`
    - `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "<orch-branch>" --feature-dir "$FEATURE_DIR" --ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl"`
    - If `DIFF_CLASS=docs_only` and `RECOMMEND=skip`, CI/smoke may be skipped entirely.
  - After any dispatch completes, record evidence (recommended; do not commit the ledger):
    - `scripts/ci-audit/ci_audit_record.sh --ledger-path "$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl" --kind <ci-testing|feature-smoke> --orch-branch "<orch-branch>" --run-id "<id>" --tested-sha "<sha>" --feature-dir "$FEATURE_DIR"`
  - **Cross-platform Planning Packs are required to use the platform-fix integration model** (not optional):
    - Set `meta.schema_version >= 2` and `meta.cross_platform: true`.
    - Declare both scopes in `tasks.json` (P3-008):
      - `meta.behavior_platforms_required: [...]` (smoke scripts required here)
      - `meta.ci_parity_platforms_required: [...]` (platform-fix tasks required here; legacy: `meta.platforms_required`)
    - Per slice, create the full task shape: `X-integ-core`, `X-integ-<platform>` (for each CI parity platform), and `X-integ` (final aggregator).
    - This is enforced mechanically by `make planning-validate` (via `scripts/planning/validate_tasks_json.py`).
  - If WSL coverage is required, use `meta.wsl_required: true` and `meta.wsl_task_mode: "bundled"|"separate"` (do not add `"wsl"` to `meta.behavior_platforms_required` or `meta.ci_parity_platforms_required`).
  - Preferred smoke dispatch examples:
    - Behavior platforms (preferred): `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=behavior WORKFLOW_REF="feat/<feature>" SMOKE_CHECKOUT_REF="<sha>"`
    - Linux + WSL bundled: `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=linux RUN_WSL=1 WORKFLOW_REF="feat/<feature>" SMOKE_CHECKOUT_REF="<sha>"`
    - WSL-only (separate WSL task): `make feature-smoke FEATURE_DIR="$FEATURE_DIR" PLATFORM=wsl WORKFLOW_REF="feat/<feature>" SMOKE_CHECKOUT_REF="<sha>"`

Sequencing rules:
- Align `docs/project_management/next/sequencing.json` with task dependencies.
- If you introduce a dependency not represented in `sequencing.json`, update `sequencing.json` or remove the dependency and fix the plan/specs.

Output requirements:
- Produce a concise summary of files created/modified.
- Do not claim work is “ready” without a quality gate; the gate is a separate step.
```
