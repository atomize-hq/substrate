# Settings Stack – Session Log

Follow the workflow from `settings_stack_plan.md`. Every entry must include:
- Timestamp (UTC), agent role (code/test/integ), and task ID.
- Commands executed (fmt/clippy/tests/scripts).
- References to commits/worktrees touched.
- Kickoff prompt paths created for subsequent agents.

Template (copy/paste and fill in):
```
## [YYYY-MM-DD HH:MM UTC] <Agent> – <task-id> – START
- Checked out feat/settings-stack, pulled latest
- Updated tasks.json + session log (commit: <hash>)
- Created worktree: wt/<...>
- Plan: <bullet list of actions/commands>
- Blockers: <none or details>

## [YYYY-MM-DD HH:MM UTC] <Agent> – <task-id> – END
- Worktree commits: <hash(es)>
- Commands: <cargo fmt / cargo clippy / tests / scripts>
- Results: <pass/fail notes>
- Kickoff prompts created: <paths or “n/a”>
- Docs commit: <hash> (updated tasks + session log)
- Next steps / blockers: <notes for next agent>
```
