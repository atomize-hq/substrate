# Task A1-integ – Integration Agent Kickoff Prompt

Task ID: **A1-integ** (Integrate manifest parser worktrees)

Summary:
- Merge the code + test worktrees so `crates/common/src/manager_manifest.rs` and the new manifest parser tests land together.
- Ensure the manager manifest loader handles base + overlay YAML, env/tilde expansion, guest install metadata, and error paths (duplicate managers, invalid regexes, overlay version mismatches).
- Confirm the workspace picks up the new serde_yaml/regex dependencies and that the tests cover overlay merge, env expansion, and failure cases.

Focus files:
- `crates/common/src/manager_manifest.rs`
- `crates/common/src/lib.rs`
- `crates/common/Cargo.toml`
- `Cargo.lock`

Commands to run:
1. `cargo fmt --all`
2. `cargo clippy -p substrate-common -- -D warnings`
3. `cargo test -p substrate-common manager_manifest`

Reminders:
- Start from `AI_AGENT_START_HERE.md`, update `tasks.json` + `session_log.md` with START/END entries.
- Work inside `wt/a1-manifest-integ`, merge `wt/a1-manifest-code` and `wt/a1-manifest-test`, and capture any fixes in the session log.
- Note command output and any follow-ups in the END log entry.
