# Session Log — P0 Platform Stability macOS Parity

Use START/END entries only. Include UTC timestamp, agent role, task ID, commands run (fmt/clippy/tests/scripts), results (pass/fail, temp roots), worktree/branches, prompts created/verified, blockers, and next steps. Do not edit from worktrees.

## [2025-12-11 19:11 UTC] Code Agent – M1-code – START
- Checked out feat/p0-platform-stability-macOS-parity, `git pull --ff-only` (up to date)
- Read plan/tasks/session log/M1-spec/kickoff prompt; updated tasks.json (M1-code → in_progress)
- Worktree pending (`mp-m1-sockets-code` to be added after docs commit)
- Plan: refresh Lima profile + warm/provision scripts for socket-activated agent, enforce SocketGroup=substrate + user group membership + linger guidance, ensure idempotent rebuild path with actionable errors, wire diagnostics into mac doctor flows, run `cargo fmt`/`cargo clippy --workspace --all-targets -- -D warnings`, commit via worktree, update docs/tasks/log at end
- Blockers: none

## [2025-12-11 19:12 UTC] Test Agent – M1-test – START
- Checked out feat/p0-platform-stability-macOS-parity, `git pull --ff-only` (up to date)
- Read plan/tasks/session log/M1-spec/kickoff prompt; updated tasks.json (M1-test → in_progress)
- Scope: add fixtures/tests validating mac Lima replacement/provision detection + socket activation reporting without touching production logic
- Plan: create `mp-m1-sockets-test` branch/worktree, explore existing mac doctor/provision harnesses, add fixture coverage for warm/provision outputs + socket state detection, run `cargo fmt` + targeted tests, capture results for END log
- Blockers: none

## [2025-12-11 19:23 UTC] Test Agent – M1-test – END
- Worktree `wt/mp-m1-sockets-test` on branch `mp-m1-sockets-test` (commit 1401bf9) adds a mac Lima doctor harness (`tests/mac/lima_doctor_fixture.sh`) plus a socket activation text-mode regression test (`crates/shell/tests/socket_activation.rs`)
- Commands: `cargo fmt` (pass); `cargo test -p substrate-shell socket_activation` (pass – exercised JSON/text doctor + shim socket activation suites); `tests/mac/lima_doctor_fixture.sh` (pass – stubbed limactl/sysctl/jq to cover vm_missing, vm_stopped, socket_missing, caps_fail, service_down, healthy scenarios with replacement guidance)
- Harness outputs captured in script logs (see /tmp/substrate-mac-doctor.* paths emitted during run); no live Lima VM required
- Blockers: none

## [2025-12-11 19:27 UTC] Integration Agent – M1-integ – START
- Checked out `feat/p0-platform-stability-macOS-parity`, `git pull --ff-only` (already up to date)
- Read plan/tasks/session log/M1-spec/kickoff prompt, updated tasks.json (M1-integ → in_progress)
- Scope: merge `M1-code` + `M1-test`, ensure Lima socket parity replacement aligns with spec, gate with `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, targeted tests, `make preflight`
- Next: create integration branch/worktree (`mp-m1-sockets-integ`), merge upstream branches, reconcile behavior/tests before running required commands
- Blockers: none

## [2025-12-11 19:32 UTC] Integration Agent – M1-integ – END
- Worktree `wt/mp-m1-sockets-integ` merged `mp-m1-sockets-code` + `mp-m1-sockets-test` (branch `mp-m1-sockets-integ` now carries socket-parity Lima warm/doctor scripts, YAML SocketGroup fixes, and the mac doctor fixture/test additions) and fast-forwarded `feat/p0-platform-stability-macOS-parity`
- Commands: `cargo fmt` (pass); `cargo clippy --workspace --all-targets -- -D warnings` (pass); `cargo test -p substrate-shell socket_activation` (pass); `tests/mac/lima_doctor_fixture.sh` (pass – fixture logs at `/tmp/substrate-mac-doctor.S4aeLX`); `make preflight` (pass – reran fmt/clippy/clean/check/test for the workspace)
- Result: Lima warm flow now enforces socket-activated layout + group perms, doctor surfaces sentinel/group/socket state, tests/scripts cover socket activation text + Lima doctor scenarios; no regressions observed on Linux builds
- Blockers: none

## [2025-12-11 20:22 UTC] Code Agent – M2-code – START
- Checked out `feat/p0-platform-stability-macOS-parity`, pulled latest, read plan/tasks/session/M2-spec/kickoff prompt
- Updated `tasks.json` (`M2-code` → `in_progress`); scope confirmed: mac installer parity (prod copy fallback, dev build option, CLI shim/uninstall cleanup-state alignment)
- Next steps: doc commit (`docs: start M2-code`), branch `mp-m2-installer-code` + worktree, then implement installer changes + run fmt/clippy before END log
- Blockers: none

## [2025-12-11 20:32 UTC] Code Agent – M2-code – END
- Worktree `wt/mp-m2-installer-code` on branch `mp-m2-installer-code` (commit 693043b) now logs/validates mac installer strategies (prod copy-first with Lima build fallback + CLI shim parity, dev install source logging, Lima agent verification) and extends the host uninstall to remove the forwarded socket
- Commands: `cargo fmt` (pass); `cargo clippy --workspace --all-targets -- -D warnings` (pass – workspace clean)
- Results: prod installer relies on `lima-warm` copy/build logging + post-run verification, dev installer reports agent source before provisioning, `lima-warm` builds individual components only when missing (agent failures now fatal with guidance), and uninstall removes the host `~/.substrate/sock/agent.sock`
- Blockers: none

## [2025-12-11 20:23 UTC] Test Agent – M2-test – START
- Checked out `feat/p0-platform-stability-macOS-parity`, ran `git pull --ff-only` (already up to date), reread plan/tasks/session log/M2-spec/kickoff prompt
- Updated `tasks.json` (`M2-test` → `in_progress`) per checklist; scope confirmed: add mac installer/uninstaller parity tests + fixtures only (prod copy-first vs build fallback, cleanup-state metadata/log guidance)
- Plan: commit docs with `docs: start M2-test`, create branch/worktree `mp-m2-installer-test`/`wt/mp-m2-installer-test`, explore existing installer harnesses, add new fixtures/tests capturing prod copy vs build fallback, cleanup-state metadata, log expectations; run `cargo fmt` + targeted `cargo test ...` suites added/touched, capture outputs for END log
- Blockers: none

## [2025-12-11 20:49 UTC] Integration Agent – M2-integ – START
- Checked out `feat/p0-platform-stability-macOS-parity`, pulled latest, read plan/tasks/session log/M2-spec/kickoff prompt per checklist
- Updated `tasks.json` (`M2-integ` → `in_progress`) and confirmed integration scope: merge M2 code + tests, enforce prod copy-first/build fallback + dev build paths, CLI shim + cleanup-state parity, ensure Linux installers unaffected
- Next steps: commit start docs, create branch `mp-m2-installer-integ` with worktree `wt/mp-m2-installer-integ`, merge upstream branches, reconcile to M2-spec, then run required commands (`cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, `make preflight`)
- Blockers: none

