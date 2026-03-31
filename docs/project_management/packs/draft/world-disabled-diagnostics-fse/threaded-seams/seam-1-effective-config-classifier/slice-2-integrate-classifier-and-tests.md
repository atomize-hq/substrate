---
slice_id: S2
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
contracts_produced: []
contracts_consumed:
  - C-01
open_remediations: []
candidate_subslices: []
---
### S2 - Integrate shared classifier and add regression tests

- **User/system value**: makes `THR-01` publishable by enforcing a single call path for both diagnostics commands and preventing regressions (divergence and “probe-before-config”).
- **Scope (in/out)**:
  - In:
    - Add (or confirm) one shared helper that both `substrate shim doctor` and `substrate health` call to determine effective `world.enabled`.
    - Route config-resolution errors to stderr + exit `2` consistently before any probing or report output.
    - Add tests that prove the posture for both entrypoints and cover override/precedence fixtures.
  - Out:
    - Disabled-mode report copy and JSON schema fields (owned by `SEAM-2` / `SEAM-3`).
- **Acceptance criteria**:
  - Both commands call the same helper and share the same error handling.
  - Tests cover config errors, `--world/--no-world` precedence, and workspace override-ignore semantics.
  - No “local precedence” or duplicate config reads remain in the call sites.
- **Dependencies**:
  - `crates/shell/src/execution/config_model.rs` (`CliConfigOverrides`, `resolve_effective_config`, env override parsing)
  - `crates/shell/src/builtins/shim_doctor/*`
  - `crates/shell/src/builtins/health.rs`
  - Existing diagnostics routing behavior in `crates/shell/src/execution/routing.rs`
- **Verification**:
  - Targeted integration tests in `crates/shell/tests/shim_doctor.rs` and `crates/shell/tests/shim_health.rs`
  - Optional unit coverage for the helper in `crates/shell/src/execution/config_model.rs` or the owning module
- **Rollout/safety**: preserve enabled-mode behavior; only tighten routing and remove ambiguous fallbacks on config errors.
- **Review surface refs**: `../../review_surfaces.md#r1---high-level-workflow`, `../../review_surfaces.md#r3---touch-surface-map`

#### S2.T1 - Introduce/centralize the classifier helper used by both commands

- **Outcome**: one shared decision function produces “world enabled?” for diagnostics using the resolver; call sites do not implement precedence.
- **Inputs/outputs**: `(cwd, CliConfigOverrides) -> Result<bool|decision>` as defined in `S1`.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - Prefer locating the helper next to `resolve_effective_config` (or in a small diagnostics module) so consumers cannot bypass the resolver.
  - Ensure `--world/--no-world` mapping is represented solely through `CliConfigOverrides.world_enabled`.
- **Acceptance criteria**:
  - Both entrypoints call this helper exactly once per command execution.
  - Errors remain typed as “user/config errors” and do not get converted into “disabled/unknown”.
- **Test notes**: cover both CLI override states and “no override” baseline.
- **Risk/rollback notes**: if a refactor touches shared routing, keep changes minimal and guarded by tests.

Checklist:
- Implement: shared helper + call-site wiring
- Test: add new tests (below)
- Validate: confirm behavior matches `docs/reference/env/contract.md`
- Cleanup: remove any duplicated precedence logic in call sites

#### S2.T2 - Add regression tests proving exit-2-before-probe/output for config errors

- **Outcome**: invalid config cannot produce misleading probe-backed output; both commands fail fast with code `2`.
- **Inputs/outputs**:
  - Inputs: a temporary workspace config file containing invalid YAML or an unreadable file
  - Outputs: exit code `2`, stderr contains “invalid YAML” / “failed to read”, and stdout contains no report output
- **Thread/contract refs**: `THR-01`, `C-01`
- **Implementation notes**:
  - Prefer table-driven tests covering both commands with the same fixture patterns.
  - If the harness makes “no probes occurred” hard to assert directly, assert that output does not include any probe-derived fields/lines and that the error is emitted before any report framing.
- **Acceptance criteria**:
  - `substrate shim doctor` test: config error -> exit `2`
  - `substrate health` test: config error -> exit `2`
  - Both tests assert consistent error messaging class (config/user error) and absence of report output
- **Test notes**:
  - `crates/shell/tests/shim_doctor.rs`: add `test_config_error_exits_2_before_output` (or equivalent)
  - `crates/shell/tests/shim_health.rs`: add `test_config_error_exits_2_before_output` (or equivalent)
- **Risk/rollback notes**: if tests reveal existing divergence, treat it as a blocking fix for `THR-01` publication.

Checklist:
- Implement: add fixtures + assertions
- Test: `cargo test -p substrate-shell --test shim_doctor` and `--test shim_health`
- Validate: verify both fail paths converge on the same exit taxonomy
- Cleanup: ensure fixtures do not leak env state across tests

#### S2.T3 - Add precedence fixtures for CLI vs env vs workspace layers

- **Outcome**: precedence drift becomes detectable; ensures `--world/--no-world` and workspace override-ignore rules remain stable.
- **Inputs/outputs**: fixtures for workspace-config on/off, `SUBSTRATE_OVERRIDE_WORLD` set/unset, and CLI overrides.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Acceptance criteria**: tests cover:
  - CLI override wins over config `world.enabled`
  - Workspace ignores `SUBSTRATE_OVERRIDE_WORLD` (but CLI still wins)
  - Outside workspace, env override may apply per resolver rules

Checklist:
- Implement: fixture matrix in tests
- Test: run targeted tests
- Validate: cross-check with `docs/reference/env/contract.md`
- Cleanup: none
