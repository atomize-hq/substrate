# Decision Register — add-non-apt-system-package-provisioning-support

Template standard:
- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md`

This register records the A/B decisions that remain authoritative for the pack-owned manager-aware contract. The mixed-manager enabled-set rule is fixed directly in `contract.md`.

## DR-0001 — Inventory schema posture for non-APT system packages

**Decision owner(s):** Shell / world-deps maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`

**Problem / context**

The pack must extend world-deps to support Arch-family system-package provisioning without introducing a distro-mapping layer that invents package-name translation rules.

**Option A — Explicit `install.method=pacman` plus `install.pacman`**

- Extend the install-method enum to include `pacman`.
- Require `install.pacman` when `install.method=pacman`.
- Preserve manager-specific package naming in authored inventory.
- Keep enabled-set resolution unchanged.

**Option B — Abstract `install.method=system_packages` with per-distro mapping**

- Introduce a manager-agnostic install method.
- Add a Substrate-owned translation layer from abstract requirement names to distro-specific package-manager names.
- Route provisioning through that translation layer before manager selection.

**Recommendation**

- **Selected:** Option A — Explicit `install.method=pacman` plus `install.pacman`
- **Rationale (crisp):** It extends the existing contract with the smallest behavior delta and avoids package-name translation policy.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`

## DR-0002 — World-manager probe precedence and contradiction handling

**Decision owner(s):** Shell / world backend maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`

**Problem / context**

Provisioning must detect the in-world system-package manager without consulting host state, and it must define one exact rule for contradictory probe results.

**Option A — `/etc/os-release` is authoritative; manager presence confirms support**

- Read in-world `/etc/os-release`.
- Normalize `ID` plus `ID_LIKE` tokens.
- Map Debian-family tokens to `apt` and Arch-family tokens to `pacman`.
- Confirm that the mapped manager executable exists in-world.
- If `/etc/os-release` is unreadable, does not map to Debian-family or Arch-family, or maps to one family while only the other manager executable is present, fail with exit `4`.
- Do not fall back to a manager executable that contradicts `/etc/os-release`.

**Option B — Manager executable presence is authoritative; `/etc/os-release` is advisory**

- Probe for supported package-manager executables first.
- Use `/etc/os-release` only for messaging.
- If multiple supported managers are present, pick one by precedence.

**Recommendation**

- **Selected:** Option A — `/etc/os-release` is authoritative; manager presence confirms support
- **Rationale (crisp):** Routing by in-world OS identity keeps manager selection deterministic and prevents silent fallback to the wrong package manager.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP0/NASP0-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/platform-parity-spec.md`

## DR-0003 — Pacman execution shape, no-op detection, and failure handling

**Decision owner(s):** Shell / world backend maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`

**Problem / context**

Pacman provisioning needs one exact command shape, one idempotency rule, one dry-run rendering rule, and one failure posture for lock or package-manager errors.

**Option A — Single `pacman -Sy --noconfirm --needed` invocation with probe-only no-op detection**

- Normalize and sort the pacman requirement set before any probe or execution.
- Read-only no-op detection uses in-world pacman queries for the normalized requirement set. If every required package is already present, provisioning is a no-op.
- When provisioning is required, execute exactly:

  ```text
  pacman -Sy --noconfirm --needed <packages...>
  ```

- Use the normalized package order in the command arguments.
- `--dry-run` prints the normalized requirement set and the exact intended command shape, but performs no mutation.
- If pacman returns non-zero, Substrate exits `4`, surfaces pacman stderr, does not retry, does not delete lock files, and does not continue with any further system-package action.

**Option B — Multi-step sync/install flow with Substrate-managed retries or lock recovery**

- Run separate pacman sync and install steps.
- Add retry loops or lock-file intervention when pacman reports contention.
- Track no-op state through Substrate-managed state rather than read-only pacman queries.

**Recommendation**

- **Selected:** Option A — Single `pacman -Sy --noconfirm --needed` invocation with probe-only no-op detection
- **Rationale (crisp):** It matches the existing pacman installer precedent in `scripts/substrate/install-substrate.sh`, stays idempotent through `--needed`, and preserves fail-closed behavior.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP2/NASP2-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/manual_testing_playbook.md`

## DR-0004 — Pacman runnable-wrapper and present-semantics scope

**Decision owner(s):** Shell / world-deps maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`, `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`, `docs/project_management/packs/implemented/world-deps-host-visible-hardening/decision_register.md`

**Problem / context**

The existing world-deps contract and host-visible hardening work already define runnable-wrapper and present semantics for script-backed packages and for runnable APT-backed packages. The pack must decide whether v1 pacman support extends that runnable surface.

**Option A — Extend runnable-wrapper and present semantics to pacman in v1**

- Allow `install.method=pacman` packages to be `runnable: true`.
- Extend wrapper generation and present detection to pacman-backed entrypoints.
- Update host-visible hardening and wrapper failure semantics for pacman-backed runnable packages.

**Option B — Constrain v1 pacman packages to non-runnable prerequisites only**

- Require `install.method=pacman` packages to be `runnable: false`.
- Do not add pacman-specific wrapper generation in this pack.
- Do not change present semantics beyond read-only package-presence checks used for provisioning and runtime fail-early.

**Recommendation**

- **Selected:** Option B — Constrain v1 pacman packages to non-runnable prerequisites only
- **Rationale (crisp):** It avoids widening the runnable-entrypoint contract and keeps this pack scoped to provisioning-time system packages.

**Surfaces impacted (must implement this selection)**

- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/contract.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/world-deps-pacman-schema-spec.md`
- `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/slices/NASP1/NASP1-spec.md`
