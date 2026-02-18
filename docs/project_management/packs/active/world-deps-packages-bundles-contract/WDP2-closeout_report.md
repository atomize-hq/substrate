# Slice Closeout Gate Report — world-deps-packages-bundles-contract / WDP2

Date (UTC): 2026-02-14T17:02:10Z

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`
- `docs/project_management/standards/EXECUTIVE_SUMMARY_STANDARD.md` (behavior delta format)

Feature directory:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract`

Slice spec:
- `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP2-spec.md`

## Behavior Delta (Existing → New → Why)

- Existing behavior:
  - Inventory and effective enabled views existed (WDP1), but there was no world-backed status surface for whether enabled items were present/blocked/missing in the active world.
  - `show` did not provide an `--explain` path that included world status + remediation.
- New behavior:
  - Adds world-backed status surfaces:
    - `substrate world deps current list applied` (default scope: current effective enabled set; includes `world=present|missing|blocked`)
    - `substrate world deps current list applied --all` (includes all visible inventory items)
    - `substrate world deps current show <item> --explain` (includes enabled provenance + world status + one-line remediation when not present)
  - Enforces fail-closed posture for world-backed reads: exit `3` when the backend is unavailable (actionable remediation in output).
- Why:
  - Completes the read-only workflow (discover → enable → observe applied status) and stabilizes exit-code semantics before install/sync mutation slices (WDP3+).
- Links:
  - Contract: `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`
  - Spec: `docs/project_management/packs/active/world-deps-packages-bundles-contract/WDP2-spec.md`
  - Implementation: `crates/shell/src/builtins/world_deps/surfaces.rs`
  - Tests: `crates/shell/tests/world_deps_applied_wdp2.rs`

## Spec Parity (No Drift)

- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none)

## Checks Run (Evidence)

- `cargo fmt`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass
  - `cargo test -p shell --test world_deps_applied_wdp2 -- --nocapture`
  - `cargo test -p shell --test replay_world -- --nocapture`
- `make integ-checks`: pass

## Cross-Platform Smoke (if applicable)

Record run ids/URLs for required platforms:
- Linux: run `22019947173` — https://github.com/atomize-hq/substrate/actions/runs/22019947173
- macOS: run `22019947173` — https://github.com/atomize-hq/substrate/actions/runs/22019947173
- WSL: run `22020059723` — https://github.com/atomize-hq/substrate/actions/runs/22020059723

## Smoke ↔ Manual Parity

- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)

Notes:
- Feature Smoke evidence is split across runs due to a self-hosted macOS runner outage during the checkpoint window (see `docs/project_management/packs/active/world-deps-packages-bundles-contract/session_log.md`).
