---
seam_id: SEAM-04
seam_slug: validation-checkpoint
type: conformance
status: proposed
execution_horizon: future
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - SEAM-01
    - SEAM-02
    - SEAM-03
  required_threads:
    - THR-01
    - THR-02
    - THR-03
    - THR-04
    - THR-05
  stale_triggers:
    - Any upstream contract changes
    - Hermetic test harness changes
    - CI checkpoint requirements change
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-04 - Validation Checkpoint and Contract Lock-in

## Goal / value
Provide hermetic validation of all contracts and behaviors, execute the CI checkpoint, and lock the feature contracts as the authoritative v1 specification.

## Scope

### In
- Hermetic test harness (`tests/installers/pkg_manager_detection_smoke.sh`)
- Fake-os-release input (via `SUBSTRATE_INSTALL_OS_RELEASE_PATH`)
- Fake PATH with stub binaries
- Precedence chain validation
- Failure path validation
- Wrapper pass-through validation
- CI checkpoint execution (compile parity + behavior smoke)
- Evidence capture

### Out
- Core implementation (SEAM-01, SEAM-02, SEAM-03)
- Feature-local smoke wrapper (thin; calls harness)

## Primary interfaces

### Inputs
- All contracts from upstream seams
- Test fixtures (os-release samples, stub binaries)
- CI infrastructure

### Outputs
- Test results
- CI checkpoint evidence
- Validated contract seal

## Key invariants / rules

1. **Single authority**: Hermetic harness is the behavior authority
2. **Deterministic**: Same inputs produce same results
3. **Complete coverage**: All contracts have validation
4. **CI parity**: Compile and quick-test across Linux/macOS/Windows

## Dependencies

### Direct blockers
- SEAM-01 (C-01, C-02, C-03, C-06 via THR-01, THR-02)
- SEAM-02 (C-04, C-05 via THR-03, THR-04)
- SEAM-03 (C-07 via THR-05)

### Transitive blockers
- All upstream transitive dependencies

### Direct consumers
- None (terminal seam)

### Derived consumers
- Downstream pack (validated contracts for persistence)
- Future maintenance (regression prevention)

## Touch surface
- `tests/installers/pkg_manager_detection_smoke.sh` (hermetic test harness)
- `docs/project_management/packs/draft/best-effort-distro-package-manager-fse/smoke/linux-smoke.sh` (thin wrapper)
- CI configuration

## Verification

1. Hermetic test covers all four distro families
2. Precedence order validated: flag → env → os_release → path_probe
3. Invalid flag → exit 2
4. Missing forced manager → exit 3
5. No manager found → exit 4
6. Multi-manager warning emitted
7. Wrapper path preserves codes
8. `SUBSTRATE_INSTALL_OS_RELEASE_PATH` works for test injection
9. CI compile parity passes
10. CI testing quick passes
11. Linux behavior smoke passes

## Risks / unknowns

| Risk | De-risk plan |
|------|--------------|
| Test environment differences | Hermetic fixtures eliminate host dependency |
| CI flakiness | Deterministic fixtures; retry policy in CI |
| Coverage gaps | Contract-based test mapping |

## Rollout / safety
- Tests validate; they don't change behavior
- CI checkpoint gates promotion
- Evidence archive for audit

## Downstream decomposition context

### Why this seam is `future`
SEAM-04 is the conformance seam that validates everything. It depends on all upstream seams being complete. It will receive only provisional planning at extraction time, then detailed planning when promoted to `active` after SEAM-03 lands. The CI checkpoint is the final gate before the feature is considered complete.

### Which threads matter most
- All threads consumed: THR-01, THR-02, THR-03, THR-04, THR-05
- THR-06 continues to downstream pack

### What the first seam-local review should focus on
1. Hermetic test design (fixtures, assertions)
2. Coverage completeness against contracts
3. CI checkpoint criteria
4. Evidence capture and archiving
5. Downstream stale trigger registration

## Expected seam-exit concerns

### Contracts likely to publish
None (consumes all upstream contracts)

### Threads likely to advance
- All upstream threads: consumed → revalidated → closed
- THR-06: published → revalidated (for downstream pack)

### Review-surface areas likely to shift after landing
- Test harness becomes frozen reference
- CI checkpoint criteria locked

### Downstream seams most likely to require revalidation
- None (terminal seam)
- Downstream pack (`persist-detected-linux-distro-pkg-manager`) receives stale trigger if validation finds contract issues

## Checkpoint Specification

The checkpoint (`CP1`) runs after `SEAM-04` landing:

- Compile parity across linux, macos, windows
- CI Testing quick across linux, macos, windows
- Linux behavior smoke for validation seam

Commands:
```bash
make ci-compile-parity ...
make ci-testing ... CI_MODE=quick ...
make feature-smoke ... PLATFORM=behavior ...
```
