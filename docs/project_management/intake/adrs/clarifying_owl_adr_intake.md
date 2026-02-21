---
codename: clarifying_owl
created: "2026-02-20T01:32:05Z"
status: brainstorming
depends_on: []
execution_order: 80
adr: ADR-0037
adr_path: docs/project_management/adrs/draft/ADR-0037-clarifying-owl.md
workstream_id: WS-clarifying_owl
lockdown_prompt: docs/project_management/system/prompts/discovery/adr_lockdown.md
---

# ADR Intake Sheet

## 1. Codename + Created + Status

- Codename: `clarifying_owl`
- Created: 2026-02-20T01:32:05Z
- Status: brainstorming
- Dependencies: []
- Related intakes (coordination only): `replaying_raccoon`

## 2. Working Title (tentative)

Make doctor/health output explain *why* world isolation is disabled (flag vs env vs config).

## 3. Problem / Motivation

- Current doctor output can say: “world isolation disabled by effective config (--no-world)” even when the actual cause is `world.enabled: false` in `~/.substrate/config.yaml` (or an env override).
- This misleads users into thinking they invoked `--no-world`, when they may be dealing with persisted config or environment overrides.
- It increases support/debug time and makes “enable later” workflows harder to follow.

## 4. Proposed Outcome

- When world is disabled, doctor/health output includes a short, accurate reason:
  - “disabled by CLI flag `--no-world`”
  - “disabled by env override `SUBSTRATE_OVERRIDE_WORLD=disabled`”
  - “disabled by config `$SUBSTRATE_HOME/config.yaml` (`world.enabled: false`)”

## 5. Non-Goals

- Changing whether/when the world is disabled (this is messaging + attribution only).
- Adding new configuration knobs for world enable/disable.

## 6. Constraints / Invariants

- Must remain correct when multiple layers interact (CLI > env > workspace > global).
- Avoid exposing sensitive paths or env values beyond what’s already printed (keep to key names + file path).
- Prefer additive JSON fields over breaking schema (or bump schema if necessary).

## 7. Interfaces / Contracts (concrete changes)

- `substrate host doctor` / `substrate world doctor` text output:
  - replace the generic “(--no-world)” wording with an attribution string.
- JSON output (if needed):
  - add `world_disable_reason` (string/enum) and optionally `world_disable_source` (details).
- `substrate health`:
  - optionally mirror the same reason in its summary when world is disabled.

## 8. Options

### Option 1 — Use config “explain” provenance to compute the disable source

**Description**
Leverage existing config resolution/explain machinery to determine which layer set `world.enabled=false` and surface that in doctor output.

**Pros**
- Most correct; aligns with the actual precedence model.

**Cons**
- Requires plumbing provenance into doctor code paths that currently only have a boolean.

**Risk notes**
- Must ensure explain computation is cheap and doesn’t alter behavior.

### Option 2 — Heuristic attribution (CLI/env/config) without full provenance

**Description**
Compute reason using a simple priority check:
1) if CLI `--no-world` was passed, report that,
2) else if env override is present, report that,
3) else if global/workspace config sets `world.enabled=false`, report config.

**Pros**
- Likely sufficient for most cases; simpler to implement.

**Cons**
- Can be wrong in edge cases where multiple layers set values (unless we still consult resolved layers carefully).

**Risk notes**
- Misattribution is worse than generic messaging; must be validated.

## 9. Recommendation (tentative) + “Choose Option X when…”

Chosen: **Option 2** (heuristic attribution).

Choose **Option 1** when we can reuse existing explain structures cheaply and want strict correctness.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): doctor/health show correct disable attribution.
  - Slice 1: Add a small internal “world enabled source” classifier (CLI/env/config) and update doctor text output.
  - Slice 2: Add additive JSON fields if needed and update `health` to display the same reason.
- ADR Candidate B (follow-up): apply the same attribution to replay warnings and other world-adjacent messaging (see `replaying_raccoon`).

## 11. Acceptance Criteria Draft (observable outcomes)

- With `~/.substrate/config.yaml` containing `world.enabled: false`, `substrate world doctor` prints “disabled by config …”, not “(--no-world)”.
- With `substrate --no-world world doctor`, output attributes disablement to the CLI flag.
- With `SUBSTRATE_OVERRIDE_WORLD=disabled`, output attributes disablement to the environment override.
- No change in actual enable/disable behavior; only messaging changes.

## 12. Open Questions / Unknowns (with priority)

- P0: Do we want to surface the exact config path for attribution, and is workspace config path also desirable?
  - Proposed answer: yes — include the path for whichever config layer is the *highest-precedence* disable source; this is relevant when a workspace patch disables world even though the global config enables it (or vice versa).
- P1: Should attribution appear in JSON only, text only, or both?
  - Proposed answer: both — update text output for humans and add stable JSON fields for tooling.
- P2: Should this also apply to replay warnings and other world-related UX?
  - Proposed answer: yes, but as a follow-up ADR (`replaying_raccoon`) so we can land doctor/health attribution first.

## 13. “Ready to Draft ADR?” checklist (yes/no with reasons)

- [ ] Option is locked (Option 2).
- [ ] Output strings confirmed (exact phrasing + where shown).
- [ ] JSON fields specified (names + enum values + precedence rules).
