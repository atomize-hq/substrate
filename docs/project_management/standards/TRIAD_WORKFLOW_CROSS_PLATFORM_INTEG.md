# Triad Workflow (Cross-Platform Integration via Self-Hosted CI)

This diagram is the “execution phase” complement to:
- `docs/project_management/standards/PLANNING_WORKFLOW_OVERVIEW.md`

It shows:
- code/test running in parallel worktrees,
- a core integration merge task,
- platform-specific integration smoke tasks running in parallel on self-hosted runners via GitHub Actions, and
- a final cross-platform integration aggregator that merges + records results.

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

  subgraph INTEG_CORE["Core Integration (host integration worktree)"]
    MERGE["X-integ-core<br/>merge X-code + X-test<br/>resolve spec drift"]
    CORE_CHECKS["Required checks:<br/>- cargo fmt<br/>- cargo clippy ... -- -D warnings<br/>- relevant tests<br/>- make integ-checks"]
  end

  CODE --> MERGE
  TEST --> MERGE
  MERGE --> CORE_CHECKS

  %% ======== Cross-platform smoke (parallel) ========
  subgraph CI["GitHub Actions (self-hosted runners)"]
    TMPBR["Create throwaway remote branch<br/>(tmp/feature-smoke/...)<br/>from X-integ-core SHA"]
    DISPATCH["Dispatch workflow from stable ref<br/>(ref: feat/policy_and_config)<br/>inputs: checkout_ref=tmp branch<br/>runner_kind=self-hosted"]

    LNX["X-integ-linux<br/>Feature Smoke: platform=linux<br/>runs-on: self-hosted / Linux / linux-host"]
    MAC["X-integ-macos<br/>Feature Smoke: platform=macos<br/>runs-on: self-hosted / macOS"]
    WIN["X-integ-windows<br/>Feature Smoke: platform=windows<br/>runs-on: self-hosted / Windows"]
    WSL["X-integ-wsl (optional)<br/>Feature Smoke: run_wsl=true<br/>runs-on: self-hosted / Linux / wsl"]
  end

  CORE_CHECKS --> TMPBR --> DISPATCH
  DISPATCH --> LNX
  DISPATCH --> MAC
  DISPATCH --> WIN
  DISPATCH --> WSL

  %% If a platform smoke fails, fix in core integration then re-dispatch
  LNX -.->|fail| MERGE
  MAC -.->|fail| MERGE
  WIN -.->|fail| MERGE
  WSL -.->|fail| MERGE

  %% ======== Final aggregator ========
  subgraph INTEG_FINAL["Final Cross-Platform Integration"]
    AGG["X-integ (final)<br/>wait for required platform smoke<br/>record run ids/URLs in session_log.md"]
    MERGE_BACK["Fast-forward merge to orchestration branch<br/>update tasks.json/session_log.md<br/>(remove worktrees)"]
  end

  LNX --> AGG
  MAC --> AGG
  WIN --> AGG
  WSL --> AGG
  CORE_CHECKS --> AGG
  AGG --> MERGE_BACK --> TASKS
```
