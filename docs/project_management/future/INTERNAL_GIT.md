# PRD: Substrate Internal Versioning & Rollback via `.substrate-git`

## 1. Overview

Substrate needs a **cross-platform, fine-grained file history and rollback mechanism** for all agent and user commands that mutate the filesystem.

We will introduce an **internal Git repository** (`.substrate-git`) per workspace that:

* Tracks **per-command file changes**.
* Supports **command-, checkpoint-, and session-level rollback**.
* Periodically **squashes history** to avoid unbounded growth.
* Integrates with existing **JSONL command/trace logs**.

On Linux, this complements existing **OverlayFS-based isolation** (coarse-grained sandbox). On macOS/Windows, Git will be the primary mechanism for history and rollback.

---

## 2. Problem Statement

Today:

* We have **full logging** of every command (JSONL, traces, etc.).
* On Linux we already get **isolation** via overlay filesystems.
* We **do not** have a structured, portable way to:

  * See exactly which files changed per command.
  * Roll back those changes at a **command-level** or **task-level**.
  * Keep this history from growing without bound.

We need a **clean, deterministic, cross-platform versioning layer** that preserves Substrate’s “full traceability” story and enables powerful undo/rollback features.

---

## 3. Goals & Non-Goals

### Goals

1. **Per-command file history**

   * Every command that changes files results in a deterministic snapshot (commit).
   * We can answer: “What did command X change?”

2. **Rollback**

   * Ability to:

     * Undo the last command.
     * Roll back to a previous **checkpoint** (task completion).
     * Roll back to a previous **session state**.

3. **Multi-resolution history**

   * Keep fine-grained history for recent commands.
   * Squash older history into coarser **checkpoint-level** and **session-level** commits.

4. **Cross-platform**

   * Works identically on Linux, macOS, and Windows.
   * Linux can still keep OverlayFS for sandboxing; Git is the logical history layer.

5. **Non-invasive**

   * Does **not** interfere with the user’s real `.git` repo.
   * All Substrate history is internal and isolated.

### Non-Goals

* Replace user’s own Git workflows.
* Implement arbitrary multi-branch workflows in `.substrate-git` (we’ll assume a simple linear history for now).
* Provide remote push/pull or multi-machine synchronization from `.substrate-git` (future maybe).

---

## 4. Core Concepts

### 4.1 Internal Git Repo: `.substrate-git`

At workspace root:

```text
/project-root
  .git/                 # user’s repo (if present)
  .substrate-git/       # Substrate internal git dir
    .git/
  ...
```

* Substrate’s versioning lives entirely under `.substrate-git/.git`.

* The **work tree** for `.substrate-git` is the **project root** (`.`).

* Git commands are always invoked with explicit `--git-dir` and `--work-tree`:

  ```bash
  git --git-dir=.substrate-git/.git --work-tree=. <cmd> ...
  ```

* `.substrate-git/` is **ignored** by the user’s repo (via `.gitignore` or `.git/info/exclude`).

### 4.2 Units of History

1. **Command-level commit**

   * Smallest unit of change.
   * One commit per command that mutates the filesystem.
   * Includes only the paths that changed.

2. **Checkpoint**

   * A logical “task complete” marker while an agent is executing a long-running task.
   * Implemented by marking the latest command commit as a **checkpoint**.

3. **Session**

   * A logical grouping of commands and checkpoints under one Substrate session.
   * When a session closes, we mark the most recent commit as `session closed`.

4. **Squashed ranges**

   * Older command-level history is squashed:

     * **Between checkpoints** into checkpoint-level commits.
     * Across sessions into session-level commits.
   * Keeps history compact while still providing meaningful rollback points.

---

## 5. User Stories

1. **As a developer**, I want to see exactly what an agent changed during a single command so I can review and trust its edits.
2. **As a developer**, I want to undo just the last command’s changes without nuking the whole session.
3. **As a developer**, I want to roll my workspace back to a stable checkpoint after an agent finishes a complex refactor.
4. **As a developer**, after closing a session, I still want the ability to roll back to a “session snapshot” even if the detailed command history has been squashed.
5. **As a platform builder**, I want the internal history store to avoid unbounded growth and stay performant over time.

---

## 6. High-Level Design

### 6.1 Initialization

On first use in a workspace:

1. Detect workspace root (Substrate’s existing logic).
2. If `.substrate-git/.git` does not exist:

   * `mkdir -p .substrate-git`
   * `git --git-dir=.substrate-git/.git --work-tree=. init`
3. Ensure `.substrate-git/` is not tracked by the user’s Git:

   * Prefer adding it to `.git/info/exclude` for non-invasive behavior.

### 6.2 Command Execution Flow

For each executed command:

