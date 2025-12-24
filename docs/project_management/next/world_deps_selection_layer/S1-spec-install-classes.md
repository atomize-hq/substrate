# S1-spec: World-Deps Install Classes (ADR-0002)

## Scope
Introduce an explicit install-class model for world-deps and enforce it in `substrate world deps status|sync|install`.

Primary objectives:
- Make install behavior explicit and auditable (“what kind of install is this?”).
- Prevent runtime OS package mutation from `world deps sync/install`.
- Ensure compatibility with full caging (pivot_root) and Landlock (when enabled) by constraining where installs can write.

This spec is greenfield:
- Breaking schema changes are allowed and are recorded explicitly.
- No migration layers.

---

## Install classes (initial set)

### `user_space`
- Installs into the Substrate-owned world-deps prefix (DR-0008): `/var/lib/substrate/world-deps`.
- Must not mutate OS package databases or write to system directories (`/usr`, `/etc`, `/var/lib/apt`, etc).
- Must be compatible with full cage:
  - the prefix must be writable inside the cage,
  - the recipe must not depend on host-specific `$HOME` contents.

### `system_packages`
- Represents OS-level packages that must be installed in the world environment.
- Must **never** be installed by runtime `world deps sync/install`.
- Must be fulfilled via the explicit provisioning route (`S2`: `substrate world deps provision`) on platforms where it is supported.
- **Present contract:** the tool is `present` iff `guest_detect.command` exits `0` inside the world (see `decision_register.md` DR-0014).

### `manual`
- Substrate will never automate installation.
- `status` shows explicit instructions; `sync` reports as blocked; `install` fails with actionable guidance.

### `copy_from_host` (reserved; not implemented in Increment 1)
- Schema may allow the value, but behavior is “unsupported” (exit `4`) until implemented.

---

## Manifest schema changes (single source of truth)

Install class metadata lives in the existing layered manager manifest (DR-0006):
- base inventory: `config/manager_hooks.yaml`
- overlays: `~/.substrate/manager_hooks.local.yaml`, `scripts/substrate/world-deps.yaml`, `~/.substrate/world-deps.local.yaml`

### Required schema changes
Manager manifest `version` must bump to `2` (breaking; greenfield; see `decision_register.md` DR-0012).

For each `managers[]` entry, `guest_install` gains new required fields when present:
```yaml
guest_install:
  class: user_space | system_packages | manual | copy_from_host
  custom: |  # only for user_space (shell script)
    ...
  system_packages:  # only for system_packages
    apt:
      - <package>
      - <package>
  manual_instructions: |  # only for manual
    ...
```

Validation rules (must be enforced at manifest load time):
- If `guest_install` is present, `guest_install.class` is required and must be one of the enumerated values.
- `class=user_space` requires `custom` and forbids `system_packages` and `manual_instructions`.
- `class=system_packages` requires `system_packages` and forbids `custom` and `manual_instructions`.
- `class=manual` requires `manual_instructions` and forbids `custom` and `system_packages`.
- `class=copy_from_host` is allowed by schema but must fail at runtime with “unsupported” until implemented.
- `class=system_packages` requires an explicit `guest_detect.command` probe that deterministically checks the prerequisite binaries are available (see `decision_register.md` DR-0014).

Notes:
- Existing `guest_install.apt` is removed in v2. All OS-level package needs must be expressed as structured packages (DR-0007).

---

## Example YAML (install class metadata)

### Tool requiring `system_packages`
Example fragment inside `config/manager_hooks.yaml` (v2):
```yaml
version: 2
managers:
  - name: pyenv
    guest_detect:
      command: "command -v gcc >/dev/null 2>&1 && command -v make >/dev/null 2>&1"
    guest_install:
      class: system_packages
      system_packages:
        apt:
          - make
          - build-essential
          - libssl-dev
          - zlib1g-dev
          - libbz2-dev
          - libreadline-dev
          - libsqlite3-dev
          - xz-utils
          - libffi-dev
          - liblzma-dev
```

### Tool that is `manual`
```yaml
version: 2
managers:
  - name: tool-with-manual-steps
    guest_install:
      class: manual
      manual_instructions: |
        Install this tool manually inside the world, then place a shim in:
          /var/lib/substrate/world-deps/bin/tool-with-manual-steps
        Example:
          curl -fsSL https://example.invalid/tool -o /usr/local/bin/tool-with-manual-steps
          chmod +x /usr/local/bin/tool-with-manual-steps
          ln -sf /usr/local/bin/tool-with-manual-steps /var/lib/substrate/world-deps/bin/tool-with-manual-steps
```

---

## Runtime enforcement (status/sync/install)

### `status`
For each tool in scope (selection or `--all`):
- `install_class` is displayed.
- `guest.status` behavior by class:
  - `user_space`: normal present/missing probe.
  - `system_packages`:
    - If `guest_detect` reports present: show `present`.
    - If `guest_detect` reports missing: show `skipped` with reason: “requires system packages; run `substrate world deps provision`”.
  - `manual`:
    - If `guest_detect` reports present: show `present`.
    - If `guest_detect` reports missing: show `skipped` with reason: “manual install required”.

### `sync`
For each tool in scope:
- `user_space`:
  - If missing in guest, run its `custom` recipe inside the world-deps environment.
- `system_packages`:
  - If `guest_detect` reports present: treat as satisfied and do not block or fail due to this tool.
  - If `guest_detect` reports missing:
    - Do not attempt install.
    - Print fixed guidance to run `substrate world deps provision`.
    - Exit `4` after evaluating the full scope (same as other unmet prerequisites).
- `manual`:
  - Do not attempt install.
  - Print manual instructions.
- `copy_from_host`:
  - Exit `4` with explicit “unsupported in this increment” (do not attempt partial behavior).

Exit code rule:
- `sync` exits `0` only if every in-scope tool is `present` (or `skipped` because the tool is not selected (S0)).
- `sync` exits `4` if any in-scope tool is not `present` due to:
  - `system_packages` (requires `provision`),
  - `manual` (manual install required),
  - `copy_from_host` (unsupported in Increment 1).

### `install TOOL...`
- Requires tool(s) are selected unless `--all` is used (S0).
- Behavior per install class matches `sync`, but failures are treated as immediate:
  - `manual` → exit `4` after printing instructions
  - `system_packages` → exit `4` with message to run `provision`
  - `copy_from_host` → exit `4` unsupported

---

## Full-cage compatibility requirements

When `world_fs.cage=full` (I2/I3), the full-cage rootfs must include:
- `/var/lib/substrate/world-deps` mounted read-write.

If full cage is enabled/required and the world-deps prefix is not writable:
- `sync/install` must fail with exit `5` (hardening conflict) and a message pointing to:
  - the required mount (`/var/lib/substrate/world-deps`)
  - the hardening spec (I2/I3)

---

## Acceptance criteria (testable, platform-aware)

- Manifest validation fails with exit `2` if any tool declares `guest_install` without `class`.
- `world deps sync` never executes OS package managers as part of runtime installs.
- A tool marked `system_packages` is not installed by `sync/install`; it produces actionable output referencing `world deps provision`.
- A tool marked `manual` never triggers a guest install attempt; `install` returns exit `4` and prints manual steps.
- Under full cage, user-space installs still succeed because `/var/lib/substrate/world-deps` is writable; if not, exit `5`.

---

## Out of scope (S1)
- Implementing `world deps provision` (S2).
- `copy_from_host` implementation (explicitly unsupported in Increment 1).
- Migration/back-compat for manifest v1 or selection-less flows (greenfield).
