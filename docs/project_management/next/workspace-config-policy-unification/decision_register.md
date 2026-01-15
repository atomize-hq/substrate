# Decision Register — Workspace Config/Policy Unification (ADR-0008)

This decision register records the architectural decisions for `ADR-0008`.

Rules:
- Each decision is recorded as exactly two options (A/B), with explicit tradeoffs and one selection.
- No backwards compatibility or migration behavior is included unless explicitly stated.

---

## DR-0001 — Workspace/global files are patches (sparse) vs full documents

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU2-spec.md`, `docs/project_management/next/workspace-config-policy-unification/WCU3-spec.md`

**Problem / Context**
- Global and workspace scope files require a representation where “unset” means “inherit”, and `reset` produces auditable on-disk intent.

### Option A (selected): Patch files (sparse YAML mappings)
- **Decision:** Treat both global and workspace `config.yaml`/`workspace.yaml` and `policy.yaml` as **patch files**:
  - Missing keys mean “inherit from the next lower precedence layer”.
  - Patch files may be empty mappings (`{}`).
- **Pros:**
  - Enables correct `reset <key>` and `reset` semantics (workspace reset can mean “stop overriding”, not “copy global values”).
  - Makes it easy to keep workspace files minimal and auditable.
  - Avoids “template drift” pressure because the canonical full effective view is produced by `current show`.
- **Cons:**
  - Requires a merge layer and patch-aware validators for both config and policy.
  - Requires a new `--explain` source/provenance view to keep the UX transparent.
- **Cascading implications:**
  - Patch parsers validate sparse mappings and enforce the schema allowlist.
  - `current show --explain` exists as the deterministic way to debug layer contributions.
- **Risks:**
  - Operators misinterpret `{}` as “not configured” without the patch-view notes and playbook coverage.
- **Unlocks:**
  - Inherit-only reset semantics across scopes without copying values.
- **Quick wins / low-hanging fruit:**
  - Empty patch initialization (`{}`) is a stable baseline for new installs and workspaces.

### Option B: Full documents only (no sparse keys)
- **Pros:**
  - Simpler parsing and validation (deserialize directly into the full structs).
- **Cons:**
  - Makes “unset/reset a single key to inherit” impossible without copying values.
  - Encourages drift and confusion because the workspace file becomes a full shadow of global.
- **Cascading implications:**
  - Requires a separate representation for “inherit” and “explicitly empty” values.
- **Risks:**
  - `reset` semantics become ambiguous and non-auditable because the file always contains a full copy.
- **Unlocks:**
  - Faster initial parsing code, followed by a more complex contract for inheritance.
- **Quick wins / low-hanging fruit:**
  - Minimal short-term implementation, followed by increased long-term maintenance.

**Recommendation**
- **Selected:** Option A — Patch files (sparse YAML mappings)
- **Rationale (crisp):** Patch files encode inheritance and reset semantics explicitly while keeping scope files minimal.

**Follow-up tasks (explicit)**
- Patch parsing + merge plumbing: `WCU2-code`, `WCU2-test`, `WCU2-integ`
- Scope CLIs + patch editor surfaces: `WCU3-code`, `WCU3-test`, `WCU3-integ`
- Validation parity (playbook + smoke): `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0002 — `current show` provenance visibility

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/next/workspace-config-policy-unification/WCU2-spec.md`

**Problem / Context**
- Effective views require provenance without breaking machine-parseable stdout for automation and scripts.

### Option A (selected): Quiet by default + explicit `--explain`
- **Decision:** `current show` prints the effective/merged config or policy on stdout and prints a single stderr notice indicating it is merged; `--explain` adds a per-key provenance breakdown.
- **Pros:**
  - Keeps default output machine- and human-friendly.
  - Makes it easy to debug “why is this value set?” without permanently noisy output.
- **Cons:**
  - Requires users to learn `--explain` to get deeper context.
- **Cascading implications:**
  - `--explain` emits a deterministic, machine-readable payload to stderr (ADR-0012).
- **Risks:**
  - Operators miss the stderr notice and assume patch views and effective views are identical.
- **Unlocks:**
  - Golden-testable provenance output and multi-source keys without polluting stdout.
- **Quick wins / low-hanging fruit:**
  - A single stderr notice disambiguates effective output without changing stdout.

### Option B: Always print provenance
- **Pros:**
  - Maximum transparency with no extra flags.
- **Cons:**
  - Noisy; complicates piping/parsing; increases incidental confusion for basic usage.
- **Cascading implications:**
  - stdout is no longer the payload; consumers must filter and parse mixed output.
- **Risks:**
  - Breaks common piping patterns and increases test fragility.
- **Unlocks:**
  - No additional capabilities beyond always-on verbosity.
- **Quick wins / low-hanging fruit:**
  - No quick wins; the default output becomes permanently noisy.

**Recommendation**
- **Selected:** Option A — Quiet by default + explicit `--explain`
- **Rationale (crisp):** Provenance is available on demand while stdout remains the stable payload contract.

**Follow-up tasks (explicit)**
- Implement deterministic provenance output (including multi-source keys): `WCU2-code`, `WCU2-test`, `WCU2-integ`
- Validate provenance behavior via playbook and smoke: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0003 — CLI surface: explicit scopes vs legacy aliases

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU3-spec.md`

