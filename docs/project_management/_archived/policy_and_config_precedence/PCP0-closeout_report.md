# Slice Closeout Gate Report — policy_and_config_precedence / PCP0

Date (UTC): 2026-01-02T18:21:33Z

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/_archived/policy_and_config_precedence/`

Slice spec:
- `docs/project_management/_archived/policy_and_config_precedence/PCP0-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior: When a workspace exists, `SUBSTRATE_*` environment variables could override conflicting values set in `<workspace_root>/.substrate/workspace.yaml` for the same keys.
- New behavior: When a workspace exists, `<workspace_root>/.substrate/workspace.yaml` overrides conflicting `SUBSTRATE_*` environment variables (CLI flags remain highest precedence).
- Why: Workspace-local configuration should be the authoritative source for workspace behavior, preventing ambient environment from unexpectedly changing behavior inside an initialized workspace.
- Links:
  - `docs/project_management/next/ADR-0005-workspace-config-precedence-over-env.md`
  - `docs/project_management/_archived/policy_and_config_precedence/PCP0-spec.md`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale)

## Checks Run (Evidence)

- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass (`cargo test -p substrate-shell --tests -- --nocapture`)
- `make integ-checks`: pass

## Cross-Platform Smoke (if applicable)

Record run ids/URLs for required platforms:
- Linux: `https://github.com/atomize-hq/substrate/actions/runs/20663926907` (job: `linux_self_hosted`, id: `59332170571`)
- macOS: `https://github.com/atomize-hq/substrate/actions/runs/20663926907` (job: `macos_self_hosted`, id: `59332170584`)
- Windows: `https://github.com/atomize-hq/substrate/actions/runs/20663926907` (job: `windows_self_hosted`, id: `59332170576`)
- WSL: N/A (not required for this slice)

If any platform-fix work was required:
- What failed: N/A
- What was changed: N/A
- Why the change is safe (guards, cfg, feature flags): N/A

## Smoke ↔ Manual Parity

- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)

Notes:
- No platform-fix branches were required for PCP0 (CI smoke green for Linux/macOS/Windows on the first run).
