# PLAN-09: Ingress Cutover and Explicit Staging Path

Source spec:
- [`SPEC-09-ingress-cutover-and-explicit-staging-path.md`](./SPEC-09-ingress-cutover-and-explicit-staging-path.md)

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md`](../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md)
- [`../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`](./SPEC-08-ingress-inventory-and-narrowed-mount-contract.md)
- [`PLAN-08.md`](./PLAN-08.md)
- [`TASKS-08.md`](./TASKS-08.md)

Plan type: source-driven phase-2 ingress-cutover slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `09`: replace the current default same-user Lima
mount posture with an explicit staged ingress path that implements the Slice
`08` narrowed contract without widening into Slice `10` sandbox unification or
Slice `11` operator-surface productization.

This plan should produce a bounded landing that:

1. removes broad host-home visibility from the hardened default,
2. removes the default requirement that the active host checkout be mounted at
   `/src`,
3. replaces the `/src` identity/build/smoke dependencies with explicit staged
   host→guest ingress rooted in an approved guest-local writable path,
4. preserves routed warm, smoke, doctor, and gateway lifecycle proof surfaces,
5. makes only the minimal doc truth updates required by the ingress cutover,
6. leaves Slice `10`, Slice `11`, and Slice `12` with explicit boundaries.

## Packet 1 source gate, staging-root decision, and execution boundary

Packet `1` should freeze the exact implementation shape before edits start.

Official source verification completed on 2026-06-12 using the required Lima
mount/vmtype/FAQ/breaking-change docs, `limactl copy`, `limactl create`, and
Apple `VZVirtioFileSystemDevice` docs:

1. mount behavior is version- and VM-type-sensitive,
2. broad host visibility is an explicit configuration choice rather than a
   platform requirement,
3. `limactl copy` is an officially documented host↔guest staging primitive,
4. `--plain` confirms mounts are convenience features that can be disabled,
5. shared directories are explicit guest exposure decisions.

Live 2026-06-12 repo-truth confirmation for Slice `09`:

1. `scripts/mac/lima/substrate.yaml` still mounts host `$HOME` and host
   `$PROJECT` at `/src`.
2. `scripts/mac/lima-warm.sh` still has three mounted-checkout dependencies:
   checkout identity proof, optional in-guest source builds, and incidental
   workflow assumptions around `/src`.
3. `scripts/mac/smoke.sh` still depends on a routed write proof that begins
   with `(cd /src ...)`.
4. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` still describe `/src`
   mirroring and mounted-project compilation/install as normal-path behavior.
5. `crates/shell/src/execution/workspace_cmd.rs` exists as a possible future
   sync surface, but its fit for this narrow ingress seam is not yet proven.

Frozen Packet `1` implementation decision:

1. the slice should prefer a script- and config-layer cutover first,
2. the replacement ingress should stage workspace/artifact input into
   `/var/lib/substrate/staged-workspace`, with Packet `2` free to create
   direct children under that root if it needs per-checkout subdivision, unless
   live proof forces another already approved writable root,
3. reusing an existing `workspace sync` surface is optional, must be proven,
   and is not the default Packet `2` implementation path,
4. Packet `2` should start with script/config-layer cutover only and should
   escalate into shell/CLI ownership surfaces only if the warm/smoke contract
   cannot stay honest otherwise.

## Packet 2 mount-profile and warm-path cutover

Packet `2` performs the actual mount and warm-path cutover.

Primary responsibilities:

1. remove the broad host-home mount from
   `scripts/mac/lima/substrate.yaml`,
2. remove or stop depending on the default `/src` mounted checkout path,
3. replace `ensure_repo_mount()`-style proof with explicit staged-input proof,
4. preserve host-provided Linux binary staging via `limactl copy`,
5. preserve a supported fallback for guest-source builds when host Linux
   binaries are absent, but route that fallback through explicit staged input
   rather than a default mounted checkout.

Likely touched files:

1. `scripts/mac/lima/substrate.yaml`
2. `scripts/mac/lima-warm.sh`

Packet `2` should review, but not edit by default:

1. `crates/shell/src/execution/workspace_cmd.rs`
2. `crates/shell/src/execution/platform/mod.rs`
3. `crates/world-mac-lima/src/lib.rs`

Escalate into those code surfaces only if the warm/staging implementation
cannot remain honest without a minimal CLI/backend assist.

## Packet 3 smoke and minimal doc-truth cutover

Packet `3` adapts the supported proof surfaces to the new ingress contract.

Primary responsibilities:

1. replace the smoke harness’s `/src`-based routed write proof with a proof that
   targets the explicit staged guest-local path,
