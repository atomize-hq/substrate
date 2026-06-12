# PLAN-08: Ingress Inventory and Narrowed Mount Contract

Source spec:
- [`SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`](./SPEC-08-ingress-inventory-and-narrowed-mount-contract.md)

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md`](../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Prior slice authority:
- [`SPEC-07-remove-default-extra-listener-surface.md`](./SPEC-07-remove-default-extra-listener-surface.md)
- [`PLAN-07.md`](./PLAN-07.md)
- [`TASKS-07.md`](./TASKS-07.md)

Plan type: source-driven phase-2 ingress inventory and narrowed-contract slice
for `macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `08`: inventory the current same-user Lima
ingress surface and freeze the narrowed default mount contract before the repo
starts actual mount-profile or sync/copy implementation work.

This plan should produce a bounded landing that:

1. inventories every default guest-visible host input that currently matters to
   supported same-user Lima operation,
2. classifies those inputs into workspace, auth, runtime, and troubleshooting
   classes,
3. states which classes remain temporarily supported direct mounts, which must
   move to Substrate-managed sync/copy work, and which belong to breakglass,
4. aligns validation and documentation expectations with that narrowed
   contract,
5. leaves actual mount-profile changes and sync/copy implementation explicitly
   deferred to Slice `09`.

## Packet 1 source gate and frozen ingress decision

Packet `1` should freeze the ingress contract before any later mount-minimizing
implementation begins.

Official source verification completed on 2026-06-12 using the required Lima
Filesystem mounts, VM types, FAQ, and breaking-changes docs plus Apple
`VZVirtioFileSystemDevice` docs:

1. Lima filesystem mounts are version- and VM-driver-sensitive.
2. Current macOS VZ guests use `virtiofs` semantics rather than older default
   mount assumptions.
3. Lima’s FAQ confirms host-home visibility is config-driven, not an automatic
   correctness requirement.
4. Lima breaking changes confirm old default-mount assumptions are unsafe.
5. Apple `VZVirtioFileSystemDevice` frames shared host directories as explicit
   guest exposures.

Live 2026-06-12 repo-truth confirmation for Slice `08`:

1. `scripts/mac/lima/substrate.yaml` still mounts host `$HOME` read-only and
   host `$PROJECT` read-write at `/src`.
2. `scripts/mac/lima-warm.sh` still requires `/src` to exist and match the
   intended checkout, but it already copies prebuilt binaries directly when
   they exist.
3. `crates/shell/src/builtins/world_gateway.rs` and
   `crates/world-service/src/gateway_runtime.rs` already center supported
   gateway auth/runtime on request-provided integrated auth handoff plus
   `/run/substrate/substrate-gateway-runtime/`, which is narrower than ambient
   mounted host-home visibility.
4. `scripts/mac/smoke.sh` already treats guest-direct gateway/readiness checks
   as breakglass evidence rather than the primary supported proof path.
5. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` still describe mounted
   project workflows more concretely than the hardened ingress contract.

Frozen Packet `1` decision:

1. Slice `08` is a contract/inventory seam, not the mount-profile rewrite
   seam.
2. Broad host-home visibility must be justified path-by-path or marked for
   removal.
3. `/src` must be broken into concrete supported needs rather than preserved as
   one convenience bucket.
4. Slice `09` owns actual mount and sync/copy implementation; Slice `10` owns
   the unit/sandbox consumer of the final ingress contract.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`

This slice should review, but not edit by default:

1. `scripts/mac/lima/substrate.yaml`
2. `scripts/mac/lima-warm.sh`
3. `scripts/mac/smoke.sh`
4. `docs/WORLD.md`
5. `docs/reference/world/platforms/macos-lima-setup.md`
6. `crates/shell/src/builtins/world_gateway.rs`
7. `crates/world-service/src/gateway_runtime.rs`

By default this slice should **not** widen into:

1. actual mount-profile edits in `scripts/mac/lima/substrate.yaml`,
2. new sync/copy machinery or CLI UX,
3. guest-unit `ProtectHome=` / `ReadWritePaths=` unification,
4. broad Phase `3` lifecycle work,
5. full breakglass/docs cutover.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is also required.

Plan consequence:

1. use live repo truth plus official Lima/Apple docs as co-equal authority,
2. keep the slice centered on contract inventory and decision-making rather
   than implementation mechanics,
3. stop if execution starts requiring real sync/copy or mount-profile changes
   that belong to Slice `09`.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when
they drive decisions:

