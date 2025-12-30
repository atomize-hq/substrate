# Triad Workflow (Cross-Platform Integration via Self-Hosted CI)

This diagram is the “execution phase” complement to:
- `docs/project_management/standards/PLANNING_WORKFLOW_OVERVIEW.md`

It shows:
- code/test running in parallel worktrees,
- a core integration merge task (`X-integ-core`),
- parallel platform-fix integration tasks (`X-integ-<platform>`) that only make changes if a platform fails, and
- a final cross-platform integration aggregator (`X-integ`) that merges platform fixes and re-validates.

This file includes two diagrams:
- **Diagram A:** The overall cross-platform platform-fix triad flow.
- **Diagram B:** The same flow with explicit emphasis on **WSL bundled-by-default** behavior.

## Diagram A — Cross-Platform Platform-Fix Triad Flow

```mermaid
flowchart TD
  %% ======== Inputs / gating ========
  ADR["ADR accepted<br/>Executive Summary drift guard OK"]
  PACK["Planning Pack ready<br/>(plan/tasks/specs/prompts)"]
  GATE["Quality Gate<br/>RECOMMENDATION: ACCEPT"]

  ADR --> PACK --> GATE

  %% ======== Triad execution ========
  subgraph ORCH["Orchestration Branch (docs edits only)"]
    START["Pick triad slice: X<br/>(e.g., C3)"]
    TASKS["tasks.json statuses<br/>START/END in session_log.md"]
  end

  GATE --> START
  START --> TASKS

  subgraph WT["Parallel Worktrees (no docs edits)"]
    CODE["X-code<br/>(prod code only)<br/>branch + worktree"]
    TEST["X-test<br/>(tests only)<br/>branch + worktree"]
  end

  TASKS --> CODE
  TASKS --> TEST

  subgraph INTEG_CORE["Core Integration (primary dev platform)"]
    MERGE["X-integ-core<br/>merge X-code + X-test<br/>resolve spec drift"]
    CORE_CHECKS["Required checks:<br/>- cargo fmt<br/>- cargo clippy ... -- -D warnings<br/>- relevant tests<br/>- make integ-checks"]
    CORE_DISPATCH["Dispatch cross-platform smoke<br/>via make feature-smoke<br/>PLATFORM=all (+ optional RUN_WSL=1)"]
  end

  CODE --> MERGE
  TEST --> MERGE
  MERGE --> CORE_CHECKS
  CORE_CHECKS --> CORE_DISPATCH

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE_ALL["Feature Smoke workflow<br/>(self-hosted runners)<br/>linux + macos + windows (+ optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE_ALL

  %% ======== Platform-fix tasks (parallel, only if needed) ========
  subgraph PF["Platform-fix Integration Tasks (worktrees on platform machines)"]
    LNX["X-integ-linux<br/>If Linux fails: fix in Linux worktree<br/>Default: includes WSL coverage via --run-wsl<br/>re-run smoke until green"]
    MAC["X-integ-macos<br/>If macOS fails: fix in macOS worktree<br/>re-run smoke until green"]
    WIN["X-integ-windows<br/>If Windows fails: fix in Windows worktree<br/>re-run smoke until green"]
    WSL["X-integ-wsl (rare)<br/>If WSL fails and you want independent ownership<br/>fix in WSL worktree + re-run smoke until green"]
  end

  SMOKE_ALL --> LNX
  SMOKE_ALL --> MAC
  SMOKE_ALL --> WIN
  SMOKE_ALL --> WSL

  %% ======== Final aggregator ========
  subgraph INTEG_FINAL["Final Cross-Platform Integration"]
    AGG["X-integ (final)<br/>merge platform-fix branches (if any)<br/>run integ checks + re-run cross-platform smoke"]
    MERGE_BACK["Fast-forward merge to orchestration branch<br/>update tasks.json/session_log.md<br/>(remove worktrees)"]
  end

  LNX --> AGG
  MAC --> AGG
  WIN --> AGG
  WSL --> AGG
  CORE_DISPATCH --> AGG
  AGG --> MERGE_BACK --> TASKS
```

## Diagram B — WSL Bundled by Default (with “Separate WSL Task” Exception)

Default:
- WSL coverage is bundled into `X-integ-linux` by dispatching Linux smoke with `--run-wsl`.

Exception:
- Create `X-integ-wsl` only when the rubric in `docs/project_management/standards/PLATFORM_INTEGRATION_AND_CI.md` justifies separate ownership.

```mermaid
flowchart TD
  %% ======== Inputs / gating ========
  ADR["ADR accepted<br/>Executive Summary drift guard OK"]
  PACK["Planning Pack ready<br/>(plan/tasks/specs/prompts)"]
  GATE["Quality Gate<br/>RECOMMENDATION: ACCEPT"]

  ADR --> PACK --> GATE

  %% ======== Triad execution ========
  subgraph ORCH["Orchestration Branch (docs edits only)"]
    START["Pick triad slice: X<br/>(e.g., C3)"]
    TASKS["tasks.json statuses<br/>START/END in session_log.md"]
  end

  GATE --> START --> TASKS

  subgraph WT["Parallel Worktrees (no docs edits)"]
    CODE["X-code<br/>(prod code only)<br/>branch + worktree"]
    TEST["X-test<br/>(tests only)<br/>branch + worktree"]
  end

  TASKS --> CODE
  TASKS --> TEST

  subgraph INTEG_CORE["Core Integration (primary dev platform)"]
    CORE["X-integ-core<br/>merge X-code + X-test<br/>resolve spec drift"]
    CORE_CHECKS["Core checks<br/>- cargo fmt<br/>- cargo clippy ... -- -D warnings<br/>- relevant tests<br/>- make integ-checks"]
    CORE_DISPATCH["Dispatch smoke via CI<br/>platform=all (+ optional WSL)"]
  end

  CODE --> CORE
  TEST --> CORE
  CORE --> CORE_CHECKS --> CORE_DISPATCH

  %% ======== Smoke validation (CI) ========
  subgraph CI["GitHub Actions (validation only)"]
    SMOKE["Feature Smoke workflow<br/>self-hosted runners<br/>linux + macos + windows (+ optional WSL)"]
  end

  CORE_DISPATCH --> SMOKE

  %% ======== Platform-fix tasks (parallel, only if needed) ========
  subgraph PF["Platform-fix Integration Tasks (only make changes if a platform fails)"]
    LNX["X-integ-linux<br/>Linux platform-fix<br/>Default: includes WSL coverage via --run-wsl"]
    MAC["X-integ-macos<br/>macOS platform-fix"]
    WIN["X-integ-windows<br/>Windows platform-fix"]
    WSL["X-integ-wsl (rare)<br/>WSL-only platform-fix<br/>Use only when rubric says so"]
  end

  SMOKE --> LNX
  SMOKE --> MAC
  SMOKE --> WIN
  SMOKE --> WSL

  %% ======== Final aggregator ========
  subgraph INTEG_FINAL["Final Cross-Platform Integration"]
    FINAL["X-integ (final)<br/>merge any platform-fix branches<br/>run integ checks + re-run smoke (all + optional WSL)"]
    MERGE_BACK["Fast-forward merge to orchestration branch<br/>update tasks.json/session_log.md<br/>(remove worktrees)"]
  end

  LNX --> FINAL
  MAC --> FINAL
  WIN --> FINAL
  WSL --> FINAL
  CORE_DISPATCH --> FINAL
  FINAL --> MERGE_BACK --> TASKS
```
