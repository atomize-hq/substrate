# ADR-0030 — Provisioning-Time APT for World-Deps (Hardened-World Compatible)

## Status
- Status: Draft
- Date (UTC): 2026-02-21
- Owner(s): ASSUMPTION: Shell maintainers; World backend maintainers

## Scope
- Feature directory: `docs/project_management/packs/active/world-deps-apt-provisioning/` (ASSUMPTION: new pack)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/standards/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`
  - `docs/project_management/standards/TASK_TRIADS_AND_FEATURE_SETUP.md`
  - `docs/project_management/standards/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md` (automation/worktree execution)

## Related Docs
- Internals (current behavior notes): `docs/internals/world/deps.md`
- World-deps contract / install classes: `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`
- System packages posture (historical): `docs/project_management/_archived/world_deps_selection_layer/`
- Linux guest-rootfs roadmap context: `docs/project_management/adrs/draft/ADR-0009-linux-guest-rootfs-backend-and-linux-system-packages-provisioning.md`
- Plan/specs/tasks/etc (to be created in the feature directory above):
  - Plan: `docs/project_management/packs/active/world-deps-apt-provisioning/plan.md`
  - Tasks: `docs/project_management/packs/active/world-deps-apt-provisioning/tasks.json`
  - Spec manifest: `docs/project_management/packs/active/world-deps-apt-provisioning/spec_manifest.md`
  - Decision Register: `docs/project_management/packs/active/world-deps-apt-provisioning/decision_register.md`
  - Impact Map: `docs/project_management/packs/active/world-deps-apt-provisioning/impact_map.md`
  - Manual Playbook: `docs/project_management/packs/active/world-deps-apt-provisioning/manual_testing_playbook.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: <run `make adr-fix ADR=docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md` after drafting>

### Changes (operator-facing)
- APT-backed world-deps become provisioning-time; runtime sync/install is user-space-only
  - Existing: `substrate world deps current sync|install` may attempt `apt-get install` for `install.method=apt` items, which can fail under hardened worlds (rootfs effectively read-only) and can violate the “no host OS mutation” posture on Linux host-native backends.
  - New: Operators run `substrate world enable --provision-deps` to provision APT/system packages required by the effective enabled world-deps set (guest backends only). Runtime `substrate world deps current sync|install` never runs APT; it fails early for APT items with actionable remediation (or manual guidance when provisioning is unsupported).
  - Why: Keep the hardened runtime sandbox fail-closed while providing an explicit, auditable workflow for OS mutation where it is safe (guest worlds).
  - Links:
    - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md#L1`
    - `docs/internals/world/deps.md#L1`
    - `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md#L1`

## Problem / Context
- Hardened world execution paths can render `/` effectively read-only (e.g. systemd `ProtectSystem=strict`), so `apt/dpkg` cannot mutate system paths/state and `substrate world deps current sync` can fail with “Read-only file system”.
- Even when technically possible, running APT during runtime “sync/install” is a contract mismatch: OS/system package mutation should be provisioning-time, not a side-effect of runtime dependency sync.
- On Linux host-native backends, running APT would mutate the workstation OS, which is typically disallowed by the threat model and operator expectations.

## Goals
- Provide an explicit provisioning-time workflow to install APT/system packages required by the effective enabled world-deps set on guest-world backends.
- Ensure runtime `substrate world deps current sync|install` does not attempt OS mutation for APT items under any backend/hardening posture.
- Make failure modes actionable and consistent across platforms/backends with stable exit-code meaning.

## Non-Goals
- Redesigning world-deps inventory schema or enabled-resolution logic.
- Relaxing hardened runtime write restrictions (no “make / writable”, no widening `ReadWritePaths`).
- Supporting additional package managers (brew, yum/dnf, apk, pacman) in this ADR.
- Adding new guest prerequisite checks (e.g., `ca-certificates`) beyond the APT provisioning workflow (defer to follow-up ADR).
- Changing collision/entrypoint enforcement behavior for world-deps wrappers.

