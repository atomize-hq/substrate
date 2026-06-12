# Spec: Slice 08 Ingress Inventory and Narrowed Mount Contract

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md`](../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)

Neighboring slice authority:
- [`SPEC-07-remove-default-extra-listener-surface.md`](./SPEC-07-remove-default-extra-listener-surface.md)
- [`PLAN-07.md`](./PLAN-07.md)
- [`TASKS-07.md`](./TASKS-07.md)
- [`../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md`](../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)

Required official source set for this slice:
- [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)
- [Lima VM types](https://lima-vm.io/docs/config/vmtype/)
- [Lima FAQ](https://lima-vm.io/docs/faq/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)
- [Apple `VZVirtioFileSystemDevice`](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: freeze the same-user Lima ingress inventory and the narrowed
default mount contract so Phase `2.2` can proceed from a source-backed
classification of guest-visible host inputs instead of the current convenience
mount posture.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `07`, and Slice `08` is now
   the next honest dependency-ordered seam in the local feature sequence.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `08` owns ingress inventory and
   narrowed mount-contract decisions, Slice `09` still owns actual ingress
   implementation and/or any Substrate-managed sync/copy path, Slice `10`
   still owns guest-unit source-of-truth plus sandbox unification, and Slice
   `12` still owns the broad breakglass/docs cutover.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because the
   default mount and directory-sharing story is version-sensitive Lima/VZ
   behavior, not just repo-local wording.
4. Live repo truth on 2026-06-12 still shows the concrete ingress posture this
   slice must inventory before any narrowing implementation begins:
   - `scripts/mac/lima/substrate.yaml` mounts host `$HOME` read-only and mounts
     the selected project path read-write at `/src`,
   - `scripts/mac/lima-warm.sh` treats `/src` as required guest-visible repo
     ingress, recreates the VM when `/src` points at the wrong checkout, and
     uses `/src` for optional in-guest Cargo builds when Linux binaries are not
     already provided from the host,
   - `scripts/mac/smoke.sh` already treats guest-direct gateway checks as
     breakglass-only evidence and performs supported gateway lifecycle proof via
     routed `substrate world gateway sync|status|restart`,
   - `crates/shell/src/builtins/world_gateway.rs` and
     `crates/world-service/src/gateway_runtime.rs` already center supported
     gateway auth on request-provided integrated auth handoff plus managed
     runtime artifacts under `/run/substrate/substrate-gateway-runtime/`,
   - `docs/WORLD.md` and
     `docs/reference/world/platforms/macos-lima-setup.md` still describe `/src`
     mirroring and in-guest mounted-project workflows prominently enough that a
     reviewer cannot yet tell which mounted host paths are truly part of the
     hardened default.
5. Slice `07` already removed the default guest TCP listener contradiction, so
   Slice `08` should move to guest-visible filesystem scope rather than reopen
   listener or routed-readiness questions.
6. This slice should freeze the inventory and decision matrix first; if the
   repo needs a new sync/copy primitive or non-trivial mount-profile rewrite to
   realize the contract, that execution belongs to Slice `09`.
7. If a live review proves that a mount can be removed immediately without a
   new sync/copy path, the decision can be recorded here, but the actual mount
   change should still stay explicitly bounded and not silently absorb the full
   Slice `09` implementation seam.

If any of these are wrong, correct them before execution.

## Packet 1 live confirmation and frozen ingress decision

Packet `1` was re-grounded on 2026-06-12 against both the required official
source set and current repo truth before any Slice `08` docs were written.

Official source verification completed on 2026-06-12 using the required Lima
Filesystem mounts, VM types, FAQ, and breaking-changes docs plus Apple
`VZVirtioFileSystemDevice` docs:

1. Lima’s filesystem-mount docs say the default mount type depends on Lima
   version and VM driver, with `virtiofs` used for VZ on macOS in current
   releases. That makes “what is mounted” and “how it is mounted” part of the
   supported contract rather than a timeless convenience detail.
2. Lima’s filesystem-mount docs also say `virtiofs` on macOS requires
   `vmType: vz`, and older/default mount behavior has changed across releases.
   This means Slice `08` must ground the contract in current VZ-based Lima
   behavior, not stale historical defaults.
3. Lima’s FAQ says the home directory is mounted read-only by default and that
   `plain: true` disables mounts and related convenience behavior. That
   reinforces that guest-visible host directories are opt-in configuration
   choices and therefore should be justified path-by-path.
4. Lima’s breaking-changes docs say VZ became the default macOS VM type in
   Lima `1.0` and the default mount type for VZ changed to `virtiofs`; they
   also note `/tmp/lima` is no longer mounted by default. That means the repo
   should not freeze older broad-ingress assumptions into the hardened macOS
   contract.
5. Apple’s `VZVirtioFileSystemDevice` docs describe the shared-directory device
   as exposing host resources to the guest file system. That makes each shared
   host path a deliberate exposure decision, not a free implementation detail.

Live 2026-06-12 repo-truth confirmation for Packet `1`:

1. `scripts/mac/lima/substrate.yaml` still declares exactly two default host
   mounts:
   - `location: "$HOME"` with `writable: false`
   - `location: "$PROJECT"` with `writable: true` and `mountPoint: "/src"`
2. `scripts/mac/lima-warm.sh` still hard-fails when `/src` is missing or
   points at the wrong checkout, which means the supported warm/repair path
   still implicitly treats project direct-mount ingress as default behavior.
3. The same warm script already copies prebuilt guest binaries with
   `limactl copy` when host-built Linux artifacts are available, which means
   not every supported path depends on building from a mounted repo checkout.
4. `scripts/mac/lima-warm.sh` only uses `$HOME/.cargo/env` inside the guest to
   bootstrap optional in-guest builds; that is guest-home usage, not proof that
   supported runtime operation requires a broad host-home mount.
5. `scripts/mac/smoke.sh` writes temporary host auth material under the host
   home directory for the smoke harness, but the actual supported gateway path
   goes through routed CLI commands rather than guest access to the mounted host
   home directory.
6. `crates/shell/src/builtins/world_gateway.rs` still constructs the
   request-provided integrated auth handoff for gateway lifecycle requests,
   which means supported gateway auth is already narrower than ambient mounted
   host-home visibility.
7. `crates/world-service/src/gateway_runtime.rs` rejects gateway startup when
   the request-provided integrated auth handoff is missing, and it stores
   managed runtime artifacts under `/run/substrate/substrate-gateway-runtime/`.
   That means supported gateway runtime already has a repo-truth path that is
   narrower than “the guest can read host `$HOME`”.
8. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` still describe `/src`
   mirroring and manual mounted-project flows prominently enough that the
   hardened ingress contract remains under-specified in user-facing docs.