**Problem / Context**
- Config and policy require explicit naming that separates “effective merged view” from “scope patch view”.

### Option A (selected): Explicit scopes; no legacy aliases
- **Decision:** Require explicit `current|global|workspace` scopes; do not keep legacy `config show` / `config set` / `policy show` / `policy set` as aliases.
- **Pros:**
  - Eliminates ambiguity between “effective view” vs “scope file view”.
  - Aligns policy and config CLI symmetry.
- **Cons:**
  - Breaking change for existing usage (explicitly accepted as greenfield).
- **Cascading implications:**
  - All docs, smoke scripts, and playbooks use only `current|global|workspace` spellings.
- **Risks:**
  - Operators relying on legacy spelling hit exit `2` usage errors until updated.
- **Unlocks:**
  - A single, unambiguous contract across config and policy.
- **Quick wins / low-hanging fruit:**
  - Avoids maintaining alias logic and a larger testing matrix.

### Option B: Keep aliases for older commands
- **Pros:**
  - Lower transition cost for existing users.
- **Cons:**
  - Preserves ambiguity and increases documentation/testing complexity.
- **Cascading implications:**
  - Requires alias mapping and continued support for ambiguous UX patterns.
- **Risks:**
  - Preserves the exact confusion motivating ADR-0008.
- **Unlocks:**
  - None beyond legacy spelling compatibility.
- **Quick wins / low-hanging fruit:**
  - Faster for a subset of users at the cost of ongoing ambiguity and maintenance.

**Recommendation**
- **Selected:** Option A — Explicit scopes; no legacy aliases
- **Rationale (crisp):** Explicit scopes eliminate ambiguity and keep config and policy symmetrical.

