# ADR-0001: Agent Hub Runtime Config + World-Deps UX + Linux Isolation Guarantees

Status: Draft (partially implemented; see “Implementation Status”)

Last updated: 2025-12-22

Owners: Shell/World/Broker maintainers

Related work:
- macOS parity triad (M4–M6): `docs/project_management/next/p0-platform-stability-macOS-parity/`
- Earlier platform stability tracks: `docs/project_management/next/p0-platform-stability/`

---

## 0) Executive Summary

Substrate is evolving into a multi-agent hub where multiple frontier-model CLIs (Codex/Claude/Gemini/etc.)
execute through a single, policy-controlled “secure execution layer”. That only works if:

1) **Policies parse reliably and predictably** (clear schema + strong validation).
2) **World-deps does not spam or mutate tooling** unless explicitly configured/selected.
3) **Isolation claims match reality** (no “false sense of security”), especially on Linux where the host
   kernel primitives vary by distro and privilege posture.

This ADR records the decisions, gaps, and an implementation plan to:
- Add a **world-deps selection/allowlist layer** and gate behavior on its existence.
- Decide **one runtime config format** (YAML-everywhere vs TOML-everywhere) with no dual-format support.
- Fix a critical Linux gap: **absolute-path writes into the project bypassing the overlay**.
- Move toward **fail-closed** security semantics for policies that require guarantees.
- Plan the larger “full caging” hardening (pivot_root/minimal rootfs and/or Landlock).

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
- World-deps should **only show and act on tools that are configured/selected**.
- If no world-deps selection config exists, world-deps should **report “not configured” and do nothing**.
- World-deps configuration and UX should live behind `substrate world …` (world is optional, but UX should be
  separate from the global `substrate config …` stack).

### 1.3 Runtime config format split (TOML vs YAML)

Current reality:
- TOML for the layered runtime config stack:
  - `~/.substrate/config.toml`
  - `.substrate/settings.toml`
  - CLI `substrate config init/show/set`
- YAML for runtime manifests and policies:
  - `config/manager_hooks.yaml`, `~/.substrate/manager_hooks.local.yaml`
  - `scripts/substrate/world-deps.yaml`, `~/.substrate/world-deps.local.yaml`
  - `.substrate/policy.yaml`, `~/.substrate/policy.yaml`, profile defaults

We want to standardize to **one** format for Substrate-owned runtime config/manifests/policies.
No dual-format support is desired for this migration (to avoid long-term complexity).

Note: Lima templates remain YAML due to external tooling expectations.

### 1.4 Linux isolation guarantee gap (absolute-path escape)

On Linux, “world” provides isolation using overlay/copy-diff and other primitives, but historically it did not
fully remap the process’s filesystem root (no pivot_root/chroot), and “caging” for the interactive shell
was primarily a `cd` guard.

Consequence: even if the project is mounted read-only through overlay/copy-diff, a process could still write
into the host project directory by using an absolute path (e.g., `touch /home/user/project/file`), bypassing
the overlay root anchored at the current working directory.

This is a major issue for an agent hub: it undermines operator expectations and makes policy claims unreliable.

---

## 2) Decisions

This ADR contains multiple related decisions. Each decision lists its status.

### D1 — World-deps must be selection-driven (and no-op when unconfigured)

Status: Proposed (design agreed; implementation pending)

Decision:
- Keep a **canonical inventory** sourced from `config/manager_hooks.yaml`.
- Introduce a **separate selection config** (an allowlist) that determines what `world deps` commands show and
  act on.
- If no selection config exists, `substrate world deps status/install/sync` should:
  - print a short “not configured” message
  - exit successfully (no side effects)
  - optionally mention how to configure (`substrate world deps init` / `... select ...`)

Rationale:
- Inventory is “what Substrate *can* manage”; selection is “what the operator *wants* Substrate to manage”.
- Avoids surprising outputs for tools the operator does not care about.
- Supports teams: repo/workspace can ship a canonical inventory while each user opts into a subset.

### D2 — World-deps selection config must not be confusing to users

Status: Proposed (naming/path pending)

Decision:
- Do not rely on “same base name, one-character extension differences” to distinguish files (too easy to
  confuse during manual edits and support).
- Prefer distinct names for distinct concepts (inventory vs selection vs overlay).

