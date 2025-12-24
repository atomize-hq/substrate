# ADR-0002: World-Deps Install Classes + Provisioning-Time System Packages
Status: Draft

Last updated: 2025-12-24

Owners: Shell/World/Installer maintainers

Related sprints/tracks:
- YAML settings migration (Y0): `docs/project_management/next/yaml-settings-migration/`
- Agent hub isolation hardening (I0–I5): `docs/project_management/next/p0-agent-hub-isolation-hardening/`
- World sync (C0–C9): `docs/project_management/next/world-sync/`
- Sequencing: `docs/project_management/next/sequencing.json`

## 0) Executive Summary

We need `substrate world deps` to be:
1) predictable and selection-driven (no surprise installs),
2) cross-platform consistent (“worlds should feel the same”),
3) compatible with upcoming full-cage hardening (pivot_root/Landlock), and
4) safe under an agent-hub threat model (no silent escalation of privileges).

Today, some world-deps guest installers assume they can mutate the guest OS at runtime (e.g., `apt-get install`).
This is fragile across platforms and conflicts with full-cage goals. This ADR proposes a capability model (“install
classes”) that makes install behavior explicit and routes OS-level packages through provisioning-time mechanisms,
while keeping world-deps as the user-space tool mirror layer by default.

## 1) Context

### 1.1 Current world-deps model (inventory + overlays)

- Canonical inventory is sourced from the manager manifest:
  - `config/manager_hooks.yaml` (plus `~/.substrate/manager_hooks.local.yaml`).
- `world-deps.yaml` is treated as an overlay layer:
  - installed overlay: `<prefix>/versions/<version>/config/world-deps.yaml` (or workspace fallback `scripts/substrate/world-deps.yaml`)
  - user overlay: `~/.substrate/world-deps.local.yaml`

This is aligned with M5a (“inventory alignment”) and the hardening ADR-0001.

### 1.2 Upcoming constraints from the next sprints

From `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`:
- World-deps must be selection-driven and no-op when unconfigured.
- Full-cage is planned (mount namespace + pivot_root), with optional Landlock allowlists.
- Fail-closed semantics are expected when guarantees are requested.

From `docs/project_management/next/yaml-settings-migration/Y0-spec.md` and `docs/project_management/next/sequencing.json`:
- Runtime settings/config are migrating to YAML-only (`config.yaml`, `.substrate/settings.yaml`).
- New settings work should not land until Y0 completes.

From `docs/project_management/next/world-sync/*`:
- Workspaces will be explicitly initialized/gated via `substrate init` and `.substrate/` (C0).
- World sync introduces a workspace-local configuration surface and a strong “protected paths” model.

These imply that “system package installs” must be treated as a distinct, explicit capability—not a default behavior
of world-deps.

## 2) Problem Statement

We currently have (or risk having) world-deps install recipes that:
- assume OS-level package managers are available and writable (`apt-get`, `dpkg`, etc.),
- may require capabilities/permissions that are unsafe to grant broadly under an agent-hub threat model, and
- will likely conflict with full-cage hardening where the guest OS is intentionally immutable or minimal.

This makes behavior inconsistent across:
- macOS (Lima guest) vs Linux (host world-agent) vs Windows (WSL),
- prod install vs dev install,
- policy modes (read-only vs writable),
- and future hardening states.

## 3) Decision

### D1 — Introduce an explicit “install class” model for world-deps

World-deps installation recipes must declare what *kind* of install they require. The system uses this metadata
to decide whether the operation is:
- allowed under the current world/policy/cage mode,
- routed to provisioning-time mechanisms, or
- rejected with a clear error.

Proposed install classes (initial):

1) `user_space` (default / preferred)
   - Installs into a Substrate-managed writable prefix inside the world (e.g. `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`
     and a stable root like `/var/lib/substrate/world-deps/`).
   - Must not mutate OS package databases or system directories.
   - Expected to work across all world backends (Linux, macOS Lima, Windows WSL) as long as the prefix is writable.

2) `system_packages`
   - Installs OS packages (e.g. via apt/dnf/apk/brew/winget inside the guest).
   - Not executed directly by “world-deps sync/install” via the standard execution path.
   - Instead, it is fulfilled by:
     - provisioning-time package installation (VM/distro bootstrap), or
     - an explicit, separately-gated “system packages” command path with elevated semantics and clear audit logging.

3) `copy_from_host`
   - Sync/copy a host-installed tool into the world prefix (where applicable and safe).
   - Must define deterministic source paths and a checksum/verification strategy.

4) `manual`
   - Recipe exists but Substrate will not automate it; world-deps prints the instructions.

### D2 — Default world-deps behavior is selection-driven and conservative

Consistent with ADR-0001:
- If selection config does not exist, world-deps does nothing (and says “not configured”).
- `--all` (or an equivalent “inventory debug” flag) must be explicit about whether it ignores selection.

