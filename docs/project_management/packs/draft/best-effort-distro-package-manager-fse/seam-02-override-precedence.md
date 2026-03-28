---
seam_id: SEAM-02
seam_slug: override-precedence
type: capability
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - SEAM-01
  required_threads:
    - THR-01
  stale_triggers:
    - SEAM-01 contracts change
    - Exit-code taxonomy shared standard changes
    - Precedence chain order changes
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

# SEAM-02 - Override Precedence and Fallback

## Goal / value
Implement explicit override mechanisms with deterministic precedence, deterministic PATH fallback, multi-manager ambiguity resolution, and feature-specific failure classes with actionable remediation.

## Scope

### In
- `--pkg-manager <manager>` CLI flag (highest precedence)
- `PKG_MANAGER=<manager>` env var (second precedence)
- Availability checking (`command -v`)
- Fixed PATH probe order: apt-get → dnf → yum → pacman → zypper
- Multi-manager PATH warning and deterministic selection
- Exit code `2`: Invalid override value
- Exit code `3`: Forced manager missing from PATH
- Exit code `4`: No supported manager selected
- Remediation content for all failure classes

### Out
- os-release parsing (in SEAM-01)
- Wrapper pass-through (in SEAM-03)
- Hermetic test harness (in SEAM-04)

## Primary interfaces

### Inputs
- CLI arguments (including `--pkg-manager`)
- Environment variables (`PKG_MANAGER`)
- os-release mapped manager (from SEAM-01 via THR-01)
- `PATH` environment variable

### Outputs
- Selected manager (via precedence chain)
- `pkg_manager.source` (flag | env | path_probe)
- Exit codes 0, 2, 3, 4
- stderr: Multi-manager warning (when applicable)
- stderr: Remediation content (on failures)

## Key invariants / rules

1. **Precedence**: flag > env > os_release > path_probe
2. **Forced selection is final**: Valid flag/env never falls back
3. **Deterministic order**: Fixed probe order, not PATH order
4. **Ambiguity warning**: Visible warning when multiple managers present
5. **Fail-closed**: No silent fallback after explicit override failure

## Dependencies

### Direct blockers
- SEAM-01 (provides os_release mapping, C-02 vocabulary via THR-01)

### Transitive blockers
- None

### Direct consumers
- SEAM-03 (needs exit-code taxonomy)
- SEAM-04 (needs validation coverage)

### Derived consumers
-Operator docs (remediation content)

## Touch surface
- `scripts/substrate/install-substrate.sh` (argument parsing, precedence chain, failure paths)
- `contract.md` (seal C-04, C-05 contracts)

## Verification

1. `--pkg-manager apt-get` overrides all other sources
2. `--pkg-manager invalid` exits 2 with remediation
3. `PKG_MANAGER=dnf` overrides os-release/PATH
4. `PKG_MANAGER=invalid` exits 2 with remediation
5. `--pkg-manager pacman` when pacman not in PATH → exits 3 with remediation
6. No supported managers in PATH → exits 4 with remediation
7. Multiple managers (apt-get + dnf) → issues warning, selects earliest in order
8. Precedence chain: flag wins over env wins over os_release wins over PATH

## Risks / unknowns

| Risk | De-risk plan |
|------|--------------|
| Operator confusion about override precedence | Clear documentation, decision line shows source |
| Tooling scripts depend on old behavior | No-change contract for non-Linux; Linux change is additive |
| Exit-code collision with other features | Aligned with shared EXIT_CODE_TAXONOMY.md |

## Rollout / safety
- Fail-closed behavior prevents silent wrong-manager installs
- Explicit override always visible in decision line
- Remediation text guides operators to fix

## Downstream decomposition context

### Why this seam is `next`
SEAM-02 builds on SEAM-01 foundation and provides the complete failure taxonomy. It can receive provisional planning attention but should not get authoritative sub-slices until SEAM-01 lands. The exit codes and precedence chain are needed by SEAM-03 (wrapper) and SEAM-04 (validation).

### Which threads matter most
- THR-01: Consumes detection data for fallback path
- THR-03: Publishes failure taxonomy to SEAM-03
- THR-04: Validation coverage thread to SEAM-04

### What the first seam-local review should focus on
1. Precedence chain completeness and edge cases
2. Exit-code meaning alignment with shared taxonomy
3. Remediation content actionability
4. Multi-manager warning clarity
5. PATH probe order justification

## Expected seam-exit concerns

### Contracts likely to publish
- C-04: Exit-code taxonomy for package-manager failures
- C-05: Multi-manager warning template and order

### Threads likely to advance
- THR-01: Consumed (revalidated if contracts change)
- THR-03: identified → defined → published
- THR-04: identified → defined

### Review-surface areas likely to shift after landing
- Exit-code meanings become contractual
- Warning template is locked for v1
- Precedence chain is sealed

### Downstream seams most likely to require revalidation
- SEAM-03 if exit codes change
- SEAM-04 if validation coverage gaps found
