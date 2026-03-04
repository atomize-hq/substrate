# world-disabled-reason-attribution — spec manifest

This file enumerates every contract/protocol/schema/env-var surface for this feature and assigns each surface to exactly one authoritative document.

Authoring standard:
- `docs/project_management/system/standards/planning/PLANNING_SPEC_DETERMINATION_STANDARD.md`

## Inputs
- Feature directory: `docs/project_management/packs/draft/world-disabled-reason-attribution/`
- ADR(s):
  - `docs/project_management/adrs/draft/ADR-0038-replaying-raccoon.md`

External authoritative inputs (this feature MUST NOT redefine these surfaces):
- World-disable attribution contract (tokens/enums/strings + precedence): `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md`
- Environment variable parsing + precedence:
  - `docs/reference/env/contract.md` (`SUBSTRATE_OVERRIDE_WORLD`, `SUBSTRATE_REPLAY_USE_WORLD`, `SUBSTRATE_REPLAY_VERBOSE`, `SUBSTRATE_HOME`)
- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`

## Required spec documents (authoritative)

Each entry is feature-local (must live under `docs/project_management/packs/draft/world-disabled-reason-attribution/`) and must be treated as authoritative for the surfaces listed.

- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/spec_manifest.md`
  - Owns (authoritative): required-doc selection + surface→doc ownership map (this file)
  - Links to (non-authoritative): ADR-0038, ADR-0037, env contract, exit code taxonomy
- `docs/project_management/packs/draft/world-disabled-reason-attribution/pre-planning/impact_map.md`
  - Owns (authoritative): touch set + cascading implications + cross-queue conflicts
  - Links to (non-authoritative): all feature-local specs + referenced upstream contracts
- `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`
  - Owns (authoritative): execution sequencing/runbook + dependency ordering (must include ADR-0037 dependency)
  - Links to (non-authoritative): slice specs + tasks.json + decision register
- `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`
  - Owns (authoritative): triad task graph + acceptance criteria IDs referencing slice spec(s)
  - Links to (non-authoritative): slice specs + plan
- `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md`
  - Owns (authoritative): A/B decisions and selections for this feature (must include ADR-0038 DR-0001/DR-0002)
  - Links to (non-authoritative): ADR-0038 + the feature-local specs affected by each decision
- `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
  - Owns (authoritative): replay operator-facing contract changes introduced by ADR-0038 (stderr messaging + stable attribution integration + redaction constraints)
  - Links to (non-authoritative): ADR-0037 (attribution strings/enums), env contract, exit code taxonomy, `docs/REPLAY.md` (must be updated later to match)
- `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
  - Owns (authoritative): trace/log contract changes introduced by ADR-0038 (replay_strategy fields for disable attribution + redaction rules + stability guarantees)
  - Links to (non-authoritative): ADR-0037 (schema/enums reused), `docs/TRACE.md` (must be updated later to match)
- `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`
  - Owns (authoritative): vertical slice behavior + acceptance criteria for “replay world-disabled reason attribution” (ADR-0038 C0–C2 collapsed into one feature slice)
  - Links to (non-authoritative): `contract.md`, `telemetry-spec.md`, ADR-0037, ADR-0038

## Coverage matrix (surface → authoritative doc)

Every surface touched by ADR-0038 must appear here.

| Surface | Authoritative doc | What must be explicitly defined |
| --- | --- | --- |
| Replay command surface: `substrate --replay <SPAN_ID>` | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | which replay stderr surfaces are in scope for this feature (origin summary + host-mode warning); statement that routing/selection semantics are unchanged |
| Replay verbose gate: `--replay-verbose` and `SUBSTRATE_REPLAY_VERBOSE=1` | `docs/reference/env/contract.md` | parsing rules + precedence between flag and env; explicit statement that the changed stderr surfaces are emitted only when verbose is enabled (absence semantics: no verbose ⇒ no origin/warn lines) |
| Replay world-toggle precedence: `--world` > `--no-world` > `SUBSTRATE_REPLAY_USE_WORLD` | `docs/reference/env/contract.md` | precedence order + allowed disable values for `SUBSTRATE_REPLAY_USE_WORLD` |
| Env disable source: `SUBSTRATE_OVERRIDE_WORLD=disabled` | `docs/reference/env/contract.md` | allowed values + precedence rules (including “ignored when workspace enabled”) |
| Config disable sources (display paths): `<workspace>/.substrate/workspace.yaml`, `$SUBSTRATE_HOME/config.yaml` | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | exact tokenized display paths; MUST NOT print absolute host paths |
| Config key used for attribution: `world.enabled` | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | type + meaning; absence semantics (enabled vs disabled) as required for attribution |
| Disable-attribution precedence (highest-precedence winner selection) | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | deterministic winner selection order and the “workspace disables env override” rule |
| Disable-attribution human strings (doctor/health canonical wording reused by replay) | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | exact string set and when each applies; replay MUST reuse verbatim (case/punctuation) |
| Replay origin summary line (stderr, verbose): `[replay] origin: …` | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | exact line template(s); exactly where the attribution appears; requirement that when host-mode is caused by world-disablement, the reason shown is the ADR-0037 attribution string (not a misleading flag/env) |
| Replay host warning line (stderr, verbose): `[replay] warn: running on host (…)` | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | exact line template; exact attribution substitution rules (see above); absence semantics (no warning when replay is host because the recorded origin is host and no opt-out/disablement applies) |
| Redaction invariant (stderr): env values and paths | `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md` | MUST NOT print raw `SUBSTRATE_OVERRIDE_WORLD` values beyond the fixed token `SUBSTRATE_OVERRIDE_WORLD=disabled`; MUST NOT print absolute paths; MUST use ADR-0037 display tokens |
| Trace event: `event_type="replay_strategy"` | `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` | additive field contract for disable attribution; conditional emission; backward-compat policy (“additive only”) |
| Trace field (existing): `origin_summary` | `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` | type (`string`), stability promise, and requirement that it matches the replay origin summary stderr line format defined in `contract.md` |
| Trace field (existing): `origin_reason` | `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` | type (`string`); meaning remains “origin selection reason” (replay toggles / recorded origin / flip); MUST NOT be the sole carrier of world-disable attribution for this feature |
| Trace field (existing): `origin_reason_code` | `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` | type (`string`); meaning remains “origin selection reason code”; MUST NOT be the sole carrier of world-disable attribution for this feature |
| Trace field (new): `world_disable_reason` | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | enum values + meaning (reused verbatim) |
| Trace field (new): `world_disable_source` | `docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md` | object keys/types + redaction rules (reused verbatim) |
| Trace field placement for `world_disable_reason` / `world_disable_source` on `replay_strategy` entries | `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md` | when present (only when replay runs host due to world-disablement) + explicit absence semantics when not applicable |
| Slice scope + acceptance criteria IDs | `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md` | complete AC list, including precedence cases and redaction assertions; explicit out-of-scope list matching ADR-0038 |

