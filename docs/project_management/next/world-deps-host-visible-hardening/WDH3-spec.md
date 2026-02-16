# WDH3 (CP2) — Installer scaffolding for `$SUBSTRATE_HOME/deps/` + examples

## Goal
Reduce operator confusion by scaffolding `$SUBSTRATE_HOME/deps/` during install/first-run init with non-enabling examples and a README describing inventory vs enabled/applied.

## Inputs (authoritative)
- `docs/project_management/next/ADR-0011-world-deps-packages-bundles-contract.md` (Appendix A)
- `docs/project_management/next/world-deps-host-visible-hardening/decision_register.md` (DR-0006, DR-0010)

## Contract

### When scaffolding occurs
The system MUST scaffold `$SUBSTRATE_HOME/deps/` during `$SUBSTRATE_HOME` initialization (first-run bootstrap) performed by the Substrate binary.

Scaffolding MUST be idempotent and MUST NOT overwrite user-modified files.

### Layout (required)
Create:
```text
$SUBSTRATE_HOME/deps/
  README.md
  packages/
    example-manual.yaml
    example-script.yaml
    example-apt.yaml
  bundles/
    example-bundle.yaml
  scripts/
    example-install.sh
```

### Example contents
- Examples are “shape only” and MUST NOT be auto-enabled.
- `README.md` MUST explain:
  - built-in inventory can appear in “available” even without `$SUBSTRATE_HOME/deps/`
  - global inventory location (`$SUBSTRATE_HOME/deps/`)
  - workspace inventory location (`<workspace_root>/.substrate/deps/`)
  - enabled lists live in patch files (global/workspace)
  - applying occurs via `current sync` and is world-backed (no host fallback)

## Exit codes
- Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`
- 1: unexpected I/O errors creating scaffold dirs/files
- 5: safety/policy denial writing to `$SUBSTRATE_HOME` (if such policy exists)

## Acceptance criteria
- After install/first-run init, `$SUBSTRATE_HOME/deps/{packages,bundles,scripts}` exist and `README.md` exists.
- No examples are enabled by default (enabled lists remain empty unless explicitly edited by operator).
- The smoke script validates the scaffold existence.

## Tests (required)

### Unit/integration tests (Rust)
Add tests (at minimum) covering:
- **Create-when-missing:** with a fresh `SUBSTRATE_HOME`, running a command that triggers `$SUBSTRATE_HOME` bootstrap (e.g. `substrate --version`) creates the full deps scaffold (dirs + `README.md` + example files).
- **Idempotent/non-overwrite:** if any scaffolded file already exists, the bootstrap MUST NOT modify its contents.
- **Wrong-type handling:** if a required scaffold path exists with the wrong type (e.g. `deps/packages` exists as a file), bootstrap MUST fail with exit `1` and an actionable error.

### Smoke
`docs/project_management/next/world-deps-host-visible-hardening/smoke/_core.sh` MUST include a WDH3 slice assertion that the scaffold exists after bootstrap under a fresh `SUBSTRATE_HOME`.
