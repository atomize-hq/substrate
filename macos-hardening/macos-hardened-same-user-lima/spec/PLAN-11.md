# PLAN-11: Substrate-Owned Lifecycle and Diagnostics Contract

Source spec:
- [`SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`](./SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md)

Source phase authority:
- [`../phase-3-substrate-owned-operations/README.md`](../phase-3-substrate-owned-operations/README.md)
- [`../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)
- [`../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md`](../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)

Prior slice authority:
- [`SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`](./SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md)
- [`PLAN-10.md`](./PLAN-10.md)
- [`TASKS-10.md`](./TASKS-10.md)

Plan type: repo-first phase-3 operator-contract slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `11`: freeze one truthful macOS operator contract
around Substrate-owned lifecycle, diagnostics, gateway, and normal ingress/sync
surfaces after Slice `10` landed unit parity.

This plan should produce a bounded landing that:

1. defines a reviewable supported/degraded/breakglass matrix for macOS
   operator-facing actions,
2. elevates already-landed Substrate-owned surfaces as the primary happy path,
3. classifies any remaining helper-backed lifecycle/sync flows honestly instead
   of letting raw guest commands define the contract,
4. keeps doctor JSON, gateway status JSON, smoke, and orchestration evidence
   central,
5. performs only the minimum productization and contract updates required to
   make the operator story coherent,
6. leaves Slice `12` with the explicit broad docs/breakglass cutover boundary.

## Packet 1 source gate, inventory, and command-matrix decision

Packet `1` should freeze the exact operator-contract direction before edits
start.

Targeted official-source verification completed on 2026-06-13 using the
required Lima docs:

1. Lima’s `limactl shell` reference documents that the command uses the host
   SSH client to connect to the instance, which makes it a guest-access
   primitive rather than a Substrate-owned operator surface by itself.
2. Lima’s SSH docs document plain SSH as an interoperability path that can be
   used instead of `limactl shell`, reinforcing that both are guest-access
   mechanisms rather than the repo’s owned lifecycle contract.
3. Lima’s Filesystem mounts docs continue to treat host exposure into the guest
   as explicit mount configuration, which reinforces that Slice `11` should
   consume Slice `09`’s narrowed ingress result and define the normal sync/copy
   path on top of it rather than reopen broad mount assumptions.

Live 2026-06-13 repo-truth confirmation for Slice `11`:

1. `docs/USAGE.md` already exposes `substrate host doctor [--json]`,
   `substrate world doctor [--json]`, `substrate world gateway
   sync|status|restart`, `substrate world enable`, and
   `substrate world deps current sync`.
2. `crates/shell/src/execution/workspace_cmd.rs` already exposes
   `substrate workspace sync` and explicitly requires world enablement plus
   `substrate world doctor`.
3. `scripts/mac/lima-doctor.sh` and `scripts/mac/smoke.sh` already suppress
   `SUBSTRATE_WORLD_SOCKET` during routed proof and keep routed readiness
   evidence primary.
4. `scripts/mac/lima-warm.sh` still owns create/start/repair automation.
5. `docs/reference/world/platforms/macos-lima-setup.md` and parts of
   `docs/WORLD.md` still normalize too much direct guest administration or
   breakglass detail for the hardened default.

Frozen Packet `1` implementation direction:

1. Slice `11` owns the operator-facing command matrix, not the full docs
   rewrite.
2. The supported path must start with already-landed Substrate-owned commands:
   `substrate host doctor`, `substrate world doctor`, and
   `substrate world gateway sync|status|restart`.
3. `substrate world enable` freezes as a `supported` CLI-owned provisioning
   entrypoint even though its current macOS implementation still routes through
   helper logic.
4. `substrate world deps current sync` freezes as a `supported` CLI-owned
   dependency-application surface for applying the effective enabled deps list
   into the world, but it does **not** define the normal workspace sync/copy
   contract by itself.
5. `scripts/mac/lima-doctor.sh` and the direct `scripts/mac/lima-warm.sh`
   lifecycle + staged-workspace copy flow freeze as
   `degraded-but-supported` wrapper-backed surfaces rather than as the primary
   happy path.
6. Raw `limactl shell`, plain SSH guest access, direct guest `systemctl`,
   direct guest socket `curl`, direct guest `journalctl`, and host-side
   `SUBSTRATE_WORLD_SOCKET` override use remain `breakglass`.
7. Packet `1` does **not** freeze `substrate workspace sync` as the normal
   macOS sync/copy path yet. The truthful interim contract remains the explicit
   guest-local staged-workspace flow already owned by `scripts/mac/lima-warm.sh`.
8. Because Lima’s official mount docs keep host exposure guest-visible only via
   explicit mount configuration, Slice `11` should consume Slice `09`’s staged
   ingress result instead of implying that mount-backed guest access is the
   operator contract.
9. Slice `11` must freeze that truthful interim contract now rather than
   pretend the sync path is already fully productized.

## Packet 2 operator matrix and owned-path consolidation

Packet `2` freezes the command matrix and primary operator contract.

Primary responsibilities:

1. define the owned command matrix for lifecycle, diagnostics, gateway, and
   normal ingress/sync actions,
2. decide which flows are `supported`, which are `degraded-but-supported`, and
   which remain `breakglass`,
3. consolidate the normal operator story around existing Substrate surfaces
   instead of around raw guest commands,
4. keep Slice `10` unit parity, Slice `09` staged ingress, and the shared-owner
   runtime story intact.

Likely touched files:

1. `docs/USAGE.md`
2. `docs/contracts/gateway/operator-contract.md`
3. `scripts/mac/lima-doctor.sh`
4. `scripts/mac/lima-warm.sh`
5. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`

Packet `2` should review, but not edit by default:

1. `docs/WORLD.md`
2. `docs/reference/world/platforms/macos-lima-setup.md`
3. `docs/contracts/gateway/status-schema.md`
4. `crates/world-mac-lima/src/lib.rs`

## Packet 3 minimal productization and evidence alignment

Packet `3` makes the operator contract executable and verifiable without
widening into the Slice `12` narrative rewrite.

Primary responsibilities:

1. align helper output, CLI messaging, and evidence instructions with the
   frozen owned/degraded/breakglass matrix,
2. make doctor JSON, gateway status JSON, smoke, and orchestration proof the
   default validation wall,
3. promote a truthful normal ingress/sync path or a truthful degraded
   interim contract,
4. apply only the smallest Rust or script changes required to make the owned
   contract honest.

Likely touched files:

1. `crates/shell/src/builtins/world_enable/`
2. `crates/shell/src/execution/workspace_cmd.rs`
3. `crates/shell/src/execution/platform/macos.rs`
4. `scripts/mac/smoke.sh`
5. `scripts/mac/orchestration-smoke.sh`

Packet `3` should stop and re-scope if it starts turning into:

1. a full setup/troubleshooting doc rewrite,
2. a new broad CLI command family unrelated to the existing macOS operator
   surfaces,
3. a reopened ingress/mount redesign,
4. cross-platform lifecycle productization outside the macOS seam.

## Packet 4 final verification and Slice 12 handoff

Packet `4` closes the slice out honestly.

Primary responsibilities:

1. confirm the owned command matrix matches live behavior,
2. confirm supported/degraded/breakglass labels are explicit and consistent,
3. confirm routed doctor/gateway evidence remains primary,
4. confirm any remaining raw guest or bypass guidance is explicitly deferred to
   Slice `12`,
5. record the exact broad docs/breakglass work still left for Slice `12`.

## Default landing boundary

Unless execution proves a small additional assist is mandatory, this slice
should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
4. `docs/USAGE.md`
5. `docs/contracts/gateway/operator-contract.md`
6. `scripts/mac/lima-doctor.sh`
7. `scripts/mac/lima-warm.sh`
8. `scripts/mac/smoke.sh`
9. `scripts/mac/orchestration-smoke.sh`
10. `crates/shell/src/builtins/world_enable/`
11. `crates/shell/src/execution/workspace_cmd.rs`
12. `crates/shell/src/execution/platform/macos.rs`

This slice should review, but not edit by default:

1. `docs/WORLD.md`
2. `docs/reference/world/platforms/macos-lima-setup.md`
3. `docs/contracts/gateway/status-schema.md`
4. `crates/world-mac-lima/src/lib.rs`

By default this slice should **not** widen into:

1. the full docs/setup/troubleshooting cutover,
2. a broad breakglass reclassification sweep across every macOS doc surface,
3. a reopened guest-unit, listener, or ingress redesign,
4. unrelated Linux or WSL owned-operations work,
5. a brand-new lifecycle command family when bounded productization of existing
   surfaces is sufficient.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is required in a targeted form.

Plan consequence:

1. use live repo truth as the primary authority,
2. use official Lima docs where classification depends on guest-access or
   mount semantics,
3. stop if the work starts turning into the broad Slice `12` docs cutover
   rather than the Slice `11` operator-contract seam.

## Official source set this plan must use

The plan assumes the resulting slice cites these official docs when they drive
decisions:

1. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
2. [Lima SSH usage](https://lima-vm.io/docs/usage/ssh/)
3. [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)

## Major components and dependencies

1. **source gate and live inventory**
   - confirm the already-landed owned surfaces
   - confirm which breakglass surfaces are still over-taught
2. **owned command matrix**
   - freeze the supported/degraded/breakglass classification
   - freeze the primary operator commands
3. **minimal productization**
   - align helper/CLI messaging and any narrow code paths with the frozen
     matrix
4. **evidence wall**
   - keep doctor JSON, gateway status JSON, smoke, and orchestration proof
     central
5. **downstream handoff**
   - leave the full doc and breakglass narrative cutover to Slice `12`

Dependency order:

1. source gate and command-matrix freeze first,
2. owned-path consolidation second,
3. minimal productization and evidence alignment third,
4. final verification and downstream handoff last.

## Locked decisions

### What this slice changes

1. It freezes the macOS operator command matrix.
2. It makes the supported/degraded/breakglass classification explicit.
3. It consolidates the primary lifecycle/diagnostics/gateway/sync story around
   Substrate-owned surfaces.
4. It aligns the minimum helper/CLI/evidence surfaces required to keep that
   contract honest.

### What this slice does not change

1. It does not perform the full Slice `12` docs cutover.
2. It does not reopen the Slice `09` ingress design or Slice `10` unit design.
3. It does not require a new lifecycle command family unless a minimal gap is
   proven unavoidable.
4. It does not silently widen into cross-platform owned-operations redesign.
