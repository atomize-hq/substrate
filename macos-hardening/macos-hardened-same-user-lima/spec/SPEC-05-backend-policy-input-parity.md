# Spec: Slice 05 Backend Policy Input Parity

Source phase authority:
- [`../phase-1-runtime-parity-foundation/README.md`](../phase-1-runtime-parity-foundation/README.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md)

Source execution authority:
- [`../EXECUTION-RUBRIC.md`](../EXECUTION-RUBRIC.md)
- [`../ROADMAP.md`](../ROADMAP.md)

Primary design inputs:
- [`design/DESIGN-macos-policy-input-parity.md`](./design/DESIGN-macos-policy-input-parity.md)
- [`design/DESIGN-macos-lima-transport-contract.md`](./design/DESIGN-macos-lima-transport-contract.md)
- [`design/DESIGN-supported-mode-and-breakglass-taxonomy.md`](./design/DESIGN-supported-mode-and-breakglass-taxonomy.md)

Neighboring slice authority:
- [`SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md`](./SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md)
- [`PLAN-04.md`](./PLAN-04.md)
- [`TASKS-04.md`](./TASKS-04.md)
- [`../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md`](../phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md)

Required official source set for this slice:
- none by default
- if implementation widens into Lima guest lifecycle, listener, or service
  reapply semantics, fetch only the targeted official source pages needed for
  that decision and cite them explicitly in the implementation closeout

Phase: `SPECIFY`
Status: draft slice authority
Slice focus: remove backend-local policy synthesis from the macOS Lima backend
and make backend-mediated execution consume the same shell-resolved policy and
world-routing truth that direct routed paths already use.

## Assumptions

ASSUMPTIONS I'M MAKING:

1. The latest landed planning authority is Slice `04`, and its spec/plan/tasks
   explicitly defer backend policy input parity to Slice `05`.
2. Per [`../ROADMAP.md`](../ROADMAP.md), Slice `05` remains the next honest
   seam after Slice `04`, while Slice `06` remains routed-path-first
   doctor/smoke/readiness truth plus docs/script cutover.
3. Live repo truth still shows that
   `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-api/src/lib.rs`
   `WorldSpec` carries reuse, network, and filesystem basics, but does not yet
   carry the full broker-resolved policy snapshot or an equivalent typed parity
   carrier for backend-mediated macOS execution.
4. Live repo truth still shows that
   `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-api/src/lib.rs`
   `ExecRequest` carries command, cwd, env, PTY, shared-world, and member
   dispatch data, but does not yet carry the same policy/world-routing contract
   that direct shell request builders send through `transport_api_types`.
5. Live repo truth still shows that
   `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs`
   `MacLimaBackend::convert_exec_request(...)` synthesizes a permissive
   `PolicySnapshotV3`, forwards `shared_world`, and drops `world_network`.
6. Live repo truth still shows that
   `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/world-mac-lima/src/lib.rs`
   `MacLimaBackend::apply_policy(...)` only stores `fs_mode` and remains a
   semantic no-op for backend policy parity.
7. Shell-side direct routed paths already resolve and forward authoritative
   policy/world inputs in:
   - `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/policy_snapshot.rs`
   - `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/execution/routing/dispatch/world_ops.rs`
   - `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/repl/async_repl.rs`
   - `/Users/spensermcconnell/.codex/worktrees/ff74/substrate/crates/shell/src/builtins/world_gateway.rs`
8. This slice may widen shared backend contracts and update compile/test fallout
   in consumers like replay, but it should not widen into Slice `06`
   readiness/docs cutover, a transport redesign, or a broker policy-schema
   rewrite.

If any of these are wrong, correct them before implementation.

## Packet 1 live confirmation and frozen carrier decision

Packet `1` was re-grounded from live repo truth and the required GitNexus gate
on 2026-06-12 before any Slice `05` code edits:

1. `GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status` reported the
   index up to date at commit `30d2934`.
2. `gitnexus context` resolved all four Packet `1` symbols:
   - `world_api::WorldSpec`
   - `world_api::ExecRequest`
   - `world_mac_lima::MacLimaBackend::convert_exec_request`
   - `world_mac_lima::MacLimaBackend::apply_policy`
3. The required exact `gitnexus impact` commands surfaced the shared-contract
   blast radius clearly enough to freeze the seam:
   - `WorldSpec` came back `CRITICAL` with 52 upstream impacts spanning replay,
     `world-service`, `world`, `world-mac-lima`, and shell bootstrap helpers.
   - `ExecRequest` came back `HIGH` with 7 upstream impacts centered on replay,
     the macOS smoke example, and Windows WSL tests.
   - the required exact function-level impact commands for
     `convert_exec_request` and `apply_policy` returned target-not-found even
     though `gitnexus context` resolved both symbols. Treat that as a lookup
     quirk, not as permission to skip the broader contract blast-radius gate.