**Follow-up tasks (explicit)**
- Implement explicit scope commands: `WCU3-code`, `WCU3-test`, `WCU3-integ`
- Validate command spellings in smoke/playbook: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0004 — Workspace disable persistence mechanism

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md`

**Problem / Context**
- Workspace discovery and effective resolution require a persistent, deterministic “disabled” state that survives shell restarts.

### Option A (selected): Persistent marker file
- **Decision:** Implement workspace disable/enable via a marker file at `<workspace_root>/.substrate/workspace.disabled`.
- **Pros:**
  - Deterministic; survives shell restarts; does not depend on parent shell env mutation.
- **Cons:**
  - Adds one more file under `.substrate/`.
- **Cascading implications:**
  - Workspace root discovery includes a marker check and ignores disabled workspaces.
- **Risks:**
  - Workspace remains disabled until explicitly enabled again.
- **Unlocks:**
  - Reliable “ignore workspace overrides” behavior for debugging and triage.
- **Quick wins / low-hanging fruit:**
  - Idempotent operations: create marker to disable, remove marker to enable.

### Option B: Environment variable / session-only toggle
- **Pros:**
  - No on-disk state.
- **Cons:**
  - Hard to apply consistently across tooling and shells; not durable.
- **Cascading implications:**
  - Requires propagating environment variables across all invocation paths.
- **Risks:**
  - Behavior becomes session-local and non-reproducible.
- **Unlocks:**
  - Avoids one marker file at the cost of determinism.
- **Quick wins / low-hanging fruit:**
  - Faster to prototype, not stable for an operator-facing contract.

**Recommendation**
- **Selected:** Option A — Persistent marker file
- **Rationale (crisp):** A marker file provides durable, deterministic state aligned with patch-file semantics.

**Follow-up tasks (explicit)**
- Implement disable/enable marker semantics and tests: `WCU1-code`, `WCU1-test`, `WCU1-integ`
- Validate disabled marker behavior in smoke/playbook: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0005 — Workspace internal git location

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md`

**Problem / Context**
- Workspace state requires a single canonical root for protected-path logic and operator mental model.

### Option A (selected): Internal git lives under `.substrate/`
- **Decision:** Store internal git at `<workspace_root>/.substrate/git/repo.git/` (replaces `<workspace_root>/.substrate-git/repo.git/`).
- **Pros:**
  - One canonical workspace state directory: `.substrate/`.
  - Cleaner sync exclusions (a single protected root).
- **Cons:**
  - Requires updating docs/specs/tests that reference `.substrate-git`.
- **Cascading implications:**
  - Workspace init/reset/remove operate under `.substrate/` and never touch the user repo `.git/`.
- **Risks:**
  - Existing legacy layouts are ignored under the greenfield posture and require manual correction.
- **Unlocks:**
  - A single protected root: `.substrate/**`.
- **Quick wins / low-hanging fruit:**
  - Simplifies sync excludes and reduces protected-path surface area.

### Option B: Keep `.substrate-git/` at workspace root
- **Pros:**
  - Matches existing docs/specs.
- **Cons:**
  - Splits workspace state across directories; hurts onboarding and increases protected-path surface.
- **Cascading implications:**
  - Protected-path logic and docs must handle multiple canonical roots.
- **Risks:**
  - Preserves the state split motivating ADR-0008.
- **Unlocks:**
  - Legacy continuity only.
- **Quick wins / low-hanging fruit:**
  - No quick wins beyond avoiding reference updates; long-term complexity remains.

**Recommendation**
- **Selected:** Option A — Internal git lives under `.substrate/`
- **Rationale (crisp):** A single canonical workspace root reduces protected-path surface area and simplifies the operator mental model.

