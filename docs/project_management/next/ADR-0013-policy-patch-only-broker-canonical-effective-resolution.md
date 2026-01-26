# ADR-0013 — Policy Patch-only Format + Broker-Canonical Effective Resolution

## Status
- Status: Approved
- Date (UTC): 2026-01-17
- Owner(s): Shell/Broker maintainers

## Scope
- Feature directories (impacted):
  - `docs/project_management/next/` (this ADR; cross-cutting contract)
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/` (planning pack + decision register)
  - `docs/project_management/_archived/workspace-config-policy-unification/` (ADR-0008 baseline contract this work enforces)
- Sequencing spine: `docs/project_management/next/sequencing.json`
- Standards:
  - `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Baseline scope/file contract:
  - `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Provenance + per-key merge strategy semantics (add-on to ADR-0008):
  - `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
- Decision Register (this body of work):
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/decision_register.md`
- Current (stale) operator docs that must be updated as part of this work:
  - `docs/CONFIGURATION.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: a49f73cdb2f8536432dcc9695f5ab84bbeeaebebb1076e1784a38da10a9236b9
Run `make adr-fix ADR=docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md` after drafting.

### Changes (operator-facing)
- Policy resolution becomes consistent across CLI + runtime execution
  - Existing: `substrate policy current show` can succeed with patch-form `policy.yaml`, while interactive runs/shims/world-agent can fail or silently fall back when the broker parses the same file as a full strict document.
  - New: `policy.yaml` is patch-only everywhere; the broker resolves the effective policy via patch merge (defaults → global patch → workspace patch), and all execution surfaces consume that single effective policy.
  - Why: prevents “valid effective policy shown, but execution disagrees” drift and eliminates silent fallbacks that undermine isolation expectations.
  - Links:
    - `crates/broker/src/policy_loader.rs#L17`
    - `crates/broker/src/policy.rs#L401`
    - `crates/shell/src/execution/policy_model.rs#L243`
    - `crates/shell/src/execution/routing/dispatch/exec.rs#L111`
    - `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`
    - `docs/CONFIGURATION.md#L174`

- Workspace disable marker is honored by the broker
  - Existing: the shell respects `.substrate/workspace.disabled`, but the broker’s workspace discovery does not, causing policy resolution to re-enable “disabled” workspaces for execution paths that depend on the broker.
  - New: broker workspace discovery matches the shell contract: a disabled workspace is treated as non-existent for policy discovery and effective resolution.
  - Why: ensures a single, predictable switch for operators to remove workspace-local policy impact without deleting files.
  - Links:
    - `crates/broker/src/policy_loader.rs#L39`
    - `crates/broker/src/profile.rs#L37`
    - `crates/shell/src/execution/workspace.rs#L5`

## Problem / Context
- Substrate currently has two incompatible on-disk interpretations for the same filenames (`policy.yaml`):
  - The shell’s policy CLI treats `policy.yaml` as a sparse patch and merges layers.
  - The broker (and all paths that call `substrate_broker::detect_profile`) treats `policy.yaml` as a full strict document and does “first match wins”.
- This causes correctness drift:
  - `substrate policy current show` can report a valid effective policy while execution paths fail to parse the same file or fall back to defaults.
  - Workspace-local patch files that intentionally omit most keys (inheritance) can break interactive execution with “missing field mode”.
- ADR-0008 establishes patch-only policy/config scope semantics; this ADR is the missing consolidation that makes the broker the canonical resolver and removes legacy “full policy” behavior from runtime codepaths and operator docs.

## Goals
- Make `policy.yaml` patch-only across the entire repo (code + docs + tests).
- Make the broker the canonical effective-policy resolver used by shell/shim/world-agent execution flows.
- Define a single effective resolution rule for policy:
  - Effective policy = defaults overlaid by global policy patch overlaid by workspace policy patch.
- Ensure broker workspace discovery honors `.substrate/workspace.disabled` exactly as in the shell contract.
- Eliminate silent fallback behavior on policy resolution errors for all runtime execution paths.

## Non-Goals
- Supporting any “full policy document” format at any location (including legacy `.substrate-profile*` surfaces).
- Supporting “first match wins” policy resolution semantics.
- Adding migration tooling, format auto-detection, or backwards compatibility shims for legacy policy files.
- Expanding policy schema or changing policy enforcement semantics beyond resolution/discovery consistency.

## User Contract (Authoritative)

### CLI
- Commands:
  - `substrate policy current show`:
    - MUST reflect the exact effective policy that will be used for subsequent execution in the same cwd (same broker resolution inputs, same merge semantics).
    - MUST succeed for valid patch-form policy files that omit inherited keys.
    - `--explain` MUST follow ADR-0012’s provenance output contract (at minimum: per-key `merge_strategy` + `sources`).
  - `substrate policy global show` / `substrate policy workspace show`:
    - MUST render the on-disk patch file for that scope (not the effective merged policy).
    - MUST treat missing patch files as “empty patch” by contract (equivalent to `{}`), without creating files as a side effect.
