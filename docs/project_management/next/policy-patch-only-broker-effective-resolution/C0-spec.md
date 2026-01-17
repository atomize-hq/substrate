# C0-spec — Broker Canonical Policy Resolver + CLI Delegation

Authoritative ADR:
- `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Policy files are patch-only everywhere (no full-policy document format is supported on disk).
- The broker resolves the effective policy via patch merge (defaults → global patch → workspace patch).
- The policy CLI’s effective view delegates to the broker (no duplicate merge implementation in the shell).
- Broker workspace discovery honors `.substrate/workspace.disabled` exactly as the shell’s workspace contract.

## Inputs and on-disk contract (authoritative)

### Policy patch files (locations)
- Global policy patch file:
  - Path: `$SUBSTRATE_HOME/policy.yaml` (where `$SUBSTRATE_HOME` is the Substrate home directory).
- Workspace policy patch file:
  - Path: `<workspace_root>/.substrate/policy.yaml`

No other filenames or locations participate in policy discovery or effective resolution.

### Workspace discovery (authoritative)
- Workspace root marker:
  - Path: `<workspace_root>/.substrate/workspace.yaml`
- Workspace disabled marker:
  - Path: `<workspace_root>/.substrate/workspace.disabled`
- If the disabled marker exists, that workspace is treated as non-existent for:
  - policy discovery,
  - effective policy resolution.

### Patch-only format (authoritative)
- A policy patch file is a YAML mapping that represents sparse overrides:
  - omitted keys inherit from the next-lower-precedence layer,
  - `{}` is an empty patch,
  - unknown keys are a hard error.
- The repo does not accept or parse any full-policy document format as a supported on-disk contract.

### Effective resolution rule (authoritative)
- Effective policy is computed as:
  1. built-in defaults,
  2. overlaid by the global policy patch (if present),
  3. overlaid by the workspace policy patch (if a workspace exists and the workspace patch file is present).

The effective policy is deterministic for the same inputs.

## Behavior changes (authoritative)

### Broker is the canonical resolver
- All runtime code paths that depend on broker policy resolution use a single broker-owned effective policy resolver.
- `substrate policy current show` and `substrate policy current show --explain` delegate effective policy computation to the broker (no duplicate merge implementation in the shell).

## CLI contract (policy current show)

### Output
- `substrate policy current show`:
  - prints the effective policy to stdout (YAML by default, JSON when `--json`),
  - prints a single note line to stderr indicating the output is an effective merged policy view.
- `substrate policy current show --explain`:
  - prints the effective policy to stdout (YAML by default, JSON when `--json`),
  - prints deterministic explain JSON to stderr (single JSON object per invocation).

### Explain JSON contract
- Explain JSON is deterministic for the same inputs.
- Explain JSON contains per-key provenance with:
  - `merge_strategy`,
  - `sources` as an ordered list of contributing layers.
- Provenance source labels MUST follow ADR-0012’s contract; for policy, the contributing layers are typically:
  - `default`,
  - `global_patch` (with `path`),
  - `workspace_patch` (with `path`).

## Exit codes (authoritative)
- `0`: success.
- `1`: unexpected internal error.
- `2`: invalid policy patch input, invalid schema/type, unknown key, invariant violation, or any policy resolution error that blocks execution.
- `3`: missing required external dependency used by a workflow step (applies to smoke/manual scripts only).
- `4`: not supported on this platform (not used by this feature’s core contract).
- `5`: policy deny or protected-path guard (not used by this feature’s core contract).

## Acceptance criteria
- Effective policy resolution is identical across:
  - broker resolution API,
  - `substrate policy current show`,
  - `substrate policy current show --explain`.
- A minimal policy patch file that omits most keys is accepted (patch-only) and does not break execution.
- `.substrate/workspace.disabled` prevents workspace policy from affecting the effective policy used by the broker.

## Out of scope
- Any compatibility support for legacy filenames or legacy directories referenced by existing operator docs.
- Any additional policy layers beyond defaults/global/workspace.