Frozen Packet `1` carrier decision:

1. the backend-facing parity carrier should live on `WorldSpec`, not on
   `ExecRequest`,
2. if Packet `2` needs field grouping, that grouping should be one adjacent
   shared backend type nested under `WorldSpec` rather than duplicated across
   both `WorldSpec` and `ExecRequest`,
3. `ExecRequest` should remain command-specific (`cmd`, `cwd`, `env`, `pty`,
   `shared_world`, `member_dispatch`) unless later proof shows a genuinely
   per-execution parity delta that cannot be represented through the session
   policy seam.

Reasoning from live repo truth:

1. `apply_policy(...)` already accepts `WorldSpec`, so a `WorldSpec`-owned
   carrier gives Packet `3` one honest place to reapply, reconcile, or
   fail-close policy state.
2. `MacLimaBackend` already caches `fs_mode` from `WorldSpec`, so widening the
   backend-held session state is the straightest extension of the current
   contract.
3. widening `ExecRequest` first would give `convert_exec_request(...)` a
   per-command carrier, but it would still leave `apply_policy(...)` without an
   honest shared source of truth unless Slice `05` introduced a second policy
   store.
4. shell direct routed paths already compute `policy_snapshot` and
   `world_network`; Packet `2` should feed those authoritative values into the
   `WorldSpec`-owned carrier rather than teaching `world-mac-lima` to infer
   them locally.

## Objective

Make backend-mediated macOS Lima execution consume one typed policy/world-input
contract derived from shell/broker truth instead of inventing its own fallback.

This slice is complete only when a reviewer can answer, without guessing:

1. where the authoritative policy snapshot and world-routing truth are resolved
   for backend-mediated macOS execution,
2. which typed carrier transports that truth through the world backend seam,
3. how `MacLimaBackend::convert_exec_request(...)` uses that carried truth
   without synthesizing a permissive local snapshot,
4. what `MacLimaBackend::apply_policy(...)` now means for a reused or already
   booted macOS world session,
5. how shared-world propagation remains intact while policy parity is fixed,
6. what still remains deferred to Slice `06`.

## Frozen in this slice

This slice freezes only:

1. one explicit backend-facing contract seam that can carry broker-resolved
   policy/world inputs into backend-mediated macOS execution,
2. the rule that backend-mediated macOS execution must not fabricate a
   permissive `PolicySnapshotV3` from `WorldFsMode` alone,
3. the rule that `world_network` and closely related world-routing inputs must
   not disappear when the macOS backend path is used,
4. the minimum `apply_policy(...)` semantics needed so backend session state is
   no longer explanation-hostile or silently stale by design,
5. preservation of the already-landed `shared_world` propagation contract while
   the parity fix lands,
6. the minimum test and evidence surface needed to prove the parity claim.

## Deferred by design

This slice intentionally does **not** freeze:

1. routed-path-first doctor/smoke/readiness truth or docs/script cutover owned
   by Slice `06`,
2. a reopened transport contract or transport-consumer redesign already owned
   by Slices `03` and `04`,
3. a broker policy-language redesign,
4. same-user ownership-boundary hardening, ingress narrowing, listener
   removal, or guest-unit sandbox work from later phases,
5. semantic parity changes for other backends beyond the minimum shared
   contract fallout required to keep the repo coherent.

## Why this slice exists

Phase `1` already says the remaining parity gap is narrower than older docs
imply. Shell-side direct routed requests already compute and forward the
relevant policy/world inputs; the backend-mediated Lima path does not.

Live repo truth shows the remaining gap clearly:

1. `policy_snapshot::bootstrap_world_spec(...)` still creates a macOS bootstrap
   `WorldSpec` with basic network/filesystem values but without a full parity
   carrier.
2. `MacLimaBackend::convert_exec_request(...)` still fabricates a permissive
   `PolicySnapshotV3` locally instead of reusing shell-resolved truth.
3. `MacLimaBackend::convert_exec_request(...)` still sets `world_network: None`
   even though direct shell request builders already compute `world_network`.
4. `MacLimaBackend::apply_policy(...)` still stores only `fs_mode`, leaving the
   backend-mediated path without an honest policy reapply story.
5. `crates/replay/src/replay/executor.rs` and other `world_api` consumers may
   need limited compile/test fallout updates if the shared backend contract
   widens, which is preferable to preserving hidden macOS-only synthesis.

If Slice `05` does not land before Slice `06`, the readiness/docs cutover would
be forced to tell a cleaner story around a backend path that still lies about
policy parity.

## Commands

