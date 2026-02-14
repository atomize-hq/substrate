# Slice Closeout Gate Report — world-sync / WS3

Date (UTC): 2026-02-12T01:19:53Z

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/next/world-sync`

Slice spec:
- `docs/project_management/next/world-sync/WS3-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior:
  - Non-PTY world command execution did not trigger `workspace sync` automatically; any sync/apply was manual (`substrate workspace sync`).
- New behavior:
  - When `sync.auto_sync=true` and a non-PTY command executes in the world backend and exits `0`, Substrate automatically runs a `workspace sync` apply (WS3).
  - Auto-sync does not run when the command exits non-zero.
  - Direction gating:
    - Effective `sync.direction=from_world` or `both` triggers a `from_world` apply.
    - Effective `sync.direction=from_host` is a no-op for WS3 auto-sync (command exit code preserved).
  - If auto-sync runs and fails, the overall exit code is the auto-sync failure’s exit code and output includes `auto-sync failed:` with the underlying reason.
- Why:
  - Implement WS3 acceptance criteria: automatic world→host apply after successful non-PTY world executions when enabled.
- Links:
  - `docs/project_management/next/world-sync/WS3-spec.md`
  - `crates/shell/src/execution/routing/dispatch/exec.rs`
  - `crates/shell/src/execution/workspace_cmd.rs`
  - `crates/shell/tests/workspace_auto_sync_ws3.rs`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale)
  - No spec changes were required during integration.

## Checks Run (Evidence)

- `cargo fmt --all`: PASS
- `cargo clippy --workspace --all-targets -- -D warnings`: PASS
- Relevant tests: PASS
  - `cargo test -p shell -- --nocapture`
  - `cargo test -p shell --test workspace_auto_sync_ws3 -- --nocapture`
- `make integ-checks`: PASS

## Cross-Platform Smoke (if applicable)

Record run ids/URLs for required platforms:
- Linux:
- macOS:

If smoke/CI was intentionally skipped:
- Reason (e.g., `ci-audit: DIFF_CLASS=docs_only`): `CI dispatch is restricted to planned checkpoint ops tasks (see ci_checkpoint_plan.md); this slice did not dispatch cross-platform smoke.`
- Last-green run evidence (run id/URL, if available):
- Evidence ledger path (if used): `docs/project_management/next/world-sync/logs/WS3/ci-audit/ledger.jsonl`

If any platform-fix work was required:
- What failed:
- What was changed:
- Why the change is safe (guards, cfg, feature flags):

## Smoke ↔ Manual Parity

- [ ] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [ ] Smoke scripts validate exit codes and key output (not just “command ran”)

Notes:
- Auto-sync coverage is enforced by `crates/shell/tests/workspace_auto_sync_ws3.rs` (contract-level stub world-agent).
