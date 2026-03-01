**PRE‑PLANNING ONLY — This document is a temporary alignment backbone and MUST be deleted or retired during full planning.**

Derived from (authoritative inputs):
- `docs/project_management/adrs/draft/ADR-0035-summoning-wombat.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/spec_manifest.md`
- `docs/project_management/packs/draft/dev-install-world-agent-staging/pre-planning/impact_map.md`

## Scope + authority

This draft MAY define:
- Cross-cutting defaults, precedence, and invariants that multiple slice specs MUST align on.
- Cross-cutting constraints that keep this feature compatible with overlapping queued work identified by the impact map.

This draft MUST NOT define:
- Slice-specific behavior details, acceptance criteria IDs, or implementation tasks.
- Detailed schemas beyond naming the canonical files/keys already surfaced by the ADR/spec-manifest.

## Defaults + precedence

### Substrate home + config file (source of truth)
- Persisted state is stored at `$SUBSTRATE_HOME/config.yaml` (default: `~/.substrate/config.yaml`).
- For `substrate world enable`, the effective `$SUBSTRATE_HOME` precedence order is:
  1. CLI flag: `substrate world enable --home <path>`
  2. Environment: `SUBSTRATE_HOME`
  3. Default: `~/.substrate`

### `world.enabled` ordering invariant
- `world.enabled` MUST remain `false` after `scripts/substrate/dev-install-substrate.sh --no-world`.
- `world.enabled` MUST be written as `true` only after `substrate world enable` provisioning verification succeeds.

### Version-dir layout (cross-cutting path seam)
- The helper’s inferred “version dir” / `RELEASE_ROOT` resolution is the source of truth for where enable/provisioning expects staged artifacts.
- Dev-install staging and enable preflight MUST agree on the staged `world-agent` path set; `contract.md` MUST pin the exact rule.

## Failure posture + invariants

### Fail-closed posture (missing staged artifact)
- On Linux, if the staged `world-agent` artifact is missing, `substrate world enable` MUST fail before any privileged/systemd/socket/readiness actions.
- The failure MUST emit exactly one actionable remediation message; `contract.md` MUST pin minimum remediation content and the exact exit code.

### Security invariants
- No `cargo build` invocation runs under `sudo` for this feature’s workflows.
- This feature MUST NOT add or modify host↔agent protocol surfaces.
- This feature MUST NOT add or modify structured log schema fields or trace span fields.

### Redaction posture
- This feature MUST NOT introduce new logging of secrets; any new messaging must remain free of credential material by construction.

## Exit-code posture

- Exit code taxonomy: `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This feature requires no new exit codes; `contract.md` MUST map all externally visible failures (including missing staged `world-agent` and unsupported platforms) to taxonomy codes deterministically.

## Cross-cutting seams / constraints

- Platform guardrail: this feature’s behavior delta is Linux-only; macOS is no-change; Windows `substrate world enable` remains unsupported.
- Preflight orthogonality: missing-`world-agent` preflight MUST remain independent of any `world enable` dependency-provisioning flags so overlapping queued work can evolve without breaking the remediation contract.
- Change minimization: avoid broad refactors in shared installer helpers; constrain edits to the touched surfaces enumerated by the impact map.
- Slice IDs are canonical and MUST remain stable for this feature: `DIWAS0`, `DIWAS1`.

## Follow-ups for full planning

- ADR coherence:
  - Reconcile ADR-0035 internal option label inconsistency and ensure the selected option is unambiguous.
  - Update ADR-0035 Related Docs paths to the canonical pre-planning locations used by this pack (`pre-planning/spec_manifest.md`, `pre-planning/impact_map.md`).
- Decision Register requirements:
  - DR-0001: pin preflight implementation locus (Rust runner vs helper script vs installer helper).
  - DR-0002: pin deterministic meaning of `scripts/substrate/dev-install-substrate.sh --no-world`.
  - DR-0003: pin `world-agent` staging profile mapping (release-only vs match `dev-install --profile`).
  - DR-0004: pin overwrite/idempotency rules for staged `bin/(linux/)world-agent` paths.
- Contract determinism:
  - Pin the exact staged artifact path rule (require BOTH `bin/world-agent` and `bin/linux/world-agent`, or accept ONE).
  - Pin `--dry-run` semantics for missing staged artifact (exit code + messaging + “no privileged actions” proof).
  - Pin exit code for missing staged artifact and for unsupported platform(s), taxonomy-aligned.
- Pack hygiene:
  - Reconcile `tasks.json` platform requirements with Linux-only behavior change (narrow to Linux or add deterministic no-change/unsupported validations).

## Draft slice skeleton (pre-planning only)

Draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): DIWAS

- slice_id: DIWAS0
  - name: Fail fast on missing staged `world-agent`
  - intent: Stabilize a deterministic enable preflight that detects missing staged artifacts and fails before any privileged provisioning actions, with one actionable remediation.
  - likely touch surfaces:
    - `scripts/substrate/world-enable.sh`
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/shell/tests/world_enable.rs`

- slice_id: DIWAS1
  - name: Stage `world-agent` during dev-install `--no-world`
  - intent: Stabilize the dev-install staging layout so “install with `--no-world`, enable later” works on Linux without manual build steps and remains coherent with enable/provisioning expectations.
  - likely touch surfaces:
    - `scripts/substrate/dev-install-substrate.sh`
    - `scripts/substrate/install-substrate.sh`
    - `tests/installers/install_smoke.sh`

Note (downstream pre-planning):
- CI-checkpoint planning MUST prefer this slice list when populating the machine-readable slice list in `pre-planning/ci_checkpoint_plan.md` (do not mechanically validate until slice tasks exist in `tasks.json`).
- Workstream triage MAY propose edits to this slice skeleton as recommendations in `pre-planning/workstream_triage.md` (do not edit this file from triage).