## [2025-12-11 20:56 UTC] Integration Agent – M2-integ – END
- Worktree `wt/mp-m2-installer-integ` merged `mp-m2-installer-code` + `mp-m2-installer-test`, reconciled installer parity (copy-first w/ build fallback, dev build logging, cleanup-state/uninstall metadata) and patched the mac installer fixture to export `SHIM_ORIGINAL_PATH` so stubs work under shimmed shells; branch `mp-m2-installer-integ` now contains merge commits + fixture fix
- Commands: `cargo fmt` (pass); `cargo clippy --workspace --all-targets -- -D warnings` (pass); `tests/mac/installer_parity_fixture.sh --all` (pass – logs at `/tmp/substrate-mac-installer-prod-copy.O1JCbq/prod-copy.log`, `.prod-build.XfW0Cf/...`, `.dev-build.bxDxYd/...`, `.cleanup-guidance.7DkDvF/...`); `make preflight` (pass – reran fmt/clippy/clean/check/test for the workspace)
- Result: prod installer now verifies Lima agent installs regardless of bundle state, dev installer logs guest build sources, uninstall removes host socket, Lima warm only builds CLI/agent when missing, and the mac installer fixture exercises copy/build/cleanup guidance on Linux hosts; Linux/WSL installers unchanged (mac-only code paths touched)
- Blockers: none

## [2025-12-12 02:42 UTC] Code Agent – M3-code – START
- Checked out `feat/p0-platform-stability-macOS-parity`, `git pull --ff-only` (up to date)
- Read plan/tasks/session log/M3-spec/kickoff prompt; updated tasks.json (M3-code → in_progress)
- Worktree pending (`mp-m3-backend-code` to be added after docs commit)
- Plan: propagate policy fs_mode through mac backend (exec + replay PTY/non-PTY), reorder forwarding/readiness to establish forwarding before probes, align mac socket/group expectations in doctor/shim-status/health and update mac manual playbooks, run `cargo fmt`/`cargo clippy --workspace --all-targets -- -D warnings`, commit via worktree, update docs/tasks/log at end
- Blockers: none

## [2025-12-12 02:46 UTC] Test Agent – M3-test – START
- Checked out `feat/p0-platform-stability-macOS-parity`, `git pull --ff-only` (up to date), read plan/tasks/session log/M3-spec/kickoff prompt
- Updated `tasks.json` (`M3-test` → `in_progress`) per checklist; scope confirmed: tests/fixtures only for mac fs_mode propagation, readiness/forwarding ordering, and doctor/shim-status/health JSON/text parity (platform-agnostic portions)
- Next steps: commit docs with `docs: start M3-test`, create branch/worktree `mp-m3-backend-test`/`wt/mp-m3-backend-test`, add required tests/fixtures, run `cargo fmt` + targeted `cargo test ...`, capture outputs for END log
- Blockers: none

