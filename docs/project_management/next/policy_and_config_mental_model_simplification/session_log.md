## START — 2025-12-28T02:07:08Z — planning — planning pack scaffolding
- Feature: `docs/project_management/next/policy_and_config_mental_model_simplification/`
- Branch: `testing`
- Goal: Produce an execution-ready Planning Pack for ADR-0003 with zero ambiguity.
- Inputs to read end-to-end:
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md`
  - `docs/project_management/next/sequencing.json`
  - `docs/project_management/standards/PLANNING_README.md`
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md`
  - `docs/project_management/standards/PLANNING_SESSION_LOG_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_GATE_REPORT_TEMPLATE.md`
  - `docs/project_management/standards/PLANNING_QUALITY_GATE_PROMPT.md`
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`

## END — 2025-12-28T02:20:53Z — planning — planning pack scaffolding
- Summary of changes (exhaustive):
  - Created the Planning Pack scaffolding (plan, specs, tasks, prompts, integration map, manual playbook, smoke scripts).
  - Rewrote the feature decision register to the two-option decision template with explicit follow-up task IDs.
  - Updated `docs/project_management/next/sequencing.json` to point the sprint to the feature directory and spec slices.
  - Updated the ADR related-doc links and validation artifact references to point to the Planning Pack.
- Files created/modified:
  - `docs/project_management/next/policy_and_config_mental_model_simplification/plan.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/tasks.json`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/session_log.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/decision_register.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/integration_map.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/manual_testing_playbook.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM0-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM1-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM2-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/PCM3-spec.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM0-code.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM0-test.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM0-integ.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM1-code.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM1-test.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM1-integ.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM2-code.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM2-test.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM2-integ.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM3-code.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM3-test.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/kickoff_prompts/PCM3-integ.md`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh`
  - `docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1`
  - `docs/project_management/next/ADR-0003-policy-and-config-mental-model-simplification.md`
  - `docs/project_management/next/sequencing.json`
- Rubric checks run (with results):
  - `rg -n --hidden --glob '!**/.git/**' "$(printf '%b' '\\b(\\x54\\x42\\x44|\\x54\\x4f\\x44\\x4f|\\x57\\x49\\x50|\\x54\\x42\\x41)\\b|\\x6f\\x70\\x65\\x6e\\x20\\x71\\x75\\x65\\x73\\x74\\x69\\x6f\\x6e|\\b\\x65\\x74\\x63\\x2e|\\x61\\x6e\\x64\\x20\\x73\\x6f\\x20\\x6f\\x6e')" "$FEATURE_DIR"` → exit `1` → no matches
  - `rg -n --hidden --glob '!**/.git/**' "$(printf '%b' '\\b(\\x73\\x68\\x6f\\x75\\x6c\\x64|\\x63\\x6f\\x75\\x6c\\x64|\\x6d\\x69\\x67\\x68\\x74|\\x6d\\x61\\x79\\x62\\x65|\\x6f\\x70\\x74\\x69\\x6f\\x6e\\x61\\x6c\\x6c\\x79|\\x6f\\x70\\x74\\x69\\x6f\\x6e\\x61\\x6c)\\b')" "$FEATURE_DIR"` → exit `1` → no matches
  - `jq -e . "$FEATURE_DIR/tasks.json" >/dev/null` → exit `0` → OK
  - `jq -e . docs/project_management/next/sequencing.json >/dev/null` → exit `0` → OK
  - `python (tasks.json required-field audit)` → exit `0` → OK
  - `python (integration smoke references audit)` → exit `0` → OK
  - `python (kickoff prompt existence audit)` → exit `0` → OK
  - `rg -n 'Do not edit docs/tasks/session_log\\.md inside the worktree\\.' "$FEATURE_DIR/kickoff_prompts" >/dev/null` → exit `0` → OK
  - `rg -n 'smoke/(linux-smoke\\.sh|macos-smoke\\.sh|windows-smoke\\.ps1)' "$FEATURE_DIR/manual_testing_playbook.md" >/dev/null` → exit `0` → OK
- Sequencing alignment:
  - `sequencing.json` reviewed: `YES`
  - Changes required: updated sprint directory and sequence ids for this feature
- Blockers:
  - `NONE`
- Next steps:
  - Quality gate reviewer: run `docs/project_management/standards/PLANNING_LINT_CHECKLIST.md` and create `docs/project_management/next/policy_and_config_mental_model_simplification/quality_gate_report.md` with `RECOMMENDATION: ACCEPT`.
  - Execution triads: start `PCM0-code` and `PCM0-test` after the quality gate is `ACCEPT`.

## START — 2025-12-28T12:49:11Z — PCM0-code — workspace + config inventory and CLI
- Orchestration branch: `feat/policy_and_config`
- Task branch: `pcm-pcm0-config-code`
- Worktree: `wt/pcm0-config-code`
- Scope: workspace discovery/init; strict config schema/discovery/precedence; `substrate workspace init` and `substrate config *` per `PCM0-spec.md`

## END — 2025-12-28T13:23:14Z — PCM0-code — workspace + config inventory and CLI
- Summary of changes:
  - Implemented workspace marker discovery (`.substrate/workspace.yaml`) and `substrate workspace init` scaffolding (`.substrate/`, `.substrate-git/repo.git/`, `.gitignore` rules) with nested-workspace refusal.
  - Implemented strict config schema parsing and effective-config precedence (CLI > env > workspace > global > defaults), including protected sync excludes and hard rejection of legacy `.substrate/settings.yaml`.
  - Implemented `substrate config show|set` (workspace scope) and `substrate config global init|show|set` (global scope) per `PCM0-spec.md`.
- Commands run (required):
  - `cargo fmt` → exit `0`
  - `cargo clippy --workspace --all-targets -- -D warnings` → exit `0`
    - Output: `Finished \`dev\` profile [unoptimized + debuginfo] target(s) in 3.17s`

