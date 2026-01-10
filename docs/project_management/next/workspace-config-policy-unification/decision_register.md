# Decision Register — Workspace Config/Policy Unification (ADR-0008)

This decision register records the architectural decisions for `ADR-0008`.

Rules:
- Each decision is recorded as exactly two options (A/B), with explicit tradeoffs and one selection.
- No backwards compatibility or migration behavior is included unless explicitly stated.

---

## DR-0001 — Workspace/global files are patches (sparse) vs full documents

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

### Option B: Full documents only (no sparse keys)
- **Pros:**
  - Simpler parsing and validation (deserialize directly into the full structs).
- **Cons:**
  - Makes “unset/reset a single key to inherit” impossible without copying values.
  - Encourages drift and confusion because the workspace file becomes a full shadow of global.

---

## DR-0002 — `current show` provenance visibility

### Option A (selected): Quiet by default + explicit `--explain`
- **Decision:** `current show` prints the effective/merged config or policy on stdout and prints a single stderr notice indicating it is merged; `--explain` adds a per-key provenance breakdown.
- **Pros:**
  - Keeps default output machine- and human-friendly.
  - Makes it easy to debug “why is this value set?” without permanently noisy output.
- **Cons:**
  - Requires users to learn `--explain` to get deeper context.

### Option B: Always print provenance
- **Pros:**
  - Maximum transparency with no extra flags.
- **Cons:**
  - Noisy; complicates piping/parsing; increases incidental confusion for basic usage.

---

## DR-0003 — CLI surface: explicit scopes vs legacy aliases

### Option A (selected): Explicit scopes; no legacy aliases
- **Decision:** Require explicit `current|global|workspace` scopes; do not keep legacy `config show` / `config set` / `policy show` / `policy set` as aliases.
- **Pros:**
  - Eliminates ambiguity between “effective view” vs “scope file view”.
  - Aligns policy and config CLI symmetry.
- **Cons:**
  - Breaking change for existing usage (explicitly accepted as greenfield).

### Option B: Keep aliases for older commands
- **Pros:**
  - Lower transition cost for existing users.
- **Cons:**
  - Preserves ambiguity and increases documentation/testing complexity.

---

## DR-0004 — Workspace disable persistence mechanism

### Option A (selected): Persistent marker file
- **Decision:** Implement workspace disable/enable via a marker file at `<workspace_root>/.substrate/workspace.disabled`.
- **Pros:**
  - Deterministic; survives shell restarts; does not depend on parent shell env mutation.
- **Cons:**
  - Adds one more file under `.substrate/`.

### Option B: Environment variable / session-only toggle
- **Pros:**
  - No on-disk state.
- **Cons:**
  - Hard to apply consistently across tooling and shells; not durable.

---

## DR-0005 — Workspace internal git location

### Option A (selected): Internal git lives under `.substrate/`
- **Decision:** Store internal git at `<workspace_root>/.substrate/git/repo.git/` (replaces `<workspace_root>/.substrate-git/repo.git/`).
- **Pros:**
  - One canonical workspace state directory: `.substrate/`.
  - Cleaner sync exclusions (a single protected root).
- **Cons:**
  - Requires updating docs/specs/tests that reference `.substrate-git`.

### Option B: Keep `.substrate-git/` at workspace root
- **Pros:**
  - Matches existing docs/specs.
- **Cons:**
  - Splits workspace state across directories; hurts onboarding and increases protected-path surface.

---

## DR-0006 — Workspace init example templates

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

### Option B: Always create example files
- **Pros:**
  - Templates always available.
- **Cons:**
  - Increases workspace clutter; encourages hand-editing and drift.

---

## DR-0007 — Override env vars are one-off; installers must not export them

### Option A (selected): Overrides are explicit one-offs only
- **Decision:** `SUBSTRATE_OVERRIDE_*` is treated as explicit one-off operator input and MUST NOT be exported by install/dev env scripts by default.
- **Pros:**
  - Prevents “global set did nothing” confusion caused by persistent override exports.
  - Restores the mental model: global is baseline unless workspace overrides exist.
- **Cons:**
  - Requires updating install/dev scripts and docs.

### Option B: Allow installers to export overrides
- **Pros:**
  - Easy for installers to force desired behavior without config edits.
- **Cons:**
  - Reintroduces precedence confusion and breaks parity between config and policy.

---

## DR-0008 — Legacy directory layouts and marker names

### Option A (selected): Single canonical `.substrate/` directory; ignore legacy silently
- **Decision:** Only `.substrate/` is recognized as the workspace state directory. Legacy alternative layouts (e.g., `.substrate-profile*`, `.substrate-git/`) are not recognized and produce no special warnings or migrations.
- **Pros:**
  - Clean onboarding and deterministic rules.
  - Keeps implementation smaller and avoids migration logic.
- **Cons:**
  - Existing users with legacy layouts must self-correct without tool guidance.

### Option B: Detection + migration + warnings for legacy layouts
- **Pros:**
  - Better transitional UX for existing users.
- **Cons:**
  - Non-greenfield complexity; requires migration plans and ongoing compat tests.

---

## DR-0009 — Patch file comment headers (clarity) vs bare YAML mappings

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

### Option B: No headers; rely on external docs only
- **Pros:**
  - Cleanest possible YAML files; minimal diffs.
- **Cons:**
  - Patch semantics are easy to forget/misinterpret; higher support burden.

---

## DR-0010 — Patch-view UX when empty: conditional stderr note vs silent output

### Option A (selected): Conditional stderr note when patch is empty
- **Decision:** `config|policy global show` and `config|policy workspace show` print the patch to stdout and, when the parsed patch is empty, emit a short stderr note pointing to `current show --explain`.
- **Pros:**
  - Preserves machine-parseable stdout while improving operator clarity in the common “empty patch” case.
  - Avoids noise when overrides are present (no note when non-empty).
- **Cons:**
  - Slightly more complexity in CLI output contracts and tests.

### Option B: Always silent for patch views
- **Pros:**
  - Simplest behavior and cleanest output.
- **Cons:**
  - Operators may confuse `{}` with “no config/policy exists” or “Substrate is broken” without a hint.
