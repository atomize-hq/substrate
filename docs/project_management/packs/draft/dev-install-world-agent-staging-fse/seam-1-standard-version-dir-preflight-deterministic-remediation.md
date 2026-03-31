---
seam_id: SEAM-1
seam_slug: standard-version-dir-preflight-deterministic-remediation
type: capability
status: closed
execution_horizon: future
plan_version: v1
basis:
  currentness: current
  source_scope_ref: scope_brief.md
  source_scope_version: v1
  upstream_closeouts: []
  required_threads: []
  stale_triggers:
    - accepted staged path set or sufficiency rule changes
    - standard version-dir derivation changes
    - helper-output suppression or visible remediation path changes
    - world.enabled ordering or --home precedence changes
    - overlapping helper-discovery or provisioning work lands first on shared world-enable surfaces
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
  source_ref: threaded-seams/seam-1-standard-version-dir-preflight-deterministic-remediation/slice-4-seam-exit-gate.md
  status: passed
  promotion_readiness: ready
open_remediations: []
---

# SEAM-1 - Standard version-dir preflight + deterministic remediation

- **Goal / value**: Make `substrate world enable` behave deterministically in the standard version-dir flow by either finding an accepted staged `world-agent` artifact or failing early with one operator-visible remediation before helper launch, privileged work, or state mutation.
- **Scope**
  - In:
    - standard version-dir preflight when `SUBSTRATE_WORLD_ENABLE_SCRIPT` is unset
    - accepted staged path derivation, search order, and “either path is sufficient” rule
    - deterministic exit `3` missing-artifact failure with one remediation block
    - shared dry-run preflight and no-write behavior
    - `world.enabled` ordering and `--home` precedence on the success path
    - explicit override carve-out preservation
  - Out:
    - Linux dev-install staging changes
    - building `world-agent` inside `substrate world enable`
    - changing helper-override discovery rules beyond preserving the carve-out
    - widening macOS or Windows runtime support
    - broader helper-discovery durability work such as `cargo clean` resilience
- **Primary interfaces**
  - Inputs:
    - `substrate world enable [--home <path>] [--profile <name>] [--dry-run] [--verbose] [--force] [--timeout <seconds>]`
    - `$SUBSTRATE_HOME`
    - `SUBSTRATE_WORLD_ENABLE_SCRIPT`
    - `<home>/bin/substrate`
    - the accepted staged path set under the standard version dir
    - shared exit-code taxonomy and no-new-config / no-new-policy constraints
  - Outputs:
    - deterministic runtime preflight and early-failure behavior
    - stable operator-facing remediation for the missing-artifact path
    - pinned ordering for dry-run, helper launch, config writes, and `world.enabled`
    - a publishable path / precedence contract that staging and conformance can consume
- **Key invariants / rules**:
  - The accepted staged path search order is fixed: `<version_dir>/bin/world-agent`, then `<version_dir>/bin/linux/world-agent`.
  - Either accepted executable path is sufficient; the command does not require both paths to exist to continue.
  - When neither accepted path exists, the command exits `3` before helper launch, provisioning, health verification, config writes, or manager-env writes.
  - The missing-artifact remediation must name both accepted paths, `scripts/substrate/dev-install-substrate.sh --no-world`, and `cargo build -p world-agent`.
  - `--dry-run` runs the same preflight, exits `0` only when an accepted artifact exists, and writes no config or system state.
  - On the non-dry-run success path, `world.enabled` stays `false` until helper execution and health verification both succeed.
  - `SUBSTRATE_WORLD_ENABLE_SCRIPT` remains the highest-priority helper path and stays outside the standard version-dir preflight guarantee.
- **Dependencies**
  - Direct blockers:
    - no internal upstream seam blocker
  - Transitive blockers:
    - shared exit-code taxonomy
    - existing `--home` / `SUBSTRATE_HOME` precedence and no-`--prefix` policy
    - overlapping helper-discovery or provisioning work on `world enable` surfaces
  - Direct consumers:
    - `SEAM-2`
    - `SEAM-3`
  - Derived consumers:
    - future provisioning or packaging work that must preserve the runtime failure contract and ordering guarantees
- **Touch surface**:
  - `crates/shell/src/builtins/world_enable/runner.rs`
  - `crates/shell/tests/world_enable.rs`
  - `scripts/substrate/world-enable.sh` as the preserved helper boundary and override-adjacent interface
  - `$SUBSTRATE_HOME/config.yaml` and the state-write ordering implied by the success path
- **Verification**:
  - If this seam **consumes** an upstream contract, verification may depend on accepted upstream evidence.
  - If this seam **produces** an owned contract, verification should describe the contract becoming concrete enough for seam-local planning and implementation rather than requiring the final accepted artifact to exist already.
  - Expected proof points:
    - standard version-dir derivation from `<home>/bin/substrate` is explicit and test-covered
    - the accepted staged path set is checked in the fixed order and either path is sufficient
    - missing-artifact dry-run exits `3`, surfaces the required remediation, and writes nothing
    - missing-artifact non-dry-run exits `3` before helper launch or privileged mutation
    - present-artifact dry-run exits `0` and writes no config, helper log, manager-env export, or systemd state
    - non-dry-run success keeps `world.enabled` `false` until helper execution and health verification both succeed
    - `SUBSTRATE_WORLD_ENABLE_SCRIPT` override behavior remains unchanged
- **Risks / unknowns**:
  - Risk: the missing-artifact remediation can become invisible if implementation drifts into the helper boundary and default CLI output keeps suppressing helper output.
  - De-risk plan: keep the preflight at the runner boundary, assert the visible stderr block in tests, and treat any helper-only remediation path as a revalidation failure.
  - Risk: accepted path derivation or precedence can drift when adjacent helper-discovery or provisioning work lands first.
  - De-risk plan: isolate the preflight contract, name the exact derivation in seam-local review, and revalidate against overlapping packs before promotion.
- **Rollout / safety**:
  - Fail closed when neither accepted staged path exists.
  - Keep dry-run side-effect-free.
  - Preserve the existing Windows unsupported posture and avoid widening macOS promises.
  - Keep the feature orthogonal to future dependency-provisioning flags.
- **Downstream decomposition context**:
  - Why this seam is `closed`: it published the first runtime-facing contracts and recorded the landing/closeout evidence in `governance/seam-1-closeout.md`.
  - Which threads matter most: `THR-01` and `THR-02`.
  - What the first seam-local review focused on: exact version-dir derivation, visible remediation rendering, dry-run no-write proof, `world.enabled` ordering, and override carve-out fidelity.
- **Expected seam-exit concerns**:
  - Contracts likely to publish: `C-01`, `C-02`, `C-03`.
  - Threads likely to advance: `THR-01`, `THR-02`.
  - Review-surface areas likely to shift after landing: accepted path diagrams, stderr remediation wording, and state-write ordering surfaces.
  - Downstream seams most likely to require revalidation: `SEAM-2`, `SEAM-3`.
  - Accepted or published owned-contract artifacts belong here and in closeout evidence, not in pre-exec verification for the producing seam.
