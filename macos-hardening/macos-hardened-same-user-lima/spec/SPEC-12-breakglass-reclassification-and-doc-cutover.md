# Spec: Slice 12 Breakglass Reclassification and Doc Cutover

Source phase authority:
- [`../phase-3-substrate-owned-operations/README.md`](../phase-3-substrate-owned-operations/README.md)
- [`../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md`](../phase-3-substrate-owned-operations/milestone-3-2-breakglass-reclassification-and-doc-cutover-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)

Neighboring slice authority:
- [`SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`](./SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md)
- [`PLAN-11.md`](./PLAN-11.md)
- [`TASKS-11.md`](./TASKS-11.md)

Required official source set for this slice:
- [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
- [Lima SSH usage](https://lima-vm.io/docs/usage/ssh/)
- [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: complete the Phase `3` macOS operator cutover by reclassifying
remaining guest-admin and bypass flows as `breakglass` or advanced material and
rewriting the default docs narrative to lead with the Slice `11` owned command
matrix, without inventing a new CLI family or reopening earlier runtime design
seams.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. Slice `11` is the latest landed dependency-ordered planning seam and already
   froze the honest supported/degraded/breakglass operator matrix for macOS
   same-user Lima.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `12` is the final numbered slice
   in this feature-local map and owns the broad docs/setup/troubleshooting
   cutover rather than another runtime or backend redesign.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   `spec-driven-development` plus targeted `source-driven-development`, because
   the breakglass classification for `limactl shell`, plain SSH, and mount- or
   guest-access-backed flows still depends on official Lima semantics.
4. Live repo truth on 2026-06-14 still shows the Slice `11` operator matrix
   already reflected in:
   - `docs/USAGE.md`, which now leads with `substrate host doctor`,
     `substrate world doctor`, `substrate world gateway sync|status|restart`,
     `substrate world enable`, and `substrate world deps current sync`,
   - `scripts/mac/lima-doctor.sh` and `scripts/mac/lima-warm.sh`, which now
     state the supported/degraded/breakglass contract explicitly,
   - `scripts/mac/smoke.sh`, which already treats
     `SUBSTRATE_WORLD_SOCKET` as advanced/test/breakglass on macOS and keeps
     routed proof primary.
5. Live repo truth on 2026-06-14 still shows the broad docs cutover is not
   complete:
   - `docs/reference/world/platforms/macos-lima-setup.md` still contains direct
     `limactl shell`, guest `systemctl`, guest socket `curl`, and guest
     `journalctl` sequences, so it remains the primary remaining cutover
     surface,
   - `docs/WORLD.md` is already largely aligned on the routed owned-path-first
     posture and should be treated as a residual wording review surface only if
     Packet `2` finds a concrete contradiction,
   - some helper and troubleshooting language still needs to be normalized so
     the supported path is obvious without reading breakglass sections first.
6. Slice `12` should consume the Slice `11` operator matrix as fixed input
   instead of reopening whether `substrate workspace sync` is the normal macOS
   sync/copy contract; unless live code proves otherwise, the docs must retain
   Slice `11`’s truthful interim stance.
7. The smallest honest implementation is a docs and classification slice: it
   may update helper/help text where needed for consistency, but it should not
   silently widen into a new lifecycle command family, ingress redesign, or
   cross-platform operator overhaul.

If any of these are wrong, correct them before execution.

## Objective

Make the default macOS same-user Lima operator story readable from Substrate
owned commands first, with guest-admin and bypass flows clearly separated as
escalation-only material.

This slice is complete only when a reviewer can answer, without guessing:

1. which macOS setup, lifecycle, diagnostics, gateway, and dependency actions
   are `supported`,
2. which remaining wrapper-backed or transitional flows are
   `degraded-but-supported`,
3. which direct guest or bypass flows are `breakglass`,
4. when the operator should escalate from owned commands to breakglass
   commands,
5. which docs are authoritative for the supported path,
6. what same-user limitation still remains after the cutover.

## Commands and evidence surfaces

Use these as the default command-truth and verification surfaces for this
slice:

```bash
cargo build --workspace
cargo test -p shell
cargo test -p world-mac-lima
target/debug/substrate host doctor --json | jq .
target/debug/substrate world doctor --json | jq .
target/debug/substrate world gateway sync
target/debug/substrate world gateway status --json | jq .
target/debug/substrate world gateway restart
target/debug/substrate world gateway status --json | jq .
target/debug/substrate world deps current sync --dry-run --verbose
scripts/mac/smoke.sh --gateway-conformance
scripts/mac/smoke.sh
scripts/mac/lima-doctor.sh
scripts/mac/orchestration-smoke.sh
```

For macOS gateway lifecycle/status proof, treat the bare repo-root
`target/debug/substrate world gateway sync|status|restart` commands as command
inventory or caller-seeded spot checks, not as the unconditional macOS
verification wall. The primary routed evidence wall remains the owned
`substrate host doctor`, `substrate world doctor`, and gateway
`sync|status|restart` surfaces plus the fixture-backed proof in
`scripts/mac/smoke.sh --gateway-conformance` and `scripts/mac/smoke.sh`,
matching Slice `11`. `scripts/mac/lima-doctor.sh` remains the
degraded-but-supported deeper post-failure helper rather than the primary proof
entry point.

If the implementation only touches docs or helper wording, use the smallest
relevant subset rather than running the entire wall mechanically.

## Repo surface map

Primary repo surfaces for this slice:

1. `docs/reference/world/platforms/macos-lima-setup.md`
   - primary setup, troubleshooting, and breakglass cutover target
2. `docs/WORLD.md`
   - secondary runtime/operator narrative review surface; only needs residual
     wording cleanup if Packet `2` finds a concrete contradiction
3. `docs/USAGE.md`
   - authoritative top-level command inventory that the cutover docs must match
4. `docs/contracts/gateway/operator-contract.md`
   - authoritative gateway operator contract referenced by the cutover docs
5. `docs/contracts/gateway/status-schema.md`
   - authoritative machine-readable gateway posture contract
6. `scripts/mac/lima-doctor.sh`
   - helper diagnostic messaging that must match the docs contract
7. `scripts/mac/lima-warm.sh`
   - helper lifecycle/provisioning messaging that must match the docs contract
8. `scripts/mac/smoke.sh`
   - evidence-path script whose output and comments must preserve routed-first
     support posture
9. `macos-hardening/macos-hardened-same-user-lima/spec/`
   - feature-local planning and closeout authority for the slice

## Project structure

This slice remains documentation- and contract-centered:

1. `docs/`
   - public operator narrative, setup instructions, and contract references
2. `scripts/mac/`
   - helper and validation messaging that must align with the docs contract
3. `macos-hardening/macos-hardened-same-user-lima/spec/`
   - feature-local `SPEC/PLAN/TASKS` execution layer

## Code style and wording conventions

This slice should preserve repo-native wording and classification style:

1. reuse the exact taxonomy labels `supported`, `degraded-but-supported`, and
   `breakglass`,
2. present Substrate-owned commands before helper-backed or guest-admin flows,
3. treat machine-readable surfaces as authoritative where they already exist,
4. keep the same-user trust-boundary limitation explicit,
5. keep `SUBSTRATE_WORLD_SOCKET` on macOS classified as
   advanced/test/breakglass rather than as the normal path.

Representative style:

```text
Primary path: run `substrate host doctor --json`, then
`substrate world doctor --json`, then
`substrate world gateway status --json`.
Escalate to `limactl shell`, guest `systemctl`, or guest socket `curl` only
after routed checks fail or when explicitly doing breakglass diagnosis.
```

## Testing strategy

This is a docs-cutover and classification slice, so verification should
emphasize:

1. command-surface truth
   - docs lead with commands that actually exist and match current CLI/help
2. classification truth
   - every retained guest-admin or bypass flow is explicitly labeled and
     bounded
3. escalation truth
   - breakglass steps appear only after owned-path failure or in explicitly
     marked advanced sections
4. evidence truth
   - doctor JSON, gateway status JSON, and smoke/orchestration proof remain the
     primary validation wall
5. same-user truth
   - the final docs do not overstate the hardening result as Linux-equivalent
     ownership separation

## Boundaries

### Always

1. Start from live repo truth, not stale planning text alone.
2. Preserve the Slice `11` operator matrix unless a concrete live-repo mismatch
   is proven.
3. Lead with supported Substrate-owned commands before helper or guest-admin
   flows.
4. Keep breakglass material concrete, but clearly separated from the default
   path.
5. Keep the same-user limitation explicit in the final narrative.

### Ask first

1. Renaming public CLI commands.
2. Promoting `substrate workspace sync` to the normal macOS sync/copy contract
   if live repo truth still does not justify that claim.
3. Introducing a new lifecycle command family instead of cutting docs over to
   the existing surfaces.
4. Broadening into cross-platform docs or operational redesign outside the
   macOS same-user Lima scope.

### Never

1. Present raw `limactl shell`, plain SSH, direct guest `systemctl`, guest
   socket `curl`, or guest `journalctl` as the default supported happy path.
2. Present host-side `SUBSTRATE_WORLD_SOCKET` override use as the normal
   macOS operator path.
3. Claim Linux-equivalent host ownership isolation for same-user Lima.
4. Reopen the Slice `09` ingress design, Slice `10` unit-source-of-truth seam,
   or Slice `11` operator-matrix seam unless a minimal correction is proven
   mandatory.

## Frozen in this slice

This slice freezes only:

1. the final docs classification of retained macOS helper, guest-admin, and
   bypass flows,
2. the default operator narrative that leads with the owned command matrix,
3. the escalation wording that separates supported operations from breakglass,
4. the minimum helper/help-text alignment required to keep the docs honest,
5. the final Phase `3` feature-local handoff state for this macOS hardening
   track.

## Deferred by design

This slice intentionally does **not** freeze:

1. a new macOS-specific lifecycle command family,
2. a reopened normal sync/copy contract if Slice `11`’s interim truth still
   holds,
3. deeper backend or transport redesign,
4. unrelated Linux or WSL operator-surface changes,
5. a claim that the same-user Lima trust model itself has been eliminated.

## Why this slice exists

Slice `11` made the operator contract explicit, but the repo still teaches too
much of the exception path as routine operator behavior.

Live repo truth now makes Slice `12` the next honest seam:

1. the owned command matrix is already documented in `docs/USAGE.md`,
2. helper scripts already state the new taxonomy explicitly,
3. the main remaining drift is narrative: setup, troubleshooting, and runtime
   docs still expose direct guest procedures too prominently,
4. without a final docs cutover, the hardening story remains operationally
   ambiguous even though the owned surfaces already exist.

## Source-gated classification conclusions

Official-source-backed conclusions for this slice:

1. Lima documents `limactl shell` as an SSH-backed guest-access path, so it
   remains guest access rather than a Substrate-owned operator surface.
2. Lima documents plain SSH as an interoperability alternative to
   `limactl shell`, so plain SSH stays in the same breakglass guest-access
   class for this feature.
3. Lima documents host↔guest visibility through explicit filesystem mount
   configuration, so the docs should not imply that broad host mount exposure
   is the normal operator path after the staged-ingress work already landed.

## Packet 1 live inventory and frozen direction

The Packet `1` source gate and repo review freeze the following direction
before any broad doc rewrites begin:

1. **Supported default operator path**
   - `docs/USAGE.md` already leads with `substrate host doctor`,
     `substrate world doctor`, `substrate world gateway sync|status|restart`,
     `substrate world enable`, and `substrate world deps current sync` where
     dependency reconciliation is the concern.
2. **Degraded-but-supported transitional material**
   - `scripts/mac/lima-doctor.sh` remains a degraded-but-supported wrapper
     around the routed doctor contract, and the current helper-backed
     `scripts/mac/lima-warm.sh` lifecycle path plus staged-workspace copy flow
     remain transitional inputs carried forward from Slice `11`; Packet `1`
     does not promote them into a new public command family or claim that they
     are the final normal sync/copy story.
3. **Breakglass / advanced material**
   - `docs/reference/world/platforms/macos-lima-setup.md` remains the primary
     remaining docs cutover surface because it still retains raw
     `limactl shell`, plain SSH, direct guest `systemctl`, guest socket `curl`,
     guest `journalctl`, and host-side `SUBSTRATE_WORLD_SOCKET` override
     guidance too prominently.
   - `docs/WORLD.md` is already largely aligned on the routed evidence posture
     and should only receive residual wording cleanup if a concrete
     contradiction with that frozen contract is found.
   - Slice `12` freezes those retained direct guest and bypass surfaces as
     breakglass or advanced material rather than supported happy path behavior.
4. **Slice `11` sync/copy truth remains fixed input**
   - `substrate workspace sync` is still not the frozen normal macOS same-user
     Lima sync/copy contract unless this same slice later proves a live repo
     change strong enough to justify that promotion.
5. **Packet boundary**
   - Packet `1` freezes classification and boundary truth only; the broad docs
     cutover itself starts in Packet `2`.

## Success criteria

1. The default macOS docs lead with the supported Substrate-owned command set:
   `substrate host doctor`, `substrate world doctor`,
   `substrate world gateway sync|status|restart`, `substrate world enable`, and
   `substrate world deps current sync` where relevant.
2. `docs/reference/world/platforms/macos-lima-setup.md` and `docs/WORLD.md`
   clearly distinguish supported, degraded-but-supported, and breakglass flows.
3. Every remaining direct guest-admin or bypass instruction is explicitly
   labeled and appears only in clearly marked escalation or deep-diagnosis
   sections.
4. The final docs do not claim that `substrate workspace sync` is the frozen
   normal macOS sync/copy contract unless live repo truth has changed and that
   change is proven in the same slice.
5. `SUBSTRATE_WORLD_SOCKET` override use is documented only as
   advanced/test/breakglass on macOS.
6. A reviewer can follow the normal supported path and gather readiness and
   gateway evidence without needing `limactl shell` first.
