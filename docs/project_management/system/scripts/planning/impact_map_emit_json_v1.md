# CONTRACT-4: `validate_impact_map.py --emit-json` contract (v1)

This document defines the stable machine contract for:

- `docs/project_management/system/scripts/planning/validate_impact_map.py --emit-json`

This is **CONTRACT-4:impact_map_emit_json_v1**.

Evolution rule (v1):
- **Additive keys only**. Never rename or remove existing keys in v1 output.

---

## Stdout / stderr / exit codes (normative)

When `--emit-json` is provided, `validate_impact_map.py` MUST:

- Write **JSON only** to stdout on success.
- Write nothing to stdout on any non-zero exit.
- Write all warnings/errors to stderr.
- Exit codes:
  - `0` on success
  - `2` on usage errors (missing/invalid args, invalid/missing `tasks.json`, invalid JSON)
  - `1` on strict-mode validation failures (e.g., missing `impact_map.md`, invalid tokens)

---

## Required keys + types (normative)

On success, stdout MUST be a single JSON object with these required keys (always present, even in legacy mode):

```json
{
  "create": "array<string> (sorted asc, unique)",
  "edit": "array<string> (sorted asc, unique)",
  "deprecate": "array<string> (sorted asc, unique)",
  "delete": "array<string> (sorted asc, unique)",
  "dir_prefixes": "array<string> (sorted asc, unique; each ends with \"/\")"
}
```

Notes:
- All arrays are deterministic for the same inputs.
- `dir_prefixes` is derived from the allowlists: it is the set of any tokens that end with `/` present in any allowlist.

---

## Token semantics (strict mode, normative)

In strict mode, all tokens MUST be:

- Repo-root-relative path tokens (relative to `git rev-parse --show-toplevel`).
- Normalized by removing any leading `./` segments until none remain.

Tokens MUST NOT contain:
- `..` path segments
- absolute paths (start with `/`)
- home-relative paths (start with `~`)
- drive-letter paths (`C:...`)
- backslashes (`\\`)
- double slashes (`//`)
- glob characters (`* ? [ ] { }`)

Directory allowlist entries MUST end with `/`.

---

## Mode gating (normative)

`validate_impact_map.py` supports:
- `--mode strict|legacy` (override), or
- derived mode from `<feature_dir>/tasks.json`:
  - strict iff `tasks.json.meta.slice_spec_version >= 2`
  - legacy otherwise

Legacy mode behavior:
- If mode is legacy and `--emit-json` is set, stdout MUST still be JSON-only and MUST have the full key set above, with all arrays empty (including `dir_prefixes`).

---

## Example payloads (valid v1)

Strict, explicit-only:

```json
{
  "create": ["docs/foo.md"],
  "edit": [],
  "deprecate": [],
  "delete": [],
  "dir_prefixes": []
}
```

Strict, prefix-present:

```json
{
  "create": ["crates/world-agent/", "docs/foo.md"],
  "edit": [],
  "deprecate": [],
  "delete": [],
  "dir_prefixes": ["crates/world-agent/"]
}
```

