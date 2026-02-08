# C5-spec — Non-interactive `-c/--command` and stdin pipe mode world-consistency

This slice implements the non-interactive consistency requirements from ADR-0016:
- when world is enabled, `cd`/`pwd`/`export`/`unset` MUST be interpreted in-world (shell semantics),
- and `:host` MUST NOT be recognized in non-interactive flows.

Authoritative specs (do not diverge):
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md` (CLI contract for `-c/--command`)
- `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md` (DR-05, DR-10)

Depends on:
- `C3` (routing changes overlap; avoid conflicting edits).

Out of scope for C5:
- Interactive persistent session protocol (`start_session` / `exec` / `command_complete`) — C0/C1/C2/C3/C4.

## Contract (C5 deliverable)

Non-interactive `-c/--command`:
- When world is enabled and available, the provided command string MUST execute inside the world (non-PTY by default) and MUST observe the in-world filesystem view.
- In this mode, `cd`/`pwd`/`export`/`unset` MUST NOT be executed as host-only builtins; they MUST be interpreted by the in-world shell/process.
- `:host` MUST NOT be recognized in this mode.

Pipe mode (stdin-driven execution):
- Pipe mode MUST follow the same routing rules as `-c` regarding host builtins and `:host` non-recognition.

## Acceptance criteria
- Running `substrate -c` against a world-only path does not reintroduce host `fs::canonicalize()`-driven failures:
  - Create a directory in the world overlay view via a non-builtin command.
  - Confirm the directory does not exist on the host filesystem.
  - Confirm `substrate -c "cd <dir> && pwd -P"` succeeds when world is enabled.
- `:host` in `-c/--command` is treated as literal command text and does not bypass world isolation.

## Validation (C5-test scope)
Add tests that cover, at minimum:
- `-c/--command` behavior with world-only paths (regression for “exists in world but cd fails”).
- `:host` non-recognition in `-c` and stdin pipe mode.

