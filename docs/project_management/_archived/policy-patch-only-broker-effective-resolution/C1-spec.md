# C1-spec — Fail-closed Policy Resolution Across Execution Paths + Docs Alignment

Authoritative ADR:
- `docs/project_management/adrs/implemented/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`

Exit codes:
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Policy resolution errors fail closed (no silent fallback) across execution paths that consult the broker:
  - shell execution routing/dispatch,
  - shim execution interception,
  - world-agent execution flows that consult broker policy.
- Operator docs are updated to match the patch-only policy contract and canonical discovery locations:
  - `docs/CONFIGURATION.md`

## Behavior (authoritative)

### Fail-closed on policy resolution errors
- A policy resolution error prevents command execution for broker-dependent execution paths.
- A policy resolution error is surfaced as an actionable user/config error:
  - CLI exit code: `2`
  - World-agent API error classification: “bad request” / user error (HTTP 400), not internal server error.

### Error triggers (policy resolution error)
- invalid YAML
- unknown keys
- type mismatches
- invariant violations
- permission/read errors when a policy file exists but is unreadable

### Non-behavioral doc alignment
- `docs/CONFIGURATION.md` is updated to remove legacy policy discovery locations and to document:
  - patch-only semantics (sparse YAML mapping),
  - canonical policy patch file locations,
  - layered effective resolution rule (defaults → global patch → workspace patch),
  - workspace disabled marker effect on policy discovery.

## Exit codes (authoritative)
- `0`: success.
- `1`: unexpected internal error.
- `2`: policy resolution error (invalid YAML/schema/type/unknown key/invariant/permission) and any blocked execution due to policy resolution failure.
- `3`: missing required external dependency used by a workflow step (smoke/manual scripts only).

## Acceptance criteria
- With an invalid policy patch file present, the following do not execute the command and return exit `2`:
  - `substrate --command "<cmd>"` (shell execution path),
  - shim execution path (intercepted commands),
  - world-agent path that consults broker policy for the request.
- `docs/CONFIGURATION.md` matches the patch-only contract and canonical policy file locations used by runtime resolution.

## Out of scope
- Migrations or compatibility for legacy policy locations or full-policy formats.

