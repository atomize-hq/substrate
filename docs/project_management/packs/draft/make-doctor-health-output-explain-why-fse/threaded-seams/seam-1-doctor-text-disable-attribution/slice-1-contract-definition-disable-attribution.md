---
slice_id: S1
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
### S1 - Contract definition: doctor disable attribution (text)

- **User/system value**: Downstream seams and reviewers can implement doctor disable attribution without guessing: exact message bodies, winner mapping inputs, fallback posture, and redaction/tokenization rules are concrete.
- **Scope (in/out)**:
  - In: make `C-01` and `C-02` concrete enough to implement and later publish from closeout.
  - Out: landing the code changes themselves (handled in S2/S3).
- **Acceptance criteria**:
  - `C-01` string set is exact and treated as a compatibility contract.
  - `C-02` mapping rules bind directly to effective-config winner provenance (no duplicated precedence logic).
  - Enabled-case omission is explicit (“emit nothing when enabled”).
  - Redaction/tokenization rules are explicit (“never print raw env values or absolute host paths”).
- **Dependencies**: none (producer seam contract-definition slice).
- **Verification**:
  - Unit tests will assert the winner-to-message mapping for all layers and for the `source unknown` fallback.
  - Integration tests will assert the enabled-case omission and that no raw paths or env values appear in doctor text.
- **Rollout/safety**:
  - Messaging-only: no change to enablement semantics or exit codes.
  - Misattribution is worse than omission: fallback to `source unknown` is mandatory when proof is unsafe.
- **Review surface refs**:
  - `../../threading.md` (C-01/C-02, THR-01/02)
  - `../../review_surfaces.md` (R1/R2)

#### Contract rules (concrete)

These are the binding contract statements this seam must land and later publish in closeout.

- **C-01 (exact message bodies)**:
  - When `world.enabled = false`, doctor text must emit exactly one of these lines (verbatim):
    - `world isolation disabled by CLI flag --no-world`
    - `world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled`
    - `world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)`
    - `world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)`
    - `world isolation disabled by default config (world.enabled: false)`
    - `world isolation disabled by effective config (source unknown)`
  - When `world.enabled = true`, doctor text must emit no disable-attribution line.
- **C-02 (winner mapping + precedence truth + fallback + redaction)**:
  - Winner mapping must use the `world.enabled` explain provenance from `resolve_effective_config_with_explain(..., explain=true)` and must not re-implement precedence.
  - Precedence truth remains:
    - CLI flags
    - workspace patch
    - override env (only when no enabled workspace exists)
    - global patch
    - default config
  - Fallback is mandatory: if winner attribution cannot be safely proven, emit only the `source unknown` message body (no guessing).
  - Doctor text must never render:
    - raw values from `SUBSTRATE_OVERRIDE_WORLD` (or any other env var)
    - absolute host filesystem paths (including any raw `ConfigExplainSource.path`)
  - When a config path must be mentioned in the message body, use only the stable tokenized display paths shown in `C-01`.

#### S1.T1 - Freeze mapping inputs and runtime loci

- **Outcome**: `C-01` and `C-02` are mapped to concrete runtime inputs and implementation loci.
- **Inputs/outputs**:
  - Inputs: `threading.md`, `config_model.rs` (`resolve_effective_config_with_explain`), doctor entrypoints in `execution/platform/*`.
  - Outputs: a checklist used by S2/S3 to ensure mapping and redaction do not drift.
- **Thread/contract refs**: `THR-01`, `THR-02`; `C-01`, `C-02`.
- **Implementation notes**:
  - Use explain provenance for `world.enabled` as the single source of truth for “who disabled it”.
  - For `cli_flag`, bind specifically to `--no-world` in the disabled case (do not invent other CLI reasons).
  - Do not emit any raw path; use tokenized display strings only.
- **Acceptance criteria**: Every contract bullet has an explicit “input” and “rendered output” definition.
- **Test notes**: Prefer unit tests for mapping and integration tests for doctor output emission/omission.
- **Risk/rollback notes**: If explain provenance is missing or ambiguous in a doctor path, ship `source unknown` rather than a heuristic.

Checklist:
- Implement: N/A (contract slice)
- Test: N/A (contract slice)
- Validate: ensure message bodies match `threading.md` exactly
- Cleanup: none