1. [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)
2. [Lima VM types](https://lima-vm.io/docs/config/vmtype/)
3. [Lima FAQ](https://lima-vm.io/docs/faq/)
4. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)
5. [Apple `VZVirtioFileSystemDevice`](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

## Major components and dependencies

1. **live ingress inventory**
   - list each current guest-visible host input
   - identify which code/docs actually depend on it
2. **ingress classification**
   - classify each input as workspace, auth, runtime, or troubleshooting
   - map each class to supported direct mount, future sync/copy, or breakglass
3. **narrowed contract freeze**
   - write the actual default mount contract in plain language
   - identify what later implementation must preserve
4. **validation and docs expectations**
   - name which proof surfaces later mount minimization must keep green
   - keep later slice boundaries explicit
5. **handoff clarity**
   - leave Slice `09` and Slice `10` with a precise contract to consume

Dependency order:

1. official-source and live-truth inventory first,
2. ingress-class decision matrix second,
3. narrowed contract freeze third,
4. validation/doc expectation freeze fourth,
5. final scope and handoff clarity last.

## Locked decisions

### What this slice changes

1. It creates the authoritative ingress inventory for current same-user Lima
   default behavior.
2. It freezes the narrowed contract for which classes justify direct mount
   versus future sync/copy or breakglass.
3. It names the exact validation expectations later implementation must satisfy.
4. It leaves a precise handoff into Slice `09` and Slice `10`.

### What this slice does not change

1. no actual mount removal yet,
2. no new sync/copy implementation,
3. no guest-unit sandbox rewrite,
4. no broad lifecycle CLI redesign,
5. no broad docs cutover.

## Implementation order

### Packet 1: Freeze the source gate and live ingress inventory

Goal:

1. confirm the authority stack, official source set, and current repo-truth
   mount/ingress posture,
2. freeze the exact ingress classes this slice must reason about,
3. record the concrete surfaces that still rely on broad host-home visibility
   or `/src`.

Primary touch surface:

1. `SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
2. `PLAN-08.md`
3. `TASKS-08.md`

Why first:

1. later narrowing decisions are meaningless unless current ingress is
   explicitly inventoried,
2. Slice `08` should not start proposing contract changes without first naming
   what today’s supported flows actually consume.

Verification checkpoint:

1. official Lima/Apple sources are cited for non-obvious mount claims,
2. the current mount inventory is explicit,
3. the key repo-truth surfaces are named explicitly:
   - `scripts/mac/lima/substrate.yaml`
   - `scripts/mac/lima-warm.sh`
   - `scripts/mac/smoke.sh`
   - `docs/WORLD.md`
   - `docs/reference/world/platforms/macos-lima-setup.md`
   - `crates/shell/src/builtins/world_gateway.rs`
   - `crates/world-service/src/gateway_runtime.rs`

### Packet 2: Classify ingress and freeze the narrowed decision matrix

Goal:

1. classify each current guest-visible input into workspace, auth, runtime, or
   troubleshooting ingress,
2. decide which current mounts remain temporarily justified, which should move
   to future sync/copy, and which become breakglass-only,
3. keep current supported gateway/runtime truth separate from older broad-home
   convenience assumptions.

Why second:

1. the slice’s core value is a contract matrix, not just a file inventory,
2. Slice `09` cannot be scoped honestly until this matrix exists.

Verification checkpoint:

1. every current default ingress path appears in the decision matrix,
2. broad host-home visibility is no longer left unclassified,
3. `/src` is decomposed into actual needs instead of preserved as a single
   convenience blob.

### Packet 3: Freeze validation and documentation expectations

Goal:

1. name the proof surfaces later mount-minimization work must keep green,
2. identify which current docs still describe convenience ingress rather than
   the narrowed contract,
3. define what later Slice `09` work must update when it turns the contract
   into real mount/profile behavior.

Why third:

1. validation expectations should be derived from the narrowed contract, not
   from today’s convenience posture,
2. this packet prevents Slice `09` from landing with silent docs drift.

Verification checkpoint:

1. later required proofs are explicit (`lima-warm`, `smoke`, gateway status,
   routed diagnostics),
2. doc surfaces requiring future cutover are identified without actually
   widening into that cutover,
3. unit-sandbox consumers reserved for Slice `10` are called out plainly.

### Packet 4: Final scope check and next-slice handoff

Goal:

1. confirm Slice `08` stayed contract-scoped,
2. restate the clean handoff into Slice `09` and Slice `10`,
3. ensure the final wording does not accidentally promise mount changes that
   this slice did not make.

Why last:

1. Slice `08` only succeeds if it leaves later implementation seams smaller and
   clearer,
2. honest deferral is part of the deliverable.

Verification checkpoint:

1. Slice `08` remains docs-only,
2. actual mount removal/sync work is still plainly deferred,
3. the next implementation seam is obvious from the final handoff.

## Risks and mitigations

1. **Risk: preserving broad `$HOME` by inertia**
   - Mitigation: require a path-by-path justification matrix rather than vague
     convenience reasoning.
2. **Risk: underestimating `/src` dependencies**
   - Mitigation: inventory warm-script, smoke, and docs consumers explicitly
     before classifying `/src`.
3. **Risk: accidentally pulling Slice `09` implementation into Slice `08`**
   - Mitigation: keep landing boundary docs-only and stop when new sync/copy or
     mount-profile work is required.
4. **Risk: Slice `10` receives an ambiguous contract**
   - Mitigation: call out the exact future sandbox consumers that depend on the
     final ingress decision.
