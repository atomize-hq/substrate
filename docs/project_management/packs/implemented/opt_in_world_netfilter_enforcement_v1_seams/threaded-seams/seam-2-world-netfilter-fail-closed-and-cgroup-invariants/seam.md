---
seam_id: SEAM-2
seam_slug: world-netfilter-fail-closed-and-cgroup-invariants
status: landed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-world-netfilter-fail-closed-and-cgroup-invariants.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
    - ../../governance/seam-3-closeout.md
  required_threads:
    - THR-02
    - THR-04
  stale_triggers:
    - "Any change to WorldSpec.isolate_network/allowed_domains routing semantics or world-agent request parity"
    - "Any new process-spawn path that bypasses cgroup attach or weakens attach-or-fail behavior under isolate_network"
    - "Any change to WORLD_NETFILTER_ENABLE guard semantics or nftables ruleset shape"
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: passed
    closeout: passed
seam_exit_gate:
  required: true
  planned_location: S3
  status: passed
open_remediations: []
---
# SEAM-2 - `crates/world` enforcement is real, fail-closed, and unavoidable

## Seam Brief (Restated)

- **Goal / value**: turn the published `SEAM-1` routing contract into real world-runtime enforcement so requested isolation either applies correctly or the command fails before an unfiltered process escapes.
- **Type**: platform
- **Scope**
  - In:
    - Convert requested isolation from warn-and-continue behavior to enforce-or-fail behavior.
    - Make deny-all (`allowed_domains=[]`) deny DNS as well as non-DNS egress.
    - Fail closed when `WORLD_NETFILTER_ENABLE` is absent or domain resolution/setup fails.
    - Eliminate cgroup attach escape hatches across overlay, fallback, and direct-exec command paths.
  - Out:
    - Publishing Snapshot V3 and host request contracts (`SEAM-1`).
    - Publishing doctor/diagnostic JSON (`SEAM-4`).
    - Final cross-platform conformance suite and smoke playbooks (`SEAM-5`).
- **Touch surface**:
  - `crates/world/src/session.rs`
  - `crates/world/src/netfilter.rs`
  - `crates/world/src/exec.rs`
  - focused Linux tests in `crates/world` and downstream smoke/test surfaces consumed by `SEAM-5`
- **Verification**:
  - focused unit/logic tests for deny-all and guard behavior in `crates/world`
  - privileged Linux coverage for install/apply/deny-all behavior in an isolated netns/cgroup scope
  - manual macOS Lima smoke evidence captured downstream once runtime behavior lands
- **Basis posture**:
  - Currentness: `current`; the seam now plans against the landed `SEAM-1` routing handoff and the published `SEAM-3` host gate rather than provisional upstream intent.
  - Upstream closeouts assumed:
    - `../../governance/seam-1-closeout.md`
    - `../../governance/seam-3-closeout.md`
  - Required threads:
    - `THR-02`
    - `THR-04`
  - Stale triggers:
    - see `basis.stale_triggers`
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-1` now publishes `C-02` and `C-03` with `promotion_readiness: ready`.
  - Downstream blocked seams:
    - `SEAM-4` needs the runtime failure taxonomy and enablement semantics this seam settles.
    - `SEAM-5` needs the landed fail-closed behavior before privileged/smoke coverage can lock it in.
  - Contracts produced (per `../../threading.md`):
    - none; this seam operationalizes the published `SEAM-1` contracts and emits the `THR-04` safety handoff.
  - Contracts consumed (per `../../threading.md`):
    - `C-02`
    - `C-03`
  - Current upstream handoff:
    - `SEAM-1` publishes the request contract that `isolate_network=true` means "enforce or fail", and the active plan now maps each current runtime escape hatch back to that contract.

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.
- `../../review_surfaces.md` is pack-level orientation only.

## Post-exec outcome

- `S1` and `S2` landed in `crates/world`, and `S3` now records the published closeout in
  `../../governance/seam-2-closeout.md`.
- `THR-04` is now published with concrete failure semantics for missing `WORLD_NETFILTER_ENABLE`, nftables
  install/runtime failures, and cgroup attach failures under requested isolation.
- No seam-local remediations remain open; downstream work continues in `SEAM-4` and `SEAM-5`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: downstream diagnostics and conformance work must consume one explicit record of what fail-closed behavior actually landed, which cgroup attach paths were hardened, and which stale triggers should force revalidation.
- **Expected contracts to publish**:
  - no new contract owner transfer; this seam realizes the operational meaning of `C-02` / `C-03`
- **Expected threads to publish / advance**:
  - `THR-04`: `identified` -> `published`
  - `THR-02`: downstream consumption already revalidated during promotion and must remain stable through landing
- **Likely downstream stale triggers**:
  - any new world execution path that can spawn outside cgroup attach
  - any change to nftables ruleset shape, especially DNS handling for deny-all
  - any change to `WORLD_NETFILTER_ENABLE` failure semantics or error taxonomy
- **Expected closeout evidence**:
  - landed errors replacing warn-and-continue behavior in `session.rs`
  - landed deny-all/no-DNS rule behavior in `netfilter.rs`
  - landed attach-or-fail behavior across bind-mount, fallback, and direct-exec paths
  - focused tests proving those invariants

## Slice index

- `S1` -> `slice-1-fail-closed-netfilter-runtime.md`
- `S2` -> `slice-2-cgroup-attach-invariants-across-exec-paths.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
