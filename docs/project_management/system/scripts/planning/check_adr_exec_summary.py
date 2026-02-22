#!/usr/bin/env python3

import argparse
import hashlib
import os
import re
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Dict, List, Optional, Sequence, Set, Tuple


EXEC_HEADING_RE = re.compile(r"^##[ \t]+Executive Summary[ \t]+\((?:Operator)\)[ \t]*\r?$", re.MULTILINE)
HASH_LINE_RE = re.compile(r"^ADR_BODY_SHA256:[ \t]*([0-9a-f]{64})[ \t]*\r?$", re.MULTILINE)

EXISTING_RE = re.compile(r"(?mi)^\s*-\s*Existing:\s+\S")
NEW_RE = re.compile(r"(?mi)^\s*-\s*New:\s+\S")
WHY_RE = re.compile(r"(?mi)^\s*-\s*Why:\s+\S")

LEGACY_STANDARDS_PREFIX = "docs/project_management/standards/"
LEGACY_STANDARDS_REF_RE = re.compile(r"docs/project_management/standards/[A-Za-z0-9_./-]+\.md")
CANONICAL_STANDARDS_ROOT = Path("docs/project_management/system/standards")
CANONICAL_STANDARDS_MANIFEST = CANONICAL_STANDARDS_ROOT / "MANIFEST.yaml"


@dataclass(frozen=True)
class ExecSection:
    start: int
    end: int
    hash_value: Optional[str]


def _find_exec_section(text: str) -> Optional[ExecSection]:
    match = EXEC_HEADING_RE.search(text)
    if not match:
        return None

    start = match.start()
    next_h2 = re.search(r"^##\s+", text[match.end() :], re.MULTILINE)
    end = (match.end() + next_h2.start()) if next_h2 else len(text)

    section_text = text[start:end]
    hash_match = HASH_LINE_RE.search(section_text)
    hash_value = hash_match.group(1) if hash_match else None
    return ExecSection(start=start, end=end, hash_value=hash_value)


def _adr_body_hash(text: str, exec_section: ExecSection) -> str:
    without_exec = text[: exec_section.start] + text[exec_section.end :]
    return hashlib.sha256(without_exec.encode("utf-8")).hexdigest()


def _upsert_hash_line(section_text: str, new_hash: str) -> Tuple[str, bool]:
    if HASH_LINE_RE.search(section_text):
        updated = HASH_LINE_RE.sub(f"ADR_BODY_SHA256: {new_hash}", section_text, count=1)
        return updated, updated != section_text

    lines = section_text.splitlines(keepends=True)
    out = []
    inserted = False
    for index, line in enumerate(lines):
        out.append(line)
        if not inserted and index == 0:
            out.append(f"\nADR_BODY_SHA256: {new_hash}\n")
            inserted = True
    return "".join(out), True


def _validate_exec_summary_structure(path: Path, exec_text: str) -> int:
    missing = []
    if not EXISTING_RE.search(exec_text):
        missing.append("Existing:")
    if not NEW_RE.search(exec_text):
        missing.append("New:")
    if not WHY_RE.search(exec_text):
        missing.append("Why:")
    if missing:
        print(f"{path}: executive summary must include bullet lines for: {', '.join(missing)}", flush=True)
        return 1
    return 0


