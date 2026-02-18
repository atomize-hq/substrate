# Slice Closeout Gate Report — workspace-config-policy-unification / WCU3

Date (UTC): 2026-01-16

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/workspace-config-policy-unification/`

Slice spec:
- `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/adrs/implemented/ADR-0012-config-schema-per-key-merge-and-provenance.md`

## Behavior Delta (Existing → New → Why)
- Existing behavior:
  - `substrate config` editor rejected/failed to handle Phase B world-deps keys and list mutations.
- New behavior:
  - Config editor supports list edits for `world.deps.enabled` (`+=` append, `-=` remove, `reset`) and allowlists enum keys `world.deps.inventory_mode` and `world.deps.builtins`.
  - `substrate config current show --explain` orders details by layer precedence.
- Why:
  - Implements ADR-0012 Phase B and WCU3 spec requirements (world deps keys + deterministic explain/provenance).
- Links:
  - WCU3 spec: `docs/project_management/_archived/workspace-config-policy-unification/WCU3-spec.md`
  - Phase A/B gates: `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`
  - Orchestration branch HEAD: `7ba0d1245ac83d4e746db608a2b645f29018c8e5`

## Spec Parity (No Drift)
- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale)

## Checks Run (Evidence)
- `cargo fmt`: `PASS` (included in `make integ-checks`)
- `cargo clippy --workspace --all-targets -- -D warnings`: `PASS` (included in `make integ-checks`)
- Relevant tests: `PASS`
  - `cargo test -p substrate-shell`
- `make integ-checks`: `PASS`
  - (Runs `cargo fmt`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo check --workspace --all-targets`, `cargo test --workspace --all-targets`)

## Cross-Platform Smoke (if applicable)
- Run: `21055761993` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21055761993
- Linux: pass
- macOS: pass
- Windows: pass
- Run: `21055761993` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21055761993
- Linux: `linux_self_hosted` (`success`)
- macOS: `macos_self_hosted` (`success`)
- Windows: `windows_self_hosted` (`success`)

## Smoke ↔ Manual Parity
- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)
