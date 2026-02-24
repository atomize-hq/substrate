### S1 — CONTRACT-3 (`pm_lift --emit-json`) published + locked

- **User/system value**: Unblock downstream integration (SEAM-5) by making `pm_lift` JSON output stable, parseable, and safely extensible (additive evolution only).
- **Scope (in/out)**:
  - In:
    - Define the JSON output contract (keys + semantics) and how additive evolution is handled.
    - Align `pm_lift.py` to emit that contract across all subcommands.
    - Establish stable error/exit behavior for machine consumption (without introducing enforcement gates).
  - Out:
    - Tuning scoring math (owned by config in SEAM-2 and computation in S2).
    - Deep schema validation beyond “shape + actionable errors” (S2 once SEAM-1 lands).
- **Acceptance criteria**:
  - `pm_lift.py ... --emit-json` emits a JSON object with stable keys:
    - `model_version`, `lift_score`, `estimated_slices`, `confidence`,
    - `triggers`, `missing_inputs`,
    - `vector`, `derived`.
- Key semantics are documented:
    - `confidence` is a low-cardinality string enum (`high|low` in v1) and is deterministic for a given input/config.
    - `triggers` and `missing_inputs` are stable, machine-friendly strings (no human prose), and ordering is deterministic.
    - `derived` contains intermediate values useful for debugging/auditing; additive keys only.
  - Exit codes are stable:
    - `0` success,
    - non-zero for invalid input / missing markers / invalid config, with actionable `stderr`.
  - Contract evolution rule is explicit: additive keys only; never rename/remove existing keys in v1.
- **Dependencies**:
  - Contracts: produces `CONTRACT-3:pm_lift_emit_json_v1`
  - Reference-only input: `work_lift_v1_seams/threading.md` (contract registry)
- **Verification**:
  - Add a contract conformance test (or minimal fixture + assertion) that checks key presence/types and stable ordering behavior.
  - Manual smoke:
    - `pm_lift.py from-git-diff --git-range HEAD~1..HEAD --emit-json` (or a small range) prints JSON-only to stdout.
- **Rollout/safety**:
  - Advisory-first: contract guarantees shape and determinism, not “correctness of scoring” until S2/S3 land.

#### S1.T1 — Write the CONTRACT-3 specification (keys, semantics, evolution)

- **Outcome**: A crisp, reviewable definition of the `--emit-json` contract that downstream tooling can depend on without reading code.
- **Inputs/outputs**:
  - Inputs:
    - `work_lift_v1_seams/threading.md` (CONTRACT-3 definition)
    - `docs/project_management/system/scripts/planning/pm_lift.py` (current output fields)
  - Outputs:
    - A short contract spec doc colocated with the tool (e.g., `docs/project_management/system/scripts/planning/pm_lift_emit_json_v1.md`) including an example JSON.
- **Implementation notes**:
  - Define:
    - required keys + types,
    - allowable values for `confidence`,
    - deterministic ordering expectations (e.g., `triggers` sorted, `missing_inputs` sorted),
    - what belongs in `derived` (audit/debug; no unstable host-specific paths).
  - Keep “additive-only” rule explicit and tie it back to `CONTRACT-3` registry text.
- **Acceptance criteria**:
  - A downstream consumer can implement parsing with no ambiguity and without consulting `pm_lift.py`.
- **Test notes**:
  - N/A (spec doc), but referenced by S1.T3 tests as the source of truth.
- **Risk/rollback notes**:
  - If “confidence” needs more states later, add them without breaking existing values (do not remove `low/high`).

Checklist:
- Implement:
  - Draft required keys + types + semantics.
  - Add at least one example JSON output.
- Validate:
  - Cross-check against `work_lift_v1_seams/threading.md` CONTRACT-3 language.
  - Cleanup:
    - Keep spec colocated with the script for discoverability.

##### CONTRACT-3: `pm_lift.py ... --emit-json` concrete contract (v1, normative)

When `--emit-json` is provided, `pm_lift.py` MUST:

- write **JSON only** to stdout,
- write nothing to stdout on any non-zero exit,
- write all warnings/errors to stderr (no JSON on stderr),
- exit `0` on success,
- exit non-zero on failure (usage errors exit `2`; runtime/validation errors exit `1`).

The stdout JSON MUST conform to this v1 schema (pseudo JSON Schema; types are normative):

