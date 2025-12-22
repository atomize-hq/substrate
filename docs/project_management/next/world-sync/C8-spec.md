# C8-spec: World-side Internal Git Bootstrap/Bridge

## Scope
- Ensure the world environment has access to `.substrate-git`:
  - Create/clone a world-side `.substrate-git/.git` (lazy on first world session) using the host repo as source (or initialize if missing).
  - Keep world commits aligned with host by mirroring Substrate commits (push/pull or shared bare repo) so rollback/checkpoint references remain valid in both environments.
- Agent/world enforcement:
  - World commands that rely on internal git should fail with a clear error if the world repo cannot be prepared.
  - Protect user `.git` inside the world (no accidental use).
- Sync interactions:
  - World-side commits should be recorded for world→host applies; host should see aligned commit ids (or mapping) after sync.
  - Document/emit logs when world-side git falls back to a degraded mode.

## Acceptance
- On first world session after init, a world `.substrate-git` exists and is tied to the host’s internal repo (clone/mirror/shared bare).
- World operations that need internal git succeed when available; otherwise surface clear errors without mutating host.
- Commit alignment: Substrate-created commits are visible/usable from both host and world (either same hash via mirror/shared repo or a stable mapping).
- User `.git` is never touched inside the world; `.substrate-git` remains isolated.

## Out of Scope
- Rollback CLI changes (handled in C7).
- Init UX (handled in C9).
