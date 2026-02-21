# ADR-0031 — Best-effort Linux distro + package-manager discovery during install (with explicit override)

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): ASSUMPTION: Substrate maintainers

## Scope
- Feature directory: `docs/project_management/packs/active/detecting-badger/` (ASSUMPTION: created during planning)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Related Docs
- Intake: `docs/project_management/intake/adrs/detecting_badger_adr_intake.md`
- Plan: `docs/project_management/packs/active/detecting-badger/plan.md` (planned)
- Tasks: `docs/project_management/packs/active/detecting-badger/tasks.json` (planned)
- Spec manifest: `docs/project_management/packs/active/detecting-badger/spec_manifest.md` (planned)
- Decision Register: `docs/project_management/packs/active/detecting-badger/decision_register.md` (required; see “Decision Summary”)
- Impact Map: `docs/project_management/packs/active/detecting-badger/impact_map.md` (required; install UX + support/debug implications)

## Executive Summary (Operator)

ADR_BODY_SHA256: a7e0e960300cb3ae065964311b001c40db2f664bbe55426171b6226ff39f3f85

### Changes (operator-facing)
- Linux installer prints detected distro and chosen package manager, and supports explicit override
  - Existing: Installer best-effort selects a package manager by probing commands in `PATH`; selection can be brittle when multiple package managers exist/are aliased; it does not clearly report the distro/package-manager decision.
  - New: Installer reads `/etc/os-release` (best-effort) for `ID` / `ID_LIKE`, chooses a default package manager using a small distro-family mapping, and prints a stable one-liner before installing prerequisites. Operator can force the selection via `--pkg-manager …` (highest precedence) or `PKG_MANAGER=…` (legacy).
  - Why: Reduces install failures and support time by making the “what did we detect / what are we doing” step explicit, while keeping escape hatches for mixed/containerized environments.
  - Links:
    - `scripts/substrate/install-substrate.sh`
    - `docs/project_management/intake/adrs/detecting_badger_adr_intake.md`

## Problem / Context
- `scripts/substrate/install-substrate.sh` must install prerequisite commands on Linux hosts (e.g., `curl`, `tar`, `jq`, `systemctl`) before provisioning can run reliably.
- Current “first supported package-manager command found in `PATH`” behavior is brittle in environments where multiple package managers exist/are aliased (containers, mixed images, custom PATHs).
- Install failures are harder to debug because the installer does not clearly report distro detection inputs or the chosen package manager.
- We want safer, more predictable defaults using distro-family hints, while retaining explicit operator override.

## Goals
- Print a stable, operator-facing one-liner during Linux installs: detected distro (`ID`, `ID_LIKE`) + selected package manager + selection source.
- Add an explicit CLI override flag `--pkg-manager <apt-get|dnf|yum|pacman|zypper>` that deterministically wins over all autodetection.
- Preserve legacy override via environment variable `PKG_MANAGER` (but ensure `--pkg-manager` wins over env).
- Fail with actionable guidance when the operator requests an unavailable/unsupported package manager, or when no supported package manager can be selected.
- Keep detection fast and local (O(1) reads such as `/etc/os-release`; no network calls).

## Options Considered

### Option A — Flag override + PATH probing only (smallest delta)
Keep default selection as “first supported package manager found in `PATH`”, add `--pkg-manager …`, and print the selected manager (and optionally the `PATH` probe results) for debugging.

**Choose Option A when…**
- We want the smallest/lowest-risk change focused on explicit override + improved visibility.
- We are not ready to commit to maintaining a distro-family mapping table yet.

### Option B — `/etc/os-release` mapping + override (recommended)
Parse `/etc/os-release` (best-effort) and use `ID` / `ID_LIKE` to pick safer defaults via a small mapping table, while preserving deterministic overrides via `--pkg-manager …` and `PKG_MANAGER=…`. Fall back to `PATH` probing when mapping is unavailable.

**Choose Option B when…**
- We want predictable defaults that match operator expectations (“Ubuntu → apt-get”, “Arch → pacman”) and reduce support/debug time.
- We can maintain a small, explicit mapping table and treat it as best-effort (with a safe fallback path).

### Option C — External command-based distro detection (avoid)
Infer distro using tools like `lsb_release` or `hostnamectl`.

**Avoid Option C because…**
- It adds more dependencies/failure modes and risks circular prerequisites (needing a package manager to install detection tools).

