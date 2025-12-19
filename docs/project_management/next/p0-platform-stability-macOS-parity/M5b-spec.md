# M5b Spec – Host detection parity (world deps)

## Goal
Make “host present” detection for world deps match the Substrate-managed host environment, so initial sync decisions reflect what users actually have available on their host shell.

This triad is about **host detection semantics**, not manifest layering or first-run UX wiring.

## Context / Problem
Many host tools are available only after manager init (nvm/asdf/pyenv/mise/etc). If world deps host detection does not source the same init environment, it will incorrectly report `host=missing`, causing first-run sync to skip tools users expect to be present.

## Scope
### Required behavior
1. **Host detection environment**
   - Host detection for world deps runs under the same manager init environment that Substrate uses for managed shells/commands.
   - Detection must be deterministic and avoid mutating user dotfiles.
2. **Compatibility**
   - Maintain support for non-bash hosts where possible; where not possible, behavior must be explicit and documented.
3. **Observability**
   - When host detection is skipped or degraded (e.g., missing shell), the reason should be surfaced clearly so users understand why sync did not occur.

### Out of scope
- Changing the set of tools (handled in M5a).
- Changing installer defaults or prompts (handled in M5c).
- Guest PATH/HOME normalization changes.

## Acceptance criteria
- On macOS, tools available via common managers (e.g., nvm/asdf/pyenv) are detected as host-present during `world deps status/sync` when they are effectively available in Substrate-managed shells.
- `world deps sync` decisions align with what users experience when running commands on the host through Substrate.
- Failure/degraded detection modes are visible and actionable.

