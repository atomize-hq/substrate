# PLAN-12: Breakglass Reclassification and Doc Cutover

Source spec:
- [`SPEC-12-breakglass-reclassification-and-doc-cutover.md`](./SPEC-12-breakglass-reclassification-and-doc-cutover.md)

Source phase authority:
- [`../phase-3-substrate-owned-operations/README.md`](../phase-3-substrate-owned-operations/README.md)
- [`../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md`](../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)

Prior slice authority:
- [`SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`](./SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md)
- [`PLAN-11.md`](./PLAN-11.md)
- [`TASKS-11.md`](./TASKS-11.md)

Plan type: repo-first phase-3 docs-cutover slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `12`: finish the macOS operator cutover by
reclassifying retained guest-admin and bypass flows as breakglass/advanced
material and rewriting the default docs path to lead with the already-landed
Slice `11` owned command matrix.

This plan should produce a bounded landing that:

1. inventories every remaining direct guest or bypass instruction in the main
   macOS docs,
2. freezes a reviewable supported/degraded/breakglass classification for those
   instructions,
3. rewrites the primary setup/runtime/troubleshooting narrative around the
   owned command matrix,
4. aligns helper/help text with that narrative where needed,
5. preserves the same-user limitation and the Slice `11` sync/copy truth,
6. closes Phase `3` without widening into a new CLI redesign.

## Packet 1 source gate, inventory, and classification freeze

Packet `1` should freeze the exact docs-cutover direction before broad rewrites
start.

Targeted official-source verification completed on 2026-06-14 using the
required Lima docs:

1. Lima’s `limactl shell` reference documents that the command uses the host’s
   SSH executable to connect to the instance, so it remains a guest-access
   primitive rather than a Substrate-owned operator surface.
2. Lima’s SSH docs document plain SSH as an interoperability alternative to
   `limactl shell`, reinforcing that raw SSH belongs in the same breakglass
   guest-access class here.
3. Lima’s Filesystem mounts docs continue to treat host visibility inside the
   guest as explicit mount configuration, reinforcing that the cutover should
   not imply broad host-mounted access as the default operator story after the
   staged-ingress work already landed.

Live 2026-06-14 repo-truth confirmation for Slice `12`:

1. `docs/USAGE.md` now reflects the Slice `11` operator matrix and already
   states that `substrate workspace sync` is not the frozen normal macOS
   sync/copy contract in this feature packet.
2. `scripts/mac/lima-doctor.sh` and `scripts/mac/lima-warm.sh` already state
   supported/degraded/breakglass classifications explicitly.
3. `scripts/mac/smoke.sh` already suppresses `SUBSTRATE_WORLD_SOCKET` during
   routed proof and treats it as advanced/test/breakglass on macOS.
4. `docs/reference/world/platforms/macos-lima-setup.md` and `docs/WORLD.md`
   still contain the main remaining drift: they preserve too much direct guest
   procedure or advanced material in the default operator narrative.
5. `docs/reference/world/platforms/macos-lima-setup.md` still includes direct
   guest `systemctl`, guest socket `curl`, guest `journalctl`, and raw
   `limactl shell` sequences, but its routed readiness proof already leads with
   `substrate host doctor`, `substrate world doctor`, and
   `substrate world gateway sync|status|restart`.
6. `docs/WORLD.md` still retains guest-level diagnosis and bypass wording that
   Packet `2` must cut over, but it already treats `substrate world doctor`,
   `substrate host doctor`, and gateway status as the primary routed evidence
   wall.

Frozen Packet `1` implementation direction:

1. Slice `12` owns the broad docs/setup/troubleshooting cutover.
2. Slice `12` consumes the Slice `11` operator matrix as fixed input instead of
   reopening that seam.
3. The default docs path must lead with:
   - `substrate host doctor`
   - `substrate world doctor`
   - `substrate world gateway sync|status|restart`
   - `substrate world enable`
   - `substrate world deps current sync` where dependency reconciliation is the
     concern
4. Packet `1` keeps the supported gateway command inventory explicit, but the
   macOS verification/evidence wall remains routed-first: bare repo-root
   `target/debug/substrate world gateway sync|status|restart` invocations are
   not frozen as unconditional proof, while `scripts/mac/lima-doctor.sh` plus
   fixture-backed `scripts/mac/smoke.sh --gateway-conformance` and
   `scripts/mac/smoke.sh` remain the primary macOS gateway evidence path.
5. `substrate workspace sync` must remain explicitly outside the frozen normal
   macOS sync/copy contract unless live repo truth changes and the same slice
   proves that change.
6. The degraded-but-supported transitional story remains
   `scripts/mac/lima-doctor.sh` as a wrapper around the routed doctor contract
   plus the existing helper-backed `scripts/mac/lima-warm.sh` lifecycle flow
   and current staged-workspace copy direction already carried by Slice `11`,
   not a new public lifecycle or sync family.
7. Raw `limactl shell`, plain SSH, direct guest `systemctl`, direct guest
   socket `curl`, direct guest `journalctl`, and host-side
   `SUBSTRATE_WORLD_SOCKET` override use remain breakglass/advanced material.
