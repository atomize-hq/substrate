#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

import validate_impact_map as vim


PATH_TOKEN_RE = re.compile(
    r"(?P<path>"
    r"(?:crates|src|scripts|dist|tests|docs)/[A-Za-z0-9_./-]+"
    r"|Cargo\.toml"
    r"|Cargo\.lock"
    r"|Makefile"
    r")"
)

TASK_FIELDS_WITH_PATHS = ("references", "start_checklist", "end_checklist")


@dataclass(frozen=True)
class TouchsetIssue:
    source_name: str
    source_path: Path | None
    referenced_path: str
    message: str
    remediation: str


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def _usage_error(msg: str) -> None:
    print(f"ERROR: {msg}", file=sys.stderr)
    raise SystemExit(2)


def _pack_rel(feature_dir: Path, path: Path) -> str:
    return path.resolve().relative_to(feature_dir.resolve()).as_posix()


def _resolve_repo_path(repo_root: Path, raw_path: str | Path) -> Path:
    path = Path(raw_path)
    if path.is_absolute():
        return path
    return repo_root / path


def _iter_md_sources(feature_dir: Path, kickoff_paths: list[Path]) -> Iterable[tuple[str, Path]]:
    fixed = [
        ("plan", feature_dir / "plan.md"),
        ("manual_testing_playbook", feature_dir / "manual_testing_playbook.md"),
        ("execution_preflight_report", feature_dir / "execution_preflight_report.md"),
    ]
    for source_name, path in fixed:
        if path.exists():
            yield (source_name, path)

    for kickoff_path in kickoff_paths:
        if kickoff_path.exists():
            yield ("kickoff_prompt", kickoff_path)

    slices_dir = feature_dir / "slices"
    if slices_dir.exists():
        for report_path in sorted(slices_dir.glob("*/*-closeout_report.md")):
            if report_path.is_file():
                yield ("closeout_report", report_path)


