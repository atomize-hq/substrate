# TASKS-08: Ingress Inventory and Narrowed Mount Contract

Source spec:
- [`SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`](./SPEC-08-ingress-inventory-and-narrowed-mount-contract.md)

Source plan:
- [`PLAN-08.md`](./PLAN-08.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `08` as the next seam,
2. Slice `09` remains reserved for actual ingress implementation and/or any
   Substrate-managed sync/copy path,
3. Slice `10` remains reserved for guest-unit source-of-truth and sandbox
   unification,
4. Slice `12` remains reserved for the broader breakglass/docs cutover,
5. the implementation owner accepts that this slice is source-driven and must
   use current official Lima mount/vmtype/FAQ/breaking-change docs plus Apple
   `VZVirtioFileSystemDevice` docs,
6. actual mount removal, sync/copy work, and sandbox changes remain out of
   default scope for this slice.

## Slice contract

This slice should land one explicit ingress-inventory and narrowed-contract
authority for:

1. the current default guest-visible host inputs for same-user Lima,
2. the classification of those inputs into workspace, auth, runtime, and
   troubleshooting classes,
3. the narrowed default decision for each class: direct mount, future
   Substrate-managed sync/copy, or breakglass,
4. the proof surfaces later mount minimization must keep green,
5. the explicit handoff boundary into Slice `09` and Slice `10`.

This slice must **not**:

1. edit the actual Lima profile mounts by default,
2. add a new sync/copy implementation,
3. redesign guest-unit source-of-truth or sandbox architecture,
4. broaden into full lifecycle CLI redesign,
5. perform a full feature-wide docs cutover.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. review-only inspection of `scripts/mac/lima/substrate.yaml`,
   `scripts/mac/lima-warm.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`,
   `docs/reference/world/platforms/macos-lima-setup.md`,
   `crates/shell/src/builtins/world_gateway.rs`, and
   `crates/world-service/src/gateway_runtime.rs`.

Treat actual edits to runtime scripts, docs outside the slice docs, or mount
profile files as scope expansion unless the user explicitly approves merging
Slice `09` implementation into this seam.

## Execution packets

### Packet 1: Source gate and live ingress inventory

Session goal:

1. confirm the authority stack, official source set, and live ingress posture,
2. record the exact default mount inventory and the concrete repo surfaces that
   depend on it,
3. name the ingress classes this slice will use.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live ingress posture
  - Acceptance: the execution pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `2`, milestone `2.2`,
    `DESIGN-macos-ingress-and-mount-contract.md`,
    `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official Lima
    mount/vmtype/FAQ/breaking-change docs plus Apple
    `VZVirtioFileSystemDevice` docs. The pass also records current repo-truth
    ingress in `scripts/mac/lima/substrate.yaml`,
    `scripts/mac/lima-warm.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`,
    `docs/reference/world/platforms/macos-lima-setup.md`,
    `crates/shell/src/builtins/world_gateway.rs`, and
    `crates/world-service/src/gateway_runtime.rs`.
  - Verify:
    - manual authority review
    - manual official-source review
    - manual repo-truth review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`

- [x] Task 1.2: Record the current mount inventory and ingress class vocabulary
  - Acceptance: the slice names the current default host-home and `/src`
    mounts, records where they are consumed, and defines the classification
    classes as workspace, auth, runtime, and troubleshooting ingress.
  - Verify:
    - `rg -n "mounts:|location: \"\\$HOME\"|location: \"\\$PROJECT\"|mountPoint: \"/src\"" scripts/mac/lima/substrate.yaml`
    - `rg -n "/src|HOME|gateway-runtime|integrated auth|breakglass" scripts/mac/lima-warm.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
    - manual coherence review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the official source gate is explicit,
2. the live default ingress inventory is explicit,
3. the slice still has not widened into actual mount or sync implementation.

Packet `1` is source-gated and inventory-complete for this landing;
later class-decision, validation, and closeout packets remain pending.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Classify ingress and freeze the narrowed decision matrix

Session goal:

1. map every current guest-visible input into a class,
2. decide whether each class remains a temporary direct mount, future
   sync/copy work, or breakglass,
3. separate supported gateway/runtime needs from broad mounted-home
   convenience.

#### Tasks

- [ ] Task 2.1: Build the ingress classification matrix
  - Acceptance: the slice contains a plain-language decision matrix covering
    workspace source input, auth/credential input, runtime artifacts, and
    troubleshooting/convenience input, with each class mapped to its intended
    hardened posture.
  - Verify:
    - `rg -n "workspace|auth|runtime|troubleshooting|direct mount|sync/copy|breakglass" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
    - manual matrix review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`

- [ ] Task 2.2: Freeze the narrowed default decision for host-home visibility and `/src`
  - Acceptance: the slice makes it explicit that broad host-home visibility is
    not preserved by default without path-by-path justification and that `/src`
    is decomposed into actual supported needs rather than retained as an
    undifferentiated convenience mount.
  - Verify:
    - `rg -n "\\$HOME|/src|path-by-path|justif|sync/copy|breakglass" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
    - manual scope review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. every ingress class has an intended hardened posture,
2. broad host-home visibility is no longer implicit,
3. `/src` has been decomposed into concrete needs.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Freeze validation and future doc-cutover expectations

Session goal:

1. name the proofs later mount minimization must keep green,
2. identify which docs still teach convenience ingress today,
3. record what Slice `09` and Slice `10` must consume from this contract.

#### Tasks

- [ ] Task 3.1: Define the later validation surfaces explicitly
  - Acceptance: the slice names the warm, smoke, routed gateway, and
    diagnostics proofs that later mount minimization must preserve.
  - Verify:
    - `rg -n "lima-warm|smoke|gateway sync|gateway status|diagnostics|proof" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
    - manual validation review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`

- [ ] Task 3.2: Record the future doc and sandbox consumers without widening into them
  - Acceptance: the slice identifies `docs/WORLD.md`,
    `docs/reference/world/platforms/macos-lima-setup.md`, and the future
    guest-unit sandbox consumer seam as downstream consumers, while keeping
    their actual cutover out of this slice.
  - Verify:
    - `rg -n "docs/WORLD.md|macos-lima-setup|Slice 09|Slice 10|ProtectHome|ReadWritePaths" macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
    - manual handoff review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`

### Packet 3 checkpoint

Packet `3` is complete only when:

1. later proof surfaces are explicit,
2. downstream docs/sandbox consumers are named,
3. the slice still has not widened into actual mount minimization or docs
   cutover work.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final scope check and next-slice handoff

Session goal:

1. validate that Slice `08` stayed contract-scoped and docs-only,
2. record the clean handoff into Slice `09` and Slice `10`,
3. make sure the final wording is honest about what did and did not land.

#### Tasks

- [ ] Task 4.1: Final scope and coherence check
  - Acceptance: final diffs remain limited to the Slice `08` docs, and the
    slice does not promise actual mount removal or sync/copy behavior it did
    not implement.
  - Verify:
    - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec/SPEC-08-ingress-inventory-and-narrowed-mount-contract.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-08.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-08.md`
    - `git status --short`
    - manual wording review
  - Files:
    - touched files only

- [ ] Task 4.2: Record the handoff boundary honestly
  - Acceptance: the final closeout states explicitly that Slice `08` landed the
    ingress inventory and narrowed-contract seam only, Slice `09` still owns
    actual mount/sync implementation, Slice `10` still owns guest-unit
    source-of-truth and sandbox unification, and Slice `12` still owns the
    broader breakglass/docs cutover.
  - Verify:
    - manual closeout review
  - Files:
    - implementation closeout or PR description

### Packet 4 checkpoint

Packet `4` is complete only when:

1. the slice remained docs-only and contract-scoped,
2. the implementation seam for Slice `09` is obvious,
3. the deferred downstream consumers are stated plainly.
