# Triad Workflow (Cross-Platform Integration via Self-Hosted CI)

This diagram is the “execution phase” complement to:
- `docs/project_management/standards/PLANNING_WORKFLOW_OVERVIEW.md`

It shows:
- code/test running in parallel worktrees (created via the triad task runner),
- a core integration merge task (`X-integ-core`),
- parallel platform-fix integration tasks (`X-integ-<platform>`) that only make changes if a platform fails, and
- a final cross-platform integration aggregator (`X-integ`) that merges platform fixes and re-validates.
- worktrees retained through the feature and removed only by the feature cleanup task (`FZ-feature-cleanup`).

Operational notes (important for correct orchestration):
- CI smoke dispatch (`make feature-smoke ...`) validates the **current `HEAD`** by creating/pushing a throwaway branch at that commit; run it from the worktree that contains the code you intend to validate (e.g., the `X-integ-core` or `X-integ` worktree).
- Platform-fix tasks should begin by merging the `X-integ-core` task branch into their own branch before running smoke or making fixes.
- Before any CI dispatch, run a **local behavioral smoke preflight** (fast fail) on the current platform when possible:
  - Build `substrate` in the integration worktree, add `target/debug` to `PATH`, and run the matching feature-local smoke script from `docs/project_management/next/<feature>/smoke/` (or `"$FEATURE_DIR/smoke/"` if you set `FEATURE_DIR`).
  - This catches obvious smoke-script/behavior drift before burning runner time and creating throwaway branches.
- Cross-platform compile parity should be validated before Feature Smoke dispatch to avoid discovering macOS/Windows compilation breaks only after creating temp branches and consuming runner time:
  - `make ci-compile-parity CI_WORKFLOW_REF="$ORCH_BRANCH" CI_REMOTE=origin CI_CLEANUP=1` (dispatches CI Testing in `mode=compile-parity` via `scripts/ci/dispatch_ci_testing.sh`).
  - If compile parity fails: treat it as **blocking** and fix it on the `X-integ-core` branch/worktree (cfg/platform guards), then re-run compile parity until green; do not dispatch Feature Smoke until it is green.
- Workflow dispatch reliability note: GitHub only allows `workflow_dispatch` by workflow file if that workflow is registered on the default branch (`main`). Operationally, this means:
  - You may need to land *workflow-file-only* changes on `main` so the workflow is registered.
  - Do not dispatch runs from `main` (or `testing`) during triad execution; dispatch from the feature’s orchestration/task ref and rely on the dispatcher’s throwaway `checkout_ref` branch to test the exact commit you care about.
- Feature Smoke should be dispatched for **behavior platforms only** (read from `tasks.json` via `PLATFORM=behavior`) unless the feature explicitly requires behavior smoke on all three OSes.
- CI Testing is a separate, stricter gate than Feature Smoke:
  - Use `mode=quick` for automation selection (skip docs/cross-build) before deciding “no-op platform fixes”.
  - Use `mode=full` (default) as the final CI gate on the final `X-integ` commit before merging to `testing`.
- The dispatch scripts are hardened with timeouts to avoid indefinite hangs; default max wait is **2 hours**. If you hit infra slowness, you can override via `FEATURE_SMOKE_WATCH_TIMEOUT_SECS` / `CI_TESTING_WATCH_TIMEOUT_SECS` (see `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md`).
- Headless Codex runs write a PID file while running: `target/triad/<feature>/codex/<task>/codex.pid`. If an orchestration session is interrupted, check for stale PID files before starting new runs.
- Orchestration wrappers should always emit their stdout contract (including `CODEX_EXIT`) even when Codex fails; if you see missing keys, treat it as an automation bug and capture the command output + `target/triad/...` artifact paths for follow-up.

## Task Granularity (Keep Triad Slices Small)

Cross-platform integration goes off the rails fastest when slices are too large to reason about. Prefer more (smaller) tasks/slices over fewer (larger) ones:
- Split along natural seams (CLI wiring vs. backend API vs. schema/tests vs. docs).
- Keep each Codex run focused on a single acceptance criterion cluster.
- Use the older “feature sprint” patterns as references for how small to go:
  - `docs/project_management/_archived/p0-platform-stability/kickoff_prompts/`
  - `docs/project_management/_archived/p0-platform-stability-macOS-parity/kickoff_prompts/`
  - `docs/project_management/_archived/policy_and_config_mental_model_simplification/kickoff_prompts/`

