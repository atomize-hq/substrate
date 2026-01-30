# Planning Spec Determination Standard (ADR → Required Specs)

This standard inserts a mandatory step between:
- drafting an ADR, and
- running the Planning Pack generation workflow in `docs/project_management/standards/PLANNING_README.md`.

Goal:
- Given an ADR, deterministically derive the **exact set of spec documents** required to make the work execution-ready with **zero ambiguity** about contracts, protocols, schemas, invariants, and environment/config surfaces.

This step exists to prevent a common failure mode:
- the ADR describes the intent, but the Planning Pack fails to explicitly pin down every contract surface (inputs/outputs/schemas/env vars/error rules), leaving “implied” behavior that later becomes drift.

---

## When to run this step

Run this immediately after an ADR is drafted (status can remain `Draft`), before the Planning Pack is produced.

Recommended ordering:
1) Draft ADR (using `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`).
2) Run this step to produce `spec_manifest.md`.
3) Update the ADR `Related Docs` section to include `spec_manifest.md` and the selected spec docs.
4) Run the impact map step to produce `impact_map.md`:
   - `docs/project_management/standards/PLANNING_IMPACT_MAP_STANDARD.md`
5) Iterate the ADR until it is complete and internally consistent.
6) Accept the ADR (status `Accepted`) only after `spec_manifest.md` and `impact_map.md` are complete and the ADR no longer implies undocumented behavior.
7) Run `docs/project_management/standards/PLANNING_README.md` to produce the remaining Planning Pack artifacts.

---

## Required output (new artifact)

Create:
- `docs/project_management/next/<feature>/spec_manifest.md`

This file is authoritative for:
- which spec documents exist for the feature, and
- which contract surfaces each document owns.

Scaffolding:
- `make planning-new-feature FEATURE=<feature>` creates `spec_manifest.md` from `docs/project_management/standards/templates/spec_manifest.md.tmpl`.

---

## Rules (non-negotiable)

1) `spec_manifest.md` must leave no implicit surfaces.
   - Every externally visible surface (CLI/config/env vars/files/protocols/schemas/log/trace fields) must appear in the coverage matrix and be assigned to an authoritative document.

2) Each surface has exactly one authoritative doc.
   - Other docs may link, but must not define conflicting contract text.

3) Absence semantics are mandatory.
   - If a field/flag/env var/config entry may be missing, the behavior when absent must be explicit and testable.

4) Determinism beats “flexibility”.
   - If the ADR permits multiple behaviors, collapse it into a single deterministic contract or elevate it to a Decision Register entry (exactly two options A/B and one selection).

---

## Spec artifact catalog (the chooser)

`spec_manifest.md` is produced by selecting from this catalog. The selection must be based on the ADR, not on preference.

Always create:
- `plan.md` (overall execution runbook)
- `tasks.json` (triad tasks)
- One or more slice specs: `<SLICE_ID>-spec*.md` (and later slices as needed)

Create `contract.md` when the ADR defines or changes any user-facing contract:
- CLI commands/flags/defaults
- Config file paths/precedence/schema constraints
- Exit code meanings
- Path semantics / protected paths
Standard: `docs/project_management/standards/CONTRACT_SURFACE_STANDARD.md`

Create additional, topic-focused specs when the ADR introduces or changes any of the following:

1) **Wire/API protocol surface**
   - Trigger: any host↔agent RPC, HTTP endpoint, WebSocket event, named pipe protocol, IPC framing, or shim↔shell metadata contract.
   - Create: `<topic>-protocol-spec.md`
   - Template: `docs/project_management/standards/templates/spec/protocol-spec.md.tmpl`
   - Must define: request/response schema, error model, versioning strategy, timeouts/retries, streaming framing (if any), ordering rules, and auth/identity model (if any).

2) **Data schema / file format surface**
   - Trigger: any JSON/YAML/TOML schema, on-disk format, or serialization boundary that must be stable.
   - Create: `<topic>-schema-spec.md`
   - Template: `docs/project_management/standards/templates/spec/schema-spec.md.tmpl`
   - Must define: full schema, constraints, canonicalization rules, defaults, forward/backward handling policy (explicit), and sample payloads.

3) **Environment variables**
   - Trigger: any new `SUBSTRATE_*`, `SHIM_*`, or `WORLD_*` variable, or changes to existing semantics.
   - Create: `env-vars-spec.md` (or fold into `contract.md` if strictly user-facing and small)
   - Template: `docs/project_management/standards/templates/spec/env-vars-spec.md.tmpl`
   - Must define: name, type, allowed values, default, precedence vs config/flags, and security/redaction implications.