This is a repo-first backend-contract slice. Start by proving the authority
stack, the current carrier gap, and the symbol set that future implementation
will have to touch.

```bash
# Review the phase-1 authority stack and adjacent slices
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/README.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-2-policy-application-parity-sow.md
sed -n '1,260p' macos-hardening/macos-hardened-same-user-lima/phase-1-runtime-parity-foundation/milestone-1-3-doctor-smoke-readiness-parity-sow.md
sed -n '1,220p' macos-hardening/macos-hardened-same-user-lima/spec/design/DESIGN-macos-policy-input-parity.md
sed -n '1,220p' macos-hardening/macos-hardened-same-user-lima/spec/SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/PLAN-04.md
sed -n '1,240p' macos-hardening/macos-hardened-same-user-lima/spec/TASKS-04.md

# Inspect the current carrier gap in repo truth
sed -n '100,220p' crates/world-api/src/lib.rs
sed -n '160,215p' crates/shell/src/execution/policy_snapshot.rs
sed -n '418,640p' crates/world-mac-lima/src/lib.rs
sed -n '1288,1498p' crates/shell/src/execution/routing/dispatch/world_ops.rs
sed -n '8470,8565p' crates/shell/src/repl/async_repl.rs
sed -n '320,380p' crates/replay/src/replay/executor.rs

# Inventory the relevant parity symbols and current drift
rg -n "WorldSpec|ExecRequest|convert_exec_request|apply_policy|PolicySnapshotV3|world_network|world_fs_mode|shared_world" \
  crates/world-api/src/lib.rs \
  crates/world-mac-lima/src/lib.rs \
  crates/shell/src/execution/policy_snapshot.rs \
  crates/shell/src/execution/routing/dispatch/world_ops.rs \
  crates/shell/src/repl/async_repl.rs \
  crates/shell/src/builtins/world_gateway.rs \
  crates/replay/src/replay/executor.rs

# Record GitNexus status and impact for the contract symbols before editing
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus status
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context WorldSpec --repo substrate --file crates/world-api/src/lib.rs
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context ExecRequest --repo substrate --file crates/world-api/src/lib.rs
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context convert_exec_request --repo substrate --file crates/world-mac-lima/src/lib.rs
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus context apply_policy --repo substrate --file crates/world-mac-lima/src/lib.rs
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Struct:crates/world-api/src/lib.rs:WorldSpec' --repo substrate --direction upstream --depth 3 --include-tests
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Struct:crates/world-api/src/lib.rs:ExecRequest' --repo substrate --direction upstream --depth 3 --include-tests
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/world-mac-lima/src/lib.rs:convert_exec_request' --repo substrate --direction upstream --depth 3 --include-tests
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus impact 'Function:crates/world-mac-lima/src/lib.rs:apply_policy' --repo substrate --direction upstream --depth 3 --include-tests

# Targeted validation for the implementation slice
cargo test -p world-api shared_world_contract_round_trips_with_canonical_shape -- --nocapture
cargo test -p world-mac-lima convert_exec_request_propagates_env_fs_mode -- --nocapture
cargo test -p world-mac-lima -- --nocapture
cargo test -p shell world_network_policy_canonicalizes_snapshot_net_allowed -- --nocapture
cargo test -p shell world_network_policy_requests_isolation_for_restrictive_allowlist -- --nocapture
cargo fmt --all -- --check
```

If the shared backend contract widens enough to touch replay, add:

```bash
cargo test -p replay -- --nocapture
```

If GitNexus reports a stale index before symbol-impact work begins, refresh it
first:

```bash
GITNEXUS_HOME=/tmp/gitnexus-ff74-only npx gitnexus analyze
```

## Project structure

This slice should stay grounded to these directories and file families:

```text
macos-hardening/macos-hardened-same-user-lima/
├── ROADMAP.md
├── EXECUTION-RUBRIC.md
├── phase-1-runtime-parity-foundation/
│   ├── README.md
│   ├── milestone-1-2-policy-application-parity-sow.md
│   └── milestone-1-3-doctor-smoke-readiness-parity-sow.md
└── spec/
    ├── SPEC-04-pty-non-pty-doctor-and-readiness-transport-convergence.md
    ├── SPEC-05-backend-policy-input-parity.md
    ├── PLAN-05.md
    ├── TASKS-05.md
    └── design/
        ├── DESIGN-macos-policy-input-parity.md
        ├── DESIGN-macos-lima-transport-contract.md
        └── DESIGN-supported-mode-and-breakglass-taxonomy.md

crates/world-api/
└── src/lib.rs                                  → shared backend contract seam

crates/world-mac-lima/
└── src/lib.rs                                  → current backend-local synthesis and apply_policy gap

crates/shell/
├── src/execution/policy_snapshot.rs            → authoritative policy/world input resolution helpers
├── src/execution/routing/dispatch/world_ops.rs → direct routed request builders
├── src/repl/async_repl.rs                      → persistent-session request authority
└── src/builtins/world_gateway.rs               → gateway request authority

crates/replay/
└── src/replay/executor.rs                      → secondary shared-contract consumer if types widen
```

