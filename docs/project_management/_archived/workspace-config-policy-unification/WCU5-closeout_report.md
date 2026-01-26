# Slice Closeout Gate Report — workspace-config-policy-unification / WCU5

Date (UTC): 2026-01-16

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/workspace-config-policy-unification/`

Slice spec:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`

## Behavior Delta (Existing → New → Why)
- Existing behavior: Manual testing playbook + smoke scripts assert ADR-0012 Phase A/B behaviors for `world.deps.*` (merge semantics + provenance) and `config current show --explain` determinism.
- New behavior: No new behavior added in WCU5; validated and kept parity/evidence green after late-cycle output contract adjustments (`--explain` note on stderr + patch-view behavior) and updated tests accordingly.
- Why: WCU5 scope is parity + validation evidence for an already-defined contract (see WCU5 spec).
- Links: `docs/project_management/_archived/workspace-config-policy-unification/WCU5-spec.md`, `docs/project_management/_archived/workspace-config-policy-unification/manual_testing_playbook.md`, `docs/project_management/_archived/workspace-config-policy-unification/smoke/linux-smoke.sh`, `docs/project_management/_archived/workspace-config-policy-unification/smoke/macos-smoke.sh`, `docs/project_management/_archived/workspace-config-policy-unification/smoke/windows-smoke.ps1`

## Spec Parity (No Drift)
- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none during this slice)

## Checks Run (Evidence)
- `cargo fmt --all`: pass
- `cargo clippy --workspace --all-targets -- -D warnings`: pass
- Relevant tests: pass
  - `cargo test -p substrate-shell --tests`
  - `cargo test --test wcu2_explain_determinism`
- `make integ-checks`: pass
- Local behavioral smoke preflight (Linux): pass
  - `bash docs/project_management/_archived/workspace-config-policy-unification/smoke/linux-smoke.sh`

## Cross-Platform Smoke (if applicable)
- Run: `21076914464` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21076914464
- Linux: pass
- macOS: pass
- Windows: pass
- (Earlier WCU5 smoke run during integ-core): `21076323809` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21076323809

## Smoke ↔ Manual Parity
- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)