**Follow-up tasks (explicit)**
- Implement internal git relocation and canonical `.substrate/` layout: `WCU1-code`, `WCU1-test`, `WCU1-integ`
- Validate layout via smoke/playbook: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0006 — Workspace init example templates

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md`

**Problem / Context**
- Operators need optional templates without increasing the default workspace footprint or creating template drift pressure.

### Option A (selected): Examples behind a flag
- **Decision:** `substrate workspace init --examples` creates both:
  - `<workspace_root>/.substrate/workspace.example.yaml`
  - `<workspace_root>/.substrate/policy.example.yaml`
  The default `workspace init` does not create example files.
- **Pros:**
  - Keeps default workspace footprint minimal and avoids stale template drift.
  - Still supports “show me a full file” when explicitly requested.
- **Cons:**
  - Users must discover `--examples` when they want templates.
- **Cascading implications:**
  - Example files are explicitly non-active and are never read by Substrate for behavior.
- **Risks:**
  - Users edit example files and expect them to apply.
- **Unlocks:**
  - Optional onboarding artifacts without changing config/policy resolution semantics.
- **Quick wins / low-hanging fruit:**
  - Creates templates without requiring a separate command surface.

### Option B: Always create example files
- **Pros:**
  - Templates always available.
- **Cons:**
  - Increases workspace clutter; encourages hand-editing and drift.
- **Cascading implications:**
  - Templates require maintenance, validation, and ongoing parity work.
- **Risks:**
  - Templates drift and create confusion about which file is active.
- **Unlocks:**
  - Faster access to templates at the cost of default workspace clutter.
- **Quick wins / low-hanging fruit:**
  - No quick wins; ongoing maintenance cost starts immediately.

**Recommendation**
- **Selected:** Option A — Examples behind a flag
- **Rationale (crisp):** Templates remain available without expanding default workspace state or introducing continuous template churn.

**Follow-up tasks (explicit)**
- Implement `workspace init --examples` and ensure examples are never read: `WCU1-code`, `WCU1-test`, `WCU1-integ`
- Validate `--examples` file effects and non-active behavior via playbook and smoke: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0007 — Override env vars are one-off; installers must not export them

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU4-spec.md`

**Problem / Context**
- Installer/dev scripts exporting override inputs by default breaks the scope model and creates hidden precedence state.

### Option A (selected): Overrides are explicit one-offs only
- **Decision:** `SUBSTRATE_OVERRIDE_*` is treated as explicit one-off operator input and MUST NOT be exported by install/dev env scripts by default.
- **Pros:**
  - Prevents “global set did nothing” confusion caused by persistent override exports.
  - Restores the mental model: global is baseline unless workspace overrides exist.
- **Cons:**
  - Requires updating install/dev scripts and docs.
- **Cascading implications:**
  - Overrides remain supported as explicit operator inputs and are not required for correctness.
- **Risks:**
  - Existing workflows relying on default exports must switch to patch files or explicit overrides.
- **Unlocks:**
  - Predictable `current show` behavior without hidden defaults.
- **Quick wins / low-hanging fruit:**
  - Removing default exports immediately reduces operator confusion in new installs.

### Option B: Allow installers to export overrides
- **Pros:**
  - Easy for installers to force desired behavior without config edits.
- **Cons:**
  - Reintroduces precedence confusion and breaks parity between config and policy.
- **Cascading implications:**
  - Requires documentation and support for environment-export behavior that overrides patch files.
- **Risks:**
  - Patch-file edits appear ineffective under common workflows.
- **Unlocks:**
  - Short-term installer convenience only.
- **Quick wins / low-hanging fruit:**
  - No quick wins aligned with the contract goals; ambiguity becomes the default state.

**Recommendation**
- **Selected:** Option A — Overrides are explicit one-offs only
- **Rationale (crisp):** Default override exports violate the scope model and are a direct source of operator confusion.

**Follow-up tasks (explicit)**
- Update install/dev scripts and validate no default override exports: `WCU4-code`, `WCU4-test`, `WCU4-integ`

---

