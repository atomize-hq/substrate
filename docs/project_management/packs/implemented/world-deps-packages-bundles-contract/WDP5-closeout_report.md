# Slice Closeout Gate Report — world-deps-packages-bundles-contract / WDP5

Date (UTC): 2026-02-14T21:50:00Z

Standards:

- `docs/project_management/system/standards/execution/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/system/standards/adr/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:

- `docs/project_management/packs/active/world-deps-packages-bundles-contract`

Slice spec:

- `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP5-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior:
  - `substrate world deps current install|sync` did not execute `install.method=apt` and did not reliably apply apt+script inventory end-to-end per the contract.
  - Hardening/cage denials were surfaced as “backend unavailable” exit `3` in some paths.
  - Bulk world probe output parsing was brittle when stdout is missing/dropped (e.g., degraded backends / test stubs).
- New behavior:
  - `current install|sync` executes apt installs (apt-first, then script installs) and enforces contract exit codes:
    - manual items remain blocked: exit `4`
    - hardening/cage denials: exit `5`
  - World probe parsing is resilient: if bulk probe stdout lacks per-item markers, it falls back to per-check exit-code probes.
  - `current list applied` expands enabled bundles to include their packages; `current show --explain` always includes an actionable hint.
- Why:
  - Align WDP5 slice behavior to `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md` and unblock deterministic dependency application.
- Links:
  - Integration core HEAD: `bd4429b3a2554ab5247024b4a58829e0975cb6e3`
  - Final slice merge HEAD: `3fac2e4c02ae7e8b8e14ebcd084e15e448ed9a9e`
  - Key code paths:
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/shell/src/builtins/world_deps/runner.rs`
    - `crates/shell/tests/world_deps_apt_install_wdp5.rs`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none in this slice)

## Checks Run (Evidence)

- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass
  - `cargo test -p shell --test world_deps_apt_install_wdp5 -- --nocapture` (pass)
- `make integ-checks`: pass

## Cross-Platform Smoke (if applicable)

Record run ids/URLs for required platforms:

- Linux: Feature Smoke run `22024645441` — https://github.com/atomize-hq/substrate/actions/runs/22024645441 (success)
- macOS: Feature Smoke run `22024645441` — https://github.com/atomize-hq/substrate/actions/runs/22024645441 (success)
- WSL: Feature Smoke run `22024645441` — https://github.com/atomize-hq/substrate/actions/runs/22024645441 (success)

## Smoke ↔ Manual Parity

- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)

Notes:

- Checkpoint CP2 evidence: CI compile parity run `22024610209` — https://github.com/atomize-hq/substrate/actions/runs/22024610209 (success)
- Platform-fix tasks `WDP5-integ-linux` and `WDP5-integ-macos` were marked completed as deterministic no-ops based on all-green Feature Smoke for `CHECKOUT_SHA=bd4429b3a2554ab5247024b4a58829e0975cb6e3`.
