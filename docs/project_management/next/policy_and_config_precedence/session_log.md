## START — 2026-01-02T00:41:02Z — planning — ADR-0005 planning pack
- Feature: `docs/project_management/next/policy_and_config_precedence/`
- Branch: `feat/policy_and_config_precedence`
- Goal: Produce an execution-ready Planning Pack for ADR-0005 with zero ambiguity.
- Inputs to read end-to-end:
  - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/next/sequencing.json`
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
  - `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
  - `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
- Commands planned (if any):
  - `make adr-check ADR=docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `make adr-fix ADR=docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `jq -e . docs/project_management/next/policy_and_config_precedence/tasks.json >/dev/null`
  - `make planning-lint FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"`

## END — 2026-01-02T00:43:55Z — planning — ADR-0005 planning pack
- Summary of changes (exhaustive):
  - Created a dedicated Planning Pack for ADR-0005 under `docs/project_management/next/policy_and_config_precedence/` (plan/tasks/specs/decision register/integration map/playbook/smoke scripts/kickoff prompts).
  - Updated `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md` to reference the new feature directory and added the missing executive-summary drift guard.
  - Updated `docs/project_management/next/sequencing.json` to add the `policy_and_config_precedence` sprint and PCP0 slice.
  - Fixed ADR executive summary hashes via `make adr-fix` where required for mechanical lint.
- Files created/modified:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/next/sequencing.json`
  - `docs/project_management/next/policy_and_config_precedence/plan.md`
  - `docs/project_management/next/policy_and_config_precedence/tasks.json`
  - `docs/project_management/next/policy_and_config_precedence/session_log.md`
  - `docs/project_management/next/policy_and_config_precedence/PCP0-spec.md`
  - `docs/project_management/next/policy_and_config_precedence/decision_register.md`
  - `docs/project_management/next/policy_and_config_precedence/integration_map.md`
  - `docs/project_management/next/policy_and_config_precedence/manual_testing_playbook.md`
  - `docs/project_management/next/policy_and_config_precedence/quality_gate_report.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/PCP0-code.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/PCP0-test.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/PCP0-integ.md`
  - `docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh`
  - `docs/project_management/next/policy_and_config_precedence/smoke/macos-smoke.sh`
  - `docs/project_management/next/policy_and_config_precedence/smoke/windows-smoke.ps1`
- Rubric checks run (with results):
  - `jq -e . docs/project_management/next/policy_and_config_precedence/tasks.json >/dev/null` → `0` → pass
  - `jq -e . docs/project_management/next/sequencing.json >/dev/null` → `0` → pass
  - `make adr-check ADR=docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md` → `0` → pass
  - `make planning-lint FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"` → `0` → pass
- Sequencing alignment:
  - `sequencing.json` reviewed: `YES`
  - Changes required: added `policy_and_config_precedence` sprint (order 26)
- Blockers:
  - `NONE`
- Next steps:
  - `PCP0-code`: implement resolver change in `crates/shell/src/execution/config_model.rs`
  - `PCP0-test`: update `crates/shell/tests/config_show.rs` precedence assertions
  - `PCP0-integ`: merge, run `make integ-checks`, run smoke, and record CI run ids/URLs

## START — 2026-01-02T01:04:22Z — docs — scaffold execution gates (feature start + slice closeout)
- Feature: `docs/project_management/next/policy_and_config_precedence/`
- Branch: `feat/policy_and_config_precedence`
- Goal: Add the execution preflight start gate and the per-slice closeout gate scaffolds so triads start with runnable smoke coverage and end with no-drift evidence.

## END — 2026-01-02T01:08:06Z — docs — scaffold execution gates (feature start + slice closeout)
- Summary of changes (exhaustive):
  - Enabled execution gates in `tasks.json` meta and added the `F0-exec-preflight` gate task.
  - Wired `PCP0-code` and `PCP0-test` to depend on `F0-exec-preflight`.
  - Added per-slice closeout gate scaffolding for PCP0 and linked it from `PCP0-integ` (references + end checklist).
  - Updated kickoff prompts to include the execution gate prerequisites.
  - Adjusted planning docs to satisfy mechanical lint bans.
