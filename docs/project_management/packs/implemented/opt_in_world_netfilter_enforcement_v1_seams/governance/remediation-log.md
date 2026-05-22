# Remediation Log - Opt-in World Netfilter Enforcement

## Open remediations

No open remediations.

## Resolved remediations

- Move resolved items here using the same schema, set `status: resolved`, and populate `resolution_evidence`.

```yaml
remediation_id: REM-005
origin_phase: post_exec
source_gate: closeout
related_seam: SEAM-2
related_slice: S3
related_thread: THR-04
related_contract: C-02
related_artifact: docs/project_management/packs/draft/opt_in_world_netfilter_enforcement_v1_seams/governance/seam-2-closeout.md
severity: blocking
status: resolved
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: seam_exit_gate.status
    value: passed
  - seam: SEAM-2
    field: promotion_readiness
    value: ready
summary: Record privileged Linux verification evidence for fail-closed nftables enforcement before publishing THR-04
required_fix: Run the privileged Linux verification surface for the landed runtime hardening (ignored `world` netfilter coverage or equivalent isolated netns/cgroup validation) and capture the resulting artifact references in `governance/seam-2-closeout.md` so THR-04 can be published without ambiguity
resolution_evidence:
  - "SEAM-2 closeout now records `seam_exit_gate.status: passed`, `promotion_readiness: ready`, `gates.post_exec.landing: passed`, and `THR-04` published from landed runtime evidence: governance/seam-2-closeout.md"
  - "SEAM-4 closeout consumes the published `THR-04` handoff without reopening a SEAM-2 blocker, confirming the runtime guard/failure taxonomy is now downstream input rather than an unresolved SEAM-2 remediation: governance/seam-4-closeout.md"
```

```yaml
remediation_id: REM-001
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: S1
related_thread: THR-01
related_contract: C-01
related_artifact: crates/transport-api-types
severity: blocking
status: resolved
owner_seam: SEAM-1
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: Decide and record `net_allowed` hostname normalization rules (casefolding + IDNA posture)
required_fix: Document the canonical normalization posture (ASCII casefolding, trailing-dot handling, IDNA posture) and the verification plan
resolution_evidence:
  - "Contract decision recorded: threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-1-publish-net-allowed-contract.md (S1.T1)"
  - "Seam gate disposition updated: threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/seam.md + review.md"
```

```yaml
remediation_id: REM-003
origin_phase: pre_exec
source_gate: review
related_seam: SEAM-3
related_slice: null
related_thread: THR-03
related_contract: C-04
related_artifact: docs/reference/config/world.md
severity: material
status: resolved
owner_seam: SEAM-3
blocked_targets: []
summary: Clarify operator workflow for enabling netfilter safely
required_fix: Land the active `SEAM-3` owner slices that publish operator-facing semantics and examples, especially `threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/slice-3-operator-docs-and-routing-handoff.md`, then update `docs/reference/config/world.md` and `docs/CONFIGURATION.md` with the three-way gate alignment (`world.net.filter`, `WORLD_NETFILTER_ENABLE`, policy `net_allowed`)
resolution_evidence:
  - "Operator docs landed: docs/reference/config/world.md + docs/CONFIGURATION.md"
  - "SEAM-3 review/seam disposition refreshed: threaded-seams/seam-3-host-config-opt-in-and-parity-env-plumbing/review.md + seam.md"
  - "SEAM-3 closeout records the landed three-way gate publication: governance/seam-3-closeout.md"
```

```yaml
remediation_id: REM-004
origin_phase: pre_exec
source_gate: contract
related_seam: SEAM-1
related_slice: S2
related_thread: THR-03
related_contract: C-04
related_artifact: docs/project_management/packs/draft/opt_in_world_netfilter_enforcement_v1_seams/threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-2-host-snapshot-and-worldspec-plumbing.md
severity: blocking
status: resolved
owner_seam: SEAM-3
blocked_targets:
  - seam: SEAM-1
    field: status
    value: exec-ready
summary: Publish the concrete `C-04` / `THR-03` host-side gating contract for `SEAM-1`; the contract shape is now decided, but implementation and landed closeout evidence are still missing
required_fix: Land the active `SEAM-3` owner slices that publish `C-04` / `THR-03` and their verification surfaces: `slice-1-publish-world-net-filter-config-contract.md`, `slice-2-override-and-parity-env-plumbing.md`, and `slice-3-operator-docs-and-routing-handoff.md`; once those artifacts land, revalidate next `SEAM-1` against the published host gate before attempting `exec-ready`
resolution_evidence:
  - "Owner config/env surfaces were already landed in code/tests: crates/shell/src/execution/config_model.rs + crates/shell/src/execution/env_scripts.rs + related shell tests"
  - "Operator docs and three-way gate semantics landed: docs/reference/config/world.md + docs/CONFIGURATION.md"
  - "SEAM-3 closeout now cites the landed host-gate evidence and publishes THR-03: governance/seam-3-closeout.md"
  - "SEAM-1 basis refresh is downstream follow-on work, not an unresolved SEAM-3 owner gap: threaded-seams/seam-1-snapshot-v3-net-allowlist-plumbing/slice-2-host-snapshot-and-worldspec-plumbing.md + review.md + seam.md"
```

```yaml
remediation_id: REM-002
origin_phase: pre_exec
source_gate: revalidation
related_seam: SEAM-2
related_slice: null
related_thread: THR-02
related_contract: C-02
related_artifact: crates/world
severity: blocking
status: resolved
owner_seam: SEAM-2
blocked_targets:
  - seam: SEAM-2
    field: status
    value: exec-ready
summary: Enumerate and harden all process spawn paths for cgroup attach under isolate_network
required_fix: Inventory all execution paths and ensure each attaches to the world cgroup or fails when isolate_network=true
resolution_evidence:
  - "Spawn-path inventory and mismatch hotspots are now explicit in threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/review.md"
  - "Attach-or-fail execution work is now bounded in threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/slice-1-fail-closed-netfilter-runtime.md and slice-2-cgroup-attach-invariants-across-exec-paths.md"
  - "SEAM-2 seam-local basis and gates were refreshed against the landed SEAM-1 and SEAM-3 closeouts in threaded-seams/seam-2-world-netfilter-fail-closed-and-cgroup-invariants/seam.md"
```
