# ADR-0001: Agent Hub Runtime Config + World-Deps UX + Linux Isolation Guarantees

Status: Draft (partially implemented; see “Implementation Status”)

Last updated: 2025-12-22

Owners: Shell/World/Broker maintainers

Related work:
- macOS parity triad (M4–M6): `docs/project_management/next/p0-platform-stability-macOS-parity/`
- Earlier platform stability tracks: `docs/project_management/next/p0-platform-stability/`

---

## Executive Summary (Operator)

ADR_BODY_SHA256: 1c743bfa97e7e8f4c6ebfd76e66481fa63ce920553348c09fc58a566dc83f649

ADR_BODY_SHA256: <run `python3 scripts/planning/check_adr_exec_summary.py --adr <this-file> --fix` after editing>

- Existing: Substrate can feel like a “best effort wrapper” where some operations (world-deps, config, isolation) are ambiguous, drift-prone, or silently degrade when guarantees are not met.
- New: Substrate behaves like a strict agent-hub execution layer: selection-driven world-deps (no-op when unconfigured), YAML-only runtime config, and Linux isolation guarantees that prevent absolute-path project writes from bypassing the overlay.
- Why: Reduces security footguns and makes guarantees auditable/reproducible across tools (humans + multiple agent CLIs).
- Links:
  - `docs/project_management/next/world_deps_selection_layer/`
  - `docs/project_management/next/yaml-settings-migration/Y0-spec.md`

## 0) Executive Summary

Substrate is evolving into a multi-agent hub where multiple frontier-model CLIs (Codex, Claude, Gemini, and similar tools)
execute through a single, policy-controlled “secure execution layer”. That only works if:

1) **Policies parse reliably and predictably** (clear schema + strong validation).
2) **World-deps does not spam or mutate tooling** unless explicitly configured/selected.
3) **Isolation claims match reality** (no “false sense of security”), especially on Linux where the host
   kernel primitives vary by distro and privilege posture.

This ADR records the decisions, gaps, and an implementation plan to:
- Add a **world-deps selection/allowlist layer** and gate behavior on its existence.
- Standardize Substrate-owned runtime settings on **YAML-only** with no dual-format support (Y0).
- Fix a critical Linux gap: **absolute-path writes into the project bypassing the overlay**.
- Move toward **fail-closed** security semantics for policies that require guarantees.
- Plan “full caging” hardening (pivot_root/minimal rootfs) with Landlock as additive hardening only.

---

## 1) Context and Problem Statement

### 1.1 Substrate as an agent hub changes the threat model

Substrate is not just a convenience wrapper around `bash`. It is intended to be the centralized execution
and policy boundary for multiple independent tools (human shell, “agent shells”, and third-party agent CLIs).
Therefore:
- We cannot rely on the *caller* to “behave” (e.g., avoid absolute paths).
- We must prevent “surprising implicit behavior” (e.g., world-deps installing things without user intent).
- We must avoid “silent degrade” when a requested security boundary cannot be enforced.

### 1.2 World-deps inventory regression (M5a alignment)

After M5a, `substrate world deps` is driven by the **canonical manager inventory** from the manager manifest
(`config/manager_hooks.yaml`). This matches the M5a spec: canonical inventory comes from `manager_hooks.yaml`,
and `world-deps.yaml` is treated as an overlay.

To reduce noise, the current UX can filter by default (host-present managers) and expose the full canonical
set behind `--all`. That helps, but it still does not solve the more important requirement: operators need an
explicit selection/allowlist so `world deps` only targets what they opted into, and does nothing when no
selection exists.

Desired behavior going forward:
- World-deps must **only show and act on tools that are configured/selected**.
- If no world-deps selection config exists, world-deps must **report “not configured” and do nothing**.
- World-deps configuration and UX must live behind `substrate world …` and must not use the global `substrate config …` stack.

### 1.3 Runtime settings format: YAML-only

Runtime settings are YAML-only (Y0):
- Global config: `~/.substrate/config.yaml`
- Workspace settings: `.substrate/settings.yaml`
- CLI: `substrate config init/show/set`

