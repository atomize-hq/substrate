# make-doctor-health-output-explain-why — impact map (pre-planning)

This file replaces the legacy `integration_map.md`.

Authoring standards:
- `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/make-doctor-health-output-explain-why/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Spec manifest:
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/spec_manifest.md`

## Touch set (explicit)

List every file expected to be created/edited/deprecated/removed. Use repo-relative paths.

Strict packs (`tasks.json` → `meta.slice_spec_version >= 2`) requirements:
- Each entry is a top-level bullet containing exactly one backticked path token.
- Empty section is exactly `- None` (case-sensitive, no extra text).
- The Touch Set must include at least one non-None entry total across all sections.
- To compute pack-derived Work Lift v1 from this Touch Set: `make pm-lift-pack PACK="docs/project_management/packs/draft/make-doctor-health-output-explain-why/"` (strict packs only).

### Create
- `crates/shell/src/execution/world_disable_attribution.rs`
- `crates/shell/tests/world_disable_attribution.rs`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/contract.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/decision_register.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/doctor-health-output-attribution-schema-spec.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/manual_testing_playbook.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/plan.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/session_log.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/DHO0-spec.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/kickoff_prompts/DHO0-code.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/kickoff_prompts/DHO0-integ.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO0/kickoff_prompts/DHO0-test.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/DHO1-spec.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/kickoff_prompts/DHO1-code.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/kickoff_prompts/DHO1-integ.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/slices/DHO1/kickoff_prompts/DHO1-test.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/macos-smoke.sh`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/smoke/windows-smoke.ps1`

### Edit
- `crates/shell/src/builtins/health.rs`
- `crates/shell/src/builtins/shim_doctor/report.rs`
- `crates/shell/src/execution/mod.rs`
- `crates/shell/src/execution/platform/linux.rs`
- `crates/shell/src/execution/platform/macos.rs`
- `crates/shell/src/execution/platform/mod.rs`
- `crates/shell/src/execution/platform/windows.rs`
- `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json`
- `docs/project_management/packs/sequencing.json`

### Deprecate
- None

### Delete
- None

## Cascading implications (behavior/UX)

For each externally visible change, list:
- direct impact (what the operator experiences),
- cascading impact (what else must change to stay coherent),
- contradiction risks (what existing semantics would conflict).

### CLI / UX
- Change: Doctor and health text output includes an accurate “why disabled” attribution line when the *effective* `world.enabled=false`.
  - Direct impact:
    - Operators no longer see the misleading attribution “world isolation disabled … (--no-world)” when the disable source is persisted config (`$SUBSTRATE_HOME/config.yaml`), workspace config (`<workspace>/.substrate/workspace.yaml`), or `SUBSTRATE_OVERRIDE_WORLD=disabled`.
    - The output becomes actionable immediately: it points to the exact knob and, for config-based disablement, a stable display-path token.
  - Cascading impact:
    - `substrate health` must surface the same attribution copy as doctor (parity requirement), and must emit the same information in `--json` mode per the spec manifest.
    - `substrate shim doctor`’s world-doctor snapshot gathering must propagate CLI `--no-world/--world` into the nested `substrate world doctor --json` invocation; otherwise `health/shim doctor --no-world` would misattribute (or fail to attribute) CLI-driven disablement.
    - All platforms must converge on one exact string set (case/punctuation/tokenization), because downstream tests (and some operator scripts) match substrings.
  - Contradiction risks:
    - Misattribution is worse than omission: if the disable source cannot be determined without lying, the contract requires a deterministic “source unknown” fallback rather than guessing.
    - Existing in-flight work that changes health classification when world is disabled (ADR-0036 / WDD pack) must not delete/rename this attribution copy; it should coexist with “disabled/skipped” status UX.

### Config / env vars / paths
- Change: Attribution must reflect the real precedence model used by effective config resolution (including the “workspace gating” rule for env overrides), while avoiding sensitive-path leakage.
  - Direct impact:
    - Operators can distinguish:
      - CLI `--no-world` disablement,
      - env override disablement (`SUBSTRATE_OVERRIDE_WORLD=disabled`) *when applicable*,
      - workspace patch disablement, and
      - global patch disablement.
  - Cascading impact:
    - Implementation must derive attribution from the same resolver precedence used in production routing (preferred: config “explain” provenance for `world.enabled`) so future precedence changes don’t silently desync attribution.
    - All path rendering must use stable display tokens (`$SUBSTRATE_HOME/config.yaml`, `<workspace>/.substrate/workspace.yaml`) and MUST NOT surface absolute host paths (including those present inside existing config “explain” structures).
  - Contradiction risks:
    - Queued ADR-0003 proposes `SUBSTRATE_WORLD=enabled|disabled` as an input layer; current implementation uses `SUBSTRATE_OVERRIDE_WORLD` as the override input and treats `SUBSTRATE_WORLD*` as exported state. This feature must follow the current resolver contract (no behavior change) and explicitly avoid adopting ADR-0003’s env input model early.

### Policy / isolation / security posture
- Change: Additive doctor/health JSON fields describing why world isolation is disabled, with explicit redaction constraints.
  - Direct impact:
    - Automation and CI can machine-detect the disable source (flag vs env vs workspace/global patch) without parsing human text.
  - Cascading impact:
    - JSON fields must be strictly additive and omit-when-enabled to preserve existing consumers; schema/enum placement must be pinned in `doctor-health-output-attribution-schema-spec.md`.
    - No new env vars or protocol changes are allowed (spec manifest), so attribution must be computed in-process (not via new env plumbing or agent APIs).
  - Contradiction risks:
    - Future “json-mode” envelope work must preserve these fields (no rename/removal) and define how they map into an envelope (`data` payload vs top-level), otherwise cross-command JSON consistency efforts can accidentally break disable-attribution tooling.

