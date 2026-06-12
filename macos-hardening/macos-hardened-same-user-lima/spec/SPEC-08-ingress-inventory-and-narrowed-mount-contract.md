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
5. `scripts/mac/smoke.sh` builds its gateway smoke harness under a
   `mktemp -d` fixture root, places `SUBSTRATE_HOME` under that temporary tree,
   and passes Codex auth into routed proof commands via environment variables.
   That is smoke-fixture setup, not proof that supported gateway auth depends
   on guest access to mounted host-home material.
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

## Packet 2 classification matrix and narrowed default decision

Packet `2` makes the intended hardened posture explicit path-by-path instead of
letting the current Lima mounts stand in as the default contract.

| Current guest-visible input | Current source of visibility | Ingress class | Live supported need proven in Packet `1` | Frozen Slice `08` posture | Deferred consumer |
| --- | --- | --- | --- | --- | --- |
| host `$HOME/**` (broad read-only mount) | `scripts/mac/lima/substrate.yaml` default `location: "$HOME"` mount | auth/credential input | no supported gateway/runtime path currently proves a need for ambient host-home credential/config visibility; request-provided integrated auth already covers the supported auth path | **breakglass / not part of the hardened default** until a later slice can justify a narrower retained subpath explicitly | if a real supported auth path appears later, it must be justified path-by-path before Slice `09` or Slice `10` consumes it |
| host `$HOME/**` (manual browsing, guest shell convenience, ad hoc troubleshooting) | same read-only host-home mount | troubleshooting/convenience input | manual guest access may be useful after failure, but Packet `1` found no supported steady-state requirement for broad host-home visibility | **breakglass / not part of the hardened default**; convenience alone is not justification | Slice `12` can update operator/docs cutover language after the default contract is implemented |
| `/src` as checkout identity proof (`ensure_repo_mount`, sentinel check) | writable `location: "$PROJECT"` mount at `/src` | workspace source input | `scripts/mac/lima-warm.sh` still requires `/src` to exist and match the intended checkout before it proceeds | **temporary direct mount only for the currently required checkout-identity proof**; this is a narrow temporary allowance, not blanket approval for a mounted repo workflow | Slice `09` should replace this with a narrower Substrate-managed ingress or equivalently explicit staging flow if possible |
| `/src` as source/build ingress for optional in-guest Cargo builds | same writable `/src` mount | workspace source input | `scripts/mac/lima-warm.sh` still reads `/src/Cargo.lock`, `cd /src`, and builds guest binaries there only when suitable Linux binaries are not already supplied from the host | **future sync/copy or staged-artifact ingress**, not a permanently justified default mount | Slice `09` owns the actual replacement path or the explicit decision to keep a narrowly scoped temporary mount while that replacement lands |
| request-provided integrated auth handoff (no mounted host path) | gateway request payload, validated by `crates/shell/src/builtins/world_gateway.rs` and `crates/world-service/src/gateway_runtime.rs` | auth/credential input | supported gateway lifecycle/runtime already depends on request-provided auth handoff rather than guest reads from mounted host-home material | **supported explicit ingress**; keep auth request-scoped instead of home-mount-scoped | Slice `09` must preserve this narrower auth contract while changing mounts |
| `/run/substrate.sock` and `/run/substrate/substrate-gateway-runtime/` | guest-local runtime artifacts, not host mounts | runtime input | supported world/gateway runtime already centers on these guest-local artifacts | **supported guest-local runtime surface**; these paths do not justify any host-home mount | Slice `10` consumes these exact runtime paths when finalizing guest-unit sandbox policy |

Frozen Packet `2` conclusions:

1. Broad host-home visibility is now explicitly classified twice: it is neither
   a supported auth requirement nor a supported troubleshooting default, and it
   should stop shaping the hardened default contract.
2. No host-home subpath is frozen as part of the hardened default in Slice
   `08`; any future exception must be justified path-by-path instead of
   inherited from Lima convenience behavior.
3. `/src` is no longer treated as one undifferentiated mount. Slice `08`
   freezes only two concrete current needs:
   - checkout identity proof during warm/repair, and
   - optional in-guest build ingress when Linux binaries are not already
     staged from the host.
