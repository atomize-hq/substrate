# Contract Surface Standard (`contract.md`)

Goal:
- Reduce drift by consolidating the user-facing contract (CLI/config/exit codes/paths) into a single, feature-local file.

## When to use

Create `docs/project_management/next/<feature>/contract.md` when the work defines or changes any of:
- CLI commands/flags/defaults
- Config files, paths, precedence, schema constraints
- Exit code meanings
- User-visible path semantics / protected paths

## Rules

- `contract.md` is the single place that holds the final contract wording.
- ADRs/specs/playbooks should link to `contract.md` rather than re-stating tables in multiple places.
- If a document needs to repeat a snippet (e.g., for readability), it must still link back to `contract.md` and must not contradict it.
- If `contract.md` exists for a feature, treat it as the conflict resolver: in case of drift, integration reconciles code/tests to `contract.md`.

## Minimum sections

- CLI
- Config
- Exit codes (including taxonomy reference + any overrides)
- Platform guarantees (Linux/macOS/Windows; include WSL if relevant)
- Protected paths / invariants (if filesystem semantics exist)

## Tooling

- Scaffold: `make planning-new-feature FEATURE=<feature>` creates `contract.md` from `docs/project_management/standards/templates/contract.md.tmpl`.
- Lint: `make planning-lint` does not (yet) enforce `contract.md`; it remains a planning-level drift prevention tool.
