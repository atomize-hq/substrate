# TASKS-12: Breakglass Reclassification and Doc Cutover

Source spec:
- [`SPEC-12-breakglass-reclassification-and-doc-cutover.md`](./SPEC-12-breakglass-reclassification-and-doc-cutover.md)

Source plan:
- [`PLAN-12.md`](./PLAN-12.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `12` as the final feature-local docs-cutover seam,
2. Slice `11` remains the authority for the operator matrix and sync/copy truth
   unless live repo proof forces a correction,
3. the implementation owner accepts that this slice is repo-first but still
   requires a targeted official-source pass over Lima guest-access and mount
   semantics,
4. the slice stays bounded to docs-cutover, breakglass classification, and
   minimum helper/help-text alignment rather than runtime redesign.

## Slice contract

This slice should land one explicit final macOS same-user Lima docs contract
for:

1. teaching the normal supported path through Substrate-owned commands first,
2. classifying retained helper-backed flows honestly as `supported` or
   `degraded-but-supported`,
3. classifying direct guest and bypass flows as `breakglass`,
4. making escalation from owned-path checks to breakglass diagnostics explicit,
5. preserving routed doctor/gateway/smoke/orchestration evidence as the
   primary validation wall,
6. keeping the same-user limitation explicit in the final narrative.

This slice must **not**:

1. invent a new public lifecycle/sync command family,
2. reopen the Slice `09`, `10`, or `11` design seams without concrete live
   evidence,
3. silently widen into unrelated Linux or WSL docs or runtime work,
4. claim that `substrate workspace sync` is now the normal macOS sync/copy
   contract unless live repo proof in this same slice justifies it,
5. relabel the support taxonomy.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `docs/reference/world/platforms/macos-lima-setup.md`,
3. `docs/WORLD.md`,
4. `scripts/mac/lima-doctor.sh`,
5. `scripts/mac/lima-warm.sh`,
6. `scripts/mac/smoke.sh`.

Treat edits outside that boundary as scope expansion unless live execution
proves a minimal assist is mandatory.

## Execution packets

### Packet 1: Source gate, inventory, and classification freeze

Session goal:

1. confirm the authority stack, targeted official source set, and live docs
   drift,
2. freeze the supported/degraded/breakglass classification for retained
   guest-admin and bypass instructions,
3. confirm the docs-cutover must preserve Slice `11`’s sync/copy truth.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live doc inventory
  - Acceptance: the execution pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `3`, milestone `3.2`,
    `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`,
    `DESIGN-macos-ingress-and-mount-contract.md`,
    `DESIGN-macos-lima-transport-contract.md`, and the official Lima
    `limactl shell`, SSH, and Filesystem mounts docs. The pass also records the
    current supported, degraded, and breakglass language across
    `docs/reference/world/platforms/macos-lima-setup.md`, `docs/WORLD.md`,
    `docs/USAGE.md`, `scripts/mac/lima-doctor.sh`, `scripts/mac/lima-warm.sh`,
    and `scripts/mac/smoke.sh`, including that `docs/USAGE.md` already carries
    the Slice `11` owned command matrix, the main remaining drift lives in
    `docs/reference/world/platforms/macos-lima-setup.md` and `docs/WORLD.md`,
    and `scripts/mac/smoke.sh` still classifies `SUBSTRATE_WORLD_SOCKET` as
    advanced/test/breakglass on macOS.
  - Verify:
    - `rg -n "limactl shell|SSH|systemctl|journalctl|curl --unix-socket|SUBSTRATE_WORLD_SOCKET|host doctor|world doctor|world gateway|world enable|world deps current sync|workspace sync|supported|degraded-but-supported|breakglass" docs/reference/world/platforms/macos-lima-setup.md docs/WORLD.md docs/USAGE.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh`
    - `rg -n "limactl shell|SSH|Filesystem mounts|workspace sync|breakglass|degraded-but-supported|SUBSTRATE_WORLD_SOCKET" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
    - manual authority review
    - manual official-source review
    - manual spec/plan/tasks coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`

- [x] Task 1.2: Freeze the docs-cutover classification direction
  - Acceptance: the slice explicitly freezes that the default docs path leads
    with the Slice `11` owned command matrix, that raw `limactl shell`, plain
    SSH, direct guest `systemctl`, guest socket `curl`, guest `journalctl`, and
    host-side `SUBSTRATE_WORLD_SOCKET` override use remain breakglass/advanced,
    and that `substrate workspace sync` is still not the frozen normal macOS
    sync/copy contract unless live repo proof changes.
    Packet `1` now freezes that direction as:
    - supported default docs path: `substrate host doctor`,
      `substrate world doctor`, `substrate world gateway sync|status|restart`,
      `substrate world enable`, and `substrate world deps current sync` where
      dependency reconciliation is the concern
    - degraded-but-supported transitional material: the existing helper-backed
      `scripts/mac/lima-warm.sh` lifecycle path plus the current staged-workspace
      copy direction carried forward from Slice `11`
    - breakglass / advanced material: raw `limactl shell`, plain SSH, direct
      guest `systemctl`, guest socket `curl`, guest `journalctl`, and host-side
      `SUBSTRATE_WORLD_SOCKET` override use
    - sync/copy truth preserved: `substrate workspace sync` remains outside the
      frozen normal macOS same-user Lima sync/copy contract in Packet `1`
    - packet boundary: broad docs rewrites remain deferred to Packet `2`
  - Verify:
    - `rg -n "host doctor|world doctor|world gateway|world enable|world deps current sync|workspace sync|limactl shell|SSH|systemctl|journalctl|SUBSTRATE_WORLD_SOCKET|supported|degraded-but-supported|breakglass" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
    - manual boundary and task-coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-12-breakglass-reclassification-and-doc-cutover.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-12.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the targeted official source gate is explicit,
2. the live docs drift inventory is explicit,
3. the supported/degraded/breakglass cutover direction is explicit,
4. the Slice `11` sync/copy truth is explicitly preserved,
5. the verification story explicitly covers agreement across `SPEC-12`,
   `PLAN-12`, and `TASKS-12`,
6. the slice has not yet widened into broad doc edits.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Primary docs cutover

Session goal:

1. rewrite the main docs to lead with the owned path,
2. separate breakglass material from the happy path,
3. preserve same-user limitation wording.

#### Tasks

- [ ] Task 2.1: Cut over `macos-lima-setup.md` to the owned-path-first narrative
  - Acceptance: the setup and troubleshooting flow leads with owned
    Substrate commands and clearly marks any remaining guest-admin instructions
    as breakglass or advanced escalation material.
  - Verify:
    - `rg -n "host doctor|world doctor|world gateway|world enable|world deps current sync|limactl shell|systemctl|journalctl|curl --unix-socket|breakglass|degraded-but-supported" docs/reference/world/platforms/macos-lima-setup.md`
    - manual diff review
  - Files:
    - `docs/reference/world/platforms/macos-lima-setup.md`

- [ ] Task 2.2: Cut over the macOS sections of `docs/WORLD.md`
  - Acceptance: the runtime/operator narrative leads with the supported path,
    keeps `SUBSTRATE_WORLD_SOCKET` advanced/test/breakglass on macOS, and moves
    guest-admin guidance into clearly bounded escalation wording.
  - Verify:
    - `rg -n "host doctor|world doctor|world gateway|SUBSTRATE_WORLD_SOCKET|limactl shell|journalctl|systemctl|breakglass|degraded-but-supported" docs/WORLD.md`
    - manual diff review
  - Files:
    - `docs/WORLD.md`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. the primary docs lead with the Slice `11` owned path,
2. breakglass material is no longer mixed into the default happy path,
3. same-user limitation wording remains explicit,
4. the slice still has not widened into helper/code alignment work beyond what
   Packet `2` needs to state the contract honestly.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Helper/help-text alignment and escalation cleanup

Session goal:

1. align helper messaging with the rewritten docs contract,
2. keep escalation wording consistent,
3. make only the smallest wording/help changes required to avoid contradiction.

#### Tasks

- [ ] Task 3.1: Align helper messaging with the owned-path-first docs contract
  - Acceptance: helper/help text no longer implies helper-backed or guest-admin
    flows are the primary path, and any retained helper role is described as
    `supported`, `degraded-but-supported`, or breakglass consistently with the
    docs.
  - Verify:
    - `rg -n "supported|degraded-but-supported|breakglass|host doctor|world doctor|world gateway|world enable|world deps current sync|workspace sync|SUBSTRATE_WORLD_SOCKET" scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh`
    - `bash -n scripts/mac/lima-doctor.sh`
    - `bash -n scripts/mac/lima-warm.sh`
    - `bash -n scripts/mac/smoke.sh`
  - Files:
    - `scripts/mac/lima-doctor.sh`
    - `scripts/mac/lima-warm.sh`
    - `scripts/mac/smoke.sh`

- [ ] Task 3.2: Preserve the evidence wall and truthful escalation path
  - Acceptance: routed doctor/gateway/smoke/orchestration proof remains the
    default validation wall, and helper messaging escalates to breakglass only
    after owned-path checks or in explicitly marked deep-diagnosis flows.
  - Verify:
    - `target/debug/substrate host doctor --json | jq .`
    - `target/debug/substrate world doctor --json | jq .`
    - `target/debug/substrate world gateway status --json | jq .`
    - `scripts/mac/lima-doctor.sh`
    - `scripts/mac/smoke.sh --gateway-conformance`
  - Files:
    - touched files only

### Packet 3 checkpoint

Packet `3` is complete only when:

1. docs and helper/help text agree on the owned-path-first contract,
2. breakglass escalation is explicit and consistent,
3. the validation wall remains routed-first,
4. the slice still has not widened into a new runtime or command-family seam.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final verification and Phase 3 closeout

Session goal:

1. validate the docs cutover honestly,
2. confirm residual drift, if any, is explicit,
3. close Phase `3` without overstating the hardening result.

#### Tasks

- [ ] Task 4.1: Final scope and coherence check
  - Acceptance: the final diff stays within the allowed execution boundary
    unless an explicitly justified minimal assist was required, and the slice
    does not claim stronger lifecycle, sync, or ownership guarantees than the
    repo actually provides.
  - Verify:
    - `git diff --stat -- docs/reference/world/platforms/macos-lima-setup.md docs/WORLD.md scripts/mac/lima-doctor.sh scripts/mac/lima-warm.sh scripts/mac/smoke.sh macos-hardening/macos-hardened-same-user-lima/spec/TASKS-12.md`
    - `git status --short`
    - manual wording review
  - Files:
    - touched files only

- [ ] Task 4.2: Record the final Phase 3 closeout state honestly
  - Acceptance: final closeout states explicitly whether the docs cutover is
    checkpoint-green, whether any drift remains, and that the same-user
    limitation still exists even if the operator narrative is now cut over.
  - Verify:
    - rerun the Packet `2` verification commands
    - rerun the Packet `3` verification commands
    - manual closeout review
  - Files:
    - touched files only

### Packet 4 closeout and downstream handoff

Slice `12` should close the final feature-local docs-cutover seam.

The downstream boundary must remain explicit:

1. Phase `3` closeout here means the macOS same-user Lima docs now lead with
   the owned operator path and classify retained guest-access material
   honestly.
2. It does **not** mean the same-user trust boundary has become equivalent to
   Linux-owned isolation.
3. Any remaining raw guest guidance that survives after this slice must remain
   explicitly bounded breakglass or advanced material rather than the primary
   supported path.
