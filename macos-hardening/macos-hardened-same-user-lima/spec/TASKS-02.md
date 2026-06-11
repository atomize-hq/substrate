# TASKS-02: Lima Version Floor and Breakglass Contract

Source spec:
- [`SPEC-02-lima-version-floor-and-breakglass-contract.md`](./SPEC-02-lima-version-floor-and-breakglass-contract.md)

Source plan:
- [`PLAN-02.md`](./PLAN-02.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)

Phase: `TASKS`  
Status: draft task set  
Execution model: three sequential packets

## Phase gate

Do not execute this slice until:

1. the user accepts Slice `02` as the next seam,
2. the scope stays bounded to version-floor and breakglass contract work,
3. the implementation owner accepts that this slice is source-driven and must
   use official Lima docs for version-sensitive decisions,
4. Slice `03` remains reserved for the canonical transport contract and Slice
   `12` remains reserved for repo-wide breakglass/docs cutover work.

## Slice contract

This slice should land the first explicit contract for:

1. the minimum supported Lima/macOS capability floor,
2. the status of `vsock-proxy` and transport assumptions needed by the
   supported mode,
3. the breakglass classification for direct guest and host-bypass workflows,
4. the supported replacement rule that normal operators should start from
   Substrate-owned commands.

This slice must **not**:

1. implement transport changes,
2. implement mount, listener, or guest-unit changes,
3. perform repo-wide docs cutover,
4. widen into runtime or provisioning edits,
5. silently absorb Slice `03`, `07`, `08`, `10`, `11`, or `12`.

Default execution boundary:

1. feature-local slice docs under
   `macos-hardening/macos-hardened-same-user-lima/spec/`
2. phase-0 milestone wording only if alignment with the slice requires it

Treat edits to `docs/WORLD.md`,
`docs/reference/world/platforms/macos-lima-setup.md`, or shell/runtime code as
scope expansion unless the orchestrator can point to a direct contradiction that
the user explicitly approves for this slice.

## Execution packets

### Packet 1: Supported environment contract freeze

Session goal:

1. freeze the supported Lima/macOS capability contract,
2. freeze the version-floor decision criteria,
3. ground the contract in official Lima sources plus current repo assumptions.

#### Tasks

- [ ] Task 1.1: Confirm the authority stack and source gate
  - Acceptance: the implementation pass explicitly grounds itself in
    `EXECUTION-RUBRIC.md`, `ROADMAP.md`, phase `0`, milestone `0.2`,
    `DESIGN-supported-mode-and-breakglass-taxonomy.md`,
    `DESIGN-macos-lima-transport-contract.md`, and the official Lima lifecycle,
    VM-type, VZ, port-forwarding, mount, breaking-changes, and `limactl shell`
    docs.
  - Verify:
    - manual authority review
    - manual official-source review
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md`

- [ ] Task 1.2: Freeze the supported environment contract in precise lifecycle/capability language
  - Acceptance: the slice replaces vague “recent Lima” wording with one
    explicit environment contract that accounts for `vmType: "vz"`, supported
    macOS/Lima capability assumptions, and the repo’s current forwarding/mount
    posture without widening into full transport design.
  - Verify:
    - `rg -n "recent Lima|v2.x|13\\.0|13\\.5|vmType|vz|vsock-proxy" macos-hardening/macos-hardened-same-user-lima/spec`
    - manual review against `scripts/mac/lima/substrate.yaml`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope/milestone-0-2-lima-version-and-breakglass-contract-sow.md` only if alignment is required

### Packet 1 checkpoint

Packet `1` is complete only when:

1. the supported environment contract is explicit,
2. official Lima sources back the version-sensitive claims,
3. the slice has not drifted into full transport-unification scope.

Do not start Packet `2` until Packet `1` is coherent.

### Packet 2: Breakglass matrix and supported replacement rule

Session goal:

1. classify direct guest and host-bypass workflows explicitly,
2. define the supported replacement path for normal operators,
3. classify compatibility probes without promoting them to the hardened default.