### D3 — Route `system_packages` through provisioning-time plans

For platforms with a VM/distro provisioning step (macOS Lima, Windows WSL), OS packages needed for managed
tooling should be installed during provisioning (or a re-provision/upgrade step), not during `world deps sync`.

For Linux host world-agent, OS packages are host packages; Substrate must not silently modify them as a side effect
of world-deps. Any host package management must be explicit, opt-in, and policy-gated.

## 4) Consequences

### Positive
- Aligns “worlds should feel the same” with upcoming full-cage hardening.
- Makes privilege boundaries explicit (no accidental escalation by running apt inside world-deps).
- Gives a scalable way to support “whatever the user wants” by adding new install classes/handlers.
- Integrates cleanly with workspace gating (`substrate init`) and YAML-only settings.

### Costs / tradeoffs
- Requires schema work: selection config + recipe metadata.
- Requires clear UX design for “this tool needs system packages” vs “this tool installs in user-space”.
- Some existing recipes will need to be reclassified and migrated.

## 5) Proposed Execution Plan (where this lands in existing sprints)

This ADR is intended to be implemented as part of the already-planned placeholder step:
- `world_deps_selection_layer` in `docs/project_management/next/sequencing.json` (inside the world-sync sprint sequence).

Concrete deliverables to add (new triad + spec directory):
- `docs/project_management/next/world-deps-selection/` (new sprint subdir under `next/`)
  - `plan.md`, `tasks.json`, `session_log.md`
  - triad specs:
    - `W0-spec.md` (world-deps selection config + CLI UX)
    - `W1-spec.md` (install class schema + enforcement + messaging)
    - `W2-spec.md` (provisioning-time system packages strategy per platform)
  - kickoff prompts per triad (code/test/integ)

Dependencies:
- Must land after YAML settings migration Y0 (YAML-only runtime config).
- Must align with agent hub isolation hardening I0/I1 semantics around “required guarantees” and fail-closed behavior.

## 6) Open Questions

1) Where should selection config live?
   - Workspace-local (preferred): `.substrate/world-deps.selection.yaml`
   - Global fallback: `~/.substrate/world-deps.selection.yaml`
2) How should install class metadata be represented?
   - Inline in selection config (per selected tool), or
   - as part of the manager manifest schema (inventory), or
   - via a dedicated overlay file (installed + user overrides).
3) What is the explicit UX for `system_packages`?
   - `substrate world deps provision`? `substrate world upgrade`?
4) How do we define “writable prefix” consistently across worlds?
   - One canonical path (e.g. `/var/lib/substrate/world-deps`) vs per-backend.
5) How do we avoid drift between “what provisioning installs” and “what world-deps expects”?
   - Versioned provisioning manifests? compatibility checks in `world doctor`?

---

## Appendix A — DETAILED RESEARCH PROMPT (for finalizing the remaining plans/triads)

You are an expert maintainer of Substrate. Your job is to produce a concrete, reviewable plan (specs + tasks + UX)
to implement ADR-0002 without ad-hoc patches and without regressing existing platforms.

Constraints:
- Do not write production code; this is a planning/research output.
- Treat YAML-only runtime config as a hard requirement (see `docs/project_management/next/sequencing.json` and
  `docs/project_management/next/yaml-settings-migration/Y0-spec.md`).
- Assume agent hub hardening will introduce full caging (pivot_root) and optional Landlock; plan must be compatible.
- World-deps must be selection-driven and no-op when unconfigured (ADR-0001 D1/D2).
- Maintain “worlds should feel the same” as a product expectation: the same workflow should succeed on macOS Lima,
  Linux, and Windows WSL where technically possible; otherwise fail with explicit, actionable errors.

Required inputs to read:
1) `docs/project_management/next/p0-agent-hub-isolation-hardening/ADR-0001-agent-hub-runtime-config-and-isolation.md`
2) `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md` through `I5-spec.md`
3) `docs/project_management/next/yaml-settings-migration/Y0-spec.md` and its plan/tasks
4) `docs/project_management/next/world-sync/plan.md`, `C0-spec.md`, `C1-spec.md` (and skim C2–C5 for gating/filters patterns)
5) Current implementation touchpoints:
   - manager manifest + overlays: `config/manager_hooks.yaml`, `scripts/substrate/world-deps.yaml`, `~/.substrate/world-deps.local.yaml`
   - world-deps runner: `crates/shell/src/builtins/world_deps/*`
   - env normalization for macOS guest: `crates/shell/src/execution/routing/dispatch/world_ops.rs` (`normalize_env_for_linux_guest`)
   - world backend constraints: `docs/WORLD.md`

Deliverables to produce:

### 0) Research artifacts (required)
- Produce two standalone artifacts (new Markdown docs under the proposed triad directory):
  - **Decision register**: every architectural decision captured using the “2 options → pros/cons/implications/risks/unlocks/quick-wins → recommended option + rationale” format.
  - **Integration map**: an explicit, end-to-end map of affected components/surfaces (installer, shell `world deps`, world backends, policy/broker, world-agent) and how the chosen design composes with Y0 (YAML settings), I0–I5 (hardening), and C0–C9 (world-sync).
    - This must call out any required sequencing adjustments and where the work lands (triad/spec/task ownership).
  - These artifacts must be persisted as:
    - `docs/project_management/next/world_deps_selection_layer/decision_register.md`
    - `docs/project_management/next/world_deps_selection_layer/integration_map.md`

### 1) Spec + triad structure
- Propose a new “world_deps_selection_layer” triad directory under `docs/project_management/next/` with:
  - `plan.md`, `tasks.json`, `session_log.md`
  - at least two spec slices:
    - Selection config + UX slice (selection is required; no-op if missing; flag semantics)
    - Install classes slice (schema, enforcement, error messages, how recipes declare class)
  - Include acceptance criteria for each spec that is testable and platform-aware.
- Explicitly map the triads into `docs/project_management/next/sequencing.json` (where it should be inserted and why).

### 2) YAML schema decisions
- Decide the exact filenames/locations for:
  - selection config (workspace vs global; precedence rules),
  - install class metadata (where it lives and how overrides work),
  - any provisioning manifest (if needed).
- Provide example YAML files for:
  - a minimal selection file selecting `nvm`, `pyenv`, `bun`,
  - a tool that requires `system_packages`,
  - a tool that is `manual`.

### 3) CLI/UX decisions
- Propose the exact CLI surface and semantics:
  - `substrate world deps status|sync|install` behavior when unconfigured,
  - how `--all` is defined (inventory vs ignore selection),
  - how the CLI communicates install class requirements.
- Provide sample outputs for:
  - unconfigured state,
  - selection configured but a tool requires system packages,
  - selection configured and tool installs user-space successfully.

### 4) Platform strategy for `system_packages`
- For macOS Lima and Windows WSL: propose how “system packages needed for managed tooling” are installed:
  - provisioning-time step(s),
  - upgrade/repair command (explicit) that re-runs provisioning package installs safely.
- For Linux host world-agent: define the policy:
  - forbid host package mutation via world-deps by default,
  - provide an explicit opt-in command if we want it (and what policy/approval it requires),
  - or treat system packages as manual guidance only.

### 5) Guardrails and failure modes
- Enumerate failure modes and what the system should do:
  - world backend unavailable,
  - selection missing,
  - tool not in inventory,
  - tool requires `system_packages` but provisioning is not supported on this platform,
  - full cage requested and prevents some install behavior.
- For each, specify exit code expectations and whether behavior is warn+continue vs fail-closed.

### 6) Acceptance matrix + automation (optional, recommended)
- If you include an acceptance matrix, it must be automatable:
  - Provide a matrix of user journeys × platforms with concrete pass/fail checks.
  - Propose corresponding smoke-check shell scripts (not manual steps) that validate the matrix rows, in addition to unit/integration tests.
  - Prefer extending existing smoke/doctor scripts in `scripts/` rather than inventing bespoke ad-hoc harnesses.

### 7) Greenfield constraint (hard requirement)
- Assume we are greenfield for this work: do not plan migration/backwards-compatibility layers.
- If a breaking change to an existing workflow is required, record it explicitly as a breaking change (with the new expected behavior) rather than proposing compatibility shims.

Output format requirements:
- Write the plan as if it will be pasted into a PR description and a set of new spec files.
- Be explicit about what is in-scope/out-of-scope for the first shipping increment.
- Avoid vague language; propose concrete file paths, flags, schemas, and acceptance tests.
- Treat cross-sprint alignment as a hard requirement:
  - Inventory all adjacent/queued work that touches world-deps/world-sync/hardening/installer flows (at minimum: Y0, I0–I5, C0–C9, and `docs/project_management/next/sequencing.json`).
  - Identify any sequencing conflicts, missing prerequisites, or spec overlaps; propose the exact sequencing adjustments needed and why.
  - Record the final, aligned sequencing outcome explicitly (no “maybe”; no open dependencies).
- Persist the research from this session as new documents (not just prose in a chat response):
  - Write new Markdown files under the proposed triad directory (e.g., `docs/project_management/next/world_deps_selection_layer/…`) that capture findings with exhaustive detail.
  - Ensure the docs are self-contained and reviewable (a teammate should be able to implement from them without interpretation).
- Record every architectural decision with zero ambiguity and no open questions/TBDs:
  - For each decision, present exactly 2 viable solutions (Option A / Option B).
  - For each option, include: pros, cons, cascading/ripple implications, risks, what the option unlocks, and any low-hanging fruit/quick wins it enables.
  - Conclude with the recommended option and a crisp rationale for why it is selected (trade-offs acknowledged; no “optional” framing).
