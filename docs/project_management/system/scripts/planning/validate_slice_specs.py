#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


V2_REQUIRED_HEADERS = [
    "## Behavior delta (single)",
    "## Scope",
    "## Behavior (authoritative)",
    "## Acceptance criteria",
    "## Out of scope",
]

FORBIDDEN_PLACEHOLDERS = [
    "None yet.",
    "TBD",
    "TODO",
    "WIP",
    "TBA",
    "[[FILL]]",
]


@dataclass(frozen=True)
class SliceTasks:
    slice_id: str
    code: dict[str, Any] | None
    test: dict[str, Any] | None
    integ: dict[str, Any] | None


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


def _as_str_list(value: Any) -> list[str]:
    if isinstance(value, list) and all(isinstance(x, str) for x in value):
        return value
    return []


def _normalize_ref_path(raw: str) -> str:
    # Normalize to forward slashes for matching and prefix checks.
    return raw.replace("\\", "/").lstrip("./")


def _resolve_reference_path(feature_dir: Path, candidate: str) -> Path:
    """
    Resolve a task reference into a filesystem path.

    Matches existing repo conventions (see validate_spec_manifest.py):
    - absolute paths are used as-is
    - repo-root-relative paths typically start with docs/ or with the feature_dir prefix
    - other paths are treated as feature-dir-relative
    """
    p = Path(candidate)
    if p.is_absolute():
        return p

    raw_norm = _normalize_ref_path(candidate)
    feature_dir_prefix = feature_dir.as_posix().rstrip("/") + "/"
    if raw_norm.startswith("docs/") or raw_norm.startswith(feature_dir_prefix) or raw_norm == feature_dir.as_posix():
        return Path(raw_norm)

    return feature_dir / raw_norm


def _derive_slices(tasks_data: dict[str, Any]) -> dict[str, SliceTasks]:
    tasks = tasks_data.get("tasks")
    if not isinstance(tasks, list):
        _fail("tasks.json must contain top-level tasks[] array")

    tasks_by_id: dict[str, dict[str, Any]] = {
        t.get("id"): t for t in tasks if isinstance(t, dict) and isinstance(t.get("id"), str)
    }

    slice_ids: set[str] = set()
    for t in tasks_by_id.values():
        task_id = t.get("id")
        task_type = t.get("type")
        if not isinstance(task_id, str) or not isinstance(task_type, str):
            continue
        if task_type == "code" and task_id.endswith("-code"):
            slice_ids.add(task_id[: -len("-code")])
        if task_type == "test" and task_id.endswith("-test"):
            slice_ids.add(task_id[: -len("-test")])

    result: dict[str, SliceTasks] = {}
    for slice_id in sorted(slice_ids):
        result[slice_id] = SliceTasks(
            slice_id=slice_id,
            code=tasks_by_id.get(f"{slice_id}-code"),
            test=tasks_by_id.get(f"{slice_id}-test"),
            integ=tasks_by_id.get(f"{slice_id}-integ"),
        )
    return result


def _find_slice_spec_path(feature_dir: Path, slice_tasks: SliceTasks) -> Path:
    target_basename = f"{slice_tasks.slice_id}-spec.md"

    candidates: list[str] = []
    for t in (slice_tasks.code, slice_tasks.test):
        if not isinstance(t, dict):
            continue
        for ref in _as_str_list(t.get("references")):
            norm = _normalize_ref_path(ref)
            if Path(norm).name == target_basename:
                candidates.append(ref)

    for c in candidates:
        p = _resolve_reference_path(feature_dir, c)
        if p.exists():
            return p

    fallback = feature_dir / target_basename
    return fallback


def _iter_non_fenced_lines(lines: list[str]) -> Iterable[tuple[int, str]]:
    in_fence = False
    for idx, line in enumerate(lines, start=1):
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        yield (idx, line)


def _collect_h2_headers(lines: list[str]) -> list[tuple[int, str]]:
    headers: list[tuple[int, str]] = []
    for idx, line in _iter_non_fenced_lines(lines):
        s = line.strip("\n")
        if s.startswith("## "):
            headers.append((idx, s.strip()))
    return headers


