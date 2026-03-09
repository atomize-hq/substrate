---
codename: detecting_badger
created: "2026-02-20T18:57:43Z"
status: ready_for_lockdown
depends_on: []
execution_order: 20
adr: ADR-0031
adr_path: docs/project_management/adrs/draft/ADR-0031-detecting-badger.md
workstream_id: WS-detecting_badger
lockdown_prompt: docs/project_management/system/prompts/discovery/adr_lockdown.md
---

# ADR Intake Sheet

## 1. Codename + date + status

- Codename: `detecting_badger`
- Created: 2026-02-20T18:57:43Z
- Status: ready_for_lockdown
- ADR draft: `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`

## 2. Working Title (tentative)

Best-effort distro + package-manager discovery during install (with explicit override flags)

## 3. Problem / Motivation (3–6 bullets)

- `scripts/substrate/install-substrate.sh` must install a small set of prerequisite commands on Linux hosts (e.g. `curl`, `tar`, `jq`, `systemctl`) before it can provision the world backend reliably.
- Today we infer the package manager primarily by “which command exists” (e.g. `apt-get`, `dnf`, `yum`, `pacman`, `zypper`) and optionally via the environment variable `PKG_MANAGER`.
- In real environments (containers, mixed images, custom PATHs, developer workstations), multiple package manager commands can exist or be aliased, making “first one found wins” brittle and hard to debug.
- We don’t have a first-class, user-facing “what distro did we detect / what package manager are we using” report during install, so support/debug time increases when installs fail.
- We need a best-effort way to detect the distro/family (primarily for diagnostics and safer defaults) while still allowing explicit user override.
- Longer-term, we need clarity on what distros/package managers we consider “supported enough” for auto-installing prerequisites (e.g., Debian/Ubuntu=`apt`, Fedora/RHEL=`dnf/yum`, Arch/Manjaro=`pacman`, openSUSE=`zypper`, Alpine=`apk` (currently not supported)).

## 4. Proposed Outcome (1–3 bullets)

- During Linux installs, the installer prints a clear one-liner indicating detected distro (best-effort) and chosen package manager.
- The installer accepts explicit flags to override discovery (at minimum `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`).
- If discovery is ambiguous or unsupported, the installer fails with actionable guidance (how to override or what commands to install manually).

## 5. Non-Goals (explicit)

- Changing Substrate runtime behavior, world isolation, or world-deps semantics.
- Introducing a new general-purpose “package manager abstraction” library across the Rust workspace (this is installer-script scoped).
- Adding support for every distro/package manager in one shot; keep the initial slice small and additive.
- Making the installer silently succeed by skipping prerequisites; failures should be explicit and actionable.

## 6. Constraints / Invariants (security, UX, compatibility, performance)

- **Security/explicitness:** privileged operations remain explicit (uses `sudo` when needed); no hidden privilege escalation.
- **Compatibility:** keep existing behavior working by default; new flags are additive and optional.
- **Determinism:** when a flag is provided, it must win over autodetection.
- **UX:** always print “detected/selected” info before attempting installs; errors must include the remediation command.
- **Performance:** detection should be O(1) reads (e.g., `/etc/os-release`) and avoid network calls.

## 7. Interfaces / Contracts (CLI/config/API/files/events) — list concrete changes