## Non-Goals
- Changing Substrate runtime behavior, world isolation, or world-deps semantics.
- Introducing a general-purpose package manager abstraction across the Rust workspace (installer-script scoped only).
- Adding broad distro/package-manager support in this slice (e.g., Alpine `apk` is explicitly out of scope here).
- Making the installer silently succeed by skipping prerequisites; failures remain explicit and actionable.
- Persisting detected distro/package-manager into `$SUBSTRATE_HOME/install_state.json` (explicit follow-up; see out-of-scope).

## Slice Decomposition

### C0 — Distro detection + reporting
Parse `/etc/os-release` (best-effort) to extract `ID` and `ID_LIKE` for diagnostics; choose a default package manager using a small mapping, and print a stable one-liner that includes detected values and the selected package manager.

### C1 — Explicit package-manager override
Add `--pkg-manager …` and honor `PKG_MANAGER=…` (legacy). Define and enforce selection precedence: flag → env → os-release mapping → path probe fallback.

### C2 — Hermetic detection tests
Add a hermetic test under `tests/installers/` that exercises the precedence rules and mapping behavior using a controlled `PATH` and a fake os-release file (no host mutations, no containers required).

## User Contract (Authoritative)

### CLI
- Script: `scripts/substrate/install-substrate.sh`
- New flag:
  - `--pkg-manager <apt-get|dnf|yum|pacman|zypper>`
    - Meaning: force the package manager used for installing Linux host prerequisites.
    - Precedence: highest (wins over `PKG_MANAGER` and all autodetection).
    - Validation: if value is not one of the allowed set, exit with code `2`.
    - Availability: if the chosen manager is not found in `PATH`, exit with code `3` and print remediation guidance.
- Legacy env override (supported):
  - `PKG_MANAGER=<apt-get|dnf|yum|pacman|zypper>`
    - Precedence: second (wins over os-release mapping and path probe, but loses to `--pkg-manager`).
    - Validation: if value is not one of the allowed set, exit with code `2`.

- Default selection algorithm (Linux only):
  1) If `--pkg-manager` is provided, use it. `pkg_manager_source=flag`.
  2) Else if `PKG_MANAGER` is set, use it. `pkg_manager_source=env`.
  3) Else, if `/etc/os-release` is readable, derive distro family from `ID` / `ID_LIKE` and map to a preferred manager (see mapping table below). `pkg_manager_source=os_release`.
     - Fedora/RHEL family prefers `dnf`, but may fall back to `yum` if `dnf` is not available.
  4) Else, probe `PATH` for supported managers (fallback to current behavior). `pkg_manager_source=path_probe`.
     - Compatibility rule: when multiple supported managers are found, the installer must behave deterministically by applying a fixed precedence order and emitting a warning that lists the other detected managers and how to override.
  5) If no supported manager can be selected, exit with code `4` and print actionable guidance:
     - how to proceed with `--pkg-manager …` / `PKG_MANAGER=…`, and
     - which prerequisite commands must be installed manually.

- Mapping table (best-effort, initial locked set):
  - Debian/Ubuntu family → `apt-get`
    - Match if `ID` in `{debian, ubuntu, linuxmint, pop}` OR `ID_LIKE` contains `debian` or `ubuntu`.
  - Fedora/RHEL family → prefer `dnf` (fallback `yum` if `dnf` missing)
    - Match if `ID` in `{fedora, rhel, centos, rocky, almalinux, ol, amzn}` OR `ID_LIKE` contains `fedora` or `rhel`.
  - Arch family → `pacman`
    - Match if `ID` in `{arch, manjaro, endeavouros, arcolinux, artix, garuda}` OR `ID_LIKE` contains `arch`.
  - SUSE family → `zypper`
    - Match if `ID` contains `suse` or `opensuse` OR `ID_LIKE` contains `suse` or `opensuse`.

- Required installer output (Linux only):
  - Before installing prerequisites, the installer prints exactly one stable one-liner to stderr:
    - `Detected distro: <id> (like: <id_like>), using package manager: <pkg_manager> (source: <flag|env|os_release|path_probe>)`
  - If `/etc/os-release` is missing/unreadable, `<id>` / `<id_like>` must be rendered as `<unknown>`.

- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: install completed successfully (or no-op by contract, e.g., prerequisites already present and no world/shims requested)
  - `2`: invalid CLI usage or invalid override value (unknown `--pkg-manager` value, invalid `PKG_MANAGER` value)
  - `3`: required dependency unavailable (forced package manager missing from `PATH`; prerequisite executable cannot be installed due to missing package-manager binary)
  - `4`: not supported / missing prerequisites (no supported package manager can be selected; operator must manually install prerequisites or provide an override)

### Config
- No new config files.
- Existing env vars remain supported; this ADR adds no new persistent config surface.

### Platform guarantees
- Linux: Contract above applies. Detection reads `/etc/os-release` locally; no network calls for detection. Privileged operations remain explicit (via `sudo` when required by existing installer behavior).
- macOS/Windows: No behavior change in this ADR.

## Architecture Shape
- Components:
  - `scripts/substrate/install-substrate.sh`: add arg parsing for `--pkg-manager`, add safe `/etc/os-release` parsing, and refactor package-manager selection into an explicit precedence pipeline that reports source + decision.
  - `tests/installers/`: add a hermetic test harness for detection precedence + mapping (no container runtime requirement).
- End-to-end flow:
  - Inputs: `--pkg-manager` flag (optional), `PKG_MANAGER` env (optional), `/etc/os-release` contents (best-effort), available package manager binaries in `PATH`.
  - Derived state: `distro_id`, `distro_id_like`, `pkg_manager`, `pkg_manager_source`, `found_managers[]`.
  - Actions: print decision line; install prerequisite packages using the selected package manager; proceed with existing world/shim install workflow.
  - Outputs: stable decision line + existing installer logs; on failure, actionable remediation text.

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → ASSUMPTION: add a “detecting-badger” entry before execution begins.
- Related/coordination-only intake: `provisioning_otter` (no hard dependency declared in intake).

## Security / Safety Posture
- Fail-closed rules:
  - If an explicit override is provided (`--pkg-manager` or `PKG_MANAGER`) but the selected manager is unavailable or invalid, fail (do not silently fall back).
  - If no supported manager can be selected, fail with actionable guidance (do not proceed with partial/unknown prereq state).
- Protected paths/invariants:
  - Detection reads `/etc/os-release` only; no writes.
  - Privileged operations remain explicit under existing `sudo` rules; this ADR does not add new privilege escalation paths.
- Parsing posture:
  - `/etc/os-release` must be parsed without executing arbitrary shell code (i.e., do not `source` it); extract `ID` and `ID_LIKE` via safe line parsing.

## Validation Plan (Authoritative)

### Tests
- Hermetic detection test:
  - Add a new `tests/installers/` script that sets a controlled `PATH` containing stub `apt-get|dnf|yum|pacman|zypper` binaries and provides a fake os-release file.
  - Assert the precedence rules (flag overrides env overrides os-release overrides path probe) and that the printed one-liner contains expected `pkg_manager` and `source`.
- Existing container smoke (optional, not required for CI gating):
  - `tests/installers/pkg_manager_container_smoke.sh` remains a local sanity check for `/etc/os-release` presence and manager availability in representative images.

### Manual validation
- Manual playbook: `docs/project_management/packs/active/detecting-badger/manual_testing_playbook.md` (planned)
  - Must include: examples for default Ubuntu selection, default Arch selection, forced override (`--pkg-manager`), legacy env (`PKG_MANAGER`), and a failure mode with remediation.

### Smoke scripts
- No new platform smoke scripts are required for this installer-only slice beyond the hermetic test above.

## Rollout / Backwards Compatibility
- Policy: additive and best-effort.
- Compat guarantees:
  - Existing installs without overrides continue to function; if os-release mapping fails, selection falls back to path probe.
  - The only intentional behavior change is improved default selection when os-release mapping is available (more predictable “Ubuntu → apt-get”, “Arch → pacman”, etc.), plus explicit override support.

## Decision Summary
- Decision Register: required (multiple architectural choices with tradeoffs). Create:
  - `docs/project_management/packs/active/detecting-badger/decision_register.md`
- What belongs in the ADR (this document):
  - Operator-facing contract (flag/env precedence, mapping table, output line, exit-code behavior).
  - Slice boundaries and validation requirements.
- What belongs in the Decision Register (pack-local):
  - Parsing approach for `/etc/os-release` (strict parser vs shell-friendly parsing constraints).
  - Whether ambiguity in path probe should warn vs fail (and under what conditions).
  - Whether to add/standardize a test hook for supplying an alternate os-release path (and whether it is supported or test-only).
