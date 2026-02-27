**PRE‑PLANNING ONLY — This document is non-authoritative and will be deleted/retired during full planning.**

# add-non-apt-system-package-provisioning-support — minimal spec draft (alignment backbone)

## Scope + authority

This draft defines only cross-cutting defaults/precedence/invariants that every spec in this pack MUST align on.

This draft does NOT define:
- slice-specific algorithms (probe parsing, requirement derivation ordering, pacman command flags)
- detailed schemas (beyond “surface exists and is owned elsewhere”)
- implementation tasks, touch lists, or validation command lists

Authoritative sources for intent and planning constraints:
- ADR: `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`
- Spec manifest (surface ownership): `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/spec_manifest.md`
- Impact map (touch/overlap decisions): `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/impact_map.md`

## Defaults + precedence

### Inputs (by class)
- CLI provisioning entrypoint:
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
- Runtime world-deps surfaces (no OS mutation):
  - `substrate world deps current sync`
  - `substrate world deps current install`
- Config / inventory surfaces (owned elsewhere):
  - world-deps inventory schema + “effective enabled set” definition are owned by:
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
  - This feature introduces no new config keys and does not change config precedence rules.
- Env vars:
  - No new env vars are introduced by ADR-0033.

### Precedence rules (highest → lowest)
1) `--dry-run` (provisioning): MUST disable all OS mutation and MUST only print intended actions.
2) Manager selection: MUST be derived from an **in-world** probe; MUST NOT be derived from host PATH or host OS detection.
3) Requirement derivation: MUST use the effective enabled world-deps set (as defined by the existing world-deps contract); CLI flags do not override `install.method` or package lists.
4) `--verbose`: MUST NOT change behavior or exit codes; it only changes emitted detail.

### Source-of-truth contract ownership (cross-pack)
- Single authoritative contract wording for the shared CLI/exit-code/remediation surfaces (`--provision-deps` + runtime fail-early for system-package methods) is owned by:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- Method-specific execution details remain owned by method packs/specs (e.g., APT provisioning specifics in `docs/project_management/packs/draft/world-deps-apt-provisioning/`).

## Failure posture + invariants

### Fail-closed posture
- Runtime `substrate world deps current sync|install` MUST NOT execute OS package managers (APT or pacman).
- If the effective enabled set contains system-package methods (`install.method=apt` or `install.method=pacman`), runtime MUST fail early with:
  - a non-zero exit code (taxonomy mapping below), and
  - remediation that includes the exact command: `substrate world enable --provision-deps`.

### No host OS mutation
- Linux host-native backend: `substrate world enable --provision-deps` is unsupported by default (no host OS mutation).
- When provisioning is unsupported, remediation MUST explicitly state that Substrate will not mutate the host OS and MUST provide manual guidance.

### Manager mismatch posture
- If provisioning is supported but the enabled system-package methods do not match the detected world OS package manager, provisioning MUST fail with actionable remediation (no partial/implicit fallback is defined in this draft).

### Guard rails (high level)
- Provisioning execution MUST be opt-in and MUST NOT weaken hardened runtime execution.
- Guard rails MUST prevent host PATH influence on manager selection and MUST prevent accidental host environment leakage into provisioning execution.

## Exit-code posture

- Exit code taxonomy (canonical): `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- New exit codes: none.
- Feature mapping (from ADR-0033):
  - `0`: success
  - `2`: invalid inventory/config schema (includes unsupported `install.method` or invalid `install.pacman` shape)
  - `3`: world backend unavailable / cannot connect to world-agent
  - `4`: unmet prerequisites or unsupported operation (includes provisioning unsupported; provisioning required; detected manager mismatch)
  - `5`: safety/policy violation (reserved; runtime fail-early prevents reaching OS-manager execution paths)

## Cross-cutting seams / constraints

Multiple specs MUST align on the following shared seams:

### Inventory schema + “effective enabled set”
- `install.method` includes `pacman` alongside existing install methods (exact schema constraints are owned by the world-deps contract pack).
- `install.pacman[]` exists and is required iff `install.method=pacman` (version pinning is out of scope in v1 per ADR-0033).
- Provisioning requirement derivation and runtime fail-early checks both operate over the same effective enabled view.

### In-world probe inputs (no host dependence)
- Probe reads `/etc/os-release` (`ID`, `ID_LIKE`) and performs manager presence checks inside the world.
- Parsing/canonicalization and precedence rules are owned by slice specs/DRs; this draft only fixes the “in-world, not host-derived” constraint.

### Paths / filesystem invariants referenced by the contract
- `/etc/os-release` is a probe input path.
- Hardened runtime writable surfaces remain constrained (examples referenced by ADR-0033: `/var/lib/substrate/world-deps`, `/tmp`).

### Provisioning execution protocol seam
- Provisioning uses an explicit world-agent execution contract (request profile + timeouts + error model) owned by:
  - `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-provisioning-protocol-spec.md`
- Runtime flows MUST NOT reuse provisioning execution profiles.

### Platform matrix seam
- Contract docs MUST be explicit and deterministic for:
  - Linux host-native (unsupported provisioning by default)
  - macOS Lima guest backends (provisioning supported only when the active guest world permits it)
  - Windows WSL backends (ADR-0033 marks this as ASSUMPTION; full planning must lock it as contract or mark unsupported)

## Follow-ups for full planning

Each follow-up is required to remove ambiguity from slice specs and the final `contract.md`.

1) DR-0001 — Inventory schema approach
   - Select: explicit `install.method=pacman` vs abstract system-packages mapping.
2) DR-0002 — World OS probe strategy
   - Define `/etc/os-release` parsing/canonicalization, manager presence check commands, precedence rules, and Arch-family classification mapping.
3) DR-0003 — Pacman invocation + idempotency
   - Define exact non-interactive command shape, idempotency posture, and deterministic failure→exit-code mapping.
4) DR-0004 — Mismatch policy
   - Define deterministic behavior for mixed enabled sets (`apt` + `pacman`) and for manager mismatch (fail-closed vs partial provision).
5) DR-0005 — Cross-pack contract boundary
   - Record the selected “single authoritative contract wording” decision and list the exact downstream docs that must defer to it (notably `world-deps-apt-provisioning` surfaces).
6) Runtime fail-early scope
   - Decide whether runtime checks evaluate system-package presence over the effective enabled set or only explicitly requested items.
7) ADR link drift
   - ADR-0033 paths reference a different feature directory; update links after this pack is accepted (do not update from this pre-planning draft).
8) ADR-0003 overlap
   - Update `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md` to include `--provision-deps` and confirm invariant compatibility (per impact map).

