# TASKS-07: Remove Default Extra Listener Surface

Source spec:
- [`SPEC-07-remove-default-extra-listener-surface.md`](./SPEC-07-remove-default-extra-listener-surface.md)

Source plan:
- [`PLAN-07.md`](./PLAN-07.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `07` as the next seam,
2. Slice `08` / Slice `09` remain reserved for ingress and mount hardening,
3. Slice `10` remains reserved for guest-unit source-of-truth and sandbox
   unification,
4. Slice `12` remains reserved for the broader breakglass/docs cutover,
5. the implementation owner accepts that this slice is source-driven and must
   use current official Lima shell, SSH, port-forwarding, environment-variable,
   and breaking-change docs,
6. if any runtime symbol outside scripts/docs needs to change, GitNexus impact
   analysis is treated as mandatory before the edit and
   `gitnexus_detect_changes()` is treated as mandatory before committing,
7. transport redesign, ingress/mount work, and guest-unit generation redesign
   remain out of default scope.

## Slice contract

This slice should land one explicit listener-surface hardening contract for:

1. removing default `SUBSTRATE_AGENT_TCP_PORT=61337` injection from the macOS
   guest service path,
2. freezing `/run/substrate.sock` as the only hardened default guest listener,
3. keeping retained host compatibility TCP routing clearly separate from the
   guest listener contract,
4. keeping routed doctor/gateway/smoke proof green without implying guest TCP
   is required,
5. tightening listener-oriented docs so guest TCP and guest-direct access are
   no longer normalized as default behavior.

This slice must **not**:

1. redesign transport selection,
2. redesign ingress or mount posture,
3. redesign guest-unit source-of-truth or sandbox architecture,
4. broaden into full lifecycle CLI redesign,
5. perform a full feature-wide docs cutover.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `scripts/mac/lima-warm.sh`,
3. `scripts/mac/lima-doctor.sh` only if evidence or breakglass labeling needs
   adjustment,
4. `scripts/mac/smoke.sh` only if evidence or breakglass labeling needs
   adjustment,
5. `docs/WORLD.md`,
6. `docs/reference/world/platforms/macos-lima-setup.md`,
7. targeted syntax checks and nearby validation only.

Treat edits to `scripts/mac/lima/substrate.yaml`,
`crates/world-mac-lima/`, `crates/world-service/`, or later ingress/unit docs
as scope expansion unless the orchestrator can point to a direct contradiction
that the user explicitly approves for this slice.

## Execution packets

### Packet 1: Listener-contract freeze and source gate

Session goal:

1. confirm the authority stack, official source set, and live listener drift,
2. freeze the exact socket-only hardened default decision,
3. inventory the script/doc sections that still imply guest TCP is normal.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live listener drift
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `2`, milestone `2.1`,
    `DESIGN-macos-lima-transport-contract.md`,
    `DESIGN-macos-guest-unit-source-of-truth.md`,
    `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official Lima
    shell, SSH, port-forwarding, environment-variable, and breaking-change
    docs. The pass also records the relevant repo-truth drift in
    `scripts/mac/lima-warm.sh`, `crates/world-service/src/lib.rs`,
    `crates/world-mac-lima/src/transport.rs`, `docs/WORLD.md`, and
    `docs/reference/world/platforms/macos-lima-setup.md`.
  - Verify:
    - manual authority review
    - manual official-source review
    - manual repo-truth review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md`

- [x] Task 1.2: Freeze the canonical listener decision and scope boundary
  - Acceptance: the slice makes it explicit that `/run/substrate.sock` is the
    only hardened default guest listener on macOS, default
    `SUBSTRATE_AGENT_TCP_PORT=61337` injection must be removed from the warm/
    repair path, retained host compatibility TCP routing is not equivalent to a
    supported guest listener contract, and unit-source-of-truth work remains
    deferred to Slice `10`.
  - Verify:
    - `rg -n "SUBSTRATE_AGENT_TCP_PORT|61337|17788|SUBSTRATE_WORLD_SOCKET|/run/substrate.sock" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md`
    - manual coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-07-remove-default-extra-listener-surface.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-07.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-07.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the hardened default listener decision is explicit,
2. the contradiction surfaces are explicit,
3. the slice still has not widened into mount or unit-source-of-truth work.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Remove default guest TCP enablement from warm/repair

Session goal:

1. cut `SUBSTRATE_AGENT_TCP_PORT=61337` from the macOS warm/repair path,
2. preserve `/run/substrate.sock` as the default guest listener contract,
3. update any local assertions or comments that still assume guest TCP.

#### Tasks

- [ ] Task 2.1: Remove default `SUBSTRATE_AGENT_TCP_PORT` injection from `scripts/mac/lima-warm.sh`
  - Acceptance: `scripts/mac/lima-warm.sh` no longer writes
    `Environment=SUBSTRATE_AGENT_TCP_PORT=61337` into the guest service unit by
    default. The script still writes the intended socket-centered service state
    and any remaining references to guest TCP are either removed or clearly
    classified as non-default compatibility/breakglass material.
  - Verify:
    - `bash -n scripts/mac/lima-warm.sh`
    - `rg -n "SUBSTRATE_AGENT_TCP_PORT|61337|SUBSTRATE_WORLD_SOCKET|/run/substrate.sock" scripts/mac/lima-warm.sh`
    - manual rendered-unit review around `write_systemd_units`
  - Files:
    - `scripts/mac/lima-warm.sh`

- [ ] Task 2.2: Keep listener evidence honest after the warm/repair cutover
  - Acceptance: if `scripts/mac/lima-doctor.sh` or `scripts/mac/smoke.sh` still
    imply that a guest TCP listener is required, update them so the tightened
    socket-only guest default is reflected honestly. If they already remain
    truthful after Task `2.1`, leave them unchanged.
  - Verify:
    - `bash -n scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
    - `rg -n "61337|SUBSTRATE_AGENT_TCP_PORT|17788|guest-direct|breakglass|compatibility" scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
    - manual proof-order review
  - Files:
    - `scripts/mac/lima-doctor.sh` only if needed
    - `scripts/mac/smoke.sh` only if needed

### Packet 2 checkpoint

Packet `2` is complete only when:

1. the warm/repair path no longer enables guest TCP by default,
2. the intended guest listener remains `/run/substrate.sock`,
3. helper evidence no longer implies guest TCP is required.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Cut over listener-oriented docs

Session goal:

1. make listener-oriented docs match the tightened guest-listener truth,
2. distinguish guest listener truth from retained host compatibility routing and
   guest-direct breakglass access.

#### Tasks

- [ ] Task 3.1: Update `docs/WORLD.md` to describe the tightened listener posture honestly
  - Acceptance: the macOS/Lima sections of `docs/WORLD.md` no longer imply that
    guest TCP is part of the hardened default. Any mention of retained host
    compatibility routing, `SUBSTRATE_WORLD_SOCKET`, or guest-direct diagnostics
    is clearly secondary to the socket-first guest listener contract.
  - Verify:
    - `rg -n "17788|61337|SUBSTRATE_AGENT_TCP_PORT|SUBSTRATE_WORLD_SOCKET|/run/substrate.sock|breakglass|compatibility" docs/WORLD.md`
    - manual wording review against Slice `06` and the support taxonomy
  - Files:
    - `docs/WORLD.md`

- [ ] Task 3.2: Update macOS setup docs to stop normalizing guest TCP
  - Acceptance:
    `docs/reference/world/platforms/macos-lima-setup.md` no longer reads as if
    guest TCP is a normal hardened default. Any remaining references to TCP
    resets, direct guest probes, or SSH-only recovery are clearly framed as
    compatibility, troubleshooting, or breakglass rather than the supported
    listener contract.
  - Verify:
    - `rg -n "17788|61337|TCP|curl --unix-socket|limactl shell|ssh|breakglass|compatibility" docs/reference/world/platforms/macos-lima-setup.md`
    - manual wording review
  - Files:
    - `docs/reference/world/platforms/macos-lima-setup.md`

### Packet 3 checkpoint

Packet `3` is complete only when:

1. listener-oriented docs match the tightened guest-listener contract,
2. compatibility TCP and guest-direct access are clearly secondary,
3. the slice still has not widened into broad docs cutover.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final validation and next-slice handoff clarity

Session goal:

1. validate that Slice `07` stayed listener-scoped,
2. run final targeted verification across scripts/docs,
3. leave a clean handoff to Slice `08` / Slice `09` and Slice `10`.

#### Tasks

- [ ] Task 4.1: Final targeted regression and scope check
  - Acceptance: syntax checks pass for touched shell scripts, final doc/script
    diffs remain limited to the intended listener-oriented surfaces, and any
    optional runtime fallout remains absent or tightly justified.
  - Verify:
    - `bash -n scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
    - `git diff --stat -- scripts/mac/lima-warm.sh scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec`
    - `git status --short`
    - `gitnexus_detect_changes()` before committing if any runtime symbols changed
  - Files:
    - touched files only

- [ ] Task 4.2: Record the handoff boundary honestly
  - Acceptance: the final closeout states explicitly that Slice `07` landed the
    listener-default removal seam only, Slice `08` / `09` still own
    ingress/mount hardening, Slice `10` still owns guest-unit source-of-truth
    work, and Slice `12` still owns the broader breakglass/docs cutover.
  - Verify:
    - manual closeout review
  - Files:
    - implementation closeout or PR description

Packet `4` closeout note:

1. Slice `07` landed listener-default removal only.
2. Slice `08` / `09` remain the ingress and mount seams.
3. Slice `10` remains the guest-unit source-of-truth seam.
4. Slice `12` remains the broader breakglass and docs-cutover seam.

### Packet 4 checkpoint

Packet `4` is complete only when:

1. the slice remained listener-scoped,
2. verification evidence shows the default guest TCP listener is gone,
3. the next deferred seams are stated plainly.