TOML settings files are unsupported after Y0:
- `~/.substrate/config.toml`
- `.substrate/settings.toml`

Note: Lima templates remain YAML due to external tooling expectations.

### 1.4 Linux isolation guarantee gap (absolute-path escape)

On Linux, “world” provides isolation using overlay/copy-diff and other primitives, but historically it did not
fully remap the process’s filesystem root (no pivot_root/chroot), and “caging” for the interactive shell
was primarily a `cd` guard.

Consequence: even if the project is mounted read-only through overlay/copy-diff, a process can still write
into the host project directory by using an absolute path (e.g., `touch /home/user/project/file`), bypassing
the overlay root anchored at the current working directory.

This is a major issue for an agent hub: it undermines operator expectations and makes policy claims unreliable.

---

## 2) Decisions

This ADR contains multiple related decisions. Each decision lists its status.

### D1 — World-deps must be selection-driven (and no-op when unconfigured)

Status: Accepted

Decision:
- Keep a **canonical inventory** sourced from `config/manager_hooks.yaml`.
- Introduce a **separate selection config** (an allowlist) that determines what `world deps` commands show and act on.
- Selection filename is fixed: `world-deps.selection.yaml`.
- Selection paths are fixed:
  - `.substrate/world-deps.selection.yaml` (workspace)
  - `~/.substrate/world-deps.selection.yaml` (global)
- Precedence is fixed: workspace overrides global.
- If no selection config exists, `substrate world deps status|install|sync|provision` must:
  - print one prominent “not configured (selection file missing)” line plus next steps
  - exit `0`
  - perform no side effects, including no world backend calls

Rationale:
- Inventory is “what Substrate *can* manage”; selection is “what the operator *wants* Substrate to manage”.
- Avoids surprising outputs for tools the operator does not care about.
- Supports teams: repo/workspace can ship a canonical inventory while each user opts into a subset.

### D2 — World-deps selection config must not be confusing to users

Status: Accepted

Decision:
- Do not rely on “same base name, one-character extension differences” to distinguish files (too easy to
  confuse during manual edits and support).
- Prefer distinct names for distinct concepts (inventory vs selection vs overlay).

Concrete naming (fixed):
- Canonical inventory: `config/manager_hooks.yaml` (existing)
- Manifest overlay(s): `scripts/substrate/world-deps.yaml` (installed) and `~/.substrate/world-deps.local.yaml`
  (user overlay) (existing)
- Selection config (new): `world-deps.selection.yaml`
  - Workspace: `.substrate/world-deps.selection.yaml`
  - Global: `~/.substrate/world-deps.selection.yaml`

### D3 — Runtime settings format is YAML-only

Status: Accepted (implemented in Y0)

Decision:
- Substrate-owned runtime settings must be YAML-only with no dual-format support.
- TOML runtime settings files are unsupported.
- The authoritative spec for this migration is `docs/project_management/next/yaml-settings-migration/Y0-spec.md`.

### D4 — `.substrate-profile` schema is greenfield (strict + versioned)

Status: Accepted (greenfield; backward-compat not a goal)

Decision:
- Substrate policy/profile schema is treated as **greenfield**: backward compatibility with legacy keys or
  older profiles is **not** a priority.
- The system must fail with actionable diagnostics for missing/invalid fields; it must not silently fall back to a permissive mode.
- Filesystem control must remain first-class (including path-level allow/deny), and it must be expressed in
  the unified model (see §4.3) rather than as legacy compatibility keys.

### D5 — Fail-closed for requested security guarantees

Status: Implemented for “project read-only overlay enforcement”; broader policy semantics proposed

Decision:
- When a policy requests a guarantee (e.g., read-only project filesystem), and the system cannot enforce it,
  Substrate must **fail closed** (refuse to run the world path) rather than silently running with weaker protection.
- When a policy does not request a guarantee, Substrate must follow the policy’s explicit `world_fs.require_world` setting (I1).