Frozen Packet `1` ingress decision:

1. Broad read-only host-home visibility is not part of the intended hardened
   default unless each surviving path class can be justified explicitly.
2. `/src` must be decomposed into concrete supported needs rather than treated
   as an undifferentiated “mounted project” convenience bucket.
3. Supported runtime artifacts remain `/run/substrate.sock` and
   `/run/substrate/substrate-gateway-runtime/`; they are runtime surfaces, not
   justification for host-home visibility.
4. Supported gateway auth should continue to prefer request-provided integrated
   handoff over ambient guest visibility into host-home credential material.
5. Slice `08` freezes the inventory and narrowed contract; Slice `09` owns any
   non-trivial mount cutover, sync/copy flow, or implementation needed to make
   the contract real.
6. Slice `10` still consumes the final ingress contract when locking guest-unit
   sandbox settings such as `ProtectHome=` and `ReadWritePaths=`.

## Objective

Make the Phase `2.2` ingress story inspectable before implementation narrowing
begins.

This slice is complete only when a reviewer can answer, without reading the
Lima profile line-by-line:

1. which host-visible paths currently reach the guest by default,
2. which of those paths remain justified for supported operation,
3. which flows should move to Substrate-managed sync/copy rather than remain
   mounts,
4. which flows are breakglass-only and should stop influencing the supported
   contract,
5. which verification surfaces must stay green after later mount minimization,
6. what work remains deferred to Slice `09`, Slice `10`, and Slice `12`.

## Frozen in this slice

This slice freezes only:

1. the current ingress inventory for same-user Lima default operation,
2. the classification matrix for workspace, auth, runtime, and troubleshooting
   ingress classes,
3. the narrowed default contract that states which classes may remain direct
   mounts temporarily, which should become sync/copy work, and which belong to
   breakglass,
4. the validation and documentation expectations that later implementation work
   must satisfy,
5. the explicit separation between this inventory/contract seam and later
   implementation/unit-sandbox seams.

