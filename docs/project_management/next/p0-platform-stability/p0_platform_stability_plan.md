# P0 Platform Stability Plan

## Context

The next statement of work bundles all three P0 backlog items (docs/BACKLOG.md):

1. **Socket-activated world-agent service** – refactor the Linux/Lima/WSL world-agent so systemd can launch it on-demand via `.socket` units while retaining backwards-compatible manual binds.
2. **Replay polish – isolation + verbose scopes** – finish the isolation follow-ups, improve replay verbosity, and expand default-to-world test coverage plus warning distinctions when replays skip world execution.
3. **Health command manager mismatch bug** – fix false “attention required” results when optional host package managers (direnv/asdf/conda/etc.) are missing both on the host and in the world.

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
- Doc/task/log edits occur on the branch root; production/test code lives in task-specific branches + worktrees.
- Each track uses the same **code / test / integration** triad found in `json-mode/` and `config-subcommand/`.

### Start Checklist (feat/p0-platform-stability)

1. `git checkout feat/p0-platform-stability && git pull --ff-only`
2. Read this plan, `tasks.json`, `session_log.md`, the relevant spec, and your kickoff prompt.
3. Set the task status to `in_progress` in `tasks.json`.
4. Add a START entry to `session_log.md`; commit docs (`docs: start <task-id>`).
5. Create the task branch from `feat/p0-platform-stability`, then add the worktree: `git worktree add wt/<task-branch> <task-branch>`.
6. Do **not** edit docs/tasks/session_log from the worktree.

### End Checklist (code/test)

1. Run required commands (code: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`. Test: `cargo fmt`; targeted `cargo test ...`). Capture outputs.
2. From inside the worktree, commit task branch changes (no docs/tasks/session_log edits).
3. From outside the worktree, fast-forward the task branch if needed. Do **not** touch `feat/p0-platform-stability`.
4. Checkout `feat/p0-platform-stability`; update `tasks.json` status, add END entry to `session_log.md` with commands/results/blockers, create downstream prompts if missing, and commit docs (`docs: finish <task-id>`).
5. Remove the worktree: `git worktree remove wt/<task-branch>`.

### End Checklist (integration)

1. Merge code/test branches into the integration worktree; reconcile to the spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight`. Capture outputs.
3. Commit integration changes to the integration branch.
4. Fast-forward merge the integration branch into `feat/p0-platform-stability`; update `tasks.json` and `session_log.md` with END entry (commands/results/blockers) and commit docs (`docs: finish <task-id>`).
5. Remove the worktree.

### Role Responsibilities

| Role | Focus | Restrictions |
| --- | --- | --- |
| Code | Production code, provisioning scripts, docs tied to implementation | No test-only edits beyond helper stubs |
| Test | Unit/integration tests, harness scripts, fixtures, kickoff prompts | No production logic beyond sanctioned helpers |
| Integration | Merge code/test branches, resolve conflicts, rerun fmt/lint/tests, update docs/logs/tasks | No new feature/test work |

## Tracks & Phases

Every backlog item is decomposed into multiple triads so each agent session stays within ~136k tokens of context. Each phase introduces its own code/test/integration trio; downstream phases depend on the prior integration task for the same backlog line.

1. **S1 – Socket-Activated World-Agent**
   - **S1a – Agent socket plumbing**: Teach `world-agent` to consume LISTEN_FDS, emit telemetry about inherited sockets, and maintain the direct-bind fallback.
   - **S1b – Shell readiness + telemetry**: Update `ensure_world_agent_ready()`, `world_enable`, shim status, and tracing so shell-side tooling gracefully handles socket activation.
   - **S1c – Provisioning & docs**: Modify Linux/Lima/WSL installers/uninstallers plus supporting docs/tests to deploy/manage `.service` + `.socket` units.
   - **S1d – Installer parity**: Update both the developer and production installers so they mirror the provisioning scripts (create the `substrate` group, add the invoking user, set socket permissions, document lingering requirements, and capture validation logs).
   - **LP1 – Linux provision parity fix**: Correct `scripts/linux/world-provision.sh` to match installer behavior (SocketGroup=substrate, group add, linger guidance) so standalone provisioning yields `root:substrate 0660` sockets without installer-dependent fixups.
2. **R1 – Replay Isolation & Visibility**
   - **R1a – Isolation fallback & diagnostics**: Finish the nft cgroup fallback, netns/rule cleanup helpers, and related documentation.
   - **R1b – Verbose scopes & warnings**: Add the `scopes: [...]` line under `--replay-verbose`, differentiate shell vs replay warnings, and update docs/help text.
   - **R1c – Replay world coverage**: Expand default-to-world/`--no-world`/env opt-out tests plus fixtures ensuring verbose output + isolation toggles behave consistently on all supported platforms.
3. **H1 – Health Manager Parity**
   - **H1a – Detection & aggregation logic**: Fix the status computation so only host-present/world-missing managers trigger “attention required”, and emit structured telemetry.
   - **H1b – CLI/doctor UX & docs**: Polish human/JSON output, doctor summaries, and documentation (USAGE/CONFIGURATION) with examples for macOS/Linux/WSL.
4. **R2 – Agent-backed Replay (follow-up branch)**
   - **R2a – Agent path default**: Prefer world-agent when healthy, keep host-only opt-outs.
   - **R2b – Fallback warnings**: Improve warning deduplication and copy-diff retries.
   - **R2c – Coverage polish**: Refresh CLI/docs/telemetry and replay fixtures.
   - **R2d – Origin-aware defaults & agent routing**: Record execution origin/transport on spans, default replays to the recorded origin, add a flip flag, and make world-mode replays agent-first with a single-warning fallback to the local backend (overlay/fuse/copy-diff) while preserving cwd/anchor/caging/env.
   - **R2e – Policy-driven world fs mode**: Add a broker policy toggle for read-only vs writable worlds, wire it through shell/world-agent, update docs/doctor, and ensure systemd defaults allow policy to take effect.
   - **R2e – Policy-driven world fs mode**: Add a policy toggle for read-only vs writable worlds (global + per-project), plumb it through broker/shell/world-agent, surface it in doctor/telemetry, and ensure systemd defaults allow policy to take effect.

Each phase maps to its own `code`, `test`, and `integration` tasks described in `tasks.json` with dedicated kickoff prompts.