Rationale:
- “Secure execution layer” must be reliable. Silent degrade becomes a security bug in an agent-hub context.

### D6 — Immediate Linux hardening: prevent absolute-path project escapes

Status: Implemented (non-PTY + PTY paths)

Decision:
- For in-world execution on Linux, run the child in a private mount namespace and **bind-mount** the merged
  overlay root onto the *real host project directory path* before executing the command.
- This ensures that absolute paths under the project root resolve into the overlay, matching relative-path
  behavior.
- If `world_fs.mode=read_only` is requested and this enforcement cannot run, fail closed (D5).

Limitations (explicitly acknowledged):
- This only fixes escapes **into the project directory**.
- It does **not** prevent a process from touching other host paths (e.g., `/tmp`, `$HOME/other_project`, and other host paths).
  Full caging requires a bigger upgrade (see D7).

### D7 — Plan “full caging” as a separate hardening milestone

Status: Proposed (next sprint(s))

Decision:
- Treat “fail-closed semantics” and “full caging isolation” as separate deliverables:
  1) **I1: Fail closed** (policy semantics and enforcement checks).
  2) **I2/I3: Full caging** via mount-namespace + pivot_root/minimal rootfs (container-style).
  3) **I4: Landlock** as additive hardening inside a full cage, never as the primary guarantee.

Rationale:
- Fail-closed stops the most dangerous failure mode (silent downgrade) immediately.
- Full caging is inherently larger and requires careful cross-distro detection and parity between PTY/non-PTY.

### D8 — Landlock is a quality-of-life feature, not a primary guarantee

Status: Proposed

Decision:
- Landlock is additive hardening only and is never the primary isolation guarantee.
- When `world_fs.cage=full`, Substrate must:
  - enforce isolation using mount namespace + `pivot_root` (I2/I3), and
  - apply Landlock restrictions inside the cage when the kernel supports it (I4).
- When `world_fs.cage=full` is requested and `pivot_root` cannot be used, Substrate must fail closed (I1); it must not fall back to Landlock-only enforcement.

### D9 — Documentation must align with actual enforced guarantees

Status: In progress (some docs updated; remaining alignment needed)

Decision:
- Avoid implying “full filesystem isolation” on Linux unless it is actually enforced.
- Explicitly document:
  - what is guaranteed today (overlay/copy-diff behavior; project-path bind mount)
  - what is not (access outside project unless full caging is enabled)
  - what triggers fail-closed vs best-effort fallback

---

## 3) Implementation Status (as of 2025-12-22)

### 3.1 `.substrate-profile` parsing resilience (Implemented; removed by I0)

Current code includes serde defaults so minimal profiles parse without requiring all fields, plus a regression test.
I0 removes these defaults and enforces a strict schema with actionable diagnostics.

### 3.2 Linux: absolute-path project escape mitigation (Implemented)

Implemented the “project bind mount” enforcement:
- Non-PTY execution: run the child inside a private mount namespace and bind-mount overlay merged root onto the
  host project dir path prior to running the command.
- PTY execution: apply the same principle by wrapping the PTY child spawn in an `unshare` wrapper and performing
  the bind mount before `exec`.
- Read-only mode fail-closed if the enforcement wrapper cannot run.

### 3.3 World-deps selection layer (Not implemented yet)

The selection-driven world-deps model is specified and sequenced:
- ADR: `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- Specs: `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`, `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`, `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`
- Sequencing: `docs/project_management/next/sequencing.json`

### 3.4 Runtime format unification (Not started)

Runtime settings format is YAML-only and is implemented by Y0:
- `docs/project_management/next/yaml-settings-migration/Y0-spec.md`

---

## 4) Final UX and schema summary

World-deps selection + install-class semantics are finalized in ADR-0002 and its triad specs:
- `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

