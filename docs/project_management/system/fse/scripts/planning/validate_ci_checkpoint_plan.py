#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import validate_pws_index as vpi


@dataclass(frozen=True)
class Checkpoint:
    checkpoint_id: str
    task_id: str | None
    slices: list[str]
    gates: dict[str, Any]
    rationale: str


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


LINTED_HEADER = "## Machine-readable plan (linted)"
DRAFT_HEADER = "## Machine-readable plan (draft; not yet mechanically validated)"
PLAIN_HEADER = "## Machine-readable plan"

DEFAULT_MIN_FIELD_ALIASES = (
    "min_draft_seams_per_checkpoint",
    "min_seams_per_checkpoint",
    "min_candidates_per_checkpoint",
    "min_slices_per_checkpoint",
    "min_triads_per_checkpoint",
)
DEFAULT_MAX_FIELD_ALIASES = (
    "max_draft_seams_per_checkpoint",
    "max_seams_per_checkpoint",
    "max_candidates_per_checkpoint",
    "max_slices_per_checkpoint",
    "max_triads_per_checkpoint",
)
CHECKPOINT_ID_FIELD_ALIASES = (
    "draft_seam_ids",
    "seam_ids",
    "candidate_ids",
    "slices",
    "triad_ids",
)


def _extract_json_block(text: str, *, accept_draft_header: bool = False) -> dict[str, Any]:
    """
    Extract the single JSON code block under the machine-readable plan section.

    The FSE lane accepts either a linted or draft header because the checkpoint
    plan is advisory during pre-planning.
    """
    headers = [LINTED_HEADER, PLAIN_HEADER]
    if accept_draft_header:
        headers.append(DRAFT_HEADER)

    start = -1
    matched_header: str | None = None
    for header in headers:
        header_start = text.find(header)
        if header_start >= 0 and (start < 0 or header_start < start):
            start = header_start
            matched_header = header
    if start < 0:
        allowed = " or ".join(repr(header) for header in headers)
        _fail(f"ci_checkpoint_plan.md missing required header: {allowed}")

    remainder = text[start:]
    m = re.search(r"```json\s*\n(?P<body>[\s\S]*?)\n```", remainder)
    if not m:
        label = matched_header or "Machine-readable plan"
        _fail(f"ci_checkpoint_plan.md missing a ```json code block under {label!r}")

    body = m.group("body").strip()
    try:
        data = json.loads(body)
    except json.JSONDecodeError as e:
        _fail(f"ci_checkpoint_plan.md JSON block is not valid JSON: {e}")
    if not isinstance(data, dict):
        _fail("ci_checkpoint_plan.md JSON block must be a JSON object")
    return data


def _slice_sort_key(slice_id: str) -> tuple[str, int, str]:
    match = re.match(r"^(?P<prefix>[A-Za-z][A-Za-z0-9]*?)(?P<num>\d+)$", slice_id)
    if match is None:
        return (slice_id, 0, slice_id)
    return (match.group("prefix"), int(match.group("num")), slice_id)


def _resolve_int_alias(
    defaults: dict[str, Any],
    aliases: tuple[str, ...],
) -> tuple[Any, list[str]]:
    present: list[str] = []
    value: Any = None
    seen_value = False
    for field_name in aliases:
        if field_name not in defaults:
            continue
        field_value = defaults.get(field_name)
        if field_value is None:
            continue
        present.append(field_name)
        if not seen_value:
            value = field_value
            seen_value = True
            continue
        if field_value != value:
            joined = ", ".join(present)
            _fail(
                "ci_checkpoint_plan.md JSON: conflicting checkpoint-size defaults across aliases "
                f"({joined}); got {value!r} vs {field_value!r}"
            )
    return (value, present)


def _resolve_checkpoint_id_alias(
    raw: dict[str, Any],
    *,
    checkpoint_index: int,
) -> list[str]:
    selected: list[str] | None = None
    selected_field: str | None = None
    present_fields: list[str] = []

    for field_name in CHECKPOINT_ID_FIELD_ALIASES:
        if field_name not in raw:
            continue
        value = raw.get(field_name)
        if value is None:
            continue
        present_fields.append(field_name)
        if not isinstance(value, list) or not all(isinstance(s, str) and s.strip() for s in value):
            _fail(
                "ci_checkpoint_plan.md JSON: "
                f"checkpoints[{checkpoint_index}].{field_name} must be an array of non-empty seam ids "
                "(legacy candidate_ids / slices / triad_ids still accepted)"
            )
        normalized = [s.strip() for s in value]
        if selected is None:
            selected = normalized
            selected_field = field_name
            continue
        if normalized != selected:
            joined = ", ".join(present_fields)
            _fail(
                "ci_checkpoint_plan.md JSON: "
                f"checkpoints[{checkpoint_index}] defines conflicting seam-group aliases ({joined}); "
                f"{selected_field}={selected!r} but {field_name}={normalized!r}"
            )

    if selected is None:
        preferred = CHECKPOINT_ID_FIELD_ALIASES[0]
        _fail(
            "ci_checkpoint_plan.md JSON: "
            f"checkpoints[{checkpoint_index}].{preferred} must be an array of non-empty seam ids "
            "(legacy seam_ids / candidate_ids / slices / triad_ids still accepted)"
        )

    return selected


