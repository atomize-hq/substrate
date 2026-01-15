# Slice Closeout Gate Report — workspace-config-policy-unification / WCU5

Date (UTC): <YYYY-MM-DD>

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/next/workspace-config-policy-unification/`

Slice spec:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`

## Behavior Delta (Existing → New → Why)
- Existing behavior:
- New behavior:
- Why:
- Links:

## Spec Parity (No Drift)
- [ ] Acceptance criteria satisfied
- [ ] Any spec changes during the slice are recorded (with rationale)

## Checks Run (Evidence)
- `cargo fmt`: pass/fail
- `cargo clippy --workspace --all-targets -- -D warnings`: pass/fail
- Relevant tests: pass/fail (list suites/commands)
- `make integ-checks`: pass/fail

## Cross-Platform Smoke (if applicable)
- Linux:
- macOS:
- Windows:

## Smoke ↔ Manual Parity
- [ ] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [ ] Smoke scripts validate exit codes and key output (not just “command ran”)
