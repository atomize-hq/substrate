# ADR Intake Sheet — provisioning_otter

1. **Codename:** `provisioning_otter`  
   **Created:** 2026-02-19T23:25:00Z  
   **Status:** ready_for_lockdown  
   **Dependencies:** []  
   **Related intakes (coordination only):** `quieting_lemur`, `clarifying_owl`, `summoning_wombat`

2. **Working Title (tentative)**

Route `install.method=apt` world-deps installs to provisioning-time (hardened-world compatible)

3. **Problem / Motivation**

- In hardened world execution paths (across platforms/backends), runtime world execution happens under restrictive sandboxing and/or read-only mounts (e.g. systemd `ProtectSystem=strict`, full-cage read-only bind mounts), so `/` is effectively read-only and `apt/dpkg` cannot mutate system paths.
- `substrate world deps current sync` currently attempts `apt-get install` for `install.method=apt` packages and can fail with “Read-only file system” for `dpkg` state/log paths.
- This behavior is also a contract mismatch with the established “system packages are provisioning-time” posture (see ADR-0002); runtime `sync/install` should not mutate OS packages even when it *would* be technically possible.
- This breaks the operator mental model: “enable deps → sync” does not reliably work for apt-backed packages under the default hardening posture.
- Operators need a supported, auditable way to install OS/system packages needed by world-deps without weakening the runtime sandbox.
- On Linux host-native backends, provisioning-time OS mutation is also a safety concern: “apt install” would mutate the workstation OS, which is typically disallowed by the threat model (even if it weren’t blocked by hardening).

4. **Proposed Outcome**

- Provide an explicit **provisioning-time** workflow to install apt/system dependencies for world-deps packages on backends where OS mutation is permitted/safe (guest worlds; future Linux guest-rootfs).
- Ensure runtime `world deps current sync|install` does not attempt OS mutation that will fail under hardening (or violate posture); instead it should guide the operator to the provisioning step (or to manual instructions when provisioning is unsupported).

5. **Non-Goals**

- Do not redesign the world-deps inventory schema.
- Do not relax hardened runtime write restrictions by default (no broadening `ReadWritePaths`, no “make / writable”).
- Do not implement arbitrary package managers (brew, yum, apk, pacman).
- Do not attempt “install node toolchains everywhere” as a bundled feature; keep this to the apt provisioning surface.
- Do not re-spec collision handling: duplicate dep names/entrypoint collisions are already part of the existing contract and should remain enforced.
- Do not add new “guest prerequisite” checks beyond the apt provisioning/workflow delta (track separately; see Candidate B).
- Do not change `world.enabled` / world-disabled UX in `substrate health` / `substrate shim doctor` (tracked separately; see related intakes).

6. **Constraints / Invariants**

- **Security:** hardened runtime execution remains effectively read-only outside Substrate-managed writable surfaces (e.g. `/var/lib/substrate/world-deps`, `/tmp`).
- **Explicitness:** OS mutation is opt-in and operator-invoked (a distinct provisioning command/surface).
- **Conceptual clarity:** `substrate world deps ...` remains “apply toolchains into the world-deps prefix”; OS/system package mutation should live behind an explicitly “world provisioning” surface (likely `substrate world enable`, which already means “prepare the world backend”).
- **Cross-platform:** runtime behavior is consistent everywhere (no implicit OS mutation during `current sync|install`); provisioning support depends on backend posture:
  - guest worlds (macOS Lima / Windows WSL): provisioning can be supported
  - Linux host-native: provisioning remains unsupported (manual guidance) unless/ until a guest-rootfs backend exists (see ADR-0009)
- **UX:** failure modes must be actionable (clear remediation steps; stable exit codes).
- **Compatibility:** preserve the current `world deps` packages/bundles contract and exit code taxonomy.

7. **Interfaces / Contracts (concrete changes)**

CLI additions/changes (locked surface; see Options for alternatives we rejected):
- Add an explicit **world provisioning** flag to install in-world OS (`apt`) requirements for apt-backed world-deps items (explicit OS mutation; operator-invoked):
  - `substrate world enable --provision-deps [--dry-run] [--verbose]`
  - Semantics (locked): derives required apt packages from the **effective enabled world-deps set** (no explicit item list in this ADR).
- Update runtime `substrate world deps current sync|install` behavior for `install.method=apt`:
  - **fail early** (before attempting `apt-get`) with a friendly, actionable error that instructs the operator to run `substrate world enable --provision-deps`.
  - On backends where provisioning is unsupported (e.g. Linux host-native), print manual guidance (and optionally point to the Linux guest-rootfs track / ADR-0009).

Likely implementation touchpoints (non-exhaustive; helps keep this slice execution-ready):
- Shell apt installer / hardening detection: `crates/shell/src/builtins/world_deps/surfaces.rs`
- Existing apt tests: `crates/shell/tests/world_deps_apt_install_wdp5.rs`
- World-agent request profile behavior (provisioning vs always-isolate): `crates/world-agent/src/service.rs` (`should_always_isolate`)
- Internals reference: `docs/internals/world/deps.md`

Docs:
- Operator guidance for script authoring and hardening constraints (already started under `docs/reference/world/deps/`).
- Add an operator-visible explanation that apt-backed packages are provisioning-time under hardening.

8. **Options**

### Option A — Extend `substrate world enable` to provision apt-backed world-deps (recommended)

**Description:** Add an opt-in provisioning flag to `substrate world enable` that installs the apt/system packages
required by the **effective enabled world-deps set**. `world deps current sync|install` becomes user-space-only and
**never** attempts apt installs; it fails early with remediation pointing to `substrate world enable --provision-deps`.
Provisioning is supported on guest-world backends where OS mutation is safe; on Linux host-native it remains unsupported
and prints manual guidance (or points to the guest-rootfs track).

