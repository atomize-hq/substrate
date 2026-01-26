# WCU4 Spec — Installer/dev env cleanup (stop exporting `SUBSTRATE_OVERRIDE_*` by default)

References:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Remove default exporting of `SUBSTRATE_OVERRIDE_*` from install/dev environment scripts so overrides are treated as explicit one-off operator input.

## Non-goals
- Changing override semantics themselves (overrides remain supported when explicitly provided by the operator).

## Authoritative contract
- Install/dev scripts MUST NOT export any `SUBSTRATE_OVERRIDE_*` environment variables by default.
- Operators may still set `SUBSTRATE_OVERRIDE_*` explicitly in their shell/session; Substrate respects them as highest-precedence inputs.

## Validation requirements (authoritative)
- Tests and/or integration checks must demonstrate:
  - a clean environment has no `SUBSTRATE_OVERRIDE_*` exports after running the relevant dev/install scripts, and
  - `config current show` behavior is not silently affected by unintended override exports.
