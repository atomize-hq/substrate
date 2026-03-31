---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: provisional
  basis_ref: seam.md#basis
  stale_triggers: []
gates:
  pre_exec:
    review: pending
    contract: pending
    revalidation: pending
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Contract definition: C-01 effective world enabled classifier

- **User/system value**: downstream seams stop guessing “disabled vs broken”; one canonical classifier and one config-error posture becomes the foundation for all disabled-mode diagnostics.
- **Scope (in/out)**:
  - In: define the concrete producer-side contract for `C-01` (rules, boundaries, and verification checklist) and the thread publication bar for `THR-01`.
  - Out: disabled/skipped copy and JSON shapes (owned by downstream seams).
- **Acceptance criteria**:
  - `C-01` contract rules are explicit (precedence, workspace override-ignore, fail-fast exit `2` posture).
  - A minimal API shape is specified for the shared classifier helper (inputs/outputs and error class).
  - Verification checklist enumerates test cases and pass/fail conditions for both `substrate shim doctor` and `substrate health`.
- **Dependencies**:
  - External authoritative precedence contract: `docs/reference/env/contract.md`
  - Resolver API: `crates/shell/src/execution/config_model.rs` (`CliConfigOverrides`, `resolve_effective_config`)
  - Exit code taxonomy basis: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- **Verification**:
  - The checklist below is executable as tests (preferred) or, if needed, deterministic integration assertions in `crates/shell/tests/*`.
- **Rollout/safety**: no new config keys or env vars; this slice only makes existing semantics explicit and testable.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r2---cli--service--data-flow`

#### C-01 contract rules (producer seam)

1. **Authority**: diagnostics must not implement ad-hoc precedence; they must use the existing effective-config resolver (`resolve_effective_config`).
2. **CLI override mapping**:
   - `--world` must set `CliConfigOverrides.world_enabled = Some(true)`
   - `--no-world` must set `CliConfigOverrides.world_enabled = Some(false)`
   - CLI overrides must take precedence over config-layer `world.enabled` in the resolver result.
3. **Workspace override-ignore**: when in an enabled workspace (per the resolver’s rules), `SUBSTRATE_OVERRIDE_*` env overrides are ignored.
4. **Fail-fast config errors**:
   - Any config resolution error (invalid YAML, unreadable config, unsupported legacy workspace config, etc.) must be treated as terminal for diagnostics classification.
   - Both `substrate shim doctor` and `substrate health` must exit with code `2` and emit stderr.
   - No diagnostic probing or user-facing report output occurs on this path.
5. **Output contract for consumers**:
   - Downstream seams consume a single “effective world enabled” decision (boolean or small enum) from a shared helper; they must not re-read config or infer enabled/disabled from probe results when config resolution failed.

#### Proposed shared helper API (minimal, non-binding)

- A single helper callable by both entrypoints, e.g.:
  - `effective_world_enabled(cwd: &Path, cli: &CliConfigOverrides) -> Result<bool>`
  - or `classify_world_enabled(...) -> Result<WorldEnabledDecision>` where errors remain “config errors” and are not converted into “disabled/unknown”.

#### Verification checklist (must be automatable)

- **Config error posture (both commands)**:
  - Invalid YAML in workspace config exits `2`, prints a config error to stderr, and prints no report output.
  - Unreadable config (permissions) exits `2` similarly.
- **Override precedence**:
  - `--no-world` forces disabled classification even if config says enabled.
  - `--world` forces enabled classification even if config says disabled.
- **Workspace override-ignore**:
  - Outside workspace, `SUBSTRATE_OVERRIDE_WORLD` can affect the effective decision (per resolver contract).
  - Inside workspace, `SUBSTRATE_OVERRIDE_WORLD` is ignored (per resolver contract), and CLI flags still win.

#### S1.T1 - Record contract and verification targets in the seam-local slices

- **Outcome**: contract rules and verification checklist are complete enough that `SEAM-2` / `SEAM-3` can plan without guessing.
- **Inputs/outputs**: `threading.md` (C-01/THR-01), `docs/reference/env/contract.md`, `EXIT_CODE_TAXONOMY.md`.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Acceptance criteria**: all checklist items above have a named test location in `S2`.

Checklist:
- Implement: N/A (contract-definition slice)
- Test: N/A (but must name test targets in `S2`)
- Validate: cross-check against `docs/reference/env/contract.md`
- Cleanup: none