This file includes two diagrams:
- **Diagram A:** The overall cross-platform platform-fix triad flow.
- **Diagram B:** The same flow with explicit emphasis on **WSL bundled-by-default** behavior.
- **Diagram C:** A zoomed-in view of the triad task execution automation (branch/worktree lifecycle, Codex headless launch, and feature cleanup).

## Diagram A — Cross-Platform Platform-Fix Triad Flow

```mermaid
flowchart TD
  %% ======== Inputs / gating ========
  ADR["ADR accepted (Executive Summary drift guard OK)"]
  PACK["Planning Pack ready (plan/tasks/specs/prompts)"]
  GATE["Quality Gate (RECOMMENDATION: ACCEPT)"]

  ADR --> PACK --> GATE

  %% ======== Triad execution ========
  subgraph ORCH["Orchestration Branch (docs edits only)"]
    ORCH_ENSURE["Ensure orchestration branch exists (make triad-orch-ensure)"]
    START["Pick triad slice: X (e.g., C3)"]
    TASKS["tasks.json statuses (START/END in session_log.md)"]
    START_PAIR["Start X-code + X-test in parallel (make triad-task-start-pair SLICE_ID=X; optional LAUNCH_CODEX=1)"]
  end

  GATE --> ORCH_ENSURE --> START
  START --> TASKS
  TASKS --> START_PAIR

  subgraph WT["Parallel Worktrees (no docs edits)"]
    CODE["X-code (prod code only; created via make triad-task-start-pair; writes .taskmeta.json)"]
    TEST["X-test (tests only; created via make triad-task-start-pair; writes .taskmeta.json)"]
  end

  START_PAIR --> CODE
  START_PAIR --> TEST

  subgraph INTEG_CORE["Core Integration (primary dev platform)"]
    MERGE["X-integ-core (merge X-code + X-test; resolve spec drift)"]
    CORE_CHECKS["Required checks: cargo fmt; cargo clippy ... -- -D warnings; relevant tests; make integ-checks"]
    CORE_LOCAL_SMOKE["Local behavioral smoke preflight (build substrate; run feature-local smoke script for this platform)"]
    CORE_PARITY["Dispatch cross-platform compile parity via make ci-compile-parity (GitHub-hosted; CI parity platforms)"]
    CORE_FIX["If parity fails: fix compile parity on X-integ-core branch (cfg/platform guards), commit, and re-run parity"]
    CORE_DISPATCH["Dispatch behavioral smoke via CI (prefer PLATFORM=behavior; optional RUN_WSL=1; WORKFLOW_REF should be the orchestration/task ref, not main/testing)"]
    CORE_RESULTS["Wait for smoke results (self-hosted runners)"]
    CORE_CI_TEST["Dispatch CI Testing (mode=quick) via scripts/ci/dispatch_ci_testing.sh (throwaway branch at HEAD)"]
    CORE_IDENTIFY["Identify failing platforms from smoke + CI Testing results"]
    CORE_START_PF["Start failing platform-fix tasks (use from-smoke-run when smoke was a single dispatch; else explicit PLATFORMS=...)"]
    CORE_START_FINAL["After fixes are green: start final aggregator (make triad-task-start-integ-final SLICE_ID=X)"]
  end

  CODE --> MERGE
  TEST --> MERGE
  MERGE --> CORE_CHECKS --> CORE_LOCAL_SMOKE --> CORE_PARITY
  CORE_PARITY -->|pass| CORE_DISPATCH
  CORE_PARITY -->|fail| CORE_FIX --> CORE_PARITY

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE_ALL["Feature Smoke workflow (self-hosted runners; behavior platforms; optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE_ALL --> CORE_RESULTS --> CORE_CI_TEST --> CORE_IDENTIFY --> CORE_START_PF

  %% ======== Platform-fix tasks (parallel, only if needed) ========
  subgraph PF["Platform-fix Integration Tasks (worktrees on platform machines)"]
    LNX["X-integ-linux (if Linux fails: fix in Linux worktree; default includes WSL via --run-wsl; re-run smoke)"]
    MAC["X-integ-macos (if macOS fails: fix in macOS worktree; re-run smoke)"]
    WIN["X-integ-windows (if Windows fails: fix in Windows worktree; re-run smoke)"]
    WSL["X-integ-wsl (rare; if WSL fails and you want separate ownership; fix + re-run smoke)"]
  end

  CORE_START_PF --> LNX
  CORE_START_PF --> MAC
  CORE_START_PF --> WIN
  CORE_START_PF --> WSL

  %% ======== Final aggregator ========
  subgraph INTEG_FINAL["Final Cross-Platform Integration"]
    AGG["X-integ (final; merge platform fixes; run integ checks; re-run cross-platform smoke)"]
    AGG_CI_TEST["Dispatch CI Testing (throwaway branch at HEAD)"]
    MERGE_BACK["Merge back to orchestration branch; update tasks.json/session_log.md (worktrees retained; cleanup at feature end)"]
  end

  LNX --> AGG
  MAC --> AGG
  WIN --> AGG
  WSL --> AGG
  CORE_DISPATCH --> AGG
  CORE_START_FINAL --> AGG
  AGG --> AGG_CI_TEST --> MERGE_BACK --> TASKS

  %% ======== Feature end cleanup ========
  CLEANUP["FZ-feature-cleanup (make triad-feature-cleanup; remove worktrees; optional prune branches)"]
  TASKS --> CLEANUP
```

