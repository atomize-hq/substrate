---
seam_id: SEAM-2
seam_slug: linux-dev-install-world-service-staging
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_seam_brief: ../../seam-2-linux-dev-install-world-service-staging.md
  source_scope_ref: ../../scope_brief.md
  upstream_closeouts:
    - ../../governance/seam-1-closeout.md
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
  promotion_readiness: ready
open_remediations: []
---
# SEAM-2 - Linux dev-install world-service staging

This seam is closed. Its authoritative exit-gate record lives in `../../governance/seam-2-closeout.md`.

## Seam Brief (Restated)

- **Goal / value**: Make Linux `dev-install-substrate.sh --no-world` leave the enable-later workflow ready by staging `world-service` into the accepted runtime paths while keeping the world disabled and skipping provisioning.
- **Type**: capability (producer seam)
- **Scope**
  - In:
    - Linux `--no-world` staging into `target/bin/world-service` and `target/bin/linux/world-service`
    - selected-profile mapping from `--profile <debug|release>` to `target/<profile>/world-service`
    - `ln -sfn` refresh behavior on repeated dev installs
    - preservation of `world.enabled: false`
    - no provisioning or systemd mutation during the `--no-world` dev-install path
    - regression alignment with installer smoke and the shared accepted path rule
  - Out:
    - runtime preflight or missing-artifact messaging changes
    - building `world-service` during `substrate world enable`
    - widening macOS Lima behavior
    - Windows support changes
    - changing production install semantics beyond staying compatible with the shared accepted path rule
- **Touch surface**:
  - `scripts/substrate/dev-install-substrate.sh`
  - `target/bin/world-service`
  - `target/bin/linux/world-service`
  - `tests/installers/install_smoke.sh`
  - `scripts/substrate/install-substrate.sh` as a regression-only reference surface
- **Verification**:
  - Consumes closeout-backed `THR-01` from `SEAM-1` and publishes `C-04` / `THR-03` for `SEAM-3`.
  - Evidence should prove selected-profile mapping, stale-link refresh, disabled-world invariants, and installer-smoke stability without expanding production-installer scope.
- **Basis posture**:
  - Currentness: `current`
  - Upstream closeouts assumed: `../../governance/seam-1-closeout.md`
  - Required threads (inbound): `THR-01` (published and revalidated)
  - Stale triggers:
    - accepted staged path set or sufficiency rule changes
    - selected-profile mapping changes
    - `ln -sfn` refresh semantics change
    - `scripts/substrate/install-substrate.sh` moves from reference-only posture into an actual touched surface
    - overlapping helper-discovery work lands first on dev-install surfaces
- **Threading constraints**
  - Upstream blockers:
    - none; `SEAM-1` published `THR-01` and `SEAM-2` revalidated that handoff against `governance/seam-1-closeout.md`
  - Downstream blocked seams:
    - `SEAM-3`
  - Contracts produced:
    - `C-04`
  - Contracts consumed:
    - `C-01`
    - `C-03`

## Review bundle

- `review.md` is the authoritative artifact for `gates.pre_exec.review`.

## Seam-exit gate plan

- **Planned location**: `S3` (`slice-3-seam-exit-gate.md`)
- **Why this seam needs an explicit exit gate**: `C-04` / `THR-03` is the final producer handoff `SEAM-3` needs before checkpoint proof can bind to landed staging truth.
- **Expected contracts to publish**:
  - `C-04`
- **Expected threads to publish / advance**:
  - `THR-03`: `defined` -> `published`
- **Likely downstream stale triggers**:
  - any change to selected-profile mapping, staged-link paths, or `ln -sfn` refresh semantics
  - any change to the meaning of `--no-world` or the disabled-world invariant
  - any widening of production-installer scope beyond regression-only reference posture
- **Expected closeout evidence**:
  - published `C-04` rule statements and staged-link evidence in `governance/seam-2-closeout.md`
  - installer-smoke disposition and disabled-world evidence
  - thread-state update record for `THR-03`

## Slice index

- `S1` -> `slice-1-c-04-contract-and-installer-scope.md`
- `S2` -> `slice-2-linux-staging-and-refresh-behavior.md`
- `S3` -> `slice-3-seam-exit-gate.md`

## Governance pointers

- Pack remediation log: `../../governance/remediation-log.md`
- Seam closeout: `../../governance/seam-2-closeout.md`
