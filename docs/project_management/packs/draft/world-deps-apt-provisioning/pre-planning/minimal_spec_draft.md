**PRE‑PLANNING ONLY: This document is not execution-ready and MUST be deleted or retired during full planning.**

# world-deps-apt-provisioning — minimal spec draft (alignment backbone)

Authority inputs:
- ADR: `docs/project_management/adrs/draft/ADR-0030-provisioning-otter.md`
- Spec manifest (doc ownership): `docs/project_management/packs/draft/world-deps-apt-provisioning/spec_manifest.md`
- Impact map (touch set + conflicts): `docs/project_management/packs/draft/world-deps-apt-provisioning/impact_map.md`

## Scope + authority

This draft is allowed to define:
- Cross-cutting invariants that ALL slice specs and `contract.md` MUST align to.
- Default posture statements (supported/unsupported matrix posture, failure posture, exit-code taxonomy posture).
- Input precedence only when a surface is shared across multiple specs.

This draft MUST NOT define:
- Slice-specific behavior details (exact APT invocation, exact output lines, exact probe/state algorithm).
- Schemas/file formats (including any provisioned-state tracking artifact schemas).
- Implementation tasks, code structure, or file-by-file work plans.

## Defaults + precedence

### Input precedence (this feature)
- Provisioning behavior is controlled by CLI flags on `substrate world enable` (including `--provision-deps`, plus any flags explicitly enumerated by `contract.md`).
- This feature introduces no new config keys and no new env vars (per `spec_manifest.md`).
- Precedence therefore reduces to: CLI flags override built-in defaults.

### Source-of-truth references (this feature)
- World-deps inventory schema and enabled-resolution model are authoritative in `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` (linked; not redefined here).
- The operator-facing contract introduced/changed by ADR-0030 is authoritative in `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` (to be created during planning).

## Failure posture + invariants

Failure posture (cross-cutting):
- Runtime APT posture is fail-closed: when APT-backed items are in-scope, runtime commands exit non-zero with remediation and perform no OS mutation.
- Provisioning posture is fail-closed: on unsupported backends or unmet prerequisites, provisioning exits non-zero and performs no OS mutation.

Runtime posture:
- `substrate world deps current sync|install` MUST NOT execute `apt-get`, `apt`, or `dpkg` under any backend/hardening posture.
- If APT-backed world-deps are in-scope for the runtime command, the command MUST fail early with an actionable remediation that includes the exact command string `substrate world enable --provision-deps`.

Provisioning posture:
- Provisioning-time OS mutation (APT/dpkg) MUST be operator-invoked via the selected entrypoint (`substrate world enable --provision-deps`) and MUST NOT be performed implicitly during runtime sync/install.
- Provisioning MUST be supported only on guest-world backends where OS mutation is in-scope.
- Provisioning MUST be rejected on Linux host-native backends with messaging that explicitly states “no host OS mutation”.

Security / redaction invariants:
- Operator-facing outputs and logs MUST NOT emit credentials or secret material (including any APT auth material if surfaced by the underlying OS/tooling).
- Provisioning guard rails (request-profile/isolation model) MUST be explicit and auditable (owned by `decision_register.md` DR-0003 and the slice specs).

## Exit-code posture

- Exit code taxonomy (canonical): `docs/project_management/system/standards/shared/EXIT_CODE_TAXONOMY.md`
- This work requires no new exit codes.
- Exit code mapping for provisioning + runtime short-circuit paths MUST use taxonomy meanings and MUST be defined in `contract.md` (ADR-0030 indicates a 0/3/4/5 subset).

## Cross-cutting seams / constraints