2. keep `substrate host doctor --json`, `substrate world doctor --json`, and
   `substrate world gateway sync|status|restart` as the primary supported
   routed proofs,
3. remove or rewrite only the doc statements that become false once broad
   mounts and mounted `/src` are no longer the hardened default,
4. keep broader operator-story/productization edits deferred.

Likely touched files:

1. `scripts/mac/smoke.sh`
2. `docs/WORLD.md`
3. `docs/reference/world/platforms/macos-lima-setup.md`

Packet `3` should not silently widen into:

1. broad Phase `3` owned-lifecycle UX work,
2. a full docs cutover,
3. guest-unit sandbox unification,
4. unrelated transport, gateway, or policy redesign.

## Packet 4 final validation and downstream handoff

Packet `4` closes the slice out honestly.

Primary responsibilities:

1. confirm the hardened default no longer depends on broad host-home visibility
   or an always-mounted `/src` checkout,
2. confirm the staged ingress path is explicit, validated, and rooted in an
   approved guest-local writable path,
3. confirm warm, smoke, routed doctor, and routed gateway proofs remain green,
4. record the exact staged-workspace/writable-root contract Slice `10` must
   consume,
5. record any residual operator-surface/productization gaps Slice `11` must
   consume,
6. state clearly what docs were minimally corrected now versus what remains for
   Slice `12`.

## Default landing boundary

Unless execution proves a small CLI/backend assist is mandatory, this slice
should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md`
4. `scripts/mac/lima/substrate.yaml`
5. `scripts/mac/lima-warm.sh`
6. `scripts/mac/smoke.sh`
7. `docs/WORLD.md`
8. `docs/reference/world/platforms/macos-lima-setup.md`

This slice should review, but not edit by default:

1. `crates/shell/src/execution/workspace_cmd.rs`
2. `crates/shell/src/execution/platform/mod.rs`
3. `crates/world-mac-lima/src/lib.rs`
4. `crates/shell/src/builtins/world_gateway.rs`
5. `crates/world-service/src/gateway_runtime.rs`

By default this slice should **not** widen into:

1. guest-unit source-of-truth or `ProtectHome=` / `ReadWritePaths=`
   unification,
2. a new broad CLI productization effort for lifecycle/sync flows,
3. support-taxonomy relabeling,
4. a full breakglass/docs cutover,
5. transport/listener re-architecture already owned by earlier slices.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is also required.

Plan consequence:

1. use live repo truth plus official Lima/Apple docs as co-equal authority,
2. keep the slice centered on actual ingress implementation rather than another
   abstract contract-only pass,
3. stop if the implementation starts demanding a milestone-scale operator UX
   redesign that belongs to Slice `11`.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when
they drive decisions:

1. [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)
2. [Lima VM types](https://lima-vm.io/docs/config/vmtype/)
3. [Lima FAQ](https://lima-vm.io/docs/faq/)
4. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)
5. [Lima `limactl copy`](https://lima-vm.io/docs/reference/limactl_copy/)
6. [Lima `limactl create`](https://lima-vm.io/docs/reference/limactl_create/)
7. [Apple `VZVirtioFileSystemDevice`](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

## Major components and dependencies

1. **mount-profile cutover**
   - remove broad default host visibility
   - remove default mounted-checkout dependence
2. **explicit staging path**
   - define guest-local staged workspace/artifact root
   - preserve checkout identity proof and build-source ingress without `/src`
3. **warm-path convergence**
   - preserve create/start/repair behavior through the new ingress path
4. **smoke and proof convergence**
   - preserve routed filesystem-diff and readiness/gateway proof surfaces
5. **minimal doc truth alignment**
   - remove only the statements made false by the cutover
6. **downstream handoff clarity**
   - give Slice `10`, Slice `11`, and Slice `12` exact boundaries

Dependency order:

1. source gate and staging-root decision first,
2. mount-profile and warm-path cutover second,
3. smoke and minimal doc cutover third,
4. final verification and downstream handoff last.

## Locked decisions

### What this slice changes

1. It implements the narrowed ingress contract frozen in Slice `08`.
2. It removes broad host-home visibility from the hardened default.
3. It removes the default mounted `/src` checkout dependency from warm/smoke
   normal-path behavior.
4. It replaces those dependencies with explicit validated staged ingress rooted
   in an approved guest-local writable path.
5. It performs minimal doc truth corrections required by that cutover.

### What this slice does not change

1. no guest-unit sandbox/source-of-truth unification,
2. no broad operator-surface productization,
3. no support-taxonomy rewrite,
4. no full docs/breakglass cutover,
5. no auth/runtime contract redesign.
