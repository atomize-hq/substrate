---
seam_id: SEAM-01
seam_slug: distro-detection
type: capability
status: proposed
execution_horizon: active
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - os-release file format standard changes
    - New distro families requiring mapping
    - Parser security findings
    - Downstream pack contract mismatches
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

# SEAM-01 - Distro Detection and Mapping

## Goal / value
Implement safe, deterministic Linux distro detection via `/etc/os-release` parsing without shell execution, and establish the distro-to-package-manager mapping table with stable decision-line output.

## Scope

### In
- Line-oriented os-release parser for `ID` and `ID_LIKE` keys only
- Safe parsing rules (no shell execution, no variable expansion)
- Value normalization (lowercase, quote stripping)
- `<unknown>` sentinel for missing data
- Distro family mapping table:
  - Debian/Ubuntu → apt-get
  - Fedora/RHEL → dnf (or yum fallback)
  - Arch → pacman
  - SUSE → zypper
- Stable stderr decision-line format and placement
- `SUBSTRATE_INSTALL_OS_RELEASE_PATH` test hook contract

### Out
- Override mechanisms (in SEAM-02)
- Failure handling beyond `<unknown>` sentinel (in SEAM-02)
- Wrapper changes (in SEAM-03)
- Hermetic test harness (in SEAM-04)

## Primary interfaces

### Inputs
- `/etc/os-release` file (or alternate from `SUBSTRATE_INSTALL_OS_RELEASE_PATH`)
- `PATH` environment variable (for availability checks, not selection)

### Outputs
- `distro_id` (normalized or `<unknown>`)
- `distro_id_like` (normalized or `<unknown>`)
- `pkg_manager.selected` (if mapping selects one)
- `pkg_manager.source` (`os_release` or unset)
- stderr decision line (before prerequisite installation)

## Key invariants / rules

1. **Safety**: Parser never executes shell code from os-release file
2. **Idempotency**: Same input produces same output
3. **Bounded**: Only reads `ID` and `ID_LIKE` keys
4. **Deterministic**: Last well-formed key value wins
5. **Stable**: Decision-line format is contractual

## Dependencies

### Direct blockers
- None (foundation seam)

### Transitive blockers
- None

### Direct consumers
- SEAM-02 (consumes parsed data, mapping table)
- SEAM-04 (consumes test hook contract)

### Derived consumers
- Downstream pack (persistence layer)

## Touch surface
- `scripts/substrate/install-substrate.sh` (os-release parser, mapping logic, decision line)
- `contract.md` (seal C-01, C-02, C-03, C-06 contracts)

## Verification

1. Parser produces expected `distro_id`/`distro_id_like` for sample os-release files
2. Debian-family input → `apt-get` selected
3. Arch-family input → `pacman` selected
4. Fedora-family with `dnf` in PATH → `dnf` selected
5. Fedora-family with only `yum` in PATH → `yum` selected
6. Missing/unreadable os-release → `<unknown>` fields, continues to fallback
7. Decision line appears exactly once, before prerequisite install
8. `SUBSTRATE_INSTALL_OS_RELEASE_PATH` alternate path works
9. Invalid alternate path → deterministic absence behavior

## Risks / unknowns

| Risk | De-risk plan |
|------|--------------|
| Edge-case os-release format | Parser contract defines explicit rules; hermetic tests cover edge cases |
| Downstream contract drift | Thread THR-06 publishes contracts; stale trigger raised on changes |
| SUSE family matching | Pattern-based matching on ID_LIKE tokens; unit tests required |

## Rollout / safety
- Linux-only; macOS/Windows no-change contract preserved
- No breaking changes to existing installer paths
- New behavior gated behind os-release presence

## Downstream decomposition context

### Why this seam is `active`
SEAM-01 provides the foundation that all subsequent seams depend on. The os-release parser, mapping table, and vocabulary contracts must be stable before override mechanisms (SEAM-02) or failure handling can be built on top. The downstream pack (`persist-detected-linux-distro-pkg-manager`) is blocked waiting for these contracts.

### Which threads matter most
- THR-01: Core detection data and vocabulary to all downstream seams
- THR-02: Test hook for hermetic validation
- THR-06: Cross-pack contract to persistence layer

### What the first seam-local review should focus on
1. Parser safety posture and rule completeness
2. Mapping table coverage for target distro families
3. Decision-line format stability
4. `<unknown>` sentinel semantics
5. Test hook contract appropriateness

## Expected seam-exit concerns

### Contracts likely to publish
- C-01: os-release parsing contract
- C-02: Selected-manager and source vocabulary
- C-03: Decision-line format contract
- C-06: `SUBSTRATE_INSTALL_OS_RELEASE_PATH` test hook contract

### Threads likely to advance
- THR-01: identified → defined → published
- THR-02: identified → defined
- THR-06: identified → defined (for downstream pack)

### Review-surface areas likely to shift after landing
- Decision-line format becomes contractual (immutable)
- Mapping table is locked for v1
- Parser rules are sealed

### Downstream seams most likely to require revalidation
- SEAM-02 if mapping table changes
- SEAM-04 if test hook contract changes
- Downstream pack if vocabulary changes