**Pros**
- Matches the intended safety posture: OS mutation is explicit and separated from runtime execution.
- Conceptually clear: “enable/provision the world” is where OS packages are decided/installed.
- Reuses an already-established provisioning surface (`substrate world enable`) that operators already understand.

**Cons**
- Requires `world enable` to understand/derive requirements from world-deps inventory/enabled config (more coupling).
- If the enabled set changes later, the operator must re-run an enable/provision command (or we need a future “refresh” concept).

**Risk notes**
- Must ensure provisioning is auditable and does not silently run on behalf of agents.
- Must keep exit codes and contract wording consistent across platforms.

### Option B — Add a `substrate world deps provision` command (keep provisioning under deps)

**Description:** Introduce `substrate world deps provision` as a sibling to `current` commands. This emphasizes that
“provision” is a distinct class of operation (system package mutation), while `current` remains focused on views and
runtime application. Runtime `current sync|install` still fails early for apt items and points to `deps provision`.

**Pros**
- Conceptually clean separation: `provision` is a “special” system-mutation verb.
- Avoids expanding the already-long `current` subtree further.

**Cons**
- Two parallel namespaces (`deps current ...` and `deps provision`) can be confusing.

**Risk notes**
- Needs clear guidance on “when to run provision” vs “when to run sync”.

### Option C — Keep `current sync|install` installing apt, but run apt in a special provisioning profile

**Description:** `current sync|install` detects apt requirements and performs them via a special execution profile that
is permitted to mutate the guest OS (either by using a different world-agent service sandbox, or bypassing the sandbox).

**Pros**
- Preserves the “sync does everything” mental model for some operators.

**Cons**
- Easy to accidentally widen the runtime attack surface.
- Hard to reason about when/why OS mutation happens; may violate the explicitness goal.
- Implementation complexity (service drop-ins, alternative agent endpoints, privilege separation).

**Risk notes**
- High risk of hardening regression if not very carefully scoped.

### Option D — Declare apt unsupported at runtime; require manual guest provisioning

**Description:** Make `install.method=apt` items always `blocked` with instructions, and do not add a provisioning
command.

**Pros**
- Minimal code changes.
- No risk of widening sandbox behavior.

**Cons**
- Poor operator experience; inconsistent with “inventory + enabled + apply” ergonomics.
- Harder to standardize in teams/CI.

**Risk notes**
- Increased support load and drift across machines/guests.

9. **Recommendation (locked)**

Recommend **Option A**.

Choose Option A when:
- The world-agent runtime sandbox must remain hardened by default, and
- We want a deterministic, explicit operator workflow for OS mutation.

10. **Slice Decomposition (required)**

### ADR Candidate A (this one): provisioning-time apt deps

Behavior delta (one sentence):
- “Apt-backed world-deps are installed during `substrate world enable --provision-deps` rather than during runtime `substrate world deps current sync|install`.”

Likely slices:
1) Add world-deps provisioning surface + plumbing (guest worlds first; Linux host-native remains manual/blocked unless guest-rootfs exists).
2) Update runtime `current sync|install` to **fail early** for apt items with actionable remediation (no attempt to run apt under hardened runtime execution).

### Candidate B (follow-up): guest prerequisite checks / base-image management

- Make common runtime prerequisites first-class (e.g. `ca-certificates`, `bash`, `curl`/`wget`) via image build or a provisioning bundle.
- Optional: add a runtime preflight that can detect missing prerequisites and produce actionable remediation (without mutating OS in runtime mode).

### Candidate C (follow-up): “provision profile” execution isolation

- If ever needed, implement a safer split between runtime execution and provisioning execution inside the guest.

11. **Acceptance Criteria Draft**

- On guest-world backends (macOS Lima, Windows WSL), running `substrate world enable --provision-deps` can provision apt-backed world-deps without encountering read-only filesystem errors from `dpkg`.
- On Linux host-native, provisioning remains explicitly unsupported (no host OS mutation); Substrate prints clear manual guidance (and/or points to the Linux guest-rootfs track if configured).
- Runtime `substrate world deps current sync` does not attempt OS mutation in hardened execution mode; it prints an actionable next step.
- Script-based world-deps continue to work when they write only under `/var/lib/substrate/world-deps` (and `/tmp`).
- Exit codes remain consistent with taxonomy (`3` backend unavailable, `4` unmet prerequisites, `5` hardening conflict, etc.).

12. **Open Questions / Unknowns**

P0:
- Runtime behavior choice (preferred direction: **fail early with friendly remediation**):
  - Locked proposal: exit `4` (“unmet prerequisites/unsupported”) for `current sync|install` when encountering `install.method=apt`, since the remediation is to run an explicit provisioning step (not a hardening violation at runtime).

P1:
- (Locked) Provisioning operates on the effective enabled set (no explicit list in this ADR).
- How do we want to represent/record “provisioned” state (probe-only vs state file)?
- Should “guest prerequisites” checks (e.g. `ca-certificates`) be handled by Candidate B (separate ADR), or do we want to pull a minimal subset into Candidate A?

13. **Ready to Draft ADR? checklist**

- [x] One behavior delta sentence locked (one sentence; no extra deltas).
- [x] Command surface chosen (`substrate world enable --provision-deps` on effective enabled set).
- [x] Runtime behavior + exit code mapping locked (fail early in `current sync|install` on apt items; exit `4`; remediation points to `substrate world enable --provision-deps`).
- [x] Platform posture stated explicitly (guest worlds supported; Linux host-native manual; Linux guest-rootfs deferred to ADR-0009).