def _parse_defaults(defaults: dict[str, Any]) -> dict[str, int]:
    min_slices, _ = _resolve_int_alias(defaults, DEFAULT_MIN_FIELD_ALIASES)
    max_slices, _ = _resolve_int_alias(defaults, DEFAULT_MAX_FIELD_ALIASES)

    if not isinstance(min_slices, int) or min_slices < 1:
        _fail(
            "ci_checkpoint_plan.md JSON: defaults.min_draft_seams_per_checkpoint "
            "(or defaults.min_seams_per_checkpoint; legacy min_candidates_per_checkpoint / "
            "min_slices_per_checkpoint / min_triads_per_checkpoint) must be an int >= 1"
        )
    if not isinstance(max_slices, int) or max_slices < min_slices:
        _fail(
            "ci_checkpoint_plan.md JSON: defaults.max_draft_seams_per_checkpoint "
            "(or defaults.max_seams_per_checkpoint; legacy max_candidates_per_checkpoint / "
            "max_slices_per_checkpoint / max_triads_per_checkpoint) must be an int >= min"
        )
    return {"min": min_slices, "max": max_slices}


def _parse_checkpoints(plan: dict[str, Any]) -> tuple[dict[str, int], list[Checkpoint]]:
    version = plan.get("version")
    if version != 1:
        _fail("ci_checkpoint_plan.md JSON: version must be 1")

    defaults = plan.get("defaults")
    if not isinstance(defaults, dict):
        _fail("ci_checkpoint_plan.md JSON: defaults must be an object")
    parsed_defaults = _parse_defaults(defaults)

    checkpoints_raw = plan.get("checkpoints")
    if not isinstance(checkpoints_raw, list) or not checkpoints_raw:
        _fail("ci_checkpoint_plan.md JSON: checkpoints must be a non-empty array")

    checkpoints: list[Checkpoint] = []
    checkpoint_ids: set[str] = set()
    task_ids: set[str] = set()
    for i, raw in enumerate(checkpoints_raw):
        if not isinstance(raw, dict):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}] must be an object")

        checkpoint_id = raw.get("checkpoint_id")
        if checkpoint_id is None:
            checkpoint_id = raw.get("id")
        task_id = raw.get("task_id")
        gates = raw.get("gates")
        rationale = raw.get("rationale")

        if not isinstance(checkpoint_id, str) or not checkpoint_id.strip():
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].checkpoint_id must be a non-empty string")
        checkpoint_id = checkpoint_id.strip()
        if checkpoint_id in checkpoint_ids:
            _fail(f"ci_checkpoint_plan.md JSON: duplicate checkpoint id {checkpoint_id!r}")
        checkpoint_ids.add(checkpoint_id)

        if task_id is not None:
            if not isinstance(task_id, str) or not task_id.strip():
                _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].task_id must be a non-empty string when present")
            task_id = task_id.strip()
            if task_id in task_ids:
                _fail(f"ci_checkpoint_plan.md JSON: checkpoints[].task_id must be unique ({task_id!r})")
            task_ids.add(task_id)

        normalized_slices = _resolve_checkpoint_id_alias(raw, checkpoint_index=i)
        if len(set(normalized_slices)) != len(normalized_slices):
            _fail(
                "ci_checkpoint_plan.md JSON: "
                f"checkpoints[{i}] seam ids contain duplicates "
                "(preferred draft_seam_ids; legacy candidate_ids / slices / triad_ids also accepted)"
            )

        if not isinstance(gates, dict):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].gates must be an object")
        for key, value in gates.items():
            if key in {"compile_parity", "feature_smoke"} and not isinstance(value, bool):
                _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].gates.{key} must be a boolean")
            if key == "ci_testing" and (not isinstance(value, str) or not value.strip()):
                _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].gates.ci_testing must be a non-empty string")

        if not isinstance(rationale, str) or not rationale.strip():
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].rationale must be a non-empty string")

        checkpoints.append(
            Checkpoint(
                checkpoint_id=checkpoint_id,
                task_id=task_id,
                slices=normalized_slices,
                gates=gates,
                rationale=rationale.strip(),
            )
        )

    return parsed_defaults, checkpoints


