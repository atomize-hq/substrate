---
seam_id: SEAM-3
status: landed
closeout_version: v0
seam_exit_gate:
  source_ref: threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
basis:
  currentness: current
  upstream_closeouts: []
  required_threads:
    - THR-03
  stale_triggers:
    - "Any change to world.net.filter precedence, explain output, or patch semantics requires SEAM-1 and SEAM-5 revalidation."
    - "Any change to SUBSTRATE_OVERRIDE_WORLD_NET_FILTER workspace applicability requires SEAM-5 and operator-doc revalidation."
    - "Any change to SUBSTRATE_WORLD_NET_FILTER naming or meaning requires SEAM-5 and downstream telemetry/debug-surface revalidation."
gates:
  post_exec:
    landing: passed
    closeout: passed
open_remediations: []
---

# Closeout - SEAM-3 Config opt-in `world.net.filter` + CLI patching + env parity

## Seam-exit gate record

- **Source artifact**: `threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-4-seam-exit-gate.md`
- **Landed evidence**:
  - `crates/shell/src/execution/config_model.rs` resolves `world.net.filter`, includes explain output, and parses `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER` with the no-workspace override rule.
  - `crates/shell/src/execution/env_scripts.rs` exports `SUBSTRATE_WORLD_NET_FILTER=1|0` in both runtime env propagation and `env.sh`.
  - `crates/shell/tests/config_set.rs` covers set/reset behavior for `world.net.filter`.
  - `crates/shell/tests/config_show.rs` covers precedence, explain provenance, no-workspace override use, workspace override suppression, and invalid env rejection.
  - `crates/shell/tests/ev0_override_split.rs` covers runtime parity export and confirms override applicability only when no workspace exists.
  - `docs/reference/config/world.md` and `docs/CONFIGURATION.md` publish the three-way gate semantics and operator examples.
- **Contracts published or changed**:
  - `C-04`: `world.net.filter` is the authoritative host-side opt-in gate and defaults to `false`.
  - `C-05`: `SUBSTRATE_OVERRIDE_WORLD_NET_FILTER` is input-only and only applies when no workspace config exists.
  - `C-06`: `SUBSTRATE_WORLD_NET_FILTER` is derived parity output only and reports the resolved effective host gate.
- **Threads published / advanced**: `THR-03` is now published as the landed host-boundary handoff consumed by `SEAM-1`, `SEAM-4`, and `SEAM-5`.
- **Review-surface delta**: operators now have one published story across config precedence, parity export, `WORLD_NETFILTER_ENABLE`, and policy `net_allowed`; downstream seams must consume that story without redefinition.
- **Planned-vs-landed delta**:
  - No remaining seam-local delta exists for `C-04` / `C-05` / `C-06`; the remaining work is intentionally downstream.
  - `SEAM-1` still owns the runtime adoption of this published gate when constructing `WorldSpec.isolate_network` and `allowed_domains`.
  - `SEAM-2` still owns runtime enforcement and fail-closed behavior once isolation is requested.
- **Downstream stale triggers raised**:
  - Any change to `world.net.filter` precedence, explain output, or override parsing requires `SEAM-1` and `SEAM-5` revalidation.
  - Any change to exported parity env naming or value semantics requires `SEAM-5` revalidation.
  - Any change to the published three-way gate rule requires `SEAM-1` review refresh before promotion.
- **Remediation disposition**:
  - `REM-003` is resolved by the landed user-facing docs and examples.
  - `REM-004` is resolved by citing the landed code/test/doc evidence for `C-04` / `THR-03`; downstream `SEAM-1` revalidation is follow-on execution, not an unresolved `SEAM-3` owner gap.
- **Promotion blockers**:
  - none at the `SEAM-3` boundary; downstream seams still have their own execution work.
- **Promotion readiness**: ready

## Post-exec gate disposition

- **Landing gate**: passed
- **Closeout gate**: passed
- **Unresolved remediations**:
  - none
- **Carried-forward remediations**:
  - `SEAM-1` must continue with its own runtime adoption of the published host gate, but that work is not a carried `SEAM-3` remediation.
