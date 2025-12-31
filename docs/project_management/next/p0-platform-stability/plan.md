# P0 Platform Stability Plan

## Context

The next statement of work bundles all three P0 backlog items (docs/BACKLOG.md):

1. **Socket-activated world-agent service** – refactor the Linux/Lima/WSL world-agent so systemd can launch it on-demand via `.socket` units while retaining backwards-compatible manual binds.
2. **Replay polish – isolation + verbose scopes** – finish the isolation follow-ups, improve replay verbosity, and expand default-to-world test coverage plus warning distinctions when replays skip world execution.
3. **Health command manager mismatch bug** – fix false “attention required” results when optional host package managers (direnv/asdf/conda and similar) are missing both on the host and in the world.

These threads are tightly coupled to platform stability and observability. We will deliver them under a single coordination umbrella so shell/world/replay changes land coherently and the doctor/health UX reflects the new behaviors.

## Goals

1. **Systemd socket readiness** – Teach `world-agent` to accept inherited LISTEN_FDS, adjust shell readiness checks, and update provisioning/uninstall scripts plus docs/tests so Linux/macOS/WSL installs deploy both `.service` and `.socket` units.
2. **Replay hardening** – Complete the Phase 4.5 isolation follow-ups (nft cgroup fallback, diagnostics, docs), add verbose scope output to `substrate --replay --replay-verbose`, clarify warning prefixes, and ensure integration tests cover default world usage, `--no-world`, and env opt-outs.
3. **Accurate health summaries** – Update `substrate health` so manager mismatches are only reported when the host supports a manager but the world does not, covering code, tests, and docs.

## Standards & References

- Repository guardrails: `AGENTS.md`.
- Backlog source: `docs/BACKLOG.md` (P0 items).
- Relevant crates & docs:
  - `crates/world-agent`, `crates/world-linux`, `crates/world-mac-lima`, `crates/world-windows-wsl`.
  - Provisioning scripts under `scripts/linux`, `scripts/mac`, `scripts/windows`.
  - `crates/shell` (`world_enable`, replay CLI, health command).
  - `crates/replay`, `docs/REPLAY.md`, `docs/WORLD.md`, `docs/TRACE.md`.
- Tooling expectations: `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted `cargo test` suites, replay/world doctor scripts as specified per kickoff prompt.

## Workflow Guardrails

- All work happens on `feat/p0-platform-stability`.
- Planning Pack doc edits occur on the orchestration branch.
- Do not edit planning docs inside the worktree.
- Each implementation slice uses the code/test/integration triad workflow.

### Start Checklist (feat/p0-platform-stability)

1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read this plan, `tasks.json`, `session_log.md`, the relevant spec, and your kickoff prompt.
3. Set the task status to `in_progress` in `tasks.json`.
4. Add a START entry to `session_log.md`, commit docs.
5. Create the task branch and `git worktree add <worktree> <branch>`.

### End Checklist (all tasks)

1. Run the required commands in the kickoff prompt.
2. Commit worktree changes.
3. Merge back to the orchestration branch (ff-only).
4. Update `tasks.json` + `session_log.md` with an END entry; commit docs.
5. Remove the worktree: `git worktree remove <worktree>`.
