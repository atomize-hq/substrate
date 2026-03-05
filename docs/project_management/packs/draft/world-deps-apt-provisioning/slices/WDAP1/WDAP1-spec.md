# WDAP1-spec — Runtime fail-early + remediation for APT-backed items

## Behavior delta (single)
- Existing: `substrate world deps current sync|install` can invoke APT/dpkg at runtime for `install.method=apt` items, which fails under hardened worlds and risks violating the “no host OS mutation” posture on Linux host-native backends.
- New: `substrate world deps current sync|install` never invokes APT/dpkg. When in-scope APT-backed items are present, Substrate derives the normalized APT requirement set, probes satisfaction read-only, and exits `4` with deterministic remediation when any requirement is unsatisfied.
- Why: keep OS package mutation provisioning-time only (`substrate world enable --provision-deps`) while making runtime behavior deterministic and fail-closed.

## Scope
- Runtime preflight + fail-early for:
  - `substrate world deps current sync [--dry-run] [--verbose] [--all]`
  - `substrate world deps current install <ITEM...> [--dry-run] [--verbose]`
- Deterministic definitions for:
  - “APT in-scope” rules (`sync` enabled-set vs `install` explicit args)
  - normalized APT requirement set derivation (DR-0001; shared with `WDAP0`)
  - probe-only satisfied detection (DR-0002)
  - remediation text invariants (must include `substrate world enable --provision-deps`)
  - `--dry-run` / `--verbose` behavior under fail-early
- Required operator-doc update targets (exact paths + headings) that must land to remove “runtime APT” contradictions.

## Behavior (authoritative)

### Invariant: no runtime APT/dpkg
- Runtime `deps current sync|install` MUST NOT execute `apt`, `apt-get`, or mutating `dpkg` operations.
- The only APT-family interaction permitted at runtime is the read-only presence probe defined in this spec.

### In-scope set selection (APT in-scope rule)
Source of truth for inventory/enabled resolution and bundle expansion:
`docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (this slice MUST NOT redefine that schema).

Rules:
- `deps current sync`:
  - Without `--all`, the in-scope set is the effective enabled world-deps set for `cwd`.
  - With `--all`, the in-scope set is all visible world-deps items from inventory.
- `deps current install <ITEM...>`:
  - Bundle expansion MUST occur for `<ITEM...>` per the upstream contract.
  - The in-scope set is the expanded `<ITEM...>` only; the effective enabled set MUST NOT be added implicitly.

### Normalized APT requirement set (DR-0001; shared algorithm with `WDAP0`)
Given the in-scope set:
1) Select APT-backed items: items whose resolved `install.method=apt`.
2) Extract requirements: concatenate each APT-backed item’s `install.apt[]` entries.
3) Normalize per DR-0001 Option A:
   - De-duplicate by `name`.
   - Stable ordering: sort by `name` (ascending, byte/ASCII order).
   - Version selection per `name`:
     - If all entries for `name` have `version` unset, the normalized entry has `version` unset.
     - If exactly one distinct non-empty `version` exists for `name`, the normalized entry uses that `version` (pins win over unpinned).
     - If two or more distinct non-empty `version` values exist for `name`, the command MUST:
       - exit `2`,
       - print a deterministic conflict report to stderr, and
       - perform no world-agent execution and no non-APT installs.

APT requirement rendering (when printed):
- Render each normalized entry as `name` (unpinned) or `name=version` (pinned).
- Print in normalized order, one entry per line.

### Read-only presence probe (DR-0002; probe-only)
When the normalized APT requirement set is non-empty, Substrate MUST perform a read-only presence probe in the selected world:
- Probe command: `dpkg-query` (read-only).
- Satisfaction rules:
  - Unpinned requirement `name` is satisfied iff `dpkg-query` reports `install ok installed` for `name`.
  - Pinned requirement `name=version` is satisfied iff `dpkg-query` reports `install ok installed` AND the installed version equals `version` (exact match).
- If `dpkg-query` cannot be executed inside the world, the requirement is unsatisfied (fail closed).
- If world-agent connectivity is required to run the probe and cannot be established, the command MUST exit `3` with actionable stderr.

### Fail-early rule (runtime)
For `deps current sync|install`:
1) Derive the normalized APT requirement set (DR-0001).
2) If the normalized APT requirement set is empty:
   - the command proceeds with non-APT installs per the upstream world-deps contract.
3) If the normalized APT requirement set is non-empty:
   - run the read-only presence probe (DR-0002),
   - if any requirement is unsatisfied:
     - exit `4`,
     - emit remediation to stderr, and
     - perform no non-APT installs (fail early before mutation).
   - if all requirements are satisfied:
     - treat APT-backed items as satisfied/no-op, and
     - proceed with non-APT installs per the upstream world-deps contract.

### Remediation output invariants (exit `4`)
When exiting `4` due to unsatisfied APT requirements, stderr MUST include:
- the exact remediation command:

  ```text
  substrate world enable --provision-deps
  ```

- platform/backends guidance:
  - Linux host-native: includes the exact phrase `Substrate will not mutate the host OS`.
  - Windows: includes the exact phrase `unsupported on Windows`.

### `--dry-run` under fail-early (follow-up #5)
When `--dry-run` is present:
- The command MUST perform no mutation (no non-APT installs and no APT/dpkg execution).
- The command MUST still apply the fail-early rule and MUST exit `4` when any required APT package is unsatisfied.
- When the normalized APT requirement set is non-empty, stdout MUST include the normalized APT requirement rendering (one per line, stable order), even when the command exits `4`.

### `--verbose` under fail-early (follow-up #5)
When `--verbose` is present and the command exits `4` due to unsatisfied APT requirements, stderr MUST include:
- the normalized APT requirement rendering (one per line, stable order).

## Operator-doc update targets (required; exact paths + headings)
These docs MUST be updated to reflect “APT is provisioning-time”, MUST describe the runtime fail-early posture, and MUST link to:
`docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
instead of restating contract tables.

