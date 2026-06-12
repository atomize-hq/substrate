# PLAN-05: Backend Policy Input Parity

Source spec:
- [`SPEC-05-backend-policy-input-parity.md`](./SPEC-05-backend-policy-input-parity.md)

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-policy-input-parity.md`](./design/DESIGN-macos-policy-input-parity.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)

Prior slice authority:
- [`SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](./SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)
- [`PLAN-04.md`](./PLAN-04.md)
- [`TASKS-04.md`](./TASKS-04.md)

Plan type: repo-first phase-1 backend-contract parity slice for
`macos-hardened-same-user-lima`
Phase: `PLAN`
Status: draft plan

## Plan summary

The next honest seam is Slice `05`: remove backend-local policy synthesis from
the macOS Lima backend and make backend-mediated execution consume the same
shell-resolved policy/world inputs that direct routed paths already use.

This plan should produce a bounded landing that:

1. freezes one shared backend-facing parity carrier,
2. keeps shell/broker policy resolution authoritative,
3. removes permissive backend-local `PolicySnapshotV3` synthesis,
4. gives `apply_policy(...)` a real parity meaning,
5. leaves routed-path-first doctor/smoke/docs truth to Slice `06`.

## Packet 1 live gate and frozen carrier decision

Packet `1` should treat the GitNexus gate as part of the slice contract, not as
optional bookkeeping.

Live 2026-06-12 repo-truth confirmation before Slice `05` code edits:

1. `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status` initially came
   back stale (`Indexed commit: 2056619`, `Current commit: 30d2934`), so the
   required `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus analyze`
   refresh had to run before the symbol gate was trustworthy.
2. After the refresh, `gitnexus context` resolved all four Packet `1` symbols:
   - `world_api::WorldSpec`
   - `world_api::ExecRequest`
   - `world_mac_lima::MacLimaBackend::convert_exec_request`
   - `world_mac_lima::MacLimaBackend::apply_policy`
3. The required exact impact commands surfaced the contract blast radius:
   - `WorldSpec` = `CRITICAL`, 52 upstream impacts
   - `ExecRequest` = `HIGH`, 7 upstream impacts
   - the exact required function-level impact commands for
     `convert_exec_request` and `apply_policy` returned target-not-found, so
     the packet should record that lookup quirk explicitly instead of treating
     it as a clean bill of health
4. Follow-up impact checks using the exact UIDs returned by `gitnexus context`
   showed `convert_exec_request` at `LOW` risk and `apply_policy` at `LOW`
   risk, confirming that the real Packet `2` / `3` blast radius lives on the
   shared contract seam rather than those local methods alone.

Frozen Packet `1` carrier decision:

1. the backend-facing parity carrier should be `WorldSpec`-owned,
2. if Packet `2` groups fields, it should do so through one adjacent shared
   backend type nested under `WorldSpec`,
3. `ExecRequest` should remain command-shaped unless later proof shows a
   genuinely per-execution parity delta that cannot be represented through the
   session policy seam.

## Default landing boundary

Unless execution proves there is an immediate contradiction that must be fixed,
this slice should land within:

1. `macos-hardening/macos-hardened-same-user-lima/spec/SPEC-05-backend-policy-input-parity.md`
2. `macos-hardening/macos-hardened-same-user-lima/spec/PLAN-05.md`
3. `macos-hardening/macos-hardened-same-user-lima/spec/TASKS-05.md`
4. `crates/world-api/src/lib.rs`
5. `crates/world-mac-lima/src/lib.rs`
6. `crates/shell/src/execution/policy_snapshot.rs` only if the parity carrier
   requires a shared builder/helper update
7. `crates/shell/src/execution/routing/dispatch/world_ops.rs` only if a
   backend-facing request/spec builder must be aligned with the widened
   contract
8. `crates/shell/src/repl/async_repl.rs` only if persistent-session bootstrap
   needs the same carrier seam
9. `crates/shell/src/builtins/world_gateway.rs` only if a shared backend-facing
   helper is reused there
10. `crates/replay/src/replay/executor.rs` only as shared-contract fallout if
    `WorldSpec` / `ExecRequest` shape changes require it
11. targeted tests in the same files or nearby existing test coverage as needed

By default this slice should **not** widen into:

1. `scripts/mac/lima-doctor.sh`, `scripts/mac/smoke.sh`,
   `docs/WORLD.md`, `docs/reference/world/platforms/macos-lima-setup.md`, or
   `docs/USAGE.md`,
2. a reopened transport contract in `crates/world-mac-lima/src/transport.rs`,
   `crates/world-mac-lima/src/forwarding.rs`, or shell transport-consumer code
   beyond tiny compile/alignment fallout,
3. broker policy-language redesign,
4. cross-backend semantic work beyond the minimum shared-contract fallout
   required to keep the repo coherent.