## Cross-queue scan (ADRs + Planning Packs)

List overlaps/conflicts with other in-flight work and resolve them deterministically.

### Relevant ADRs (queued/unimplemented)
- ADR: `docs/project_management/adrs/draft/ADR-0036-quieting-lemur.md`
  - Overlap surfaces:
    - `substrate health` and `substrate shim doctor` world-disabled handling (same entrypoints this feature must modify for attribution).
  - Conflict: yes (shared files + health UX semantics when world is disabled)
  - Resolution (explicit):
    - Sequence boundary: land ADR-0036 / WDD’s “disabled/skipped” classification first (it restructures health/shim-doctor semantics), then land this feature’s attribution so it can be integrated into the final “disabled” UX without churn.
    - Contract boundary: WDD owns “disabled/skipped” status enums; this feature owns “why disabled” attribution. Both must be present and must not contradict each other.

- ADR: `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`
  - Overlap surfaces:
    - shared disable-attribution semantics + phrasing reused by replay warnings/origin summaries.
  - Conflict: no (dependency)
  - Resolution (explicit):
    - Ensure this feature produces a reusable, tested attribution helper + stable enums/JSON fields so ADR-0038 can reuse it rather than re-implementing heuristics.

- ADR: `docs/project_management/adrs/queued/ADR-0003-policy-and-config-mental-model-simplification.md`
  - Overlap surfaces:
    - world-selection terminology and env var semantics presented to operators/tooling.
  - Conflict: yes (env input model differs from current implementation)
  - Resolution (explicit):
    - Implement attribution against the current effective-config resolver (CLI overrides + `SUBSTRATE_OVERRIDE_WORLD` + workspace/global patch), and treat ADR-0003 alignment as a later migration once ADR-0003 is implemented and its env terminology becomes canonical.

### Relevant Planning Packs (queued/unimplemented)
- Planning Pack: `docs/project_management/packs/draft/world-disabled-diagnostics/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs`
    - `crates/shell/src/builtins/shim_doctor/report.rs`
    - health/shim JSON schema evolution when world is disabled
  - Conflict: yes (same files; both add JSON surface area)
  - Resolution (explicit):
    - Adopt the sequencing boundary above (WDD first), then ensure this feature’s JSON additions are compatible with WDD’s disabled/skipped status enums (no field name collisions; both omit/emit rules must be coherent).

- Planning Pack: `docs/project_management/packs/draft/json-mode/`
  - Overlap surfaces:
    - cross-command JSON envelope design and stability expectations; doctor/health JSON shapes are explicit targets in that plan.
  - Conflict: yes (schema reshaping risk)
  - Resolution (explicit):
    - Non-overlap boundary: this feature adds fields to the existing payload objects; json-mode must preserve them inside any envelope and must not rename/remove them without an explicit compatibility posture.

- Planning Pack: `docs/project_management/packs/draft/world-deps-apt-provisioning/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` (health output + JSON schema changes)
  - Conflict: yes (shared file + health JSON evolution)
  - Resolution (explicit):
    - Sequence boundary: if landed before this feature, health changes must not assume “world disabled” implies “unknown reason”; if landed after, provisioning work must preserve the disable-attribution fields and copy.

- Planning Pack: `docs/project_management/packs/draft/add-non-apt-system-package-provisioning-support/`
  - Overlap surfaces:
    - `crates/shell/src/builtins/health.rs` (operator “next steps” guidance; JSON payload evolution)
  - Conflict: yes (shared file + operator messaging)
  - Resolution (explicit):
    - Treat disable-attribution as orthogonal; provisioning packs must branch their guidance on “world disabled” and must not erase the attribution line that tells operators *why* it is disabled.

- Planning Pack: `docs/project_management/packs/draft/dev-install-world-agent-staging/`
  - Overlap surfaces:
    - “install with `--no-world`, enable later” workflows where attribution reduces confusion about what to flip back on.
  - Conflict: no
  - Resolution (explicit):
    - Keep phrasing consistent: avoid implying “broken backend” when the world is disabled by choice; the attribution line should align with dev-install remediation guidance.

## Follow-ups (explicit)

- Decision Register entries required:
  - DR-0001 — Resolve ADR-0037’s contradiction: provenance-based attribution vs heuristic attribution; selection MUST preserve “attribution matches effective winner” invariant.
  - DR-0002 — Lock the JSON contract: field placement for health JSON, enum value set (including whether `default` is real/emittable), and redaction rules for `path_display`/`env`/`flag`.
- Spec updates required (if any):
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/tasks.json` — populate triad tasks (`DHO0-*`, `DHO1-*`), deps, kickoff prompt paths, and checkpoint boundaries.
  - `docs/project_management/packs/draft/make-doctor-health-output-explain-why/pre-planning/ci_checkpoint_plan.md` — define cross-platform checkpoint groups aligned to `tasks.json`.
  - `docs/project_management/packs/sequencing.json` — add the sequencing entry referenced by ADR-0037.
  - `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` — reconcile Recommendation vs Decision Summary and update Related Docs links once pack artifacts exist.
