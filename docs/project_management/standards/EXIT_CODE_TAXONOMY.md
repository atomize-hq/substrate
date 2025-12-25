# Exit Code Taxonomy (Canonical)

This document defines the **default exit code taxonomy** for new Substrate CLI work planned under `docs/project_management/next/…`.

Goal:
- Keep exit codes consistent across specs, playbooks, smoke scripts, and tasks.
- Avoid drift where the same failure mode maps to different exit codes in different documents.

## Canonical exit codes (default)

Unless a feature explicitly overrides this taxonomy, use the following meanings:

| Exit code | Meaning (default) | Examples |
|---:|---|---|
| 0 | Success / no-op by contract | Command succeeded; “not configured” status path that is explicitly a no-op; validation passed |
| 1 | Unexpected internal error | Bug, panic, invariant violation, unhandled I/O error that is not attributable to user input |
| 2 | User input / CLI usage / config error | Invalid flag/arg; invalid YAML; invalid schema; unknown tool id; invalid path value |
| 3 | Required dependency unavailable | World backend unreachable when required; missing executable required by the workflow (e.g., `jq` for a JSON-only check) |
| 4 | Not supported / missing prerequisites | Feature unsupported on this platform; requires provisioning; privileged capability missing and feature is not allowed to degrade |
| 5 | Safety / policy / protected-path violation | Attempt to touch protected paths; policy deny; safety guard triggered (e.g., protected directories) |

## Overrides (allowed, but must be explicit)

If a feature needs different exit codes, it must:
1) Declare the override in the ADR “User Contract (Authoritative)” section.
2) Repeat the override in each spec that defines CLI behavior.
3) Use the overridden codes consistently in:
   - `manual_testing_playbook.md`
   - smoke scripts under `docs/project_management/next/<feature>/smoke/`
   - `tasks.json` acceptance criteria / checklists

An override is valid only if it:
- is complete (defines all exit codes used),
- is unambiguous (one meaning per code),
- and is referenced from every document that uses exit codes.

## Spec snippet (copy/paste)

Use this snippet in specs that define CLI behavior:

```md
### Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 0: <feature-specific success/no-op semantics>
- 2: <feature-specific config/usage error semantics>
- 3: <feature-specific dependency-unavailable semantics>
- 4: <feature-specific unsupported/prereq-missing semantics>
- 5: <feature-specific safety/policy violation semantics>
```