8. Helper wording may be aligned where needed, but the slice should not turn
   into a new command-family implementation effort.

## Packet 2 primary docs cutover

Packet `2` rewrites the main operator narrative around the frozen owned path.

Primary responsibilities:

1. rewrite `docs/reference/world/platforms/macos-lima-setup.md` so the happy
   path leads with Substrate-owned commands,
2. rewrite the relevant macOS sections of `docs/WORLD.md` so supported and
   breakglass flows are clearly separated,
3. preserve links or references to `docs/USAGE.md`,
   `docs/contracts/gateway/operator-contract.md`, and
   `docs/contracts/gateway/status-schema.md` as the authoritative command and
   machine-readable contract surfaces,
4. keep the same-user limitation explicit instead of overstating the hardening
   result.

Likely touched files:

1. `docs/reference/world/platforms/macos-lima-setup.md`
2. `docs/WORLD.md`

Packet `2` should review, but not edit by default:

1. `docs/USAGE.md`
2. `docs/contracts/gateway/operator-contract.md`
3. `docs/contracts/gateway/status-schema.md`

## Packet 3 helper/help-text alignment and escalation cleanup

Packet `3` aligns the supporting helper surfaces with the rewritten docs
contract.

Primary responsibilities:

1. ensure helper/help text does not imply that helper or guest-admin flows are
   the primary happy path,
2. ensure breakglass escalation wording is explicit and consistent across the
   scripts and docs,
3. align any remaining references to `SUBSTRATE_WORLD_SOCKET` with the
   advanced/test/breakglass classification on macOS,
4. preserve the truthful Slice `11` sync/copy wording unless live proof
   justifies anything stronger.

Likely touched files:

1. `scripts/mac/lima-doctor.sh`
2. `scripts/mac/lima-warm.sh`
3. `scripts/mac/smoke.sh`

Packet `3` should stop and re-scope if it starts turning into:

1. a new lifecycle command family,
2. a reopened ingress or sync design,
3. a cross-platform documentation cleanup campaign,
4. a broad runtime/transport implementation slice rather than docs-contract
   alignment.

## Packet 4 final verification and Phase 3 closeout handoff

Packet `4` closes the feature-local track honestly.

Primary responsibilities:

1. confirm the docs lead with the owned path and classify retained guest-admin
   material explicitly,
2. confirm the helper/help text no longer displaces the owned path,
3. confirm the verification/evidence wall remains routed-first,
4. record any residual drift explicitly instead of silently calling the cutover
   complete,
5. leave an honest final handoff note for any post-Phase-3 work that is truly
   outside this feature-local slice map.

## Default landing boundary

Unless execution proves a small additional assist is mandatory, this slice
should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
4. `docs/reference/world/platforms/macos-lima-setup.md`
5. `docs/WORLD.md`
6. `scripts/mac/lima-doctor.sh`
7. `scripts/mac/lima-warm.sh`
8. `scripts/mac/smoke.sh`

This slice should review, but not edit by default:

1. `docs/USAGE.md`
2. `docs/contracts/gateway/operator-contract.md`
3. `docs/contracts/gateway/status-schema.md`

By default this slice should **not** widen into:

1. a new CLI-owned lifecycle/sync command family,
2. a reopened Slice `09`, `10`, or `11` design decision,
3. unrelated Linux or WSL docs or runtime work,
4. backend or transport redesign outside the macOS docs-cutover seam.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is required in a targeted form.

Plan consequence:

1. use live repo truth as the primary authority,
2. use official Lima docs where classification depends on guest-access or mount
   semantics,
3. stop if the work starts turning into a new runtime or command-family slice
   instead of the final docs cutover.

## Official source set this plan must use

The plan assumes the resulting slice cites these official docs when they drive
decisions:

1. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
2. [Lima SSH usage](https://lima-vm.io/docs/usage/ssh/)
3. [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)

## Major components and dependencies

1. **source gate and live inventory**
   - confirm the retained direct guest and bypass instructions
   - freeze the classification matrix for the cutover
2. **primary docs cutover**
   - rewrite setup/runtime/troubleshooting flow around the owned path
3. **helper/help-text alignment**
   - keep scripts and docs consistent about escalation and breakglass
4. **evidence wall**
   - preserve routed doctor/gateway/smoke/orchestration proof as the primary
     validation path
5. **phase closeout**
   - record any remaining residual drift honestly

Dependency order:

1. source gate and classification freeze first,
2. primary docs cutover second,
3. helper/help-text alignment third,
4. final verification and closeout last.

## Locked decisions

### What this slice changes

1. It performs the broad macOS docs and breakglass cutover deferred by Slice
   `11`.
2. It makes the supported/degraded/breakglass narrative explicit across the
   main operator docs.
3. It aligns helper/help text with the owned-path-first contract.
4. It closes the feature-local Phase `3` narrative seam.

### What this slice does not change

1. It does not invent a new public CLI family.
2. It does not reopen the normal sync/copy contract unless live repo truth
   forces that correction.
3. It does not silently widen into transport, unit, or ingress redesign.
4. It does not claim that same-user Lima now has Linux-equivalent ownership
   isolation.
