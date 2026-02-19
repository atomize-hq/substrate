# ADR Intake Sheet — provisioning_otter

1. **Codename:** `provisioning_otter`  
   **Created:** 2026-02-19  
   **Status:** brainstorming

2. **Working Title (tentative)**

Route `install.method=apt` world-deps installs to provisioning-time (hardened-world compatible)

3. **Problem / Motivation**

- In hardened worlds (macOS Lima, Windows WSL), runtime world execution happens under a restrictive sandbox (e.g. `ProtectSystem=strict`), so `/` is effectively read-only and `apt/dpkg` cannot mutate system paths.
- `substrate world deps current sync` currently attempts `apt-get install` for `install.method=apt` packages and fails with “Read-only file system” for `dpkg` state/log paths.
- This breaks the operator mental model: “enable deps → sync” does not reliably work for apt-backed packages under the default hardening posture.
- Operators need a supported, auditable way to install OS/system packages needed by world-deps without weakening the runtime sandbox.
- Even for script installs, runtime failures can be confusing when basic prerequisites are missing in the guest image (example: `ca-certificates` for HTTPS downloads).

4. **Proposed Outcome**

- Provide an explicit **provisioning-time** workflow to install apt/system dependencies for world-deps packages in supported guests (Lima/WSL).
- Ensure runtime `world deps current sync|install` does not attempt OS mutation that will fail under hardening; instead it should guide the operator to the provisioning step.

5. **Non-Goals**

- Do not redesign the world-deps inventory schema.
- Do not relax hardened runtime write restrictions by default (no broadening `ReadWritePaths`, no “make / writable”).
- Do not implement arbitrary package managers (brew, yum, apk, pacman).
- Do not attempt “install node toolchains everywhere” as a bundled feature; keep this to the apt provisioning surface.

6. **Constraints / Invariants**

- **Security:** hardened runtime execution remains effectively read-only outside Substrate-managed writable surfaces (e.g. `/var/lib/substrate/world-deps`, `/tmp`).
- **Explicitness:** OS mutation is opt-in and operator-invoked (a distinct provisioning command/surface).
- **Cross-platform:** provisioning is supported only where we can do it safely and deterministically (macOS Lima + Windows WSL guests); Linux host backend may remain “manual” depending on existing posture.
- **UX:** failure modes must be actionable (clear remediation steps; stable exit codes).
- **Compatibility:** preserve the current `world deps` packages/bundles contract and exit code taxonomy.

7. **Interfaces / Contracts (concrete changes)**

Tentative CLI additions/changes (exact naming TBD; see Options):
- Add a provisioning command to install `apt` requirements for world-deps items (likely for the effective enabled set).
- Update runtime `substrate world deps current sync|install` behavior for `install.method=apt`:
  - either: block with a clear message directing the operator to provisioning, or
  - or: only attempt apt in a provisioning context (if available).

Runtime preflight checks (small, operator-facing):
- Before running script installers that use HTTPS, ensure guest prerequisites are present (at least detect missing `ca-certificates` and present remediation; optionally include it as a provisioned dependency).

Docs:
- Operator guidance for script authoring and hardening constraints (already started under `docs/reference/world/deps/`).
- Add an operator-visible explanation that apt-backed packages are provisioning-time under hardening.

8. **Options**

### Option A — Add a dedicated `world deps provision` command (guest provisioning path)

**Description:** Introduce a provisioning command that runs in a provisioning context (outside the hardened runtime
execution sandbox) to install apt packages required by enabled world-deps items. Runtime `current sync|install` never
mutates OS packages; it only performs prefix installs (scripts + wrappers) and status/probes.

**Pros**
- Matches the intended safety posture: OS mutation is explicit and separated from runtime execution.
- Aligns with the “hardened worlds” model (no need to relax runtime sandbox).
- Clear operator story: “provision system deps once; sync does user-space installs and wrappers”.

**Cons**
- Requires a new/returning CLI surface and documentation.
- Requires platform-specific implementation for Lima and WSL provisioning mechanics.

**Risk notes**
- Must ensure provisioning is auditable and does not silently run on behalf of agents.
- Must keep exit codes and contract wording consistent across platforms.

### Option B — Keep `current sync|install` installing apt, but run apt in a special provisioning profile

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

### Option C — Declare apt unsupported at runtime; require manual guest provisioning

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

9. **Recommendation (tentative)**

Recommend **Option A**.

Choose Option A when:
- The world-agent runtime sandbox must remain hardened by default, and
- We want a deterministic, explicit operator workflow for OS mutation.

10. **Slice Decomposition (required)**

### ADR Candidate A (this one): provisioning-time apt deps

Behavior delta (one sentence):
- “Apt-backed world-deps are installed via an explicit provisioning command rather than during runtime `current sync|install`.”

Likely slices:
1) Add provisioning command surface + plumbing (macOS Lima + Windows WSL).
2) Update runtime `current sync|install` to block or skip apt with actionable remediation.
3) Add basic prerequisite checks (`ca-certificates`) and remediation wording.

### Candidate B (follow-up): guest base-image prerequisite management

- Make common runtime prerequisites first-class (bash/curl/wget/ca-certificates) via image build or provisioning bundle.

### Candidate C (follow-up): “provision profile” execution isolation

- If ever needed, implement a safer split between runtime execution and provisioning execution inside the guest.

11. **Acceptance Criteria Draft**

- On macOS (Lima), an operator can provision apt-backed world-deps without encountering read-only filesystem errors from `dpkg`.
- On Windows (WSL), the same provisioning workflow exists (or returns a clear unsupported message if not available).
- Runtime `substrate world deps current sync` does not attempt OS mutation in hardened execution mode; it prints an actionable next step.
- Script-based world-deps continue to work when they write only under `/var/lib/substrate/world-deps` (and `/tmp`).
- When `ca-certificates` is missing in the guest and a script requires HTTPS, Substrate surfaces an actionable remediation (and/or makes it provisionable).
- Exit codes remain consistent with taxonomy (`3` backend unavailable, `4` unmet prerequisites, `5` hardening conflict, etc.).

12. **Open Questions / Unknowns**

P0:
- What is the exact CLI surface name? (`substrate world deps provision` vs `substrate world deps current provision` vs `substrate world provision deps`).
- Should runtime `current sync|install` hard-fail on apt items (exit `4`/`5`) or skip them and report `blocked`?
- What are the minimum “guest prerequisites” we should check/provision (at least `ca-certificates`; also `bash`, `curl`/`wget`)?

P1:
- Should provisioning operate on the effective enabled set, a requested list, or both?
- How do we want to represent/record “provisioned” state (probe-only vs state file)?

13. **Ready to Draft ADR? checklist**

- **No** — still missing decisions on CLI naming and the exact runtime behavior when apt items are enabled.
- **No** — need clarity on the minimum prerequisite list and where those checks live (provision vs runtime preflight).

