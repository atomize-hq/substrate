# Slice Closeout Gate Report — workspace-config-policy-unification / WCU4

Date (UTC): 2026-01-16

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/workspace-config-policy-unification/`

Slice spec:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

## Behavior Delta (Existing → New → Why)
- Existing behavior: dev/install scripts could export `SUBSTRATE_OVERRIDE_*` by default, making overrides unintentionally “sticky” across sessions.
- New behavior: installer/dev install scripts no longer export any `SUBSTRATE_OVERRIDE_*` by default; overrides remain supported only when explicitly provided by the operator.
- Why: align with ADR-0008’s scope model and avoid silently affecting `config current show` / effective behavior via ambient overrides.
- Links: `scripts/substrate/install-substrate.sh`, `scripts/substrate/dev-install-substrate.sh`, `crates/shell/tests/installer_env_wcu4.rs`

## Spec Parity (No Drift)
- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale)

## Checks Run (Evidence)
- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass (`cargo test -p substrate-shell --test installer_env_wcu4 -- --nocapture`)
- `make integ-checks`: pass

## Cross-Platform Smoke (if applicable)
- Linux: CI run `21071405891` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21071405891
- macOS: CI run `21071405891` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21071405891
- Windows: CI run `21071405891` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21071405891

## Smoke ↔ Manual Parity
- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)
