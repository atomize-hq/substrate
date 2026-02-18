#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class Checkpoint:
    checkpoint_id: str
    task_id: str
    slices: list[str]
    gates: dict[str, Any]
    rationale: str


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def _read_tasks_json(feature_dir: Path) -> dict[str, Any]:
    tasks_path = feature_dir / "tasks.json"
    if not tasks_path.exists():
        _fail(f"missing tasks.json: {tasks_path}")
    try:
        return json.loads(tasks_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        _fail(f"tasks.json is not valid JSON: {e}")
    return {}


def _extract_json_block(text: str) -> dict[str, Any]:
    """
    Extract the single JSON code block under '## Machine-readable plan (linted)'.
    """
    header = "## Machine-readable plan (linted)"
    start = text.find(header)
    if start < 0:
        _fail(f"ci_checkpoint_plan.md missing required header: {header!r}")

    remainder = text[start:]
    m = re.search(r"```json\s*\n(?P<body>[\s\S]*?)\n```", remainder)
    if not m:
        _fail("ci_checkpoint_plan.md missing a ```json code block under 'Machine-readable plan'")

    body = m.group("body").strip()
    try:
        data = json.loads(body)
    except json.JSONDecodeError as e:
        _fail(f"ci_checkpoint_plan.md JSON block is not valid JSON: {e}")
    if not isinstance(data, dict):
        _fail("ci_checkpoint_plan.md JSON block must be a JSON object")
    return data


def _slice_ids_from_tasks(tasks_data: dict[str, Any]) -> list[str]:
    tasks = tasks_data.get("tasks")
    if not isinstance(tasks, list):
        _fail("tasks.json must contain top-level tasks[] array")

    slice_ids: set[str] = set()
    for t in tasks:
        if not isinstance(t, dict):
            continue
        task_id = t.get("id")
        task_type = t.get("type")
        if task_type != "integration":
            continue
        if isinstance(task_id, str) and task_id.endswith("-integ") and not task_id.endswith("-integ-core"):
            slice_ids.add(task_id[: -len("-integ")])

    return sorted(slice_ids, key=_slice_sort_key)


def _slice_sort_key(slice_id: str) -> tuple[str, int, str]:
    """
    Deterministic slice ordering.

    Typical slice ids are PREFIX + NUMBER (e.g. WCU0, WCU10). Lexicographic sorting breaks that.
    """
    m = re.match(r"^(?P<prefix>[A-Za-z][A-Za-z0-9]*?)(?P<num>\d+)$", slice_id)
    if not m:
        return (slice_id, 0, slice_id)
    return (m.group("prefix"), int(m.group("num")), slice_id)


def _parse_checkpoints(plan: dict[str, Any]) -> tuple[dict[str, int], list[Checkpoint]]:
    version = plan.get("version")
    if version != 1:
        _fail("ci_checkpoint_plan.md JSON: version must be 1")

    defaults = plan.get("defaults")
    if not isinstance(defaults, dict):
        _fail("ci_checkpoint_plan.md JSON: defaults must be an object")
    min_n = defaults.get("min_triads_per_checkpoint")
    max_n = defaults.get("max_triads_per_checkpoint")
    if not isinstance(min_n, int) or min_n < 1:
        _fail("ci_checkpoint_plan.md JSON: defaults.min_triads_per_checkpoint must be an int >= 1")
    if not isinstance(max_n, int) or max_n < min_n:
        _fail("ci_checkpoint_plan.md JSON: defaults.max_triads_per_checkpoint must be an int >= min")

    checkpoints_raw = plan.get("checkpoints")
    if not isinstance(checkpoints_raw, list) or not checkpoints_raw:
        _fail("ci_checkpoint_plan.md JSON: checkpoints must be a non-empty array")

    checkpoints: list[Checkpoint] = []
    for i, raw in enumerate(checkpoints_raw):
        if not isinstance(raw, dict):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}] must be an object")
        cid = raw.get("id")
        task_id = raw.get("task_id")
        slices = raw.get("slices")
        gates = raw.get("gates")
        rationale = raw.get("rationale")
        if not isinstance(cid, str) or not cid:
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].id must be a non-empty string")
        if not isinstance(task_id, str) or not task_id:
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].task_id must be a non-empty string")
        if not isinstance(slices, list) or not all(isinstance(s, str) and s for s in slices):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].slices must be an array of non-empty strings")
        if len(set(slices)) != len(slices):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].slices contains duplicates")
        if not isinstance(gates, dict):
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].gates must be an object")
        if not isinstance(rationale, str) or not rationale.strip():
            _fail(f"ci_checkpoint_plan.md JSON: checkpoints[{i}].rationale must be a non-empty string")

        checkpoints.append(
            Checkpoint(
                checkpoint_id=cid,
                task_id=task_id,
                slices=slices,
                gates=gates,
                rationale=rationale.strip(),
            )
        )

    # Ensure task ids are unique (no ambiguity).
    task_ids = [c.task_id for c in checkpoints]
    if len(set(task_ids)) != len(task_ids):
        _fail("ci_checkpoint_plan.md JSON: checkpoints[].task_id must be unique")

    return {"min": min_n, "max": max_n}, checkpoints


