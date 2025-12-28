# World Deps Selection Layer — Integration Map

This document maps every component and surface area touched by ADR-0002 and the `world_deps_selection_layer` triad, including how the design composes with:
- **Y0** (YAML settings migration),
- **I0–I5** (agent hub isolation hardening),
- **C0–C9** (world-sync and `.substrate/` workspace model).

Authoritative decisions are in:
- `docs/project_management/next/world_deps_selection_layer/decision_register.md`

Specs in this triad:
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

---

## Scope
Covered:
- `substrate world deps status|sync|install|init|select|provision`
- selection config lookup + precedence (workspace/global)
- install class metadata (manifest schema) + enforcement
- provisioning-time routing for `system_packages` (apt-first)
- platform parity behavior and explicit failure modes

Explicitly not covered:
- backwards-compat/migration layers (greenfield constraint)
- implementing `copy_from_host` in the first increment (schema reserved only)

---

## End-to-end data flow (single mental model)

### Inputs
1) Inventory + overlays (existing layering):
   - `config/manager_hooks.yaml` (+ `~/.substrate/manager_hooks.local.yaml`)
   - `scripts/substrate/world-deps.yaml` (+ `~/.substrate/world-deps.local.yaml`)
2) Selection config (new; YAML-only):
   - `.substrate/world-deps.selection.yaml` (workspace)
   - `~/.substrate/world-deps.selection.yaml` (global)
3) World backend availability + platform identity:
   - determined via existing world backend factory + `/v1/capabilities`

### Derived state
1) **Active selection** (configured vs unconfigured; precedence)
2) **Active tool scope**:
   - selection scope by default
   - inventory scope when `--all` is used
3) **Routing decisions per tool** (install class):
   - `user_space`: runtime install allowed
   - `system_packages`: runtime install blocked; provisioning required
   - `manual`: runtime blocked; instructions required

### Actions
- `status`: no side effects; best-effort probes
- `sync/install`: world backend required; installs only `user_space`
- `provision`: world backend required; installs `system_packages` only on supported platforms (Lima/WSL)

---

## Component map (what changes where)

### Host CLI / Shell (`substrate`)
- **Primary code surface:** `crates/shell/src/builtins/world_deps/*`
- **Responsibilities added:**
  - selection config lookup + precedence + strict validation (S0)
  - new actions: `init`, `select`, `provision` (S0/S2)
  - new `--all` meaning (S0; breaking)
  - exit code taxonomy + consistent failure modes (S0/S2)
  - install class routing (S1)

### Shared manifest schema (`crates/common`)
- **Code:** `crates/common/src/manager_manifest/schema.rs`, `crates/common/src/world_deps_manifest.rs`
- **Responsibilities added:**
  - manifest v2 schema:
    - `guest_install.class`
    - `guest_install.system_packages.apt[]`
    - `guest_install.manual_instructions`
  - load-time validation rules that prevent unsafe/ambiguous recipes (S1)

### World backend and env normalization
- **Code:** `crates/shell/src/execution/routing/dispatch/world_ops.rs`
- **Composition requirements (no new code in this triad, but must remain compatible):**
  - Ensure `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR` and PATH normalization still points to `/var/lib/substrate/world-deps/bin` on macOS.
  - Ensure full-cage mounts include `/var/lib/substrate/world-deps` RW (hardening prerequisites; see below).

### World-agent (guest)
- **Surface:** existing `/v1/execute` + `/v1/stream`
- **No new endpoints required** for ADR-0002.
- `provision` uses normal execute path, but runs **only** when explicitly invoked by the operator.

### Broker / policy
- **No direct policy schema changes required** for selection layer.
- Hardening compatibility requirements:
  - full cage must mount `/var/lib/substrate/world-deps` RW (so installs work without writing into the project).
  - Landlock allowlists (if enabled) must not block writes into the world-deps prefix for `world deps` commands.

### Installer / provisioning scripts
- **Feature-local smoke entrypoints:** `docs/project_management/next/world_deps_selection_layer/smoke/`
- **macOS prerequisites:** `scripts/mac/lima-warm.sh`
- **Windows prerequisites:** `scripts/windows/wsl-warm.ps1`
- **Linux prerequisites:** `scripts/linux/world-provision.sh`
- **Responsibilities added:**
  - smoke coverage for new journeys (selection gating, system package provisioning, user-space install) via the feature-local smoke scripts
  - no change to baseline WSL/Lima provisioning required to ship S0/S1 (but S2 needs smoke coverage updates)

---

## Composition with adjacent tracks (Y0, I0–I5, C0–C9)

### Y0 (YAML settings migration)
Dependency:
- WDL slices depend on Y0. Runtime settings are YAML-only, and `world-deps.selection.yaml` is a YAML runtime artifact by design.

Composition:
- Selection config is **not** stored in `config.yaml` / `settings.yaml`; it is a dedicated YAML file by design (avoids mixing “world roots” and “tool selection” concerns).

### I0–I5 (hardening)
Hard requirements imposed on hardening:
- I2/I3 full cage must mount `/var/lib/substrate/world-deps` RW (DR-0008).
- If Landlock is enabled (I4), it must not block writes to the world-deps prefix during world-deps commands (explicitly allow that path).

Hard requirements imposed on world-deps:
- `sync/install` must never mutate OS package state at runtime (S1).
- `provision` must be explicit and logged; Linux host host-package mutation remains unsupported (DR-0010).

### C0–C9 (world-sync)
Dependencies:
- Workspace selection file is under `.substrate/`; C0 establishes `.substrate/` and sets precedent that it is a protected path.

Composition:
- World-deps must not store managed tooling inside the project tree; it uses `/var/lib/substrate/world-deps` to avoid sync interactions.
- If any world-deps artifacts appear in fs diffs, world-sync filters must treat `.substrate/` and `.substrate-git/` as protected, but **world-deps does not rely on those paths**.

---

## Sequencing alignment (final; no open dependencies)

### Required prerequisites (must land first)
1) **Y0**: YAML settings migration (Y0-spec) must land before any WDL slices.
2) **Hardening**: I2/I3 full cage must incorporate the world-deps prefix mount requirement before WDL2 is considered “full-cage compatible”.
3) **World sync**: C0 and C1 land before WDL0 to avoid CLI and workspace-model conflicts.

### Final insertion point in `docs/project_management/next/sequencing.json`
We insert this triad into the **world_sync** sprint after `C1`:
- Rationale: C1 touches CLI parsing/settings surfaces; sequencing WDL after C1 avoids churn/conflicts, and `.substrate/` already exists after C0.

Exact sequence entries (implemented in `docs/project_management/next/sequencing.json`):
- `WDL0` → `S0-spec-selection-config-and-ux.md`
- `WDL1` → `S1-spec-install-classes.md`
- `WDL2` → `S2-spec-system-packages-provisioning.md`

---

## Ownership map (triad/spec/task)
- WDL0 (S0): Shell maintainers (CLI UX, selection parsing, exit codes)
- WDL1 (S1): Shell + common manifest maintainers (schema + routing + manifest edits)
- WDL2 (S2): Shell + installer/script maintainers (provision command + smoke coverage)
