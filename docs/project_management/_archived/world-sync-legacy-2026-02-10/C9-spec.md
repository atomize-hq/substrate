# C9-spec: Init UX & Migration

## Scope
- Improve `substrate init` UX:
  - Interactive/setup flags to choose defaults (world enablement, sync direction/conflict policy, ignores, `sync.auto_sync`).
  - Clear summary of what will be created/updated; dry-run option.
- Migration for existing workspaces:
  - Detect existing `.substrate/` or `.substrate-git/`; repair missing pieces without clobbering data.
  - Warn/guide when multiple roots or stale configs exist; add safe “repair” flow.
- Gating UX:
  - When a command is blocked due to missing init or incomplete state, provide actionable guidance and a one-shot helper suggestion: `substrate init --force`.
- Docs/help updates as part of code task (inline help/messages); broader docs can follow in integration.

## Acceptance
- `substrate init` supports a non-interactive path, an interactive path, and a dry-run path; it reports planned changes and actual changes.
- Existing workspaces with partial state can be repaired without data loss; user `.git` untouched.
- Gating errors are actionable and reference the needed init/repair commands.
- Defaults applied match the current spec (from C1, C2, C3, and later slices) unless overridden.

## Out of Scope
- Transport-level sync changes.
- Internal git mechanics beyond guiding/repairing presence.