def _expected_slice_order_from_authority(
    feature_dir: Path,
    workstream_triage: str,
) -> tuple[list[str], str | None]:
    triage_path, authority, errors = vpi._load_slice_authority(
        feature_dir,
        workstream_triage,
        advisory=True,
        require_v2=False,
    )
    if triage_path is not None and errors:
        _fail(f"invalid workstream triage slice authority: {errors[0].message}")
    if authority is None:
        return ([], None)
    return (authority.accepted_slice_order, str(triage_path))


def _validate_against_authority(
    feature_dir: Path,
    defaults: dict[str, int],
    checkpoints: list[Checkpoint],
    workstream_triage: str,
) -> None:
    flattened_slices: list[str] = []
    for checkpoint in checkpoints:
        flattened_slices.extend(checkpoint.slices)

    duplicates = sorted({slice_id for slice_id in flattened_slices if flattened_slices.count(slice_id) > 1}, key=_slice_sort_key)
    if duplicates:
        _fail(f"ci_checkpoint_plan.md assigns seam ids to multiple checkpoints: {', '.join(duplicates)}")

    expected_slice_order, authority_path = _expected_slice_order_from_authority(feature_dir, workstream_triage)
    if expected_slice_order:
        expected_index = {slice_id: idx for idx, slice_id in enumerate(expected_slice_order)}
        extra = sorted(set(flattened_slices) - set(expected_slice_order), key=_slice_sort_key)
        if extra:
            authority_note = f" from accepted seam/slice order {authority_path}" if authority_path else ""
            _fail(
                "ci_checkpoint_plan.md references seam ids outside the accepted seam/slice authority"
                f"{authority_note}: {extra}"
            )
        actual_positions = [expected_index[slice_id] for slice_id in flattened_slices]
        if actual_positions != sorted(actual_positions):
            authority_note = f" from accepted seam/slice order {authority_path}" if authority_path else ""
            _fail(
                "ci_checkpoint_plan.md must preserve the relative order of seam ids from accepted seam/slice authority"
                f"{authority_note}; got {flattened_slices}"
            )

    total = len(flattened_slices)
    min_n = defaults["min"]
    max_n = defaults["max"]
    for checkpoint in checkpoints:
        count = len(checkpoint.slices)
        if count > max_n:
            _fail(f"ci_checkpoint_plan.md checkpoint {checkpoint.checkpoint_id!r} has {count} seam ids; max is {max_n}")
        if total >= min_n and count < min_n:
            _fail(
                f"ci_checkpoint_plan.md checkpoint {checkpoint.checkpoint_id!r} has {count} seam ids; "
                f"min is {min_n} (total seam ids={total})"
            )


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Validate ci_checkpoint_plan.md as advisory FSE checkpoint intent.",
    )
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    ap.add_argument(
        "--workstream-triage",
        default=vpi.DEFAULT_TRIAGE_REL,
        help=(
            "Path to workstream_triage.md (absolute or feature-dir-relative). "
            f"Default: {vpi.DEFAULT_TRIAGE_REL} (legacy fallback: {vpi.LEGACY_TRIAGE_REL})"
        ),
    )
    ap.add_argument(
        "--ci-checkpoint-plan",
        default="pre-planning/ci_checkpoint_plan.md",
        help="Path to ci_checkpoint_plan.md (absolute or feature-dir-relative). Default: pre-planning/ci_checkpoint_plan.md",
    )
    args = ap.parse_args()

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists():
        _fail(f"feature dir does not exist: {feature_dir}")

    plan_path = Path(args.ci_checkpoint_plan)
    if not plan_path.is_absolute():
        plan_path = feature_dir / plan_path
    if not plan_path.exists():
        if args.ci_checkpoint_plan == "pre-planning/ci_checkpoint_plan.md":
            legacy = feature_dir / "ci_checkpoint_plan.md"
            if legacy.exists():
                plan_path = legacy
            else:
                _fail(f"missing ci checkpoint plan: {plan_path} (also missing legacy: {legacy})")
        else:
            _fail(f"missing ci checkpoint plan: {plan_path}")

    text = plan_path.read_text(encoding="utf-8")
    plan = _extract_json_block(text, accept_draft_header=True)
    defaults, checkpoints = _parse_checkpoints(plan)
    _validate_against_authority(feature_dir, defaults, checkpoints, args.workstream_triage)
    print(f"OK: ci_checkpoint_plan validation passed: {plan_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