## Deferred by design

This slice intentionally does **not** freeze:

1. the actual Lima profile mount rewrite if it requires new sync/copy
   mechanics,
2. the final Substrate-owned sync or workspace-ingress UX from Slice `09`,
3. guest-unit source-of-truth or sandbox unification from Slice `10`,
4. a broad operator lifecycle redesign from Phase `3`,
5. the full breakglass/docs cutover from Slice `12`,
6. unrelated listener or transport redesign already settled by prior slices.

## Why this slice exists

Phase `2` is only honest if it names the guest-visible host contract as
explicitly as it now names the listener contract.

Live repo truth shows the remaining gap clearly:

1. the current Lima profile still exposes broad host `$HOME` visibility and a
   writable mounted project checkout,
2. some existing paths still depend on `/src`, but not every supported path
   does,
3. supported gateway lifecycle/runtime already has a narrower contract than
   “read host home from the guest,”
4. docs still describe mounted-project workflows more concretely than the
   hardened ingress contract itself,
5. Slice `10` cannot honestly lock service sandbox paths until the ingress
   contract is explicit.

If Slice `08` does not land now, the repo continues Phase `2` with a hidden
filesystem contradiction: the listener surface is narrowed, but the guest-visible
host surface is still wider than the hardening docs can currently justify.

## Source-driven grounding

This slice is source-driven because the mount contract depends on official
Lima/VZ semantics:

1. Lima’s default mount behavior changes across versions and VM drivers, so the
   contract cannot rely on stale historical assumptions.
2. VZ/virtiofs shared directories are explicit host-resource exposures, which
   makes mount entries security-relevant design decisions.
3. Lima documents host-home mount behavior and convenience toggles as config,
   which means the repo must justify each retained mounted class rather than
   treat current YAML as self-validating truth.

Because of that, Slice `08` should keep its conclusions anchored in current
official Lima/Apple documentation plus live repo truth.

## Commands

Start by confirming the authority stack, official source set, and current
ingress inventory.

```bash
# Review the phase-2 authority stack and adjacent slices
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md

# Confirm the current default mounts and where they are consumed
sed -n '1,220p' scripts/mac/lima/substrate.yaml
rg -n "mount|/src|HOME|Cargo.lock|limactl copy|gateway-runtime|auth" \
  scripts/mac/lima-warm.sh \
  scripts/mac/smoke.sh \
  docs/WORLD.md \
  docs/reference/world/platforms/macos-lima-setup.md

# Confirm gateway auth/runtime no longer requires ambient host-home visibility
sed -n '360,460p' crates/shell/src/builtins/world_gateway.rs
sed -n '990,1035p' crates/world-service/src/gateway_runtime.rs
sed -n '1188,1310p' crates/world-service/src/gateway_runtime.rs
```

## Touched repo surfaces

Default Slice `08` landing boundary:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`

Inspection surfaces that this slice must review but should not edit by default:

1. `scripts/mac/lima/substrate.yaml`
2. `scripts/mac/lima-warm.sh`
3. `scripts/mac/smoke.sh`
4. `docs/WORLD.md`
5. `docs/reference/world/platforms/macos-lima-setup.md`
6. `crates/shell/src/builtins/world_gateway.rs`
7. `crates/world-service/src/gateway_runtime.rs`

## Boundaries

### Always

1. Keep Slice `08` narrower than actual mount/sync implementation.
2. Reuse the frozen support taxonomy from Slice `01`.
3. Cite official Lima/Apple sources for non-obvious mount and exposure claims.
4. State clearly which decisions are supported, future sync/copy, or
   breakglass.

### Ask first

1. Pulling actual mount-profile edits from Slice `09` into this slice.
2. Reclassifying a currently supported path as breakglass if live repo truth
   shows an unplanned operator impact.
3. Widening into guest-unit sandbox settings that belong to Slice `10`.

### Never

1. Treat “currently mounted” as equivalent to “hardened default.”
2. Keep broad host-home visibility merely because it is convenient for manual
   debugging.
3. Let Slice `08` silently absorb sync/copy implementation or full docs cutover
   work.

## Success criteria

1. Slice `08` records a source-backed ingress inventory for current default
   same-user Lima operation.
2. Slice `08` names a narrowed default contract for each ingress class.
3. Slice `08` leaves a reviewer with a plain handoff into Slice `09` and Slice
   `10`.
