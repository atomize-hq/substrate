# Env Var Taxonomy + Override Split — plan

## Scope
- Feature directory: `docs/project_management/_archived/env_var_taxonomy_and_override_split/`
- Orchestration branch: `feat/env_var_taxonomy_and_override_split`
- Governing ADR: `docs/project_management/next/ADR-0006-env-var-taxonomy-and-override-split.md`

## Goal
- Establish a repo-wide environment variable taxonomy and eliminate dual-use config overrides by splitting:
  - exported state: `SUBSTRATE_*` (output-only for config resolution), and
  - override inputs: `SUBSTRATE_OVERRIDE_*` (inputs to effective config resolution).

## Guardrails (non-negotiable)
- Specs are the single source of truth; integration reconciles code/tests to the spec.
- Planning Pack docs are edited only on the orchestration branch (never inside task worktrees).
- Do not edit planning docs inside the worktree.
- Greenfield breaking is allowed (no backwards compatibility for legacy override semantics).
- Exit codes follow: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Sequencing prerequisite (macro)
- This sprint is sequenced after `policy_and_config_precedence` (ADR-0005 / PCP0) to avoid simultaneous churn in the same resolver layer.
  - Sequencing spine: `docs/project_management/next/sequencing.json`

## Triads overview (spec slices)

1) **EV0 — Override split for effective config**
- Implement `SUBSTRATE_OVERRIDE_*` parsing for config-shaped overrides.
- Ensure `SUBSTRATE_*` exported state values are not consulted as override inputs by the effective-config resolver.
- Perform an explicit repo-wide grep/audit to confirm no commands bypass effective config resolution by reading config-shaped `SUBSTRATE_*` values directly as inputs (and fix any hits that do).
- Update docs and the canonical env-var catalog references per ADR-0006.

Specs (single source of truth):
- `docs/project_management/_archived/env_var_taxonomy_and_override_split/EV0-spec.md`

## Cross-platform integration model
- Model: schema v2 cross-platform integration tasks (core + per-platform + final) encoded in `tasks.json` (meta.schema_version=3, meta.platforms_required set).
- Validation mechanism:
  - Preferred: GitHub Actions self-hosted runners via `make feature-smoke` (see `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`).
  - Local smoke execution is valid only on the matching platform.

## Primary code touchpoints (expected)
- Effective config resolution:
  - `crates/shell/src/execution/config_model.rs`
- Shell invocation planning (consumes effective config):
  - `crates/shell/src/execution/invocation/plan.rs`
- Canonical env-var catalog:
  - `docs/ENVIRONMENT_VARIABLES.md`
- Operator configuration reference (update as part of EV0 integration):
  - `docs/CONFIGURATION.md`

## Start checklist (all tasks)
1. `make triad-orch-ensure FEATURE_DIR="docs/project_management/_archived/env_var_taxonomy_and_override_split"`
2. Read: `plan.md`, `tasks.json`, `session_log.md`, `EV0-spec.md`, and your kickoff prompt.
3. Update task status to `in_progress` in `tasks.json`.
4. Add a START entry to `session_log.md`; commit docs (`docs: start <task-id>`).
5. Create the task worktree per the kickoff prompt (prefer triad automation where available).
6. Do not edit planning docs inside the worktree.

## End checklist (integration)
1. Run required checks (`cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, `make integ-checks`).
2. Dispatch and record cross-platform smoke results in `session_log.md`.
3. Complete `EV0-closeout_report.md`.
4. Update `tasks.json` + add END entry to `session_log.md`; commit docs (`docs: finish <task-id>`).