```json
{
  "model_version": "integer (>=1)",
  "lift_score": "integer (>=0)",
  "estimated_slices": "integer (>=1)",
  "confidence": "\"high\" | \"low\"",
  "triggers": "array<string> (sorted asc, unique)",
  "missing_inputs": "array<string> (sorted asc, unique)",
  "vector": "object (the input Lift Vector, see CONTRACT-1)",
  "derived": "object (debug/audit; additive keys allowed)"
}
```

Stable semantics (v1):

- `confidence`:
  - `low` iff `missing_inputs` is non-empty OR the derived-pack input indicates prefix entries (`dir_prefixes` non-empty from CONTRACT-4).
  - `high` otherwise.
- `missing_inputs` entries are Lift Vector JSON paths, e.g. `touch.crates_touched`.
- For every `missing_inputs` entry `X`, `triggers` MUST contain `missing_inputs:X`.
- `triggers` contains only machine tokens (no prose). V1 tokens are:
  - `split_required:behavior_deltas>1`
  - `likely_split:crates_touched>2`
  - `likely_split:touch_files_sum>12`
  - `likely_split:contract_surface_sum>4`
  - `likely_split:lift_score>24`
  - `split_required:estimated_slices>3`
  - `touch_set_contains_prefix_entries`
  - plus any number of `missing_inputs:<json_path>` entries.
- `derived` MUST include:
  - `base_points` (number)
  - `risk_multiplier` (number)
  - For `from-impact-map` mode only:
    - `impact_map_touch_counts` (object)
    - `touch_effective_for_scoring` (object)
  - Additional `derived` keys are allowed and MUST be treated as non-breaking additive evolution.

#### S1.T2 — Normalize `pm_lift.py --emit-json` behavior to match CONTRACT-3

- **Outcome**: The tool emits a stable JSON contract for all supported inputs, with deterministic ordering and machine-friendly values.
- **Inputs/outputs**:
  - Inputs:
    - Existing `pm_lift.py` emit logic
    - CONTRACT-3 spec from S1.T1
  - Output:
    - Updated `pm_lift.py` output emission and error handling to meet the CONTRACT-3 requirements exactly (required keys/types/enums/exit codes; additive-only evolution).
- **Implementation notes**:
  - Enforce determinism:
    - sort `triggers`/`missing_inputs`,
    - ensure `vector` and `derived` remain JSON-serializable (no Path objects, no floats as NaN/Inf).
  - Keep stdout JSON-only for `--emit-json`; send warnings/errors to stderr.
  - Ensure missing inputs never crash: `null` becomes confidence downgrade + missing_inputs entry (per seam brief).
- **Acceptance criteria**:
  - `--emit-json` output is contract-conformant and stable for the same input.
  - On error, stdout is empty (or at minimum not “partial JSON”) and stderr is actionable.
- **Test notes**:
  - Covered by S1.T3 contract conformance test and S3 negative cases.
- **Risk/rollback notes**:
  - If behavior changes are risky, gate with a temporary internal flag, but do not ship two competing JSON shapes.

Checklist:
- Implement:
  - Align output keys and ordering.
  - Standardize error paths for common invalid inputs (missing markers, invalid JSON).
- Test:
  - Manual: run each subcommand with `--emit-json` and confirm JSON-only stdout.
- Cleanup:
  - Keep the contract-labeled fields stable; do not rename keys.

#### S1.T3 — Add a minimal contract conformance test for CONTRACT-3

- **Outcome**: A fast test that fails on accidental contract drift (missing keys, wrong types, non-deterministic ordering).
- **Inputs/outputs**:
  - Inputs:
    - `pm_lift.py`
    - A minimal fixture input (either a tiny markdown with a Lift Vector block, or a mocked `from-git-diff` call)
  - Outputs:
    - New test file under `docs/project_management/system/scripts/planning/tests/` asserting CONTRACT-3 keys + types.
- **Implementation notes**:
  - Prefer a fixture that does not depend on repo history (avoid brittle `git-range` assumptions).
  - Assert:
    - required keys exist,
    - types are correct,
    - `triggers` and `missing_inputs` are arrays of strings,
    - output is parseable JSON and includes no extra stdout noise.
- **Acceptance criteria**:
  - Test fails if any required key is removed/renamed or if output becomes non-JSON for `--emit-json`.
- **Test notes**:
  - Runs as part of the existing python test suite invocation used in planning scripts.
- **Risk/rollback notes**:
  - Keep assertions tolerant of additive keys: only require the known-required keys and type-shape.

Checklist:
- Implement:
  - Add fixture + test harness (subprocess call).
- Validate:
  - Run the test locally and confirm stability across runs.
- Cleanup:
  - Keep the fixture minimal and human-readable.
