---
seam_id: SEAM-2
seam_slug: world-netfilter-fail-closed-and-cgroup-invariants
type: platform
status: exec-ready
execution_horizon: active
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-3-closeout.md
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
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: S3
  status: pending
open_remediations: []
---

# SEAM-2 - `crates/world` enforcement is real, fail-closed, and unavoidable

- **Goal / value**: Make outbound egress enforcement actually effective when requested: deny-all truly denies DNS, hostname resolution failures fail closed, and every executed process is attached to the cgroup scope used by nftables rules (or the command fails).
- **Scope**
  - In:
    - When `WorldSpec.isolate_network=true`, netfilter installation/application is mandatory (no warn-and-continue).
    - Deny-all (`allowed_domains=[]`) installs rules that deny all egress (including DNS) — no implicit DNS allow rules.
    - If a hostname entry cannot be resolved during setup, return an error (fail-closed).
    - Ensure every execution path (PTY and non-PTY) attaches the spawned process to the world cgroup; missing attach becomes a hard error when isolation is requested.
    - Respect `WORLD_NETFILTER_ENABLE=1` safety gate; missing env must be an error when isolation is requested.
  - Out:
    - Defining/normalizing Snapshot V3 inputs (owned by `SEAM-1`).
    - Adding operator-facing config gate and CLI (owned by `SEAM-3`).
- **Primary interfaces**
  - Inputs:
    - `WorldSpec.isolate_network` and `WorldSpec.allowed_domains` (from `SEAM-1` routing).
    - `WORLD_NETFILTER_ENABLE` (world-agent service env).
  - Outputs:
    - Applied nftables/netfilter rules scoped to the correct cgroup/netns.
    - Structured error diagnostics on any enforcement setup failure.
- **Key invariants / rules**:
  - If isolation is requested and cannot be applied, execution fails (no unfiltered escape).
  - cgroup-scoped enforcement must cover all processes; bypass paths must be eliminated or refused under isolate_network.
  - `WORLD_NETFILTER_ENABLE` is mandatory to proceed when isolation is requested.
- **Dependencies**
  - Direct blockers:
    - none; `SEAM-1` now publishes the isolate-network + allowed-domains handoff in closeout.
  - Transitive blockers:
    - installer/service configuration work to actually set `WORLD_NETFILTER_ENABLE=1` in deployed environments
  - Direct consumers:
    - `SEAM-4` doctor/diagnostics should surface last failure reason / enablement state
  - Derived consumers:
    - `SEAM-5` privileged integration tests and macOS smoke
- **Touch surface**:
  - `crates/world/src/session.rs` (isolate_network handling)
  - `crates/world/src/netfilter.rs` (rule install + resolution)
  - Any “direct exec” spawn path that currently bypasses cgroup attach
- **Verification**:
  - Logic-level tests: deny-all produces no DNS allow rules.
  - Privileged integration test (ignored) that verifies rules install and deny-all behavior in an isolated netns/cgroup scope.
  - Manual macOS Lima smoke: deny-all fails ping in both `-c` and REPL.
- **Current blocker posture**:
  - none at the promotion boundary; `SEAM-1` now supplies a passed seam-exit handoff (`promotion_readiness: ready`), and
    this seam’s execution work is now bounded in active slices instead of a provisional future brief.
- **Basis posture**:
  - Currentness: `current`; the active plan is refreshed against landed upstream routing in `governance/seam-1-closeout.md`
    and the published host-gate semantics in `governance/seam-3-closeout.md`.
  - Upstream closeouts assumed:
    - `governance/seam-1-closeout.md`
    - `governance/seam-3-closeout.md`
  - Required threads:
    - `THR-02`
    - `THR-04`
  - Stale triggers:
    - see `basis.stale_triggers`
- **Risks / unknowns**:
  - Risk: cgroup attach invariants may require re-architecting spawn pathways or introducing an exec helper.
  - De-risk plan: enumerate spawn paths in seam-local review; add a hard “attach or fail” guard under isolate_network to catch stragglers early.
- **Rollout / safety**:
  - Enforcement cannot activate without explicit opt-in request + `WORLD_NETFILTER_ENABLE=1`.
- **Downstream decomposition context**:
  - Why this seam is now `active`: `SEAM-1` and `SEAM-3` are both landed with passed seam-exit gates, so the remaining
    critical-path work is the platform/runtime fail-closed implementation.
  - Which threads matter most: `THR-02`, `THR-04`.
  - What the first seam-local review should focus on: cgroup attach coverage, nftables rule correctness, and failure diagnostics that are actionable for operators.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - Tightened semantics for `C-02` under opt-in
  - Threads likely to advance:
    - `THR-04` to `published`
    - confirm `THR-02` remains stable after the active runtime hardening lands
  - Review-surface areas likely to shift after landing:
    - doctor/diagnostics detail level and failure taxonomy
  - Downstream seams most likely to require revalidation:
    - `SEAM-4` and `SEAM-5` as new failure modes are discovered
