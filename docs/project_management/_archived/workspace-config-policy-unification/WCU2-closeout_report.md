# Slice Closeout Gate Report — workspace-config-policy-unification / WCU2

Date (UTC): 2026-01-16

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/workspace-config-policy-unification/`

Slice spec:
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`

## Behavior Delta (Existing → New → Why)
- Existing behavior: `config current show --explain` output ordering and WCU2 Phase A provenance coverage were incomplete for multi-layer derived keys.
- New behavior: Per-key merge strategy is applied for `world.deps.*`, `world.deps.enabled` merges (concat+dedupe) across global/workspace with deterministic multi-source provenance, and explain keys serialize in lexicographic dotpath order.
- Why: ADR-0012 Phase A contract for per-key merge strategies and deterministic multi-source provenance.
- Links: `docs/project_management/_archived/workspace-config-policy-unification/WCU2-spec.md`, `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/_archived/workspace-config-policy-unification/PHASE_A_B_GATES_ADR_0012.md`

## Spec Parity (No Drift)
- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none during this slice)

## Checks Run (Evidence)
- `cargo fmt --all`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass
  - `cargo test -p substrate-shell -- --nocapture`
  - `cargo test -p substrate --test wcu2_explain_determinism -- --nocapture`
- `make integ-checks`: pass

## Cross-Platform Smoke (if applicable)
- Run: `21052957916` (`https://github.com/atomize-hq/substrate/actions/runs/21052957916`) → `success`
- Linux: pass
- macOS: pass
- Windows: pass

## Smoke ↔ Manual Parity
- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)
