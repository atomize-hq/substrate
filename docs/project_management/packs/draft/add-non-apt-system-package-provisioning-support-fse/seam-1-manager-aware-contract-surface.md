---
seam_id: SEAM-1
seam_slug: manager-aware-contract-surface
type: integration
status: closed
execution_horizon: future
plan_version: v2
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
  - shared CLI/runtime wording changes in ADR-0033 or overlapping pack contracts
  - exit-code or request-profile posture changes before seam-local review
  - v1 pacman scope or authority-handoff targets change and make the extracted contract
    basis stale
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

# SEAM-1 - Manager-aware contract surface

- **Goal / value**:
  - Freeze one authoritative manager-aware contract and accepted decision set so downstream seams can plan against a single truth instead of inheriting APT-only drift or overlapping pack ambiguity.
- **Scope**
  - In:
    - Shared manager-aware semantics for `substrate world enable --provision-deps`
    - Shared runtime invariant that `substrate world deps current sync|install` must not mutate system package managers
    - Authority handoff from the older APT-only pack and the upstream bundles contract
    - Exit-code posture, request-profile posture, platform/backend guarantees, and mixed-manager failure rule
    - Accepted decisions DR-0001 through DR-0004 for schema posture, probe precedence, pacman execution shape, and v1 pacman runnable scope
  - Out:
    - Implementing the in-world probe and support gate
    - Implementing `install.method=pacman` schema validation or inventory views
    - Implementing provisioning-time requirement derivation or pacman dispatch
    - Implementing runtime read-only probes or remediation rendering
    - Smoke/manual evidence and shared-doc reconciliation landing work
- **Primary interfaces**
  - Inputs:
    - `ADR-0033-routing-weasel.md`
    - `ADR-0030-provisioning-otter.md`
    - `EXIT_CODE_TAXONOMY.md`
    - upstream bundles contract and existing APT pack contract surfaces
  - Outputs:
    - `C-01` shared manager-aware operator contract
    - explicit authority/defer map consumed by every downstream seam
    - accepted decision basis for `C-02`, `C-03`, `C-04`, and `C-05`
- **Key invariants / rules**:
  - `substrate world enable --provision-deps` remains the only operator-facing provisioning entrypoint for world-deps system-package mutation
  - runtime `substrate world deps current sync|install` must not invoke mutating `apt`, `dpkg`, or `pacman`
  - no new config key, environment variable, structured log field, trace field, protocol field, or agent API request field is introduced
  - manager selection is derived in-world and must not route from host PATH, host installer detection, or host package-manager state
  - Linux host-native and Windows provisioning stay unsupported and fail-closed in v1
  - `install.method=pacman` remains a provisioning-only, non-runnable system-package method in v1
- **Dependencies**
  - Direct blockers:
    - none; this seam exists because the source pack explicitly treated contract and decision settlement as the prerequisite workstream
  - Transitive blockers:
    - overlapping ADR and planning-pack docs can stale the authority story if they move before reconciliation lands
    - shared code surfaces in `world_enable` and `world-agent` can expose hidden assumptions that still contradict the contract
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - Derived consumers:
    - operators and support engineers
    - overlapping APT and bundles contract packs
    - future Linux guest/system-package work that wants to reuse the same CLI surface
- **Touch surface**:
  - Primary planning surfaces:
    - `contract.md`
    - `decision_register.md`
  - Key lineage surfaces:
    - `pre-planning/spec_manifest.md`
    - `pre-planning/impact_map.md`
  - Likely downstream reconciliation surfaces once seam-local planning begins:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md`
    - `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md`
- **Verification**:
  - Because this seam **produces** the owned shared contract, verification should prove the contract is concrete enough for seam-local planning and implementation rather than requiring the final accepted external doc updates to exist already.
  - The first seam-local review should try to falsify:
    - whether any second truth still exists for the shared `--provision-deps` entrypoint
    - whether the mixed-manager rule, request-profile posture, or runtime no-mutation posture are still ambiguous
    - whether any downstream seam could still interpret pacman support as runnable or host-routable
  - A passing pre-exec posture should leave probe, schema, provisioning, runtime, and validation seams able to plan against one stable operator contract.
- **Risks / unknowns**:
  - Risk:
    - overlapping ADR and pack docs can continue to present a second truth until the conformance seam lands reconciliation updates.
  - De-risk plan:
    - keep the shared-contract reconciliation risk visible as `REM-001` under `SEAM-6`, while making `C-01` explicit enough now that downstream planning is possible.
  - Risk:
    - accepted decision wording can drift if adjacent packs update the same shared files or examples.
  - De-risk plan:
    - force downstream seams to consume `C-01` through `THR-01` and revalidate before decomposition.
- **Rollout / safety**:
  - This seam should land as an additive contract-definition seam only. It must not broaden scope into host mutation, new protocol/config surfaces, or runnable pacman behavior.
  - Fail-closed behavior is part of the contract surface and should be treated as load-bearing, not as implementation detail.
- **Downstream decomposition context**:
  - This seam is closed and out of the forward planning window; downstream seams now consume published `C-01` via `THR-01` and revalidate on its recorded stale triggers.
  - The most important thread is `THR-01`.
  - The first seam-local review should focus on authority handoff, exit-code meanings, mixed-manager posture, request-profile boundaries, and pacman v1 scope.
  - Source-plan lineage: `NASP-PWS-contract`, `contract.md`, and `decision_register.md`.
- **Expected seam-exit concerns**:
  - Contracts likely to publish:
    - `C-01`
  - Threads likely to advance:
    - `THR-01` from `defined` to `published`
  - Review-surface areas likely to shift after landing:
    - the provisioning-vs-runtime workflow diagram
    - the contract authority handoff across ADR-0033, the APT pack, and the bundles contract
  - Downstream seams most likely to require revalidation:
    - `SEAM-2`
    - `SEAM-3`
    - `SEAM-4`
    - `SEAM-5`
    - `SEAM-6`
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