- Exit codes:
  - Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
  - `0`: command succeeds (including “no policy file present; defaults apply” views)
  - `2`: invalid YAML, invalid schema/type, unknown keys, or unreadable patch file for the requested scope
  - `5`: policy deny decisions for command execution (as enforced by the broker/shim), not policy-parse failures

### Config
- Files and locations (precedence for effective policy):
  1. Workspace policy patch: `<workspace_root>/.substrate/policy.yaml` (when a workspace exists and is enabled)
  2. Global policy patch: `$SUBSTRATE_HOME/policy.yaml`
  3. Built-in defaults
- Workspace discovery:
  - Workspace root is identified by `<workspace_root>/.substrate/workspace.yaml`.
  - If `<workspace_root>/.substrate/workspace.disabled` exists, that workspace is treated as non-existent for policy discovery and effective resolution.
- Schema and format:
  - `policy.yaml` files are patch-only: a YAML mapping where omitted keys mean “inherit”.
  - Unknown keys are a hard error (no best-effort tolerance).
  - The repo MUST NOT accept, parse, or write any full-policy document format as a supported on-disk contract.

### Platform guarantees
- Linux/macOS/Windows:
  - The same patch-only policy resolution semantics apply to shell execution, shim execution, and world-agent execution paths.
  - Policy resolution errors are not ignored on any platform-specific path that relies on the broker for execution decisions.

## Architecture Shape
- Components:
  - `crates/broker`:
    - Owns policy patch schema, patch parsing, patch merge, and policy invariant validation for effective policy.
    - Exposes a single API used by shell/shim/world-agent to resolve effective policy for a cwd.
    - Removes legacy “full strict policy file” parse/write surfaces from the crate.
  - `crates/shell`:
    - CLI surfaces for rendering/editing policy patches remain in the shell.
    - `policy current show` delegates effective resolution to the broker (no duplicate merge logic).
  - `crates/shim` and `crates/world-agent`:
    - Continue to call `substrate_broker::detect_profile` (or a successor API), but those calls now resolve patch-only policies and must not ignore policy resolution errors.
  - `crates/common`:
    - Hosts shared workspace discovery utilities (including disabled marker semantics) used by broker and shell.
- End-to-end flow:
  - Inputs:
    - cwd (for workspace discovery)
    - workspace patch file (if enabled)
    - global patch file (if present)
    - built-in defaults
  - Derived state:
    - workspace root (with `.substrate/workspace.disabled` honored)
    - effective policy (defaults → global patch → workspace patch)
  - Actions:
    - broker caches the effective policy for downstream evaluation
    - execution paths consult the broker-owned effective policy (world_fs/net/cmd approvals)
  - Outputs:
    - consistent `policy current show` output matching runtime behavior
    - deterministic policy deny/allow decisions and isolation requirements

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/next/sequencing.json` → `workspace_config_policy_unification`
- Prerequisite integration task IDs:
  - None; this ADR is a consolidation/remediation to bring broker/runtime behavior into compliance with ADR-0008’s patch-only policy contract.

## Security / Safety Posture
- Fail-closed rules:
  - Invalid YAML, unknown keys, or type mismatches in any policy patch file are actionable user errors (exit `2`) and MUST prevent command execution that depends on broker policy resolution.
  - A policy resolution error MUST NOT be downgraded to a warning + default policy fallback on execution paths.
- Protected paths/invariants:
  - Policy discovery reads only:
    - `<workspace_root>/.substrate/policy.yaml` (if workspace enabled)
    - `$SUBSTRATE_HOME/policy.yaml`
  - No other filenames/locations are consulted (no `.substrate-profile*`, no `.substrate-policy.yaml`).
- Observability:
  - Effective policy provenance is explainable via `policy current show --explain` (ADR-0012), but policy resolution correctness MUST NOT depend on `--explain` being used.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - Broker patch parsing + merge semantics (defaults → global → workspace) for representative keys (including `world_fs.require_world`).
  - Workspace discovery honors `.substrate/workspace.disabled`.
- Integration tests:
  - A workspace with a minimal policy patch (e.g., only `world_fs.require_world: true`) must:
    - succeed for `substrate policy current show`, and
    - not break `detect_profile`-dependent execution paths.
  - Regression coverage for previously “ignored error” call sites:
    - shim execution (`crates/shim`)
    - world-agent full isolation paths (`crates/world-agent`)

### Manual validation
- Manual playbook: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/manual_testing_playbook.md`

### Smoke scripts
- Linux: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/smoke/linux-smoke.sh`
- macOS: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/smoke/macos-smoke.sh`
- Windows: `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/smoke/windows-smoke.ps1`

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed.
- Compat work: none; legacy full-policy documents and legacy policy discovery locations are removed (not migrated).

## Decision Summary
- Decision Register entries:
  - `docs/project_management/_archived/policy-patch-only-broker-effective-resolution/decision_register.md`:
    - DR-0001