#### Tasks

- [ ] Task 2.1: Freeze the breakglass classification for direct guest and host-bypass workflows
  - Acceptance: the slice explicitly classifies direct `limactl shell`, direct
    guest `systemctl`, direct guest socket curls, and host-side
    `SUBSTRATE_WORLD_SOCKET` override use, using the Slice `01` taxonomy
    without inventing new categories.
  - Verify:
    - `rg -n "limactl shell|SUBSTRATE_WORLD_SOCKET|breakglass|degraded-but-supported|supported" macos-hardening/macos-hardened-same-user-lima/spec`
    - manual review against feature-local and phase-0 docs
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md`
    - milestone `0.2` only if alignment is required

- [ ] Task 2.2: Freeze the supported replacement rule and compatibility-path framing
  - Acceptance: the slice states that normal lifecycle, diagnostics, and
    validation flows should start from Substrate-owned commands, and it
    classifies host TCP `17788` and stale `7788` behavior as compatibility or
    breakglass material rather than the supported default.
  - Verify:
    - `rg -n "17788|7788|host TCP|compatibility|Substrate-owned commands|gateway status|world doctor|host doctor" macos-hardening/macos-hardened-same-user-lima/spec`
    - manual review against `crates/world-mac-lima/src/lib.rs`,
      `crates/shell/src/execution/platform/macos.rs`, and
      `crates/shell/src/builtins/world_gateway.rs`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md`

### Packet 2 checkpoint

Packet `2` is complete only when:

1. breakglass workflows are explicitly classified,
2. the supported operator story clearly starts from Substrate-owned commands,
3. retained compatibility probes are not mistaken for the supported default,
4. the slice still remains feature-local and docs-first.

Do not start Packet `3` until Packet `2` verification is green.

### Packet 3: Explicit deferrals and next-slice handoff clarity

Session goal:

1. validate that Slice `02` stayed narrow,
2. leave a clean handoff to Slice `03` and Slice `12`,
3. keep the short-prompt planning path reusable.

#### Tasks

- [ ] Task 3.1: Validate explicit deferrals to later slices
  - Acceptance: the touched docs explicitly leave canonical transport
    unification to Slice `03`, runtime parity convergence to later phase-1
    slices, and repo-wide breakglass/docs cutover to Slice `12`.
  - Verify:
    - `rg -n "Slice 03|Slice 12|transport|docs cutover|mount|listener|unit|lifecycle" macos-hardening/macos-hardened-same-user-lima/spec`
  - Files:
    - `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-02-lima-version-floor-and-breakglass-contract.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-02.md`
    - `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-02.md`

- [ ] Task 3.2: Final diff and coherence review
  - Acceptance: the final diff is narrow, official-source-backed claims are
    visible, and a future short prompt can identify Slice `03` cleanly from the
    landed materials without reopening Slice `02`.
  - Verify:
    - `git diff --stat -- macos-hardening/macos-hardened-same-user-lima/spec macos-hardening/macos-hardened-same-user-lima/phase-0-security-contract-and-scope`
    - manual coherence review
  - Files:
    - all files touched by this slice only as required by final cleanup

### Packet 3 checkpoint

Packet `3` is complete only when:

1. Slice `02` stayed bounded,
2. Slice `03` remains the next honest transport seam,
3. Slice `12` remains the docs-cutover seam,
4. the planning stack is coherent enough for a short future prompt to continue
   without hidden assumptions.

## Prompt artifact expectation

After Packet `3` planning is stable, the slice should have one ready-to-paste
prompt artifact for fresh orchestration sessions. That prompt artifact should:

1. provide one prompt per packet,
2. require source-driven official-Lima verification before any contract change,
3. require a fresh GPT-5.4 high implementation subagent using
   `$incremental-implementation` for any accepted doc edits,
4. require a fresh GPT-5.4 high review subagent using
   `$code-review-and-quality`,
5. require fix subagents when review finds issues,
6. require commits between implementation, review-driven fix rounds, and the
   next packet boundary.
