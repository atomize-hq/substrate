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
  end

  GATE --> ORCH_ENSURE --> START
  START --> TASKS

  subgraph WT["Parallel Worktrees (no docs edits)"]
    CODE["X-code (prod code only; make triad-task-start; writes .taskmeta.json)"]
    TEST["X-test (tests only; make triad-task-start; writes .taskmeta.json)"]
  end

  TASKS --> CODE
  TASKS --> TEST

  subgraph INTEG_CORE["Core Integration (primary dev platform)"]
    MERGE["X-integ-core (merge X-code + X-test; resolve spec drift)"]
    CORE_CHECKS["Required checks: cargo fmt; cargo clippy ... -- -D warnings; relevant tests; make integ-checks"]
    CORE_DISPATCH["Dispatch cross-platform smoke via make feature-smoke (PLATFORM=all; optional RUN_WSL=1)"]
  end

  CODE --> MERGE
  TEST --> MERGE
  MERGE --> CORE_CHECKS
  CORE_CHECKS --> CORE_DISPATCH

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE_ALL["Feature Smoke workflow (self-hosted runners; linux + macos + windows; optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE_ALL

  %% ======== Platform-fix tasks (parallel, only if needed) ========
  subgraph PF["Platform-fix Integration Tasks (worktrees on platform machines)"]
    LNX["X-integ-linux (if Linux fails: fix in Linux worktree; default includes WSL via --run-wsl; re-run smoke)"]
    MAC["X-integ-macos (if macOS fails: fix in macOS worktree; re-run smoke)"]
    WIN["X-integ-windows (if Windows fails: fix in Windows worktree; re-run smoke)"]
    WSL["X-integ-wsl (rare; if WSL fails and you want separate ownership; fix + re-run smoke)"]
  end

  SMOKE_ALL --> LNX
  SMOKE_ALL --> MAC
  SMOKE_ALL --> WIN
  SMOKE_ALL --> WSL

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
  end

  GATE --> ORCH_ENSURE --> START --> TASKS

  subgraph WT["Parallel Worktrees (no docs edits)"]
    CODE["X-code (prod code only; make triad-task-start; writes .taskmeta.json)"]
    TEST["X-test (tests only; make triad-task-start; writes .taskmeta.json)"]
  end

  TASKS --> CODE
  TASKS --> TEST

  subgraph INTEG_CORE["Core Integration (primary dev platform)"]
    CORE["X-integ-core (merge X-code + X-test; resolve spec drift)"]
    CORE_CHECKS["Core checks: cargo fmt; cargo clippy ... -- -D warnings; relevant tests; make integ-checks"]
    CORE_DISPATCH["Dispatch smoke via CI (platform=all; optional WSL)"]
  end

  CODE --> CORE
  TEST --> CORE
  CORE --> CORE_CHECKS --> CORE_DISPATCH

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE["Feature Smoke workflow (self-hosted runners; linux + macos + windows; optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE

  %% ======== Platform-fix tasks (parallel, only if needed) ========
  subgraph PF["Platform-fix Integration Tasks (only make changes if a platform fails)"]
    LNX["X-integ-linux (Linux platform-fix; default includes WSL via --run-wsl)"]
    MAC["X-integ-macos (macOS platform-fix)"]
    WIN["X-integ-windows (Windows platform-fix)"]
    WSL["X-integ-wsl (rare; WSL-only platform-fix; use only when rubric says so)"]
  end

  SMOKE --> LNX
  SMOKE --> MAC
  SMOKE --> WIN
  SMOKE --> WSL

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
  FINAL --> MERGE_BACK --> TASKS

  %% ======== Feature end cleanup ========
  CLEANUP["FZ-feature-cleanup (make triad-feature-cleanup; remove worktrees; optional prune branches)"]
  TASKS --> CLEANUP
```
