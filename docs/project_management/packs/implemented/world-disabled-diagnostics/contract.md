# world-disabled-diagnostics — contract surface

This file is the single place to consolidate the user-facing contract for ADR-0036 (CLI behavior, JSON output posture, exit codes).

Decision inputs:
- `docs/project_management/packs/draft/world-disabled-diagnostics/decision_register.md` (DR-0001/2/3)
- `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`

## CLI

### Commands in scope

- `substrate shim doctor` (text)
- `substrate shim doctor --json`
- `substrate health` (text)
- `substrate health --json`

Global flags in scope (authoritative parsing/precedence lives elsewhere; see Config section):
- `--world`
- `--no-world`

### Definitions

- “World disabled” means the effective config resolves `world.enabled=false` after precedence resolution.
- “World enabled” means the effective config resolves `world.enabled=true` after precedence resolution.

### Effective-config resolution (required)

`substrate shim doctor` and `substrate health` MUST resolve effective `world.enabled` using the same effective-config resolver used for normal Substrate invocations.

Authoritative precedence and env-var semantics are owned by:
- `docs/reference/env/contract.md` (effective-config precedence + `SUBSTRATE_OVERRIDE_*` ignore rule)
- `docs/CONFIGURATION.md` (config keys, including `world.enabled`)

### Behavior — world disabled (`world.enabled=false`)

When world is disabled, diagnostics MUST degrade quietly and explicitly:

- **No probes (invariant):**
  - MUST NOT probe the world backend for diagnostics purposes (no world-service socket calls).
  - MUST NOT spawn `substrate world doctor --json` from inside diagnostics.
  - MUST NOT compute world-deps “applied” state (probe-backed).

- **Text output (deterministic copy; DR-0003):**
  - `substrate health` MUST include the following exact lines (verbatim):

    ```text
    World backend: disabled
      Next: run `substrate world enable` to provision
    World deps: skipped (world disabled)
    ```

  - `substrate health` MUST NOT print enabled-world world-deps remediation guidance (for example, it MUST NOT print lines containing `substrate world deps current`).

  - `substrate shim doctor` MUST include the following exact lines (verbatim):

    ```text
    World backend:
      Status: disabled
      Next: run `substrate world enable` to provision
    World deps:
      Status: skipped (world disabled)
    ```

    - MUST NOT represent disabled/skipped as an error (no `Error:` line for these disabled/skipped states).

- **JSON output (deterministic schema; DR-0001/2):**
  - Must follow: `docs/project_management/packs/draft/world-disabled-diagnostics/world-disabled-diagnostics-json-schema-spec.md`
  - Summary posture: disabled/skipped is non-error; legacy error fields are omitted per DR-0002.

### Behavior — world enabled (`world.enabled=true`)

When world is enabled, diagnostics MUST remain fail-visible:
- If the world backend is broken/unreachable, diagnostics MUST report “needs attention” with actionable error details (no masking as disabled/skipped).
- World-deps “applied” probing behavior remains enabled (unless separately skipped for other reasons not introduced by this feature).

## Config

- Effective config precedence is authoritative and unchanged by this feature:
  - `docs/reference/env/contract.md`
- This feature introduces **no new config keys** and **no new environment variables**.

### Effective-config resolution failure

If effective-config resolution fails (invalid YAML, unreadable config, invalid override env value), diagnostics MUST:
- emit a user-facing error to stderr, and
- exit with a non-zero code (see Exit codes).

Diagnostics MUST NOT silently misclassify “disabled” vs “enabled” when the effective config cannot be determined.

## Exit codes

- Taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- Overrides: none

Mapping for `substrate health` and `substrate shim doctor`:
- `0`: report generated successfully (including when output contains “needs attention” or “disabled/skipped” statuses).
- `2`: user/config error (invalid flags/args, invalid YAML/config, invalid config-shaped env values, or other effective-config resolution failures).
- `1`: unexpected internal error (I/O failures unrelated to user config, JSON serialization failure, invariant violations).

## Platform guarantees

Disabled/skipped semantics MUST be consistent across Linux/macOS/Windows:
- Same meaning for “world disabled” (effective-config-derived, not socket-derived).
- Same JSON field paths and enum spellings (see schema spec).
- Same disabled/skipped operator copy lines (above), modulo platform-specific paths printed elsewhere in the report.
