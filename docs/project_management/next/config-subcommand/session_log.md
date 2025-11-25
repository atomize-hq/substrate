# Config Subcommand – Session Log

Follow the workflow in `config_subcommand_plan.md`. Every entry must include:
- Timestamp (UTC), agent role (code/test/integ), and task ID.
- Commands executed (fmt/clippy/tests/scripts) with pass/fail notes.
- Commits/worktrees referenced.
- Kickoff prompts authored for downstream roles.

Template:
```
## [YYYY-MM-DD HH:MM UTC] <Agent> – <task-id> – START
- Checked out feat/config-subcommand, pulled latest
- Updated tasks.json + session log (commit: <hash>)
- Created worktree: wt/<...>
- Plan: <bullet list of intended actions/commands>
- Blockers: <none or description>

## [YYYY-MM-DD HH:MM UTC] <Agent> – <task-id> – END
- Worktree commits: <hash(es)>
- Commands: <cargo fmt / cargo clippy / tests / scripts>
- Results: <pass/fail summary, skips with justification>
- Kickoff prompts created: <paths or “n/a”>
- Docs commit: <hash> (tasks/session log updates)
- Next steps / blockers: <notes for next agent>
```
