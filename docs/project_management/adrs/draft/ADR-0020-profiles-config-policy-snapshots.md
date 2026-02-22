# ADR-0020 — Profiles: Full Config/Policy Snapshots + Surface Scoping

## Status
- Status: Draft
- Date (UTC): 2026-01-30
- Owner(s): Shell maintainers

## Scope
- Feature directories (impacted):
  - `docs/project_management/_archived/next/` (this ADR; cross-cutting contract)
- Sequencing spine: `docs/project_management/packs/sequencing.json`
- Standards:
  - `docs/project_management/system/standards/adr/ADR_STANDARD_AND_TEMPLATE.md`
  - `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Related Docs
- Policy/config scope + patch files:
  - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- Env override taxonomy:
  - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`
- World-first REPL (motivation for surface scoping + drift messaging):
  - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`

## Executive Summary (Operator)

ADR_BODY_SHA256: 954c6e25ffe2599ff28304e9a842cae35f5806b60450ee221e4f2fce4909e939
### Changes (operator-facing)
- Profiles can pin full behavior across config + policy
  - Existing: effective policy/config are derived by merging defaults + global + workspace patches (plus CLI/env overrides) based on the current directory.
  - New: an optional “profile” can be activated to provide a complete, explicit config+policy snapshot for selected command surfaces (REPL vs `-c`), preventing accidental leakage of defaults/global/workspace layers.
  - Why: provides deterministic, intent-aligned behavior for humans vs agents without multiplying ad-hoc override knobs; improves debuggability by making “what is active?” explicit.
  - Links:
    - `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
    - `docs/project_management/adrs/draft/ADR-0016-world-first-repl-persistent-pty.md`

## Problem / Context
- Substrate is used by both:
  - humans (interactive REPL orchestration), and
  - agents (non-interactive `substrate -c ...` execution).
- Today, effective behavior is assembled from multiple layers (defaults/global/workspace + overrides). This is powerful but can be confusing:
  - operators want deterministic, explainable “known-good” bundles for different usage modes (human vs agent),
  - and certain REPL behaviors (drift restarts, caging/anchor semantics) make cross-layer interactions more visible and more surprising.
- Without a “single source of truth” mechanism, we keep accumulating surface-specific knobs and precedence exceptions.

## Goals
- Provide a first-class “profile” concept that can supply a **full** config and policy snapshot.
- Ensure profile activation is explicit, inspectable, and explainable.
- Allow profile scoping by command surface (at minimum: interactive REPL vs non-interactive `-c`).
- Provide tooling to validate and mechanically update profiles when schema changes.

## Non-Goals
- Defining a new sparse/merge format for profiles (profiles are complete snapshots, not patches).
- Backwards compatibility for legacy profile formats (greenfield).
- Solving all future “profile priority vs workspace” policy interactions (this ADR defines a default and a safe extension point, but does not require a merge/intersection engine in v1).

## User Contract (Authoritative)

### Terminology
- **Profile**: a named bundle containing a complete config snapshot and a complete policy snapshot, plus profile metadata.
- **Surface**: an execution entrypoint category (e.g. interactive REPL vs `substrate -c ...`).

### Storage
- Profiles live under `SUBSTRATE_HOME`:
  - `SUBSTRATE_HOME/profiles/<name>/profile.yaml` (metadata; includes surface scoping)
  - `SUBSTRATE_HOME/profiles/<name>/config.yaml` (full config snapshot)
  - `SUBSTRATE_HOME/profiles/<name>/policy.yaml` (full policy snapshot)

### Completeness rule (no leakage)
- When a profile is applied for a given surface:
  - `config.yaml` MUST be treated as the **entire effective config**.
  - `policy.yaml` MUST be treated as the **entire effective policy**.
  - Defaults/global/workspace layers MUST NOT be consulted to fill missing values.
- Therefore:
  - Profile `config.yaml` and `policy.yaml` MUST be validated as **complete** objects under the current schema.
  - Profiles with missing required keys MUST be rejected (hard error).

### Activation model
- Exactly zero or one profile may be active at a time.
- Activation is represented via a lightweight pointer under:
  - `SUBSTRATE_HOME/profiles/current/<name>` (marker file).
  - The `current/` directory MUST contain at most one entry.
- A profile may be marked disabled via:
  - `SUBSTRATE_HOME/profiles/disabled/<name>` (marker), in which case:
    - it MUST NOT be eligible for activation, and
    - attempts to activate it MUST error with an actionable message.
- “Available profiles” are the set of installed profiles under `SUBSTRATE_HOME/profiles/<name>/` that are not disabled.
- Reserved profile names: `current`, `disabled`.

### Surface scoping (profile metadata)
`profile.yaml` MUST include:
- `schema_version: <int>`
- `name: <string>`
- `applies_to`: explicit surface gates:
  - `repl: true|false`
  - `command: true|false` (non-interactive `substrate -c ...` and stdin pipe execution)
  - (optional future) `subcommands: true|false` (e.g., `substrate world doctor`, `substrate config ...`)

Rule:
- If a profile is active but does not apply to the current surface, Substrate MUST behave as if no profile is active for that invocation.

### Precedence
- When no profile applies, effective policy/config resolution remains as defined by existing ADRs and current behavior (defaults + global patch + workspace patch + operator overrides).
- When a profile applies:
  - Substrate MUST ignore defaults/global/workspace policy+config layers for that surface.
  - Operator override inputs (as defined by ADR-0006) MAY still apply on top of the active profile (explicit operator action; not “leakage”).

