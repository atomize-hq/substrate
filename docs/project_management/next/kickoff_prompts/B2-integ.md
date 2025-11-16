# Task B2-integ – Integration Agent Kickoff Prompt

Task ID: **B2-integ** (Integrate shim doctor/repair CLI)

Summary:
- Merge the code worktree (`wt/b2-doctor-code`) with the new test worktree (`wt/b2-doctor-test`, commit 711f9d5) so the CLI + coverage land together on `feat/isolated-shell-plan`.
- New integration tests live in `crates/shell/tests/shim_doctor.rs`. They spin up temporary HOME directories, feed a custom manifest via `SUBSTRATE_MANAGER_MANIFEST`, and assert:
  1. `substrate shim doctor` (human output) prints manifest paths, PATH diagnostics (`Host PATH includes Substrate shims: yes/no`), and a per-manager table with the latest hint text.
  2. `substrate shim doctor --json` emits a structured payload with `states`, `hints`, and `path` fields (see assertions around `ShimDoctorReport`).
  3. `substrate shim repair --manager <name> --yes` appends/replaces a delimited block in `~/.substrate_bashenv`, writes `~/.substrate_bashenv.bak`, and remains idempotent.
- Until B2-code lands, both doctor invocations fail with `error: unrecognized subcommand 'shim'`. Capture the updated output once the CLI is wired up.

Commands to run after merging code + tests:
1. `cargo fmt --all`
2. `cargo test -p substrate-shell shim_doctor`
3. `cargo test -p substrate-shell --test shim_doctor` (equivalent but explicit target)

Notes & expectations:
- Tests assume the CLI reuses the manifest + `~/.substrate/trace.jsonl` parsing paths implemented in B2-code. Ensure the commands honor `HOME`, `USERPROFILE`, `SUBSTRATE_MANAGER_MANIFEST`, and `SHIM_TRACE_LOG` overrides injected by the fixtures so they never touch real dotfiles.
- When validating `shim repair`, confirm it writes the delimited block exactly once, refreshes the `.bak`, and logs a `shim_repair` event.
- Human output should include the shim dir path and whether the host PATH already contains it; JSON output should surface the same diagnostics under `path.host_contains_shims`/`path.shim_dir`.
- Record the passing `cargo test` output in the session log once both commands succeed.