def _validate_against_tasks(feature_dir: Path, tasks_data: dict[str, Any], defaults: dict[str, int], checkpoints: list[Checkpoint]) -> None:
    slice_ids = _slice_ids_from_tasks(tasks_data)
    if not slice_ids:
        _fail("tasks.json does not contain any slice final integration tasks (*-integ); cannot validate checkpoint coverage")

    meta = tasks_data.get("meta") if isinstance(tasks_data, dict) else None
    if not isinstance(meta, dict):
        meta = {}
    schema_version = meta.get("schema_version", 1)
    cross_platform = meta.get("cross_platform") is True

    slices_in_plan: list[str] = []
    for c in checkpoints:
        slices_in_plan.extend(c.slices)

    # Ordering / contiguity: the plan must define a single ordered partition of the slice list.
    # This removes ambiguity and enables mechanical gating checks between checkpoint groups.
    if slices_in_plan != slice_ids:
        _fail(
            "ci_checkpoint_plan.md must assign slices in deterministic order and as contiguous groups; "
            f"expected slice order {slice_ids}, got {slices_in_plan}"
        )

    # Coverage: every slice must belong to exactly one checkpoint.
    duplicates = sorted({s for s in slices_in_plan if slices_in_plan.count(s) > 1})
    if duplicates:
        _fail(f"ci_checkpoint_plan.md assigns slices to multiple checkpoints: {', '.join(duplicates)}")

    missing = sorted(set(slice_ids) - set(slices_in_plan))
    extra = sorted(set(slices_in_plan) - set(slice_ids))
    if missing:
        _fail(f"ci_checkpoint_plan.md missing slices present in tasks.json: {', '.join(missing)}")
    if extra:
        _fail(f"ci_checkpoint_plan.md references slices not present in tasks.json: {', '.join(extra)}")

    # Schema v4 cross-platform packs require tasks.json meta.checkpoint_boundaries to match the
    # checkpoint boundaries (the last slice of each checkpoint group) exactly.
    if cross_platform and isinstance(schema_version, int) and schema_version >= 4:
        boundaries = meta.get("checkpoint_boundaries")
        if not isinstance(boundaries, list) or not all(isinstance(x, str) and x for x in boundaries):
            _fail("tasks.json meta.checkpoint_boundaries must be an array of non-empty strings for schema v4 cross-platform packs")
        if len(set(boundaries)) != len(boundaries):
            _fail("tasks.json meta.checkpoint_boundaries contains duplicates")
        expected_boundaries = [c.slices[-1] for c in checkpoints]
        if boundaries != expected_boundaries:
            _fail(
                "tasks.json meta.checkpoint_boundaries must match the checkpoint group boundaries in ci_checkpoint_plan.md "
                f"(expected {expected_boundaries}, got {boundaries})"
            )

    # Bounds (default): enforce min/max per checkpoint, except when total slices < min.
    total = len(slice_ids)
    min_n = defaults["min"]
    max_n = defaults["max"]
    for c in checkpoints:
        n = len(c.slices)
        if n > max_n:
            _fail(f"ci_checkpoint_plan.md checkpoint {c.checkpoint_id!r} has {n} slices; max is {max_n}")
        if total >= min_n and n < min_n:
            _fail(f"ci_checkpoint_plan.md checkpoint {c.checkpoint_id!r} has {n} slices; min is {min_n} (total slices={total})")

    # Task existence and shape.
    tasks = tasks_data.get("tasks")
    tasks_by_id = {t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)}
    feature_dir_prefix = feature_dir.as_posix().rstrip("/") + "/"
    for c in checkpoints:
        t = tasks_by_id.get(c.task_id)
        if t is None:
            _fail(f"ci_checkpoint_plan.md references missing checkpoint task id in tasks.json: {c.task_id!r}")
        if t.get("type") != "ops":
            _fail(f"checkpoint task {c.task_id!r} must have type='ops'")
        kickoff = t.get("kickoff_prompt")
        if not isinstance(kickoff, str) or not kickoff:
            _fail(f"checkpoint task {c.task_id!r} must have kickoff_prompt set")
        kickoff_path = Path(kickoff)
        if not kickoff_path.is_absolute():
            kickoff_norm = kickoff.replace("\\", "/").lstrip("./")
            # tasks.json commonly stores repo-root-relative paths (e.g., docs/...).
            if kickoff_norm.startswith("docs/") or kickoff_norm.startswith(feature_dir_prefix):
                kickoff_path = Path(kickoff_norm)
            else:
                kickoff_path = feature_dir / kickoff_norm
        if not kickoff_path.exists():
            _fail(f"checkpoint task {c.task_id!r} kickoff_prompt file does not exist: {kickoff_path}")

        # Dependency wiring: checkpoint must depend on the core integration task of its ending slice.
        last_slice = c.slices[-1]
        expected_dep = f"{last_slice}-integ-core"
        deps = t.get("depends_on")
        if not isinstance(deps, list) or expected_dep not in deps:
            _fail(f"checkpoint task {c.task_id!r} must depend_on {expected_dep!r}")

    # Gating: the first slice of the next checkpoint group must depend on the prior checkpoint task.
    # This prevents starting work past a checkpoint until the cross-platform CI gate is complete.
    for i in range(len(checkpoints) - 1):
        cp = checkpoints[i]
        next_cp = checkpoints[i + 1]
        next_first_slice = next_cp.slices[0]

        for task_suffix in ("code", "test"):
            tid = f"{next_first_slice}-{task_suffix}"
            t = tasks_by_id.get(tid)
            if t is None:
                _fail(f"tasks.json missing required task for gating check: {tid!r}")
            deps = t.get("depends_on")
            if not isinstance(deps, list) or cp.task_id not in deps:
                _fail(f"{tid!r} must depend_on prior checkpoint task {cp.task_id!r}")


def main() -> int:
    ap = argparse.ArgumentParser(description="Validate ci_checkpoint_plan.md against tasks.json.")
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    ap.add_argument(
        "--ci-checkpoint-plan",
        default="ci_checkpoint_plan.md",
        help="Path to ci_checkpoint_plan.md (absolute or feature-dir-relative). Default: ci_checkpoint_plan.md",
    )
    args = ap.parse_args()

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists():
        _fail(f"feature dir does not exist: {feature_dir}")

    plan_path = Path(args.ci_checkpoint_plan)
    if not plan_path.is_absolute():
        plan_path = feature_dir / plan_path
    if not plan_path.exists():
        _fail(f"missing ci checkpoint plan: {plan_path}")

    text = plan_path.read_text(encoding="utf-8")
    plan = _extract_json_block(text)
    defaults, checkpoints = _parse_checkpoints(plan)
    tasks_data = _read_tasks_json(feature_dir)
    _validate_against_tasks(feature_dir, tasks_data, defaults, checkpoints)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