### CLI
New commands (draft contract; exact UX strings may iterate, behavior is normative):
- `substrate profile list`: list available profiles and whether they are disabled/current.
- `substrate profile status`: print the active profile (if any) and which surfaces it applies to.
- `substrate profile activate <name>`: activate a profile (writes/updates `profiles/current/` pointer).
- `substrate profile deactivate`: deactivate the active profile (removes `profiles/current/` pointer).
- `substrate profile disable <name>` / `substrate profile enable <name>`: manage disabled state markers.
- `substrate profile check [<name>|--all]`: validate profile(s) for completeness and schema compatibility.
- `substrate profile check --fix [<name>|--all] --default`: rewrite profiles to the current schema, filling missing keys from Substrate defaults (not from workspace/global).
- `substrate profile show <name>`: show the profile’s config/policy snapshots as stored.

Exit codes:
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Profile errors MUST be fail-closed for the affected invocation:
  - Invalid/missing active profile contents (missing files, invalid YAML, schema mismatch, incomplete snapshot) MUST be a hard error.
  - Disabled profile activation MUST be a hard error.

### Explainability
- When a profile applies, `substrate config show --explain` and `substrate policy show --explain` MUST surface that a profile is active and that workspace/global/default layers are ignored for that invocation.
- When a workspace-local policy/config exists but is being ignored due to an active profile, Substrate SHOULD emit a one-time high-signal operator note (per invocation) indicating that workspace layers are ignored.

### Workspace vs profile priority (default + safe extension point)
- Default behavior (v1):
  - Applied profiles override workspace/global/default layers for the surfaces they apply to (as defined above).
- Optional future extension:
  - Add an explicit policy-level “restrictive merge” mode where workspace policy may further restrict (but never loosen) a profile policy, for repos that must enforce minimum safety constraints.
  - This extension is out of scope for v1 unless explicitly accepted by a decision register entry.

### Platform guarantees
- Linux/macOS/Windows: marker-file activation MUST be supported and profile selection semantics MUST match.
- Windows: profile activation MUST NOT require symlinks (marker files only).

## Architecture Shape
- Components:
  - `crates/shell`:
    - profile discovery/activation (filesystem pointers under `SUBSTRATE_HOME/profiles/`)
    - surface detection (REPL vs `-c` / stdin pipe)
    - loader that selects either profile snapshots or the existing layered resolution path
    - `--explain` provenance updates to reflect profile selection
  - `crates/broker`:
    - policy snapshot parsing/validation for profile `policy.yaml` (complete object validation)
  - `docs/internals/env/inventory.md`:
    - update taxonomy documentation as needed to clarify that profiles are not override-env inputs
- End-to-end flow:
  - Inputs:
    - CLI surface (REPL vs `-c`)
    - `SUBSTRATE_HOME/profiles/current/*` activation pointer
    - profile directory contents (`profile.yaml`, `config.yaml`, `policy.yaml`)
  - Derived state:
    - “active profile for this surface?” boolean
    - validated complete config + policy snapshots
  - Actions:
    - if active+applies: use profile snapshots as the effective config/policy
    - else: use existing effective resolution (defaults/global/workspace + overrides)
  - Outputs:
    - normal Substrate execution with explicit explainability and fail-closed behavior on profile invalidity

## Sequencing / Dependencies
- Sequencing entry: `docs/project_management/packs/sequencing.json` (add a profile workstream before accepting this ADR)
- Dependencies:
  - Must remain consistent with ADR-0008 patch file and scope model (no schema conflicts).
  - If profile schema changes require env taxonomy updates, align with ADR-0006 guidance.

## Security / Safety Posture
- Fail-closed rules:
  - If a profile applies to the current surface and is invalid/incomplete, Substrate MUST fail closed (do not fall back to layered resolution).
  - Profile activation MUST be explicit and inspectable (avoid silent behavior changes).
- Invariants:
  - A profile snapshot is treated as the whole truth for config/policy when applied (no “leaking” workspace/global/default layers).
  - Substrate MUST surface (via `--explain` and/or explicit notes) when workspace layers are being ignored due to an active profile.

## Validation Plan (Authoritative)

### Tests
- Unit tests:
  - parsing/validation of `profile.yaml` metadata (schema version, applies_to gates)
  - “complete snapshot” validation: reject missing keys; reject unknown keys; enforce schema version
- Integration tests:
  - active profile applies to REPL but not `-c` (and vice versa)
  - `--explain` output indicates profile selection and layer suppression
  - `profile check --fix --default` produces a schema-complete profile from an older schema version

### Manual validation
- Manual playbook: not required while `Status: Draft` (required once accepted; must live under the `Feature directory` declared in the Accepted ADR header).

### Smoke scripts
- Not required for Draft.

## Rollout / Backwards Compatibility
- Policy: greenfield breaking is allowed
- Compat work: none (no backwards compatibility with any pre-existing profile prototypes)

## Decision Summary
- No decision register is included in this Draft ADR.
- If this ADR is accepted and remains non-trivial (surface scope rules, workspace-vs-profile priority behavior, and fix tooling), create:
  - `docs/project_management/_archived/next/<feature>/decision_register.md`
  and move A/B selections there (ADR remains the authoritative end-to-end contract).
