# Planning Session Log Template (Docs-First Passes)

This template is for **planning/research/documentation sessions** that modify a Planning Pack under:
- `docs/project_management/next/<feature>/`

The planning standard requires START/END entries only (no mid-stream commentary). Use this template to keep planning work as strict, traceable, auditable, and repeatable as execution triads.

Apply inside the feature’s `session_log.md`:
- `docs/project_management/next/<feature>/session_log.md`

---

## START entry (planning)

Paste a START entry like this:

```md
## START — <YYYY-MM-DDTHH:MM:SSZ> — planning — <short title>
- Feature: `docs/project_management/next/<feature>/`
- Branch: `<branch>`
- Goal: <single sentence>
- Inputs to read end-to-end:
  - <explicit list of files>
- Commands planned (if any):
  - `<command>`
```

Rules:
- The “Inputs to read end-to-end” list must match what you actually read.
- Do not claim to have read files you did not read end-to-end.

---

## END entry (planning)

Paste an END entry like this:

```md
## END — <YYYY-MM-DDTHH:MM:SSZ> — planning — <short title>
- Summary of changes (exhaustive):
  - <bullet per file or per logical change>
- Files created/modified:
  - `<path>`
- Rubric checks run (with results):
  - `rg '<pattern>' <scope>` → `<exit code>` → `<result>`
  - `jq -e . tasks.json` → `<exit code>` → `<result>`
- Sequencing alignment:
  - `sequencing.json` reviewed: `<YES|NO>`
  - Changes required: `<NONE|explicit list>`
- Blockers:
  - `<NONE|explicit list>`
- Next steps:
  - `<explicit next actions + which triad owns them>`
```

Rules:
- “Summary of changes” must be exhaustive.
- “Rubric checks run” must list actual commands executed.
- If you changed `tasks.json`, include a statement that required fields exist and that kickoff prompts exist.