## Slice Decomposition

### C0 — Provisioning surface for APT requirements
Add an explicit provisioning-time surface that installs APT/system packages required by the effective enabled world-deps set, supported only on guest-world backends where OS mutation is permitted/safe.

### C1 — Runtime fail-early + remediation for APT items
Update runtime `substrate world deps current sync|install` so it never invokes APT; when an enabled world-deps item requires `install.method=apt`, fail early with actionable remediation pointing to the provisioning step (or manual guidance when provisioning is unsupported).

## Options (Viable) + Recommendation

### Option A — `substrate world enable --provision-deps` (recommended)
Add an opt-in provisioning flag to `substrate world enable` that provisions APT/system packages required by the effective enabled world-deps set. Runtime `world deps current sync|install` never invokes APT and fails early with remediation pointing to this provisioning step.

### Option B — `substrate world deps provision`
Add a `substrate world deps provision` command that provisions APT/system packages required by the effective enabled world-deps set. Runtime `world deps current sync|install` never invokes APT and fails early with remediation pointing to this provisioning command.

### Recommendation (selection guidance)
- Choose Option A when you want a single, operator-recognizable “provision the world backend” surface where all guest-OS mutation lives, and you are comfortable with `world enable` being coupled to world-deps requirement derivation.
- Choose Option B when you want OS mutation to remain within the `world deps` namespace (to avoid growing `world enable` responsibilities), and you can accept a second top-level workflow verb under `world deps`.

## User Contract (Authoritative)

### CLI
- Provisioning (new):
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
    - Derives required APT packages from the effective enabled world-deps set (no explicit item list in v1).
    - When supported, performs OS mutation to install/ensure required packages are present in the guest world.
    - `--dry-run` prints the derived APT package list and intended actions; performs no mutation.
- Runtime world-deps (changed behavior for APT items):
  - `substrate world deps current sync`
  - `substrate world deps current install`
    - MUST NOT invoke APT/dpkg.
    - If the effective enabled set contains `install.method=apt` items:
      - Exit non-zero with a friendly, actionable error explaining that APT-backed items are provisioning-time.
      - Remediation MUST include the exact command: `substrate world enable --provision-deps`.
      - On backends where provisioning is unsupported, remediation MUST provide manual guidance and explicitly state that the host OS will not be mutated by Substrate.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (authoritative unless explicitly overridden here)
  - `0`: success
  - `3`: world backend unavailable / cannot connect to world-agent
  - `4`: unmet prerequisites or unsupported operation (includes: APT provisioning not supported on this backend; runtime sync/install encountered APT items and requires provisioning)
  - `5`: hardening conflict / fail-closed safety violation (reserved; runtime flow should avoid triggering this for APT by failing early)

### Config
- Effective enabled world-deps set (existing):
  - Provisioning derives required APT packages from the same effective enabled view used by `substrate world deps current ...`.
  - This ADR introduces no new config keys and does not change config precedence rules.
  - `install.method=apt` remains the discriminator for “system packages” vs “user-space/script” installs (per existing contract).

### Platform guarantees
- Linux (host-native backend):
  - `substrate world enable --provision-deps` is unsupported by default (no host OS mutation).
  - Runtime `world deps current sync|install` fails early with explicit manual guidance.
  - Future: if/when a Linux guest-rootfs backend exists, provisioning MAY be supported there (see related ADR-0009).
- macOS (Lima guest backend):
  - `substrate world enable --provision-deps` is supported and runs with an execution profile that permits guest OS mutation (provisioning-time), distinct from hardened runtime execution.
- Windows (WSL backend):
  - ASSUMPTION: `substrate world enable` is (or will be) supported on Windows+WSL; when supported, `--provision-deps` provisions inside the WSL guest and never mutates the Windows host OS.

