# Kickoff: WDH0-code (code)

## Scope

- Production code only; no new tests.
- Spec: `docs/project_management/packs/active/world-deps-host-visible-hardening/WDH0-spec.md`
- Execution workflow standard: `docs/project_management/system/standards/triad/TASK_TRIADS_WORKTREE_EXECUTION_STANDARD.md`

## Requirements

- Implement sanitized env construction (PTY + non-PTY) and the config lever `world.env.inherit_from_host`.
- Do not edit planning docs inside the worktree.