## DR-0008 — Legacy directory layouts and marker names

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU1-spec.md`

**Problem / Context**
- This body of work is explicitly greenfield and does not include migrations, warnings, or backwards-compat behavior for legacy layouts.

### Option A (selected): Single canonical `.substrate/` directory; ignore legacy silently
- **Decision:** Only `.substrate/` is recognized as the workspace state directory. Legacy alternative layouts (e.g., `.substrate-profile*`, `.substrate-git/`) are not recognized and produce no special warnings or migrations.
- **Pros:**
  - Clean onboarding and deterministic rules.
  - Keeps implementation smaller and avoids migration logic.
- **Cons:**
  - Existing users with legacy layouts must self-correct without tool guidance.
- **Cascading implications:**
  - Workspace discovery checks only for the canonical marker `.substrate/workspace.yaml`.
- **Risks:**
  - Users with legacy layouts experience “no workspace found” until corrected.
- **Unlocks:**
  - A single canonical layout for downstream contracts and validation artifacts.
- **Quick wins / low-hanging fruit:**
  - Avoids designing and validating migration behavior.

### Option B: Detection + migration + warnings for legacy layouts
- **Pros:**
  - Better transitional UX for existing users.
- **Cons:**
  - Non-greenfield complexity; requires migration plans and ongoing compat tests.
- **Cascading implications:**
  - Requires a migration policy and on-disk transformation rules.
- **Risks:**
  - Partial migrations create mixed states and hard-to-debug failures.
- **Unlocks:**
  - Transitional UX outside this pack’s greenfield constraints.
- **Quick wins / low-hanging fruit:**
  - No quick wins without committing to a migration contract and test matrix.

**Recommendation**
- **Selected:** Option A — Single canonical `.substrate/` directory; ignore legacy silently
- **Rationale (crisp):** Greenfield posture requires a single canonical layout without migration behavior.

**Follow-up tasks (explicit)**
- Implement canonical discovery and layout with no legacy compatibility logic: `WCU1-code`, `WCU1-test`, `WCU1-integ`

---

## DR-0009 — Patch file comment headers (clarity) vs bare YAML mappings

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU3-spec.md`

**Problem / Context**
- Patch files require an on-disk affordance that communicates “this file is an override patch” and points to the effective merged view.

### Option A (selected): Add short comment headers to patch files
- **Decision:** When Substrate creates `config.yaml`, `policy.yaml`, `.substrate/workspace.yaml`, or `.substrate/policy.yaml`, it writes a short comment header explaining:
  - that the file is a sparse override patch at this scope,
  - that the file can be edited directly or via CLI (CLI recommended),
  - and how to inspect the effective merged view via `current show --explain`.
- **Pros:**
  - Makes patch semantics self-documenting and reduces “why didn’t this take effect?” confusion.
  - Encourages safe workflows (`current show --explain` for debugging sources).
  - Keeps the file minimal while still discoverable for new operators.
- **Cons:**
  - Adds non-functional lines to patch files (slightly noisier diffs).
- **Cascading implications:**
  - `init` writes the header and `reset` preserves it.
- **Risks:**
  - Header drift requires stable text and tests.
- **Unlocks:**
  - Faster onboarding without requiring external docs lookup.
- **Quick wins / low-hanging fruit:**
  - A small header reduces “empty patch” confusion immediately.

### Option B: No headers; rely on external docs only
- **Pros:**
  - Cleanest possible YAML files; minimal diffs.
- **Cons:**
  - Patch semantics are easy to forget/misinterpret; higher support burden.
- **Cascading implications:**
  - Requires external docs lookup for patch semantics and inheritance behavior.
- **Risks:**
  - Increased operator confusion and higher support burden.
- **Unlocks:**
  - None aligned with an operator-facing contract.
- **Quick wins / low-hanging fruit:**
  - Short-term smaller diffs at the cost of lower usability.

**Recommendation**
- **Selected:** Option A — Add short comment headers to patch files
- **Rationale (crisp):** Patch semantics are communicated at the file boundary and preserved by the reset/init contract.

**Follow-up tasks (explicit)**
- Implement header creation and preservation semantics: `WCU3-code`, `WCU3-test`, `WCU3-integ`
- Validate headers in playbook and smoke: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0010 — Patch-view UX when empty: conditional stderr note vs silent output

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, `docs/project_management/next/workspace-config-policy-unification/WCU3-spec.md`

**Problem / Context**
- Patch views frequently print `{}` and require a deterministic hint that disambiguates “empty patch” from “effective view”.

### Option A (selected): Conditional stderr note when patch is empty
- **Decision:** `config|policy global show` and `config|policy workspace show` print the patch to stdout and, when the parsed patch is empty, emit a short stderr note pointing to `current show --explain`.
- **Pros:**
  - Preserves machine-parseable stdout while improving operator clarity in the common “empty patch” case.
  - Avoids noise when overrides are present (no note when non-empty).