- Files created/modified:
  - `docs/project_management/next/policy_and_config_precedence/tasks.json`
  - `docs/project_management/next/policy_and_config_precedence/execution_preflight_report.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/F0-exec-preflight.md`
  - `docs/project_management/next/policy_and_config_precedence/PCP0-closeout_report.md`
  - `docs/project_management/next/policy_and_config_precedence/plan.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/PCP0-code.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/PCP0-test.md`
  - `docs/project_management/next/policy_and_config_precedence/kickoff_prompts/PCP0-integ.md`
  - `docs/project_management/next/policy_and_config_precedence/quality_gate_report.md`
- Rubric checks run (with results):
  - `jq -e . docs/project_management/next/policy_and_config_precedence/tasks.json >/dev/null` → `0` → pass
  - `make planning-validate FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"` → `0` → pass
  - `make planning-lint FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"` → `0` → pass
- Blockers:
  - `NONE`
- Next steps:
  - Run `F0-exec-preflight` and fill `execution_preflight_report.md` before starting `PCP0-code` / `PCP0-test`.

## START — 2026-01-02T01:18:48Z — docs — F0-exec-preflight (execution preflight gate)
- Feature: `docs/project_management/next/policy_and_config_precedence/`
- Branch: `feat/policy_and_config_precedence`
- Goal: Run the execution preflight start gate and produce an ACCEPT/REVISE recommendation.
- Notes:
  - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"` initially failed because this Planning Pack is not yet automation-enabled (`tasks.json` meta.schema_version=1 / no meta.automation); preflight will upgrade the pack to schema v3 + automation and re-run `triad-orch-ensure`.

## END — 2026-01-02T01:31:48Z — docs — F0-exec-preflight (execution preflight gate)
- Recommendation: `ACCEPT`
- Summary of changes (exhaustive):
  - Created and pushed the orchestration branch: `feat/policy_and_config_precedence`.
  - Upgraded the Planning Pack to the current automation + cross-platform integration model:
    - `tasks.json` schema v3 + `meta.automation.*`
    - `PCP0-integ-core` + `PCP0-integ-{linux,macos,windows}` + `PCP0-integ` (final)
    - `FZ-feature-cleanup` task + kickoff prompt
  - Hardened smoke scripts to match the manual playbook and validate exit codes:
    - Asserts workspace-over-env precedence for `world.caged` (and prints the observed value on failure)
    - Asserts `substrate config show --json` exits `2` when no workspace exists
    - Missing prerequisites (e.g., `substrate`, `jq`) exit `3` per smoke/playbook convention
  - Updated `plan.md` to reflect the schema v2/v3 integration model.
- Evidence (commands):
  - `make triad-orch-ensure FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"` → success
  - `make planning-lint FEATURE_DIR="docs/project_management/next/policy_and_config_precedence"` → success
  - `bash docs/project_management/next/policy_and_config_precedence/smoke/linux-smoke.sh` (with `substrate` on PATH) → fails on baseline as expected (feature not implemented yet)
- CI readiness:
  - Self-hosted runner inventory verified via GitHub API: `linux-manjaro-runner`, `macOS-runner`, `windows11-runner` (WSL runner present but not required).
- Required fixes before starting `PCP0-code` / `PCP0-test`: none.

## START — 2026-01-02T01:42:48Z — docs — PCP0-test (workspace config precedence over env — tests)
- Feature: `docs/project_management/next/policy_and_config_precedence/`
- Branch: `feat/policy_and_config_precedence` (orchestration), `pcp-pcp0-precedence-test` (task)
- Goal: Update `crates/shell/tests/config_show.rs` precedence assertions to match ADR-0005 while preserving protected excludes assertions.
- Notes:
  - Worktree: `wt/pcp0-precedence-test`

## START — 2026-01-02T01:42:27Z — code — PCP0-code (workspace config precedence over env)
- Feature: `docs/project_management/next/policy_and_config_precedence/`
- Branch: `feat/policy_and_config_precedence`
- Task branch: `pcp-pcp0-precedence-code`
- Worktree: `wt/pcp0-precedence-code`
- Goal: Implement the precedence change in the effective-config resolver: when a workspace exists, workspace config overrides `SUBSTRATE_*` env vars (production code only).

## END — 2026-01-02T01:46:52Z — code — PCP0-code (workspace config precedence over env)
- Summary of changes:
  - Updated effective-config precedence so that, when a workspace exists, `<workspace_root>/.substrate/workspace.yaml` wins over `SUBSTRATE_*` env vars.
- Files changed:
  - `crates/shell/src/execution/config_model.rs`
- Evidence (commands):
  - `cargo fmt` → success
  - `cargo clippy --workspace --all-targets -- -D warnings` → success