def _extract_section(lines: list[str], header: str) -> tuple[list[tuple[int, str]], int]:
    """
    Returns (section_lines, header_line_number).
    section_lines includes lines after the header until the next H2 header or EOF (excluding fenced blocks).
    """
    headers = _collect_h2_headers(lines)
    header_line = None
    for lineno, text in headers:
        if text == header:
            header_line = lineno
            break
    if header_line is None:
        return ([], -1)

    # Section bounds: from header_line+1 to next header (exclusive).
    next_header_line = None
    for lineno, _ in headers:
        if lineno > header_line:
            next_header_line = lineno
            break

    section: list[tuple[int, str]] = []
    in_fence = False
    for idx, line in enumerate(lines, start=1):
        if idx <= header_line:
            continue
        if next_header_line is not None and idx >= next_header_line:
            break
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        section.append((idx, line))

    return (section, header_line)


def _scan_forbidden_placeholders(
    section_name: str, section_lines: list[tuple[int, str]], path: Path, slice_id: str
) -> None:
    for lineno, line in section_lines:
        for token in FORBIDDEN_PLACEHOLDERS:
            if token in ("TBD", "TODO", "WIP", "TBA"):
                if re.search(rf"\b{re.escape(token)}\b", line):
                    _fail(
                        f"{slice_id} spec contains forbidden placeholder {token!r} in section {section_name!r} "
                        f"({path}:{lineno})"
                    )
            else:
                if token in line:
                    _fail(
                        f"{slice_id} spec contains forbidden placeholder {token!r} in section {section_name!r} "
                        f"({path}:{lineno})"
                    )


def _scan_forbidden_placeholders_whole_doc(lines: list[str], path: Path, slice_id: str) -> None:
    """
    v2 tightening: forbid placeholders anywhere in the spec, not only inside required sections.

    This closes a loophole where a placeholder like '[[FILL]]' in the title line could pass.
    """
    non_fenced = [(lineno, line) for (lineno, line) in _iter_non_fenced_lines(lines)]
    _scan_forbidden_placeholders("whole document", non_fenced, path, slice_id)


def _validate_behavior_delta(section_lines: list[tuple[int, str]], path: Path, slice_id: str) -> None:
    patterns = {
        "Existing": re.compile(r"^\s*[-*]\s+Existing:"),
        "New": re.compile(r"^\s*[-*]\s+New:"),
        "Why": re.compile(r"^\s*[-*]\s+Why:"),
    }
    counts = {k: 0 for k in patterns}
    for _, line in section_lines:
        for k, pat in patterns.items():
            if pat.search(line):
                counts[k] += 1

    bad = {k: v for k, v in counts.items() if v != 1}
    if bad:
        _fail(
            f"{slice_id} spec Behavior delta shape invalid in {path}: "
            + ", ".join(f"{k}={v} (expected 1)" for k, v in bad.items())
        )


def _leading_spaces(line: str) -> int:
    return len(line) - len(line.lstrip(" "))


def _validate_acceptance_criteria(
    section_lines: list[tuple[int, str]], path: Path, slice_id: str
) -> list[str]:
    bullet_re = re.compile(r"^\s*[-*]\s+")
    bullets: list[tuple[int, str]] = [(ln, s) for (ln, s) in section_lines if bullet_re.match(s)]
    if not bullets:
        _fail(f"{slice_id} spec missing acceptance criteria bullets in {path}")

    top_indent = min(_leading_spaces(s) for _, s in bullets)

    ac_ids: list[str] = []
    top_level_bullets = [(ln, s) for (ln, s) in bullets if _leading_spaces(s) == top_indent]
    ac_re = re.compile(rf"^\s*[-*]\s+(AC-{re.escape(slice_id)}-\d\d):\s+.+")
    for lineno, line in top_level_bullets:
        m = ac_re.match(line)
        if not m:
            _fail(
                f"{slice_id} spec acceptance criteria top-level bullet must start with "
                f"'AC-{slice_id}-NN:' ({path}:{lineno})"
            )
        ac_ids.append(m.group(1))

    if not (1 <= len(ac_ids) <= 8):
        _fail(f"{slice_id} spec acceptance criteria count is {len(ac_ids)}; must be 1..8 ({path})")

    dupes = sorted({x for x in ac_ids if ac_ids.count(x) > 1})
    if dupes:
        _fail(f"{slice_id} spec contains duplicate AC IDs: {', '.join(dupes)} ({path})")

    return ac_ids


def _validate_behavior_has_subheading(section_lines: list[tuple[int, str]], path: Path, slice_id: str) -> None:
    for _, line in section_lines:
        if re.match(r"^\s*###\s+.+", line):
            return
    _fail(f"{slice_id} spec missing '###' subsection under '## Behavior (authoritative)' ({path})")


