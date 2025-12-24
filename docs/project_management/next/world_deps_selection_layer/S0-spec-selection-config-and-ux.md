# S0-spec: World-Deps Selection Config + CLI/UX (ADR-0002)

## Scope
Define the YAML-only selection config and the exact CLI/UX semantics for `substrate world deps …` under the selection-driven model required by ADR-0001 D1/D2 and ADR-0002.

This spec is **greenfield**:
- No migration layers.
- Any behavior changes are recorded explicitly as breaking changes.

## Non-negotiable requirements
- Selection is required; missing selection → **no-op** for `status|sync|install|provision`, exit 0, no side effects.
- YAML-only runtime config (no TOML surfaces; requires Y0 landed).
- Worlds must feel the same across platforms: where technically possible the same workflow succeeds; otherwise it fails with explicit, actionable errors.
- `--all` semantics are explicit and consistent across subcommands (see DR-0005).

---

## Files and precedence

### Selection config paths
Selection filename is fixed (DR-0002): `world-deps.selection.yaml`.

Paths (DR-0001):
1) Workspace selection: `.substrate/world-deps.selection.yaml`
2) Global selection: `~/.substrate/world-deps.selection.yaml`

Precedence:
- If workspace selection exists, it is the **active** selection and global selection is ignored (“shadowed”).
- Otherwise, if global selection exists, it is the active selection.
- Otherwise selection is “not configured”.

### Related manifests (unchanged by S0; used by later slices)
- Canonical inventory: `config/manager_hooks.yaml` (+ `~/.substrate/manager_hooks.local.yaml`)
- Installed overlay: `scripts/substrate/world-deps.yaml`
- User overlay: `~/.substrate/world-deps.local.yaml`

---

## YAML schema (selection config)

### Schema
```yaml
version: 1
selected:
  - <tool_name>
  - <tool_name>
```

Rules:
- `version` is required and must be `1`.
- `selected` is required and must be a YAML list of non-empty strings.
- Tool name matching is case-insensitive, but the system normalizes names to lower-case in output.
- Every selected tool name must exist in the loaded inventory (including overlays). Unknown tools are a configuration error.

### Example: minimal selection file selecting `nvm`, `pyenv`, `bun`
Workspace example (`.substrate/world-deps.selection.yaml`):
```yaml
version: 1
selected:
  - nvm
  - pyenv
  - bun
```

---

## CLI surface (required)

### Commands
Existing:
- `substrate world deps status [--json] [--all] [TOOL ...]`
- `substrate world deps sync [--all] [--dry-run] [--verbose]`
- `substrate world deps install [--all] [--dry-run] [--verbose] TOOL ...`

New (required for selection-driven UX):
- `substrate world deps init [--workspace|--global] [--force]`
  - Writes a selection file containing an empty selection list.
  - Default target: `--workspace` if `.substrate/` exists; otherwise `--global`.
- `substrate world deps select [--workspace|--global] TOOL ...`
  - Adds tools to the selection file (creates it if missing at that scope).
  - Validates tool names against inventory; rejects unknown tools.

`S2` adds:
- `substrate world deps provision [--all] [--dry-run] [--verbose]`

---

## Selection gating + scope rules

### Configured vs unconfigured states

Unconfigured selection:
- No selection file exists at either scope.
- Behavior for `status|sync|install|provision`:
  - Print a single prominent line: `substrate: world deps not configured (selection file missing)`
  - Print next steps (exact paths + init/select commands).
  - Exit code: `0`.
  - Side effects: none (including no world-agent calls).

Configured selection:
- An active selection file exists (workspace or global).
- The tool scope is determined by:
  - default scope: selected tools only
  - `--all`: ignore selection and use full inventory scope (DR-0005)
  - explicit `TOOL ...` args: filter the active scope down to the named tools

### Configured but empty selection (`selected: []`)
This state is created by `substrate world deps init` and is **valid configuration** (see `decision_register.md` DR-0013).

Semantics when `selected` is empty and `--all` is not used:
- `status`:
  - Prints: “Selection configured but empty; no tools selected.”
  - Exits `0`.
  - Makes **no world-agent calls** (no guest probes) because scope is empty.
- `sync` and `provision`:
  - Print: “No tools selected; nothing to do.”
  - Exit `0`.
  - Make **no world-agent calls**.
- `install TOOL...`:
  - Fails with exit `2` unless `--all` is used, because named tools are not selected.
  - Message: “tool not selected; add it to selection or pass --all”.

When `--all` is used, selection is ignored and normal behavior applies (including world-agent calls for `sync/provision`).

### Explicit tool args behavior
- `status TOOL...`:
  - If a tool is not selected (and `--all` not used), it is still shown but marked `selected=false` and `guest.status=skipped` with reason “not selected”.
  - Rationale: supports discovery and reduces confusion (“why didn’t it install?”) without expanding install scope.
- `install TOOL...`:
  - If `--all` is **not** used: every named tool must be selected; otherwise fail with exit code `2` (usage/config error) and a message “tool not selected; add it to selection or pass --all”.
  - If `--all` is used: selection is ignored; install scope is the named tools.

