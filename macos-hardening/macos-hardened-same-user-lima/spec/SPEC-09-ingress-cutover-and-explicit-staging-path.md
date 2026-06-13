# Spec: Slice 09 Ingress Cutover and Explicit Staging Path

Source phase authority:
- [`../phase-2-same-user-hardening/README.md`](../phase-2-same-user-hardening/README.md)
- [`../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md`](../phase-2-same-user-hardening/milestone-2-2-mount-minimization-and-ingress-contract-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-ingress-and-mount-contract.md`](./design/DESIGN-macos-ingress-and-mount-contract.md)
- [`design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`](./design/DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md)
- [`design/DESIGN-macos-guest-unit-source-of-truth.md`](./design/DESIGN-macos-guest-unit-source-of-truth.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Neighboring slice authority:
- [`SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`](./SPEC-08-ingress-inventory-and-narrowed-mount-contract.md)
- [`PLAN-08.md`](./PLAN-08.md)
- [`TASKS-08.md`](./TASKS-08.md)
- [`../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md`](../phase-2-same-user-hardening/milestone-2-3-guest-service-sandbox-and-unit-unification-sow.md)
- [`../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md`](../phase-3-substrate-owned-operations/milestone-3-1-substrate-managed-diagnostics-and-lifecycle-sow.md)

Required official source set for this slice:
- [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)
- [Lima VM types](https://lima-vm.io/docs/config/vmtype/)
- [Lima FAQ](https://lima-vm.io/docs/faq/)
- [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)
- [Lima `limactl copy`](https://lima-vm.io/docs/reference/limactl_copy/)
- [Lima `limactl create`](https://lima-vm.io/docs/reference/limactl_create/)
- [Apple `VZVirtioFileSystemDevice`](https://developer.apple.com/documentation/virtualization/vzvirtiofilesystemdevice)

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: implement the Slice `08` narrowed ingress contract by removing
broad default host-home visibility and replacing mounted-project default
behavior with an explicit, validated host-to-guest staging path that keeps the
same-user Lima warm, smoke, routed doctor, and routed gateway lifecycle proofs
green.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `08`, and Slice `09` is now
   the next honest dependency-ordered seam in the local feature sequence.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `09` owns the actual ingress
   implementation and/or any staged copy/sync path, Slice `10` still owns
   guest-unit source-of-truth plus sandbox unification, Slice `11` still owns
   the broader Substrate-owned lifecycle/diagnostics contract, and Slice `12`
   still owns the broad breakglass/docs cutover.
3. Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md), this slice requires
   both `spec-driven-development` and `source-driven-development` because the
   implementation boundary depends on current Lima mount, VZ, and host↔guest
   copy semantics rather than repo-local wording alone.
4. Live repo truth on 2026-06-12 still shows the concrete ingress posture this
   slice must cut over:
   - `scripts/mac/lima/substrate.yaml` mounts host `$HOME` read-only and mounts
     the selected project path read-write at `/src`,
   - `scripts/mac/lima-warm.sh` still hard-fails when `/src` is missing or
     points at the wrong checkout, uses a sentinel written into the host repo
     to verify checkout identity, installs Linux binaries into the guest via
     `limactl copy` when available, and only falls back to in-guest Cargo
     builds when suitable Linux binaries are absent,
   - `scripts/mac/smoke.sh` still contains a routed proof that writes through
     `(cd /src 2>/dev/null || cd "${REPO_ROOT}")`, which means the current
     supported regression harness still assumes mounted workspace ingress,
   - `docs/WORLD.md` and
     `docs/reference/world/platforms/macos-lima-setup.md` still describe `/src`
     mirroring and in-guest mounted-project build/install flows prominently
     enough that the hardened default ingress story is still false after Slice
     `08` unless this slice updates it.
5. `crates/shell/src/execution/workspace_cmd.rs` already contains an existing
   `workspace sync` surface, but this slice should only reuse or extend that
   surface if live validation proves it is the smallest honest way to satisfy
   the Phase `2` ingress cutover. Slice `09` must not silently widen into the
   broader operator-surface/productization seam that belongs to Slice `11`.
6. The replacement ingress path should stay inside guest-local writable roots
   already frozen by Slice `08`, preferring a child path under
   `/var/lib/substrate` rather than introducing a brand-new guest write root
   that Slice `10` would then have to special-case.
7. Request-provided integrated gateway auth handoff plus managed runtime
   artifacts under `/run/substrate/substrate-gateway-runtime/` remain the
   supported auth/runtime posture and must not regress while workspace ingress
   changes.
8. If a live implementation pass proves that guest-source compilation still
   needs staged source input, that source input should arrive through an
   explicit guest-local stage/copy path. This slice should not silently retain
   broad default project mounts just because guest compilation currently exists.

If any of these are wrong, correct them before execution.

## Packet 1 official-source confirmation and frozen cutover direction

Packet `1` was re-grounded on 2026-06-12 against both the required official
source set and current repo truth before any Slice `09` docs were written.

Official source verification completed on 2026-06-12 using the required Lima
Filesystem mounts, VM types, FAQ, breaking-changes, `limactl copy`, and
`limactl create` docs plus Apple `VZVirtioFileSystemDevice` docs:

1. Lima’s filesystem-mount docs say mount behavior is VM-type and
   version-sensitive and that current macOS VZ guests use `virtiofs`, so the
   hardened implementation cannot treat “currently mounted” as a timeless
   contract.
2. Lima’s VM-type and breaking-changes docs say VZ is the default macOS VM type
   in modern Lima releases and that the default mount behavior changed across
   releases. That makes broad host visibility an explicit configuration choice,
   not a required invariant.
3. Lima’s `limactl copy` docs say Lima supports explicit host↔guest file copy
   with `auto`, `rsync`, and `scp` backends. That provides an official,
   supported primitive for a staged ingress path instead of a permanent mounted
   checkout dependency.
4. Lima’s `limactl create` docs say `--plain` disables mounts, port forwarding,
   containerd, and related conveniences. That reinforces that broad defaults
   are optional conveniences, not hard requirements of the platform contract.
5. Lima’s breaking-changes docs show that default mount posture has changed
   over time (`/tmp/lima` no longer mounted by default, SSH key loading
   defaults changed, VZ became the default on macOS), so this slice should not
   preserve old convenience assumptions without proof.
6. Apple’s `VZVirtioFileSystemDevice` docs describe shared directories as host
   resources explicitly exposed to the guest. That means each retained shared
   path is a deliberate exposure decision.

Live 2026-06-12 repo-truth confirmation for Packet `1`:

1. `scripts/mac/lima/substrate.yaml` still declares exactly two default host
   mounts:
   - `location: "$HOME"` with `writable: false`
   - `location: "$PROJECT"` with `writable: true` and `mountPoint: "/src"`
2. `scripts/mac/lima-warm.sh` still uses `ensure_repo_mount()` plus a host-side
   sentinel written into `${PROJECT_PATH}` to prove that `/src` maps to the
   intended checkout before proceeding.
3. The same warm script already stages Linux binaries into the guest with
   `limactl copy`, which proves the current implementation already relies on an
   explicit copy primitive for some supported assets.
4. The same warm script falls back to `cd /src` and in-guest `cargo build`
   only when suitable Linux binaries are not already available from the host.
   That means the mounted checkout is currently carrying both identity proof and
   optional build-source ingress.
5. `scripts/mac/smoke.sh` still performs a routed filesystem-diff proof by
   writing through `(cd /src 2>/dev/null || cd "${REPO_ROOT}")`, so the
   supported regression harness still bakes in a mounted workspace assumption.
6. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` still teach `/src`
   mirroring and guest-side mounted-project compilation/install as if they were
   normal-path behavior.
7. No live repo-truth evidence currently proves a supported need for the broad
   host-home mount once request-provided auth handoff and guest-local runtime
   artifacts are preserved.

Frozen Packet `1` cutover direction:

1. Slice `09` is the real ingress-cutover seam, not another inventory-only
   seam.
2. Broad read-only host-home visibility must be removed from the hardened
   default implementation unless a later slice proves a narrower explicit
   exception path-by-path.
3. The `/src` default mounted-project contract must stop being the normal-path
   ingress dependency for warm/repair and smoke proof.
4. The replacement ingress must be explicit host→guest staging/copy into
   `/var/lib/substrate/staged-workspace` (or a direct child underneath that
   root if Packet `2` needs per-checkout subdivision), because
   `/var/lib/substrate` is already inside the approved guest-local writable
   set from Slice `08`.
5. Packet `2` should start from script/config-layer primitives already proven
   in repo truth (`limactl copy` plus guest-local validation) rather than
   assuming any new CLI/backend ownership work.
6. This slice may reuse existing repo surfaces such as `limactl copy` and, only
   if live proof supports it, an already-landed `workspace sync` surface, but
   that reuse is escalation-only and must not silently widen into a full Phase
   `3` operator-surface redesign.
7. Request-provided gateway auth handoff and guest-local runtime artifacts stay
   unchanged; this slice changes workspace ingress and mounted host visibility,
   not the gateway auth contract.
8. Slice `10` will consume the final guest-local staging and writable-path
   contract when unifying `ProtectHome=` and `ReadWritePaths=`.

## Packet 2 staged workspace contract and mount-removal boundary

Packet `2` makes the actual Slice `09` target explicit enough to implement and
review without silently absorbing Slice `10` or Slice `11`.

| Current dependency | Current source | Frozen Slice `09` target | Boundary this slice must preserve |
| --- | --- | --- | --- |
| Broad host `$HOME/**` read-only visibility | default Lima mount in `scripts/mac/lima/substrate.yaml` | remove from hardened default | do not reintroduce a narrower replacement unless it is explicitly justified and validated in this slice |
| `/src` checkout identity proof | `ensure_repo_mount()` sentinel in `scripts/mac/lima-warm.sh` | replace with explicit staged workspace identity proof inside a guest-local path | preserve the “guest is using the intended checkout/input” guarantee without requiring a direct mounted checkout |
| `/src` source/build ingress for missing Linux binaries | in-guest `cargo build` from `/src` in `scripts/mac/lima-warm.sh` | replace with explicit staged source/artifact ingress rooted in an approved guest-local writable path | preserve the ability to provision supported binaries without falling back to broad default mounts |
| Mounted-workspace smoke write proof | routed `(cd /src || cd "${REPO_ROOT}")` payload in `scripts/mac/smoke.sh` | replace with a proof that targets the explicit guest-local staged workspace/input path | preserve routed proof priority and filesystem-diff evidence |
| Request-provided integrated auth handoff | `crates/shell/src/builtins/world_gateway.rs` + `crates/world-service/src/gateway_runtime.rs` | preserve unchanged | do not widen workspace-ingress work into auth-surface redesign |
| Guest-local runtime artifacts | `/run/substrate.sock`, `/run/substrate/substrate-gateway-runtime/`, `/var/lib/substrate`, guest `SUBSTRATE_HOME`, `/tmp` | preserve unchanged except where an explicit staged workspace child path is added under an already-approved root | give Slice `10` a precise writable-root handoff rather than a moving target |

Frozen Packet `2` conclusions:

1. The hardened default after Slice `09` should not depend on either a broad
   host-home mount or an always-mounted host checkout at `/src`.
2. The minimum honest replacement is an explicit staged workspace/artifact
   ingress rooted in guest-local writable storage already compatible with the
   Slice `08` contract.
3. The preferred root for that staged ingress is
   `/var/lib/substrate/staged-workspace`, with Packet `2` free to create
   direct children beneath that root for per-checkout or per-run material if
   implementation needs that shape. This keeps the cutover inside the already
   approved guest-local writable set and aligns naturally with Slice `10`’s
   future sandbox unification.
4. Packet `1` evidence does not yet force any new shell/CLI/backend ownership
   surface. The default execution boundary therefore stays script/config/docs
   first, with CLI/backend escalation allowed only if Packet `2` proves the
   warm/smoke cutover cannot remain honest otherwise.
5. Minimal compatibility doc updates that become necessary once `/src` and
   host-home mounts stop being true are in scope for this slice, but the broad
   operator-story rewrite remains deferred.
6. A new user-facing sync UX is not required unless the current script-level
   and/or already-landed repo surfaces prove insufficient. If a broader CLI
   productization is needed, freeze the exact follow-on boundary into Slice
   `11` rather than silently absorbing it here.

## Packet 3 frozen validation surfaces and downstream consumers

Packet `3` freezes what the implementation must prove and which later slices
consume the result.

Frozen Slice `09` validation surfaces:

1. `scripts/mac/lima-warm.sh` must still prove create/start/repair readiness,
   but it must do so without treating a mounted host checkout or broad host-home
   visibility as the default ingress model.
2. `scripts/mac/smoke.sh` must keep the routed gateway lifecycle and routed
   readiness proofs green:
   - `substrate world gateway sync`
   - `substrate world gateway status --json`
   - `substrate world gateway restart`
   - `substrate host doctor --json`
   - `substrate world doctor --json`
3. The smoke harness must also keep a routed filesystem-diff proof green after
   the workspace ingress cutover, but that proof must stop depending on
   `(cd /src ...)` as the normal-path assumption.
4. Guest-direct `limactl shell`, guest `systemctl`, guest `curl`, and guest
   `journalctl` remain breakglass/post-failure evidence rather than the success
   path for this slice.
5. If the implementation reuses an existing `workspace sync` surface, this slice
   must prove that the reused surface actually satisfies the warm/smoke ingress
   contract without widening into broader operator-surface redesign.
6. The preserved guest-local writable/runtime roots later slices must inherit
   remain `/var/lib/substrate`, `/run`, `/run/substrate`,
   `/run/substrate.sock`, `/run/substrate/substrate-gateway-runtime/`,
   `/sys/fs/cgroup`, guest `SUBSTRATE_HOME`, and `/tmp`. Any new staged
   workspace leaf path introduced here must stay inside one of those approved
   roots rather than inventing a new top-level writable root.

Frozen downstream consumers:

1. Slice `10` consumes the final staged-workspace and writable-root contract
   when it unifies guest unit source-of-truth plus sandbox settings such as
   `ProtectHome=` and `ReadWritePaths=`.
2. Slice `11` consumes the ingress result when it decides how much of the
   remaining create/warm/repair or sync behavior should be productized into a
   clearer Substrate-owned operator contract.
3. `docs/WORLD.md` and
   `docs/reference/world/platforms/macos-lima-setup.md` consume this slice
   immediately for minimal truth-telling edits about what is or is not mounted
   by default, but they still defer the broader support-taxonomy and operator
   cutover language to Slice `12`.
4. Slice `12` still owns the broad breakglass/docs cutover once the hardened
   ingress path and the later owned-operations story are stable.
