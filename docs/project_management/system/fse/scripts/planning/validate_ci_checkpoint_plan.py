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


def _extract_json_block(text: str, *, accept_draft_header: bool = False) -> dict[str, Any]:
    """
    Extract the single JSON code block under the machine-readable plan section.

    The FSE lane accepts either a linted or draft header because the checkpoint
    plan is advisory during pre-planning.
    """
    headers = [LINTED_HEADER]
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
        if accept_draft_header:
            allowed = " or ".join(repr(header) for header in headers)
            _fail(f"ci_checkpoint_plan.md missing required header: {allowed}")
        _fail(f"ci_checkpoint_plan.md missing required header: {LINTED_HEADER!r}")

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


def _parse_defaults(defaults: dict[str, Any]) -> dict[str, int]:
    min_slices = defaults.get("min_candidates_per_checkpoint")
    max_slices = defaults.get("max_candidates_per_checkpoint")

    if min_slices is None:
        min_slices = defaults.get("min_slices_per_checkpoint")
    if max_slices is None:
        max_slices = defaults.get("max_slices_per_checkpoint")

    if min_slices is None:
        min_slices = defaults.get("min_triads_per_checkpoint")
    if max_slices is None:
        max_slices = defaults.get("max_triads_per_checkpoint")

    if not isinstance(min_slices, int) or min_slices < 1:
        _fail(
            "ci_checkpoint_plan.md JSON: defaults.min_candidates_per_checkpoint "
            "(or legacy min_slices_per_checkpoint / min_triads_per_checkpoint) must be an int >= 1"
        )
    if not isinstance(max_slices, int) or max_slices < min_slices:
        _fail(
            "ci_checkpoint_plan.md JSON: defaults.max_candidates_per_checkpoint "
            "(or legacy max_slices_per_checkpoint / max_triads_per_checkpoint) must be an int >= min"
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
        slices = raw.get("candidate_ids")
        if slices is None:
            slices = raw.get("slices")
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

        if not isinstance(slices, list) or not all(isinstance(s, str) and s.strip() for s in slices):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].candidate_ids must be an array of non-empty strings")
        normalized_slices = [s.strip() for s in slices]
        if len(set(normalized_slices)) != len(normalized_slices):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].candidate_ids contains duplicates")

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
        _fail(f"ci_checkpoint_plan.md assigns slices to multiple checkpoints: {', '.join(duplicates)}")

    expected_slice_order, authority_path = _expected_slice_order_from_authority(feature_dir, workstream_triage)
    if expected_slice_order:
        expected_index = {slice_id: idx for idx, slice_id in enumerate(expected_slice_order)}
        extra = sorted(set(flattened_slices) - set(expected_slice_order), key=_slice_sort_key)
        if extra:
            authority_note = f" from accepted slice order {authority_path}" if authority_path else ""
            _fail(
                "ci_checkpoint_plan.md references slices outside the accepted slice authority"
                f"{authority_note}: {extra}"
            )
        actual_positions = [expected_index[slice_id] for slice_id in flattened_slices]
        if actual_positions != sorted(actual_positions):
            authority_note = f" from accepted slice order {authority_path}" if authority_path else ""
            _fail(
                "ci_checkpoint_plan.md must preserve the relative order of slices from accepted slice authority"
                f"{authority_note}; got {flattened_slices}"
            )

    total = len(flattened_slices)
    min_n = defaults["min"]
    max_n = defaults["max"]
    for checkpoint in checkpoints:
        count = len(checkpoint.slices)
        if count > max_n:
            _fail(f"ci_checkpoint_plan.md checkpoint {checkpoint.checkpoint_id!r} has {count} slices; max is {max_n}")
        if total >= min_n and count < min_n:
            _fail(
                f"ci_checkpoint_plan.md checkpoint {checkpoint.checkpoint_id!r} has {count} slices; "
                f"min is {min_n} (total slices={total})"
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
