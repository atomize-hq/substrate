# C9-spec: Init UX & Migration

## Scope
- Improve `substrate init` UX:
  - Interactive/setup flags to choose defaults (world enablement, sync direction/conflict policy, ignores, auto_sync).
  - Clear summary of what will be created/updated; dry-run option.
- Migration for existing workspaces:
  - Detect existing `.substrate/` or `.substrate-git/`; repair missing pieces without clobbering data.
  - Warn/guide when multiple roots or stale configs exist; add safe “repair” flow.
- Gating UX:
  - When world is blocked due to missing init or incomplete state, provide actionable guidance and a one-shot helper (e.g., `substrate init --repair` suggestion).
- Docs/help updates as part of code task (inline help/messages); broader docs can follow in integration.

## Acceptance
- `substrate init` offers a non-interactive and interactive/dry-run path; reports planned/actual changes.
- Existing workspaces with partial state can be repaired without data loss; user `.git` untouched.
- Gating errors are actionable and reference the needed init/repair commands.
- Defaults applied match the current spec (from C1/C2/C3/etc.) unless overridden.

## Out of Scope
- Transport-level sync changes.
- Internal git mechanics beyond guiding/repairing presence.
