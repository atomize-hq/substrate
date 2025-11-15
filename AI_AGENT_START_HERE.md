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

## 2. Task Selection & Coordination Branch

- All coordination artifacts (`tasks.json`, `session_log.md`, kickoff prompts) live on the `feat/isolated-shell-plan` branch. Perform coordination there before switching to per-task worktrees.

Steps:
1. Ensure the main worktree is on `feat/isolated-shell-plan` (`git checkout feat/isolated-shell-plan` if needed).
2. Open `docs/project_management/next/tasks.json`.
3. Find the earliest task with `"status": "pending"` that you are authorized to perform (code/test/integration).
4. Respect dependencies listed in `depends_on`. Do not start a task until its prerequisites are `"completed"`.
5. Update the task’s `status` to `"in_progress"` **while on `feat/isolated-shell-plan`**.
6. Append a START entry to `docs/project_management/next/session_log.md` (also on `feat/isolated-shell-plan`).
7. Only after the above should you switch to the specific git worktree noted in the task.

> **Note:** Tasks are grouped by phase (see execution plan). Parallelism is encouraged where `concurrent_with` lists other tasks.

---

## 3. Session Log Procedure (coordination only)

File: `docs/project_management/next/session_log.md` (maintained on `feat/isolated-shell-plan`)

At the **start** of every task:
1. While on `feat/isolated-shell-plan`, read the log to understand context.
2. Append a START entry (using the template below) documenting the task you’re beginning.

At the **end**:
1. After finishing the work in the dedicated worktree, return to `feat/isolated-shell-plan`.
2. Append an END entry summarizing results/tests and any follow-ups.
3. Update the task’s status in `tasks.json` (`completed` or `blocked`).

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
1. On `feat/isolated-shell-plan`, set status to `in_progress` and log START (see §2–3).
2. Gather requirements (planning docs, file audit entries, dependency notes).
3. Switch into the code worktree (`worktree` field) and implement changes. Do **not** modify `tasks.json` or `session_log.md` from within that worktree.
4. **Before writing code**, craft the Test Agent Kickoff Prompt (task id, summary, files touched, commands, reminder to read this doc). Save it locally and be ready to record it when you return to the coordination branch.
5. Run the commands listed in the task’s prompt while still in the code worktree.
6. After coding/tests are complete, return to the main worktree on `feat/isolated-shell-plan`, append the END log entry, update task status to `completed`, and record the Test Agent Kickoff Prompt (e.g., include it in the session log).

### Test Tasks
1. On `feat/isolated-shell-plan`, read this doc, planning references, and session log. Set status to `in_progress`, log START.
2. Consume the Test Agent Kickoff Prompt left by the code task (session log).
3. Switch into the designated test worktree and implement tests there; keep coordination files untouched in that worktree.
4. Run the required commands/tests.
5. **Before finishing**, craft the Integration Agent Kickoff Prompt.
6. After tests pass, return to `feat/isolated-shell-plan`, record the Integration prompt + END log entry, set task status to `completed`.

### Integration Tasks
1. On `feat/isolated-shell-plan`, mark status `in_progress` and log START.
2. Read session log plus the Integration Kickoff Prompt provided by the test task.
3. Merge/resolve changes inside the integration worktree (or designated merge tree). Avoid editing coordination artifacts there.
4. Run the verification commands.
5. When integration worktree is clean and tests pass:
   - Merge its results back into the main coordination worktree/branch `feat/isolated-shell-plan`.
   - While on `feat/isolated-shell-plan`, append the END entry with results, update task status (`completed` or `blocked`), and commit any remaining coordination files (session log, tasks.json, kickoff prompts).
   - If the execution plan specifies a literal “next” task (e.g., the subsequent phase’s first code task), create/update the kickoff prompt for that task so the next agent can start immediately. Record that prompt location in the session log.

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
