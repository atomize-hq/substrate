#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path
from typing import Iterable


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


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


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


def _scan_forbidden_placeholders_whole_doc(lines: list[str], path: Path, slice_id: str) -> None:
    for lineno, line in _iter_non_fenced_lines(lines):
        for token in FORBIDDEN_PLACEHOLDERS:
            if token in ("TBD", "TODO", "WIP", "TBA"):
                if re.search(rf"\b{re.escape(token)}\b", line):
                    _fail(f"{slice_id} spec contains forbidden placeholder {token!r} ({path}:{lineno})")
            else:
                if token in line:
                    _fail(f"{slice_id} spec contains forbidden placeholder {token!r} ({path}:{lineno})")


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


def _validate_acceptance_criteria(section_lines: list[tuple[int, str]], path: Path, slice_id: str) -> None:
    bullet_re = re.compile(r"^\s*[-*]\s+")
    bullets: list[tuple[int, str]] = [(ln, s) for (ln, s) in section_lines if bullet_re.match(s)]
    if not bullets:
        _fail(f"{slice_id} spec missing acceptance criteria bullets ({path})")

    top_indent = min(_leading_spaces(s) for _, s in bullets)
    top_level_bullets = [(ln, s) for (ln, s) in bullets if _leading_spaces(s) == top_indent]
    ac_re = re.compile(rf"^\s*[-*]\s+(AC-{re.escape(slice_id)}-\d\d):\s+.+")

    ac_ids: list[str] = []
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


def _validate_spec(path: Path) -> None:
    name = path.name
    suffix = "-spec.md"
    if not name.endswith(suffix):
        _fail(f"slice spec filename must end with {suffix!r} (got {name!r}) ({path})")

    slice_id = name[: -len(suffix)]
    if not slice_id:
        _fail(f"unable to derive slice id from filename: {name!r} ({path})")

    try:
        text = path.read_text(encoding="utf-8")
    except OSError as e:
        _fail(f"unable to read slice spec: {path} ({e})")

    lines = text.splitlines(keepends=False)

    _scan_forbidden_placeholders_whole_doc(lines, path, slice_id)

    # Required H2 headers must exist exactly once.
    headers = _collect_h2_headers(lines)
    header_lines: dict[str, list[int]] = {h: [] for h in V2_REQUIRED_HEADERS}
    for lineno, h in headers:
        if h in header_lines:
            header_lines[h].append(lineno)

    for h in V2_REQUIRED_HEADERS:
        if not header_lines[h]:
            _fail(f"{slice_id} spec missing header: {h!r} ({path})")
        if len(header_lines[h]) > 1:
            _fail(f"{slice_id} spec duplicate header: {h!r} (lines={header_lines[h]}) ({path})")

    sections: dict[str, list[tuple[int, str]]] = {}
    for header in V2_REQUIRED_HEADERS:
        section_lines, header_line = _extract_section(lines, header)
        if header_line < 0:
            # Should never happen after required-header check, but keep deterministic.
            _fail(f"{slice_id} spec missing header: {header!r} ({path})")
        sections[header] = section_lines

    _validate_behavior_delta(sections["## Behavior delta (single)"], path, slice_id)
    _validate_acceptance_criteria(sections["## Acceptance criteria"], path, slice_id)
    _validate_behavior_has_subheading(sections["## Behavior (authoritative)"], path, slice_id)
    _validate_out_of_scope_non_empty(sections["## Out of scope"], path, slice_id)


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Validate slice spec v2 structural invariants without tasks.json (doc-only hard gate)."
    )
    ap.add_argument(
        "--paths",
        nargs="+",
        required=True,
        help="Slice spec path(s) (absolute or repo-relative).",
    )
    args = ap.parse_args()

    for raw in args.paths:
        p = Path(str(raw))
        if not p.is_absolute():
            p = (Path.cwd() / p).resolve()
        if not p.exists():
            _fail(f"slice spec path does not exist: {raw!r} (resolved to {p})")
        if not p.is_file():
            _fail(f"slice spec path is not a file: {raw!r} (resolved to {p})")
        _validate_spec(p)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
