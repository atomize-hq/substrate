# Milestone 2.2: Mount Minimization and Ingress Contract

## Status

Draft

## Purpose / outcome

Replace the current broad Lima mount posture with a named ingress contract that states exactly which host inputs the guest may consume for normal operation, how they are mounted, and which flows must move to explicit Substrate-owned copying, syncing, or breakglass procedures.

## Why this milestone exists

The current Lima profile in [scripts/mac/lima/substrate.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate.yaml) mounts all of `$HOME` read-only and mounts the active project at `/src` read-write. That is convenient for bootstrapping, but it overexposes host content and leaves the real runtime contract underspecified. The same-user model already cannot replicate Linux ownership boundaries; broad host mounts make that limitation materially worse.

This milestone exists to shrink what the guest can see and to name the remaining required ingress paths instead of hiding them behind a full-home mount.

## In-scope

- Define the normal-operation ingress contract for the macOS Lima guest.
- Decide which host inputs remain mounted, which become copied or synchronized by Substrate-owned flows, and which are only available via breakglass.
- Narrow the default Lima profile mounts accordingly.
- Update docs and validation to reflect the new ingress model.

## Out-of-scope

- Designing the full Substrate-owned command UX for sync and diagnostics; that belongs to Phase 3.
- Replacing all guest package/tool bootstrap mechanics if the hardening can be achieved with a narrower temporary ingress path.
- Native macOS world execution outside Lima.

## Architectural approach

- Start from the runtime needs rather than from the existing convenience mounts.
- Separate ingress into concrete classes:
  - workspace source input needed for builds and smoke runs
  - host auth or credential inputs needed for specific flows
  - runtime artifacts that must persist across guest operations
  - operator troubleshooting inputs that should move to breakglass
- For each class, decide whether the right primitive is:
  - direct mount
  - Substrate-managed copy/sync
  - no default ingress
- Encode the result in the Lima profile and in the surrounding docs so the hardening stance is inspectable.

## Dependencies / sequencing

- This milestone depends on the current Lima backend staying in place.
- Milestone 2.1 should complete first so the ingress contract is defined against the reduced listener surface.
- Milestone 2.3 should consume the finalized ingress paths when locking service sandbox `ReadWritePaths`, `ProtectHome`, and related unit settings.
- Phase 3 lifecycle work depends on this milestone because Substrate-owned operations should not rely on a hidden full-home mount.

## Concrete repo surfaces and file pointers

- [scripts/mac/lima/substrate.yaml](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima/substrate.yaml)
  - current home and project mount definitions
  - likely primary place where mount minimization is expressed
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
  - currently assumes `/src` exists and that the mounted repo can be built or copied from inside the guest
  - contains sentinel logic that detects whether the intended host checkout is mounted
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
  - depends on the mounted repo and may expose additional ingress assumptions
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
  - currently states that `/src` mirrors the active checkout
  - should be tightened to describe only the allowed ingress
- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)
  - currently documents the full-home mount as part of normal setup
- [crates/world-agent/src/lib.rs](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/crates/world-agent/src/lib.rs)
  - not a mount definition point, but a useful anchor for separating transport hardening from guest-visible filesystem scope

## Deliverables

- A written ingress contract for same-user Lima normal operation.
- A mount-minimized Lima profile plan naming each surviving mount and its purpose.
- A migration map for flows currently depending on `$HOME` visibility.
- Updated evidence expectations for smoke, doctor, and operator documentation.

## Acceptance criteria

- The hardened macOS plan no longer depends on mounting all of `$HOME` by default.
- Every remaining default host mount is justified by a named runtime need and file pointer.
- Flows that still need broader host visibility are explicitly categorized as future Substrate-owned sync work or breakglass operations.
- A reviewer can tell which guest-visible host paths remain after hardening without reading the Lima profile line-by-line.

## Validation / evidence plan

- Produce a before/after mount inventory from `scripts/mac/lima/substrate.yaml`.
- Run `scripts/mac/lima-warm.sh --check-only` and a standard warm path to prove the narrowed mounts still support the intended happy path.
- Run `scripts/mac/smoke.sh` to identify any hidden assumptions about `$HOME`, auth locations, or tool caches.
- Review docs for commands that still imply “the guest can see my host home directory” and either remove or reclassify them.

## Risks / open questions

- Some auth flows may currently rely on dotfiles under `$HOME` without declaring that dependency.
- Narrowing mounts can create friction for developer-only debugging paths; those paths need deliberate breakglass handling rather than silent re-expansion.
- The right boundary between “direct mount” and “Substrate-managed sync” may depend on implementation work that lands in Phase 3.
