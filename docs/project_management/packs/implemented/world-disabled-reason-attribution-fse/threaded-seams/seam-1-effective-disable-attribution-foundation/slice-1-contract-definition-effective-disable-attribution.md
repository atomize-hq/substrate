---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - ADR-0037 winning-layer precedence changes for world.enabled=false
    - tokenized display rules change for workspace or global config paths
    - allowlisted env token display rules change for SUBSTRATE_OVERRIDE_WORLD
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
  - C-01
  - C-02
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Contract definition: effective-disable attribution (foundation)

- **User/system value**: Downstream seams can wire replay copy and telemetry without re-deriving precedence or redaction rules; `C-01`/`C-02` are concrete and bounded to replay-safe structured metadata.
- **Scope (in/out)**:
  - In: make `C-01` and `C-02` concrete enough to implement and later publish from closeout.
  - Out: replay copy/telemetry wiring (`SEAM-2`).
- **Acceptance criteria**:
  - Layer vocabulary is explicit and stable.
  - `source_unknown` fallback is explicit and used whenever winner proof is missing/ambiguous.
  - No raw env values or absolute host paths are emitted; only fixed allowlisted tokens and tokenized display paths may appear.
  - Workspace config continues to beat override env when a workspace exists (no precedence drift).
- **Dependencies**: none (producer seam contract-definition slice).
- **Verification**:
  - Deterministic tests cover every layer and the fallback behavior.
  - Negative assertions prove redaction invariants (no raw paths/env values).
- **Rollout/safety**:
  - Foundation only; no new user-visible output is introduced in this slice.
- **Review surface refs**:
  - `../../threading.md` (C-01/C-02, THR-01/02)
  - `../../review_surfaces.md` (R1/R2)

#### Contract rules (concrete)

These are the binding contract statements this seam must land and later publish in closeout.

- **C-01 (classifier result shape)**:
  - The classifier is evaluated only when the effective value is `world.enabled=false`.
  - The classifier returns a structured source object with:
    - `key = "world.enabled"`
    - `layer ∈ {"cli_flag","override_env","workspace_patch","global_patch","default","source_unknown"}`
    - `value_display = false` always
    - exactly one of:
      - `flag = "--no-world"` when `layer = cli_flag`
      - `env = "SUBSTRATE_OVERRIDE_WORLD"` when `layer = override_env`
      - `path_display = "<workspace>/.substrate/workspace.yaml"` when `layer = workspace_patch`
      - `path_display = "$SUBSTRATE_HOME/config.yaml"` when `layer = global_patch`
      - no `flag|env|path_display` when `layer = default` or `source_unknown`
- **C-02 (provenance / precedence / redaction semantics)**:
  - Attribution must bind to effective-config provenance; it must not re-run precedence logic in downstream seams.
  - Winner proof must be treated as *unsafe* when:
    - the provenance entry is missing
    - the provenance entry has zero sources
    - the provenance entry has more than one source
    - the provenance layer is unknown
  - In all unsafe cases, the layer must be `source_unknown` and must not guess.
  - The raw source path, if present in provenance, must never be emitted; only tokenized `path_display` may be returned.

#### S1.T1 - Establish the shared helper + type naming

- **Outcome**: one shared helper/type is usable by replay and tests without depending on doctor-specific wording.
- **Inputs/outputs**:
  - Input: `ConfigExplainV1.keys["world.enabled"]`
  - Output: `C-01`/`C-02` structured attribution result
- **Thread/contract refs**: `THR-01` / `C-01`, `THR-02` / `C-02`
- **Implementation notes**:
  - Prefer reusing `world_disable_attribution(...)` in `crates/shell/src/execution/config_model.rs`.
  - If doctor naming is a coupling hazard, introduce a replay-safe wrapper type that carries only the structured source contract.
- **Acceptance criteria**:
  - Layer vocabulary matches the contract.
  - The function never emits raw paths or raw env values.