## Determinism checklist (must be satisfied before quality gate)

Each required document must explicitly define the items below (no implied behavior).

### `docs/project_management/packs/draft/world-disabled-reason-attribution/contract.md`
- Exact stderr line templates for the replay origin summary and the host-mode warning, including:
  - when each line is emitted (inputs + gating; absence semantics)
  - where the attribution string appears (and that the attribution text itself is owned by ADR-0037)
- Deterministic attribution selection rule for replay messaging:
  - “highest-precedence disable source” MUST match ADR-0037
  - if attribution provenance cannot be determined, the contract MUST define a single fallback string (must not misattribute to `--no-world`)
- Explicit redaction rules (singular and testable):
  - absolute host paths MUST NOT appear
  - `SUBSTRATE_OVERRIDE_WORLD` MUST be referenced without leaking arbitrary values (only the fixed token allowed by ADR-0037)
- Exit codes:
  - reference `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
  - explicitly state “no exit code changes for this feature”
- Platform guarantees (Linux/macOS/Windows): same attribution semantics and redaction guarantees

### `docs/project_management/packs/draft/world-disabled-reason-attribution/telemetry-spec.md`
- Trace event(s) in scope (at minimum: `replay_strategy`) and stability policy (additive-only change)
- Exact field list for disable attribution on `replay_strategy`, including:
  - `world_disable_reason` / `world_disable_source` emission conditions (and explicit absence semantics)
  - explicit absence semantics (when fields MUST be omitted)
- Exact meaning of existing fields (`origin_reason`, `origin_reason_code`, `origin_summary`) after this feature ships, including any redefinition (must be singular)
- Redaction rules for each affected field (including tokenized paths and env key-only constraints)

### `docs/project_management/packs/draft/world-disabled-reason-attribution/slices/WDRA0/WDRA0-spec.md`
- Scope:
  - enumerate the replay stderr surfaces to change (origin summary + host-mode warning)
  - enumerate the trace surface to change (replay_strategy fields)
- Acceptance criteria:
  - test matrix covering disable attribution winners (CLI `--no-world`, env override, workspace config, global config)
  - redaction assertions (no absolute paths; no env value leaks beyond allowed fixed tokens)
  - explicit “no routing/selection semantics changes” assertions (only attribution/messaging)
- Dependencies:
  - ADR-0037 must be integrated before this slice executes

### `docs/project_management/packs/draft/world-disabled-reason-attribution/decision_register.md`
- DR-0001: shared helper reuse vs replay-local duplication (A/B; one selection)
- DR-0002: trace/JSON surface strategy for replay (`world_disable_reason` fields vs reuse/overload existing `origin_reason(_code)`; A/B; one selection)

### `docs/project_management/packs/draft/world-disabled-reason-attribution/plan.md`
- Sequencing dependency on ADR-0037 integration (explicit gating)
- Validation commands to run (unit + integration targets), matching ADR-0038

### `docs/project_management/packs/draft/world-disabled-reason-attribution/tasks.json`
- Slice tasks for `WDRA0` (code/test/integ) referencing `slices/WDRA0/WDRA0-spec.md`

## Follow-ups

- ADR precedence conflict: ADR-0038 lists env override before workspace config, but ADR-0037 + `docs/reference/env/contract.md` state that `SUBSTRATE_OVERRIDE_WORLD` is ignored when a workspace is enabled. Planning must reconcile this and adopt ADR-0037 as the single authoritative precedence contract for attribution.
- Replay semantics clarification: ADR-0038 attributes host-mode replay to workspace/global config disablement; current replay docs emphasize replay-local toggles. The slice spec must explicitly state the exact condition under which replay is considered “host due to world isolation being disabled” (vs “host due to recorded origin” vs “host due to replay-only opt-out”).
- Decision register hygiene: ADR-0038 requires DR-0002; the decision register must record A/B options but the selected replay trace contract must remain additive and must not overload `origin_reason(_code)` as the only carrier of world-disable attribution.
- Docs sync (out of scope for this constrained write): once the contract is finalized, update `docs/REPLAY.md` and `docs/TRACE.md` to match (feature contract remains authoritative).