- **Cons:**
  - Slightly more complexity in CLI output contracts and tests.
- **Cascading implications:**
  - Patch-view commands detect “empty after parse” and emit a single stable note string.
- **Risks:**
  - Inconsistent wording across commands reduces testability and increases confusion.
- **Unlocks:**
  - Reduces operator confusion without changing stdout parse contracts.
- **Quick wins / low-hanging fruit:**
  - Smoke and manual playbook can assert the exact note strings.

### Option B: Always silent for patch views
- **Pros:**
  - Simplest behavior and cleanest output.
- **Cons:**
  - Operators may confuse `{}` with “no config/policy exists” or “Substrate is broken” without a hint.
- **Cascading implications:**
  - Requires external docs lookup for basic “where is my config?” confusion.
- **Risks:**
  - Increases support burden and repeated operator confusion.
- **Unlocks:**
  - None beyond slightly simpler output.
- **Quick wins / low-hanging fruit:**
  - No quick wins; the UX remains ambiguous.

**Recommendation**
- **Selected:** Option A — Conditional stderr note when patch is empty
- **Rationale (crisp):** The stderr note disambiguates `{}` without changing stdout parsing contracts.

**Follow-up tasks (explicit)**
- Implement patch-view empty-note behavior and tests: `WCU3-code`, `WCU3-test`, `WCU3-integ`
- Validate empty-note behavior in playbook and smoke: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0011 — Per-key merge strategy is schema-defined vs hardcoded key list

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/next/workspace-config-policy-unification/WCU2-spec.md`

**Problem / Context**
- Multi-source keys require an auditable, explicit merge rule per key so `--explain` reports truthful behavior.

### Option A (selected): Schema-defined per key
- **Decision:** The config schema registry stores an explicit `merge_strategy` per key (defaulting to `replace` when unspecified).
- **Pros:**
  - Auditable, explicit behavior; avoids ad-hoc special cases.
  - Enables `--explain` to report merge strategy mechanically for every key.
- **Cons:**
  - Requires schema plumbing changes even for a small number of merge keys.
- **Cascading implications:**
  - Future merge keys are added via schema changes and are visible in provenance output.
- **Risks:**
  - Incorrect schema assignment breaks merge behavior deterministically.
- **Unlocks:**
  - Downstream contracts (world-deps) can depend on explicit merge semantics.
- **Quick wins / low-hanging fruit:**
  - The schema becomes the single authoritative list of merge keys.

### Option B: Hardcode merge keys in resolver
- **Pros:**
  - Minimal code changes for the first merge key.
- **Cons:**
  - Not auditable; encourages drift and one-off exceptions.
  - Makes `--explain` incomplete or misleading for merge behavior.
- **Cascading implications:**
  - Merge behavior becomes a scattered implementation detail, not a contract.
- **Risks:**
  - Hidden behavior changes and drift between docs and code.
- **Unlocks:**
  - Faster first-key landing at the cost of long-term non-auditable behavior.
- **Quick wins / low-hanging fruit:**
  - Minimal initial wiring only.

**Recommendation**
- **Selected:** Option A — Schema-defined per key
- **Rationale (crisp):** Schema-defined merge strategies make merge behavior explicit, auditable, and explainable.

**Follow-up tasks (explicit)**
- Implement schema-defined merge strategies + tests: `WCU2-code`, `WCU2-test`, `WCU2-integ`

---

## DR-0012 — `config current show --explain` provenance payload format

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/next/workspace-config-policy-unification/WCU2-spec.md`

**Problem / Context**
- Provenance output requires a deterministic, parseable contract that does not pollute stdout.

### Option A (selected): JSON on stderr with stable, parseable schema
- **Decision:** `--explain` emits a machine-readable JSON object to stderr using a versioned `kind` and a `keys` map keyed by dotpath.
- **Pros:**
  - Scriptable and testable; supports golden files.
  - Deterministic by construction (sorted keys + stable source ordering).
