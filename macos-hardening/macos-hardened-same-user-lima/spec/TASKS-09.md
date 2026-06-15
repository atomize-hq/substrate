# TASKS-09: Ingress Cutover and Explicit Staging Path

Source spec:
- [`SPEC-09-ingress-cutover-and-explicit-staging-path.md`](./SPEC-09-ingress-cutover-and-explicit-staging-path.md)

Source plan:
- [`PLAN-09.md`](./PLAN-09.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `09` as the next seam,
2. Slice `10` remains reserved for guest-unit source-of-truth and sandbox
   unification,
3. Slice `11` remains reserved for the broader Substrate-owned
   lifecycle/diagnostics and sync productization seam,
4. Slice `12` remains reserved for the broader breakglass/docs cutover,
5. the implementation owner accepts that this slice is source-driven and must
   use current official Lima mount/vmtype/FAQ/breaking-change docs,
   `limactl copy`, `limactl create`, and Apple
   `VZVirtioFileSystemDevice` docs,
6. the slice stays bounded to actual ingress cutover plus only the minimal doc
   truth corrections made necessary by that cutover.

## Slice contract

This slice should land one explicit ingress implementation authority for:

1. removing broad default host-home visibility from same-user Lima,
2. removing the default mounted-checkout dependency at `/src`,
3. replacing those dependencies with explicit staged host→guest workspace and/or
   artifact ingress rooted in an approved guest-local writable path,
4. preserving routed warm, smoke, doctor, and gateway lifecycle proofs,
5. handing a precise staged-workspace/writable-root contract to Slice `10`,
6. handing any remaining operator-surface productization gap honestly to Slice
   `11`.

This slice must **not**:

1. widen into guest-unit source-of-truth or sandbox unification,
2. widen into a broad new CLI lifecycle/sync productization effort unless a
   minimal assist proves unavoidable,
3. relabel the support taxonomy,
4. perform the full docs/breakglass cutover,
5. reopen listener/transport seams already frozen earlier in the feature.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `scripts/mac/lima/substrate.yaml`,
3. `scripts/mac/lima-warm.sh`,
4. `scripts/mac/smoke.sh`,
5. `docs/WORLD.md`,
6. `docs/reference/world/platforms/macos-lima-setup.md`.

Treat edits to shell/backend code outside that boundary as scope expansion
unless live execution proves a minimal assist is mandatory to keep the ingress
cutover honest.

## Execution packets

### Packet 1: Source gate, staging-root decision, and live dependency inventory

Session goal:

1. confirm the authority stack, official source set, and live ingress-dependent
   repo surfaces,
2. freeze the exact guest-local staging-root direction,
3. decide whether the cutover can stay in scripts/config/docs or needs a
   minimal CLI/backend assist.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live ingress dependencies
  - Acceptance: the execution pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `2`, milestone `2.2`, milestone
    `3.1`, `DESIGN-macos-ingress-and-mount-contract.md`,
    `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-macos-guest-unit-source-of-truth.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official Lima
    mount/vmtype/FAQ/breaking-change/`limactl copy`/`limactl create` docs plus
    Apple `VZVirtioFileSystemDevice` docs. The pass also records the concrete
    ingress-bearing dependencies in `scripts/mac/lima/substrate.yaml`,
    `scripts/mac/lima-warm.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`, and
    `docs/reference/world/platforms/macos-lima-setup.md`.
  - Verify:
    - manual authority review
    - manual official-source review
    - manual repo-truth review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md`

- [x] Task 1.2: Freeze the guest-local staging-root and minimal implementation boundary
  - Acceptance: the slice names the preferred guest-local staging root,
    confirms it stays within an already-approved writable parent (preferably a
    child path under `/var/lib/substrate`), and states whether the ingress
    cutover can stay in scripts/config/docs or needs a minimal assist from
    existing CLI/backend surfaces.
  - Verify:
    - `rg -n "workspace sync|/var/lib/substrate|/tmp|staged|stage|copy" crates/shell/src/execution/workspace_cmd.rs scripts/mac/lima-warm.sh macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md`
    - manual boundary review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-09-ingress-cutover-and-explicit-staging-path.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-09.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the official source gate is explicit,
2. the live ingress-dependent repo surfaces are explicit,
3. the preferred guest-local staging-root direction is explicit,
4. the slice has not yet widened into actual code changes.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Mount-profile and warm-path cutover

Session goal:

1. remove the broad host-home mount and default `/src` mount dependency,
2. replace mounted-checkout identity/build assumptions with explicit staged
   ingress,
3. preserve warm/create/start/repair behavior.

#### Tasks

- [x] Task 2.1: Narrow the Lima mount profile to the hardened default
  - Acceptance: `scripts/mac/lima/substrate.yaml` no longer preserves broad
    host-home visibility as the hardened default, and any retained workspace
    ingress is explicit, minimal, and justified by the Slice `09` staged-path
    design rather than by convenience mounts.
  - Verify:
    - `rg -n "mounts:|location: \"\\$HOME\"|location: \"\\$PROJECT\"|mountPoint: \"/src\"|writable:" scripts/mac/lima/substrate.yaml`
    - manual diff review
  - Files:
    - `scripts/mac/lima/substrate.yaml`

- [x] Task 2.2: Replace warm-path `/src` dependence with explicit staged ingress
  - Acceptance: `scripts/mac/lima-warm.sh` no longer requires `/src` as the
    normal-path proof of checkout identity or build-source ingress; instead it
    validates and uses the explicit guest-local staged input path while still
    supporting host-staged Linux binaries and an honest fallback when Linux
    binaries are absent.
  - Verify:
    - `rg -n "ensure_repo_mount|/src|limactl copy|Cargo.lock|cargo build|staged|stage|/var/lib/substrate" scripts/mac/lima-warm.sh`
    - `bash -n scripts/mac/lima-warm.sh`
    - targeted manual warm-path review
  - Files:
    - `scripts/mac/lima-warm.sh`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. broad host-home visibility is gone from the hardened default,
2. mounted `/src` is no longer the required warm-path ingress model,
3. the replacement staged ingress path is explicit and guest-local,
4. the slice still has not widened into broader CLI/operator-surface redesign.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Smoke and minimal doc-truth cutover

Session goal:

1. adapt the smoke harness to the explicit staged ingress path,
2. remove the doc statements that became false after the cutover,
3. preserve routed proof priority.

#### Tasks

- [x] Task 3.1: Replace the smoke harness’s mounted-workspace proof
  - Acceptance: `scripts/mac/smoke.sh` no longer depends on `(cd /src ...)` as
    the normal-path routed write proof and instead validates filesystem-diff
    behavior through the explicit staged guest-local ingress path while keeping
    routed doctor and gateway lifecycle proof primary.
  - Verify:
    - `rg -n "/src|gateway sync|gateway status|gateway restart|host doctor|world doctor|world-mac-smoke|staged|stage" scripts/mac/smoke.sh`
    - `bash -n scripts/mac/smoke.sh`
    - targeted manual smoke-proof review
  - Files:
    - `scripts/mac/smoke.sh`

- [x] Task 3.2: Perform only the minimal doc truth corrections required by the cutover
  - Acceptance: `docs/WORLD.md` and
    `docs/reference/world/platforms/macos-lima-setup.md` no longer describe
    broad host-home visibility or mounted `/src` workflows as the hardened
    default, while broader operator-surface and breakglass/doc-cutover language
    remains deferred.
  - Verify:
    - `rg -n "/src|mount|HOME|limactl shell substrate bash -lc 'cd /src|mirrors the host repo checkout|broad host-home" docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
    - manual scope review
  - Files:
    - `docs/WORLD.md`
    - `docs/reference/world/platforms/macos-lima-setup.md`

### Packet 3 checkpoint

Packet `3` is complete only when:

1. the smoke harness no longer encodes mounted `/src` as the normal path,
2. the docs no longer say false things about the hardened default ingress,
3. routed doctor/gateway proof priority is preserved,
4. the slice still has not widened into full operator-story or docs cutover
   work.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final validation and downstream handoff

Session goal:

1. validate the final ingress cutover honestly,
2. record the precise handoff into Slice `10`, Slice `11`, and Slice `12`,
3. ensure the final wording distinguishes what landed from what remains.

#### Tasks

- [x] Task 4.1: Final scope and coherence check
  - Acceptance: the final diff stays within the allowed execution boundary
    unless an explicitly justified minimal assist was required, and the slice
    does not claim a broader operator-surface/productization landing than it
    actually implemented.
  - Verify:
    - `git diff --stat -- scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-09.md`
    - `git status --short`
    - manual wording review
  - Files:
    - touched files only

- [x] Task 4.2: Record the downstream handoff honestly
  - Acceptance: the final closeout states explicitly that Slice `09` landed the
    ingress cutover and explicit staged-path seam only, Slice `10` still owns
    guest-unit source-of-truth and sandbox unification, Slice `11` still owns
    broader Substrate-owned lifecycle/sync productization, and Slice `12`
    still owns the broad breakglass/docs cutover. The same closeout must also
    link any Packet `4` complete/checkpoint-green claim to current green
    evidence from the rerun Packet `2` / Packet `3` verification wall for the
    landed warm-path staging and routed smoke/doctor/gateway proof surfaces
    rather than treating those proofs as implied.
  - Verify:
    - rerun the Packet `2` verification commands for `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima-warm.sh`
    - rerun the Packet `3` verification commands for `scripts/mac/smoke.sh`, `docs/WORLD.md`, and `docs/reference/world/platforms/macos-lima-setup.md`
    - manual closeout review that the Packet `4` wording explicitly ties
      closeout/checkpoint-green status to that rerun Packet `2` / Packet `3`
      evidence
  - Files:
    - touched files only

Packet `4` closeout note:

1. Slice `09` landed the ingress cutover and explicit staged-path seam only.
2. The hardened default now stages the requested project path into
   `/var/lib/substrate/staged-workspace/current`; broad host-home visibility
   and a mounted `/src` checkout are no longer the hardened default ingress
   path.
3. Slice `10` still owns guest-unit source-of-truth and sandbox unification,
   consuming the staged-workspace and writable-root contract rooted in
   `/var/lib/substrate` plus the already-approved guest-local runtime roots
   (`/run`, `/run/substrate`, `/run/substrate.sock`,
   `/run/substrate/substrate-gateway-runtime/`, `/sys/fs/cgroup`, guest
   `SUBSTRATE_HOME`, and `/tmp`).
4. Slice `11` still owns broader Substrate-owned lifecycle, diagnostics, and
   sync productization beyond the script/config/docs cutover that landed here.
5. Slice `12` still owns the broad breakglass/docs cutover; Slice `09` only
   made the minimal doc truth corrections required by the ingress cutover.
6. Packet `4` closeout and checkpoint-green status should only be claimed
   alongside the rerun Packet `2` / Packet `3` verification wall that kept the
   landed warm-path staging, routed smoke proof, and routed doctor/gateway
   proof surfaces intact in the current tree.

### Packet 4 checkpoint

Packet `4` is complete only when:

1. the hardened default no longer depends on broad host-home visibility or a
   mounted `/src` checkout,
2. the explicit staged ingress path is clear and validated,
3. routed warm/smoke/doctor/gateway proofs remain green,
4. downstream boundaries into Slice `10`, Slice `11`, and Slice `12` are
   explicit and honest.

Do not declare the slice implementation-ready until all four conditions are
true.

Packet `4` is checkpoint-green for this landing once the verification checks
above are green, the closeout explicitly points at the current green
warm/smoke/routed doctor/routed gateway evidence from that rerun Packet `2` /
Packet `3` verification wall, and the final closeout keeps Slice `10`,
Slice `11`, and Slice `12` unblocked without widening into their
implementation seams.