## [2025-12-12 03:00 UTC] Code Agent – M3-code – END
- Worktree `wt/mp-m3-backend-code` on branch `mp-m3-backend-code` (commit 91bafdb) propagates policy `fs_mode` to the mac Lima backend, fixes pre-forwarding readiness by probing the guest socket before host forwarding, and aligns mac shim-status + world doctor JSON/text with Linux P0 (socket activation state and `agent_socket`/`world_socket` parity).
- Commands: `cargo fmt` (pass); `cargo clippy --workspace --all-targets -- -D warnings` (pass)
- Results: clippy built `substrate-shell`, `world-mac-lima`, and `substrate` with no warnings.
- Scripts executed: n/a
- Kickoff prompts created: n/a (M3-test/M3-integ prompts already present)
- Docs commit: pending (`docs: finish M3-code`)
- Blockers: none

## [2025-12-12 03:13 UTC] Test Agent – M3-test – END
- Worktree `wt/mp-m3-backend-test` on branch `mp-m3-backend-test` (commit e91e690) adds replay fs_mode propagation regression, mac forwarding ordering unit test, mac ExecuteRequest fs_mode env propagation unit test, fixture-backed shim doctor/health fs_mode surfacing checks, and shim-status fs_mode parity tests.
- Commands: `cargo fmt --all` (pass); `cargo test -p substrate-shell --test shim_doctor` (pass); `cargo test -p substrate-shell --test shim_health` (pass); `cargo test -p substrate-shell --test shim_status_fs_mode` (fail – shim status missing `world_fs_mode` in JSON/text); `cargo test -p substrate-replay reconstruct_state_exports_world_fs_mode_from_replay_context` (fail – replay context fs_mode not exported to env).
- Mac-only coverage: unit tests in `crates/world-mac-lima` are `#[cfg(target_os = "macos")]` and were not executed on Linux; verify on mac during M3-integ.
- Blockers: pending M3-code integration to surface `world_fs_mode` in shim-status outputs and honor replay-context fs_mode during replay.

## [2025-12-12 03:30 UTC] Integration Agent – M3-integ – START
- Checked out `feat/p0-platform-stability-macOS-parity`, ran `git pull --ff-only` (already up to date), read plan/tasks/session log/M3-spec/kickoff prompt per checklist
- Updated `tasks.json` (`M3-integ` → `in_progress`); integration scope confirmed: merge `M3-code` + `M3-test`, reconcile mac fs_mode propagation, forwarding/readiness ordering, doctor/shim-status/health parity with M3-spec
- Next steps: commit this START docs update (`docs: start M3-integ`), create branch/worktree `mp-m3-backend-integ`/`wt/mp-m3-backend-integ`, merge upstream branches, fix outstanding test failures (shim-status `world_fs_mode` surfacing and replay-context fs_mode env export), then run required commands (`cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, relevant tests, `make preflight`)
- Blockers: none

## [2025-12-12 03:51 UTC] Integration Agent – M3-integ – END
- Worktree `wt/mp-m3-backend-integ` merged `mp-m3-backend-code` + `mp-m3-backend-test`, resolved minor test conflicts/duplication, and reconciled to M3-spec. Added shim-status `world_fs_mode` parity (JSON/text) and exported replay-context fs_mode via `SUBSTRATE_WORLD_FS_MODE`; forwarding ordering and mac backend fs_mode propagation retained from M3-code.
- Commands: `cargo fmt` (pass); `cargo clippy --workspace --all-targets -- -D warnings` (pass after removing duplicate tests and fixing a clippy lint in `shim_status_fs_mode`); `cargo test -p substrate-shell --test shim_doctor` (pass); `cargo test -p substrate-shell --test shim_health` (pass); `cargo test -p substrate-shell --test shim_status_fs_mode` (pass); `cargo test -p substrate-replay reconstruct_state_exports_world_fs_mode_from_replay_context` (pass); `make preflight` (pass – reran fmt/clippy/clean/check/test for workspace).
- Result: mac shim-status/health/doctor outputs now surface fs_mode and socket activation in parity with Linux P0, mac backend honors policy fs_mode across exec/replay, and readiness no longer probes pre-forwarding. Workspace remains green on non-mac targets.
- Blockers: none

## [2025-12-19 14:38 UTC] Code Agent – M4-code – START
- Checked out `feat/p0-platform-stability-macOS-parity`, `git pull --ff-only` (up to date)
- Read plan/tasks/session log/M4-spec/kickoff prompt; updated tasks.json (M4-code → in_progress)
- Worktree pending (`mp-m4-world-deps-manifest-code` to be added after docs commit)
- Plan: update world deps manifest resolution to prefer installed `<prefix>/versions/<version>/config/world-deps.yaml` by default, retain workspace fallback to repo `scripts/substrate/world-deps.yaml`, preserve `SUBSTRATE_WORLD_DEPS_MANIFEST` override semantics across status/install/sync, and ensure status JSON/human output surfaces resolved paths; run `cargo fmt`/`cargo clippy --workspace --all-targets -- -D warnings`, commit via worktree, update docs/tasks/log at end
- Blockers: none
