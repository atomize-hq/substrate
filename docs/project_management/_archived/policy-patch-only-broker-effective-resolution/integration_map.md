# Integration Map — Policy Patch-only + Broker-Canonical Effective Resolution

## Scope
- Policy patch-only parsing and merge semantics become broker-owned and are consumed consistently by:
  - `crates/shell` (CLI + command dispatch),
  - `crates/shim` (process interception),
  - `crates/world-agent` (in-world execution service).

## Slice map (what lands when)
- C0:
  - broker canonical effective policy resolution (patch-only, layered merge, disabled workspace semantics)
  - `substrate policy current show` delegates effective resolution to broker
- C1:
  - fail-closed behavior on policy resolution errors across broker-dependent execution paths (shell/shim/world-agent)
  - `docs/CONFIGURATION.md` updated to match patch-only contract and canonical paths

## Inputs → Derived State → Actions → Outputs

### Inputs
- `cwd` (for workspace discovery)
- Global policy patch file: `$SUBSTRATE_HOME/policy.yaml`
- Workspace marker: `<workspace_root>/.substrate/workspace.yaml`
- Workspace disabled marker: `<workspace_root>/.substrate/workspace.disabled`
- Workspace policy patch file (when workspace enabled): `<workspace_root>/.substrate/policy.yaml`
- Built-in policy defaults

### Derived state
- Workspace root (or “no workspace”) via marker discovery with disabled-marker semantics
- Parsed global policy patch (or empty)
- Parsed workspace policy patch (or empty)
- Effective policy computed deterministically:
  - defaults → global patch → workspace patch
- Explain/provenance object for `policy current show --explain`

### Actions
- Broker:
  - parse policy patch YAML (deny unknown keys)
  - apply patches over defaults
  - validate policy invariants
  - publish effective policy for evaluation and world settings
- Shell:
  - `policy current show` delegates effective resolution to broker
  - command execution path calls broker resolution and fails closed on error
- Shim:
  - on process interception, resolves effective policy via broker and fails closed on error
- World-agent:
  - on execution requests that require policy, resolves effective policy via broker and fails closed on error

### Outputs
- Consistent effective policy output and provenance via CLI
- Deterministic allow/deny and world enforcement settings derived from the same effective policy across all execution surfaces

## Component map (where changes land)
- `crates/broker`:
  - canonical policy patch schema, parsing, merge, invariant validation
  - canonical effective-policy resolution API
  - workspace discovery honoring `.substrate/workspace.disabled`
- `crates/shell`:
  - remove duplicate policy merge implementation for the effective policy view
  - delegate effective policy to broker for `policy current show`
  - treat broker policy resolution errors as actionable (exit `2`) for command execution
- `crates/shim`:
  - remove error swallowing around policy detection/resolution
  - fail closed on policy resolution errors
- `crates/world-agent`:
  - remove error swallowing around policy detection/resolution
  - classify policy resolution errors as user/config errors for API responses used by the shell
- `docs/CONFIGURATION.md`:
  - update policy file locations and patch-only semantics; remove legacy discovery locations

## Sequencing alignment
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- This feature is sequenced after the patch-only policy contract is established (ADR-0008) and before any work that assumes broker/runtime policy semantics are aligned.