## Code style

This slice should centralize parity around typed carriers and remove backend
guessing.

Required style:

1. prefer one explicit typed carrier for resolved policy/world inputs instead of
   reconstructing them from `fs_mode`, env, or backend-local defaults,
2. keep shell/broker-resolved truth authoritative rather than re-resolving
   policy inside `world-mac-lima`,
3. preserve existing `shared_world` semantics and naming rather than creating a
   parallel macOS-only parity path,
4. if shared contract types widen, use serde defaults and field naming that
   keep the contract explanation-ready,
5. comments should explain why a field exists for backend parity, not hand-wave
   that env injection alone is “close enough,”
6. do not create a second hidden policy source of truth in helper methods or
   transient structs.

Example shape:

```rust
pub struct WorldSpec {
    // existing fields...
    pub backend_policy_inputs: Option<BackendPolicyInputs>,
}

pub struct BackendPolicyInputs {
    pub policy_snapshot: transport_api_types::PolicySnapshotV3,
    pub world_network: transport_api_types::WorldNetworkRoutingV1,
}

fn convert_exec_request(&self, req: &ExecRequest, spec: &WorldSpec) -> ExecuteRequest {
    // consume carried policy/world-input truth from spec/backend state instead
    // of synthesizing it
}
```

The exact nested type name may differ, but Packet `1` freezes the location:
there should be one obvious `WorldSpec`-owned backend-facing place where
resolved parity inputs live.

## Testing strategy

This slice is a backend-contract and parity slice. Validation should emphasize
typed carrier integrity, backend behavior, and bounded fallout.

Validation levels:

1. **Contract serialization / defaults**
   - prove any widened shared types remain coherent and round-trip cleanly
2. **Backend conversion parity**
   - prove `convert_exec_request(...)` consumes carried truth instead of
     synthesizing a permissive local fallback
3. **Backend apply-policy semantics**
   - prove `apply_policy(...)` is no longer a semantic no-op
4. **Shell authority preservation**
   - prove shell-side policy/world-input resolution remains authoritative if
     any builder/helper is touched
5. **Shared-consumer fallout**
   - prove replay or other shared consumers stay coherent if the shared types
     widen
6. **Scope validation**
   - prove the slice did not absorb Slice `06` docs/script cutover or reopen
     the transport contract

## Boundaries

- Always:
  - keep shell/broker-resolved policy truth authoritative
  - remove backend-local permissive policy synthesis rather than documenting
    around it
  - preserve `shared_world` propagation while fixing policy parity
  - treat GitNexus impact analysis as mandatory before editing touched symbols
    and `gitnexus_detect_changes()` as mandatory before committing
  - keep Slice `06` reserved for routed-path-first readiness/docs cutover
- Ask first:
  - widening the change into helper scripts or top-level docs
  - introducing backend parity semantics that require cross-backend behavioral
    changes rather than compile/test fallout only
  - pulling in external platform docs because implementation has unexpectedly
    widened into transport or guest lifecycle semantics
- Never:
  - preserve a permissive `PolicySnapshotV3` fallback as the macOS backend’s
    “good enough” story
  - silently absorb Slice `06`
  - silently reopen the Slice `03` / Slice `04` transport contract
  - treat env-only injection as a substitute for the typed backend parity
    contract

## Success criteria

Slice `05` is successful when:

1. one explicit backend-facing contract seam carries the policy/world inputs
   needed for backend-mediated macOS parity,
2. `MacLimaBackend::convert_exec_request(...)` no longer fabricates a
   permissive `PolicySnapshotV3` from `WorldFsMode` alone,
3. backend-mediated macOS requests carry the same relevant policy and
   `world_network` truth that direct routed shell paths already compute,
4. `MacLimaBackend::apply_policy(...)` is no longer a semantic no-op,
5. `shared_world` propagation remains intact and test-covered,
6. the slice stays bounded enough that Slice `06` remains the next honest
   seam.

## Remaining open questions for later slices

After Slice `05`, the follow-on questions should be narrower and more honest:

1. which doctor/status JSON fields, if any, should surface the effective
   backend policy state to support readiness evidence in Slice `06`?
2. which helper scripts and top-level docs can be safely rewritten to lead with
   routed doctor/gateway/smoke validation once backend parity is real?
3. whether any compatibility-only transport or guest-direct diagnostics can be
   removed entirely only after the readiness/docs cutover lands.