- **Cons:**
  - Less human-friendly than formatted text when reading stderr directly.
- **Cascading implications:**
  - Provenance schema uses a versioned `kind` and a deterministic key ordering strategy.
- **Risks:**
  - Schema drift requires explicit versioning and regression coverage.
- **Unlocks:**
  - Enables stable golden tests for provenance output and downstream tooling consumption.
- **Quick wins / low-hanging fruit:**
  - Smoke asserts on expected keys and ordering without regex.

### Option B: Human-formatted text on stderr
- **Pros:**
  - Easier to read in terminals.
- **Cons:**
  - Harder to test/golden; prone to whitespace and formatting drift.
  - Encourages parsing-by-regex, which is brittle.
- **Cascading implications:**
  - Output formatting becomes part of the contract and changes become breaking.
- **Risks:**
  - Non-determinism via formatting/ordering drift is likely.
- **Unlocks:**
  - Direct readability only.
- **Quick wins / low-hanging fruit:**
  - No quick wins aligned with testability and determinism.

**Recommendation**
- **Selected:** Option A — JSON on stderr with stable, parseable schema
- **Rationale (crisp):** JSON on stderr provides a deterministic, testable contract while keeping stdout as the payload.

**Follow-up tasks (explicit)**
- Implement `--explain` JSON payload contract + determinism tests: `WCU2-code`, `WCU2-test`, `WCU2-integ`
- Validate provenance via playbook and smoke: `WCU5-code`, `WCU5-test`, `WCU5-integ`

---

## DR-0013 — Config editor list mutation syntax for merge keys

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/next/workspace-config-policy-unification/WCU3-spec.md`

**Problem / Context**
- Merge keys require a deterministic, automation-friendly mutation surface that works at both global and workspace scopes.

### Option A (selected): `set <key>+=<item>`
- **Decision:** List merge keys are mutated with the operator syntax `+=` as part of `set`, e.g. `substrate config workspace set world.deps.enabled+=bun`.
- **Pros:**
  - Minimal new CLI surface; consistent with existing `set` usage.
  - Easy to use in automation; works for both scopes symmetrically.
- **Cons:**
  - Requires parser support for operator-style assignments.
- **Cascading implications:**
  - Operator mutation syntax becomes part of the authoritative contract and requires tests.
- **Risks:**
  - Parsing bugs impact multiple keys; regression coverage is required.
- **Unlocks:**
  - Enables world-deps edits without adding new verbs.
- **Quick wins / low-hanging fruit:**
  - Reuses the existing `set` verb for list mutations.

### Option B: Dedicated subcommands (`add` / `remove`)
- **Pros:**
  - More explicit semantics than operator syntax.
- **Cons:**
  - Adds new CLI verbs; larger surface area and more docs/testing.
- **Cascading implications:**
  - Requires separate command surfaces for each scope and verb.
- **Risks:**
  - Increased long-term maintenance burden for a small UX benefit.
- **Unlocks:**
  - Slightly clearer human UX at the cost of a larger contract.
- **Quick wins / low-hanging fruit:**
  - No quick wins; requires new CLI scaffolding and docs.

**Recommendation**
- **Selected:** Option A — `set <key>+=<item>`
- **Rationale (crisp):** Operator syntax keeps the contract minimal and symmetric while supporting automation.

**Follow-up tasks (explicit)**
- Implement and test list mutation operators for merge keys: `WCU3-code`, `WCU3-test`, `WCU3-integ`

---

## DR-0014 — De-duplication policy for ordered-set merges

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/next/workspace-config-policy-unification/WCU2-spec.md`

**Problem / Context**
- The ordered-set merge strategy requires a deterministic de-duplication policy that is explainable for operators and stable for tests.

### Option A (selected): First occurrence wins
- **Decision:** `concat_dedupe_ordered_set` keeps the first occurrence of an item across concatenated layers and drops later duplicates.
- **Pros:**
  - Deterministic and stable; preserves lower-precedence intent ordering.
  - Matches ADR-0012’s ordered-set semantics.
