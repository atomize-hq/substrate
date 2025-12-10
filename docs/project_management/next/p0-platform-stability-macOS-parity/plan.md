# P0 Platform Stability – macOS Parity Plan

## Purpose
Bridge every behavior shipped in `p0-platform-stability` to macOS. This includes socket-activated world-agent provisioning, installer/uninstaller parity, policy-driven world fs mode, replay/doctor UX alignment, and doctor/manual flows so mac hosts behave like Linux/WSL.

## Guardrails
- Triads only: code / test / integration. No mixed roles.
- Code: production code/scripts only; no tests. Required commands: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`. Optional targeted sanity checks allowed.
- Test: tests/fixtures/harnesses only; no production logic. Required commands: `cargo fmt`; targeted `cargo test ...` for suites added/touched.
- Integration: merges code+tests, reconciles to spec, and must run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, **and `make preflight`** (required).
- Docs/tasks/session_log edits happen only on the orchestration branch (`feat/p0-platform-stability-macOS-parity`), never from worktrees.
- Respect protected paths (no touching `.git`, sockets, device nodes, host-sensitive roots) when adding migration or cleanup logic.

## Branch & Worktree Conventions
- Orchestration branch: `feat/p0-platform-stability-macOS-parity`.
- Branch naming: `mp-<triad>-<scope>-<role>` (e.g., `mp-m1-migration-code`).
- Worktrees: `wt/<branch>` (e.g., `wt/mp-m1-migration-code`).

## Triad Overview
- **M1 – Lima migration & socket parity:** Detect/migrate existing Lima VMs to the socket-activated layout, ensure agent binary + units + permissions align with Linux/WSL expectations, and make warm/provisioning idempotent.
- **M2 – Installer parity (dev/prod):** Align mac installers with Linux behavior: build/copy Linux agent (and optional CLI shim) inside Lima when missing, honor cleanup-state metadata, and keep uninstall parity.
- **M3 – Backend & doctor parity:** Propagate policy fs_mode to mac backend, fix forwarding/readiness ordering, align socket/group expectations and doctor/manual flows with P0 outputs, and refresh docs/tests accordingly.

## Start Checklist (all tasks)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: this plan, tasks.json, session_log.md, the relevant spec, and your kickoff prompt.
3. Set the task status to `in_progress` in tasks.json (orchestration branch only).
4. Add a START entry to session_log.md; commit docs (`docs: start <task-id>`).
5. Create task branch and worktree: `git worktree add wt/<branch> <branch>`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## End Checklist (code/test)
1. Run required commands (code: fmt + clippy; test: fmt + targeted tests).
2. Commit worktree changes.
3. Merge back fast-forward into orchestration branch (`git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only && git merge --ff-only <task-branch>`).
4. Update tasks.json status; add END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish <task-id>`).
5. Remove worktree: `git worktree remove wt/<branch>`.

## End Checklist (integration)
1. Merge code/test branches into the integration worktree; reconcile to the spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight` (required). Capture outputs.
3. Commit integration changes.
4. Merge back fast-forward into orchestration branch; update tasks.json and session_log.md with END entry; commit docs (`docs: finish <task-id>`).
5. Remove worktree.
