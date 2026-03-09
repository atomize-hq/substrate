# WDD0-spec — Resolve effective `world.enabled` for diagnostics (shared seam)

## Behavior delta (single)
- Existing: `substrate shim doctor` / `substrate health` do not have a single shared, deterministic “is the world enabled?” classifier based on the authoritative effective-config precedence stack, so downstream slices cannot reliably gate disabled/skipped behavior without duplicating precedence logic.
- New: diagnostics resolve effective `world.enabled` via the existing effective-config resolver (the same precedence stack used for normal invocations) and expose a single shared classifier for later disabled/skipped gating, while failing fast (exit code `2`) on effective-config resolution errors (no silent misclassification).
- Why: make “disabled vs enabled-but-broken” deterministic and testable, and ensure config-resolution failures don’t produce misleading diagnostics.

## Scope
- Add a shared helper (or equivalent) that computes `effective_world_enabled` for diagnostics given:
  - current working directory, and
  - CLI world overrides (`--world` / `--no-world`).
- Add unit coverage for precedence and failure posture.
- Ensure diagnostics entrypoints treat effective-config resolution failures as user/config errors (exit `2`) and do not proceed to probing/output.

Likely touch surfaces (non-authoritative):
- `crates/shell/src/execution/config_model.rs` (caller patterns; `CliConfigOverrides`)
- `crates/shell/src/execution/routing.rs` (exit-code mapping for `shim doctor` user errors)
- `crates/shell/src/builtins/shim_doctor/report.rs` (call-site integration for downstream gating)
- `crates/shell/src/builtins/health.rs` (call-site integration for downstream gating)

## Inputs (authoritative)
- Effective-config precedence and `SUBSTRATE_OVERRIDE_*` ignore rule: `docs/reference/env/contract.md`
- Effective-config resolver API: `crates/shell/src/execution/config_model.rs` (`resolve_effective_config`)
- Diagnostics exit-code posture: `docs/project_management/packs/draft/world-disabled-diagnostics/contract.md` (Exit codes)

## Behavior (authoritative)

### Effective world-enabled resolution
- Diagnostics MUST compute `effective_world_enabled` using the existing resolver:
  - `config_model::resolve_effective_config(cwd, CliConfigOverrides { world_enabled: <override>, .. })`
- The CLI override MUST be derived as:
  - `Some(true)` when `--world`
  - `Some(false)` when `--no-world`
  - `None` when neither is provided
- Diagnostics MUST NOT implement ad-hoc precedence (no direct parsing of `SUBSTRATE_OVERRIDE_WORLD` or workspace markers outside the resolver).

### Failure posture (no misclassification)
- If effective-config resolution fails (invalid YAML, unreadable config, invalid override env value):
  - emit a user-facing error to stderr, and
  - exit with code `2` (user/config error),
  - and MUST NOT proceed to gather world/world-deps diagnostics (avoid “disabled/enabled” guesses).

## Acceptance criteria
- AC-WDD0-01: With invalid YAML at `$SUBSTRATE_HOME/config.yaml`, both `substrate shim doctor` and `substrate health` fail fast with exit code `2` and a stderr error that names the offending path.
- AC-WDD0-02: Outside an enabled workspace, with `SUBSTRATE_OVERRIDE_WORLD=disabled` (and no CLI world override), the shared classifier resolves `effective_world_enabled=false`.
- AC-WDD0-03: Inside an enabled workspace (`<workspace>/.substrate/workspace.yaml` present), even when `SUBSTRATE_OVERRIDE_WORLD=disabled` is set, the resolver ignores the override per `docs/reference/env/contract.md`, and the shared classifier resolves based on workspace/global/default layers (e.g., a workspace patch `world.enabled: true` wins).
- AC-WDD0-04: Precedence: `--world` wins over config/env disablement (`effective_world_enabled=true`), and `--no-world` wins over config/env enablement (`effective_world_enabled=false`).

## Out of scope
- Disabled/skipped UX, JSON status enums, and probe short-circuiting for diagnostics (implemented in WDD1/WDD2).
- “Why disabled” attribution strings/enums (explicitly out of scope for ADR-0036).

