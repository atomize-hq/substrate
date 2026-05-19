# Milestone 3.2: Breakglass Reclassification and Doc Cutover

## Status

Draft

Last updated: 2026-05-19

## Purpose / outcome

Reclassify remaining direct macOS/Lima guest administration as breakglass and
cut the repo documentation over so the default operator path is the
Substrate-owned lifecycle, diagnostics, and gateway lifecycle/status surface.

## Why this milestone exists

Even after owned commands exist, the repo will continue to communicate the
wrong security posture if the docs still teach:

- building or installing the guest agent through raw `limactl shell substrate ...`
- enabling or restarting guest services manually as the first-line flow
- probing the guest socket directly as the normal health check
- using direct guest logs and shell access without labeling them as exceptional

That mismatch is more visible now because the repo already has the surfaces the
docs should lead with:

- `substrate host doctor`
- `substrate world doctor`
- `substrate world gateway sync|status|restart`
- `substrate world gateway status --json`
- managed runtime artifact paths under
  `/run/substrate/substrate-gateway-runtime/`

This milestone exists because the hardening story is incomplete until the operator-facing documentation and troubleshooting posture match the new operational boundary.

## In-scope

- Reclassify direct guest commands in macOS docs and helper guidance as breakglass where they still exist.
- Rewrite the macOS setup and operations narrative to lead with Substrate-owned commands.
- Align troubleshooting and evidence sections with the new contract.
- Classify host-side `SUBSTRATE_WORLD_SOCKET` override use as advanced/test or
  breakglass rather than the default supported Lima path.
- Mark legacy scripts or workflows as supported, degraded-but-supported, or
  breakglass/unsupported as appropriate.

## Out-of-scope

- Inventing new lifecycle commands beyond the set frozen in milestone 3.1.
- Removing all deep-debugging guidance; this milestone reclassifies it, not erases it.
- General documentation cleanup outside the macOS hardened-Lima scope.

## Architectural approach

- Separate every operator action into the phase-0 support taxonomy:
  - supported: owned by Substrate and documented first
  - degraded-but-supported: retained temporarily with an explicit migration note
    and bounded expectations
  - breakglass/unsupported: allowed for deep debugging or recovery, but not
    part of normal setup or maintenance
- Apply that classification consistently across `README`-style docs, setup docs, troubleshooting sections, and helper-script messaging.
- Treat `substrate world gateway sync|status|restart` and status JSON as part of
  the supported operator contract, not as optional or experimental side paths.
- Treat runtime artifact references under
  `/run/substrate/substrate-gateway-runtime/` as supported operator evidence
  when surfaced by Substrate-owned commands, while keeping raw guest shelling as
  breakglass.
- Ensure the docs explicitly call out the unresolved same-user ownership gap so users do not mistake the cutover for Linux-equivalent authority separation.

## Dependencies / sequencing

- Depends on milestone 3.1 so the docs can point to concrete owned commands rather than aspirational placeholders.
- Should be the final milestone in this planning packet because it completes the user-visible hardening posture.

## Concrete repo surfaces and file pointers

- `docs/cross-platform/mac_world_setup.md`
  - primary setup and troubleshooting cutover target
- `docs/WORLD.md`
  - primary runtime and operator contract target
- `docs/contracts/substrate-gateway-operator-contract.md`
  - already-landed gateway lifecycle operator contract that docs should lead with
- `docs/contracts/substrate-gateway-status-schema.md`
  - status JSON contract the docs should reference instead of redefining
- `scripts/mac/lima-doctor.sh`
  - messaging and role should match the reclassification
- `scripts/mac/lima-warm.sh`
  - usage/help text should reflect whether it is supported,
    degraded-but-supported, or breakglass/unsupported
- `scripts/mac/smoke.sh`
  - evidence instructions should match the owned operations narrative

## Deliverables

- A doc cutover plan for macOS hardened-Lima setup, lifecycle, troubleshooting, and validation.
- A breakglass classification table for the remaining direct guest actions.
- Updated script/help-text expectations aligned with the new classifications.
- A final operator narrative that distinguishes same-user hardening from
  Linux-equivalent ownership and leads with already-landed doctor/gateway
  commands.

## Acceptance criteria

- The default macOS docs lead with Substrate-owned lifecycle and diagnostic commands.
- The default macOS docs also lead with
  `substrate world gateway sync|status|restart` and status JSON for managed
  gateway lifecycle.
- Remaining `limactl shell` and direct guest systemctl/socket instructions are
  explicitly labeled breakglass/unsupported.
- Any degraded-but-supported path that remains is a compatibility wrapper
  around a Substrate-owned command surface, not a raw direct guest procedure.
- Troubleshooting sections make clear when a user should escalate from owned commands to breakglass commands.
- A reviewer can read the macOS docs and understand both the hardened default and the remaining same-user limitation without external context.
- Host-side `SUBSTRATE_WORLD_SOCKET` override use is not documented as the
  normal supported Lima path.

## Validation / evidence plan

- Perform a doc inventory of direct guest commands and host-side overrides in
  `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` and classify
  each one.
- Review script help output and inline messaging to ensure it matches the new operator contract.
- Re-run the documented happy path using only Substrate-owned commands and
  verify that the required evidence can be gathered without breakglass access.
- Verify that any remaining breakglass guidance is concrete, bounded, and clearly separated from the normal path.

## Risks / open questions

- Transitional periods are easy to misdocument; helper scripts can silently remain the de facto primary workflow unless the docs are explicit.
- Some advanced troubleshooting paths may still need raw guest commands that are too useful to hide; the breakglass policy must stay practical.
- The docs must avoid overstating the hardening result: Phase 3 improves operational ownership, but it does not eliminate the same-user Lima trust boundary gap.