- **Cons:**
  - Higher-precedence layers cannot “re-order” an item without removing it from lower layers (not supported).
- **Cascading implications:**
  - Provenance ordering remains stable and matches layer application order.
- **Risks:**
  - Operators expect “last wins” behavior and require explicit documentation.
- **Unlocks:**
  - Stable ordered-set semantics for downstream consumers (world-deps).
- **Quick wins / low-hanging fruit:**
  - A deliberate duplicate across scopes is a simple test of the contract.

### Option B: Last occurrence wins
- **Pros:**
  - Allows higher-precedence layers to implicitly re-order by re-adding.
- **Cons:**
  - Less intuitive for “enabled set” semantics; surprises operators.
  - Harder to reason about provenance and ordering stability.
- **Cascading implications:**
  - Provenance for duplicates becomes less intuitive because “later overrides” change ordering.
- **Risks:**
  - Re-ordering surprises operators and makes tests brittle.
- **Unlocks:**
  - Re-ordering behavior that is not required by this pack’s consumer contract.
- **Quick wins / low-hanging fruit:**
  - No quick wins aligned with stable ordered-set semantics.

**Recommendation**
- **Selected:** Option A — First occurrence wins
- **Rationale (crisp):** First occurrence wins provides stable ordering aligned with the ordered-set merge contract.

**Follow-up tasks (explicit)**
- Implement and test concat+dedupe ordering rules: `WCU2-code`, `WCU2-test`, `WCU2-integ`

---

## DR-0015 — Patch file representation for “explicit empty list” vs omitted key

**Decision owner(s):** spenser  
**Date:** 2026-01-15  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0012-config-schema-per-key-merge-and-provenance.md`, `docs/project_management/next/workspace-config-policy-unification/WCU2-spec.md`

**Problem / Context**
- Merge keys require a representation for “explicit empty list” that is distinct from “inherit via omitted key” so patch views and provenance remain auditable.

### Option A (selected): Preserve the difference (`[]` is explicit; omitted is inherit)
- **Decision:** For list merge keys, a key present with value `[]` is treated as explicitly set at that scope and is visible in patch views and `--explain` sources; an omitted key contributes nothing.
- **Pros:**
  - Auditable: file reflects operator intent (“I explicitly set this empty”).
  - Improves provenance fidelity for `--explain`.
- **Cons:**
  - Requires explaining the difference in docs and `--explain`.
- **Cascading implications:**
  - `--explain` includes a contributing layer when a key is present as `[]`.
- **Risks:**
  - Operators treat `[]` as “unset”; docs and playbook must distinguish it from omitted keys.
- **Unlocks:**
  - Auditable “explicit empty” cases and trustworthy provenance.
- **Quick wins / low-hanging fruit:**
  - Deterministic edge-case behavior that is straightforward to test.

### Option B: Treat `[]` the same as omitted
- **Pros:**
  - Fewer edge cases; simpler mental model.
- **Cons:**
  - Loses operator intent; makes patch views and provenance less trustworthy.
- **Cascading implications:**
  - Patch views and provenance cannot distinguish “explicit empty” from “inherit”.
- **Risks:**
  - Operators cannot audit or reason about why a list is empty.
- **Unlocks:**
  - Simpler representation at the cost of reduced auditability.
- **Quick wins / low-hanging fruit:**
  - No quick wins aligned with explainability and auditability.

**Recommendation**
- **Selected:** Option A — Preserve the difference (`[]` is explicit; omitted is inherit)
- **Rationale (crisp):** Distinguishing `[]` from omitted keys is required for auditable patch views and trustworthy provenance.

**Follow-up tasks (explicit)**
- Implement and test omitted-vs-empty semantics in merge + provenance: `WCU2-code`, `WCU2-test`, `WCU2-integ`
