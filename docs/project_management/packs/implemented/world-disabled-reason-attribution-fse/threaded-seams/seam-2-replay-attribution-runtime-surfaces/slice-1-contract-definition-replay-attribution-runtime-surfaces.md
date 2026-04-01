---
slice_id: S1
seam_id: SEAM-2
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - SEAM-1 changes replay-safe attribution fields or redaction posture
    - replay copy fragments or telemetry enums drift from the source contract
    - a reachable runtime case appears outside the enumerated effective-disable set
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
  - C-04
contracts_consumed:
  - C-01
  - C-02
open_remediations: []
candidate_subslices: []
---
### S1 - Contract definition: replay attribution runtime surfaces

- **User/system value**: implementation and review can proceed without guessing the exact replay fragments, telemetry keys, omit rules, or redaction behavior for runtime adoption.
- **Scope (in/out)**:
  - In: make `C-03` and `C-04` concrete enough that replay stderr and `replay_strategy` can implement them against the published `SEAM-1` handoff.
  - Out: landing the runtime code changes themselves (handled in S2/S3).
- **Acceptance criteria**:
  - `C-03` makes the exact replay copy fragments and recorded-host punctuation explicit.
  - `C-04` makes the additive telemetry field names, enum values, object keys, and omit rules explicit.
  - Replay-local opt-out cases remain unchanged and do not emit `world_disable_source`.
  - Redaction/tokenization rules are explicit for env and path displays.
- **Dependencies**:
  - `../../threading.md`
  - `../../governance/seam-1-closeout.md`
  - source anchors in `../../../world-disabled-reason-attribution/contract.md`, `telemetry-spec.md`, `decision_register.md`, and `slices/WDRA1/WDRA1-spec.md`
- **Verification**:
  - Extend `crates/shell/tests/replay_world.rs` with exact-string coverage for origin summaries, recorded-host output, host warnings, and `replay_strategy` fields.
  - Reuse helper-level attribution tests from `SEAM-1`; do not duplicate precedence logic in replay-specific tests.
- **Rollout/safety**:
  - Additive only: replay keeps the same selection behavior, lines, timeouts, and exit semantics.
  - If implementation reveals a reachable runtime case outside the enumerated effective-disable set, open a blocker rather than widening the contract implicitly.
- **Review surface refs**:
  - `../../threading.md`
  - `../../review_surfaces.md`

#### Contract rules (concrete)

These are the binding contract statements this seam must land and later publish in closeout.

- **C-03 (replay copy contract)**:
  - Replay-local reason fragments remain unchanged:
    - `--world flag`
    - `--no-world flag`
    - `SUBSTRATE_REPLAY_USE_WORLD=disabled`
    - `--flip-world`
    - `recorded origin (span)`
    - `recorded origin (replay_context)`
    - `default origin`
  - Effective-disable attribution uses these exact fragments:
    - `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
    - `world isolation disabled by config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
    - `world isolation disabled by config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`
    - `world isolation disabled by effective config (source unknown)`
  - Origin summary formats remain:
    - direction-changing case: `[replay] origin: <from> -> <to> (<reason>)`
    - recorded-host case: `[replay] origin: host (recorded; <reason>)`
  - Host warning remains: `[replay] warn: running on host (<reason>)`
  - Replay does not emit additional replay lines outside the existing origin summary and host warning.
- **C-04 (replay_strategy telemetry contract)**:
  - The replay runtime publishes only this narrowed effective-disable layer set:
    - `override_env`
    - `workspace_patch`
    - `global_patch`
    - `unknown`
  - The corresponding runtime `origin_reason_code` values are:
    - `world_disabled_override_env`
    - `world_disabled_workspace_patch`
    - `world_disabled_global_patch`
    - `world_disabled_unknown`
  - Helper `source_unknown` maps to the runtime `unknown` case and is the only safe fallback when provenance is unavailable.
  - A reachable helper `default` layer is not a valid runtime publication under `C-03` / `C-04`; if the helper can still produce `default` for a replay path, that path remains a pre-land blocker until this contract is explicitly revised.
  - `origin_reason` equals the same exact fragment used in replay stderr.
  - `origin_reason_code` gains only these additive effective-disable values:
    - `world_disabled_override_env`
    - `world_disabled_workspace_patch`
    - `world_disabled_global_patch`
    - `world_disabled_unknown`
  - `world_disable_source` is optional and emitted only when `origin_reason_code` is one of the `world_disabled_*` values above.
  - `world_disable_source` keys:
    - `key` is always `world.enabled`
    - `layer` is one of `override_env | workspace_patch | global_patch | unknown`
    - `env` appears only for `override_env` and must be `SUBSTRATE_OVERRIDE_WORLD`
    - `path_display` appears only for `workspace_patch` or `global_patch` and must be `<workspace>/.substrate/workspace.yaml` or `$SUBSTRATE_HOME/config.yaml`
    - `value_display` is always `false`
  - No raw env values, raw config values, or absolute filesystem paths may appear.
  - If implementation discovers a reachable runtime case that would require a new `world_disabled_*` enum beyond the list above, that is a pre-land blocker to resolve explicitly.

#### S1.T1 - Freeze runtime copy + telemetry rules and verification loci

- **Outcome**: `C-03` and `C-04` are mapped to the exact runtime surfaces and tests that will prove them.
- **Inputs/outputs**:
  - Inputs: published `SEAM-1` attribution contract, replay source contract, telemetry spec, and current replay wiring surfaces.
  - Outputs: a concrete execution checklist for S2 and S3.
- **Thread/contract refs**: `THR-01`, `THR-02`; `C-01`, `C-02`, `C-03`, `C-04`
- **Implementation notes**:
  - Reuse upstream winner provenance; do not add a second precedence table inside replay.
  - Keep runtime copy and telemetry derived from the same attribution decision so they cannot drift.
  - Treat the helper-level `default` layer as an explicit blocker if it appears in a reachable replay path and cannot be mapped under the published runtime contract cleanly.
- **Acceptance criteria**: every required field, fragment, enum value, and test location is explicit.
