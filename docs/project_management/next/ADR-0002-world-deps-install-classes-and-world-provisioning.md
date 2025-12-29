# ADR-0002: World-Deps Install Classes + Provisioning-Time System Packages

Status: Accepted

Last updated: 2025-12-24

Owners: Shell / World / Installer maintainers

Planning Pack directory:
- `docs/project_management/next/world_deps_selection_layer/`

Intended execution branch:
- `feat/world-sync`

Exit code taxonomy:
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (this ADR’s exit-code meanings must remain consistent with this taxonomy)

Authoritative implementation docs:
- Decision register: `docs/project_management/next/world_deps_selection_layer/decision_register.md`
- Integration map: `docs/project_management/next/world_deps_selection_layer/integration_map.md`
- Manual testing: `docs/project_management/next/world_deps_selection_layer/manual_testing_playbook.md`
- Specs:
  - `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
  - `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
  - `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
- Sequencing: `docs/project_management/next/sequencing.json`

Related tracks:
- YAML settings migration (Y0): `docs/project_management/next/yaml-settings-migration/Y0-spec.md`
- Agent hub isolation hardening (I0–I5): `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
- World sync (C0–C9): `docs/project_management/next/world-sync/plan.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 05ff5a8dca5d85f9f583a0bba032853dbd0bd6f00a854635e6d6991d3e0dfdba

ADR_BODY_SHA256: <run `python3 scripts/planning/check_adr_exec_summary.py --adr <this-file> --fix` after editing>

- Existing: `substrate world deps *` behavior is not fully selection-driven; some operations risk feeling “implicit” or “surprising” across platforms.
- New: `substrate world deps status|sync|install|provision` is selection-driven and no-ops (exit `0`, no world calls) when no selection file exists.
- Why: Prevents accidental tooling mutation/noise and makes behavior consistent, auditable, and safe for an agent-hub threat model.
- Links:
  - `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`

- Existing: “Install” can be interpreted as “mutate the OS”, which is unsafe/irreproducible during runtime `sync/install`.
- New: Install classes are explicit; OS packages route only through `substrate world deps provision` on supported guest platforms; runtime `sync/install` never mutate OS packages.
- Why: Keeps runtime behavior reproducible and avoids hidden privileged mutations.
- Links:
  - `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
  - `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

## 0) Executive summary

`substrate world deps` must be selection-driven, cross-platform consistent, and safe under an agent-hub threat model.
OS-level package mutation must never occur during runtime `substrate world deps sync` or `substrate world deps install`.

This ADR makes install behavior explicit by introducing install classes and routing OS packages through an explicit
provisioning command on supported guest platforms only.

## 1) Decisions (final)

### D1 — Selection is required; missing selection is a no-op

- `substrate world deps status|sync|install|provision` must be a no-op when no selection file exists.
- No-op means:
  - exit code `0`
  - no world backend calls
  - one prominent “not configured (selection file missing)” line plus next steps

Selection filename is fixed:
- `world-deps.selection.yaml`

Selection paths and precedence are fixed:
1) `.substrate/world-deps.selection.yaml` (workspace)
2) `~/.substrate/world-deps.selection.yaml` (global)

If both exist, the workspace file is active and the global file is shadowed.

### D2 — Install classes are explicit metadata in the manager manifest

Install class metadata must live in the layered manager manifest schema (inventory + overlays). It must not live in
the selection file.

The initial install class set is:
- `user_space`: runtime install allowed into the Substrate-managed prefix
- `system_packages`: runtime install blocked; provisioning required
- `manual`: runtime blocked; instructions required
- `copy_from_host`: schema allowed; runtime fails with exit code `4` (“unsupported”)

### D3 — Substrate-owned user-space installs use one writable prefix

All `user_space` installs must target a single, stable writable prefix inside the world:
- `/var/lib/substrate/world-deps`

### D4 — Command surface is stable and explicit

World-deps commands under this ADR are:
- `substrate world deps status [--json] [--all] [TOOL ...]`
- `substrate world deps sync [--all] [--dry-run] [--verbose]`
- `substrate world deps install [--all] [--dry-run] [--verbose] TOOL ...`
- `substrate world deps init [--workspace|--global] [--force]`
- `substrate world deps select [--workspace|--global] TOOL ...`
- `substrate world deps provision [--all] [--dry-run] [--verbose]`

`--all` must ignore selection and use full inventory scope. It must not override the “selection missing → no-op”
contract.

### D5 — Provisioning-time system packages are supported only in guests

`substrate world deps provision` must install OS packages only on these platforms:
- macOS (Lima guest): supported, apt-only
- Windows (WSL guest): supported, apt-only

Linux host backend is explicitly unsupported for provisioning:
- `provision` must exit `4` and print manual package guidance.

### D6 — Full cage compatibility is a hard requirement

When full cage is enabled (`world_fs.cage=full`), the world rootfs must mount:
- `/var/lib/substrate/world-deps` read-write

If `world deps sync|install|provision` cannot write to the prefix due to hardening/cage constraints, it must exit `5`
with a message pointing to the required mount and the hardening specs (I2/I3).

### D7 — Exit codes are stable across all world-deps subcommands

World-deps exit codes are:
- `0`: success, including intentional no-op due to missing selection
- `2`: configuration/usage error (invalid YAML, unknown tool name, schema mismatch)
- `3`: world backend unavailable when required for the operation
- `4`: unmet prerequisites or unsupported operation for the requested scope
- `5`: hardening/cage conflict prevents the operation

## 2) Operational model (single mental model)

Inputs:
- Inventory + overlays:
  - base: `config/manager_hooks.yaml`
  - overlays: `~/.substrate/manager_hooks.local.yaml`, `scripts/substrate/world-deps.yaml`, `~/.substrate/world-deps.local.yaml`
- Selection:
  - `.substrate/world-deps.selection.yaml` (workspace)
  - `~/.substrate/world-deps.selection.yaml` (global)

Derived state:
- `configured` vs `not configured` selection
- active scope (selection vs inventory via `--all`)
- routing per tool via install class

Actions:
- `status`: no side effects; best-effort probes
- `sync/install`: install only `user_space`; block on `system_packages` and `manual`
- `provision`: install only `system_packages` on supported guest platforms

## 3) Sequencing contract (final)

This ADR is implemented by the `world_deps_selection_layer` triad:
- WDL0 → selection config + UX
- WDL1 → install classes
- WDL2 → provisioning-time system packages

Sequencing must match `docs/project_management/next/sequencing.json` and the prerequisites listed in
`docs/project_management/next/world_deps_selection_layer/integration_map.md`.