These constraints apply across `contract.md`, `WDAP0` spec, and `WDAP1` spec:
- Remediation text invariant: runtime fail-early messaging MUST include the exact command string `substrate world enable --provision-deps`.
- Failure-class determinism: unsupported provisioning vs world backend unavailable vs missing prerequisites MUST be distinguishable by exit code and minimum remediation content (owned by `contract.md`).
- Exit-code partitioning: exit code `4` MUST represent unsupported/unmet-prereq paths for this feature; exit code `5` MUST remain reserved for safety/policy violations (exact per-command mapping is owned by `contract.md`).
- `world enable` UX determinism: when both “world backend provisioning” and “deps provisioning” are triggered, output MUST distinguish them (especially under `--dry-run` and `--verbose`).
- `world enable` base-contract alignment: `--provision-deps` MUST be an additive flag surface and MUST honor the existing `world enable` home/paths contract (impact-map overlap with ADR-0003).
- CLI entrypoint stability: `substrate world enable --provision-deps` is the canonical provisioning entrypoint; `world deps provision` MUST NOT be introduced unless it is explicitly retained as an alias with a documented compat posture (impact-map overlap with ADR-0009 and archived references).
- Contract ownership boundary (cross-pack): `docs/project_management/packs/draft/world-deps-apt-provisioning/contract.md` is authoritative for the shared CLI/exit-code/remediation surface introduced by ADR-0030; non-APT system-package provisioning work MUST link/extend rather than redefine the base surface (impact-map selection: Option B).
- External contract coherence: `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` MUST NOT retain runtime APT “apply apt first” semantics that contradict the ADR-0030 runtime prohibition; ownership must be reconciled during planning/implementation.

## Follow-ups for full planning

1) Define runtime fail-early scope (single rule; testable):
   - Decide whether runtime short-circuit triggers based on the effective enabled set, explicitly requested items, or a union; define behavior for `--all`.

2) Define provisioning APT invocation contract (single deterministic contract):
   - Pin exact APT commands/flags (update/install), non-interactive posture, ordering + de-dup, retries/timeouts, and error→exit mapping.

3) Define `--verbose` output invariants:
   - Specify what additional information is guaranteed vs omitted, and which stream(s) are used.

4) Decide provisioned-state tracking strategy (DR-0002):
   - Select probe-only vs persisted state file; if persisted, define exact path, schema, ownership (host-side vs world-side), and absence semantics (and assign schema ownership in `spec_manifest.md`).

5) Make Linux host-native “unsupported by default” deterministic:
   - Decide whether an override exists; if it exists, define its name/type/default/guard rails in `contract.md` and the relevant slice spec(s).

6) Make Windows posture deterministic:
   - `substrate world enable` support on Windows is currently not guaranteed; define the v1 operator contract for `--provision-deps` on Windows/WSL (supported vs unsupported) and the required remediation path.

7) Enumerate operator-doc update targets by exact path:
   - Replace `docs/reference/world/deps/…` placeholders with exact file paths/headings and ensure they link back to `contract.md`.

8) Reconcile ADR-0030 “Related Docs” path drift for world-deps schema contract:
   - Update downstream planning docs to reference `docs/project_management/packs/implemented/world-deps-packages-bundles-contract/contract.md` as the schema authority unless/until that contract is relocated.

9) Define provisioning execution profile isolation model (DR-0003):
   - Define how the provisioning request profile is selected, what it relaxes, and what guard rails prevent misuse under hardened runtime assumptions.

## Draft slice skeleton (pre-planning only)

Disclaimer: draft; may split/merge; do not wire `tasks.json` yet.

Slice prefix (draft): WDAP

Downstream notes:
- CI-checkpoint prefers this slice list when populating the machine-readable slices list in `ci_checkpoint_plan.md` (do not validate mechanically until slice tasks exist in `tasks.json`).
- Workstream triage can propose edits to this slice skeleton as recommendations in `workstream_triage.md` (but must not edit this file).

### Slice entries (draft)

- slice_id: WDAP0
  name: Provision APT requirements
  intent: Stabilize the provisioning-time APT/system package install surface for the effective enabled world-deps set on supported guest backends, including `--dry-run` and idempotency definition.
  likely touch surfaces:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP0/WDAP0-spec.md`
    - `crates/shell/src/builtins/world_enable/runner.rs`
    - `crates/world-agent/src/service.rs`
    - `scripts/substrate/world-enable.sh`

- slice_id: WDAP1
  name: Runtime fail-early for APT items
  intent: Stabilize the runtime prohibition (“no APT/dpkg at runtime”) by enforcing an early-exit posture with deterministic exit code + remediation when APT-backed items are in-scope.
  likely touch surfaces:
    - `docs/project_management/packs/draft/world-deps-apt-provisioning/slices/WDAP1/WDAP1-spec.md`
    - `crates/shell/src/builtins/world_deps/surfaces.rs`
    - `crates/shell/tests/world_deps_apt_install_wdp5.rs`
