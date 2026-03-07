#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import validate_ci_checkpoint_plan as vccp
import validate_pws_index as vpi


PHASE_PRE_TASKS = "pre_tasks_checkpoints"
PHASE_PRE_FULL_PLANNING = "pre_full_planning"
PHASE_EXECUTION_READY = "execution_ready"

SLICE_ID_RE = re.compile(r"^(?P<prefix>[A-Za-z][A-Za-z0-9]*?)(?P<num>\d+)$")
PLAN_HEADING_RE = re.compile(r"^###\s+(?P<slice_id>[A-Za-z][A-Za-z0-9]*\d+)\b")
MIN_SPEC_SLICE_RE = re.compile(
    r"^\s*-\s+(?:`?slice_id`?)\s*:\s*`(?P<slice_id>[A-Za-z][A-Za-z0-9]*\d+)`"
)
SLICE_SPEC_PATH_RE = re.compile(r"^slices/(?P<slice_id>[A-Za-z][A-Za-z0-9]*\d+)/(?P=slice_id)-spec\.md$")


@dataclass(frozen=True)
class SliceSource:
    name: str
    ordered: list[str]
    path: Path | None = None
    ordered_required: bool = True
    exact_set_required: bool = True

    @property
    def label(self) -> str:
        if self.path is None:
            return self.name
        return f"{self.name} ({self.path})"


@dataclass(frozen=True)
class CoherenceIssue:
    source_name: str
    message: str
    path: Path | None = None

    @property
    def label(self) -> str:
        if self.path is None:
            return self.source_name
        return f"{self.source_name} ({self.path})"


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def _slice_sort_key(slice_id: str) -> tuple[str, int, str]:
    match = SLICE_ID_RE.match(slice_id)
    if not match:
        return (slice_id, 0, slice_id)
    return (match.group("prefix"), int(match.group("num")), slice_id)


def _ordered_unique(values: list[str]) -> list[str]:
    seen: set[str] = set()
    out: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        out.append(value)
    return out


def _read_text_if_exists(path: Path) -> str | None:
    if not path.exists():
        return None
    return path.read_text(encoding="utf-8")


def _extract_min_spec_slice_ids(feature_dir: Path) -> SliceSource | None:
    path = feature_dir / "pre-planning" / "minimal_spec_draft.md"
    text = _read_text_if_exists(path)
    if text is None:
        return None

    ordered: list[str] = []
    for line in text.splitlines():
        match = MIN_SPEC_SLICE_RE.match(line)
        if match:
            ordered.append(match.group("slice_id"))
    ordered = _ordered_unique(ordered)
    if not ordered:
        return None
    return SliceSource(name="minimal_spec_draft", ordered=ordered, path=path)


def _extract_triage_slice_ids(
    feature_dir: Path,
    workstream_triage: str = vpi.DEFAULT_TRIAGE_REL,
    require_v2: bool = False,
) -> tuple[SliceSource | None, str | None, Path | None, list[vpi.ValidationError]]:
    triage_path, authority, errors = vpi._load_slice_authority(
        feature_dir,
        workstream_triage,
        advisory=True,
        require_v2=require_v2,
    )
    if authority is None:
        return (None, None, triage_path, errors)

    ordered = _ordered_unique(authority.accepted_slice_order)
    if not ordered:
        return (None, authority.slice_prefix or None, triage_path, errors)

    return (
        SliceSource(name="workstream_triage", ordered=ordered, path=authority.triage_path),
        authority.slice_prefix or None,
        authority.triage_path,
        errors,
    )


def _extract_plan_slice_ids(feature_dir: Path) -> SliceSource | None:
    path = feature_dir / "plan.md"
    text = _read_text_if_exists(path)
    if text is None:
        return None

    ordered: list[str] = []
    for line in text.splitlines():
        match = PLAN_HEADING_RE.match(line.strip())
        if match:
            ordered.append(match.group("slice_id"))
    ordered = _ordered_unique(ordered)
    if not ordered:
        return None
    return SliceSource(name="plan", ordered=ordered, path=path)


def _slice_prefixes_for_scan(slice_ids: list[str], triage_prefix: str | None) -> list[str]:
    prefixes: list[str] = []
    if triage_prefix:
        prefixes.append(triage_prefix)
    for slice_id in slice_ids:
        match = SLICE_ID_RE.match(slice_id)
        if match:
            prefix = match.group("prefix")
            if prefix not in prefixes:
                prefixes.append(prefix)
    return prefixes


def _scan_text_for_slice_ids(text: str, prefixes: list[str]) -> list[str]:
    if not prefixes:
        return []
    pattern = re.compile(r"\b(?:" + "|".join(re.escape(prefix) + r"\d+" for prefix in prefixes) + r")\b")
    return _ordered_unique(pattern.findall(text))