## Diagram B — WSL Bundled by Default (with “Separate WSL Task” Exception)

Default:
- WSL coverage is bundled into `X-integ-linux` by dispatching Linux smoke with `--run-wsl`.

Exception:
- Create `X-integ-wsl` only when the rubric in `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md` justifies separate ownership.

```mermaid
flowchart TD
  %% ======== Inputs / gating ========
  ADR["ADR accepted (Executive Summary drift guard OK)"]
  PACK["Planning Pack ready (plan/tasks/specs/prompts)"]
  GATE["Quality Gate (RECOMMENDATION: ACCEPT)"]

  ADR --> PACK --> GATE

  %% ======== Triad execution ========
  subgraph ORCH["Orchestration Branch (docs edits only)"]
    ORCH_ENSURE["Ensure orchestration branch exists (make triad-orch-ensure)"]
    START["Pick triad slice: X (e.g., C3)"]
    TASKS["tasks.json statuses (START/END in session_log.md)"]
    START_PAIR["Start X-code + X-test in parallel (make triad-task-start-pair SLICE_ID=X; optional LAUNCH_CODEX=1)"]
  end

  GATE --> ORCH_ENSURE --> START --> TASKS --> START_PAIR

  subgraph WT["Parallel Worktrees (no docs edits)"]
    CODE["X-code (prod code only; created via make triad-task-start-pair; writes .taskmeta.json)"]
    TEST["X-test (tests only; created via make triad-task-start-pair; writes .taskmeta.json)"]
  end

  START_PAIR --> CODE
  START_PAIR --> TEST

  subgraph INTEG_CORE["Core Integration (primary dev platform)"]
    CORE["X-integ-core (merge X-code + X-test; resolve spec drift)"]
    CORE_CHECKS["Core checks: cargo fmt; cargo clippy ... -- -D warnings; relevant tests; make integ-checks"]
    CORE_LOCAL_SMOKE["Local behavioral smoke preflight (build substrate; run feature-local smoke script for this platform)"]
    CORE_PARITY["Dispatch cross-platform compile parity via make ci-compile-parity (GitHub-hosted; CI parity platforms)"]
    CORE_FIX["If parity fails: fix compile parity on X-integ-core branch (cfg/platform guards), commit, and re-run parity"]
    CORE_DISPATCH["Dispatch behavioral smoke via CI (prefer PLATFORM=behavior; optional WSL; WORKFLOW_REF should be the orchestration/task ref, not main/testing)"]
    CORE_RESULTS["Wait for smoke results (self-hosted runners)"]
    CORE_CI_TEST["Dispatch CI Testing (mode=quick) via scripts/ci/dispatch_ci_testing.sh (throwaway branch at HEAD)"]
    CORE_IDENTIFY["Identify failing platforms from smoke + CI Testing results"]
    CORE_START_PF["Start failing platform-fix tasks (use from-smoke-run when smoke was a single dispatch; else explicit PLATFORMS=...)"]
    CORE_START_FINAL["After fixes are green: start final aggregator (make triad-task-start-integ-final SLICE_ID=X)"]
  end

  CODE --> CORE
  TEST --> CORE
  CORE --> CORE_CHECKS --> CORE_LOCAL_SMOKE --> CORE_PARITY
  CORE_PARITY -->|pass| CORE_DISPATCH
  CORE_PARITY -->|fail| CORE_FIX --> CORE_PARITY

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE["Feature Smoke workflow (self-hosted runners; behavior platforms; optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE --> CORE_RESULTS --> CORE_CI_TEST --> CORE_IDENTIFY --> CORE_START_PF

  %% ======== Platform-fix tasks (parallel, only if needed) ========
  subgraph PF["Platform-fix Integration Tasks (only make changes if a platform fails)"]
    LNX["X-integ-linux (Linux platform-fix; default includes WSL via --run-wsl)"]
    MAC["X-integ-macos (macOS platform-fix)"]
    WIN["X-integ-windows (Windows platform-fix)"]
    WSL["X-integ-wsl (rare; WSL-only platform-fix; use only when rubric says so)"]
  end

  CORE_START_PF --> LNX
  CORE_START_PF --> MAC
  CORE_START_PF --> WIN
  CORE_START_PF --> WSL

  %% ======== Final aggregator ========
  subgraph INTEG_FINAL["Final Cross-Platform Integration"]
    FINAL["X-integ (final; merge platform fixes; run integ checks; re-run smoke (all; optional WSL))"]
    FINAL_CI_TEST["Dispatch CI Testing (throwaway branch at HEAD)"]
    MERGE_BACK["Merge back to orchestration branch; update tasks.json/session_log.md (worktrees retained; cleanup at feature end)"]
  end

  LNX --> FINAL
  MAC --> FINAL
  WIN --> FINAL
  WSL --> FINAL
  CORE_DISPATCH --> FINAL
  CORE_START_FINAL --> FINAL
  FINAL --> FINAL_CI_TEST --> MERGE_BACK --> TASKS

  %% ======== Feature end cleanup ========
  CLEANUP["FZ-feature-cleanup (make triad-feature-cleanup; remove worktrees; optional prune branches)"]
  TASKS --> CLEANUP
```

