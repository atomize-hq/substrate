### S1 — CONTRACT-4 (`validate_impact_map.py --emit-json`) published + locked

- **User/system value**: Make Planning Pack-derived lift inputs deterministic and machine-consumable, so downstream tooling (SEAM-3) and workflow integration (SEAM-5) can depend on a stable shape without re-parsing markdown.
- **Scope (in/out)**:
  - In:
    - Specify CONTRACT-4 (keys + types + semantics + evolution rules).
    - Ensure `validate_impact_map.py --emit-json` conforms across strict and legacy modes.
    - Document how directory/prefix entries are represented and surfaced (`dir_prefixes`), without changing the authored `impact_map.md`.
  - Out:
    - Implementing lift scoring or output shape for `pm_lift --emit-json` (owned by SEAM-3 / CONTRACT-3).
    - Prefix expansion implementation details inside `pm_lift` (wiring owned by SEAM-3; semantics are specified in S2 here).
- **Acceptance criteria**:
  - CONTRACT-4 is written down in a single crisp spec doc (keys, types, semantics):
    - `create/edit/deprecate/delete`: arrays of normalized path tokens (strings),
    - `dir_prefixes`: array of directory-prefix tokens (strings, each ending with `/`), sorted and stable.
  - Legacy mode behavior is explicit:
    - `--emit-json` still emits the full object with all keys present (allowlists empty; `dir_prefixes` empty).
  - Evolution rule is explicit:
    - additive keys only; never rename/remove existing v1 keys.
  - `validate_impact_map.py --emit-json` stdout is JSON-only; warnings/errors go to stderr.
- **Dependencies**:
  - Produces: `CONTRACT-4:impact_map_emit_json_v1`
  - Coordinate-with consumer: SEAM-3 (`pm_lift` derived-pack handling) for any semantic/key changes.
- **Verification**:
  - Conformance tests run `validate_impact_map.py --emit-json` against fixture packs and assert:
    - required keys exist,
    - types are correct,
    - ordering/determinism (sorted arrays) is stable,
    - legacy mode returns the same shape with empty arrays.
- **Rollout/safety**:
  - Advisory-first: this contract is an internal interface for tooling; it MUST NOT introduce new enforcement behavior on its own.

##### CONTRACT-4: `validate_impact_map.py --emit-json` concrete contract (v1, normative)

When invoked with `--emit-json`, `docs/project_management/system/scripts/planning/validate_impact_map.py` MUST:

- write **JSON only** to stdout,
- write nothing to stdout on any non-zero exit,
- write warnings/errors to stderr,
- exit `0` on success,
- exit `1` on validation failure,
- exit `2` on usage errors (missing/invalid args, invalid/missing `tasks.json`, invalid JSON).

The stdout JSON MUST be a single object with these required keys (always present, even in legacy mode):

```json
{
  "create": "array<string> (sorted asc, unique)",
  "edit": "array<string> (sorted asc, unique)",
  "deprecate": "array<string> (sorted asc, unique)",
  "delete": "array<string> (sorted asc, unique)",
  "dir_prefixes": "array<string> (sorted asc, unique; each ends with \"/\")"
}
```

Token semantics (strict mode):

- Every token is a repo-root-relative path token (relative to `git rev-parse --show-toplevel`).
- Normalization MUST remove any leading `./` segments until none remain.
- Tokens MUST NOT contain:
  - `..` path segments,
  - absolute paths (`/…`),
  - home-relative paths (`~…`),
  - drive-letter paths (`C:…`),
  - backslashes (`\\`),
  - double slashes (`//`),
  - glob characters (`* ? [ ] { }`).
- Directory allowlist entries MUST end with `/`.

Mode gating:

- `--mode` is one of `strict|legacy`.
- Without `--mode`, the tool derives mode from `tasks.json` under `--feature-dir`:
  - strict iff `tasks.json.meta.slice_spec_version >= 2`
  - legacy otherwise

Legacy mode contract:

- If mode is legacy and `--emit-json` is set, stdout MUST still be JSON-only and MUST have the full key set above, with all arrays empty (including `dir_prefixes`).

#### S1.T1 — Write the CONTRACT-4 specification (keys, semantics, evolution)

- **Outcome**: A reviewable contract doc that downstream tools can implement against without reading `validate_impact_map.py`.
- **Inputs/outputs**:
  - Inputs:
    - `work_lift_v1_seams/threading.md` (CONTRACT-4 registry entry)
    - `docs/project_management/system/scripts/planning/validate_impact_map.py` (current behavior)
  - Outputs:
    - A short spec doc colocated with the script, e.g.:
      - `docs/project_management/system/scripts/planning/impact_map_emit_json_v1.md`
- **Implementation notes**:
  - Define:
    - required keys + types,
    - normalization rules (no `./`, no `..`, no glob tokens, no absolute paths),
    - strict vs legacy semantics (shape always stable),
    - `dir_prefixes` meaning: “entries ending with `/` present anywhere in allowlists”.
  - Explicitly state additive-only evolution for v1.
- **Acceptance criteria**:
  - A consumer can parse and validate the JSON without consulting code.