def _validate_out_of_scope_non_empty(section_lines: list[tuple[int, str]], path: Path, slice_id: str) -> None:
    for _, line in section_lines:
        if re.match(r"^\s*[-*]\s+.+", line):
            return
    _fail(f"{slice_id} spec Out of scope must contain at least one bullet ({path})")


def _validate_task_ac_ids(task_id: str, task: dict[str, Any], expected: set[str], spec_path: Path) -> None:
    ac_ids = task.get("ac_ids")
    if not isinstance(ac_ids, list) or not all(isinstance(x, str) and x for x in ac_ids):
        _fail(f"{task_id}.ac_ids must exist and be an array of non-empty strings (spec {spec_path})")
    if len(set(ac_ids)) != len(ac_ids):
        _fail(f"{task_id}.ac_ids contains duplicates (spec {spec_path})")

    actual = set(ac_ids)
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing or extra:
        _fail(
            f"{task_id}.ac_ids mismatch vs spec ({spec_path}): "
            f"missing={missing}, extra={extra}"
        )


def main() -> int:
    ap = argparse.ArgumentParser(description="Validate slice spec invariants (v1 legacy-safe; v2 strict when opted in).")
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/next/<feature> (or any feature dir)")
    ap.add_argument("--force-v2", action="store_true", help="Enforce v2 regardless of tasks.json meta.slice_spec_version")
    args = ap.parse_args()

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists():
        _fail(f"feature dir does not exist: {feature_dir}")

    tasks_data = _read_tasks_json(feature_dir)
    meta = tasks_data.get("meta") if isinstance(tasks_data.get("meta"), dict) else {}
    slice_spec_version = meta.get("slice_spec_version")

    v2_mode = bool(args.force_v2) or (isinstance(slice_spec_version, int) and slice_spec_version >= 2)

    slices = _derive_slices(tasks_data)
    if not slices:
        if v2_mode:
            _fail(f"no slice triad tasks discovered in tasks.json for feature dir: {feature_dir}")
        return 0

    # Pairing check: v2 requires both code and test tasks per slice.
    if v2_mode:
        missing_pairs = [s.slice_id for s in slices.values() if s.code is None or s.test is None]
        if missing_pairs:
            _fail(f"v2 requires both code and test tasks per slice; missing pairs for: {', '.join(missing_pairs)}")

    tasks_list = tasks_data.get("tasks")
    tasks_by_id: dict[str, dict[str, Any]] = {
        t.get("id"): t for t in tasks_list if isinstance(t, dict) and isinstance(t.get("id"), str)
    }

    for slice_id, st in slices.items():
        spec_path = _find_slice_spec_path(feature_dir, st)
        if not spec_path.exists():
            _fail(f"missing slice spec for {slice_id}: {spec_path}")

        try:
            text = spec_path.read_text(encoding="utf-8")
        except OSError as e:
            _fail(f"unable to read slice spec for {slice_id}: {spec_path} ({e})")

        if not v2_mode:
            continue

        lines = text.splitlines(keepends=False)

        _scan_forbidden_placeholders_whole_doc(lines, spec_path, slice_id)

        sections: dict[str, list[tuple[int, str]]] = {}
        for header in V2_REQUIRED_HEADERS:
            section_lines, header_line = _extract_section(lines, header)
            if header_line < 0:
                _fail(f"{slice_id} spec missing header: {header!r} ({spec_path})")
            sections[header] = section_lines

        # Placeholder scan (required sections only).
        for header, section_lines in sections.items():
            _scan_forbidden_placeholders(header, section_lines, spec_path, slice_id)

        _validate_behavior_delta(sections["## Behavior delta (single)"], spec_path, slice_id)
        ac_ids = _validate_acceptance_criteria(sections["## Acceptance criteria"], spec_path, slice_id)
        _validate_behavior_has_subheading(sections["## Behavior (authoritative)"], spec_path, slice_id)
        _validate_out_of_scope_non_empty(sections["## Out of scope"], spec_path, slice_id)

        expected_set = set(ac_ids)
        for required_task_suffix in ("code", "test", "integ"):
            task_id = f"{slice_id}-{required_task_suffix}"
            task = tasks_by_id.get(task_id)
            if task is None:
                _fail(f"v2 requires task {task_id!r} to exist for traceability (spec {spec_path})")
            _validate_task_ac_ids(task_id, task, expected_set, spec_path)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