## Architecture Shape
- Components:
  - `crates/shell/src/builtins/world_enable/…`: extend `world enable` to derive APT requirements from the effective enabled world-deps set and run a provisioning-time install step (guest worlds only).
  - `crates/shell/src/builtins/world_deps/surfaces.rs`: change runtime `current sync|install` path to preflight-detect APT items and fail early with remediation instead of building/running APT commands.
  - `crates/world-agent/src/service.rs`: ensure provisioning execution is possible without weakening hardened runtime execution (e.g., distinct request profile or explicit guard rails).
  - Docs: add operator-facing explanation that APT-backed world-deps are provisioning-time under hardening (`docs/reference/world/deps/…`).
- End-to-end flow:
  - Inputs:
    - world-deps inventory (built-ins + global + workspace chain)
    - effective enabled selection (global + workspace patches)
  - Derived state:
    - APT requirement set for enabled items with `install.method=apt`
    - backend capability: provisioning supported vs unsupported
  - Actions:
    - provisioning path: install/ensure APT packages in guest OS (or `--dry-run` print only)
    - runtime path: abort early with remediation (no APT)
  - Outputs:
    - clear operator stdout/stderr guidance
    - stable exit codes per taxonomy and mapping above

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` → ASSUMPTION: add a new sprint entry for `world-deps-apt-provisioning`
- Prerequisite integration task IDs:
  - None required by this ADR (follow-ups may depend on Windows `world enable` support and/or Linux guest-rootfs availability).

## Security / Safety Posture
- Fail-closed rules:
  - Runtime `world deps current sync|install` must never attempt APT/dpkg execution under hardened runtime execution.
  - Linux host-native must not mutate the host OS via provisioning.
- Protected paths/invariants:
  - Provisioning OS mutation is operator-invoked and explicitly surfaced via `--provision-deps`.
  - Runtime world-deps continues to be constrained to Substrate-managed writable surfaces (e.g., `/var/lib/substrate/world-deps`, `/tmp`).

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Add a contract test ensuring runtime APT items short-circuit with exit `4` and remediation text (shell crate).
- Integration tests:
  - Guest-world: verify provisioning path issues APT install actions when supported (can be mocked/recorded, depending on existing test harness patterns).
  - Linux host-native: verify provisioning is rejected with exit `4` and clear “no host mutation” messaging.
  - Existing APT install tests must be updated/repurposed to reflect provisioning-time behavior.

### Manual validation
- Manual playbook: `docs/project_management/packs/active/world-deps-apt-provisioning/manual_testing_playbook.md`
  - Must cover: guest provisioning success, runtime remediation behavior, Linux host-native unsupported behavior.

### Smoke scripts
- Linux: `docs/project_management/packs/active/world-deps-apt-provisioning/smoke/linux-smoke.sh`
- macOS: `docs/project_management/packs/active/world-deps-apt-provisioning/smoke/macos-smoke.sh`
- Windows: `docs/project_management/packs/active/world-deps-apt-provisioning/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none
- Behavior change note: runtime `substrate world deps current sync|install` no longer performs APT installs; operators must provision APT requirements explicitly via `substrate world enable --provision-deps` (or follow manual guidance when unsupported).

## Decision Summary
- Options in the ADR (body-of-work shape):
  - Option A (recommended): provisioning is a flag on `substrate world enable` (`--provision-deps`).
  - Option B (viable alternative): introduce `substrate world deps provision` as a separate verb under `world deps`.
  - This ADR documents the user-facing contract for Option A and the associated behavior delta.
- Decision Register vs ADR:
  - ADR should hold: the operator contract, exit-code mapping, platform posture (guest vs host-native), and the “no runtime APT mutation” invariant.
  - Decision Register should hold: fine-grained A/B decisions like “how provisioning runs relative to hardening profiles”, “whether to persist provisioned state vs probe-only”, “Windows support gating”, and “how to format derived APT package lists for `--dry-run`”.
- Decision Register entries:
  - `docs/project_management/packs/active/world-deps-apt-provisioning/decision_register.md`:
    - DR-0001 (Option A vs Option B selection)
    - DR-0002 (provisioned-state tracking: probe-only vs state file)
    - DR-0003 (provisioning execution profile isolation model)
