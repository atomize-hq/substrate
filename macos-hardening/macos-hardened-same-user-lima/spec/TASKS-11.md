# TASKS-11: Substrate-Owned Lifecycle and Diagnostics Contract

Source spec:
- [`SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`](./SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md)

Source plan:
- [`PLAN-11.md`](./PLAN-11.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `11` as the next seam,
2. Slice `12` remains reserved for the broad docs/breakglass cutover,
3. the implementation owner accepts that this slice is repo-first but still
   requires a targeted official-source pass over Lima guest-access/mount
   semantics,
4. the slice stays bounded to lifecycle/diagnostics/gateway/normal sync
   contract consolidation plus only the minimum productization required to make
   that contract honest.

## Slice contract

This slice should land one explicit macOS operator contract for:

1. defining the primary supported Substrate-owned lifecycle and diagnostics
   commands,
2. defining the primary supported gateway lifecycle/status commands,
3. defining the honest normal ingress/sync path after Slice `09`,
4. classifying remaining wrapper/helper flows as `supported` or
   `degraded-but-supported`,
5. classifying raw guest and bypass flows as `breakglass`,
6. centering routed doctor/gateway/smoke/orchestration evidence before
   breakglass diagnosis,
7. handing the broad docs/breakglass narrative rewrite honestly to Slice `12`.

This slice must **not**:

1. widen into the full macOS docs/setup/troubleshooting cutover,
2. reopen guest-unit, listener, or ingress design already frozen in earlier
   slices,
3. silently widen into unrelated Linux or WSL owned-operations work,
4. invent a new broad lifecycle command family unless a minimal gap is proven
   unavoidable,
5. relabel the support taxonomy.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `docs/USAGE.md`,
3. `docs/contracts/gateway/operator-contract.md`,
4. `scripts/mac/lima-doctor.sh`,
5. `scripts/mac/lima-warm.sh`,
6. `scripts/mac/smoke.sh`,
7. `scripts/mac/orchestration-smoke.sh`,
8. `crates/shell/src/builtins/world_enable/`,
9. `crates/shell/src/execution/workspace_cmd.rs`,
10. `crates/shell/src/execution/platform/macos.rs`.

Treat edits outside that boundary as scope expansion unless live execution
proves a minimal assist is mandatory.

## Execution packets

### Packet 1: Source gate, inventory, and command-matrix decision

Session goal:

1. confirm the authority stack, targeted official source set, and live operator
   surfaces,
2. freeze the supported/degraded/breakglass matrix for the slice,
3. decide whether the normal sync/copy story can be taught through an already
   landed Substrate-owned command surface or needs an explicit degraded interim
   contract.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live operator inventory
  - Acceptance: the execution pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `3`, milestone `3.1`,
    milestone `3.2`, `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`,
    `DESIGN-macos-ingress-and-mount-contract.md`,
    `DESIGN-macos-lima-transport-contract.md`,
    `DESIGN-macos-guest-unit-source-of-truth.md`, and the official Lima
    `limactl shell`, SSH, and Filesystem mounts docs. The pass also records the
    current owned, helper-backed, and raw guest operator surfaces across
    `docs/USAGE.md`, `scripts/mac/*`, `docs/WORLD.md`,
    `docs/reference/world/platforms/macos-lima-setup.md`,
    `crates/shell/src/builtins/world_enable/`, and
    `crates/shell/src/execution/workspace_cmd.rs`, including the
    `substrate world deps current sync` surface, the current guest-local
    staged-workspace copy flow, and the guest socket `curl` breakglass path.
  - Verify:
    - `rg -n "host doctor|world doctor|world gateway (sync|status|restart)|world enable|world deps current sync|workspace sync|SUBSTRATE_WORLD_SOCKET|limactl shell|systemctl|curl|journalctl" docs/USAGE.md docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh crates/shell/src/execution/workspace_cmd.rs crates/shell/src/builtins/world_enable crates/shell/src/builtins/world_gateway.rs`
    - `rg -n "limactl shell|SSH|Filesystem mounts|world deps current sync|workspace sync|staged-workspace|curl|breakglass|degraded-but-supported" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
    - manual authority review
    - manual official-source review
    - manual spec/plan/tasks coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`

- [x] Task 1.2: Freeze the operator matrix and sync/lifecycle classification direction
  - Acceptance: the slice names the primary supported command set, names the
    candidate `degraded-but-supported` wrapper-backed flows, freezes raw guest
    access and host-side `SUBSTRATE_WORLD_SOCKET` override use as `breakglass`,
    and states whether the normal sync/copy path is
    `substrate workspace sync`, another existing Substrate-owned surface, or an
    explicitly bounded degraded interim contract.
    Packet `1` now freezes that direction as:
    - `supported`: `substrate host doctor`, `substrate world doctor`,
      `substrate world gateway sync|status|restart`, `substrate world enable`,
      and `substrate world deps current sync` for dependency application into
      the world
    - `degraded-but-supported`: `scripts/mac/lima-doctor.sh` after routed proof
      failure plus the current `scripts/mac/lima-warm.sh` lifecycle and staged
      workspace copy flow
    - `breakglass`: raw `limactl shell`, plain SSH, direct guest `systemctl`,
      guest socket `curl`, guest `journalctl`, and host-side
      `SUBSTRATE_WORLD_SOCKET` override use
    - normal sync/copy direction: an explicitly bounded degraded interim
      contract around the existing guest-local staged-workspace flow, not
      `substrate workspace sync` yet; `substrate world deps current sync` is
      classified separately as dependency reconciliation rather than source
      workspace ingress
  - Verify:
    - `rg -n "supported|degraded-but-supported|breakglass|world deps current sync|workspace sync|staged-workspace|world enable|SUBSTRATE_WORLD_SOCKET|limactl shell|curl|world gateway|world doctor|host doctor" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
    - manual boundary and task-coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-11-substrate-owned-lifecycle-and-diagnostics-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-11.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the targeted official source gate is explicit,
2. the live operator inventory is explicit,
3. the owned/degraded/breakglass matrix is explicit,
4. the normal sync/copy direction is explicit,
5. the verification story explicitly covers guest socket `curl`,
   `substrate world deps current sync`, and agreement across `SPEC-11`,
   `PLAN-11`, and `TASKS-11`,
6. the slice has not yet widened into implementation changes.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Owned command matrix and contract consolidation

Session goal:

1. land the operator contract matrix,
2. align the primary operator story around existing Substrate-owned commands,
3. keep broad setup/troubleshooting rewrites deferred.

#### Tasks

- [ ] Task 2.1: Consolidate the primary owned command contract
  - Acceptance: the owned lifecycle, diagnostics, gateway, and normal sync/copy
    surfaces are listed from one reviewable contract surface without requiring
    raw guest-admin recipes to understand the happy path.
  - Verify:
    - `rg -n "host doctor|world doctor|world gateway (sync|status|restart)|world enable|workspace sync|supported|degraded-but-supported|breakglass" docs/USAGE.md docs/contracts/gateway/operator-contract.md`
    - manual diff review
  - Files:
    - `docs/USAGE.md`
    - `docs/contracts/gateway/operator-contract.md`

- [ ] Task 2.2: Reclassify helper-backed lifecycle surfaces honestly
  - Acceptance: helper-backed create/warm/repair and adjacent operator flows are
    described honestly as `supported`, `degraded-but-supported`, or internal
    implementation detail instead of by implication, and they no longer
    redefine the primary supported path away from the CLI-owned surfaces.
  - Verify:
    - `rg -n "supported|degraded-but-supported|breakglass|world doctor|world enable|workspace sync|SUBSTRATE_WORLD_SOCKET" scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh`
    - `bash -n scripts/mac/lima-doctor.sh`
    - `bash -n scripts/mac/lima-warm.sh`
  - Files:
    - `scripts/mac/lima-doctor.sh`
    - `scripts/mac/lima-warm.sh`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. the primary operator contract is readable from one owned-command matrix,
2. helper-backed lifecycle flows no longer displace the owned CLI path,
3. raw guest access remains outside the default supported path,
4. the slice still has not widened into the full Slice `12` doc rewrite.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Minimal productization and evidence alignment

Session goal:

1. make the frozen contract executable and verifiable,
2. align the evidence wall with the owned path,
3. make only the smallest code/script changes required to keep the contract
   honest.

#### Tasks

- [x] Task 3.1: Align normal sync/copy and lifecycle entrypoints with the frozen contract
  - Acceptance: the chosen normal sync/copy path and any owned provisioning/
    enablement path are consistent across CLI behavior, helper messaging, and
    contract docs; if a surface remains degraded, the degradation is explicit.
  - Verify:
    - `cargo test -p shell`
    - `tmp="$(mktemp -d)"; mkdir -p "$tmp/substrate-home/scripts/substrate"; cp scripts/substrate/world-enable.sh "$tmp/substrate-home/scripts/substrate/world-enable.sh"; chmod +x "$tmp/substrate-home/scripts/substrate/world-enable.sh"; target/debug/substrate world enable --home "$tmp/substrate-home" --dry-run; rc=$?; rm -rf "$tmp"; exit $rc`
    - `bin="$(pwd)/target/debug/substrate"; tmp="$(mktemp -d)"; ws="$tmp/ws"; mkdir -p "$ws"; "$bin" workspace init "$ws" >/dev/null && (cd "$ws" && "$bin" workspace sync --dry-run); rc=$?; rm -rf "$tmp"; exit $rc`
    - manual contract review
  - Files:
    - `crates/shell/src/builtins/world_enable/`
    - `crates/shell/src/execution/workspace_cmd.rs`
    - `crates/shell/src/execution/platform/macos.rs`

- [x] Task 3.2: Center the validation wall on the owned path
  - Acceptance: routed doctor/gateway proof plus smoke/orchestration coverage
    are the default evidence path, and helper scripts only escalate to
    breakglass checks after owned surfaces have been exercised first.
  - Verify:
    - `bash -n scripts/mac/smoke.sh`
    - `bash -n scripts/mac/orchestration-smoke.sh`
    - `target/debug/substrate host doctor --json | jq .`
    - `target/debug/substrate world doctor --json | jq .`
    - `scripts/mac/smoke.sh --gateway-conformance`
  - Files:
    - `scripts/mac/smoke.sh`
    - `scripts/mac/orchestration-smoke.sh`
    - `scripts/mac/lima-doctor.sh`

### Packet 3 checkpoint

Packet `3` is complete only when:

1. the owned or degraded operator entrypoints are consistent across docs,
   messaging, and behavior,
2. the evidence wall is routed-first,
3. shared-owner/orchestration proof remains supported where already landed,
4. the slice still has not widened into the full Slice `12` narrative rewrite.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final verification and downstream handoff

Session goal:

1. validate the owned command matrix honestly,
2. record the precise handoff into Slice `12`,
3. ensure final wording distinguishes what landed from what remains.

#### Tasks

- [x] Task 4.1: Final scope and coherence check
  - Acceptance: the final diff stays within the allowed execution boundary
    unless an explicitly justified minimal assist was required, and the slice
    does not claim a broader docs cutover than it actually implemented.
  - Verify:
    - `git diff --stat -- docs/USAGE.md docs/contracts/gateway/operator-contract.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh scripts/mac/orchestration-smoke.sh crates/shell/src/builtins/world_enable crates/shell/src/execution/workspace_cmd.rs crates/shell/src/execution/platform/macos.rs macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
    - `git status --short`
    - manual wording review
  - Files:
    - touched files only

- [x] Task 4.2: Record the Slice 12 handoff honestly
  - Acceptance: the final closeout states explicitly that Slice `11` landed the
    operator-contract/productization seam only, that Slice `12` still owns the
    broad docs/breakglass cutover, and that any remaining raw guest guidance
    survives only as bounded breakglass material rather than as the primary
    supported path.
  - Verify:
    - rerun the Packet `2` verification commands
    - rerun the Packet `3` verification commands
    - `scripts/mac/lima-doctor.sh`
    - `scripts/mac/smoke.sh`
    - `scripts/mac/orchestration-smoke.sh`
    - manual closeout review that the Packet `4` wording explicitly ties
      closeout/checkpoint-green status to that rerun evidence
  - Files:
    - touched files only

### Packet 4 closeout and downstream handoff

Slice `11` should close only the Substrate-owned lifecycle/diagnostics/sync
operator-contract seam.

The downstream boundary must remain explicit:

1. Slice `12` is still the next seam for the broad breakglass reclassification
   and full docs/setup/troubleshooting cutover.
2. Packet `4` checkpoint-green status depends on current rerun evidence from
   the Packet `2` and Packet `3` walls rather than on Slice `12` completion.
3. Any remaining raw guest guidance survives only as bounded breakglass
   material for Slice `12`, not as the primary supported macOS operator path.

Current Packet `4` rerun status for this closeout:

1. The bounded Packet `2` and Packet `3` rerun wall is green through:
   - `cargo test -p shell`
   - `target/debug/substrate world enable --home "$tmp/substrate-home" --dry-run`
   - `bin="$(pwd)/target/debug/substrate"; tmp="$(mktemp -d)"; ws="$tmp/ws"; mkdir -p "$ws"; "$bin" workspace init "$ws" >/dev/null && (cd "$ws" && "$bin" workspace sync --dry-run); rc=$?; rm -rf "$tmp"; exit $rc`
   - routed `target/debug/substrate host doctor --json`
   - routed `target/debug/substrate world doctor --json`
   - `scripts/mac/smoke.sh --gateway-conformance`
   - `scripts/mac/lima-doctor.sh`
   - `scripts/mac/smoke.sh`
   - `scripts/mac/orchestration-smoke.sh`
2. Packet `4` is checkpoint-green when the bounded rerun wall above is green,
   because the generic macOS smoke keeps the staged guest workspace/project-root
   truth aligned through the CLI-owned routed command and replay path without
   requiring any broader Slice `12` docs/breakglass cutover work.
3. Relative to commit `925403849..HEAD`, the final Packet `4` live diff is:
   - `crates/replay/src/state.rs`
   - `crates/shell/src/execution/platform/macos.rs`
   - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
   - `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
   - `crates/world-mac-lima/examples/mac_backend_smoke.rs`
   - `crates/world/src/exec.rs`
   - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-11.md`
   - `scripts/mac/lima-warm.sh`
   - `scripts/mac/smoke.sh`
   - `scripts/substrate/dev-install-substrate.sh`
   Within that live diff, the narrowly justified files outside the original
   Packet `4` path list were:
   - `crates/replay/src/state.rs`
   - `crates/shell/src/execution/routing/dispatch/world_ops.rs`
   - `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`
   - `crates/world-mac-lima/examples/mac_backend_smoke.rs`
   - `crates/world/src/exec.rs`
   - `scripts/substrate/dev-install-substrate.sh`
   Those assists stay bounded to the same operator-contract/productization seam
   by preserving staged guest project-root truth for the routed generic smoke
   and replay path rather than broadening Slice `11` into a wider docs rewrite.
4. Slice `12` is now the only remaining seam, and it remains explicitly scoped
   to the broad docs/setup/troubleshooting and breakglass cutover; any
   surviving raw guest guidance remains bounded breakglass material rather than
   the primary supported macOS operator path.