4) **Policy surface**
   - Trigger: policy broker YAML schema changes, new policy decisions, allow/deny/isolation semantics, approval caching changes, or enforcement mode shifts.
   - Create: `policy-spec.md`
   - Template: `docs/project_management/standards/templates/spec/policy-spec.md.tmpl`
   - Must define: decision inputs, evaluation rules, precedence, default posture, and audit/log output expectations.

5) **Telemetry / observability surface**
   - Trigger: new trace span fields, log schema changes, new metrics, or new redaction behavior.
   - Create: `telemetry-spec.md`
   - Template: `docs/project_management/standards/templates/spec/telemetry-spec.md.tmpl`
   - Must define: field names, types, redaction rules, stability guarantees, and consumer impact.

6) **Filesystem semantics**
   - Trigger: overlays, protected paths, path rewriting, mount/layout semantics, diff collection rules, or any “what files do we touch” contract.
   - Create: `filesystem-semantics-spec.md`
   - Template: `docs/project_management/standards/templates/spec/filesystem-semantics-spec.md.tmpl`
   - Must define: path invariants, resolution rules, failure modes, and platform differences (if any).

7) **Platform parity / divergence**
   - Trigger: any platform-specific behavior, OS integration, or backend differences that must be tracked as a contract.
   - Create: `platform-parity-spec.md`
   - Template: `docs/project_management/standards/templates/spec/platform-parity-spec.md.tmpl`
   - Must define: per-platform guarantees, allowed divergences, and required validation evidence.

8) **Rollout / compatibility**
   - Trigger: the ADR explicitly mandates migrations, backwards compatibility, deprecation, or staged rollout.
   - Create: `compatibility-spec.md`
   - Template: `docs/project_management/standards/templates/spec/compatibility-spec.md.tmpl`
   - Must define: compat policy, end condition for compat removal, and test/validation evidence.

---

## Required structure for `spec_manifest.md`

Use the template:
- `docs/project_management/standards/templates/spec_manifest.md.tmpl`

When creating domain-specific spec documents selected by `spec_manifest.md`, use the corresponding templates:
- `docs/project_management/standards/templates/spec/`

`spec_manifest.md` must include:
1) ADR inputs (paths) and the feature directory.
2) A concrete list of the exact spec docs to create (with canonical filenames).
3) A coverage matrix mapping each surface to its authoritative doc.
4) A determinism checklist for each selected spec doc (what it must explicitly define).

---

## Prompt (copy/paste)

Use this prompt to generate `spec_manifest.md` (and update the ADR’s `Related Docs` list).

```md
You are the Spec Determination agent for <FEATURE>.

Goal:
- Read the ADR(s) for <FEATURE>.
- Produce `docs/project_management/next/<feature>/spec_manifest.md` that deterministically selects the exact spec documents required for this body of work.
- Ensure every contract surface is explicitly owned by exactly one authoritative document.

Constraints (non-negotiable):
- Do not write production code.
- Do not invent new scope; derive artifacts from the ADR and its stated goals/contract.
- No ambiguous wording in normative statements (every behavior statement must be singular and testable).
- No implied surfaces: every protocol/schema/env var/path/exit-code/log-field touched by the ADR must be enumerated and assigned to a doc.

Required reading:
- `docs/project_management/standards/PLANNING_SPEC_DETERMINATION_STANDARD.md`
- `docs/project_management/standards/ADR_STANDARD_AND_TEMPLATE.md`
- `docs/project_management/standards/CONTRACT_SURFACE_STANDARD.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

Inputs:
- ADR(s): <list exact paths>
- Feature directory: `docs/project_management/next/<feature>/`

Output requirements:
1) Write/overwrite: `docs/project_management/next/<feature>/spec_manifest.md` using the template structure.
2) In `spec_manifest.md`, include:
   - The exact list of required spec docs (filenames under the feature dir).
   - A coverage matrix mapping every surface to an authoritative doc.
   - For each required spec doc, list the deterministic items it must define (schemas, defaults, precedence, error rules, invariants).
3) Update the ADR `Related Docs` section to include:
   - `spec_manifest.md`
   - the selected spec docs
4) If the ADR implies a surface but does not define it precisely, record a Decision Register entry (A/B) and update the ADR to reflect the chosen option (do not leave it unpinned).
```