Checklist:
- Implement:
  - Write required keys/types and strict-vs-legacy semantics.
  - Include at least one example JSON payload (small).
- Validate:
  - Cross-check with `work_lift_v1_seams/threading.md` CONTRACT-4 language.
- Cleanup:
  - Keep the spec short, contract-focused, and tool-agnostic.

#### S1.T2 — Ensure `validate_impact_map.py --emit-json` always conforms to CONTRACT-4

- **Outcome**: Implementation matches the contract in both strict and legacy modes, with deterministic ordering and JSON-only stdout.
- **Inputs/outputs**:
  - Inputs:
    - `validate_impact_map.py` current JSON emission
    - CONTRACT-4 spec from S1.T1
  - Outputs:
    - Any required updates to `validate_impact_map.py` output emission + help text.
- **Implementation notes**:
  - Determinism:
    - sort allowlists and `dir_prefixes`,
    - keep key set stable (always include all keys).
  - Ensure `dir_prefixes` is derived from the allowlists (not separately maintained).
- **Acceptance criteria**:
  - Contract conformance tests pass.
  - No warnings are printed to stdout in `--emit-json` mode.

Checklist:
- Implement:
  - Align emission (keys + sorting + legacy behavior).
- Test:
  - Run conformance tests (S1.T3).
- Cleanup:
  - Keep failure modes actionable and stable.

#### S1.T3 — Add contract conformance tests for CONTRACT-4 (strict + legacy)

- **Outcome**: Fast tests that fail on accidental drift (missing keys, wrong types, unstable ordering).
- **Inputs/outputs**:
  - Inputs:
    - `validate_impact_map.py`
    - Minimal fixture pack(s) with `tasks.json` + `impact_map.md`
  - Outputs:
    - New tests under `docs/project_management/system/scripts/planning/tests/`
    - Fixture directories under a new `tests/fixtures/` folder (small, self-contained)
- **Concrete test module + cases (pinned)**:
  - Test module:
    - `docs/project_management/system/scripts/planning/tests/test_validate_impact_map_emit_json_contract.py`
  - Fixture strategy:
    - runtime-written feature dirs under:
      - `target/pm_script_tests/validate_impact_map/emit_json_contract/<case>/`
    - strict-mode fixtures include only Create entries (Edit/Deprecate/Delete are `- None`) to avoid filesystem coupling.
  - Cases (must remain deterministic):
    - `STRICT-A` explicit-only: Create tokens `__impact_map_test__/a.txt`, `__impact_map_test__/b.txt` → `dir_prefixes: []`
    - `STRICT-B` prefix-present: Create tokens `__impact_map_test__/c.txt`, `__impact_map_test__/no_such_prefix/` → `dir_prefixes: ["__impact_map_test__/no_such_prefix/"]`
    - `STRICT-C` normalization: Create token `./__impact_map_test__/d.txt` → output token `__impact_map_test__/d.txt`
    - `LEGACY-A` shape stability: `meta.slice_spec_version = 1` and no `impact_map.md` → full key set with empty arrays
    - `USAGE-A` missing args → exit 2, stdout empty, stderr includes `usage:`
    - `STRICT-FAIL-A` missing `impact_map.md` → exit 1, stdout empty, stderr includes `missing required path`
    - `STRICT-FAIL-B` invalid glob token `__impact_map_test__/*.txt` → exit 1, stdout empty, stderr includes `glob tokens are not allowed`
- **Implementation notes**:
  - Cover at least:
    - explicit-only pack (no prefixes → `dir_prefixes: []`),
    - prefix-only or mixed pack (prefixes present → `dir_prefixes` contains those tokens),
    - legacy mode pack (`meta.slice_spec_version < 2`) returns empty allowlists and empty `dir_prefixes` but same keys.
  - Keep fixtures tiny and stable (no dependency on repo history).
- **Acceptance criteria**:
  - Tests assert required keys/types and deterministic sorted arrays.
  - Tests do not depend on system-specific paths or git state.

Checklist:
- Implement:
  - Add fixtures and subprocess-based tests.
  - Assert shape + types + sorted ordering.
- Validate:
  - Run tests locally.
- Cleanup:
  - Keep fixtures minimal and readable.

#### S1.T4 — Document directory/prefix semantics in the Impact Map standard

- **Outcome**: Humans/agents understand how directory tokens must be authored and how they affect advisory lift.
- **Inputs/outputs**:
  - Inputs:
    - `docs/project_management/system/standards/planning/PLANNING_IMPACT_MAP_STANDARD.md`
    - CONTRACT-4 spec (S1.T1)
  - Output:
    - Updated standard doc section describing:
      - directory tokens must end with `/`,
      - they count as 1 in raw counts,
      - prefix tokens are expanded deterministically for lift estimation only (per SEAM-4; no rewrites),
      - confidence is degraded when prefixes are present.
- **Acceptance criteria**:
  - Standard doc aligns with both validator behavior and lift semantics (no contradictions).

Checklist:
- Implement:
  - Add a short “Directory/prefix entries” section.
- Validate:
  - Cross-check language matches CONTRACT-4 and SEAM-4 invariants.
- Cleanup:
  - Keep it advisory-first; avoid enforcement language unless it already exists.
