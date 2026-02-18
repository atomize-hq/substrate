# Slice Closeout Gate Report — full-isolation-landlock-overlayfs-compat / C0

Date (UTC): 2026-01-20T01:52:53Z

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat`

Slice spec:
- `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/C0-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior: with `world_fs.isolation=full` and `world_fs.mode=writable`, allowlisted project writes can fail with `Operation not permitted` when Landlock is supported and overlayfs is the active filesystem strategy.
- New behavior: allowlisted project writes succeed consistently under full isolation with Landlock enabled; non-allowlisted writes remain denied.
- Why: Landlock must allow overlayfs internal backing directory writes (`upperdir` / `workdir`) for overlayfs to service allowlisted project writes.
- Links:
  - `docs/project_management/adrs/implemented/ADR-0015-full-isolation-landlock-overlayfs-backing-dirs.md`
  - `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/C0-spec.md`
  - `docs/project_management/_archived/full-isolation-landlock-overlayfs-compat/decision_register.md (DR-0001, DR-0002, DR-0003, DR-0004)`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale)

## Checks Run (Evidence)

- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass (`cargo test` via `make integ-checks`)
- `make integ-checks`: pass
- CI compile parity: `RUN_ID=21173624758` (`https://github.com/atomize-hq/substrate/actions/runs/21173624758`)

## Cross-Platform Smoke (if applicable)

Record run ids/URLs for required platforms:
- Linux:
  - `RUN_ID=21183298683` (`https://github.com/atomize-hq/substrate/actions/runs/21183298683`) — job `linux_self_hosted` succeeded
- macOS:
  - `RUN_ID=21183298683` (`https://github.com/atomize-hq/substrate/actions/runs/21183298683`) — job `macos_self_hosted` succeeded
- Windows: n/a for behavior smoke (CI parity covered by `RUN_ID=21173624758`)
- WSL: n/a

If any platform-fix work was required:
- What failed:
  - Feature smoke `RUN_ID=21173728283` (`https://github.com/atomize-hq/substrate/actions/runs/21173728283`) — failed on `linux_self_hosted` + `macos_self_hosted`
  - Feature smoke `RUN_ID=21183010775` (`https://github.com/atomize-hq/substrate/actions/runs/21183010775`) — failed
- What was changed:
  - Fixes landed on `feat/full-isolation-landlock-overlayfs-compat` and smoke reran green (`RUN_ID=21183298683`).
- Why the change is safe (guards, cfg, feature flags):
  - Verified by CI compile parity (`RUN_ID=21173624758`) and CI smoke (`RUN_ID=21183298683`) on the required behavior platforms (Linux + macOS).

## Smoke ↔ Manual Parity

- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)

Notes:
- Feature smoke (passing): `RUN_ID=21183298683` (`https://github.com/atomize-hq/substrate/actions/runs/21183298683`)
