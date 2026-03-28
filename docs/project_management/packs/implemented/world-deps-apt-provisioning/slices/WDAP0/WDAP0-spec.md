# WDAP0-spec — Provisioning-time APT requirements + provisioning wiring

## Behavior delta (single)
- Existing: `substrate world deps current sync|install` may invoke APT/dpkg at runtime for `install.method=apt` items, which can fail under hardened worlds and can violate the “no host OS mutation” posture on Linux host-native backends. `substrate world enable` provisions the world backend via a helper script and does not provide an explicit, deterministic provisioning-time APT workflow.
- New: `substrate world enable --provision-deps` derives a normalized APT requirement set from the effective enabled world-deps set, validates conflicts deterministically, probes presence read-only, and installs missing APT packages **only** inside supported guest worlds using world-agent request `profile=world-deps-provision`. Linux host-native and Windows remain fail-closed (no host OS mutation) and exit `4` for provisioning.
- Why: Make OS package mutation explicit, auditable, and compatible with hardened runtime while preserving “no host OS mutation” guard rails.

## Scope
- Add `--provision-deps` to `substrate world enable` (CLI + builtin wiring).
- Implement APT requirement derivation + normalization for the effective enabled world-deps set (DR-0001).
- Implement probe-only “already satisfied” detection (DR-0002).
- Implement guest-only APT execution using Agent API request `profile=world-deps-provision`, ignoring `SUBSTRATE_WORLD_REQUEST_PROFILE` (DR-0003).
- Define deterministic stdout/stderr invariants for `--dry-run` and `--verbose` for the provisioning workflow.
- Wire helper and installer flows so provisioning-time APT occurs before runtime `world deps current sync`, and so downstream remediation remains deterministic.
- Require at least one supported guest-backend non-dry-run validation that performs a real APT install and proves the provisioning path succeeds outside dry-run mode.

## Behavior (authoritative)
### Inputs and derivation
- Source of truth for world-deps inventory and enabled resolution: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (this slice MUST NOT redefine that schema).
- Provisioning in-scope set:
  - The effective enabled world-deps set for `cwd` (same scope as `substrate world deps current sync` without `--all`).
- APT-backed item selection:
  - Filter the in-scope set to items whose resolved `install.method=apt`.
- APT requirement extraction:
  - Concatenate each APT-backed item’s `install.apt[]` entries in the in-scope set.

### Normalization + conflict policy (DR-0001; selected Option A)
Given the extracted `install.apt[]` entries:
1) De-duplicate by `name`.
2) Stable ordering: sort by `name` (ascending, byte/ASCII order).
3) Version selection per `name`:
   - If all entries for `name` have `version` unset, the normalized entry has `version` unset.
   - If exactly one distinct non-empty `version` exists for `name`, the normalized entry uses that `version` (pins win over unpinned).
   - If two or more distinct non-empty `version` values exist for `name`, the command MUST:
     - exit `2`, and
     - print a deterministic conflict report to stderr, and
     - perform no world-agent execution (no probe, no APT/dpkg).

### APT requirement rendering (operator-visible)
When the normalized requirement set is printed (see `--dry-run` / `--verbose`):
- Each entry MUST be rendered as:
  - `name` when `version` is unset, or
  - `name=version` when `version` is set.
- Rendering MUST be in the stable normalized order (sorted by `name`), one entry per line.

### Backend capability gate (no host OS mutation)
Provisioning APT execution is permitted only on guest backends that guarantee OS mutation is guest-only.

Rules:
- macOS Lima guest backend: supported.
- Linux host-native backend: unsupported (Substrate MUST NOT mutate the host OS via APT/dpkg).
- Windows: unsupported for this feature.

Unsupported behavior (applies to both `--dry-run` and non-dry-run):
- The command MUST exit `4`.
- Stderr MUST include:
  - the exact remediation command `substrate world enable --provision-deps`, and
  - a platform-specific statement:
    - Linux: includes the exact phrase `Substrate will not mutate the host OS`.
    - Windows: includes the exact phrase `unsupported on Windows`.

### `--dry-run` (provisioning preview; no mutation)
When invoked as `substrate world enable --provision-deps --dry-run`:
- The command MUST:
  - derive and normalize the APT requirement set, and
  - print it to stdout using the APT requirement rendering rules above.
- The command MUST NOT:
  - run the world enable helper script,
  - connect to world-agent, or
  - execute any in-world commands (no probe, no APT/dpkg).
- Exit codes:
  - `0` on supported platforms/backends when derivation succeeds (including empty requirement set),
  - `2` on DR-0001 conflict,
  - `4` on unsupported platforms/backends.

### Provisioning execution (non-dry-run)
When invoked as `substrate world enable --provision-deps` (without `--dry-run`):

#### Ordering (two-phase sequence)
1) World-backend enable (baseline `substrate world enable` behavior).
2) APT provisioning for the in-scope set (this slice).

The APT provisioning phase MUST run only after world-backend enable succeeds.

