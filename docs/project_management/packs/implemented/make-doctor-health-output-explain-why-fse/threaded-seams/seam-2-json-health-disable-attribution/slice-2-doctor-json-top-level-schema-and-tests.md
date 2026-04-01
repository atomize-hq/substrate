---
slice_id: S2
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - doctor JSON root placement changes
    - structured source fields diverge from the published winner truth
    - unsupported-platform doctor payloads special-case the schema and drift
gates:
  pre_exec:
    review: passed
    contract: passed
    revalidation: passed
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S2 - Doctor JSON top-level schema and tests

- **User/system value**: host and world doctor JSON payloads expose the disable reason/source as stable top-level fields, so automation does not need to scrape nested text or infer the winning layer.
- **Scope (in/out)**:
  - In: add the structured source builder, reuse the exact reason string, and emit the two additive top-level fields from doctor JSON entrypoints across supported platform payloads.
  - Out: health-specific plumbing and human-mode parity work (S3).
- **Acceptance criteria**:
  - The same helper logic drives `world_disable_reason` and `world_disable_source`.
  - Fields are emitted only when disabled and are omitted together when enabled.
  - Root placement is consistent across host doctor and world doctor JSON payloads.
  - Existing unsupported/fixture-backed doctor payload semantics remain additive-only.
- **Dependencies**:
  - `slice-1-contract-definition-json-health-disable-attribution.md`
  - `crates/shell/src/execution/config_model.rs`
  - `crates/shell/src/execution/platform/mod.rs`
  - `crates/shell/src/execution/platform/{linux,macos,windows}.rs`
- **Verification**:
  - Extend `crates/shell/tests/doctor_scopes_ds0.rs` to assert the new top-level fields for disabled host/world doctor JSON and their omission in enabled cases.
  - Cover at least one case each for `cli_flag`, `override_env`, `workspace_patch`, and `source_unknown`, with the remaining layers proven via helper/unit coverage if needed.
- **Rollout/safety**:
  - Preserve current `schema_version`, `platform`, `world_enabled`, `ok`, `host`, and `world` semantics.
  - Do not force consumers to dig into nested blocks to understand why world is disabled.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### S2.T1 - Add a shared structured disable-source builder

- **Outcome**: a helper produces the top-level reason plus the serialized `world_disable_source` object from the existing explain winner.
- **Inputs/outputs**:
  - Inputs: effective `world.enabled`, `ConfigExplainKey` for `world.enabled`, and the exact `C-01` reason mapping.
  - Outputs: omitted fields for enabled mode, otherwise one exact reason string plus one serialized source object.
- **Thread/contract refs**: `THR-01`; `C-01`, `C-02`, `C-03`.
- **Implementation notes**:
  - Keep the builder close to the existing doctor-attribution helper or promote both into one shared module so text and JSON cannot drift.
  - The builder must never serialize raw `ConfigExplainSource.path`.
  - Prefer a single struct that can be attached to doctor JSON and later reused by health plumbing.
- **Acceptance criteria**: every supported disable source maps to exactly one serialized object shape.
- **Test notes**: add focused helper tests for `layer`, `flag`, `env`, and `path_display` emission rules.
- **Risk/rollback notes**: ambiguous provenance must serialize as `source_unknown`, not guess at a layer.

Checklist:
- Implement: shared structured builder + root-field plumbing for doctor JSON
- Test: helper coverage + `doctor_scopes_ds0.rs` JSON assertions
- Validate: confirm field order/placement remains root-level and additive
- Cleanup: remove any temporary duplicated mapping logic
