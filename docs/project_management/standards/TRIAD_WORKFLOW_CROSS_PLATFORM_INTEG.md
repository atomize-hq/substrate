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
  ADR[ADR accepted\n+ Executive Summary drift guard OK]
  PACK[Planning Pack ready\n(plan/tasks/specs/prompts)]
  GATE[Quality Gate\nRECOMMENDATION: ACCEPT]

  ADR --> PACK --> GATE

  %% ======== Triad execution ========
  subgraph ORCH[Orchestration Branch (docs edits only)]
    START[Pick triad slice: X\n(e.g., C3)]
    TASKS[tasks.json statuses\nSTART/END in session_log.md]
  end

  GATE --> START
  START --> TASKS

  subgraph WT[Parallel Worktrees (no docs edits)]
    CODE[X-code\n(prod code only)\nbranch+worktree]
    TEST[X-test\n(tests only)\nbranch+worktree]
  end

  TASKS --> CODE
  TASKS --> TEST

  subgraph INTEG_CORE[Core Integration (host integration worktree)]
    MERGE[X-integ-core\nmerge X-code + X-test\nresolve spec drift]
    CORE_CHECKS[Required checks:\n- cargo fmt\n- cargo clippy ... -D warnings\n- relevant tests\n- make integ-checks]
  end

  CODE --> MERGE
  TEST --> MERGE
  MERGE --> CORE_CHECKS

  %% ======== Cross-platform smoke (parallel) ========
  subgraph CI[GitHub Actions (self-hosted runners)]
    TMPBR[Create throwaway remote branch\n(tmp/feature-smoke/…)\nfrom X-integ-core SHA]
    DISPATCH[Dispatch workflow from stable ref\n(ref: feat/policy_and_config)\ninputs: checkout_ref=tmp branch,\nrunner_kind=self-hosted]

    LNX[X-integ-linux\nFeature Smoke: platform=linux\nruns-on: [self-hosted, Linux, linux-host]]
    MAC[X-integ-macos\nFeature Smoke: platform=macos\nruns-on: [self-hosted, macOS]]
    WIN[X-integ-windows\nFeature Smoke: platform=windows\nruns-on: [self-hosted, Windows]]
    WSL[X-integ-wsl (optional)\nFeature Smoke: run_wsl=true\nruns-on: [self-hosted, Linux, wsl]]
  end

  CORE_CHECKS --> TMPBR --> DISPATCH
  DISPATCH --> LNX
  DISPATCH --> MAC
  DISPATCH --> WIN
  DISPATCH --> WSL

  %% If a platform smoke fails, fix in core integration then re-dispatch
  LNX -. fail -> MERGE
  MAC -. fail -> MERGE
  WIN -. fail -> MERGE
  WSL -. fail -> MERGE

  %% ======== Final aggregator ========
  subgraph INTEG_FINAL[Final Cross-Platform Integration]
    AGG[X-integ (final)\nwait for required platform smoke\nrecord run ids/URLs in session_log.md]
    MERGE_BACK[Fast-forward merge to orchestration branch\nupdate tasks.json/session_log.md\n(remove worktrees)]
  end

  LNX --> AGG
  MAC --> AGG
  WIN --> AGG
  WSL --> AGG
  CORE_CHECKS --> AGG
  AGG --> MERGE_BACK --> TASKS
```