Concrete naming proposal (recommended):
- Canonical inventory: `config/manager_hooks.yaml` (existing)
- Manifest overlay(s): `scripts/substrate/world-deps.yaml` (installed) and `~/.substrate/world-deps.local.yaml`
  (user overlay) (existing)
- Selection config (new): `~/.substrate/world-deps.yaml` **or** `~/.substrate/world-deps.selection.yaml`

Notes:
- The originally proposed `~/.substrate/world-deps.yaml` does not path-collide with the installed overlay,
  but it may still be confusing because the name is extremely close to `world-deps.local.yaml`.
- If we want the clearest UX, prefer `world-deps.selection.yaml` for selection and reserve `world-deps*.yaml`
  for overlays only.

### D3 — Pick a single runtime config format (YAML-only vs TOML-only)

Status: Open (research item; no decision yet)

Decision:
- Choose **one** format for Substrate runtime config/manifests/policies: either YAML-only or TOML-only.
- Do **not** implement dual-format support.
- After choosing, create a migration plan and execute it (code + tests + docs).

Evaluation criteria:
- Amount of refactor: number of crates/files touched, tests updated, downstream script/tool impacts.
- Risk surface: parsing/schema validation complexity, error messages, and migration cost.
- Operator ergonomics: editing, comments, anchors/includes (YAML), typing and tooling (TOML), clarity for policies.
- External constraints: Lima YAML templates remain YAML regardless.

### D4 — `.substrate-profile` schema is greenfield (strict + versioned)

Status: Accepted (greenfield; backward-compat not a goal)

Decision:
- Substrate policy/profile schema is treated as **greenfield**: backward compatibility with legacy keys or
  older profiles is **not** a priority.
- We should still be resilient in the face of partial configs (good error messages; no silent fallback to a
  permissive mode), but we can introduce **breaking schema changes** as we converge on the final model.
- Filesystem control must remain first-class (including path-level allow/deny), but it should be expressed in
  the unified model (see §4.3) rather than as legacy compatibility keys.

### D5 — Fail-closed for requested security guarantees

Status: Implemented for “project read-only overlay enforcement”; broader policy semantics proposed

Decision:
- When a policy requests a guarantee (e.g., read-only project filesystem), and the system cannot enforce it,
  Substrate should **fail closed** (refuse to run the world path) rather than silently running with weaker
  protection.
- For less strict modes (e.g., `writable` where the guarantee is not security-critical), Substrate may still
  degrade gracefully with warnings.

Rationale:
- “Secure execution layer” must be reliable. Silent degrade becomes a security bug in an agent-hub context.

### D6 — Immediate Linux hardening: prevent absolute-path project escapes

Status: Implemented (non-PTY + PTY paths)

Decision:
- For in-world execution on Linux, run the child in a private mount namespace and **bind-mount** the merged
  overlay root onto the *real host project directory path* before executing the command.
- This ensures that absolute paths under the project root resolve into the overlay, matching relative-path
  behavior.
- If `world_fs_mode: read_only` is requested and this enforcement cannot run, fail closed (D5).

Limitations (explicitly acknowledged):
- This only fixes escapes **into the project directory**.
- It does **not** prevent a process from touching other host paths (e.g., `/tmp`, `$HOME/other_project`, etc.).
  Full caging requires a bigger upgrade (see D7).

### D7 — Plan “full caging” as a separate hardening milestone

Status: Proposed (next sprint(s))

Decision:
- Treat “fail-closed semantics” and “full caging isolation” as separate deliverables:
  1) **Sprint A: Fail closed** (policy semantics and enforcement checks), without changing isolation mechanics.
  2) **Sprint B: Full caging** via mount-namespace + pivot_root/minimal rootfs (container-style) and/or Landlock.

Rationale:
- Fail-closed stops the most dangerous failure mode (silent downgrade) immediately.
- Full caging is inherently larger and requires careful cross-distro detection and parity between PTY/non-PTY.

### D8 — Landlock is a quality-of-life feature, not a primary guarantee

Status: Proposed

Decision:
- Landlock can be added as an additional restriction layer when available (unprivileged path-based sandboxing),
  but it should not be our only enforcement mechanism:
  - It is not enabled on all kernels/distros.
  - It has feature/version variability.
- A pragmatic plan is:
  - Implement full caging with mount namespace + pivot_root where available.
  - Add Landlock as “extra hardening” and/or as a fallback on hosts where pivot_root cannot be used.

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

### 3.1 `.substrate-profile` parsing resilience (Implemented; may be removed)

