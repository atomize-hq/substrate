# Kickoff – M5b-code (Host detection parity)

## Role
Code agent: production code/scripts only. No tests. Do not edit docs/tasks/session logs from the worktree.

## Goal
Implement M5b-spec: make world deps host detection reflect the Substrate-managed host environment (manager init semantics).

## Read first
- `docs/project_management/next/p0-platform-stability-macOS-parity/plan.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/tasks.json`
- `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
- `docs/project_management/next/p0-platform-stability-macOS-parity/M5b-spec.md`

## Start checklist (must follow)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Set `M5b-code` to `in_progress` in `tasks.json` (orchestration branch only).
3. Add START entry to `session_log.md`; commit docs (`docs: start M5b-code`).
4. Create branch `mp-m5b-world-deps-host-detect-code`, then worktree: `git worktree add wt/mp-m5b-world-deps-host-detect-code mp-m5b-world-deps-host-detect-code`.
5. Do not edit docs/tasks/session_log from the worktree.

## Constraints / guardrails
- Do not expand tool inventory (that is M5a).
- Do not change installer/first-run wiring (that is M5c).
- Host detection must not mutate user dotfiles.
- Treat this task as **bash-first** on macOS: implement the minimal behavior needed to make host detection reflect the Substrate-managed bash environment. Only add non-bash support if it is clearly low-risk and already supported by existing plumbing.

## Scope risk watchlist (stop and split if triggered)
If you hit any of the following, **do not broaden scope inside M5b**. Instead, stop and request a new triad (create M5d/M5e specs + tasks + prompts on the orchestration branch):
- You need to add support for multiple interactive shells (zsh/fish) with materially different init behavior.
- You need to source user rc files (`~/.bashrc`, `~/.zshrc`, etc.) or otherwise risk mutating user environments.
- You need cross-platform changes that alter Linux/WSL behavior beyond minor refactors.
- The change requires large refactors across routing/manager init/world deps that won’t fit comfortably in a single PR-sized triad.

## Escape hatches (acceptable scoped fallbacks)
- **Bash-only**: If non-bash shells would add complexity, explicitly scope to bash on macOS and ensure output makes the limitation clear.
- **Degraded-but-honest detection**: If the manager init environment cannot be applied, report a clear reason and ensure sync skips tools rather than guessing.
- **Explicit opt-in**: If needed, gate more aggressive detection behavior behind an env var and document it (but keep defaults safe).

## Required commands (record output in END entry)
- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`

## End checklist
1. Run required commands and capture outputs.
2. Commit changes inside the worktree (no docs edits).
3. Switch back to orchestration branch; mark task completed; add END entry; commit docs (`docs: finish M5b-code`).
4. Remove worktree `wt/mp-m5b-world-deps-host-detect-code`.