def _repo_root_for_path(start_dir: Path) -> Optional[Path]:
    try:
        out = subprocess.check_output(
            ["git", "-C", str(start_dir), "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception:
        out = ""
    if out:
        return Path(out)

    # Fallback: walk upwards looking for the canonical standards manifest.
    current = start_dir.resolve()
    for parent in (current, *current.parents):
        manifest = parent / CANONICAL_STANDARDS_MANIFEST
        if manifest.is_file():
            return parent
    return None


def _strip_yaml_scalar(value: str) -> str:
    v = value.strip()
    if len(v) >= 2 and ((v[0] == v[-1] == '"') or (v[0] == v[-1] == "'")):
        return v[1:-1]
    return v


def _load_manifest_standard_paths(repo_root: Path) -> List[str]:
    """
    Extract `path:` entries from docs/project_management/system/standards/MANIFEST.yaml.

    We intentionally do a minimal parse to avoid adding dependencies (PyYAML).
    """
    manifest_path = repo_root / CANONICAL_STANDARDS_MANIFEST
    if not manifest_path.is_file():
        return []

    out: List[str] = []
    for raw in manifest_path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if not line.startswith("path:"):
            continue
        _, value = line.split("path:", 1)
        path_value = _strip_yaml_scalar(value)
        if path_value:
            out.append(path_value)
    return out


def _build_manifest_index(paths: Sequence[str]) -> Dict[str, Set[str]]:
    index: Dict[str, Set[str]] = {}
    for p in paths:
        base = os.path.basename(p)
        if not base:
            continue
        index.setdefault(base, set()).add(p)
    return index


def _canonical_candidates_for_basename(repo_root: Path, basename: str, manifest_index: Dict[str, Set[str]]) -> List[str]:
    candidates: Set[str] = set()
    candidates.update(manifest_index.get(basename, set()))

    # Filesystem fallback: allow canonical resolution even if MANIFEST.yaml is missing/out of date.
    standards_root = repo_root / CANONICAL_STANDARDS_ROOT
    if standards_root.is_dir():
        for p in standards_root.rglob(basename):
            if p.is_file():
                try:
                    candidates.add(p.relative_to(repo_root).as_posix())
                except Exception:
                    # Shouldn't happen if under repo_root, but keep defensive.
                    candidates.add(str(p))

    return sorted(candidates)


def _find_legacy_standards_refs(text: str) -> List[str]:
    return sorted(set(LEGACY_STANDARDS_REF_RE.findall(text)))


def _rewrite_legacy_standards_refs(
    *,
    path: Path,
    text: str,
    repo_root: Path,
) -> Tuple[str, bool, List[str], Dict[str, str]]:
    legacy_refs = _find_legacy_standards_refs(text)
    if not legacy_refs:
        return text, False, [], {}

    manifest_paths = _load_manifest_standard_paths(repo_root)
    manifest_index = _build_manifest_index(manifest_paths)

    mapping: Dict[str, str] = {}
    errors: List[str] = []

    for legacy in legacy_refs:
        basename = os.path.basename(legacy)
        candidates = _canonical_candidates_for_basename(repo_root, basename, manifest_index)
        if not candidates:
            errors.append(
                f"{path}: legacy standards ref {legacy!r} has no canonical match under {CANONICAL_STANDARDS_ROOT.as_posix()!r} (basename={basename!r})"
            )
            continue
        if len(candidates) > 1:
            shown = ", ".join(candidates[:5])
            extra = "" if len(candidates) <= 5 else f" (+{len(candidates) - 5} more)"
            errors.append(
                f"{path}: legacy standards ref {legacy!r} is ambiguous for basename {basename!r}; candidates: {shown}{extra}"
            )
            continue
        mapping[legacy] = candidates[0]

    if errors:
        return text, False, errors, {}

    updated = text
    for old, new in mapping.items():
        updated = updated.replace(old, new)

    return updated, updated != text, [], mapping


def _print_legacy_standards_failure(*, path: Path, text: str, repo_root: Optional[Path]) -> None:
    refs = _find_legacy_standards_refs(text)
    if not refs:
        return

    print(
        f"{path}: legacy standards paths are not allowed; use 'docs/project_management/system/standards/...'",
        flush=True,
    )

    manifest_index: Dict[str, Set[str]] = {}
    if repo_root is not None:
        manifest_index = _build_manifest_index(_load_manifest_standard_paths(repo_root))

    for legacy in refs:
        print(f"- found: {legacy}", flush=True)
        if repo_root is None:
            print("  -> cannot suggest replacement (repo root not found)", flush=True)
            continue
        basename = os.path.basename(legacy)
        candidates = _canonical_candidates_for_basename(repo_root, basename, manifest_index)
        if len(candidates) == 1:
            print(f"  -> replace with: {candidates[0]}", flush=True)
        elif not candidates:
            print(
                f"  -> no canonical match under {CANONICAL_STANDARDS_ROOT.as_posix()}/**/{basename}",
                flush=True,
            )
        else:
            shown = ", ".join(candidates[:5])
            extra = "" if len(candidates) <= 5 else f" (+{len(candidates) - 5} more)"
            print(f"  -> ambiguous; candidates: {shown}{extra}", flush=True)


def check_adr(path: Path, fix: bool) -> int:
    text = path.read_text(encoding="utf-8")

    legacy_refs = _find_legacy_standards_refs(text)
    repo_root = _repo_root_for_path(path.parent) if legacy_refs or fix else None
    if legacy_refs and not fix:
        _print_legacy_standards_failure(path=path, text=text, repo_root=repo_root)
        return 1

    legacy_changed = False
    legacy_mapping: Dict[str, str] = {}
    if legacy_refs and fix:
        if repo_root is None:
            print(f"{path}: cannot rewrite legacy standards refs (repo root not found)", flush=True)
            return 1
        rewritten, changed, errors, mapping = _rewrite_legacy_standards_refs(path=path, text=text, repo_root=repo_root)
        if errors:
            for e in errors:
                print(e, flush=True)
            return 1
        text = rewritten
        legacy_changed = changed
        legacy_mapping = mapping

    section = _find_exec_section(text)
    if section is None:
        print(f"{path}: missing `## Executive Summary (Operator)` section", flush=True)
        return 1

    exec_text = text[section.start : section.end]
    if _validate_exec_summary_structure(path=path, exec_text=exec_text) != 0:
        return 1

    body_hash = _adr_body_hash(text, section)
    if section.hash_value == body_hash:
        if fix and legacy_changed:
            # Rewrite happened but the body hash might not require an update if changes were confined to exec section.
            path.write_text(text, encoding="utf-8")
            print(f"FIXED: {path} rewrote {len(legacy_mapping)} legacy standards reference(s)", flush=True)
        else:
            print(f"OK: {path} executive summary hash matches", flush=True)
        return 0

    if not fix:
        # If we're not fixing, also prefer reporting legacy standards refs when present.
        if legacy_refs:
            _print_legacy_standards_failure(path=path, text=text, repo_root=repo_root)
        if section.hash_value is None:
            print(f"{path}: missing ADR_BODY_SHA256 (expected {body_hash})", flush=True)
        else:
            print(f"{path}: ADR_BODY_SHA256 mismatch (found {section.hash_value}, expected {body_hash})", flush=True)
        return 1

    updated_exec_text, changed = _upsert_hash_line(exec_text, body_hash)
    if not changed:
        print(f"{path}: could not update ADR_BODY_SHA256", flush=True)
        return 1

    updated = text[: section.start] + updated_exec_text + text[section.end :]
    path.write_text(updated, encoding="utf-8")
    if legacy_changed:
        print(f"FIXED: {path} rewrote {len(legacy_mapping)} legacy standards reference(s)", flush=True)
    print(f"FIXED: {path} updated ADR_BODY_SHA256 to {body_hash}", flush=True)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="Check ADR Executive Summary drift (ADR_BODY_SHA256).")
    parser.add_argument("--adr", required=True, help="Path to ADR markdown file")
    parser.add_argument("--fix", action="store_true", help="Rewrite ADR_BODY_SHA256 to match current ADR body")
    args = parser.parse_args()

    adr_path = Path(args.adr)
    if not adr_path.exists():
        print(f"Missing ADR file: {adr_path}", flush=True)
        return 2

    return check_adr(path=adr_path, fix=args.fix)


if __name__ == "__main__":
    raise SystemExit(main())