def _extract_aux_source(feature_dir: Path, rel_path: str, name: str, prefixes: list[str]) -> SliceSource | None:
    path = feature_dir / rel_path
    text = _read_text_if_exists(path)
    if text is None:
        return None
    ordered = _scan_text_for_slice_ids(text, prefixes)
    if not ordered:
        return None
    return SliceSource(name=name, ordered=ordered, path=path, ordered_required=False, exact_set_required=True)


def _extract_slice_specs(feature_dir: Path) -> SliceSource | None:
    specs_dir = feature_dir / "slices"
    if not specs_dir.exists():
        return None

    ordered: list[str] = []
    for spec_path in sorted(specs_dir.glob("*/*-spec.md")):
        match = re.match(r"^(?P<slice_id>[A-Za-z][A-Za-z0-9]*\d+)-spec\.md$", spec_path.name)
        if match:
            ordered.append(match.group("slice_id"))

    ordered = sorted(_ordered_unique(ordered), key=_slice_sort_key)
    if not ordered:
        return None
    return SliceSource(name="slice_specs", ordered=ordered, path=specs_dir, ordered_required=False, exact_set_required=True)


def _resolve_ci_checkpoint_plan(feature_dir: Path) -> Path | None:
    default_path = feature_dir / "pre-planning" / "ci_checkpoint_plan.md"
    if default_path.exists():
        return default_path
    legacy_path = feature_dir / "ci_checkpoint_plan.md"
    if legacy_path.exists():
        return legacy_path
    return None


def _extract_checkpoint_plan(feature_dir: Path) -> SliceSource | None:
    plan_path = _resolve_ci_checkpoint_plan(feature_dir)
    if plan_path is None:
        return None

    plan = vccp._extract_json_block(plan_path.read_text(encoding="utf-8"))
    _, checkpoints = vccp._parse_checkpoints(plan)
    ordered: list[str] = []
    for checkpoint in checkpoints:
        ordered.extend(checkpoint.slices)
    ordered = _ordered_unique(ordered)
    if not ordered:
        return None
    return SliceSource(name="ci_checkpoint_plan", ordered=ordered, path=plan_path)


def _extract_tasks_slice_ids(feature_dir: Path) -> SliceSource:
    tasks_path = feature_dir / "tasks.json"
    tasks_data = json.loads(tasks_path.read_text(encoding="utf-8"))

    tasks = tasks_data.get("tasks")
    if not isinstance(tasks, list):
        _fail(f"tasks.json must contain top-level tasks[] array ({tasks_path})")

    ordered: list[str] = []
    for task in tasks:
        if not isinstance(task, dict):
            continue
        task_id = task.get("id")
        if not isinstance(task_id, str):
            continue
        for suffix in ("-code", "-test", "-integ"):
            if task_id.endswith(suffix):
                ordered.append(task_id[: -len(suffix)])
                break

    ordered = sorted(_ordered_unique(ordered), key=_slice_sort_key)
    return SliceSource(name="tasks_json", ordered=ordered, path=tasks_path)


def _choose_baseline(sources: list[SliceSource]) -> SliceSource:
    # minimal_spec_draft is a draft starting point; workstream_triage may adopt a
    # different accepted slice skeleton without mutating the draft file.
    for preferred in ("workstream_triage", "plan", "slice_specs", "ci_checkpoint_plan", "minimal_spec_draft"):
        for source in sources:
            if source.name == preferred and source.ordered:
                return source
    _fail("could not determine accepted slice inventory from any planning surface")
    raise AssertionError("unreachable")


def _requires_strict_compare(source: SliceSource, baseline: SliceSource) -> bool:
    if source.name == baseline.name:
        return False
    if source.name == "minimal_spec_draft" and baseline.name != "minimal_spec_draft":
        return False
    return True


def _compare_ordered(source: SliceSource, expected: list[str], baseline_name: str) -> None:
    issue = _ordered_issue(source, expected, baseline_name)
    if issue is not None:
        _fail(issue.message)


def _compare_set(source: SliceSource, expected: list[str], baseline_name: str) -> None:
    issue = _set_issue(source, expected, baseline_name)
    if issue is not None:
        _fail(issue.message)


def _ordered_issue(source: SliceSource, expected: list[str], baseline_name: str) -> CoherenceIssue | None:
    if source.ordered == expected:
        return None
    return CoherenceIssue(
        source_name=source.name,
        path=source.path,
        message=(
            f"{source.label} disagrees with accepted slice order from {baseline_name}: "
            f"expected {expected}, got {source.ordered}"
        ),
    )


def _set_issue(source: SliceSource, expected: list[str], baseline_name: str) -> CoherenceIssue | None:
    expected_set = set(expected)
    actual_set = set(source.ordered)
    if actual_set == expected_set:
        return None
    missing = sorted(expected_set - actual_set, key=_slice_sort_key)
    extra = sorted(actual_set - expected_set, key=_slice_sort_key)
    parts: list[str] = []
    if missing:
        parts.append(f"missing {missing}")
    if extra:
        parts.append(f"extra {extra}")
    return CoherenceIssue(
        source_name=source.name,
        path=source.path,
        message=f"{source.label} disagrees with accepted slice set from {baseline_name}: " + "; ".join(parts),
    )


