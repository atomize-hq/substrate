---
pack_id: world-disabled-reason-attribution-seam-pack
pack_version: v1
pack_status: extracted
source_ref: world-disabled-reason-attribution.zip::world-disabled-reason-attribution/
execution_horizon:
  active_seam: SEAM-1
  next_seam: SEAM-2
---

# Scope Brief - world-disabled-reason-attribution

- **Goal**:
  - Make replay explain host fallback caused by `world.enabled=false` using the same winning-layer semantics already locked by the source pack for ADR-0037 / ADR-0038.
  - Publish that attribution in both human-visible replay copy and machine-readable replay telemetry without changing replay selection behavior.
- **Why now**:
  - The source pack shows a concrete operator gap: replay can fall back to host because world isolation is effectively disabled, but replay can otherwise look generic or appear to blame the wrong knob.
  - The deep-researched pack already locked the exact fragments, telemetry fields, and platform rules; downstream work now benefits from a governance-ready seam pack that preserves that detail without prematurely decomposing future work.
- **Primary user(s) + JTBD**:
  - Operators running `substrate --replay <span_id>` who need to understand why replay stayed on host.
  - Maintainers of replay routing, config provenance, and trace surfaces who need one deterministic contract for precedence, redaction, and telemetry.
  - Trace consumers and documentation maintainers who need stable field names and examples.
- **In-scope**:
  - A shared replay-safe classifier for effective world-disable attribution.
  - Replay origin-summary and host-warning copy for effective-disable cases.
  - Additive `replay_strategy` provenance fields and enum values for effective-disable cases.
  - Regression coverage, docs alignment, smoke wrappers, manual playbook content, and Linux/macOS/Windows parity assertions for the final contract.
- **Out-of-scope**:
  - New replay subcommands, new JSON envelopes, new config keys, or new environment variables.
  - Replay selection-precedence changes, replay backend-selection changes, timeout changes, or exit-code changes.
  - Doctor/health semantics beyond reuse of the already-established ADR-0037 effective-disable classifier semantics.
- **Success criteria**:
  - Replay origin summaries and host warnings use the exact effective-disable fragments locked by the source pack.
  - Replay-local opt-out fragments remain unchanged and distinct from effective-config attribution.
  - `replay_strategy` gains additive structured provenance only for effective-disable cases.
  - Linux, macOS, and Windows use the same reason fragments, `origin_reason_code` values, and tokenized display rules for equivalent cases.
  - No absolute config path and no raw env value leaks into replay stderr or telemetry.
- **Constraints**:
  - Precedence remains: `--world`, `--no-world`, `SUBSTRATE_REPLAY_USE_WORLD`, recorded origin plus `--flip-world`.
  - Effective-disable attribution reuses the winning-layer order already captured by the source pack: override env when no workspace exists, workspace patch, global patch, then unknown-source fallback.
  - Replay emits no additional replay lines outside the origin summary and the existing host warning.
  - Telemetry changes must be additive only and must preserve existing `replay_strategy` fields.
  - This extracted pack must remain one level above seam decomposition.
- **External systems / dependencies**:
  - Replay routing in `crates/shell/src/execution/routing/replay.rs`
  - Config provenance and redaction behavior in `crates/shell/src/execution/config_model.rs` or an adjacent helper seam
  - Replay trace emission in `crates/replay/src/replay/executor.rs`
  - Replay tests in `crates/shell/tests/replay_world.rs`
  - `docs/REPLAY.md`, `docs/TRACE.md`, `docs/COMMANDS.md`
  - Source-pack external anchors: ADR-0037 semantics, ADR-0038 feature scope, and the environment/config contract referenced by the deep-researched plan
  - Supported behavior platforms: `linux`, `macos`, `windows`
- **Known unknowns / risks**:
  - Helper placement must stay narrow enough that replay can reuse winning-layer semantics without inheriting unrelated doctor/health rendering logic.
  - Unknown-source fallback must stay deterministic even when provenance cannot be trusted.
  - Runtime copy, telemetry fields, docs, and smoke assertions can drift if the shared contract is not kept single-source.
  - Platform-local backend differences may add noise outside the reason fragment even though the fragment itself must stay stable.
- **Assumptions**:
  - The source pack is authoritative for this feature's exact reason fragments, telemetry field names, redaction rules, and cross-platform expectations.
  - The source pack's `WDRA0 -> WDRA1 -> WDRA2` ordering is the correct proxy for the feature's critical path.
  - Linux, macOS, and Windows remain the required parity platforms.
  - No hidden requirements outside the source pack require a fourth seam.
  - Any future seam-local planning will treat upstream closeouts and published thread states as the authoritative basis before execution.