Current code includes serde defaults so minimal profiles parse without requiring all fields, plus a regression
test. If we decide to enforce a strict schema (recommended for greenfield), we can remove these defaults and
instead fail loudly with targeted diagnostics.

### 3.2 Linux: absolute-path project escape mitigation (Implemented)

Implemented the “project bind mount” enforcement:
- Non-PTY execution: run the child inside a private mount namespace and bind-mount overlay merged root onto the
  host project dir path prior to running the command.
- PTY execution: apply the same principle by wrapping the PTY child spawn in an `unshare` wrapper and performing
  the bind mount before `exec`.
- Read-only mode fail-closed if the enforcement wrapper cannot run.

### 3.3 World-deps selection layer (Not implemented yet)

The selection-driven design exists as a planning outcome, but it still needs:
- Config schema and path decision
- CLI UX (`status`, `sync`, `install`, plus `init/select`)
- Updated docs and tests

### 3.4 Runtime format unification (Not started)

Format choice still pending (YAML-only vs TOML-only).

---

## 4) Proposed UX and Config Design

### 4.1 World-deps: selection config schema (proposal)

Minimal selection file:
```yaml
version: 1
selected:
  - nvm
  - pyenv
  - bun
```

Optional: include per-tool options that *only* affect world-deps behavior (not global manager manifest):
```yaml
version: 1
selected:
  - name: nvm
    enabled: true
  - name: pyenv
    enabled: true
  - name: bun
    enabled: false
```

### 4.2 World-deps: command behavior (proposal)

- `substrate world deps status`
  - If selection config missing: print “world deps not configured” and exit 0.
  - Otherwise: show only the selected tools (or selected+present subset, depending on `--all`).
- `substrate world deps sync`
  - If selection config missing: no-op with message; exit 0.
  - Otherwise: sync only selected tools by default; `--all` overrides selection (debug/ops flag).
- `substrate world deps status --all`
  - Debug/ops mode: include canonical inventory entries even if unselected.

Note: The `--all` semantics must be explicit in help and docs to avoid conflating:
- “show the canonical inventory” vs
- “show host-missing managers” vs
- “ignore selection config”

We should pick one meaning for `--all` and introduce a second flag if needed (e.g. `--inventory`).

### 4.3 Policy: unify filesystem policy and caging knobs (proposal)

Today we have:
- `world_fs_mode` (new high-level switch)
- legacy `fs_read`/`fs_write` patterns (path allowlists)

Proposal: unify under a single `world_fs` block (breaking change acceptable):
```yaml
world_fs:
  mode: read_only         # read_only | writable
  cage: project           # project | full
  write_allowlist:        # optional; only meaningful when cage=full or for future enforcement
    - "$PROJECT/**"
    - "/tmp/**"
  read_allowlist:
    - "**"
```

Migration story:
- Prefer a single versioned schema and clear error messages (no silent fallback).
- If we change keys, provide a short migration note in release notes/docs.

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
- Bind-mount only the allowed directories (project overlay, `/usr`, `/lib*`, etc.) as needed.
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

## 7) Open Questions

1) Runtime format choice: YAML-only vs TOML-only (and the migration mechanics).
2) World-deps selection config naming: `world-deps.yaml` vs `world-deps.selection.yaml`.
3) Exact semantics for `world_fs_mode: read_only`:
   - Does it imply only “project read-only”, or should it imply “full cage + read-only”?
4) Cross-distro constraints:
   - How do we detect and communicate “unshare/mount namespace/pivot_root not allowed”?
   - What is the correct fallback ladder (pivot_root vs Landlock vs best-effort)?
5) Docs alignment:
   - `docs/VISION.md` includes examples of `fs_read/fs_write` that imply enforcement; we need to ensure the
     docs are accurate at each phase.

---

## 8) References

- `README.md` (product framing: secure execution layer + multi-agent hub)
- `docs/WORLD.md` (world architecture, routing, and world-deps UX)
- `docs/VISION.md` (roadmap and policy examples; needs alignment with enforcement guarantees)
- `docs/project_management/future/PHASE_4_5_ISOLATION_UPGRADE.md` (design draft; notes mount hardening as future)
- M5a inventory alignment spec: `docs/project_management/next/p0-platform-stability-macOS-parity/M5a-spec.md`
- M5a integration notes: `docs/project_management/next/p0-platform-stability-macOS-parity/session_log.md`
