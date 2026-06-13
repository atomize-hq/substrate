# TASKS-10: Guest Unit Source of Truth and Sandbox Unification

Source spec:
- [`SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`](./SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md)

Source plan:
- [`PLAN-10.md`](./PLAN-10.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`
Status: draft task set
Execution model: four sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `10` as the next seam,
2. Slice `11` remains reserved for the broader Substrate-owned lifecycle,
   diagnostics, and sync productization seam,
3. Slice `12` remains reserved for the broad breakglass/docs cutover,
4. the implementation owner accepts that this slice is source-driven and must
   use current official `systemd.exec` and `systemd.socket` docs plus the
   targeted inherited Lima `limactl copy` and Filesystem mounts docs,
5. the slice stays bounded to guest unit source-of-truth and sandbox
   unification plus only the minimal doc truth corrections made necessary by
   that unification.

## Slice contract

This slice should land one explicit unit authority for:

1. removing separately handwritten guest service/socket authority across the
   create/bootstrap and warm/repair paths,
2. unifying hardening-critical service fields including environment,
   `ProtectHome=`, `ReadWritePaths=`, capabilities, runtime directories, and
   socket ownership/mode,
3. preserving the Slice `07` socket-first listener contract,
4. preserving the Slice `09` staged-workspace and guest-local writable-root
   contract,
5. adding validation that proves rendered-unit parity,
6. handing remaining operator-surface productization honestly to Slice `11`.

This slice must **not**:

1. widen into broad lifecycle/sync UX productization,
2. widen into the full breakglass/docs cutover,
3. reopen ingress redesign after Slice `09`,
4. silently widen into unrelated Rust/backend work unless a minimal assist is
   proven mandatory,
5. relabel the support taxonomy.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`,
2. `scripts/mac/lima/substrate.yaml`,
3. `scripts/mac/lima-warm.sh`,
4. the new canonical unit-source files under `scripts/mac/lima/`,
5. `scripts/mac/lima-doctor.sh`,
6. `scripts/mac/smoke.sh`,
7. `docs/WORLD.md`,
8. `docs/reference/world/platforms/macos-lima-setup.md`.

Treat edits outside that boundary as scope expansion unless live execution
proves a minimal assist is mandatory.

## Execution packets

### Packet 1: Source gate, drift inventory, and canonical-source decision

Session goal:

1. confirm the authority stack, official source set, and live unit drift,
2. freeze the exact authoritative unit direction,
3. decide whether the cutover can stay in scripts/config/docs or needs a
   minimal backend assist.

#### Tasks

- [x] Task 1.1: Confirm the authority stack, source gate, and live unit drift
  - Acceptance: the execution pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, Phase `2`, milestone `2.3`,
    Phase `3`, milestone `3.1`, `DESIGN-macos-guest-unit-source-of-truth.md`,
    `DESIGN-macos-ingress-and-mount-contract.md`,
    `DESIGN-macos-lima-transport-contract.md`,
    `DESIGN-macos-operator-lifecycle-and-diagnostics-contract.md`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`, and the official
    `systemd.exec`, `systemd.socket`, Lima `limactl copy`, and Lima Filesystem
    mounts docs. The pass also records the concrete service/socket drift across
    `scripts/mac/lima/substrate.yaml` and `scripts/mac/lima-warm.sh`.
  - Verify:
    - `rg -n "substrate-world-service\.(service|socket)|ProtectHome=|ReadWritePaths=|RuntimeDirectory=|StateDirectory=|CapabilityBoundingSet=|AmbientCapabilities=|Environment=" scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh`
    - manual authority review
    - manual official-source review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md`

- [x] Task 1.2: Freeze the canonical-source mechanism and target sandbox contract
  - Acceptance: the slice names the authoritative unit-source mechanism,
    confirms that `scripts/mac/lima/substrate.yaml` reduces itself to bootstrap
    prerequisites only while the canonical install path owns the actual guest
    service/socket units, freezes the final Phase `2`
    writable-path/environment/capability/socket contract that must be rendered
    identically across fresh-create and warm/repair, explicitly preserves the
    managed gateway-runtime surface under
    `/run/substrate/substrate-gateway-runtime/`, explicitly freezes
    `Environment=RUST_LOG=info`, `Group=substrate`, `UMask=0027`,
    `RuntimeDirectoryMode=0750`, `StateDirectoryMode=0750`,
    `WorkingDirectory=/var/lib/substrate`, and `ProtectSystem=strict`, and
    states that Packet `2` can remain
    scripts/config/docs first unless parity proof forces a minimal backend
    assist.
  - Verify:
    - `rg -n "staged-workspace|substrate-gateway-runtime|SUBSTRATE_HOME|SUBSTRATE_WORLD_SOCKET|WORLD_NETFILTER_ENABLE|RUST_LOG=info|Group=substrate|UMask=0027|WorkingDirectory=/var/lib/substrate|ProtectSystem=strict|ProtectHome|ReadWritePaths|RuntimeDirectory=substrate|RuntimeDirectoryMode=0750|StateDirectory=substrate|StateDirectoryMode=0750|CapabilityBoundingSet|AmbientCapabilities|CAP_CHOWN|ListenStream|SocketMode|SocketUser|SocketGroup|DirectoryMode|RemoveOnStop" scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md`
    - manual boundary review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-10-guest-unit-source-of-truth-and-sandbox-unification.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-10.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md`

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the official source gate is explicit,
2. the live hardening-critical unit drift is explicit,
3. the authoritative source mechanism is explicit,
4. the frozen target sandbox contract and Packet `2` boundary are explicit,
5. the slice has not yet widened into implementation changes.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Canonical unit source and create/repair convergence

Session goal:

1. land one authoritative service/socket source,
2. make both fresh-create and warm/repair consume the same rendered contract,
3. preserve the Phase `2` listener and ingress results.

#### Tasks

- [x] Task 2.1: Introduce the canonical checked-in unit authority
  - Acceptance: one checked-in canonical source exists for
    `substrate-world-service.service` and `.socket`, and any supported
    parameterization is explicit and minimal rather than split across separate
    handwritten unit bodies.
  - Verify:
    - `rg -n "substrate-world-service\.(service|socket)|ProtectHome=|ReadWritePaths=|RuntimeDirectory=|StateDirectory=|SocketMode=|SocketGroup=" scripts/mac/lima scripts/mac/lima-warm.sh`
    - manual diff review
  - Files:
    - new canonical unit-source files under `scripts/mac/lima/`
    - `scripts/mac/lima-warm.sh`

- [x] Task 2.2: Remove bootstrap-vs-repair service drift
  - Acceptance: `scripts/mac/lima/substrate.yaml` and
    `scripts/mac/lima-warm.sh` no longer act as separate handwritten guest
    service authorities; both paths either consume the same rendered unit
    contract or the bootstrap profile is explicitly reduced to prerequisites
    only while the canonical install path owns the units.
  - Verify:
    - `rg -n "cat >/etc/systemd/system/substrate-world-service|tee /etc/systemd/system/substrate-world-service|ProtectHome=|ReadWritePaths=|CapabilityBoundingSet=|AmbientCapabilities=" scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh scripts/mac/lima`
    - `bash -n scripts/mac/lima-warm.sh`
    - targeted manual create/repair parity review
  - Files:
    - `scripts/mac/lima/substrate.yaml`
    - `scripts/mac/lima-warm.sh`
    - new canonical unit-source files under `scripts/mac/lima/`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. there is no longer dual handwritten guest unit authority,
2. the final service/socket contract is explicit and reviewable,
3. Slice `07` and Slice `09` results remain preserved,
4. the slice still has not widened into broader Phase `3` productization.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Validation parity and minimal doc-truth cutover

Session goal:

1. prove rendered-unit parity through supported evidence surfaces,
2. remove the doc statements made false by dual unit authority,
3. preserve routed proof priority.

#### Tasks

- [x] Task 3.1: Add rendered-unit parity validation to doctor/smoke surfaces
  - Acceptance: `scripts/mac/lima-doctor.sh` and/or `scripts/mac/smoke.sh`
    explicitly prove that the guest service/socket contract matches the
    authoritative rendered contract, not merely that the VM is alive.
  - Verify:
    - `rg -n "substrate-world-service\.(service|socket)|ProtectHome|ReadWritePaths|RuntimeDirectory|StateDirectory|SocketMode|SocketGroup|systemctl cat|systemctl show|cmp|sha256sum" scripts/mac/lima-doctor.sh scripts/mac/smoke.sh`
    - `bash -n scripts/mac/lima-doctor.sh`
    - `bash -n scripts/mac/smoke.sh`
  - Files:
    - `scripts/mac/lima-doctor.sh`
    - `scripts/mac/smoke.sh`

- [x] Task 3.2: Perform only the minimal doc truth corrections required by unit unification
  - Acceptance: `docs/WORLD.md` and
    `docs/reference/world/platforms/macos-lima-setup.md` now point to one
    authoritative guest unit contract and no longer imply that bootstrap and
    repair may legitimately define different hardening settings.
  - Verify:
    - `rg -n "substrate-world-service\.(service|socket)|ProtectHome|ReadWritePaths|SUBSTRATE_HOME|WORLD_NETFILTER_ENABLE|authoritative|canonical" docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md`
    - manual scope review
  - Files:
    - `docs/WORLD.md`
    - `docs/reference/world/platforms/macos-lima-setup.md`

### Packet 3 checkpoint

Packet `3` is complete only when:

1. doctor/smoke can prove rendered-unit parity,
2. the docs no longer describe dual unit authority as acceptable,
3. routed proof priority is preserved,
4. the slice still has not widened into the Phase `3` operator-story seam.

Do not start Packet `4` until Packet `3` verification is green.

### Packet 4: Final validation and downstream handoff

Session goal:

1. validate unit unification honestly,
2. record the precise handoff into Slice `11` and Slice `12`,
3. ensure final wording distinguishes what landed from what remains.

#### Tasks

- [x] Task 4.1: Final scope and coherence check
  - Acceptance: the final diff stays within the allowed execution boundary
    unless an explicitly justified minimal assist was required, and the slice
    does not claim broader lifecycle/sync productization than it actually
    implemented.
  - Verify:
    - `git diff --stat -- scripts/mac/lima/substrate.yaml scripts/mac/lima-warm.sh scripts/mac/lima scripts/mac/lima-doctor.sh scripts/mac/smoke.sh docs/WORLD.md docs/reference/world/platforms/macos-lima-setup.md macos-hardening/macos-hardened-same-user-lima/spec/TASKS-10.md`
    - `git status --short`
    - manual wording review
  - Files:
    - touched files only

- [x] Task 4.2: Record the downstream handoff honestly
  - Acceptance: the final closeout states explicitly that Slice `10` landed the
    unit source-of-truth and sandbox-unification seam only, Slice `11` still
    owns broader Substrate-owned lifecycle/diagnostics/sync productization, and
    Slice `12` still owns the broad breakglass/docs cutover. The same closeout
    must tie any Packet `4` complete/checkpoint-green claim to current green
    evidence from the rerun Packet `2` / Packet `3` verification wall.
  - Verify:
    - rerun the Packet `2` verification commands for `scripts/mac/lima/substrate.yaml`, `scripts/mac/lima-warm.sh`, and the canonical unit-source files
    - rerun the Packet `3` verification commands for `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`, `docs/WORLD.md`, and `docs/reference/world/platforms/macos-lima-setup.md`
    - `scripts/mac/lima-warm.sh --check-only`
    - `scripts/mac/lima-doctor.sh`
    - manual closeout review that the Packet `4` wording explicitly ties
      closeout/checkpoint-green status to that rerun evidence
  - Files:
    - touched files only

#### Packet 4 closeout and downstream handoff

Closeout verified on 2026-06-13 against the rerun Packet `2` / Packet `3`
verification wall. Slice `10` closes only the guest unit source-of-truth and
sandbox-unification seam. A narrow validation-unblocking repair in
`scripts/mac/lima-warm.sh` was required so staged-workspace verification runs
under the actual guest ownership boundary and optional in-guest Linux CLI build
failures no longer abort mandatory `world-service` / `substrate-gateway`
provisioning or closeout cleanup.

The downstream boundary remains explicit:

1. Slice `11` is still the next seam for broader Substrate-owned
   lifecycle/diagnostics/sync productization beyond the landed unit-parity
   contract.
2. Slice `12` is still the next seam for the broad breakglass/docs cutover and
   support-taxonomy reclassification beyond the minimal Slice `10` doc truth
   corrections.
3. Packet `4` checkpoint-green status depends on the current rerun evidence
   surface (`git diff --stat`, `git status --short`, Packet `2` command reruns,
   Packet `3` command reruns, `scripts/mac/lima-warm.sh --check-only`, and
   `scripts/mac/lima-doctor.sh`) rather than on broader Slice `11` / Slice `12`
   completion.
