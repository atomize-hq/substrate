# Milestone 3.2: Breakglass Reclassification and Doc Cutover

## Status

Draft

## Purpose / outcome

Reclassify remaining direct macOS/Lima guest administration as breakglass and cut the repo documentation over so the default operator path is the Substrate-owned lifecycle and diagnostics surface.

## Why this milestone exists

Even after owned commands exist, the repo will continue to communicate the wrong security posture if the docs still teach:

- building or installing the guest agent through raw `limactl shell substrate ...`
- enabling or restarting guest services manually as the first-line flow
- probing the guest socket directly as the normal health check
- using direct guest logs and shell access without labeling them as exceptional

This milestone exists because the hardening story is incomplete until the operator-facing documentation and troubleshooting posture match the new operational boundary.

## In-scope

- Reclassify direct guest commands in macOS docs and helper guidance as breakglass where they still exist.
- Rewrite the macOS setup and operations narrative to lead with Substrate-owned commands.
- Align troubleshooting and evidence sections with the new contract.
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
- Ensure the docs explicitly call out the unresolved same-user ownership gap so users do not mistake the cutover for Linux-equivalent authority separation.

## Dependencies / sequencing

- Depends on milestone 3.1 so the docs can point to concrete owned commands rather than aspirational placeholders.
- Should be the final milestone in this planning packet because it completes the user-visible hardening posture.

## Concrete repo surfaces and file pointers

- [docs/cross-platform/mac_world_setup.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/cross-platform/mac_world_setup.md)
  - primary setup and troubleshooting cutover target
- [docs/WORLD.md](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/docs/WORLD.md)
  - primary runtime and operator contract target
- [scripts/mac/lima-doctor.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-doctor.sh)
  - messaging and role should match the reclassification
- [scripts/mac/lima-warm.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/lima-warm.sh)
  - usage/help text should reflect whether it is supported,
    degraded-but-supported, or breakglass/unsupported
- [scripts/mac/smoke.sh](/Users/spensermcconnell/__Active_Code/atomize-hq/substrate/scripts/mac/smoke.sh)
  - evidence instructions should match the owned operations narrative

## Deliverables

- A doc cutover plan for macOS hardened-Lima setup, lifecycle, troubleshooting, and validation.
- A breakglass classification table for the remaining direct guest actions.
- Updated script/help-text expectations aligned with the new classifications.
- A final operator narrative that distinguishes same-user hardening from Linux-equivalent ownership.

## Acceptance criteria

- The default macOS docs lead with Substrate-owned lifecycle and diagnostic commands.
- Remaining `limactl shell` and direct guest systemctl/socket instructions are
  explicitly labeled breakglass/unsupported.
- Any degraded-but-supported path that remains is a compatibility wrapper
  around a Substrate-owned command surface, not a raw direct guest procedure.
- Troubleshooting sections make clear when a user should escalate from owned commands to breakglass commands.
- A reviewer can read the macOS docs and understand both the hardened default and the remaining same-user limitation without external context.

## Validation / evidence plan

- Perform a doc inventory of direct guest commands in `docs/WORLD.md` and `docs/cross-platform/mac_world_setup.md` and classify each one.
- Review script help output and inline messaging to ensure it matches the new operator contract.
- Re-run the documented happy path using only Substrate-owned commands and verify that the required evidence can be gathered without breakglass access.
- Verify that any remaining breakglass guidance is concrete, bounded, and clearly separated from the normal path.

## Risks / open questions

- Transitional periods are easy to misdocument; helper scripts can silently remain the de facto primary workflow unless the docs are explicit.
- Some advanced troubleshooting paths may still need raw guest commands that are too useful to hide; the breakglass policy must stay practical.
- The docs must avoid overstating the hardening result: Phase 3 improves operational ownership, but it does not eliminate the same-user Lima trust boundary gap.
