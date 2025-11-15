# AI Agent Entry Point – Substrate Isolated Shell Initiative

Welcome! This repository uses a multi-agent workflow to deliver the isolated-shell/manager-init/world-deps project. Every execution (code/test/integration) must follow the steps below to ensure consistency, traceability, and fast concurrent delivery.

---

## 1. Repository Orientation

- Root: `/home/spenser/__Active_code/substrate`
- Planning docs: `docs/project_management/next/`
  - `substrate_isolated_shell_plan.md`
  - `substrate_isolated_shell_file_audit.md`
  - `substrate_isolated_shell_data_map.md`
  - `substrate_isolated_shell_dependency_graph.md`
  - `substrate_isolated_shell_execution_plan.md`
  - `tasks.json` (authoritative task list/status/commands)
  - `session_log.md` (running log for every agent session)
- Source crates/scripts referenced per file audit.

Always read the relevant planning document(s) before editing code or tests.

---

## 2. Task Selection

1. Open `docs/project_management/next/tasks.json`.
2. Find the earliest task with `"status": "pending"` that you are authorized to perform (code/test/integration).
3. Respect dependencies listed in `depends_on`. Do not start a task until its prerequisites are `"completed"`.
4. Record your intent by updating the task’s `status` to `"in_progress"` (and note it in `session_log.md`).

> **Note:** Tasks are grouped by phase (see execution plan). Parallelism is encouraged where `concurrent_with` lists other tasks.

---

## 3. Session Log Procedure

File: `docs/project_management/next/session_log.md`

At the **start** of every task:
1. Read the entire log to understand context.
2. Append an entry announcing you’re starting the task.

At the **end**:
1. Append a completion note summarizing work done, tests executed, and follow-ups.
2. Update the task’s status in `tasks.json` (`completed` or `blocked`).

### Log Entry Template
```
## [YYYY-MM-DD HH:MM UTC] <agent name> – <task id> – START/END
- Summary / intent or results
- Commands run (if END)
- Next steps / blockers
```

---

## 4. Execution Flow per Task Type

### Code Tasks
1. Select task (see §2) and set status to `in_progress`.
2. Gather requirements:
   - Read planning docs + file audit entries referenced in task kickoff prompt.
   - Inspect related files.
3. **Before writing code**, craft a “Test Agent Kickoff Prompt” for the paired test task:
   - Include task id, summary of new behavior, files touched, commands to run, and remind the test agent to start from this document.
   - Provide this prompt to the user (they will spin up the test agent).
4. Execute the code changes in the designated worktree (`worktree` field).
5. Run the commands listed in the task’s `kickoff_prompt`.
6. Document everything in `session_log.md`, update `tasks.json` status to `completed`.

### Test Tasks
1. Start from this file, read session log and planning docs.
2. Consume the “Test Agent Kickoff Prompt” produced by the code task.
3. Implement tests in the specified worktree.
4. Run the required commands.
5. **Before finishing**, craft an “Integration Agent Kickoff Prompt” describing how to merge code+tests and which verification commands/logs to capture.
6. Update session log + task status.

### Integration Tasks
1. Read both code/test worktrees and session log.
2. Use the Integration kickoff prompt supplied by the test task.
3. Merge worktrees (named in `tasks.json`), resolve conflicts, run commands, capture results.
4. Record completion in session log, mark task status `completed`.

---

## 5. Worktrees & Commands

- Each task specifies a `worktree` name. Create it once using `git worktree add ../<name> <branch>` if not existing.
- Always run commands from the repo root unless instructed otherwise.
- Avoid touching unrelated files; follow file audit strictly.

---

## 6. Communication & Prompts

- When prompted to provide a kickoff prompt (for test or integration agents), format it as plain text with:
  - Task ID + type
  - Summary of implemented behavior
  - Files/directories touched or to be tested
  - Commands to execute
  - Reminder to read this AI entry file first and update session log
  - Related documentation references

This keeps the workflow mostly autonomous—only the initial user prompt is required.

---

## 7. Status Lifecycle

`tasks.json` uses:
- `pending`
- `in_progress`
- `blocked`
- `completed`

Only advance statuses forward; if blocked, document why in session log and revert `status` to `blocked`.

---

Follow these steps for every task. This ensures consistent context sharing, auditable progress, and predictable hand-offs between code, test, and integration agents. Good luck!***
