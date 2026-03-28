---
pack_id: best-effort-distro-package-manager-fse
pack_version: v1
pack_status: extracted
source_ref: docs/project_management/packs/draft/best-effort-distro-package-manager/
execution_horizon:
  active_seam: SEAM-01
  next_seam: SEAM-02
---

# Scope Brief - Best-Effort Distro Package Manager

## Goal
Ship the ADR-0031 installer contract as four bounded seams that preserve Linux-only behavior change, keep operator docs aligned, and end with one checkpoint-backed validation seam.

## Why now
The Substrate installer needs explicit package-manager detection and override capabilities for Linux hosts. Currently, detection is implicit and operators lack visibility into or control over package manager selection. This feature provides deterministic detection, explicit overrides, and stable failure semantics.

## Primary user(s) + JTBD
- **Primary user**: Linux system administrators and developers installing Substrate on Linux hosts
- **Job-to-be-done**:
  - See which package manager was selected and why
  - Override automatic selection when needed
  - Receive actionable error messages when detection fails
  - Ensure consistent prerequisite installation across diverse Linux distributions

## In-scope
- Linux hosted package-manager detection via `/etc/os-release` parsing
- Explicit override mechanisms (`--pkg-manager` flag, `PKG_MANAGER` env var)
- Deterministic fallback `PATH` probing with precedence order
- Multi-manager `PATH` ambiguity warning and resolution
- Stable stderr decision-line reporting
- Feature-specific exit codes (`2`, `3`, `4`) with remediation content
- Wrapper exit-status pass-through (`scripts/substrate/install.sh`)
- Operator documentation updates (`docs/INSTALLATION.md`, `docs/reference/env/contract.md`)
- Hermetic validation via stubbed harness
- CI checkpoint at validation seam

## Out-of-scope
- macOS behavior changes (no delta)
- Windows behavior changes (no delta)
- Runtime world-deps behavior changes
- Persistence into `install_state.json` (owned by downstream pack)
- In-world guest provisioning semantics
- `scripts/substrate/dev-install-substrate.sh` changes
- New config file or persistent config key
- Structured trace/log field additions
- Network calls for detection

## Success criteria
1. Linux installers print stable decision line showing distro ID, ID_LIKE, selected manager, and source
2. `--pkg-manager` and `PKG_MANAGER` overrides work with proper precedence
3. Invalid override values exit `2` with actionable remediation
4. Forced manager missing from `PATH` exits `3` with actionable remediation
5. No supported manager selected exits `4` with actionable remediation
6. Multi-manager `PATH` scenarios emit warning and deterministic selection
7. Wrapper (`install.sh`) preserves feature-specific exit codes
8. Operator docs reflect new CLI, env vars, and failure classes
9. Hermetic tests validate precedence, mapping, and failure paths
10. CI checkpoint validates compile parity and behavior smoke across platforms

## Constraints
- Linux-only behavior change (macOS/Windows: no-change contract)
- No new config file or persistent config key
- No runtime crate changes
- No network calls during detection
- Safe parsing of `/etc/os-release` (no shell execution)
- Preserve source-time invariants in `install-substrate.sh` (world-enable sources this file)

## External systems / dependencies
- `/etc/os-release` file format and location on Linux hosts
- `PATH` environment variable and available package manager binaries
- Downstream pack: `persist-detected-linux-distro-pkg-manager` (persistence contract)
- Related ADRs: ADR-0032 (stashing ferret), ADR-0030 (provisioning otter), ADR-0035 (summoning wombat)
- CI infrastructure for compile parity and behavior smoke

## Known unknowns / risks
| Risk | De-risk plan |
|------|--------------|
| Parser drift between production and test | Single parser implementation in installer, inherited by test harness |
| Cross-platform CI parity inflation | Explicit Linux-only task model, CI parity for other platforms without behavior change evidence |
| Downstream pack contract alignment | `basis.stale_triggers` registered; `pkg_manager.source`, `<unknown>` sentinel, `SUBSTRATE_INSTALL_OS_RELEASE_PATH` contracts published |
| Wrapper pass-through regression | Explicit acceptance criteria for wrapper path in hermetic tests |
| os-release file format edge cases | Parser contract defines exact rules for comments, quotes, duplicates, normalization |

## Assumptions
- Host Linux systems follow os-release standard for `ID` and `ID_LIKE` fields
- CI environment can provide stub PATH binaries for hermetic testing
- Operators prefer stable, deterministic behavior over attempting to auto-select "best" manager
- The four canonical distro families (Debian/Ubuntu, Fedora/RHEL, Arch, SUSE) cover majority of target hosts
