---
seam_id: SEAM-03
seam_slug: wrapper-integration
type: integration
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
  required_threads:
    - THR-01
    - THR-03
  stale_triggers:
    - SEAM-02 exit-code contracts change
    - Wrapper implementation changes
    - Operator doc standard changes
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

# SEAM-03 - Wrapper Integration and Operator Documentation

## Goal / value
Ensure the wrapper entrypoint (`install.sh`) preserves the feature-specific exit-code taxonomy, and align all operator-facing documentation with the implemented contracts.

## Scope

### In
- `scripts/substrate/install.sh` exit-code pass-through for codes 0, 2, 3, 4
- `docs/INSTALLATION.md` updates (flag, decision line, precedence, remediation)
- `docs/reference/env/contract.md` updates (`PKG_MANAGER`, `SUBSTRATE_INSTALL_OS_RELEASE_PATH`)
- Contract reuse without drift

### Out
- Core detection logic (in SEAM-01)
- Override mechanisms (in SEAM-02)
- Hermetic validation (in SEAM-04)

## Primary interfaces

### Inputs
- Upstream installer exit status (from SEAM-02 via THR-03)
- Contracts C-03 (decision line), C-04 (exit codes), C-06 (env var)

### Outputs
- Wrapper exit status matching upstream for feature codes
- Updated operator documentation

## Key invariants / rules

1. **Pass-through preservation**: Wrapper must not collapse 2/3/4 to 1
2. **Doc alignment**: Docs must reuse contracts, not redefine
3. **Linux-only labeling**: Explicit no-change for macOS/Windows

## Dependencies

### Direct blockers
- SEAM-01 (decision line contract C-03)
- SEAM-02 (exit-code contract C-04 via THR-03)

### Transitive blockers
- SEAM-01 (via SEAM-02 dependency chain)

### Direct consumers
- SEAM-04 (wrapper contract C-07 for validation)

### Derived consumers
- Operators (via documentation)

## Touch surface
- `scripts/substrate/install.sh` (exit-status handling)
- `docs/INSTALLATION.md` (Linux install flow, offline wrapper flow, options table)
- `docs/reference/env/contract.md` (`PKG_MANAGER`, `SUBSTRATE_INSTALL_OS_RELEASE_PATH`)

## Verification

1. Wrapper exits 0 when installer exits 0
2. Wrapper exits 2 when installer exits 2
3. Wrapper exits 3 when installer exits 3
4. Wrapper exits 4 when installer exits 4
5. `docs/INSTALLATION.md` describes `--pkg-manager` flag
6. `docs/INSTALLATION.md` describes decision line
7. `docs/INSTALLATION.md` describes override precedence
8. `docs/INSTALLATION.md` includes Linux-only scope statement
9. `docs/reference/env/contract.md` defines `PKG_MANAGER`
10. `docs/reference/env/contract.md` defines `SUBSTRATE_INSTALL_OS_RELEASE_PATH`

## Risks / unknowns

| Risk | De-risk plan |
|------|--------------|
| Existing wrapper error handling | Explicit pass-through for known codes only; other codes use existing behavior |
| Doc drift from contracts | Doc updates required to reference contracts, not restate |
| Translation/translation delays | v1 English only; translations after contract stabilizes |

## Rollout / safety
- Wrapper change is additive (preserving more information)
- Doc changes describe existing behavior
- Rollback: revert wrapper to collapse codes (not recommended after operator dependency)

## Downstream decomposition context

### Why this seam is `future`
SEAM-03 depends on SEAM-02 for stable exit-code contracts. It will be promoted from `future` to `next` when SEAM-02 lands. At extraction time, it receives only seam-brief depth planning. Provisional candidate subslices will be created during seam-local planning.

### Which threads matter most
- THR-03: Exit-code taxonomy from SEAM-02
- THR-05: Wrapper contract to SEAM-04

### What the first seam-local review should focus on
1. Wrapper pass-through implementation approach
2. Doc organization and contract reference structure
3. Linux-only labeling consistency
4. Backward compatibility considerations

## Expected seam-exit concerns

### Contracts likely to publish
- C-07: Wrapper exit-status pass-through contract

### Threads likely to advance
- THR-03: consumed (revalidated if contracts change)
- THR-05: identified → defined → published

### Review-surface areas likely to shift after landing
- Wrapper behavior is locked for feature codes
- Doc structure for installer options

### Downstream seams most likely to require revalidation
- SEAM-04 if wrapper contract changes
