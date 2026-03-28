---
seam_id: SEAM-2
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/slice-3-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts:
    - seam-1-closeout.md
    - seam-3-closeout.md
  required_threads:
    - THR-02
    - THR-04
  stale_triggers:
    - "Any change to runtime failure taxonomy or operator-facing error text around missing WORLD_NETFILTER_ENABLE, nft install, or cgroup attach failures requires SEAM-4 and SEAM-5 revalidation."
    - "Any new process-spawn path or attach strategy change under isolate_network requires SEAM-4 and SEAM-5 revalidation."
    - "Any change to nftables ruleset shape, cgroup/netns scoping, or deny-all DNS semantics requires SEAM-4 and SEAM-5 revalidation."
    - "Any installer or service-env change that alters WORLD_NETFILTER_ENABLE propagation requires SEAM-4 revalidation and SEAM-5 smoke reruns."
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-2 `crates/world` enforcement is real, fail-closed, and unavoidable

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `crates/world/src/session.rs` now fails `WorldSession::setup()` when requested isolation cannot install netfilter, instead of warning and continuing, and `setup_network_filter()` resolves domains plus installs rules before publishing an active filter.
  - `crates/world/src/netfilter.rs` now treats missing `WORLD_NETFILTER_ENABLE` as a hard error, fails closed on unresolved/no-address allowlist entries, and keeps deny-all free of any implicit DNS allow rule.
  - `crates/world/src/session.rs` rejects `SUBSTRATE_WORLD_EXEC_FORCE_DIRECT` when `isolate_network=true`, requires helper execution for isolated runs, and converts helper attach wrapper failures into pre-command execution errors rather than plain-exec fallback.
  - `crates/world/src/exec.rs` now injects required `cgroup.procs` targets for isolated helper paths and both helper scripts call `attach_to_cgroup_or_fail` before the inner command starts.
  - Verification landed in `crates/world/src/netfilter.rs`, `crates/world/src/session.rs`, and `crates/world/src/exec.rs`; `cargo test -p world --lib -- --nocapture` passed with `64 passed; 0 failed; 1 ignored`, where the single ignored test is the privileged nftables install test.
- **Contracts published or changed**:
  - none; `SEAM-2` remains the runtime realization of upstream `C-02` / `C-03` rather than a new contract owner.
- **Threads published / advanced**:
  - `THR-04` is now published: isolated execution fails with concrete diagnostics when `WORLD_NETFILTER_ENABLE` is missing, nft rule installation fails, allowed domains cannot resolve, or helper-path cgroup attach cannot be guaranteed.
  - `THR-02` remains revalidated: the routed `isolate_network=true` request now corresponds to actual enforce-or-fail runtime behavior.
- **Review-surface delta**:
  - `crates/world` now has one explicit failure taxonomy for requested isolation across setup, rule installation, and process attach.
  - deny-all now means deny-all, including DNS.
  - isolated execution paths now converge on attach-or-fail helpers, with direct exec explicitly rejected instead of silently bypassing cgroup scope.
- **Planned-vs-landed delta**:
  - The landed implementation tightened the direct-exec gap by rejecting `SUBSTRATE_WORLD_EXEC_FORCE_DIRECT` under isolation instead of introducing a new attach-preserving direct path; this is stricter than the pre-exec hotspot inventory but aligned with seam intent.
  - Privileged install/apply verification and operator-facing doctor surfacing remain intentionally downstream in `SEAM-5` and `SEAM-4`; no seam-local delivery delta remains.
- **Downstream stale triggers raised**:
  - Any change to runtime failure taxonomy or operator-facing error text around missing `WORLD_NETFILTER_ENABLE`, nft install, or cgroup attach failures requires `SEAM-4` and `SEAM-5` revalidation.
  - Any new process-spawn path or attach strategy change under `isolate_network` requires `SEAM-4` and `SEAM-5` revalidation.
  - Any change to nftables ruleset shape, cgroup/netns scoping, or deny-all DNS semantics requires `SEAM-4` and `SEAM-5` revalidation.
  - Any installer or service-env change that alters `WORLD_NETFILTER_ENABLE` propagation requires `SEAM-4` revalidation and `SEAM-5` smoke reruns.
- **Remediation disposition**:
  - `REM-002` remains resolved; the spawned-process inventory is now backed by landed helper-path and direct-exec regression coverage instead of a planning-only boundary.
  - No new seam-local remediations were opened during closeout.
- **Promotion blockers**:
  - none at the `SEAM-2` boundary; downstream observability (`SEAM-4`) and conformance (`SEAM-5`) still own their own publication work.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - none
- **Carried-forward remediations**:
  - none; remaining observability and conformance work is owned by downstream seams, not an unresolved `SEAM-2` gap.