1. **Pre-check**:

   * Ensure `.substrate-git` initialized.
   * Optionally detect non-Substrate changes (dirty work tree from external tools):

     * Either auto-commit as “external changes” or mark in metadata.

2. **Execute command** (agent or user).

3. **Detect file changes**:

   * Prefer using Substrate’s knowledge of which paths the command wrote to.
   * Optionally verify via Git status:

     ```bash
     git --git-dir=.substrate-git/.git --work-tree=. status --porcelain
     ```

4. **If no changes:**

   * No commit; still log the command in JSONL.

5. **If changes:**

   * Stage changed paths:

     ```bash
     git --git-dir=.substrate-git/.git --work-tree=. add <paths...>
     ```

   * Commit:

     ```bash
     git --git-dir=.substrate-git/.git --work-tree=. commit -m "cmd: <session-id>/<command-id> <summary>"
     ```

   * Capture resulting commit hash.

6. **Log mapping** in JSONL trace:

   ```json
   {
     "ts": "2025-11-23T17:12:03Z",
     "session_id": "s-123",
     "command_id": "c-456",
     "command": "rg foo src && sed -i ...",
     "changed_files": ["src/foo.rs", "src/bar.rs"],
     "substrate_git_commit": "abc1234"
   }
   ```

### 6.3 Checkpoints

When an agent declares a logical task “done” (e.g. end of a long refactor):

1. Determine latest command commit `C` in that session.

2. Mark `C` as a **checkpoint**:

   * Via **tag**:

     ```bash
     git --git-dir=.substrate-git/.git tag cp/task/<session-id>/<seq> <C>
     ```

   * Commit message convention:

     * The commit already has `cmd: ...`; optional subsequent meta-commit is not required.
     * Alternatively, if we want a dedicated commit, we can add a no-op “checkpoint marker” commit.

3. Record checkpoint in internal metadata (e.g., a small DB or JSON index):

   ```json
   {
     "type": "checkpoint",
     "session_id": "s-123",
     "checkpoint_seq": 4,
     "commit": "abc1234"
   }
   ```

### 6.4 Session Closed

When a session ends:

1. Determine latest commit `C_session`.

2. Tag:

   ```bash
   git --git-dir=.substrate-git/.git tag cp/session/<session-id>/closed <C_session>
   ```

3. Optionally write a small “session closed” note/record.

---

## 7. Rollback Semantics

### 7.1 Undo Last Command

Given current session and last command with a `substrate_git_commit = C`:

1. Parent commit `P = C^`.

2. Restore changed files to their state at `P`:

   ```bash
   git --git-dir=.substrate-git/.git --work-tree=. restore --source=P -- <changed-files...>
   ```

3. Optionally create a new commit to record rollback:

   ```bash
   git --git-dir=.substrate-git/.git --work-tree=. commit -am "cmd: <session>/<command-id> rollback"
   ```

Or leave as uncommitted changes and only record in metadata.

### 7.2 Rollback to a Checkpoint

Given a checkpoint tag `cp/task/<session-id>/<seq>` → commit `C_cp`:

* **Option A: full tree rollback**

  ```bash
  git --git-dir=.substrate-git/.git --work-tree=. restore --source=C_cp -- .
  ```

* **Option B: subset rollback**

  * Use metadata to track which files were in-scope for that task and only restore those.

### 7.3 Rollback to a Session Snapshot

Given a session-closed tag `cp/session/<session-id>/closed` → commit `C_session`:

```bash
git --git-dir=.substrate-git/.git --work-tree=. restore --source=C_session -- .
```

This is our coarse-grained “take me back to how things were at the end of that session.”

---

## 8. History Compaction / Squashing

We want to prevent `.substrate-git` size from ballooning, while keeping useful rollback ability.

### 8.1 Policy

Configurable but sensible defaults, e.g.:

* **Keep per-command history** for:

  * Last **N checkpoints per session** (e.g. N = 3–5), and/or
  * Last **M sessions** (e.g. M = 3–5).
* For older ranges:

  * Squash **command-level** commits into one commit per checkpoint.
  * Squash **checkpoint-level** commits into one commit per session.

Consequence:

* Recent commands → full per-command rollback.
* Older history → only checkpoint/session rollback.

### 8.2 Checkpoint-Level Squash

Given a session with checkpoints: `C1`, `C2`, `C3`:

```text
C0 --- c1.1 --- c1.2 --- C1 --- c2.1 --- c2.2 --- C2 --- ...
 ^ parent                ^ cp1                    ^ cp2
```

To squash between `C1` and `C2`:

1. Determine:

   * `C_start` (e.g. C1)
   * `C_end` (e.g. C2)
   * `Parent = parent(C_start)` (or previous retained commit).
   * Final tree = tree at `C_end`.

