# world-deps-apt-provisioning — spec manifest

This file enumerates every contract/protocol/schema surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`

## Required spec documents (authoritative)

Spec templates:
- `docs/project_management/system/templates/spec/`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`
  - Owns (authoritative): spec selection + contract surface ownership map (this file).
  - Links to: all docs listed below.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/impact_map.md`
  - Owns (authoritative): touch set + cascading implications + cross-queue conflicts.
  - Links to: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/plan.md`
  - Owns (authoritative): execution runbook + sequencing overview for C0/C1 slices.
  - Links to: `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/tasks.json`
  - Owns (authoritative): triad task graph + acceptance criteria for C0/C1 slices.
  - Links to: slice specs + `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - Owns (authoritative): user-facing contract introduced/changed by ADR-0030:
    - CLI: `substrate world enable --provision-deps [--dry-run] [--verbose]`.
    - CLI: APT-specific behavior of `substrate world deps current sync|install` (no APT/dpkg; fail-early + remediation).
    - Exit code mapping for the above flows (and any explicit taxonomy overrides).
    - Operator-visible error/remediation message invariants (including required exact command strings).
    - Platform support matrix (Linux host-native vs macOS Lima vs Windows WSL) for `--provision-deps`.
    - Host OS mutation prohibition and required messaging when provisioning is unsupported.
    - Protected paths/invariants relevant to this feature.
  - Links to (non-authoritative):
    - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (world-deps inventory schema + enable/merge semantics; non-APT behavior)
    - `docs/WORLD.md` (world / world-agent overview; background reference only)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
  - Owns (authoritative): open decisions explicitly called out by ADR-0030, at minimum:
    - DR-0002: provisioned-state tracking (probe-only vs state file).
    - DR-0003: provisioning execution profile isolation model.
  - Links to: `docs/project_management/packs/draft/world-deps-apt-provisioning/world-deps-apt-provisioning-protocol-spec.md`.

- `docs/project_management/packs/draft/world-deps-apt-provisioning/world-deps-apt-provisioning-protocol-spec.md`
  - Owns (authoritative): host↔world-agent protocol contract required by ADR-0030:
    - How provisioning-time APT execution is requested (including the exact `ExecuteRequest.profile` value, if used).
    - Guard rails that ensure provisioning execution does not weaken hardened runtime execution.
    - Error model + timeouts/retries relevant to provisioning execution.
  - Links to (non-authoritative):
    - `docs/CONFIGURATION.md` (reference for `SUBSTRATE_WORLD_REQUEST_PROFILE`, if reused)
    - `docs/WORLD.md` (baseline endpoint list)

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C0/C0-spec.md`
  - Owns (authoritative): slice C0 behavior (Provisioning surface for APT requirements):
    - Derivation of the required APT package set from the effective enabled world-deps set.
    - Provisioning execution steps (including idempotency rules and `--dry-run` output requirements).
    - Backend capability detection and the mapping to exit codes/messages.
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/world-deps-apt-provisioning-protocol-spec.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

- `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C1/C1-spec.md`
  - Owns (authoritative): slice C1 behavior (Runtime fail-early + remediation for APT items):
    - The preflight/detection rules for `install.method=apt` items in the runtime path.
    - The required fail-early behavior and guarantees that APT/dpkg is never invoked.
    - The required remediation content and exit code mapping for runtime `sync|install` when APT items are present.
  - Links to (non-authoritative):
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

## Coverage matrix (surface → authoritative doc)

