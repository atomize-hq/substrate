# PLAN-02: Lima Version Floor and Breakglass Contract

Source spec:
- [`SPEC-02-lima-version-floor-and-breakglass-contract.md`](./SPEC-02-lima-version-floor-and-breakglass-contract.md)

Source phase authority:
- [`../phase-0-security-contract-and-scope/README.md`](../phase-0-security-contract-and-scope/README.md)
- [`../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md`](../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)

Plan type: source-driven phase-0 contract slice for the macOS hardened
same-user Lima program  
Phase: `PLAN`  
Status: draft plan

## Plan summary

The next honest seam is Slice `02`: freeze the minimum supported Lima/macOS
capability contract and the breakglass boundary that later transport, docs, and
lifecycle slices must inherit.

This plan should produce a narrow docs-and-contract landing that:

1. turns vague version assumptions into one explicit environment contract,
2. turns repeated direct-guest escape hatches into one explicit breakglass
   matrix,
3. states the minimum transport assumptions needed for that classification,
4. keeps canonical transport unification and repo-wide docs cutover deferred to
   later slices.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md`
4. `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md` only if the milestone wording must be tightened to match the slice
5. the feature-local design docs only if bounded terminology clarification is
   required

Top-level repo docs such as `docs/WORLD.md` and
`docs/reference/world/platforms/macos-lima-setup.md` are follow-on scope unless
execution proves an immediate contradiction and the scope expansion is
explicitly approved.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice
2. `source-driven-development` is also required

Plan consequence:

1. use live repo truth plus official Lima docs as co-equal authorities,
2. prefer exact lifecycle/version/capability claims over ambiguous “recent
   Lima” language,
3. surface any unresolved version-floor or forwarding-precondition conflict as
   an explicit open question rather than silently guessing.

## Official source set this plan must use

The plan assumes the resulting slice cites the following official docs when they
drive decisions:

1. [Lima releases lifecycle](https://lima-vm.io/docs/releases/)
2. [Lima VM types](https://lima-vm.io/docs/config/vmtype/)
3. [Lima VZ](https://lima-vm.io/docs/config/vmtype/vz/)
4. [Lima Port Forwarding](https://lima-vm.io/docs/config/port/)
5. [Lima Filesystem mounts](https://lima-vm.io/docs/config/mount/)
6. [Lima `limactl shell`](https://lima-vm.io/docs/reference/limactl_shell/)
7. [Lima breaking changes](https://lima-vm.io/docs/releases/breaking/)

## Major components and dependencies

1. **supported environment contract**
   - define the minimum supported Lima/macOS capability set for hardened mode
   - decide whether the supported floor should track the currently supported
     Lima major line
2. **breakglass workflow contract**
   - classify direct guest shell/admin flows and host-side bypasses
   - define the supported replacement expectation for normal operation
3. **minimal transport baseline for classification**
   - define only the minimum transport assumptions needed for breakglass
     classification
   - explicitly defer full endpoint/adapter unification to Slice `03`
4. **future-slice handoff clarity**
   - leave Slice `03` as transport unification
   - leave Slice `12` as repo-wide breakglass reclassification and docs cutover

Dependency order:

1. official-source-backed environment assumptions first
2. breakglass matrix second
3. minimal transport/bypass classification third
4. validation and explicit deferrals last

## Locked decisions

### What this slice changes

1. It freezes one explicit environment and lifecycle contract for supported
   same-user Lima mode.
2. It freezes the breakglass classification for direct guest and host-bypass
   workflows.
3. It states what normal operators should use instead of those workflows.
4. It frames retained compatibility probes and bypasses without mistaking them
   for the supported default.

### What this slice does not change

1. no transport implementation rewrite
2. no PTY/non-PTY/readiness/doctor convergence work
3. no mount minimization or guest unit/source-of-truth work
4. no repo-wide macOS doc cutover
5. no runtime code, provisioning, or health-command implementation changes

## Implementation order

### Packet 1: Freeze the supported environment contract from repo truth plus official Lima sources

Goal:

1. define the minimum supported Lima/macOS capability contract,
2. capture the version-floor decision criteria explicitly,
3. identify which current repo assumptions already depend on that contract.

Primary touch surface:

1. `SPEC-02-lima-version-floor-and-breakglass-contract.md`
2. `../phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md` only if slice-local alignment is required

Why first:

1. breakglass classification depends on the supported environment contract,
2. transport assumptions cannot be classified honestly without the lifecycle and
   capability baseline.

Verification checkpoint:

1. official Lima lifecycle/VZ/VM-type/mount/port-forwarding docs are cited for
   version-sensitive claims,
2. the repo assumptions in `scripts/mac/lima/substrate.yaml` and
   `crates/world-mac-lima` are accounted for,
3. no full transport-unification decisions were made yet.

### Packet 2: Freeze the breakglass matrix and supported replacement rule

Goal:

1. classify direct guest administration and host-side bypasses explicitly,
2. define which Substrate-owned commands remain the normal operator path,
3. state the status of retained compatibility probes and exceptional paths.

Primary touch surface:

1. `SPEC-02-lima-version-floor-and-breakglass-contract.md`
2. `TASKS-02.md`
3. milestone `0.2` only if wording alignment is required

Why second:

1. the matrix needs the environment contract from Packet `1`,
2. later slices need one inherited classification before transport or docs
   cleanup begins.

Verification checkpoint:

1. `limactl shell`, direct guest `systemctl`, direct guest socket curls, and
   host-side `SUBSTRATE_WORLD_SOCKET` override use are classified explicitly,
2. normal operation clearly starts from Substrate-owned doctor/gateway
   surfaces,
3. the slice still does not absorb repo-wide doc cutover.

### Packet 3: Validate scope discipline and leave a clean Slice `03` / Slice `12` handoff

Goal:

1. confirm the slice stayed at contract level,
2. name exactly what remains for Slice `03` and Slice `12`,
3. keep short-prompt continuation viable.

Primary touch surface:

1. `SPEC-02-lima-version-floor-and-breakglass-contract.md`
2. `PLAN-02.md`
3. `TASKS-02.md`

Why third:

1. the handoff is only honest after environment and breakglass decisions are
   stable,
2. this packet prevents Slice `02` from quietly broadening into adjacent work.

Verification checkpoint:

1. Slice `03` remains the canonical guest endpoint and transport contract seam,
2. Slice `12` remains the repo-wide breakglass reclassification/docs-cutover
   seam,
3. a future short prompt can continue from the landed materials without
   reopening Slice `02`.

## Risks and mitigations

### Risk 1: Slice `02` absorbs Slice `03` transport work

Mitigation:

1. allow only the minimal transport statements needed to classify supported
   versus breakglass operation,
2. defer endpoint/adapter unification and stale-constant cleanup details to
   Slice `03`.

### Risk 2: The slice guesses a version floor without strong source backing

Mitigation:

1. require official Lima lifecycle and capability sources for every
   version-sensitive claim,
2. freeze the macOS floor as `13.0+` when the official VZ and mount docs back
   that requirement, and avoid silently tightening the support matrix beyond
   those official minimums when the repo already pins `vmType: "vz"`
   explicitly.

### Risk 3: The slice over-promotes breakglass flows into the normal operator story

Mitigation:

1. require one explicit breakglass matrix,
2. require the supported replacement path to start from Substrate-owned
   doctor/gateway flows.

### Risk 4: The slice silently widens into repo-wide docs cutover

Mitigation:

1. keep top-level docs out of the default landing boundary,
2. treat any wider doc edits as separately approved contradiction cleanup.

## Parallelism guidance

This slice is partly parallelizable.

Parallelizable work:

1. reading official Lima source pages,
2. inventorying current repo assumptions and breakglass-sensitive wording,
3. enumerating direct guest/admin workflows from docs and scripts

Sequential work:

1. freezing the supported environment contract,
2. freezing the breakglass matrix,
3. validating handoff boundaries to Slice `03` and Slice `12`

## Exit criteria

This plan is ready to hand off to `TASKS-02` only when:

1. the slice remains bounded to version-floor and breakglass contract work,
2. the required official source set is explicit,
3. the touched-file boundary is narrow and docs-first,
4. retained transport compatibility paths are classified without becoming the
   supported contract,
5. Packet-level execution can proceed without inventing additional scope.
