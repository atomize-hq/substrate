---
seam_id: SEAM-02
status: proposed
closeout_version: v0
seam_exit_gate:
  source_ref:
  status: pending
  promotion_readiness: blocked
basis:
  currentness: provisional
  upstream_closeouts:
    - SEAM-01
  required_threads:
    - THR-01
  stale_triggers:
    - SEAM-01 contracts change
    - Exit-code taxonomy shared standard changes
    - Precedence chain order changes
gates:
  post_exec:
    landing: pending
    closeout: pending
open_remediations: []
---

# Closeout - SEAM-02 Override Precedence and Fallback

## Seam-exit gate record

**Status**: Not yet realized. This seam is `proposed`.

This seam is `next` in the execution horizon. It requires SEAM-01 to reach `closed` before it can become `exec-ready`.

When closeout occurs, this section will capture:

- **Source artifact**: Implementation that passed landing gate
- **Landed evidence**: Commit SHAs, test results, exit-code behavior samples
- **Contracts published or changed**: C-04, C-05
- **Threads published / advanced**: THR-03 (defined→published), THR-04 (defined)
- **Review-surface delta**: Planned vs landed warning template
- **Planned-vs-landed delta**: Any deviations from extraction brief
- **Downstream stale triggers raised**: For any contract changes during execution
- **Remediation disposition**: Any remediations carried forward or accepted as risk
- **Promotion blockers**: List of issues blocking promotion to next seam

## Post-exec gate disposition

- **Landing gate**: pending
- **Closeout gate**: pending
- **Unresolved remediations**: None
- **Carried-forward remediations**: None

## Pre-exec checklist (for seam-local planning phase)

Before this seam becomes `exec-ready`, the following must be satisfied:

- [ ] `threaded-seams/seam-02-override-precedence/review.md` completed
- [ ] All pre-exec gates passed:
  - [ ] `gates.pre_exec.review: passed`
  - [ ] `gates.pre_exec.contract: passed`
  - [ ] `gates.pre_exec.revalidation: passed`
- [ ] `basis.currentness: current`
- [ ] No blocking pre-exec remediations open
- [ ] SEAM-01 `closed` and `seam_exit_gate.promotion_readiness: ready`
- [ ] THR-01 `published` or `revalidated`

## Post-exec checklist (after execution completes)

Before this seam becomes `closed`, the following must be satisfied:

- [ ] Landing evidence documented
- [ ] Thread states advanced:
  - [ ] THR-01: revalidated (consumed)
  - [ ] THR-03: published
  - [ ] THR-04: defined
- [ ] Contracts published:
  - [ ] C-04: exit-code taxonomy
  - [ ] C-05: warning template
- [ ] Downstream stale triggers recorded (if any)
- [ ] Seam-exit gate realized
- [ ] `seam_exit_gate.status: passed`
- [ ] `seam_exit_gate.promotion_readiness` assessed

## Promotion readiness

Current: blocked (upstream seam not yet closed)

Blockers:
- SEAM-01 must be `closed`
- THR-01 must be `published`

Promote when:
- SEAM-01 `seam_exit_gate.status: passed`
- All required threads are `published` or `revalidated`
- Contracts are stable
- `seam_exit_gate.status: passed`
