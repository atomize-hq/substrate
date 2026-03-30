# Decision Register — add-non-apt-system-package-provisioning-support-fse

This register captures the accepted decision basis that `C-01` and `C-03` depend on. The contract file is authoritative for operator-facing behavior; this register records the fixed choices behind it.

These decisions publish the additive pacman schema and inventory-view basis that `C-03` carries through `THR-03`.

## DR-0001 — Explicit `install.method=pacman` plus `install.pacman`

**Decision owner(s):** Shell / world-deps maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support-fse/contract.md`, `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`

**Problem / context**

The pack needs a non-APT system-package shape that stays readable in inventory and does not introduce a translation layer.

**Decision**

- Represent pacman-backed system packages with an explicit `install.method=pacman`.
- Require an `install.pacman` list for those items.
- Preserve manager-specific package naming in authored inventory.
- Preserve authored `install.pacman` order in stored definitions and rendered inventory views.
- Keep inventory resolution and enabled-set semantics unchanged.

**Downstream constraints**

- The schema extension must remain additive on `version: 1`.
- Pacman support in this pack remains provisioning-only and non-runnable.
- Inventory views must keep `install.method=pacman` explicit and preserve authored `install.pacman` order.
- No distro-translation layer, remap layer, or abstract `system_packages` method is introduced.

## DR-0002 — In-world probe strategy for manager selection

**Decision owner(s):** Shell / world backend maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support-fse/contract.md`

**Problem / context**

Provisioning must pick the world package manager without consulting host state, and it needs one deterministic fail-closed rule for contradictory probe results.

**Decision**

- Derive the detected world manager in-world from `/etc/os-release` plus in-world package-manager presence.
- Restrict probe inputs to `/etc/os-release` fields `ID` and `ID_LIKE`, plus `command -v pacman`.
- Normalize both `ID` and `ID_LIKE` to lower-case; treat `ID_LIKE` as whitespace-separated tokens.
- Classify Debian-family when `ID == debian` or any `ID_LIKE` token equals `debian` or `ubuntu`.
- Classify Arch-family when `ID == arch` or any `ID_LIKE` token equals `arch` or `archlinux`.
- Treat both family flags true as `ambiguous_family_mapping` and fail closed.
- Treat neither family flag true as `unmapped_family` and fail closed.
- Record `pacman_present: true|false` from in-world `command -v pacman`.
- Treat Arch-family with `pacman_present == false` as `arch_family_pacman_missing` and fail closed.
- Preserve Debian-family selection as `apt` even when `pacman_present == true`; `pacman_present` alone is not an Arch signal.
- Accept only `apt` or `pacman` as supported manager outputs.
- Never route from host PATH, host package-manager presence, or host installer detection.

**Downstream constraints**

- Manager selection must remain stable across host platforms.
- Any ambiguous, unmapped, unreadable, or contradictory result must exit `4` before mutation.

## DR-0003 — Pacman execution shape and ordering

**Decision owner(s):** Shell / world backend maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0033-routing-weasel.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support-fse/contract.md`

**Problem / context**

Pacman provisioning needs one deterministic command shape and one idempotent ordering rule.

**Decision**

- Normalize the pacman requirement set before execution.
- Execute pacman provisioning with the exact command shape:

  ```text
  pacman -Sy --noconfirm --needed <packages...>
  ```

- Pass package arguments in normalized order.
- Use `--needed` for no-op suppression rather than Substrate-managed retries or lock recovery.

**Downstream constraints**

- Pacman provisioning must remain deterministic in dry-run, verbose, and live execution paths.
- Pacman provisioning must not invoke AUR helpers, retries, or lock-file intervention.

## DR-0004 — Mismatch policy and partial-provisioning rule

**Decision owner(s):** Shell / world backend maintainers  
**Date (UTC):** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`, `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support-fse/contract.md`

**Problem / context**

The pack needs one fail-closed rule for enabled sets that require multiple managers or do not match the detected manager.

**Decision**

- If both normalized requirement sets are non-empty, exit `4` before any package-manager command runs.
- If the detected manager does not match the enabled set’s required manager, exit `4`.
- Never partially provision one manager and defer the other.
- Never fall back from `apt` to `pacman` or from `pacman` to `apt`.

**Downstream constraints**

- Error text must identify the mismatch and point back to `substrate world enable --provision-deps`.
- This pack does not invent new remediation surfaces or new operator controls to resolve the mismatch.
