# P0 Platform Stability – macOS Parity Plan

## Purpose
Bridge every behavior shipped in `p0-platform-stability` to macOS. This includes socket-activated world-agent provisioning, installer/uninstaller parity, policy-driven world fs mode, replay/doctor UX alignment, and doctor/manual flows so mac hosts behave like Linux/WSL.

## Guardrails
- Triads only: code / test / integration. No mixed roles.
- Code: production code/scripts only; no tests. Required commands: `cargo fmt`; `cargo clippy --workspace --all-targets -- -D warnings`. Optional targeted sanity checks allowed.
- Test: tests/fixtures/harnesses only; no production logic. Required commands: `cargo fmt`; targeted `cargo test ...` for suites added/touched.
- Integration: merges code+tests, reconciles to spec, and must run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, **and `make preflight`** (required).
- Docs/tasks/session_log edits happen only on the orchestration branch (`feat/p0-platform-stability-macOS-parity`), never from worktrees.
- Respect protected paths (no touching `.git`, sockets, device nodes, host-sensitive roots) when adding provisioning or cleanup logic.
- Treat the existing Lima environment as replaceable: we re-provision/upgrade mac hosts to the new socket layout and do not maintain backwards compatibility or perform in-place user carry-over.

## Branch & Worktree Conventions
- Orchestration branch: `feat/p0-platform-stability-macOS-parity`.
- Branch naming: `mp-<triad>-<scope>-<role>` (e.g., `mp-m1-sockets-code`).
- Worktrees: `wt/<branch>` (e.g., `wt/mp-m1-sockets-code`).

## Triad Overview
- **M1 – Lima socket parity upgrade:** Replace the current Lima environment with the socket-activated layout, ensure agent binary + units + permissions align with Linux/WSL expectations, and keep warm/provisioning idempotent.
- **M2 – Installer parity (dev/prod):** Align mac installers with Linux behavior: prod copies bundled Linux agent into Lima (build only on missing/invalid bundle); dev may build in-guest; optional CLI shim parity; cleanup-state metadata; uninstall parity.
- **M3 – Backend & doctor parity:** Propagate policy fs_mode to mac backend, fix forwarding/readiness ordering, align socket/group expectations and doctor/manual flows (including shim-status and health parity) with P0 outputs, and refresh docs/tests accordingly.
- **M4 – World deps base manifest parity:** Ensure `substrate world deps` uses the installed/bundled manifest by default (not the repo/CWD fallback), and that manifest resolution matches `docs/CONFIGURATION.md`.
- **M5a – World deps inventory & layering:** Define and implement the authoritative base inventory for `world deps` (aligned with shim doctor/health) and treat `world-deps.yaml` as an overlay/override layer.
- **M5b – Host detection parity:** Make “host present” detection for world deps match the Substrate-managed host environment (manager init semantics), so first-run parity reflects what users actually have available on the host.
- **M5c – First-run UX wiring:** Ensure installer/provision flows (including `--sync-deps`) and health/doctor recommendations produce a coherent “feels like host” out-of-box experience.
- **M6 – World deps failure safety (macOS):** Prevent misleading host fallbacks during world deps operations on macOS; surface actionable errors when the world backend/forwarding is unavailable so “sync” can’t report success without affecting the guest.

## Start Checklist (all tasks)
1. `git checkout feat/p0-platform-stability-macOS-parity && git pull --ff-only`
2. Read: this plan, tasks.json, session_log.md, the relevant spec, and your kickoff prompt.
3. Set the task status to `in_progress` in tasks.json (orchestration branch only).
4. Add a START entry to session_log.md; commit docs (`docs: start <task-id>`).
5. Create the task branch from `feat/p0-platform-stability-macOS-parity`, then add the worktree: `git worktree add wt/<branch> <branch>`.
6. Do **not** edit docs/tasks/session_log from the worktree.

## End Checklist (code/test)
1. Run required commands (code: fmt + clippy; test: fmt + targeted tests) and capture outputs.
2. From inside the worktree, commit task branch changes (no docs/tasks/session_log edits).
3. From outside the worktree, fast-forward the task branch to include the worktree commit if needed. Do **not** touch the orchestration branch.
4. Checkout `feat/p0-platform-stability-macOS-parity`; update tasks.json status; add an END entry to session_log.md with commands/results/blockers; create downstream prompts if missing; commit docs (`docs: finish <task-id>`).
5. Remove the worktree: `git worktree remove wt/<branch>`.

## End Checklist (integration)
1. Merge code/test branches into the integration worktree; reconcile behavior to the spec.
2. Run `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, then `make preflight` (required). Capture outputs.
3. Commit integration changes to the integration branch.
4. Fast-forward merge the integration branch into `feat/p0-platform-stability-macOS-parity`; update tasks.json and session_log.md with the END entry (commands/results/blockers); commit docs (`docs: finish <task-id>`).
5. Remove the worktree.
