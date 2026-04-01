---
slice_id: S2
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: exec-ready
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
### S2 - Shared helper + winner mapping tests

- **User/system value**: A single shared helper produces the doctor attribution line from effective-config provenance, ensuring all platforms and call paths reuse the same mapping, fallback, and redaction/tokenization rules.
- **Scope (in/out)**:
  - In: implement the shared disable-attribution helper and unit tests for winner-to-message mapping.
  - Out: platform-specific output wiring and parity evidence (S3).
- **Acceptance criteria**:
  - Helper accepts only the minimum inputs needed to avoid heuristic behavior (effective config + explain provenance + CLI flag provenance if required).
  - Mapping implements `C-01` and `C-02` exactly, including enabled-case omission and `source unknown` fallback.
  - Helper never returns raw paths or env values; only stable tokenized display strings are possible outputs.
- **Dependencies**:
  - `crates/shell/src/execution/config_model.rs::resolve_effective_config_with_explain`
  - existing `Cli` flags `--world` / `--no-world` plumbing in `execution/platform/mod.rs`
- **Verification**:
  - Unit tests cover each explain-layer mapping (`cli_flag`, `override_env`, `workspace_patch`, `global_patch`, `default`) plus missing/ambiguous provenance => `source unknown`.
  - Unit tests prove the helper cannot emit `ConfigExplainSource.path` or any env value.
- **Rollout/safety**:
  - Ensure mapping tables are centralized and reused by all renderers.
  - Prefer enums for disable-source classification, but keep text message bodies as literal stable strings.
- **Review surface refs**:
  - `../../threading.md` (C-01/C-02 layers and precedence)
  - `../../review_surfaces.md` (R2)

#### S2.T1 - Add a shared disable-attribution helper

- **Outcome**: A helper returns `Option<&'static str>` (or equivalent) for the doctor attribution line, based on `world.enabled` + provenance.
- **Inputs/outputs**:
  - Inputs: effective `world.enabled`, `ConfigExplainV1` (or just the `world.enabled` key’s first source), and CLI no-world provenance.
  - Output: `None` when enabled; otherwise one of the exact `C-01` message bodies.
- **Thread/contract refs**: `THR-01`, `THR-02`; `C-01`, `C-02`.
- **Implementation notes**:
  - Do not expose raw explain paths to callers; tokenization is mandatory.
  - Do not re-implement precedence: trust effective config + explain sources.
  - If the explain payload is missing, malformed, or lacks a provable source for `world.enabled`, return `source unknown`.
- **Acceptance criteria**: The helper can be used identically by host doctor and world doctor code paths.
- **Test notes**: Add unit tests adjacent to the helper module.
- **Risk/rollback notes**: If provenance is hard to plumb to a call site, use `source unknown` until the provenance path is fixed.

Checklist:
- Implement: add helper + mapping table + safe inputs
- Test: unit tests for mapping and fallback
- Validate: ensure message bodies are byte-for-byte stable
- Cleanup: none