Policy schema is finalized by I0/I1:
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I0-spec.md`
- `docs/project_management/next/p0-agent-hub-isolation-hardening/I1-spec.md`

---

## 5) Isolation Hardening: Options and Tradeoffs

### 5.1 What we have today (Linux)

Current “world” primitives provide meaningful isolation but not full caging:
- Overlay/copy-diff for non-PTY command filesystem effects and fs_diff capture.
- Cgroups, nftables, and (in some flows) netns.
- “Caged root” guard for `cd` in interactive shell (a UX guard, not kernel enforcement).

After the “project bind mount” fix (D6):
- Absolute paths under the project root map into the overlay.
- The project escape via `/home/user/project/...` is mitigated.

Still not solved:
- Access to other host paths.

### 5.2 Option A: Mount namespace + pivot_root (container-style)

Goal: make the process’s `/` no longer the host’s `/`, so host paths are not nameable unless explicitly mounted.

Conceptual steps:
- `unshare(CLONE_NEWNS)`; make mounts private.
- Build a minimal filesystem tree (new root).
- Bind-mount only the allowed directories (project overlay, `/usr`, `/lib*`, and other required read-only system mounts) as needed.
- Mount a fresh `/proc`, a minimal `/dev`, writable `/tmp` (tmpfs).
- `pivot_root(new_root, new_root/old_root)`; unmount `old_root`.

Pros:
- Strongest “full caging” semantics without relying on LSM availability.
- Predictable; matches container mental model.

Cons:
- Host variance: user namespaces and mount operations may be restricted by distro policy.
- Requires careful parity between PTY and non-PTY paths.
- More complex to maintain.

### 5.3 Option B: Landlock (unprivileged filesystem sandbox)

Goal: restrict the process’s filesystem access by path allowlists enforced by the kernel (LSM).

Pros:
- Unprivileged; often easier to deploy where mount namespace operations are blocked.
- Directly aligned with path-level allowlists (fits policy UX well).

Cons:
- Not universally available (kernel version/config).
- Semantics are allowlist-based and require careful defaulting.
- Still needs consistent integration with exec and PTY flows.

### 5.4 Recommended path: layered hardening

1) Adopt clear policy semantics + fail-closed behavior (Sprint A).
2) Implement pivot_root full-cage mode where possible (Sprint B).
3) Add Landlock as extra hardening and/or fallback when pivot_root cannot be used (Sprint C / QoL).

This keeps us honest about guarantees while improving security steadily.

---

## 6) Consequences

### Positive
- Prevents silent policy failure (critical for an agent hub).
- Removes a concrete and easy-to-hit Linux escape into the project directory.
- Makes world-deps predictable and intent-driven.
- Sets a path to unify policy schema without legacy baggage.

### Negative / costs
- More explicit errors in environments that previously “worked” with hidden fallback.
- Additional complexity in Linux execution wrappers (especially PTY).
- Format unification is a large refactor (either direction).

---

## 7) Resolved alignment points

1) Runtime settings format is YAML-only (Y0) with no dual-format support.
2) World-deps selection config naming is fixed to `world-deps.selection.yaml` (ADR-0002).
3) `world_fs.mode=read_only` means “project is read-only”; `world_fs.cage` controls whether only the project path is protected (`project`) or whether the process is fully caged (`full`) (I0/I1/I2/I3).
4) Full-cage fallback ladder is explicit:
   - If `world_fs.cage=full` is requested and cannot be enforced, Substrate fails closed (I1).
   - Landlock is additive hardening only and never a fallback for full cage (I4).
5) Docs alignment is owned by I5, and documentation must match actual enforced guarantees.

---

## 8) References

- `README.md` (product framing: secure execution layer + multi-agent hub)
- `docs/WORLD.md` (world architecture, routing, and world-deps UX)
- `docs/VISION.md` (roadmap and policy examples; needs alignment with enforcement guarantees)
- `docs/project_management/future/PHASE_4_5_ISOLATION_UPGRADE.md` (design draft; notes mount hardening as future)
- M5a inventory alignment spec: `docs/project_management/next/p0-platform-stability-macOS-parity/M5a-spec.md`
- M5a integration notes: `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