## Skill gate resolution

Per [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md):

1. `spec-driven-development` is required for this slice,
2. `source-driven-development` is light / targeted rather than heavy by
   default.

Plan consequence:

1. use live repo truth as the primary authority,
2. keep the design discussion centered on contract propagation rather than
   external platform semantics,
3. pull targeted external docs only if implementation widens into guest
   lifecycle or transport semantics that are not already repo-local.

## Official source set this plan must use

Default required official source set:

1. none beyond repo-local authority

Conditional official sources:

1. if `apply_policy(...)` implementation unexpectedly depends on guest service
   reload or socket-activation semantics, fetch only the exact official pages
   needed and cite them in the implementation closeout,
2. if the carrier choice unexpectedly requires transport- or forwarding-level
   claims, treat that as scope pressure and verify the exact official source
   pages before proceeding.

## Major components and dependencies

1. **shared parity carrier**
   - freeze the one backend-facing carrier that can hold resolved policy/world
     inputs
   - prefer widening shared backend seams over preserving macOS-only synthesis
2. **authoritative builder alignment**
   - connect the shell-side policy/world-input authority to the widened carrier
     without creating a second policy-resolution path
3. **backend parity implementation**
   - make `MacLimaBackend::convert_exec_request(...)` consume the carried
     truth
   - make `MacLimaBackend::apply_policy(...)` semantically real
4. **validation and handoff clarity**
   - prove typed contract coherence and backend parity
   - leave Slice `06` with a truthful backend foundation

Dependency order:

1. freeze the carrier seam first,
2. align authoritative builders second,
3. implement backend parity semantics third,
4. run shared-consumer fallout fixes and validation last.

## Locked decisions

### What this slice changes

1. It defines one shared backend-facing carrier for resolved policy/world
   inputs.
2. It removes permissive backend-local policy synthesis from the macOS backend.
3. It makes backend-mediated macOS execution carry `world_network` and related
   world-input truth instead of dropping it.
4. It gives `apply_policy(...)` a real semantic role for backend session state.

### What this slice does not change

1. no routed-path-first docs/script cutover,
2. no transport-contract redesign,
3. no broker policy-schema redesign,
4. no same-user ownership-boundary hardening,
5. no broad Windows/backend feature work beyond contract fallout.

## Implementation order

### Packet 1: Freeze the carrier seam and symbol-impact gate

Goal:

1. confirm the authority stack and repo-floor gap,
2. choose the explicit backend-facing parity carrier seam,
3. inventory the exact symbols whose change would widen the blast radius.

Primary touch surface:

1. `SPEC-05-backend-policy-input-parity.md`
2. `PLAN-05.md`
3. the shared backend contract docs and notes only as needed

Why first:

1. the slice should not start “threading fields around” without first deciding
   where authoritative parity state is supposed to live,
2. shared-contract widening affects multiple crates, so the blast radius has to
   be explicit before code edits begin.

Frozen Packet `1` decision from the required live gate:

1. default Packet `2` to a `WorldSpec`-owned parity carrier,
2. keep `ExecRequest` command-scoped unless later proof shows an unavoidable
   per-execution parity delta,
3. if field grouping is needed, use one adjacent shared backend type nested
   under `WorldSpec` rather than duplicating top-level parity fields across
   both structs.

Why this is the bounded choice:

1. `apply_policy(...)` already accepts `WorldSpec`, so a `WorldSpec`-owned
   carrier gives Packet `3` one honest place to reconcile or reject drift,
2. `MacLimaBackend` already stores `fs_mode` from `WorldSpec`, so widening the
   backend-held session state follows the existing contract,
3. widening `ExecRequest` first would leave `apply_policy(...)` semantically
   underpowered unless Slice `05` created a second policy source of truth.

Verification checkpoint:

1. the exact Packet `2` / `3` symbol set is explicit:
   - `world_api::WorldSpec`
   - `world_api::ExecRequest`
   - `world_mac_lima::MacLimaBackend::convert_exec_request`
   - `world_mac_lima::MacLimaBackend::apply_policy`
   - any shell-side builder/helper that must feed the widened carrier
2. GitNexus commands are pinned to
   `GITNEXUS_HOME=/tmp/gitnexus-ff74-only`,
3. any `HIGH` or `CRITICAL` impact result is surfaced before implementation
   proceeds,
4. the slice still has not absorbed Slice `06` docs/script cutover.

Recorded Packet `1` GitNexus gate results:

1. `WorldSpec` impact = `CRITICAL`
   - 52 upstream impacts
   - direct/shared fallout includes replay, `world-service`, `world`,
     `world-mac-lima`, and shell bootstrap/helper consumers
   - highest-signal affected processes include
     `build_agent_client_and_member_dispatch_request_impl`,
     `build_agent_client_and_request_impl`, and `handle_legacy_start`
