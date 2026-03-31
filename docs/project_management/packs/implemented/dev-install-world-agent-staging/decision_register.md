# dev-install-world-agent-staging — decision register

This file records the binary decisions that lock the planning pack contract.

## DR-0001 — Linux missing-artifact preflight locus
- Decision statement:
  - choose the implementation locus for the operator-visible missing-`world-agent` preflight in the standard version-dir flow
- Option A:
  - implement the preflight in the Rust `substrate world enable` runner before helper launch
- Option B:
  - implement the preflight in `scripts/substrate/world-enable.sh` or in `scripts/substrate/install-substrate.sh`
- Selected option:
  - Option A
- Why:
  - the Rust runner owns the user-facing command boundary and can fail before helper output is hidden in a log file. This keeps the remediation singular and visible in dry-run and non-dry-run.
- Tradeoff:
  - the runner must define the accepted staged path rule directly.
- Impacted surfaces:
  - `contract.md`
  - `slices/DIWAS0/DIWAS0-spec.md`
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/tests/world_enable.rs`

## DR-0002 — Meaning of `dev-install-substrate.sh --no-world`
- Decision statement:
  - choose whether `--no-world` skips only provisioning or skips all world-related build and staging work
- Option A:
  - `--no-world` skips provisioning only and still stages `world-agent`
- Option B:
  - `--no-world` skips provisioning and skips all world-related build outputs
- Selected option:
  - Option A
- Why:
  - the workflow goal is “enable later,” not “build later.” Staging at dev-install time keeps `substrate world enable` provisioning-focused and removes the manual artifact gap.
- Tradeoff:
  - Linux dev-install keeps the `world-agent` build cost even when the world remains disabled after install.
- Impacted surfaces:
  - `contract.md`
  - `slices/DIWAS1/DIWAS1-spec.md`
  - `scripts/substrate/dev-install-substrate.sh`
  - `tests/installers/install_smoke.sh`

## DR-0003 — Profile mapping for staged `world-agent`
- Decision statement:
  - choose whether staging always targets `release` or mirrors the selected dev-install profile
- Option A:
  - stage `world-agent` from the selected dev-install profile
- Option B:
  - stage `world-agent` from `release` regardless of the dev-install profile
- Selected option:
  - Option A
- Why:
  - the selected-profile rule matches the current dev-install bridge model for other binaries and keeps debug and release dev installs deterministic.
- Tradeoff:
  - the helper log label and the staged binary source can differ when an operator passes `substrate world enable --profile release` after a debug dev install.
- Impacted surfaces:
  - `contract.md`
  - `slices/DIWAS1/DIWAS1-spec.md`
  - `scripts/substrate/dev-install-substrate.sh`

## DR-0004 — Idempotency and overwrite rule for staged bridges
- Decision statement:
  - choose whether repeated dev installs refresh staged `world-agent` links or keep any existing bridge until it is absent
- Option A:
  - refresh both staged links with `ln -sfn` on every dev install
- Option B:
  - keep any existing staged link and update only when the link is absent
- Selected option:
  - Option A
- Why:
  - repeated debug and release dev installs need a deterministic bridge target. Refreshing the bridge removes stale-profile drift and keeps the accepted path set aligned with the latest build.
- Tradeoff:
  - the current staged bridge target changes on every dev install, so operators need to treat the latest dev-install run as the source of truth.
- Impacted surfaces:
  - `contract.md`
  - `slices/DIWAS1/DIWAS1-spec.md`
  - `scripts/substrate/dev-install-substrate.sh`
  - `smoke/linux-smoke.sh`

## DR-0005 — Accepted staged path sufficiency rule
- Decision statement:
  - choose whether `substrate world enable` requires BOTH accepted staged paths to exist, or accepts either path as sufficient
- Option A:
  - accept either path as sufficient; search order remains `bin/world-agent`, then `bin/linux/world-agent`
- Option B:
  - require BOTH `bin/world-agent` and `bin/linux/world-agent` to exist before provisioning
- Selected option:
  - Option A
- Why:
  - the host runner can proceed with a single deterministic staged artifact while still supporting the Linux-specific fallback path when present.
- Tradeoff:
  - a partially staged layout can still pass preflight; downstream provisioning/verification must remain the source of truth for full enable success.
- Impacted surfaces:
  - `contract.md`
  - `slices/DIWAS0/DIWAS0-spec.md`
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/tests/world_enable.rs`
  - `smoke/linux-smoke.sh`
