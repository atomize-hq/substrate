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

## Packet 2 frozen classification matrix

Packet `2` converts the Packet `1` inventory into an explicit path-by-path
decision matrix so Slice `09` inherits a real contract instead of vague mount
intuition.

| Input/path | Class | Frozen posture in Slice `08` | Reason the posture is constrained this way |
| --- | --- | --- | --- |
| host `$HOME/**` as ambient credential/config visibility | auth | breakglass / not part of the hardened default | supported gateway auth already uses request-provided integrated auth handoff, so Packet `2` found no supported need for broad mounted host-home credential visibility |
| host `$HOME/**` as manual guest browsing or convenience troubleshooting | troubleshooting | breakglass / not part of the hardened default | currently mounted is not the same as hardened default, and Packet `1` found only post-failure/manual value here |
| `/src` for checkout identity proof during warm/repair | workspace | temporary direct mount allowance | `scripts/mac/lima-warm.sh` still verifies that the intended checkout is mounted at `/src`, so Slice `08` can only narrow this to the concrete identity-proof need, not remove it yet |
| `/src` for optional in-guest source builds when Linux binaries are absent | workspace | future sync/copy or staged-artifact ingress | current repo truth proves this is a real consumer today, but it is an implementation seam for Slice `09`, not a permanent default-mount justification |
| request-provided integrated auth handoff | auth | supported explicit ingress | this is the supported auth contract already enforced by shell/runtime code and should remain narrower than host-home visibility |
| `/run/substrate.sock` and `/run/substrate/substrate-gateway-runtime/` | runtime | supported guest-local runtime surface | these are the runtime artifacts later sandbox work must preserve, but they do not justify any host mount |

Frozen Packet `2` planning consequences:

1. Broad host-home visibility is fully classified and is no longer implicit.
2. No host-home subpath survives as a frozen hardened-default exception in this
   slice.
3. `/src` is decomposed into checkout-identity and optional-build ingress
   rather than preserved as a single convenience blob.
4. Packet `2` still does not authorize any actual mount rewrite or sync/copy
   implementation; it only narrows the contract Slice `09` must implement.

## Frozen Packet 3 validation and downstream-consumer expectations

Packet `3` turns the narrowed contract into an explicit “what must stay green”
and “who must consume this later” handoff without widening into the downstream
edits themselves.

Frozen validation expectations:

1. `scripts/mac/lima-warm.sh` remains the warm/repair proof surface for the
   current temporary `/src` checkout-identity allowance until Slice `09`
   replaces that ingress path with something narrower.
2. `scripts/mac/smoke.sh` remains the authoritative supported proof harness for
   later ingress minimization:
   - routed gateway lifecycle proof via `substrate world gateway sync`,
     `substrate world gateway status --json`, and
     `substrate world gateway restart`
   - routed readiness diagnostics via `substrate host doctor --json` and
     `substrate world doctor --json`
3. Guest-direct diagnostics stay breakglass-only evidence. Later mount work
   must keep the routed proof path green rather than silently shifting support
   back toward `limactl shell`, guest `curl`, guest `journalctl`, or other
   post-failure checks.
4. Guest-local runtime/write paths that later slices must preserve are
   `/var/lib/substrate`, `/run`, `/run/substrate`, `/run/substrate.sock`,
   `/run/substrate/substrate-gateway-runtime/`, `/sys/fs/cgroup`, and the
   guest-local `SUBSTRATE_HOME`/`/tmp` surfaces already wired through the warm
   script and world docs.

Frozen downstream consumers:

1. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` are named doc
   consumers because they still describe mounted-project convenience flows that
   Slice `09` must later cut over to the narrowed default.
2. Slice `09` consumes the contract as the actual mount/sync implementation
   seam and must preserve all warm, smoke, routed gateway, and routed
   diagnostics proofs while removing unsupported ambient ingress.
3. Slice `10` consumes the contract as the guest-unit sandbox seam and must
   preserve the approved guest-local runtime/write paths (`/var/lib/substrate`,
   `/run`, `/run/substrate`, `/run/substrate.sock`,
   `/run/substrate/substrate-gateway-runtime/`, `/sys/fs/cgroup`, guest
   `SUBSTRATE_HOME`, and `/tmp` where already required) when finalizing
   `ProtectHome=` and `ReadWritePaths=`.
4. Slice `12` still owns the broader operator/docs cutover after the narrower
   contract has been implemented for real.

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
2. name the ingress classes this slice must reason about,
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
4. the matrix distinguishes temporary direct-mount allowance from future
   sync/copy work path-by-path.

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

Packet `4` closeout for this plan:

1. Slice `08` lands only the ingress inventory, classification matrix,
   narrowed default contract, validation surfaces, and downstream-consumer
   handoff in the Slice `08` planning docs.
2. Slice `09` remains solely responsible for actual mount narrowing and/or any
   staged sync/copy implementation needed to replace the temporary `/src`
   allowances while keeping the routed proof surfaces green.
3. Slice `10` remains solely responsible for guest-unit source-of-truth and
   sandbox unification against the frozen guest-local runtime/write-path
   contract.
4. Slice `12` remains solely responsible for the broader operator/breakglass
   docs cutover after the implementation slices land.
5. This Packet `4` closeout leaves Slice `09` unblocked without widening Slice
   `08` into implementation.

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