## START — 2025-12-28T13:48:44Z — PCM0-integ — workspace + config inventory and CLI (integration)
- Orchestration branch: `feat/policy_and_config`
- Task branch: `pcm-pcm0-config-integ`
- Worktree: `wt/pcm0-config-integ`
- Scope: merge PCM0 code+tests, reconcile to `PCM0-spec.md`, and run fmt/clippy/tests/preflight + smoke scripts.

## START — 2025-12-28T13:50:26Z — PCM0-test — workspace + config inventory and CLI (test)
- Orchestration branch: `feat/policy_and_config`
- Task branch: `pcm-pcm0-config-test`
- Worktree: `wt/pcm0-config-test`
- Scope: tests for PCM0 workspace discovery/init, config precedence + strict parsing, and protected excludes per `PCM0-spec.md`.

## END — 2025-12-28T14:12:56Z — PCM0-test — workspace + config inventory and CLI (test)
- Summary of changes:
  - Added integration tests covering workspace init/discovery semantics, config CLI (global + workspace scope), strict parsing, and config precedence per `PCM0-spec.md`.
  - Verified protected sync excludes are always present in effective config and cannot be removed via config updates.
- Commands run (required):
  - `cargo fmt` → exit `0`
  - `cargo test -p substrate-shell --test config_init --test config_show --test config_set --test workspace_init -- --nocapture` → exit `0`

## END — 2025-12-28T14:54:20Z — PCM0-integ — workspace + config inventory and CLI (integration)
- Summary of changes:
  - Fixed integration/unit tests and world enable plumbing to match the strict PCM0 config schema (no legacy `install.*` / `root_mode` / `root_path`).
  - Made `substrate world enable` overwrite invalid legacy config files with fresh PCM0 defaults before enabling `world.enabled`.
  - Hardened `substrate-shell` integration tests to run with an isolated `$HOME`/`$SUBSTRATE_HOME` so host config does not break CI.
- Commands run (required):
  - `cargo fmt` → exit `0`
  - `cargo clippy --workspace --all-targets -- -D warnings` → exit `0`
  - `cargo test -p substrate-shell --test config_init --test config_show --test config_set --test workspace_init -- --nocapture` → exit `0`
  - `make preflight` → exit `0`
- Smoke scripts:
  - `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/linux-smoke.sh` (with `PATH=target/debug:$PATH`) → exit `1` (fails on `$SUBSTRATE_HOME/env.sh` check; `substrate policy *` not yet implemented in PCM0)
  - `bash docs/project_management/next/policy_and_config_mental_model_simplification/smoke/macos-smoke.sh` → exit `0` (SKIP: not macOS)
  - `pwsh -File docs/project_management/next/policy_and_config_mental_model_simplification/smoke/windows-smoke.ps1` → exit `127` (`pwsh` not found)

## START — 2025-12-28T15:02:47Z — PCM1-code — policy inventory and CLI
- Orchestration branch: `feat/policy_and_config`
- Task branch: `pcm-pcm1-policy-code`
- Worktree: `wt/pcm1-policy-code`
- Scope: policy schema/discovery/strict parsing/invariants; `substrate policy *` per `PCM1-spec.md`