Linux installer (`scripts/substrate/install-substrate.sh`):
- Add flags (names tentative):
  - `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
- Detection behavior (best-effort):
  - Read `/etc/os-release` when present and extract `ID` / `ID_LIKE` for diagnostics.
  - Choose a package manager using:
    1) explicit `--pkg-manager` (if provided),
    2) env `PKG_MANAGER` (existing),
    3) distro-family mapping (best-effort; locked mapping set below),
    4) command-exists fallback (current behavior).
- Output contract:
  - Print: `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager>`

Locked initial distro-family mapping set (Candidate A):
- Debian/Ubuntu family → `apt-get`
  - Match if `ID` in `{debian,ubuntu,linuxmint,pop}` or `ID_LIKE` contains `debian` or `ubuntu`.
- Fedora/RHEL family → prefer `dnf` (fallback `yum` if `dnf` missing)
  - Match if `ID` in `{fedora,rhel,centos,rocky,almalinux,ol,amzn}` or `ID_LIKE` contains `fedora` or `rhel`.
- Arch family → `pacman`
  - Match if `ID` in `{arch,manjaro,endeavouros,arcolinux,artix,garuda}` or `ID_LIKE` contains `arch`.
  - Note: popular “alternatives” like `yay`, `paru`, and `pamac` are wrappers/AUR helpers around `pacman`, not the base system package manager. Candidate A provisions prerequisites via `pacman` only; AUR support is out-of-scope.
- SUSE family → `zypper`
  - Match if `ID` contains `suse` or `opensuse` OR `ID_LIKE` contains `suse` or `opensuse`.

## 8. Options (at least 2)

### Option 1 — Add `--pkg-manager` override + keep current “command exists” detection (smallest delta)

**Description (1 paragraph)**
Keep discovery the same (first supported package manager command found in PATH), but add an explicit `--pkg-manager`
flag and print the selected value. If multiple managers exist, the user can disambiguate with the flag.

**Pros**
- Lowest implementation risk; minimal code changes.
- Works even when `/etc/os-release` is missing/unreliable (some containers).

**Cons**
- Still “guessy” by default; doesn’t leverage distro-family info to pick a safer default.
- Doesn’t help us reason about “supported distros” beyond “command exists”.

**Risk notes**
- Could mask an underlying broken PATH if the “wrong” manager happens to exist first.

### Option 2 — Best-effort distro detection via `/etc/os-release` + `--pkg-manager` override (recommended)

**Description (1 paragraph)**
Add distro detection by parsing `/etc/os-release` and mapping well-known distro families to a preferred package
manager. Still fall back to command-exists when mapping fails, and allow `--pkg-manager` to override everything.
Always print detected distro fields and chosen manager.

**Pros**
- More debuggable and safer defaults; aligns with operator expectations (“Ubuntu → apt”, “Arch → pacman”).
- Keeps explicit override for edge cases.

**Cons**
- Requires maintaining a mapping table and defining “supported enough” distro families.

**Risk notes**
- Some distros have `ID_LIKE` values that are broad or misleading; mapping must remain best-effort and fail-safe.

### Option 3 — Distro detection via external commands (`lsb_release`, `hostnamectl`) (avoid)

**Description (1 paragraph)**
Try to infer distro using `lsb_release`, `hostnamectl`, or other commands, then map to a package manager.

**Pros**
- Sometimes richer metadata than `/etc/os-release`.

**Cons**
- Adds more prerequisites and failure modes (those commands may not exist).
- More moving parts than needed.

**Risk notes**
- Circular dependency risk (needing a package manager to install the tool used to detect the distro).

## 9. Recommendation (tentative) + “Choose Option X when…”

Locked proposal: **Option 2**.

Choose Option 2 when we want more predictable defaults and better diagnostics without sacrificing explicit override.
Choose Option 1 when we want the smallest safe improvement (override + visibility) before adding mapping.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): installer prints distro + supports `--pkg-manager` override (best-effort mapping + fallback).
  - Slice 1: Add `/etc/os-release` parsing + “detected/selected” output; keep current behavior as fallback.
  - Slice 2: Add `--pkg-manager` flag and ensure it overrides env + autodetection.
  - Slice 3 (locked): Add a hermetic smoke test under `tests/installers/` that:
    - feeds fake `/etc/os-release` content (temp file),
    - uses a controlled `PATH` with stub `apt-get|dnf|yum|pacman|zypper` commands, and
    - asserts chosen `PKG_MANAGER` + `source` (`flag|env|os_release|path_probe`).
    Optional (not required): add a local container smoke harness (2 images: `ubuntu` + `archlinux`) runnable via `make installers-container-smoke` (do not run in CI by default).
- Candidate B (follow-up): persist detected distro/pkg-manager into `$SUBSTRATE_HOME/install_state.json` for later consumers (see `stashing_ferret`).
- Candidate C (follow-up): propagate the same detection/override semantics to dev-install flows where applicable (only if they install host prereqs).

## 11. Acceptance Criteria Draft (<= 8 items, observable outcomes)

- On Ubuntu/Debian hosts, the installer reports distro fields and selects `apt-get` by default.
- On Arch/Manjaro hosts, the installer reports distro fields and selects `pacman` by default.
- When `--pkg-manager pacman` is provided on a non-Arch host, the installer uses `pacman` (and fails with actionable guidance if it’s unavailable).
- When `PKG_MANAGER=dnf` is set, the installer uses `dnf` unless `--pkg-manager` is also provided.
- When distro detection fails, the installer still falls back to command-exists detection and prints what it chose.
- When no supported package manager can be chosen, the installer exits with an error that lists required commands and how to proceed (manual install or override).

## 12. Dependencies

- depends_on_adrs: []
- depends_on_work_items: []
- blocks: []
- Related intakes (coordination only): [`provisioning_otter`]

## 13. Lift Summary

### Lift Vector v1

<!-- PM_LIFT_VECTOR:BEGIN -->
```json
{
  "touch": {
    "create_files": 1,
    "edit_files": 1,
    "delete_files": 0,
    "deprecate_files": 0,
    "crates_touched": 0,
    "boundary_crossings": 0
  },
  "contract": {
    "cli_flags": 1,
    "config_keys": 0,
    "exit_codes": 0,
    "file_formats": 0,
    "behavior_deltas": 1
  },
  "qa": { "new_test_files": 1, "new_test_cases": 3 },
  "docs": { "new_docs_files": 0 },
  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },
  "risk": {
    "cross_platform": false,
    "security_sensitive": true,
    "concurrency_or_ordering": false,
    "migration_or_backfill": false,
    "unknowns_high": 0
  },
  "notes": "Estimate based on: add one user-facing installer flag + one hermetic installer test file."
}
```
<!-- PM_LIFT_VECTOR:END -->

### Computed outputs (from `make pm-lift-intake`)

```text
Lift Score (v1): 16
Estimated slices: 2
Confidence: high
```

## 14. Open Questions / Unknowns (with priority)

- (Locked) Initial distro-family mapping set is defined in Interfaces/Contracts.
- (Locked) No `--distro-id` flag in Candidate A; keep only `--pkg-manager` override.
- (Locked) Defer Alpine support (`apk`) to Candidate B.
- (Locked) Testing strategy: hermetic test under `tests/installers/`; optional container smoke in `.github/workflows/ci-testing.yml`.
- (Locked) Optional container smoke runs locally via `make installers-container-smoke` (not CI).

## 15. Ready-to-lockdown checklist (yes/no with reasons)

- [x] Candidate A behavior delta is locked (Option 2).
- [x] Flag names are locked (`--pkg-manager` required).
- [x] Initial distro-family mapping set is agreed (defined in Interfaces/Contracts).
- [x] Acceptance criteria align with the desired “best-effort + override” UX.
