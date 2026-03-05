# WDAP3-spec — Docs reconciliation for provisioning-time APT + runtime fail-early

## Behavior delta (single)
- Existing: operator docs and upstream contracts still describe runtime APT/dpkg (“APT first” / “world image installs first (apt)”), and Agent API docs omit the request `profile` field. This creates contradictions with the new contract that runtime `world deps` must not mutate OS packages.
- New: the required doc surfaces are updated so there is a single coherent story: runtime `substrate world deps current sync|install` never runs APT/dpkg; APT system packages are provisioned only via `substrate world enable --provision-deps` on supported guest backends; docs link to this pack’s `contract.md` instead of duplicating tables.
- Why: make the provisioning-time APT posture discoverable, remove “runtime APT” contradictions, and keep contract surfaces consistent across operator reference, internals, and upstream docs.

## Scope
- Update the operator-doc and internal-doc targets listed in `pre-planning/impact_map.md` to reflect the authoritative behavior in:
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
  - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
- Reconcile the upstream world-deps contract doc sections that currently mandate runtime APT/dpkg.
- Document the Agent API request `profile` field and clarify the operator workflow vs advanced/testing knobs.
- Validate that the updated docs do not restate this pack’s contract tables and do not contain “runtime APT” claims.

## Behavior (authoritative)

### Single-authority rule (no duplicated tables)
For the operator-doc update targets in this spec:
- They MUST link to `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`.
- They MUST NOT restate the contract’s platform/backends support matrix, exit-code mapping tables, or DR decision tables.
- They MAY summarize the behavior in prose, but MUST keep the contract single-sourced (links, not copy/paste).

### Operator doc: `docs/reference/world/deps/README.md`
Update the following headings:

#### `## APT packages (current limitation in hardened worlds)`
This section MUST:
- State the runtime invariant: `substrate world deps current sync|install` never runs `apt`, `apt-get`, or mutating `dpkg`.
- State the provisioning workflow (supported guest backends only):
  - The operator entrypoint is `substrate world enable --provision-deps [--dry-run] [--verbose]`.
- State the fail-early posture (runtime):
  - If an in-scope APT-backed item’s APT requirements are unsatisfied, runtime exits `4` and prints remediation containing the exact command:

    ```text
    substrate world enable --provision-deps
    ```

- Include backend guidance without duplicating tables:
  - Linux host-native: include the exact phrase `Substrate will not mutate the host OS`.
  - Windows: include the exact phrase `unsupported on Windows`.
- Link to this pack’s `contract.md` for the authoritative contract.

#### `## Commands you will use`
This section MUST include `substrate world enable --provision-deps` as the explicit provisioning-time APT workflow (with `--dry-run` and `--verbose` noted as applicable), and MUST link to this pack’s `contract.md` for details.

### Internal doc: `docs/internals/world/deps.md`
Update the following headings:

#### `## High-level flow`
Replace any “runtime APT first” language with a flow consistent with `contract.md` + `WDAP1`:
- Resolve inventory + enabled set (unchanged).
- Determine the in-scope set and the subset of APT-backed items (`install.method=apt`).
- If the normalized APT requirement set is non-empty:
  - Perform the read-only presence probe (`dpkg-query`) inside the world.
  - If any requirement is unsatisfied, fail early (exit `4`) with remediation containing `substrate world enable --provision-deps`, and perform no other installs.
- If APT requirements are satisfied (or none exist), proceed with non-APT installs (scripts/wrappers) per the upstream world-deps contract.
- State that APT/dpkg OS mutation occurs only during `substrate world enable --provision-deps` on supported guest backends (per `WDAP0`).

The updated section MUST NOT contain the exact string `apt packages first (world image / OS mutation)`.

#### `## APT installs vs hardening`
This section MUST:
- Explain that hardening makes runtime APT/dpkg mutation incompatible with guest service sandboxing, and therefore runtime APT is prohibited.
- Explain that the supported APT mutation workflow is provisioning-time only (`substrate world enable --provision-deps`) and uses the provisioning execution posture selected in DR-0003 (Agent API request `profile=world-deps-provision`).
- Link to this pack’s `contract.md` (operator-facing) and `decision_register.md` (DR-0003 rationale) rather than re-stating their tables.

### Upstream contract reconciliation: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
Update the following headings to remove “runtime APT” contradictions and align to this pack’s `contract.md` and `WDAP1`:

#### `#### substrate world deps current install <item_name...> [--dry-run] [--verbose]`
This section MUST:
- Remove the requirement `Apply **world image** installs first (apt).`
- Replace it with a deterministic statement that runtime does not run APT/dpkg:
  - When the explicit `<item_name...>` set (after bundle expansion) includes APT-backed items, the command derives the normalized APT requirement set, probes satisfaction read-only, and fails early with exit `4` when any requirement is unsatisfied.
  - When APT requirements are satisfied, APT-backed items are treated as satisfied/no-op and the command proceeds with non-APT installs only.
