# Triad Workflow (Cross-Platform Integration via Self-Hosted CI)

This diagram is the “execution phase” complement to:
- `docs/project_management/standards/PLANNING_WORKFLOW_OVERVIEW.md`

It shows:
- code/test running in parallel worktrees (created via the triad task runner),
- a core integration merge task (`X-integ-core`),
- parallel platform-fix integration tasks (`X-integ-<platform>`) that only make changes if a platform fails, and
- a final cross-platform integration aggregator (`X-integ`) that merges platform fixes and re-validates.
- worktrees retained through the feature and removed only by the feature cleanup task (`FZ-feature-cleanup`).

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
    CORE_DISPATCH["Dispatch cross-platform smoke via make feature-smoke (PLATFORM=all; optional RUN_WSL=1; WORKFLOW_REF=ORCH_BRANCH)"]
    CORE_RESULTS["Wait for smoke results (self-hosted runners)"]
    CORE_IDENTIFY["Identify failing platforms from runner results"]
    CORE_START_PF["Start failing platform-fix tasks (make triad-task-start-platform-fixes-from-smoke SMOKE_RUN_ID=...)"]
    CORE_START_FINAL["After fixes are green: start final aggregator (make triad-task-start-integ-final SLICE_ID=X)"]
  end

  CODE --> MERGE
  TEST --> MERGE
  MERGE --> CORE_CHECKS --> CORE_DISPATCH

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE_ALL["Feature Smoke workflow (self-hosted runners; linux + macos + windows; optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE_ALL --> CORE_RESULTS --> CORE_IDENTIFY --> CORE_START_PF

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
    MERGE_BACK["Fast-forward merge to orchestration branch; update tasks.json/session_log.md (worktrees retained; cleanup at feature end)"]
  end

  LNX --> AGG
  MAC --> AGG
  WIN --> AGG
  WSL --> AGG
  CORE_DISPATCH --> AGG
  CORE_START_FINAL --> AGG
  AGG --> MERGE_BACK --> TASKS

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
    CORE_DISPATCH["Dispatch smoke via CI (platform=all; optional WSL; WORKFLOW_REF=ORCH_BRANCH)"]
    CORE_RESULTS["Wait for smoke results (self-hosted runners)"]
    CORE_IDENTIFY["Identify failing platforms from runner results"]
    CORE_START_PF["Start failing platform-fix tasks (make triad-task-start-platform-fixes-from-smoke SMOKE_RUN_ID=...)"]
    CORE_START_FINAL["After fixes are green: start final aggregator (make triad-task-start-integ-final SLICE_ID=X)"]
  end

  CODE --> CORE
  TEST --> CORE
  CORE --> CORE_CHECKS --> CORE_DISPATCH

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE["Feature Smoke workflow (self-hosted runners; linux + macos + windows; optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE --> CORE_RESULTS --> CORE_IDENTIFY --> CORE_START_PF

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
    MERGE_BACK["Fast-forward merge to orchestration branch; update tasks.json/session_log.md (worktrees retained; cleanup at feature end)"]
  end

  LNX --> FINAL
  MAC --> FINAL
  WIN --> FINAL
  WSL --> FINAL
  CORE_DISPATCH --> FINAL
  CORE_START_FINAL --> FINAL
  FINAL --> MERGE_BACK --> TASKS

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
    CODEX_OPT["optional: codex exec (headless) from worktree using kickoff prompt via stdin; writes last_message file"]
    DEV["agent work happens here (code/test/integ)"]
    FINISH_RUN["make triad-task-finish TASK_ID=... (optional: SMOKE=1 TASK_PLATFORM=...; do not also run make feature-smoke)"]
  end

  %% ======== task_finish internals ========
  subgraph FINISH_INT["scripts/triad/task_finish.sh (automation)"]
    FINISH_META["require .taskmeta.json and task_id match"]
    FINISH_GUARDS["guardrails: refuse planning doc edits; integration merge is FF-only"]
    FINISH_CHECKS["run required_make_targets (or verify-only)"]
    FINISH_COMMIT["commit changes to task branch (unless --no-commit)"]
    FINISH_MERGE["if integration task: fast-forward merge task branch -> orchestration branch"]
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