2. `ExecRequest` impact = `HIGH`
   - 7 upstream impacts
   - direct fallout centers on replay, the macOS smoke example, and Windows
     WSL tests
3. the required exact function-level impact commands for
   `convert_exec_request` and `apply_policy` returned target-not-found while
   `gitnexus context` resolved both symbols; treat that as a lookup quirk and
   keep Packet `1` bounded to seam freezing plus blast-radius documentation.

### Packet 2: Widen the shared backend contract

Goal:

1. add the minimum shared contract fields or adjacent carrier needed for
   backend policy parity,
2. keep serde/default behavior coherent,
3. limit compile fallout to intentional shared consumers.

Primary touch surface:

1. `crates/world-api/src/lib.rs`
2. `crates/replay/src/replay/executor.rs` only if widened structs require it
3. nearby tests in `world-api` and shared consumers

Why second:

1. the backend cannot consume authoritative policy/world truth until the shared
   seam can carry it,
2. contract widening has to settle before backend implementation details are
   coded.

Verification checkpoint:

1. the shared carrier can represent the required parity inputs explicitly,
2. round-trip tests remain coherent,
3. no macOS-only hidden side channel was added to avoid widening the contract.

### Packet 3: Implement backend parity semantics in `world-mac-lima`

Goal:

1. make `convert_exec_request(...)` consume the widened carrier,
2. remove permissive local policy synthesis,
3. make `apply_policy(...)` update or reconcile backend session state in an
   honest way.

Primary touch surface:

1. `crates/world-mac-lima/src/lib.rs`
2. any shell builder/helper surface that must feed the widened carrier

Why third:

1. backend logic should be written only after the carrier contract is frozen,
2. keeping this packet separate makes it obvious whether the slice widened into
   a broker or transport redesign.

Verification checkpoint:

1. `convert_exec_request(...)` no longer fabricates a permissive
   `PolicySnapshotV3`,
2. `world_network` no longer disappears on the backend-mediated path,
3. `apply_policy(...)` is no longer explanation-hostile or semantically empty,
4. `shared_world` propagation remains intact.

### Packet 4: Targeted validation and clean handoff to Slice `06`

Goal:

1. run targeted tests and scope checks,
2. run GitNexus change-scope verification before commits,
3. leave the next seam clearly as routed-path-first readiness/docs truth.

Primary touch surface:

1. Slice `05` docs,
2. targeted tests and limited fallout surfaces only as required by the parity
   implementation

Why fourth:

1. the slice is only honest once the shared-contract and backend changes are
   validated together,
2. this packet prevents Slice `05` from quietly absorbing Slice `06`.

Verification checkpoint:

1. shared-contract and backend tests pass,
2. GitNexus reports only the expected symbol/process impact,
3. replay/shared-consumer fallout is either intentionally updated or remains
   untouched,
4. Slice `06` remains the next honest readiness/docs seam.

## Risks and mitigations

### Risk 1: shared-contract widening ripples farther than expected

Mitigation:

1. freeze the carrier seam before coding,
2. treat replay or shared-consumer fallout as explicit and bounded,
3. surface any `HIGH` / `CRITICAL` GitNexus results before proceeding.

### Risk 2: the repo ends up with two policy sources of truth anyway

Mitigation:

1. keep shell/broker resolution authoritative,
2. forbid backend-local permissive synthesis,
3. reject env-only or helper-local reconstruction as the parity story.

### Risk 3: `apply_policy(...)` becomes technically non-empty but still dishonest

Mitigation:

1. require explicit semantics for what session state is stored or reapplied,
2. make tests prove observable policy parity behavior rather than just field
   plumbing.

### Risk 4: Slice `05` absorbs Slice `06`

Mitigation:

1. keep scripts and top-level docs out of the default landing boundary,
2. stop at backend parity and hand off readiness/docs truth as the next slice.

## Parallelism guidance

This slice is only lightly parallelizable.

Parallelizable work:

1. authority review across Phase `1`, Milestone `1.2`, Slice `04`, and the
   parity design doc,
2. inventorying the exact `WorldSpec` / `ExecRequest` / `convert_exec_request`
   / `apply_policy` call graph and shared-consumer fallout,
3. recording `gitnexus status`, `gitnexus context`, and `gitnexus impact`
   results for the Packet `1` symbol set.

Sequential work:

1. freezing the carrier seam,
2. widening the shared contract,
3. implementing backend parity semantics,
4. running targeted validation and GitNexus detect-changes.

## Exit criteria

This plan is ready to hand off to `TASKS-05` only when:

1. the slice remains bounded to backend policy/world-input parity,
2. the shared carrier seam is explicit enough to review,
3. the code touch set is small enough to avoid silently absorbing Slice `06`,
4. GitNexus impact/detect-changes gates are part of the execution contract,
5. the next seam after implementation is still Slice `06`.
