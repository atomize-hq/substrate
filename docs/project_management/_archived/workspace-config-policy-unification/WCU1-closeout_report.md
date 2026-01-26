# Slice Closeout Gate Report — workspace-config-policy-unification / WCU1

Date (UTC): 2026-01-16

Standards:
- `docs/project_management/standards/SLICE_CLOSEOUT_GATE_STANDARD.md`

Feature directory:
- `docs/project_management/_archived/workspace-config-policy-unification/`

Slice spec:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

## Behavior Delta (Existing → New → Why)
- Existing behavior: Workspace state and internal git location were not unified under a single canonical `.substrate/` workspace root, and nested workspace init refusal/disable-marker semantics were not enforced per ADR-0008.
- New behavior: Workspace state is canonicalized under `<workspace_root>/.substrate/` (workspace + policy patches + internal git), discovery respects `.substrate/workspace.disabled`, and nested `workspace init` is refused deterministically (exit `2`).
- Why: Implements ADR-0008 workspace scope and on-disk layout contract.
- Links: `docs/project_management/_archived/workspace-config-policy-unification/WCU1-spec.md`, `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

## Spec Parity (No Drift)
- [x] Acceptance criteria satisfied
- [x] Any spec changes during the slice are recorded (with rationale) (none during this slice)

## Checks Run (Evidence)
- `cargo fmt --all`: pass (validated during slice integration; also green in later full integration gates)
- `cargo clippy --workspace --all-targets -- -D warnings`: pass (validated during slice integration; also green in later full integration gates)
- Relevant tests: pass (covered by `make integ-checks` for the feature’s final integration)
- `make integ-checks`: pass (feature final integration HEAD `97fc1f457d82b1c99cf0959c66585ce933cdf547`)

## Cross-Platform Smoke (if applicable)
- Run: `21042397938` (`success`) — https://github.com/atomize-hq/substrate/actions/runs/21042397938
- Linux: pass
- macOS: pass
- Windows: pass

## Smoke ↔ Manual Parity
- [x] Smoke scripts run the same commands/workflows as the manual testing playbook (minimal viable subset)
- [x] Smoke scripts validate exit codes and key output (not just “command ran”)
