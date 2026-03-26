---
seam_id: SEAM-2
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/slice-3-seam-exit-gate.md
  status: failed
  promotion_readiness: blocked
basis:
  currentness: current
  upstream_closeouts:
    - governance/seam-1-closeout.md
    - governance/seam-3-closeout.md
  required_threads:
    - THR-02
    - THR-04
  stale_triggers:
    - "Any installer or world-agent service configuration change affecting WORLD_NETFILTER_ENABLE requires SEAM-2, SEAM-4, and SEAM-5 revalidation."
    - "Any change to nftables ruleset shape or DNS handling for deny-all requires SEAM-4 and SEAM-5 revalidation."
    - "Any new world execution path or weaker attach-or-fail behavior under isolate_network requires SEAM-4 and SEAM-5 revalidation."
    - "Any new enforcement failure class or diagnostic wording change requires SEAM-4 review refresh and SEAM-5 coverage refresh."
gates:
  post_exec:
    landing: passed
    closeout: failed
open_remediations:
  - REM-005
---

# Closeout - SEAM-2 `crates/world` enforcement is real, fail-closed, and unavoidable

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/slice-3-seam-exit-gate.md`
- **Landed evidence**:
  - `crates/world/src/session.rs` now fails setup when requested isolation cannot be enforced, rejects `SUBSTRATE_WORLD_EXEC_FORCE_DIRECT` when `isolate_network=true`, and refuses isolated helper fallback before plain command execution.
  - `crates/world/src/netfilter.rs` now requires `WORLD_NETFILTER_ENABLE=1|true|yes`, keeps deny-all free of implicit DNS allow rules, and returns hard errors on unresolved allowlist hosts and nft command failures.
  - `crates/world/src/exec.rs` helper wrappers attach to `cgroup.procs` before `exec` and fail closed when the attach target is missing or unwritable.
  - `cargo test -p world --lib -- --nocapture` passed in the current workspace, covering `session::tests::setup_fails_when_requested_isolation_cannot_install_netfilter`, `session::tests::execute_rejects_forced_direct_exec_when_isolation_is_requested`, `session::tests::isolated_helper_flow_does_not_fall_back_to_plain_exec_when_attach_fails`, `netfilter::tests::test_deny_all_rule_bodies_do_not_allow_dns`, `netfilter::tests::test_install_rules_requires_world_netfilter_enable`, `netfilter::tests::test_install_rules_errors_when_nft_fails`, `exec::tests::project_bind_mount_fails_closed_when_cgroup_attach_target_is_missing`, and `exec::tests::world_deps_bind_mount_fails_closed_when_cgroup_attach_target_is_missing`.
  - A privileged Linux proof surface exists as ignored coverage in `netfilter::tests::test_nftables_rules`, but no successful run artifact is recorded in this pack closeout yet.
- **Contracts published or changed**:
  - none; `SEAM-2` does not publish new contract ownership.
  - `SEAM-2` operationalizes the already-published `C-02` / `C-03` semantics in the runtime.
- **Threads published / advanced**:
  - `THR-02` remains stable: the landed runtime behavior matches the published `isolate_network=true` enforce-or-fail contract.
  - `THR-04` remains `identified`, not `published`, because the seam does not yet have recorded privileged verification evidence for the env-guard and fail-closed enforcement path.
- **Review-surface delta**:
  - The seam-local review hotspots in `threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/review.md` are now closed in landed code for fail-closed setup, deny-all DNS handling, helper attach-or-fail behavior, and direct-exec rejection under isolation.
  - The remaining review gap is governance evidence, not runtime semantics: downstream seams still lack a recorded privileged Linux artifact they can cite when consuming `THR-04`.
- **Planned-vs-landed delta**:
  - The direct-exec bypass was closed by explicitly rejecting forced direct exec when `isolate_network=true` rather than by introducing a new direct-exec attach path.
  - Helper-based execution now fails before command start when cgroup attach cannot be established, which is tighter than the original warn-only review posture.
  - Privileged Linux verification was planned as closeout evidence but has not yet been captured in the pack.
- **Downstream stale triggers raised**:
  - Any installer or world-agent service configuration change affecting `WORLD_NETFILTER_ENABLE` requires `SEAM-2`, `SEAM-4`, and `SEAM-5` revalidation.
  - Any change to nftables ruleset shape or DNS handling for deny-all requires `SEAM-4` and `SEAM-5` revalidation.
  - Any new world execution path or weaker attach-or-fail behavior under `isolate_network` requires `SEAM-4` and `SEAM-5` revalidation.
  - Any new enforcement failure class or diagnostic wording change requires `SEAM-4` review refresh and `SEAM-5` coverage refresh.
- **Remediation disposition**:
  - `REM-002` remains resolved; the spawn-path inventory and attach-or-fail implementation are now reflected in landed code and tests.
  - `REM-005` is open to capture the missing privileged Linux verification artifact required to publish `THR-04`.
- **Promotion blockers**:
  - No recorded successful privileged Linux verification artifact exists yet for the landed nftables install/apply behavior in an isolated netns/cgroup scope.
  - Manual macOS Lima smoke evidence remains downstream work for `SEAM-5` and does not unblock `SEAM-2` promotion by itself.
- **Promotion readiness**: blocked

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: failed
- **Unresolved remediations**:
  - `REM-005`
- **Carried-forward remediations**:
  - `REM-005` must land privileged Linux verification evidence in this closeout before `THR-04` can be published and pack promotion can advance to `SEAM-4`.