---

## `--all` semantics (breaking change)
`--all` is redefined (DR-0005):
- It ignores selection and uses the full inventory scope for the operation.
- It does **not** override the unconfigured-state no-op requirement (DR-0004).

Breaking change from current behavior (`docs/WORLD.md`):
- Previously `--all` primarily meant “include host-missing tools”.
- Now it means “ignore selection and use full inventory”.

---

## Sample outputs (required by ADR-0002)

### Unconfigured state (`status|sync|install|provision`)
```
substrate: world deps not configured (selection file missing)
Next steps:
  - Create a selection file: substrate world deps init --workspace
  - Discover available tools: substrate world deps status --all
```

### Selection configured but a selected tool is `system_packages`
```
Selection: .substrate/world-deps.selection.yaml (workspace)
pyenv: blocked (install_class=system_packages)
  Requires OS packages. Run:
    substrate world deps provision
```

### Selection configured and user-space tool installs successfully
```
Selection: .substrate/world-deps.selection.yaml (workspace)
Installing `bun` (install_class=user_space)...
✓ `bun` installed successfully.
```

---

## Output requirements (human + JSON)

### Human output (status)
`status` must show:
- Active selection path and whether it’s workspace or global.
- Selected tool set size.
- When `--all` is used, a clear line: “Selection ignored due to --all”.
- A table containing at least:
  - tool name
  - selected yes/no
  - install class (from S1)
  - host detected yes/no (best-effort)
  - guest status (present/missing/skipped/unavailable) with reason

### JSON output (status)
`status --json` must include:
- `selection` block:
  - `configured: bool`
  - `active_path: string|null`
  - `active_scope: workspace|global|null`
  - `shadowed_paths: [string]`
  - `selected: [string]` (normalized)
  - `ignored_due_to_all: bool`
- `tools[]` entries include `selected: bool` and `install_class: string`.

This is intentionally additive to existing JSON output; JSON-mode track (J*) will later standardize formats.

---

## Exit codes (stable taxonomy)

All world-deps subcommands use these exit codes:
- `0`: success, including intentional “no-op” due to missing selection
- `2`: configuration / usage error (invalid YAML, unknown tool name, schema mismatch)
- `3`: world backend unavailable when required for the operation (e.g., `sync/install/provision`)
- `4`: operation did not complete due to unmet prerequisites (e.g., `system_packages` required but not provisioned/supported)
- `5`: hardening/cage prevents the operation (policy/cage conflict; requires explicit operator action)

Notes:
- `status` must avoid returning non-zero for backend unavailability; instead surface `guest.status=unavailable` so diagnostics remain usable in scripts.
- `sync/install/provision` are “action” commands and must return non-zero when they cannot make progress due to backend/prereq/hardening constraints.

---

## Failure modes (required behaviors)

1) **Selection missing**
- Behavior: no-op, exit `0`, print init/select guidance.

2) **Selection YAML invalid**
- Behavior: fail fast, exit `2`, print path + parse error + schema example.

3) **Selected tool not in inventory**
- Behavior: fail fast, exit `2`, list unknown names and suggest `status --all` for discovery.

4) **World backend unavailable**
- `status`: exit `0`, set each guest status to `unavailable` with reason.
- `sync/install/provision`: exit `3` with actionable error (doctor command + platform-specific guidance).

5) **Full cage requested prevents required writes**
- `sync/install/provision`: exit `5` with message pointing to required writable mount `/var/lib/substrate/world-deps` (DR-0008) and the hardening spec (I2/I3).

---

## Acceptance criteria (testable, platform-aware)

### Cross-platform (all)
- With no selection file at either scope:
  - `substrate world deps status` prints “not configured” guidance and exits `0`.
  - `substrate world deps sync` exits `0` and performs no installs.
  - `substrate world deps install nvm` exits `0` and performs no installs.

- With a selection file that is configured but empty (`selected: []`) and `--all` is not used:
  - `substrate world deps status` exits `0` and makes no guest probes.
  - `substrate world deps sync` exits `0` and makes no world-agent calls.
  - `substrate world deps provision` exits `0` and makes no world-agent calls.

- With a valid selection file:
  - `status --json` includes the required `selection.*` fields.
  - `--all` causes `ignored_due_to_all=true` and expands scope to the full inventory.

### macOS (Lima) and Windows (WSL)
- When the world backend is unavailable:
  - `status` exits `0` with guest status `unavailable`.
  - `sync/install` exit `3` and reference `substrate world doctor --json` in output.

### Linux
- `status` remains usable even if world backend is disabled (prints `world_disabled_reason` and exits `0`).

---

## Out of scope (S0)
- Install class schema and routing (S1).
- System package provisioning command (S2).
- Any compatibility with legacy selection formats (greenfield).

---

## Breaking changes (explicit)
- `substrate world deps status|sync|install|provision` do nothing unless a selection file exists (no-op + exit 0).
- `--all` is redefined to mean “ignore selection and use full inventory scope”.
- New YAML selection config file name and locations:
  - `.substrate/world-deps.selection.yaml`
  - `~/.substrate/world-deps.selection.yaml`
