# JSON Mode – Session Log

Use the same START/END template as other programs. Include:
- UTC timestamp, agent role, task ID.
- Commands executed (fmt/clippy/tests/scripts) with pass/fail notes.
- Worktree/commit references.
- Kickoff prompts authored.

Template:
```
## [YYYY-MM-DD HH:MM UTC] <Agent> – <task-id> – START
- Checked out feat/json-mode, pulled latest
- Updated tasks.json + session log (commit: <hash>)
- Created worktree: wt/<...>
- Plan: <steps/commands>
- Blockers: <none or description>

## [YYYY-MM-DD HH:MM UTC] <Agent> – <task-id> – END
- Worktree commits: <hash(es)>
- Commands: <cargo fmt / cargo clippy / tests / scripts>
- Results: <pass/fail notes, skips with reasons>
- Kickoff prompts created: <paths or “n/a”>
- Docs commit: <hash> (tasks/log updates)
- Next steps / blockers: <handoff notes>
```