- Update `--dry-run` so it no longer guarantees exit `0`:
  - It MUST perform no mutation.
  - It MUST still apply the APT fail-early rule and MUST exit `4` when APT requirements are unsatisfied.
  - When the normalized APT requirement set is non-empty, stdout MUST include the normalized APT requirement rendering (one per line, stable order).
- Ensure remediation wording references `substrate world enable --provision-deps` (exact string) and links to this pack’s `contract.md` rather than duplicating exit-code/platform tables.

#### `#### substrate world deps current sync [--dry-run] [--verbose] [--all]`
This section MUST:
- Explicitly state that APT-backed items do not trigger runtime APT/dpkg mutation.
- State that the same APT fail-early posture applies to the `sync` in-scope set (effective enabled set, or all visible items under `--all`).
- Ensure remediation links to `substrate world enable --provision-deps` and this pack’s `contract.md`.

### Agent API schema doc: `docs/WORLD.md`
Under `## 5) Agent API (over UDS)` → `POST /v1/execute`:
- The documented request body MUST include an optional `profile` field (string) alongside `cmd`, `cwd`, `env`, and other existing fields.
- The doc MUST state (briefly) that `profile` selects an execution isolation profile, and that provisioning-time APT uses an explicit profile (`world-deps-provision`) as part of the operator workflow in this pack.

### Env var registry doc: `docs/CONFIGURATION.md`
In the `SUBSTRATE_WORLD_REQUEST_PROFILE` row:
- The text MUST explicitly state this is advanced/testing and is not the operator-facing provisioning workflow for this feature.
- The text MUST direct operators to `substrate world enable --provision-deps` and MUST state that provisioning uses an explicit request `profile=world-deps-provision` and ignores `SUBSTRATE_WORLD_REQUEST_PROFILE` for provisioning executions.
- The row MUST link to this pack’s `contract.md` rather than duplicating its tables.

### Command reference doc: `docs/COMMANDS.md`
In `### world Subcommand` row for `substrate world enable`:
- The flags list MUST include `--provision-deps`.
- Notes MUST NOT imply that operators must use `--profile` or `SUBSTRATE_WORLD_REQUEST_PROFILE` to activate provisioning-time APT; the operator entrypoint is `--provision-deps` (per this pack’s `contract.md`).

## Acceptance criteria
- AC-WDAP3-01: `docs/reference/world/deps/README.md` heading `## APT packages (current limitation in hardened worlds)` states the “no runtime APT/dpkg” invariant and includes the exact remediation command `substrate world enable --provision-deps`.
- AC-WDAP3-02: `docs/reference/world/deps/README.md` heading `## Commands you will use` includes `substrate world enable --provision-deps` and links to `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`.
- AC-WDAP3-03: `docs/internals/world/deps.md` heading `## High-level flow` no longer contains `apt packages first (world image / OS mutation)` and describes the runtime fail-early + read-only probe posture for APT-backed items.
- AC-WDAP3-04: `docs/internals/world/deps.md` heading `## APT installs vs hardening` states runtime APT is prohibited and identifies provisioning-time APT (`world enable --provision-deps`) as the only supported APT mutation workflow, referencing DR-0003 for the request `profile` posture.
- AC-WDAP3-05: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` heading `#### substrate world deps current install ...` no longer mandates runtime APT and defines exit `4` fail-early semantics plus remediation containing `substrate world enable --provision-deps`.
- AC-WDAP3-06: `docs/WORLD.md` `POST /v1/execute` request body documents optional `profile` and notes that provisioning-time APT uses `world-deps-provision`.
- AC-WDAP3-07: `docs/CONFIGURATION.md` `SUBSTRATE_WORLD_REQUEST_PROFILE` row states it is advanced/testing, not the operator workflow, and states provisioning ignores it.
- AC-WDAP3-08: `docs/COMMANDS.md` lists `--provision-deps` for `substrate world enable` and does not direct operators to `--profile`/`SUBSTRATE_WORLD_REQUEST_PROFILE` to perform provisioning.

## Out of scope
- Any code changes (CLI, shell, world-agent, scripts) required to implement `WDAP0`, `WDAP1`, or `WDAP2`.
- Doc updates outside the explicit targets listed in this spec (unless needed to remove an otherwise unresolvable “runtime APT” contradiction introduced by those targets).
- ADR edits (`ADR-0030-provisioning-otter.md`) and pack-wide planning artifacts (`plan.md`, `quality_gate_report.md`, `tasks.json`) owned by other PWS roles.