#### Probe-only satisfied/no-op semantics (DR-0002; selected Option A)
- If the normalized APT requirement set is empty, the APT provisioning phase MUST be a no-op and MUST exit `0`.
- Otherwise, the command MUST perform a read-only presence probe inside the world:
  - Unpinned requirement `name` is satisfied iff `dpkg-query` reports `install ok installed`.
  - Pinned requirement `name=version` is satisfied iff `dpkg-query` reports `install ok installed` AND the installed version equals `version` (exact match).
- If all requirements are satisfied, the APT provisioning phase MUST be a no-op and MUST exit `0` without invoking `apt-get` or `dpkg`.

#### APT install execution (guest-only; DR-0003)
If any requirement is unsatisfied on a supported backend:
- The command MUST execute APT provisioning inside the world using the Agent API `/v1/execute` request field:
  - `profile: "world-deps-provision"`
- The provisioning request profile MUST be set explicitly and MUST NOT be derived from `SUBSTRATE_WORLD_REQUEST_PROFILE`.
- The command MUST execute APT using the normalized APT requirement rendering values in stable order.
- If APT execution fails:
  - exit `5` when the in-world command exits `5` (safety/protected-path violation),
  - otherwise exit `4` and include a stderr snippet from the in-world failure output.

Validation requirement:
- WDAP0 validation MUST include at least one supported guest-backend non-dry-run run with a real
  APT-backed dep whose package is absent before the run and installed after the run. Dry-run-only
  validation is insufficient for this slice.

#### Dependency-unavailable handling
If world-agent connectivity is required (probe or install) and cannot be established:
- The command MUST exit `3` and emit an actionable stderr message indicating the world backend is unavailable.

### `--verbose`
When `--verbose` is present with `--provision-deps` (dry-run or non-dry-run), stdout MUST include:
- the selected provisioning request profile value (`world-deps-provision`), and
- the derived normalized APT requirement set (same content and ordering as `--dry-run`).

### Helper and installer wiring
When invoked as `substrate world enable --provision-deps` (without `--dry-run`):
- `scripts/substrate/world-enable.sh` MUST be invoked with `--no-sync-deps`.
- The helper MUST emit the exact line `Skipping world deps sync (--no-sync-deps)` when `--no-sync-deps` is present.
- `substrate world deps current sync` MUST run only after provisioning-time APT completes successfully or is a no-op by contract.

When invoked as `substrate world enable --provision-deps --dry-run`:
- no helper script is executed, and
- no `substrate world deps current sync` execution occurs.

Installer rule:
- When `scripts/substrate/install-substrate.sh --sync-deps` observes downstream `substrate world deps current sync` exit `4`, stderr MUST include the exact remediation command:

  ```text
  substrate world enable --provision-deps
  ```

- The installer MUST still exit `0` after emitting the remediation note.

## Acceptance criteria
- AC-WDAP0-01: `substrate world enable --provision-deps --dry-run` prints the normalized APT requirement set to stdout with one entry per line rendered as `name` or `name=version`, in sorted-by-name order, and performs no helper, world-agent, APT, or mutating `dpkg` execution.
- AC-WDAP0-02: If two enabled items require the same APT `name` with two distinct non-empty `version` pins, `substrate world enable --provision-deps` exits `2`, prints a deterministic conflict report to stderr, and performs no helper or world-agent execution.
- AC-WDAP0-03: On Linux host-native and Windows, `substrate world enable --provision-deps` exits `4`; Linux stderr includes the exact phrase `Substrate will not mutate the host OS`, and Windows stderr includes the exact phrase `unsupported on Windows`.
- AC-WDAP0-04: On supported guest backends, provisioning probe and install requests use Agent API `profile=world-deps-provision` even when `SUBSTRATE_WORLD_REQUEST_PROFILE` is set to a different value.
- AC-WDAP0-05: When the normalized APT requirement set is empty, `substrate world enable --provision-deps` exits `0` and the APT provisioning phase is a no-op.
- AC-WDAP0-06: When all normalized APT requirements are already satisfied, `substrate world enable --provision-deps` exits `0` without invoking `apt-get` or mutating `dpkg`.
- AC-WDAP0-07: With `substrate world enable --provision-deps` (non-dry-run), `scripts/substrate/world-enable.sh` is invoked with `--no-sync-deps`, the helper emits `Skipping world deps sync (--no-sync-deps)`, runtime `substrate world deps current sync` runs only after provisioning-time APT completes, and WDAP0 validation includes at least one supported guest-backend run (macOS Lima minimum for this pack) where a real APT-backed dep is absent before the run and installed afterward.
- AC-WDAP0-08: If world-agent connectivity is required for probe or install and cannot be established, `substrate world enable --provision-deps` exits `3` with actionable stderr; when `scripts/substrate/install-substrate.sh --sync-deps` observes downstream exit `4`, it prints remediation containing `substrate world enable --provision-deps` and still exits `0`.

## Out of scope
- Runtime fail-early behavior for `substrate world deps current sync|install` (owned by `WDAP1`).
- Planning-pack task wiring and checkpoint scaffolding (`tasks.json`, kickoff prompts, quality gate report).