- `docs/reference/world/deps/README.md` (headings: `## APT packages (current limitation in hardened worlds)`, `## Commands you will use`)
- `docs/internals/world/deps.md` (headings: `## High-level flow`, `## APT installs vs hardening`)
- `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (headings: `#### substrate world deps current install <item_name...>`, `#### substrate world deps current sync [--dry-run] ...`)
- `docs/WORLD.md` (heading: `## 5) Agent API (over UDS)` → `POST /v1/execute` request body fields, when the provisioning posture relies on request `profile`)
- `docs/CONFIGURATION.md` (heading: `SUBSTRATE_WORLD_REQUEST_PROFILE` row; state it is not the operator-facing provisioning workflow)
- `docs/COMMANDS.md` (heading: `### world Subcommand` row for `substrate world enable` flags)

## Acceptance criteria
- AC-WDAP1-01: `substrate world deps current sync` with an in-scope APT-backed item and an unsatisfied APT requirement exits `4`, emits remediation to stderr that includes `substrate world enable --provision-deps`, and does not execute `apt`, `apt-get`, or mutating `dpkg`.
- AC-WDAP1-02: `substrate world deps current install <ITEM...>` with an explicit APT-backed item and an unsatisfied APT requirement exits `4`, emits remediation to stderr that includes `substrate world enable --provision-deps`, and does not execute `apt`, `apt-get`, or mutating `dpkg`.
- AC-WDAP1-03: `substrate world deps current install <ITEM...>` does not include enabled items implicitly: if the effective enabled set contains an APT-backed item, but `<ITEM...>` contains only non-APT items, the APT fail-early posture does not trigger.
- AC-WDAP1-04: `substrate world deps current sync --all` applies the fail-early posture to any visible in-scope APT-backed items (not only the effective enabled set).
- AC-WDAP1-05: With `--dry-run`, `deps current sync|install` performs no mutation, still exits `4` when APT requirements are unsatisfied, and prints the normalized APT requirement rendering to stdout (stable order; `name` or `name=version`).
- AC-WDAP1-06: With `--verbose` and exit `4` due to unsatisfied APT requirements, stderr includes the normalized APT requirement rendering (stable order; `name` or `name=version`).
- AC-WDAP1-07: On Linux host-native, fail-early remediation includes the exact phrase `Substrate will not mutate the host OS`.
- AC-WDAP1-08: On Windows, fail-early remediation includes the exact phrase `unsupported on Windows`.
- AC-WDAP1-09: If normalized requirement derivation encounters a version-pin conflict (DR-0001), `deps current sync|install` exits `2`, prints a deterministic conflict report to stderr, and performs no world-agent execution and no non-APT installs.
- AC-WDAP1-10: If world-agent connectivity is required for the read-only presence probe and cannot be established, `deps current sync|install` exits `3` with actionable stderr.

## Out of scope
- Provisioning-time APT execution (`substrate world enable --provision-deps`) details (owned by `WDAP0`).
- Helper/installer wiring and ordering changes (owned by `WDAP2`).
- Operator-doc updates (owned by `WDAP3`), except for enumerating the required targets in this spec.