def inspect_pre_full_planning(
    feature_dir: Path,
    workstream_triage: str = vpi.DEFAULT_TRIAGE_REL,
) -> tuple[SliceSource | None, str | None, list[CoherenceIssue]]:
    issues: list[CoherenceIssue] = []
    triage_source, triage_prefix, triage_path, triage_errors = _extract_triage_slice_ids(
        feature_dir,
        workstream_triage,
        require_v2=True,
    )
    if triage_source is None:
        if triage_path is None:
            triage_path = Path(workstream_triage)
            if not triage_path.is_absolute():
                triage_path = feature_dir / workstream_triage
        message = (
            triage_errors[0].message
            if triage_errors
            else (
                f"missing or invalid triage slice authority: expected accepted slice order from "
                f"{triage_path}"
            )
        )
        issues.append(
            CoherenceIssue(
                source_name="workstream_triage",
                path=triage_path if triage_path.exists() else None,
                message=message,
            )
        )
        return (None, triage_prefix, issues)

    accepted_order = triage_source.ordered
    prefixes = _slice_prefixes_for_scan(accepted_order, triage_prefix)
    baseline_label = triage_source.label

    checkpoint_source = _extract_checkpoint_plan(feature_dir)
    if checkpoint_source is not None:
        issue = _ordered_issue(checkpoint_source, accepted_order, baseline_label)
        if issue is not None:
            issues.append(issue)

    for rel_path, name in (
        ("pre-planning/spec_manifest.md", "spec_manifest"),
        ("pre-planning/impact_map.md", "impact_map"),
    ):
        aux_source = _extract_aux_source(feature_dir, rel_path, name, prefixes)
        if aux_source is None:
            continue
        issue = _set_issue(aux_source, accepted_order, baseline_label)
        if issue is not None:
            issues.append(issue)

    return (triage_source, triage_prefix, issues)


def validate(feature_dir: Path, phase: str, workstream_triage: str = vpi.DEFAULT_TRIAGE_REL) -> None:
    if phase == PHASE_PRE_FULL_PLANNING:
        _, _, issues = inspect_pre_full_planning(feature_dir, workstream_triage)
        if issues:
            _fail(issues[0].message)
        return

    min_spec_source = _extract_min_spec_slice_ids(feature_dir)
    triage_source, triage_prefix, _, _ = _extract_triage_slice_ids(feature_dir, workstream_triage)
    plan_source = _extract_plan_slice_ids(feature_dir)
    slice_specs_source = _extract_slice_specs(feature_dir)
    checkpoint_source = _extract_checkpoint_plan(feature_dir)

    candidate_sources = [s for s in (min_spec_source, triage_source, plan_source, checkpoint_source, slice_specs_source) if s is not None]
    if not candidate_sources:
        _fail("no slice-bearing planning surfaces found")

    baseline = _choose_baseline(candidate_sources)
    accepted_order = baseline.ordered
    prefixes = _slice_prefixes_for_scan(accepted_order, triage_prefix)

    aux_sources = [
        _extract_aux_source(feature_dir, "pre-planning/spec_manifest.md", "spec_manifest", prefixes),
        _extract_aux_source(feature_dir, "pre-planning/impact_map.md", "impact_map", prefixes),
        _extract_aux_source(feature_dir, "pre-planning/alignment_report.md", "alignment_report", prefixes),
    ]

    for source in candidate_sources:
        if not _requires_strict_compare(source, baseline):
            continue
        if source.ordered_required:
            _compare_ordered(source, accepted_order, baseline.label)
        elif source.exact_set_required:
            _compare_set(source, accepted_order, baseline.label)

    for aux_source in aux_sources:
        if aux_source is None:
            continue
        _compare_set(aux_source, accepted_order, baseline.label)

    if phase == PHASE_EXECUTION_READY:
        tasks_source = _extract_tasks_slice_ids(feature_dir)
        _compare_set(tasks_source, accepted_order, baseline.label)
        _compare_ordered(tasks_source, sorted(accepted_order, key=_slice_sort_key), f"natural order for {baseline.label}")


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Validate accepted slice inventory coherence across planning surfaces.")
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    ap.add_argument("--phase", choices=[PHASE_PRE_TASKS, PHASE_PRE_FULL_PLANNING, PHASE_EXECUTION_READY], required=True)
    ap.add_argument(
        "--workstream-triage",
        default=vpi.DEFAULT_TRIAGE_REL,
        help=(
            "Path to workstream_triage.md (absolute or feature-dir-relative). "
            f"Default: {vpi.DEFAULT_TRIAGE_REL} (legacy fallback: {vpi.LEGACY_TRIAGE_REL})"
        ),
    )
    args = ap.parse_args(argv)

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists():
        _fail(f"feature dir does not exist: {feature_dir}")

    validate(feature_dir.resolve(), args.phase, args.workstream_triage)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
