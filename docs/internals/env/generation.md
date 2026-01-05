# Environment Variable Inventory Generation

`docs/internals/env/inventory.md` is maintained as a **semi-generated** artifact:

- The **set of variables and evidence sites** is derived from repo searches (Rust + scripts + workflows).
- The **taxonomy and exclusion notes** are hand-curated to keep the inventory accurate (avoid script-local variables, docs-only mentions, and planned-but-unused names).

This is intentionally repo-grounded: `docs/**` is not treated as a source of truth for env var *usage*.

## Update Workflow

1. Regenerate a usage report (paths + read/write direction) by searching the same directories the inventory covers:
   - Rust: `crates/**`, `src/**`
   - Scripts/fixtures: `scripts/**`, `tests/**`
   - Config: `config/**` (notably `config/manager_hooks.yaml`)
   - CI: `.github/workflows/**`

2. Update the catalog table in `docs/internals/env/inventory.md` from that report:
   - Ensure every row has at least one concrete usage site (file path + literal env name).
   - Ensure `SUBSTRATE_OVERRIDE_*` reserved names are excluded unless the repo actually reads/writes them.

3. Review for false positives/negatives:
   - False positives usually come from **script-local shell variables** that were never exported, never defaulted from `${VAR:-...}`, and never passed as `VAR=value cmd`.
   - False negatives usually come from **wrappers/helpers** (e.g., `get_env_u64(...)`, `parse_allowlist_env(...)`) and **compile-time env macros** (`env!`, `option_env!`, `cargo:rustc-env=...`).

## Recommended Search Patterns

These are the minimum patterns that must be covered when updating the inventory.

### Rust (runtime env)

- Reads/writes via stdlib:
  - `rg -n \"\\bstd::env::(var|var_os|set_var|remove_var)\\b\" crates src`
  - `rg -n \"\\benv::(var|var_os|set_var|remove_var)\\b\" crates src`
- Child process env injection:
  - `rg -n \"\\.(env|env_remove)\\(\" crates src`
- Non-stdlib/env wrappers (repo-specific):
  - `rg -n \"\\bget_env_[a-z0-9_]+\\(\\\"[A-Z0-9_]+\\\"\" crates src`
  - `rg -n \"\\bparse_allowlist_env\\(\" crates src`
- libc-style reads:
  - `rg -n \"\\bgetenv\\(\\\"[A-Z0-9_]+\\\"\\)\" crates src`

### Rust (build-time env)

- Compile-time env access and build script exports:
  - `rg -n \"\\b(env|option_env)!\\(\\\"[A-Z0-9_]+\\\"\\)\" crates src`
  - `rg -n \"cargo:rustc-env=[A-Z0-9_]+=\" crates`

### Shell scripts (.sh) and embedded shell fragments

- Exports:
  - `rg -n \"^\\s*export\\s+[A-Z_][A-Z0-9_]*\" scripts tests config`
- Defaulting reads:
  - `rg -n \"\\$\\{[A-Z_][A-Z0-9_]*:-\" scripts tests config`
- Environment assignment prefixes (writes to child process env):
  - `rg -n \"\\b[A-Z_][A-Z0-9_]*=.*\\s+[^=\\s]+\" scripts tests`

### PowerShell (.ps1)

- Reads/writes:
  - `rg -n \"\\$env:[A-Za-z0-9_]+\" scripts tests`

### GitHub Actions workflows

- Environment blocks:
  - `rg -n \"^\\s*env:\\s*$\" .github/workflows`
  - `rg -n \"^\\s+[A-Z_][A-Z0-9_]*:\\s\" .github/workflows`

## Platform-Specific Variables

When a variable is platform-specific, keep it in the catalog with its OS-specific evidence site:

- Windows: `scripts/windows/*.ps1`, `crates/world-windows-wsl/**`
- macOS: `scripts/mac/*.sh`, `crates/world-mac-lima/**`
- Linux: `scripts/linux/*.sh`, `crates/world/**`, `crates/world-agent/**`

The catalog’s evidence columns must include at least one real platform-specific usage site for these variables.
