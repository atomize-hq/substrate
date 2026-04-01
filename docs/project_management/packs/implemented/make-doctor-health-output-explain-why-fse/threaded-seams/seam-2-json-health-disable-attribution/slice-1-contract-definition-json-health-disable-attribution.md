---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - SEAM-1 changes exact message bodies or winner precedence after deferred native parity proof
    - top-level doctor or health payload shape changes
    - disabled-path helpers start exposing raw env values or absolute paths
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
  - THR-02
contracts_produced:
  - C-03
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S1 - Contract definition: JSON + health disable attribution

- **User/system value**: implementation and review can proceed without guessing the shape, placement, omit rules, or parity requirements for the structured disable-attribution contract.
- **Scope (in/out)**:
  - In: make `C-03` concrete enough that doctor JSON and health can implement it against the already-published `C-01` / `C-02` truth.
  - Out: landing the runtime code changes themselves (handled in S2/S3).
- **Acceptance criteria**:
  - The root-level field names, object keys, enum vocabulary, conditional keys, and omit rules are explicit.
  - `world_disable_reason` is tied to the exact `C-01` string set and not a paraphrase.
  - `world_disable_source` uses the same winner truth as `world.enabled`; no duplicated precedence logic is introduced.
  - Redaction/tokenization rules are explicit for env and path displays.
- **Dependencies**:
  - `../../threading.md` (`C-01`, `C-02`, `THR-01`, `THR-02`)
  - `../../governance/seam-1-closeout.md`
- **Verification**:
  - Extend `crates/shell/tests/doctor_scopes_ds0.rs` with JSON emit/omit coverage for CLI, env, workspace, global, default, and source-unknown cases.
  - Extend `crates/shell/tests/shim_health.rs` with disabled health JSON and human-mode parity coverage, including `--no-world`.
  - Use `crates/shell/tests/shim_doctor.rs` only where shim-doctor report changes are needed to preserve disabled-path attribution into health.
- **Rollout/safety**:
  - Additive only: existing JSON readers that ignore unknown fields remain compatible.
  - Health text must reuse the already-published reason string instead of inventing a second wording table.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### Contract rules (concrete)

These are the binding contract statements this seam must land and later publish in closeout.

- **C-03 root placement**:
  - On every in-scope disabled JSON payload, emit both `world_disable_reason` and `world_disable_source` at the payload root, alongside existing top-level fields such as `world_enabled` and `ok`.
  - On enabled JSON payloads, omit both fields together.
- **`world_disable_reason`**:
  - The value is exactly one published `C-01` message body:
    - `world isolation disabled by CLI flag --no-world`
    - `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
    - `world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
    - `world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`
    - `world isolation disabled by default config (world.enabled: false)`
    - `world isolation disabled by effective config (source unknown)`
- **`world_disable_source` object**:
  - `key` is always `world.enabled`.
  - `layer` is one of `cli_flag | override_env | workspace_patch | global_patch | default | source_unknown`.
  - `value_display` is always `false`.
  - Conditional fields:
    - `flag` appears only for `cli_flag` and must be `--no-world`.
    - `env` appears only for `override_env` and must be `SUBSTRATE_OVERRIDE_WORLD`.
    - `path_display` appears only for `workspace_patch` or `global_patch` and must be `<workspace>/.substrate/workspace.yaml` or `$SUBSTRATE_HOME/config.yaml`.
  - No raw env values, raw config values, or absolute filesystem paths may appear.
- **Health text parity**:
  - When disabled, human-mode `substrate health` must print the same `C-01` reason string that doctor text would print for the same winner, before any “Next” guidance.
  - When enabled, no disable-attribution line is printed.

#### Verification plan

- **Tests to add or extend**:
  - `crates/shell/tests/doctor_scopes_ds0.rs`
  - `crates/shell/tests/shim_health.rs`
  - `crates/shell/tests/shim_doctor.rs` if report-shape changes are required
- **Edge cases to cover**:
  - CLI `--no-world`
  - env override without an enabled workspace
  - workspace override defeating env override
  - global/default disable
  - missing or ambiguous provenance mapping to `source_unknown`
  - enabled-case omission on every in-scope payload
- **Pass/fail**:
  - Pass only if the JSON fields are present exactly when disabled, omitted exactly when enabled, and every string/token matches the rules above.
  - Fail on any root-placement drift, wording drift, raw path leakage, or `cli_flag` loss in health/shim disabled paths.

#### S1.T1 - Freeze `C-03` inputs and verification loci

- **Outcome**: `C-03` is mapped to the exact runtime sources and test files that will prove it.
- **Inputs/outputs**:
  - Inputs: `C-01`, `C-02`, current doctor JSON envelopes, shim-doctor report shape, and health summary behavior.
  - Outputs: a concrete execution checklist for S2 and S3.
- **Thread/contract refs**: `THR-01`, `THR-02`; `C-01`, `C-02`, `C-03`.
- **Implementation notes**:
  - Reuse upstream winner provenance; do not add a second precedence table.
  - Treat unsupported Windows host-doctor behavior as additive-only compatibility work, not a reason to fork the schema.
- **Acceptance criteria**: every required field and every required test location is explicit.
- **Test notes**: prefer JSON integration tests for schema and health tests for human-mode parity.
- **Risk/rollback notes**: if a payload cannot safely preserve attribution, block that implementation path rather than inventing a fallback beyond `source_unknown`.

Checklist:
- Implement: N/A (contract slice)
- Test: N/A (contract slice)
- Validate: ensure every contract bullet maps back to `C-01` and `C-02`
- Cleanup: none
