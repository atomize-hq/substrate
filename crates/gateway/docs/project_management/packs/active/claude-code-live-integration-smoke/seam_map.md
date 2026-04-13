# Seam Map - Claude Code Live Integration Smoke

This seam map extracts the remaining operator-facing live integration work from the already-landed gateway and Azure transport basis. It starts from closeout-backed truth instead of reopening normalization, public-surface, policy, or transport design.

Constraint posture:

- `C-03`, `C-04`, `C-05`, `C-07`, and `C-08` remain upstream basis and are not re-owned here
- the remaining gap is the operator path above the landed gateway contract: bootstrap, real Claude Code smoke, and supportable troubleshooting ownership
- live Azure verification remains first-class, but this pack upgrades it from gateway-backed smoke to the real Claude Code operator path

## Horizon summary

- **Active seam**: `SEAM-3`
- **Next seam**: none remaining in this pack
- **Future seams**: none remaining in this pack

The default v2.3 horizon policy is explicit here:

- only `SEAM-1` is eligible for authoritative downstream deep planning by default
- `SEAM-2` may later receive seam-local review and slices, but only provisional deeper planning until the active seam publishes its bootstrap truth
- `SEAM-3` remains a seam brief only and should not receive deep planning until the earlier seams land and publish their operator evidence

## Seam roster

| Seam | Horizon / state | Type | Why this is a seam | Likely value | Touch surface | Verification path |
| --- | --- | --- | --- | --- | --- | --- |
| `SEAM-1` `claude-code-operator-bootstrap` | `future` / `landed` | `integration` | it froze the reproducible setup path from Azure prerequisites through gateway config/startup and Claude Code attachment instead of scattering that knowledge across README fragments and code | a real operator can reach a first valid Claude Code through gateway session without reverse-engineering env vars, config files, or startup order | `gateway/README.md`, `gateway/config/*.toml`, `gateway/src/cli/mod.rs`, startup/config validation surfaces, operator scripts/checklists | closeout-backed `C-09` truth now provides the bootstrap basis for later work |
| `SEAM-2` `live-session-smoke-verification` | `future` / `landed` | `conformance` | it turned the bootstrap path into real Claude Code proof for normal, think, and tool-loop continuation flows instead of relying on gateway-only `/v1/messages` checks | operators can prove the live path works through the client they actually use and can capture the right routing/tracing evidence when it does not | Claude Code launch/config instructions, statusline/tracing surfaces, smoke procedure docs, redacted transcript or evidence manifest surfaces | closeout-backed `C-10` truth now provides the live-smoke basis for downstream troubleshooting work |
| `SEAM-3` `troubleshooting-and-support-boundary` | `active` / `exec-ready` | `conformance` | it freezes failure ownership and troubleshooting surfaces so later operators can tell whether a problem lives in Claude Code attachment, gateway runtime/config, or Azure transport without re-reading source | supportable operator guidance, reusable checklists, and a bounded escalation path that preserves the public boundary | troubleshooting docs, ownership matrix, evidence review checklist, redaction rules, any bounded diagnostic scripts or templates | a reviewer can classify likely failures by owner and required evidence without conflating public capability language with provider or planner/executor internals |

## Ordering rationale

1. `SEAM-1` landed first because the remaining practical blocker was the canonical operator bootstrap path; without that, later live smoke work would have kept rediscovering setup details ad hoc.
2. `SEAM-2` then landed because real Claude Code smoke only became meaningful once bootstrap truth was concrete and reproducible.
3. `SEAM-3` is now active because troubleshooting ownership can finally freeze on top of published bootstrap and live-smoke truth rather than provisional assumptions.

## Non-seams and pruned candidates

- A new Azure transport seam was rejected because `azure-foundry-provider-transport` already landed `C-07` and `C-08`; this pack consumes those truths.
- A new public `/v1/messages` seam was rejected because `C-03` already fixes the public client contract and tool-loop semantics below this operator-facing work.
- A new planner/executor seam was rejected because `C-04` already fixes the internal routing posture; this pack only needs operator-visible verification of that behavior.
- A production deployment or infra seam was rejected because the user explicitly scoped this work to a realistic operator smoke path, not a full deployment program.
- A generic docs cleanup seam was rejected because the remaining work is not broad documentation hygiene; it is a bounded operator path with clear live-verification and support surfaces.
