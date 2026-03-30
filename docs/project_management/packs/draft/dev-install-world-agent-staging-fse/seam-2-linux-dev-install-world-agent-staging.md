---
seam_id: SEAM-2
seam_slug: linux-dev-install-world-agent-staging
type: capability
status: proposed
execution_horizon: next
plan_version: v1
basis:
  currentness: provisional
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads:
    - THR-01
  stale_triggers:
    - accepted staged path set or sufficiency rule changes
    - selected-profile mapping changes
    - ln -sfn refresh semantics change
    - scripts/substrate/install-substrate.sh moves from reference-only posture into an actual touched surface
    - overlapping helper-discovery work lands first on dev-install surfaces
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
seam_exit_gate:
  required: true
  planned_location: reserved_final_slice
  status: pending
open_remediations: []
---

# SEAM-2 - Linux dev-install world-agent staging

- **Goal / value**: Make Linux `dev-install-substrate.sh --no-world` leave the enable-later workflow ready by staging `world-agent` into the accepted runtime paths while keeping the world disabled and skipping provisioning.
- **Scope**
  - In:
    - Linux `--no-world` staging into `target/bin/world-agent` and `target/bin/linux/world-agent`
    - selected-profile mapping from `--profile <debug|release>` to `target/<profile>/world-agent`
    - `ln -sfn` refresh behavior on repeated dev installs
    - preservation of `world.enabled: false`
    - no provisioning or systemd mutation during the `--no-world` dev-install path
    - regression alignment with installer smoke and the shared accepted path rule
  - Out:
    - runtime preflight or missing-artifact messaging changes
    - building `world-agent` during `substrate world enable`
    - widening macOS Lima behavior
    - Windows support changes
    - changing production install semantics beyond staying compatible with the shared accepted path rule
- **Primary interfaces**
  - Inputs:
    - `scripts/substrate/dev-install-substrate.sh [--prefix <path>] [--profile <debug|release>] [--no-world]`
    - `target/debug/world-agent` and `target/release/world-agent`
    - the accepted staged path rule and state-ordering guarantees from `SEAM-1`
    - `tests/installers/install_smoke.sh`
  - Outputs:
    - executable staged links at both accepted path locations
    - deterministic selected-profile mapping and refresh behavior
    - retained `world.enabled: false` state and skipped provisioning during `--no-world`
    - a publishable staging contract for the checkpoint evidence to consume
- **Key invariants / rules**:
  - Linux `--no-world` means “skip provisioning, not staging”.
  - The selected dev-install profile controls the staged bridge target.
  - Re-running the script refreshes both staged links with `ln -sfn`.
  - `substrate world enable --profile` does not change the staged bridge target.
  - The accepted runtime search order remains `bin/world-agent`, then `bin/linux/world-agent`.
  - Production installer behavior remains reference-only outside the accepted path rule unless seam-local review explicitly narrows or widens that touch surface.
- **Dependencies**
  - Direct blockers:
    - `THR-01` must publish the landed accepted path rule, override carve-out, and state-ordering guarantees from `SEAM-1`
  - Transitive blockers:
    - overlap on `scripts/substrate/dev-install-substrate.sh` with helper-discovery work
    - any scope change that turns production installer semantics into an owned behavior delta
    - shared exit taxonomy and no-new-config / no-new-policy constraints
  - Direct consumers:
    - `SEAM-3`
  - Derived consumers:
    - future dev-install or packaging work that depends on the staged bridge layout or “enable later” meaning of `--no-world`
- **Touch surface**:
  - `scripts/substrate/dev-install-substrate.sh`
  - `target/bin/world-agent`
  - `target/bin/linux/world-agent`
  - `tests/installers/install_smoke.sh`
  - `scripts/substrate/install-substrate.sh` as a possible regression-sensitive reference surface pending seam-local scope confirmation
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Expected proof points:
    - debug-profile dev install stages both accepted links to `target/debug/world-agent`
    - release-profile dev install stages both accepted links to `target/release/world-agent`
    - repeated debug→release or release→debug runs refresh stale link targets deterministically
    - `world.enabled` remains `false` and no provisioning or systemd mutation occurs during `--no-world`
    - installer smoke remains green and does not show unintended production-installer drift
    - a staged artifact produced here satisfies the runtime preflight contract from `SEAM-1`
- **Risks / unknowns**:
  - Risk: the source pack leaves ambiguity about whether `scripts/substrate/install-substrate.sh` is a true touched surface or only a regression boundary.
  - De-risk plan: resolve that scope question in the first seam-local review and keep the change set narrow if installer behavior remains reference-only.
  - Risk: selected-profile staging can look inconsistent with the helper’s default release-oriented log labeling.
  - De-risk plan: keep profile mapping explicit in seam-local review, smoke output, and closeout evidence so operators understand that staging source and log label are different contracts.
- **Rollout / safety**:
  - Keep the behavior delta Linux-only.
  - Do not broaden production install semantics accidentally.
  - Preserve `world.enabled: false` until runtime enable verifies success.
  - Keep the staging change orthogonal to helper override behavior and future provisioning flags.
- **Downstream decomposition context**:
  - Why this seam is `next`: it is tightly bounded and already well-specified by the source pack, but it should promote only after `SEAM-1` publishes the accepted path and ordering contracts as landed truth.
  - Which threads matter most: `THR-01` and `THR-03`.
  - What the first seam-local review should focus on: selected-profile mapping, stale-link replacement, disabled-world invariants, regression boundaries for installer smoke, and the exact scope of production-installer references.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-04`.
  - Threads likely to advance: `THR-03`, plus consumption-side revalidation against `THR-01`.
  - Review-surface areas likely to shift after landing: staged-link layout, profile mapping notes, and installer-regression evidence summaries.
  - Downstream seams most likely to require revalidation: `SEAM-3`.
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