def _load_tasks(feature_dir: Path) -> dict:
    tasks_path = feature_dir / "tasks.json"
    if not tasks_path.exists():
        _usage_error(f"missing tasks.json: {tasks_path}")
    try:
        return json.loads(tasks_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        _usage_error(f"tasks.json is not valid JSON: {exc}")


def _normalize_candidate(raw: str) -> str | None:
    candidate = raw.strip().strip("`\"'")
    if not candidate:
        return None
    candidate = candidate.split("#", 1)[0].rstrip(".,:;!?)]]}")
    if not candidate or candidate.endswith("/"):
        return None
    return candidate


def _is_implementation_facing(feature_dir_rel: str, rel_path: str) -> bool:
    if rel_path.startswith(feature_dir_rel.rstrip("/") + "/"):
        return False
    if rel_path.startswith("docs/project_management/"):
        return False
    return True


def _extract_text_paths(text: str, *, feature_dir_rel: str) -> list[str]:
    found: list[str] = []
    seen: set[str] = set()
    for match in PATH_TOKEN_RE.finditer(text):
        candidate = _normalize_candidate(match.group("path"))
        if candidate is None or candidate in seen:
            continue
        if _is_implementation_facing(feature_dir_rel, candidate):
            seen.add(candidate)
            found.append(candidate)
    return found


def _kickoff_paths(tasks_data: dict, *, repo_root: Path) -> list[Path]:
    tasks = tasks_data.get("tasks")
    if not isinstance(tasks, list):
        return []
    paths: list[Path] = []
    for task in tasks:
        if not isinstance(task, dict):
            continue
        kickoff = task.get("kickoff_prompt")
        if isinstance(kickoff, str) and kickoff:
            paths.append(_resolve_repo_path(repo_root, kickoff))
    return paths


def _extract_tasks_paths(tasks_data: dict, *, feature_dir_rel: str) -> list[str]:
    tasks = tasks_data.get("tasks")
    if not isinstance(tasks, list):
        return []

    seen: set[str] = set()
    found: list[str] = []
    for task in tasks:
        if not isinstance(task, dict):
            continue
        for field in TASK_FIELDS_WITH_PATHS:
            values = task.get(field)
            if not isinstance(values, list) or not all(isinstance(x, str) for x in values):
                continue
            for value in values:
                for rel_path in _extract_text_paths(value, feature_dir_rel=feature_dir_rel):
                    if rel_path not in seen:
                        seen.add(rel_path)
                        found.append(rel_path)
    return found


def _load_impact_map_tokens(feature_dir: Path) -> dict[str, list[str]]:
    mode = vim._derive_mode(feature_dir, None)
    if mode == "legacy":
        return {"create": [], "edit": [], "deprecate": [], "delete": []}

    impact_map_path = vim._resolve_impact_map_path(feature_dir, None)
    if not impact_map_path.exists():
        _fail(f"missing required path: {impact_map_path}")

    repo_root = vim._repo_root_strict()
    lines = impact_map_path.read_text(encoding="utf-8").splitlines(keepends=True)
    sections = vim._parse_sections_strict(lines)
    return vim._validate_sections_strict(sections, repo_root=repo_root)


def _matching_sections(rel_path: str, impact_tokens: dict[str, list[str]]) -> list[str]:
    matches: list[str] = []
    for section in ("create", "edit", "deprecate", "delete"):
        for token in impact_tokens.get(section, []):
            if token == rel_path or (token.endswith("/") and rel_path.startswith(token)):
                matches.append(section)
                break
    return matches


def inspect_feature_dir(feature_dir: Path) -> list[TouchsetIssue]:
    feature_dir = feature_dir.resolve()
    repo_root = vim._repo_root_strict()
    tasks_data = _load_tasks(feature_dir)
    feature_dir_rel = feature_dir.relative_to(repo_root).as_posix()
    impact_tokens = _load_impact_map_tokens(feature_dir)

    references: list[tuple[str, Path | None, str]] = []
    for rel_path in _extract_tasks_paths(tasks_data, feature_dir_rel=feature_dir_rel):
        references.append(("tasks_json", feature_dir / "tasks.json", rel_path))

    kickoff_paths = _kickoff_paths(tasks_data, repo_root=repo_root)
    for source_name, source_path in _iter_md_sources(feature_dir, kickoff_paths):
        try:
            text = source_path.read_text(encoding="utf-8")
        except FileNotFoundError:
            continue
        for rel_path in _extract_text_paths(text, feature_dir_rel=feature_dir_rel):
            references.append((source_name, source_path, rel_path))

    issues: list[TouchsetIssue] = []
    seen_refs: set[tuple[str, str]] = set()
    for source_name, source_path, rel_path in references:
        ref_key = (
            source_path.resolve().as_posix() if source_path is not None else source_name,
            rel_path,
        )
        if ref_key in seen_refs:
            continue
        seen_refs.add(ref_key)

        matches = _matching_sections(rel_path, impact_tokens)
        if not matches:
            issues.append(
                TouchsetIssue(
                    source_name=source_name,
                    source_path=source_path,
                    referenced_path=rel_path,
                    message=(
                        f"{source_name} references implementation-facing path {rel_path!r} "
                        "that is not covered by impact_map.md"
                    ),
                    remediation="safe",
                )
            )
            continue

        distinct_matches = sorted(set(matches))
        if len(distinct_matches) > 1:
            issues.append(
                TouchsetIssue(
                    source_name=source_name,
                    source_path=source_path,
                    referenced_path=rel_path,
                    message=(
                        f"{source_name} references implementation-facing path {rel_path!r} "
                        f"that is ambiguously covered by multiple impact_map sections: {distinct_matches}"
                    ),
                    remediation="hard_fail",
                )
            )
            continue

        matched_section = distinct_matches[0]
        path_exists = (repo_root / rel_path).exists()
        if path_exists and matched_section == "create":
            issues.append(
                TouchsetIssue(
                    source_name=source_name,
                    source_path=source_path,
                    referenced_path=rel_path,
                    message=(
                        f"{source_name} references existing implementation-facing path {rel_path!r}, "
                        "but impact_map.md covers it only under Create"
                    ),
                    remediation="safe",
                )
            )
        elif not path_exists and matched_section != "create":
            issues.append(
                TouchsetIssue(
                    source_name=source_name,
                    source_path=source_path,
                    referenced_path=rel_path,
                    message=(
                        f"{source_name} references non-existent implementation-facing path {rel_path!r}, "
                        "but impact_map.md does not cover it under Create"
                    ),
                    remediation="safe",
                )
            )

    return issues


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Validate late-pack implementation path coverage against impact_map touch set.")
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    args = ap.parse_args(argv)

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists() or not feature_dir.is_dir():
        _usage_error(f"--feature-dir must be an existing directory: {feature_dir}")

    issues = inspect_feature_dir(feature_dir)
    if issues:
        first = issues[0]
        source_path = f" ({_pack_rel(feature_dir, first.source_path)})" if first.source_path is not None else ""
        _fail(f"{first.message}{source_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