2. Create new commit:

   * Parent: `Parent`
   * Tree: tree of `C_end`
   * Message: `cp: session <id> [C1..C2] squashed`

3. Move the main internal branch reference to include this new commit instead of the old chain.

4. Retag `C_end` checkpoint tag to point at the new squashed commit (or create a new checkpoint tag).

We can implement via Git plumbing (`git commit-tree` + `git update-ref`) or scripted `rebase`.

### 8.3 Session-Level Squash

For older sessions beyond retention window:

* For each session:

  * Determine closing commit `C_session_closed`.
  * Create a **single** commit representing that session’s final tree, with message:

    * `cp: session <session-id> squashed`
  * Retain or create tag:

    * `cp/session/<session-id>/squashed`.

We end up with a history like:

```text
[old sessions]: S1_squashed -- S2_squashed -- S3_squashed -- ...
[recent sessions]: detailed commits + checkpoints
```

### 8.4 Impact on Trace Metadata

After squash:

* Old `substrate_git_commit` hashes in JSONL may no longer refer to existing commits.
* That’s acceptable **by design**; we update the contract:

  * **Recent commands** (within retention window) → command-level rollback and exact diff.
  * **Older commands** → only guaranteed checkpoint/session-level rollback.

We should track which ranges have been squashed in a small index so the UI/CLI can reflect that accurately.

---

## 9. Edge Cases & Considerations

### 9.1 External Edits

* User edits files outside Substrate (editor, git checkout, etc.).
* When Substrate detects a dirty tree before running a command:

  * Option A: auto-commit `cmd: external-changes` with all changes staged.
  * Option B: refuse to run until user acknowledges / chooses behavior.
* This keeps history consistent and rollbackable.

### 9.2 Large Repos / Unwanted Paths

* Introduce an internal ignore mechanism (`.substrateignore` or config) to avoid:

  * `node_modules/`, `dist/`, build artifacts, binaries, etc.
* Ensure `.substrate-git` itself, `.git`, and other special dirs are excluded.

### 9.3 Cleanup / GC

* Periodic background or manual “compact” operation:

  * Run checkpoint/session squashing.
  * Run `git gc` to pack objects and prune unreachable commits.
* Expose metrics:

  * `.substrate-git` size.
  * Number of commits/sessions.
  * When compaction last ran.

---

## 10. API / Interface Sketch

### 10.1 Internal API (pseudo)

```ts
interface SubstrateGitStore {
  initRepo(workspaceRoot: string): Promise<void>;

  recordCommandResult(args: {
    workspaceRoot: string;
    sessionId: string;
    commandId: string;
    changedPaths: string[];
    summary: string;
  }): Promise<{ commit: string | null }>;

  markCheckpoint(args: {
    workspaceRoot: string;
    sessionId: string;
    checkpointId: string;
  }): Promise<{ commit: string }>;

  markSessionClosed(args: {
    workspaceRoot: string;
    sessionId: string;
  }): Promise<{ commit: string }>;

  rollbackLastCommand(args: {
    workspaceRoot: string;
    sessionId: string;
  }): Promise<void>;

  rollbackToCheckpoint(args: {
    workspaceRoot: string;
    sessionId: string;
    checkpointId: string;
  }): Promise<void>;

  rollbackToSession(args: {
    workspaceRoot: string;
    sessionId: string;
  }): Promise<void>;

  compactCheckpoints(args: {
    workspaceRoot: string;
    retentionCheckpointsPerSession: number;
  }): Promise<void>;

  compactSessions(args: {
    workspaceRoot: string;
    retentionSessions: number;
  }): Promise<void>;
}
```

All of these wrap the `git --git-dir=.substrate-git/.git --work-tree=.` calls and Git plumbing necessary.

---

## 11. Open Questions

1. **Retention defaults**

   * What are the default values for:

     * `retentionCheckpointsPerSession`?
     * `retentionSessions`?
   * Should these be per-workspace config or global?

2. **User-facing UX**

   * How much of this should be visible?

     * Commands like `substrate undo`, `substrate diff`, `substrate checkpoint list`?
   * Do we expose `.substrate-git` to power users, or treat it as internal only?

3. **Concurrency**

   * How do we handle multiple agents / sessions writing in the same workspace concurrently?
   * Likely include session/branch separation, or serialize writes to `.substrate-git`.

4. **Error handling**

   * If `.substrate-git` gets corrupted, what’s the recovery story?
   * Do we allow “reset history” for the internal repo?

5. **Linux overlay integration**

   * Exact behavior when both OverlayFS (coarse isolation) and `.substrate-git` (fine history) are active:

     * There’s no conflict conceptually, but we should document the order-of-operations and expected behavior when tearing down overlays.
