---
slice_id: S3
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - effective-config precedence for world.enabled changes
    - exact doctor disable-attribution message bodies change
    - platform doctor renderers bypass tokenized path or env redaction
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
  - THR-02
contracts_produced:
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S3 - Wire doctor output + parity evidence

- **User/system value**: Host and world doctor text surfaces emit the correct disable attribution line (or omit it) across platforms without leaking secrets or paths.
- **Scope (in/out)**:
  - In: plumb explain provenance into doctor entrypoints, call the shared helper, and update platform doctor output to print the attribution line when disabled.
  - Out: health + JSON surfaces (owned by `SEAM-2`).
- **Acceptance criteria**:
  - Host doctor and world doctor use `resolve_effective_config_with_explain(..., explain=true)` at the entrypoint (or an equivalent safe wrapper) so winner attribution is provenance-backed.
  - When enabled, no attribution line is printed.
  - When disabled, exactly one `C-01` line is printed and matches the effective winner; otherwise `source unknown`.
  - No raw env values or absolute paths appear in the output.
- **Dependencies**:
  - shared helper from S2
  - `crates/shell/src/execution/platform/mod.rs` doctor routing
  - platform doctor renderers in `crates/shell/src/execution/platform/{linux,macos,windows}.rs`
- **Verification**:
  - Integration tests for doctor output (text mode) across the winner set (CLI, env, workspace, global, default, unknown).
  - Manual parity evidence: run doctor on Linux/macOS/Windows (where supported) and confirm wording + omission behavior.
- **Rollout/safety**:
  - Keep output changes minimal: add one attribution line only when disabled.
  - Preserve existing exit codes and ok/fail behavior.
- **Review surface refs**:
  - `../../review_surfaces.md` (R1/R2)
  - `../../threading.md` (THR-01/02; C-01/C-02)

#### S3.T1 - Plumb explain provenance into doctor entrypoints

- **Outcome**: Doctor entrypoints have access to provenance-backed `world.enabled` winner sources.
- **Inputs/outputs**:
  - Inputs: `Cli` flags (`--world`/`--no-world`) and the current working directory.
  - Outputs: effective config + explain payload (or safe fallback) for the attribution helper.
- **Thread/contract refs**: `THR-01`; `C-02`.
- **Implementation notes**:
  - Prefer a small wrapper function at the entrypoint to prevent call sites from accidentally using non-explain resolution.
  - Ensure `workspace_patch` vs `override_env` semantics match the existing resolver (env only applies when no workspace exists).
- **Acceptance criteria**: Every doctor code path uses the same provenance plumbing.
- **Test notes**: Add a focused integration test that proves `cli_flag` attribution survives end-to-end for `--no-world`.
- **Risk/rollback notes**: If a platform doctor path cannot obtain explain provenance, it must print `source unknown` rather than a guessed layer.

Checklist:
- Implement: wire explain into host/world doctor entrypoints; print attribution when disabled
- Test: integration tests for all disable sources + enabled omission
- Validate: run `substrate host doctor` and `substrate world doctor` manually on supported platforms
- Cleanup: none