## Diagram C — Task Execution Automation (Zoom In)

This diagram zooms in on the automation pieces used during execution:
- orchestration branch bootstrap (`make triad-orch-ensure`)
- code+test parallel worktree creation (`make triad-task-start-pair`)
- failing platform-fix worktree creation (`make triad-task-start-platform-fixes-from-smoke`)
- single-task worktree creation + registration (`make triad-task-start`)
- optional headless Codex launch (from inside the worktree; kickoff prompt via stdin)
- task finishing (`make triad-task-finish`)
- feature-level cleanup (worktree retention model; `make triad-feature-cleanup`)

```mermaid
flowchart TD
  %% ======== Orchestration setup ========
  subgraph ORCH["Orchestration worktree (docs edits only)"]
    ENSURE["make triad-orch-ensure (ensures orchestration branch exists, checks out, ff-only pull)"]
    DOCS_START["docs: set task status in tasks.json; START entry in session_log.md; commit"]
    START_PAIR_RUN["make triad-task-start-pair FEATURE_DIR=... SLICE_ID=X (code+test; optional LAUNCH_CODEX=1)"]
    START_PF_RUN["make triad-task-start-platform-fixes-from-smoke FEATURE_DIR=... SLICE_ID=X SMOKE_RUN_ID=<run-id> (optional LAUNCH_CODEX=1)"]
    START_FINAL_RUN["make triad-task-start-integ-final FEATURE_DIR=... SLICE_ID=X (optional LAUNCH_CODEX=1)"]
    START_RUN["make triad-task-start FEATURE_DIR=... TASK_ID=... (single task; optional LAUNCH_CODEX=1)"]
    DOCS_END["docs: update tasks.json; END entry in session_log.md; commit"]
  end

  %% ======== task_start internals ========
  subgraph START_INT["scripts/triad/task_start.sh (automation)"]
    START_PARSE["parse tasks.json (requires schema_version>=3 and meta.automation.enabled=true)"]
    START_WT["git worktree add (create branch/worktree if needed)"]
    START_META["write .taskmeta.json in worktree root"]
    START_REG["update registry at <git-common-dir>/triad/features/<feature>/worktrees.json"]
    START_STDOUT["stdout contract: WORKTREE=..., TASK_BRANCH=..., ORCH_BRANCH=..., KICKOFF_PROMPT=..., NEXT=..."]
  end

  %% ======== task_start_pair internals ========
  subgraph START_PAIR_INT["scripts/triad/task_start_pair.sh (automation)"]
    PAIR_PARSE["parse tasks.json (requires schema_version>=3 and meta.automation.enabled=true)"]
    PAIR_VALIDATE["validate code/test types + concurrent_with linkage + deps completed"]
    PAIR_CALLS["call task_start.sh twice (code then test)"]
    PAIR_CODEX["optional: launch codex exec for both tasks (in parallel)"]
    PAIR_STDOUT["stdout contract: CODE_WORKTREE=..., TEST_WORKTREE=..., ..."]
  end

  %% ======== task_start_platform_fixes internals ========
  subgraph START_PF_INT["scripts/triad/task_start_platform_fixes.sh (automation)"]
    PF_PARSE["parse tasks.json (requires schema_version>=3 and meta.automation.enabled=true)"]
    PF_SELECT["select X-integ-<platform> tasks for failing platforms"]
    PF_CALLS["call task_start.sh for each platform task (sequential setup)"]
    PF_CODEX["optional: launch codex exec for all selected tasks (in parallel)"]
    PF_STDOUT["stdout contract: repeated PLATFORM/TASK_ID/WORKTREE/TASK_BRANCH/CODEX_EXIT"]
  end

  %% ======== task_start_integ_final internals ========
  subgraph START_FINAL_INT["scripts/triad/task_start_integ_final.sh (automation)"]
    FINAL_PARSE["parse tasks.json (requires schema_version>=3 and meta.automation.enabled=true)"]
    FINAL_VALIDATE["require X-integ exists and merge_to_orchestration=true"]
    FINAL_DEPS["require all X-integ depends_on tasks are completed"]
    FINAL_START["call task_start.sh for X-integ (optional codex launch)"]
  end

  %% ======== Worktree execution ========
  subgraph WT["Task worktree (no planning doc edits)"]
    CODEX_OPT["optional: codex exec (headless) from worktree using kickoff prompt via stdin; writes codex.pid + last_message.md"]
    DEV["agent work happens here (code/test/integ)"]
    FINISH_RUN["make triad-task-finish TASK_ID=... (optional: SMOKE=1 TASK_PLATFORM=...; do not also run make feature-smoke)"]
  end

  %% ======== task_finish internals ========
  subgraph FINISH_INT["scripts/triad/task_finish.sh (automation)"]
    FINISH_META["require .taskmeta.json and task_id match"]
    FINISH_GUARDS["guardrails: refuse planning doc edits; integration merge-back preserves orchestration Planning Pack files"]
    FINISH_CHECKS["run required_make_targets (or verify-only)"]
    FINISH_COMMIT["commit changes to task branch (unless --no-commit)"]
    FINISH_MERGE["if integration task: merge task branch -> orchestration branch (preserve orchestration Planning Pack files)"]
    FINISH_STDOUT["stdout contract: TASK_BRANCH=..., WORKTREE=..., HEAD=..., COMMITS=..., CHECKS=..., SMOKE_RUN=..."]
  end

  %% ======== Feature cleanup (retention model) ========
  subgraph CLEAN["Feature end (cleanup task)"]
    CLEANUP["make triad-feature-cleanup FEATURE_DIR=... REMOVE_WORKTREES=1 PRUNE_LOCAL=1 (optional PRUNE_REMOTE=origin; optional FORCE=1)"]
    CLEAN_INT["scripts/triad/feature_cleanup.sh removes worktrees using registry and prunes branches (with safety checks)"]
  end

  ENSURE --> DOCS_START
  DOCS_START --> START_PAIR_RUN --> PAIR_PARSE --> PAIR_VALIDATE --> PAIR_CALLS --> PAIR_CODEX --> PAIR_STDOUT
  DOCS_START --> START_RUN --> START_PARSE --> START_WT --> START_META --> START_REG --> START_STDOUT
  DOCS_START --> START_PF_RUN --> PF_PARSE --> PF_SELECT --> PF_CALLS --> PF_CODEX --> PF_STDOUT
  DOCS_START --> START_FINAL_RUN --> FINAL_PARSE --> FINAL_VALIDATE --> FINAL_DEPS --> FINAL_START
  START_STDOUT --> CODEX_OPT --> DEV --> FINISH_RUN
  FINISH_RUN --> FINISH_META --> FINISH_GUARDS --> FINISH_CHECKS --> FINISH_COMMIT --> FINISH_MERGE --> FINISH_STDOUT
  FINISH_STDOUT --> DOCS_END
  DOCS_END --> CLEANUP --> CLEAN_INT
```
