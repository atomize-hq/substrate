# Slice Closeout Gate Report — world-overlayfs-enumeration / WO0

Date (UTC): 2026-01-06T22:16:25Z

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/next/world-overlayfs-enumeration/`

Slice spec:
- `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior:
  - Linux world overlay mounts could succeed while directory enumeration in the merged view was broken (e.g., `ls` empty while `stat` worked), making world runs unsafe/confusing and difficult to reproduce.
  - Strategy selection and fallback behavior was not deterministic/observable (doctor/trace lacked an explicit contract for “primary vs final” strategy and fallback reason).
- New behavior:
  - Linux world execution refuses to proceed on any filesystem strategy that fails an enumeration health probe and deterministically selects a strategy + fallback chain (`overlay` → `fuse`).
  - When no viable strategy exists: fail closed with exit code `3` if world is required; otherwise execute on host and emit the exact warning line `substrate: warn: world unavailable; falling back to host`.
  - `substrate world doctor --json` and trace `command_complete` events include the WO0 strategy + probe metadata required by ADR-0004/WO0-spec.
  - Project cage mount topology uses `mount --move` (not `mount --bind`) to place the overlay at the project path inside the private mount namespace.
- Why:
  - Restore the core world contract (enumeration correctness) and make host variance debuggable via deterministic selection + explicit observability.
- Links:
  - ADR: `docs/project_management/next/ADR-0004-world-overlayfs-directory-enumeration-reliability.md`
  - Spec: `docs/project_management/next/world-overlayfs-enumeration/WO0-spec.md`
  - Smoke: `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`
  - Manual playbook: `docs/project_management/next/world-overlayfs-enumeration/manual_testing_playbook.md`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale)

## Checks Run (Evidence)

- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass
  - `cargo test -p substrate-shell --test world_overlayfs_enumeration_wo0 -- --nocapture`
  - `cargo test -p world-agent --test overlayfs_enumeration -- --nocapture`
  - `cargo test -p world --test overlayfs_enumeration_fallback -- --nocapture`
- `docs/project_management/next/world-overlayfs-enumeration/smoke/linux-smoke.sh`: pass
- `make integ-checks`: pass

## Cross-Platform Smoke (when applicable)

Record run ids and URLs for required platforms:
- Linux: local run (no CI run id)
- macOS: N/A (ADR scope is Linux-only)
- Windows: N/A (ADR scope is Linux-only)
- WSL: N/A

If any platform-fix work was required:
- What failed:
- What was changed:
- Why the change is safe (guards, cfg, feature flags):

## Smoke and Manual Parity

- [x] Smoke scripts run the same commands and workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output

Notes:
- Smoke/playbook create an isolated temp workspace (`substrate workspace init .`) so the test runs in `world_fs.mode=writable` even when the operator’s global policy is `world_fs.mode=read_only`.
