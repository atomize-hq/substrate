#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


ALLOWED_H3 = {
    "### Create": "create",
    "### Edit": "edit",
    "### Deprecate": "deprecate",
    "### Delete": "delete",
}

PLACEHOLDER_SUBSTRINGS = [
    "<path>",
    "None yet.",
]

PLACEHOLDER_WORDS = [
    "TBD",
    "TODO",
    "WIP",
]

GLOB_CHARS = set("*?[]{}")


@dataclass(frozen=True)
class Occurrence:
    section: str
    lineno: int
    token: str


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _fail(msg: str) -> None:
    _eprint(f"FAIL: {msg}")
    raise SystemExit(1)


def _usage_error(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def _read_json(path: Path) -> dict[str, Any]:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        _usage_error(f"missing required path: {path}")
    except json.JSONDecodeError as e:
        _usage_error(f"invalid JSON: {path}: {e}")
    return {}


def _derive_mode(feature_dir: Path, mode_override: str | None) -> str:
    if mode_override is not None:
        return mode_override

    tasks = _read_json(feature_dir / "tasks.json")
    meta = tasks.get("meta")
    if isinstance(meta, dict):
        slice_spec_version = meta.get("slice_spec_version")
        if isinstance(slice_spec_version, int) and slice_spec_version >= 2:
            return "strict"
    return "legacy"


def _repo_root_strict() -> Path:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as e:
        _fail(f"failed to locate repo root via git: {e}")
    if not out:
        _fail("git rev-parse returned empty repo root")
    return Path(out)


def _iter_region_lines(lines: list[str], start_lineno: int, end_lineno: int) -> Iterable[tuple[int, str]]:
    for idx in range(start_lineno, end_lineno + 1):
        yield (idx, lines[idx - 1].rstrip("\n"))


def _find_touch_set_region(lines: list[str]) -> tuple[int, int] | None:
    header_lineno = None
    for idx, line in enumerate(lines, start=1):
        if line.strip() == "## Touch set (explicit)":
            header_lineno = idx
            break

    if header_lineno is None:
        return None

    in_fence = False
    end_lineno = len(lines)
    for idx in range(header_lineno + 1, len(lines) + 1):
        s = lines[idx - 1]
        if s.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        if s.startswith("## "):
            end_lineno = idx - 1
            break

    return (header_lineno + 1, end_lineno)


def _iter_non_fenced_lines(region_lines: list[tuple[int, str]]) -> Iterable[tuple[int, str]]:
    in_fence = False
    for lineno, line in region_lines:
        if line.strip().startswith("```"):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        yield (lineno, line)


def _has_placeholder_tokens(line: str) -> str | None:
    for s in PLACEHOLDER_SUBSTRINGS:
        if s in line:
            return s
    for w in PLACEHOLDER_WORDS:
        if re.search(rf"\b{re.escape(w)}\b", line):
            return w
    return None


def _extract_backticked_token(line: str, section: str, lineno: int) -> str:
    matches = re.findall(r"`([^`]+)`", line)
    if len(matches) != 1:
        _fail(
            f"Touch Set bullet must contain exactly one backticked token (found {len(matches)}): "
            f"{section} (impact_map.md:{lineno})"
        )
    return matches[0]


def _normalize_path_token(raw: str, section: str, lineno: int) -> str:
    token = raw

    while token.startswith("./"):
        token = token[2:]

    if not token:
        _fail(f"empty path token is not allowed: {section} (impact_map.md:{lineno})")

    if token.startswith("/"):
        _fail(f"absolute paths are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    if token.startswith("~"):
        _fail(f"home-relative paths are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    if re.match(r"^[A-Za-z]:", token):
        _fail(f"drive-letter paths are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    if "\\" in token:
        _fail(f"backslashes are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    if "//" in token:
        _fail(f"double slashes are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    if any(c in GLOB_CHARS for c in token):
        _fail(f"glob tokens are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    parts = token.split("/")
    if ".." in parts:
        _fail(f"'..' segments are not allowed: {token!r} ({section}, impact_map.md:{lineno})")

    if token.startswith("./"):
        _fail(f"unexpected './' prefix after normalization: {token!r} ({section}, impact_map.md:{lineno})")

    return token


def _parse_sections_strict(lines: list[str]) -> dict[str, list[tuple[int, str]]]:
    region = _find_touch_set_region(lines)
    if region is None:
        _fail("missing required header: '## Touch set (explicit)'")

    start, end = region
    region_lines = list(_iter_region_lines(lines, start, end))

    sections: dict[str, list[tuple[int, str]]] = {k: [] for k in ALLOWED_H3.values()}
    seen: set[str] = set()
    current: str | None = None

    for lineno, line in _iter_non_fenced_lines(region_lines):
        s = line.strip()
        if s.startswith("### "):
            if s not in ALLOWED_H3:
                _fail(f"unknown H3 heading in Touch Set region: {s!r} (impact_map.md:{lineno})")
            if s in seen:
                _fail(f"duplicate H3 heading in Touch Set region: {s!r} (impact_map.md:{lineno})")
            seen.add(s)
            current = ALLOWED_H3[s]
            continue

        if current is not None:
            sections[current].append((lineno, line))

    missing = [h for h in ALLOWED_H3.keys() if h not in seen]
    if missing:
        _fail(f"missing required Touch Set H3 headings: {', '.join(missing)}")

    return sections


def _validate_sections_strict(sections: dict[str, list[tuple[int, str]]], repo_root: Path) -> dict[str, list[str]]:
    normalized_by_section: dict[str, list[str]] = {k: [] for k in sections.keys()}
    occurrences_by_token: dict[str, list[Occurrence]] = {}
    any_non_none = False

    for section, lines in sections.items():
        content_lines = [(lineno, line) for (lineno, line) in lines if line.strip() != ""]

        if not content_lines:
            _fail(f"empty Touch Set section is not allowed; use exactly '- None': {section}")

        if len(content_lines) == 1 and content_lines[0][1] == "- None":
            continue

        for lineno, line in content_lines:
            if line == "- None":
                _fail(f"section mixes '- None' with other entries: {section} (impact_map.md:{lineno})")

            placeholder = _has_placeholder_tokens(line)
            if placeholder is not None:
                _fail(f"placeholder token {placeholder!r} is not allowed in strict mode ({section}, impact_map.md:{lineno})")

            if line[:1].isspace():
                _fail(f"indented lines are not allowed in Touch Set sections ({section}, impact_map.md:{lineno})")

            if not (line.startswith("- ") or line.startswith("* ")):
                _fail(f"invalid Touch Set line (expected top-level bullet): {section} (impact_map.md:{lineno})")

            raw_token = _extract_backticked_token(line, section=section, lineno=lineno)
            token = _normalize_path_token(raw_token, section=section, lineno=lineno)

            any_non_none = True
            normalized_by_section[section].append(token)

            occurrences_by_token.setdefault(token, []).append(Occurrence(section=section, lineno=lineno, token=token))

            fs_path = repo_root / token
            is_dir_entry = token.endswith("/")

            if fs_path.exists() and fs_path.is_dir() and not is_dir_entry:
                _fail(f"declared directory token must end with '/': {token!r} ({section}, impact_map.md:{lineno})")

            if section in ("edit", "deprecate", "delete"):
                if not fs_path.exists():
                    _fail(f"declared path does not exist: {token!r} ({section}, impact_map.md:{lineno})")
                if is_dir_entry and not fs_path.is_dir():
                    _fail(f"declared directory allow entry is not a directory: {token!r} ({section}, impact_map.md:{lineno})")

            if section == "create":
                if fs_path.exists():
                    if is_dir_entry and not fs_path.is_dir():
                        _eprint(f"WARN: create directory allow entry exists but is not a directory: {token!r} (impact_map.md:{lineno})")
                    elif not is_dir_entry:
                        _eprint(f"WARN: create entry already exists: {token!r} (impact_map.md:{lineno})")

    if not any_non_none:
        _fail("strict Touch Set must include at least one non-'- None' entry across all sections")

    dupes = {t: occs for (t, occs) in occurrences_by_token.items() if len(occs) > 1}
    if dupes:
        lines: list[str] = []
        for token, occs in sorted(dupes.items(), key=lambda x: x[0]):
            locs = ", ".join(f"{o.section}:impact_map.md:{o.lineno}" for o in sorted(occs, key=lambda o: (o.section, o.lineno)))
            lines.append(f"{token} -> {locs}")
        _fail("duplicate Touch Set entries across sections:\n" + "\n".join(lines))

    for section in normalized_by_section.keys():
        normalized_by_section[section] = sorted(set(normalized_by_section[section]))

    return normalized_by_section


def _emit_json(normalized_by_section: dict[str, list[str]]) -> None:
    dir_prefixes: set[str] = set()
    for items in normalized_by_section.values():
        for t in items:
            if t.endswith("/"):
                dir_prefixes.add(t)

    out = {
        "create": normalized_by_section.get("create", []),
        "edit": normalized_by_section.get("edit", []),
        "deprecate": normalized_by_section.get("deprecate", []),
        "delete": normalized_by_section.get("delete", []),
        "dir_prefixes": sorted(dir_prefixes),
    }
    print(json.dumps(out, indent=2))


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Validate impact_map.md Touch Set for strict planning packs.")
    ap.add_argument("--feature-dir", required=True, help="Planning pack directory under docs/project_management/next/<feature>")
    ap.add_argument("--mode", choices=["strict", "legacy"], help="Override auto mode derivation")
    ap.add_argument("--emit-json", action="store_true", help="Emit JSON allowlists to stdout (stdout is JSON-only)")
    args = ap.parse_args(argv)

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists() or not feature_dir.is_dir():
        _usage_error(f"--feature-dir must be an existing directory: {feature_dir}")

    mode = _derive_mode(feature_dir, args.mode)

    if mode == "legacy":
        if args.emit_json:
            _emit_json({"create": [], "edit": [], "deprecate": [], "delete": []})
        else:
            _eprint("WARN: impact_map touch-set enforcement disabled (meta.slice_spec_version < 2).")
        return 0

    impact_map = feature_dir / "impact_map.md"
    if not impact_map.exists():
        _fail(f"missing required path: {impact_map}")

    repo_root = _repo_root_strict()
    lines = impact_map.read_text(encoding="utf-8").splitlines(keepends=True)

    sections = _parse_sections_strict(lines)
    normalized = _validate_sections_strict(sections, repo_root=repo_root)

    if args.emit_json:
        _emit_json(normalized)
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