4. The first `/src` need is only a temporary direct-mount allowance; the
   second is a deferred sync/copy or staged-artifact consumer owned by Slice
   `09`, not a reason to preserve a broad mounted checkout indefinitely.
5. Supported auth/runtime ingress stays anchored in request-provided auth plus
   guest-local runtime paths, which means the hardened contract is already
   narrower than the current `$HOME` mount.

## Packet 3 frozen validation surfaces and downstream consumers

Packet `3` freezes what later ingress minimization must prove and who consumes
that proof, without widening Slice `08` into any real mount rewrite or docs
cutover.

Frozen Packet `3` validation surfaces:

1. `scripts/mac/lima-warm.sh` remains the first same-user Lima proof surface
   for VM create/start/repair and for the currently temporary `/src`
   checkout-identity allowance. Slice `09` may replace the underlying ingress
   mechanic, but it must preserve an equally explicit warm/repair proof that
   the intended checkout or staged workspace input is the one the guest uses.
2. `scripts/mac/smoke.sh` remains the authoritative end-to-end proof harness
   for supported ingress consumers. Later mount minimization must keep the
   routed gateway lifecycle proof green:
   - `substrate world gateway sync`
   - `substrate world gateway status --json`
   - `substrate world gateway restart`
   - `substrate world gateway status --json` after restart
3. The same smoke harness must keep routed readiness diagnostics green:
   - `substrate host doctor --json`
   - `substrate world doctor --json`
   These are supported diagnostics proofs; guest-direct checks remain fallback
   and breakglass evidence rather than the default success path.
4. Guest-direct compatibility and readiness checks (`limactl shell`, guest
   `systemctl`, guest `curl --unix-socket`, guest `journalctl`, and the
   compatibility/readiness helpers in `scripts/mac/smoke.sh`) remain
   post-failure or breakglass-only evidence. Slice `09` must not accidentally
   promote them back into the supported default proof path while narrowing
   mounts.
5. Supported runtime artifacts and required guest write paths that later
   validation must continue to preserve are still `/var/lib/substrate`,
   `/run/substrate.sock`, `/run/substrate/substrate-gateway-runtime/`,
   `/sys/fs/cgroup`, and the guest-local `SUBSTRATE_HOME`/`/tmp` flow already
   described by `docs/WORLD.md` and the warm script. Later ingress work may
   change how workspace input arrives, but it must not break these guest-local
   runtime/sandbox surfaces.

Frozen Packet `3` downstream consumers:

1. `docs/WORLD.md` is a downstream consumer because it still teaches the
   current same-user Lima proof order and still mentions `/src` mirroring.
   Slice `08` records that this doc must be updated after Slice `09` lands the
   real ingress change, but does not perform that cutover here.
2. `docs/reference/world/platforms/macos-lima-setup.md` is another downstream
   consumer because it still documents mounted-project build/install examples
   under `/src` plus the current routed-vs-breakglass diagnostics order. Slice
   `09` must leave this doc with a more honest default-ingress story, but
   Slice `08` only names the dependency.
3. Slice `09` consumes this contract operationally: it may change the actual
   mount profile and/or introduce staged sync/copy ingress, but it must keep
   the warm proof, routed gateway lifecycle proof, and routed diagnostics proof
   green while removing broad `$HOME` visibility from the hardened default.
4. Slice `10` consumes this contract at the guest-unit sandbox seam. When it
   finalizes `ProtectHome=` and `ReadWritePaths=`, it must preserve the
   approved guest-local runtime/write paths (`/var/lib/substrate`,
   `/run/substrate.sock`, `/run/substrate/substrate-gateway-runtime/`,
   `/sys/fs/cgroup`, guest `SUBSTRATE_HOME`, and `/tmp` where already
   required) without re-introducing ambient host-home visibility as a
   supported default.
5. Slice `12` remains the broader docs/breakglass cutover seam: it can update
   operator-facing wording once Slice `09` and Slice `10` make the narrowed
   contract real, but that downstream cutover is not part of Slice `08`.

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
