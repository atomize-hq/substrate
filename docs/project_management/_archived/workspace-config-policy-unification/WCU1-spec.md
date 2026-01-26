# WCU1 Spec — Workspace directory + internal git unification (ADR-0008)

References:
- `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`
- `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`

## Scope
- Unify workspace state under a single canonical directory: `<workspace_root>/.substrate/`.
- Define the workspace root marker, disable marker behavior, and internal git location.
- Ensure nested workspace init is refused deterministically.

## Non-goals
- Any support for alternative workspace state directories beyond `<workspace_root>/.substrate/`.
- Migrations, warnings, or auto-repair of legacy layouts unless explicitly mandated by ADR-0008.

## Authoritative definitions
- **Workspace root**: the nearest ancestor directory containing `<dir>/.substrate/workspace.yaml` such that `<dir>/.substrate/workspace.disabled` does not exist.
- **Workspace disabled**: a directory that has `<workspace_root>/.substrate/workspace.disabled`; it is treated as non-existent for discovery and effective resolution.

## Contract (authoritative behaviors)

### Workspace layout (canonical)
When a workspace exists, the canonical paths are:
- Workspace config patch: `<workspace_root>/.substrate/workspace.yaml`
- Workspace policy patch: `<workspace_root>/.substrate/policy.yaml`
- Workspace disable marker: `<workspace_root>/.substrate/workspace.disabled` (presence disables the workspace)
- Internal git: `<workspace_root>/.substrate/git/repo.git/`

### `substrate workspace init <path>`
- Creates `<path>/.substrate/` and required files/directories:
  - `<path>/.substrate/workspace.yaml` (patch file; valid YAML mapping; default `{}`; includes a short comment header)
  - `<path>/.substrate/policy.yaml` (patch file; valid YAML mapping; default `{}`; includes a short comment header)
  - `<path>/.substrate/git/repo.git/` (internal git directory)
- Updates `<path>/.gitignore` to:
  - ignore `.substrate/` by default, and
  - explicitly allowlist the two patch files:
    - `!.substrate/workspace.yaml`
    - `!.substrate/policy.yaml`
- Nested workspace refusal:
  - If any parent directory of `<path>` contains `.substrate/workspace.yaml`, `workspace init` exits `2` and performs no writes outside `<path>/.substrate/` and `<path>/.gitignore`.

#### `--force` (repair mode)
- Repairs missing workspace entries only.
- It MUST NOT overwrite existing non-empty patch files:
  - `<path>/.substrate/workspace.yaml`
  - `<path>/.substrate/policy.yaml`

#### `--examples`
- Creates these template files under the workspace root:
  - `<path>/.substrate/workspace.example.yaml`
  - `<path>/.substrate/policy.example.yaml`
- Substrate MUST NOT read these example files for any behavior.

### `substrate workspace disable <path>` / `enable`
- `disable` creates `<workspace_root>/.substrate/workspace.disabled` (idempotent).
- `enable` removes `<workspace_root>/.substrate/workspace.disabled` if present (idempotent).
- When disabled, the workspace is ignored for:
  - discovery (commands behave as if no workspace exists), and
  - effective resolution for `config current ...` / `policy current ...`.

## Exit codes (authoritative)
Use `docs/project_management/standards/EXIT_CODE_TAXONOMY.md`.
- Refusing nested workspace init: exit `2`.

## Validation requirements (authoritative)
- Unit/integration tests must cover:
  - workspace root discovery from nested directories,
  - disabled marker behavior (workspace ignored),
  - internal git directory path under `.substrate/git/repo.git/`,
  - nested workspace init refusal (exit `2`) with no unintended writes.