Every surface that ADR-0030 touches MUST appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| CLI: `substrate world enable --provision-deps [--dry-run] [--verbose]` | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | flags, defaults, supported backends, exit codes, examples, stdout/stderr invariants |
| Behavior: derive required APT packages from the effective enabled world-deps set | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C0/C0-spec.md` | selection rules, bundle expansion rules, de-duplication + ordering rules, version handling rules |
| Behavior: provisioning execution (APT install/ensure) | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C0/C0-spec.md` | idempotency/no-op semantics, non-interactive behavior, failure posture, exit-code mapping points |
| Protocol: provisioning-time execution request to world-agent | `docs/project_management/packs/draft/world-deps-apt-provisioning/world-deps-apt-provisioning-protocol-spec.md` | request shape deltas (if any), `profile` value semantics (if any), timeouts/budgets, guard rails vs hardened runtime |
| CLI: runtime `substrate world deps current sync|install` MUST NOT invoke APT/dpkg | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | user-visible behavior, required remediation text (including exact command), exit code mapping, “no host mutation” messaging when unsupported |
| Behavior: runtime APT preflight detection + “fail early” invariants | `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C1/C1-spec.md` | detection algorithm, ordering (“fail early” definition), guarantees that no APT/dpkg exec occurs, interactions with `--dry-run`/`--verbose` |
| Exit codes (feature-specific mapping for the affected flows) | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | mapping to `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`, plus any explicit overrides; required error text invariants per code |
| Platform guarantees: Linux host-native vs macOS Lima vs Windows WSL | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | supported/unsupported matrix for `--provision-deps`, host OS mutation prohibition, and deterministic remediation guidance per platform/backend |
| World backend unavailable / cannot connect to world-agent | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | what “unavailable” means, exit code `3`, and required remediation guidance |
| World-deps inventory schema for APT items (`install.method=apt`, `install.apt[].name`, `install.apt[].version`) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | schema, constraints, defaults, and version semantics |
| Effective enabled world-deps resolution (inventory sources + enabled patches) | `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` | source paths, precedence/merge rules, and “effective enabled set” definition used by provisioning |
| Filesystem/path invariants referenced by ADR-0030 (`/var/lib/substrate/world-deps`, `/tmp`, “no host OS mutation”) | `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` | invariants, failure posture, and platform-specific notes |
| Decision: provisioned-state tracking model | `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md` | exactly two options (A/B), one selection, and a deterministic outcome that is implementable and testable |
| Decision: provisioning execution profile isolation model | `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md` | exactly two options (A/B), one selection, and a deterministic outcome that constrains the protocol + world-agent behavior |

## Determinism checklist (must be satisfied before quality gate)

### `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
- MUST define all affected commands/flags/defaults with at least one worked example per platform/backend class.
- MUST define exit codes for every failure mode described in ADR-0030, referencing `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`.
- MUST define required remediation message invariants, including the exact command string `substrate world enable --provision-deps`.
- MUST define a deterministic support matrix for provisioning (Linux host-native, macOS Lima, Windows WSL) and the exact behavior when unsupported.

### `docs/project_management/packs/draft/world-deps-apt-provisioning/world-deps-apt-provisioning-protocol-spec.md`
- MUST define the provisioning execution request contract (including whether `ExecuteRequest.profile` is used and its exact value, if used).
- MUST define guard rails that keep hardened runtime execution fail-closed and prevent provisioning behavior from leaking into runtime execution paths.
- MUST define timeouts/budgets and the error model exposed to the CLI surfaces (including mapping points to exit code `3` vs `4`).

### `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C0/C0-spec.md`
- MUST define the APT requirement derivation algorithm from the effective enabled set (including bundle expansion, de-duplication, ordering, and version handling).
- MUST define provisioning execution semantics (idempotency, `--dry-run` output requirements, and what constitutes “no mutation”).
- MUST define failure posture and deterministic error mapping for unsupported backends and missing prerequisites.

### `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/C1/C1-spec.md`
- MUST define “fail early” precisely (what it means operationally and what side effects are prohibited).
- MUST define how runtime `sync` and runtime `install <ITEM...>` behave when APT items exist (including interactions with `--dry-run` and `--verbose`).
- MUST define exact remediation invariants and the required “no host mutation” messaging when provisioning is unsupported.

### `docs/project_management/packs/draft/world-deps-apt-provisioning/decision_register.md`
- MUST contain DR-0002 and DR-0003 as A/B decisions with one explicit selection each.
- MUST specify the contract consequences of each selected decision (which spec(s) it constrains) with no TBDs.

## Follow-ups

- ADR-0030 links `docs/project_management/packs/active/world-deps-packages-bundles-contract/contract.md`, but the contract currently lives at `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`. The ADR’s Related Docs list MUST be corrected to avoid broken references.
- ADR-0030 changes the APT behavior of `substrate world deps current sync|install`, which is currently specified (including runtime APT execution) in `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`. The ownership boundary for “APT provisioning-time vs runtime” MUST be made explicit and the older contract MUST be updated to avoid contradictory wording.
- ADR-0030 does not specify whether runtime `deps current sync|install` should perform any non-APT work when APT items are present (e.g., apply script installs but still exit non-zero). The contract MUST choose exactly one deterministic behavior and tests MUST enforce it.
- ADR-0030 does not specify whether runtime `deps current install <ITEM...>` evaluates APT presence over (a) the effective enabled set or (b) only the explicitly requested items. The contract MUST choose exactly one deterministic behavior and tests MUST enforce it.
- ADR-0030 does not specify the exact world-agent execution request profile name/value (if a profile mechanism is used). The protocol spec MUST define the exact value and semantics, or explicitly state that no profile field is used.
- Windows behavior is marked ASSUMPTION in ADR-0030. The contract MUST define the exact v1 behavior on Windows when `substrate world enable` and/or provisioning is unsupported.
