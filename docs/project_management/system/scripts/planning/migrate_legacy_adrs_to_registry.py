#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Iterator


def run(cmd: list[str], *, cwd: Path | None = None, check: bool = True) -> subprocess.CompletedProcess:
    return subprocess.run(cmd, cwd=cwd, check=check, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)


def git_root() -> Path:
    res = run(["git", "rev-parse", "--show-toplevel"])
    return Path(res.stdout.strip())


def git_status_dirty(repo_root: Path) -> bool:
    res = run(["git", "status", "--porcelain=v1"], cwd=repo_root)
    return bool(res.stdout.strip())


def is_text_file(path: Path) -> bool:
    # Heuristic: treat known project file types as text (include logs for repo-wide rewrites).
    text_suffixes = {
        ".md",
        ".txt",
        ".log",
        ".json",
        ".jsonl",
        ".yaml",
        ".yml",
        ".toml",
        ".tmpl",
        ".sh",
        ".ps1",
        ".py",
    }
    if path.suffix in text_suffixes:
        return True
    if path.name.endswith(".md.tmpl"):
        return True
    return False


def should_skip_dir(dirname: str) -> bool:
    # Keep this conservative: avoid rewriting vendored/build/worktree content.
    return dirname in {
        ".git",
        "target",
        "node_modules",
        "dist",
        "build",
        "out",
        "wt",
        ".venv",
        "__pycache__",
    }


def iter_repo_text_files(repo_root: Path, *, exclude_repo_paths: set[Path]) -> Iterator[Path]:
    for root, dirnames, filenames in os.walk(repo_root):
        dirnames[:] = [d for d in dirnames if not should_skip_dir(d)]
        for name in filenames:
            path = Path(root) / name
            try:
                rel = path.relative_to(repo_root)
            except ValueError:
                continue
            if rel in exclude_repo_paths:
                continue
            if is_text_file(path):
                yield path


def rewrite_paths_in_file(
    path: Path, replacements: list[tuple[str, str]], *, max_bytes: int
) -> tuple[bool, str]:
    try:
        raw = path.read_bytes()
    except OSError:
        return False, ""

    if len(raw) > max_bytes:
        return False, ""

    if b"\x00" in raw:
        return False, ""

    try:
        text = raw.decode("utf-8")
    except UnicodeDecodeError:
        return False, ""

    original = text
    for old, new in replacements:
        text = text.replace(old, new)

    if text == original:
        return False, ""

    return True, text


def find_remaining_needles(repo_root: Path, needles: list[str], *, max_bytes: int) -> list[tuple[Path, str]]:
    remaining: list[tuple[Path, str]] = []
    exclude_repo_paths = default_exclude_repo_paths(repo_root)
    for path in iter_repo_text_files(repo_root, exclude_repo_paths=exclude_repo_paths):
        try:
            raw = path.read_bytes()
        except OSError:
            continue
        if len(raw) > max_bytes:
            continue
        if b"\x00" in raw:
            continue
        try:
            text = raw.decode("utf-8")
        except UnicodeDecodeError:
            continue
        for needle in needles:
            if needle in text:
                remaining.append((path.relative_to(repo_root), needle))
                break
    return remaining


STATUS_PATTERNS: list[re.Pattern[str]] = [
    re.compile(r"(?mi)^\s*Status:\s*([^\r\n]+?)\s*$"),
    re.compile(r"(?mi)^\s*-\s*Status:\s*([^\r\n]+?)\s*$"),
    re.compile(r"(?mi)^\s*-\s*\*\*Status:\*\*\s*([^\r\n]+?)\s*$"),
]


def parse_status(text: str) -> str | None:
    for pat in STATUS_PATTERNS:
        m = pat.search(text)
        if m:
            return m.group(1).strip().strip("*").strip()
    return None


def normalize_status(status: str) -> str:
    s = status.strip()
    s = re.sub(r"\s+", " ", s)
    # Normalize common variants.
    lower = s.lower()
    if lower in {"in review", "in-review"}:
        return "In Review"
    if lower in {"accepted"}:
        return "Accepted"
    if lower in {"approved"}:
        return "Approved"
    if lower in {"draft"}:
        return "Draft"
    if lower in {"proposed"}:
        return "Proposed"
    if lower in {"superseded"}:
        return "Superseded"
    if lower in {"replaced"}:
        return "Replaced"
    if lower in {"implemented"}:
        return "Implemented"
    if lower in {"done"}:
        return "Done"
    if lower in {"rejected"}:
        return "Rejected"
    if lower in {"withdrawn"}:
        return "Withdrawn"
    return s


def bucket_for_adr(status: str, *, text: str) -> str:
    if status in {"Draft", "Proposed", "In Review"}:
        return "draft"

    if status in {"Superseded", "Replaced"}:
        return "superseded"

    if status in {"Implemented", "Done"}:
        return "implemented"

    if status in {"Accepted", "Approved"}:
        if "docs/project_management/_archived/" in text:
            return "implemented"
        return "queued"

    raise ValueError(f"Unsupported ADR status for bucketing: {status}")


@dataclass(frozen=True)
class AdrMove:
    src_repo: Path
    dst_repo: Path
    status: str
    bucket: str


def collect_legacy_adrs(repo_root: Path) -> list[AdrMove]:
    legacy_root = repo_root / "docs" / "project_management" / "next"
    if not legacy_root.is_dir():
        raise ValueError(f"Legacy ADR root not found: {legacy_root}")

    legacy_adrs = sorted(p for p in legacy_root.rglob("ADR-*.md") if p.is_file())
    moves: list[AdrMove] = []

    dst_seen: set[Path] = set()
    for src_abs in legacy_adrs:
        text = src_abs.read_text(encoding="utf-8")
        raw_status = parse_status(text)
        if raw_status is None:
            raise ValueError(f"Could not parse ADR status in: {src_abs.relative_to(repo_root)}")
        status = normalize_status(raw_status)
        bucket = bucket_for_adr(status, text=text)

        dst_repo = Path("docs") / "project_management" / "adrs" / bucket / src_abs.name
        if dst_repo in dst_seen:
            raise ValueError(f"ADR destination collision for {dst_repo} (multiple legacy ADRs share the same filename)")
        dst_seen.add(dst_repo)

        moves.append(
            AdrMove(
                src_repo=src_abs.relative_to(repo_root),
                dst_repo=dst_repo,
                status=status,
                bucket=bucket,
            )
        )

    return moves


def default_exclude_repo_paths(repo_root: Path) -> set[Path]:
    exclude: set[Path] = set()
    try:
        exclude.add(Path(__file__).resolve().relative_to(repo_root))
    except Exception:
        pass
    # Wrapper scripts must not be considered as "repo content" for needle checks.
    exclude.add(Path("scripts") / "planning" / "migrate_legacy_adrs_to_registry.py")
    return exclude


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Migrate legacy ADRs from docs/project_management/next/** into docs/project_management/adrs/<bucket>/, "
            "then rewrite in-repo references.\n\n"
            "Bucketing rules:\n"
            "  - Draft|Proposed|In Review -> adrs/draft/\n"
            "  - Accepted|Approved -> adrs/queued/ (unless ADR references docs/project_management/_archived/, then adrs/implemented/)\n"
            "  - Implemented|Done -> adrs/implemented/\n"
            "  - Superseded|Replaced -> adrs/superseded/\n"
        )
    )
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--dry-run", action="store_true", help="Print planned actions without modifying anything")
    mode.add_argument("--apply", action="store_true", help="Apply ADR moves and reference rewrites")
    mode.add_argument(
        "--verify-only",
        action="store_true",
        help="Verify no legacy ADR files/references remain (no modifications)",
    )
    parser.add_argument(
        "--scope",
        default="whole-repo",
        choices=["whole-repo"],
        help="Rewrite scope (only whole-repo is supported for this migration)",
    )
    parser.add_argument(
        "--allow-dirty",
        action="store_true",
        help="Allow running with a dirty git working tree (not recommended)",
    )
    args = parser.parse_args()

    repo_root = git_root()
    if not args.allow_dirty and git_status_dirty(repo_root):
        print("ERROR: git working tree is dirty; commit/stash changes or pass --allow-dirty", file=sys.stderr)
        return 2

    # Strict needles: no legacy ADR paths should remain in scanned text files post-migration.
    # Note: keep these needles out of the rewrite scan by excluding this script from the scan set.
    next_prefix = "docs/project_management/next/"
    strict_needles = [
        next_prefix + "ADR-",
        "./" + next_prefix + "ADR-",
    ]

    if args.verify_only:
        remaining = find_remaining_needles(repo_root, strict_needles, max_bytes=50 * 1024 * 1024)
        legacy_left = sorted((repo_root / "docs" / "project_management" / "next").rglob("ADR-*.md"))
        if legacy_left or remaining:
            if legacy_left:
                print("LEGACY_ADR_FILES=1", file=sys.stderr)
                for p in legacy_left:
                    print(f"LEGACY_ADR={p.relative_to(repo_root)}", file=sys.stderr)
            if remaining:
                print(f"REMAINING_REFERENCES={len(remaining)}", file=sys.stderr)
                for path, needle in remaining:
                    print(f"REMAINING={path.as_posix()} needle={needle}", file=sys.stderr)
            print("FAIL: legacy ADR files/references remain", file=sys.stderr)
            return 2
        print("OK: no legacy ADR files/references remain")
        return 0

    try:
        moves = collect_legacy_adrs(repo_root)
    except ValueError as e:
        print(f"ERROR: {e}", file=sys.stderr)
        return 2

    if not moves:
        remaining = find_remaining_needles(repo_root, strict_needles, max_bytes=50 * 1024 * 1024)
        if remaining:
            print(f"REMAINING_REFERENCES={len(remaining)}", file=sys.stderr)
            for path, needle in remaining:
                print(f"REMAINING={path.as_posix()} needle={needle}", file=sys.stderr)
            print("FAIL: legacy ADR references remain (no ADR files left to move)", file=sys.stderr)
            return 2
        print("OK: no legacy ADRs found under docs/project_management/next/**")
        return 0

    print(f"REPO_ROOT={repo_root}")
    print(f"SCOPE={args.scope}")
    print(f"MODE={'dry-run' if args.dry_run else 'apply'}")
    print("")
    print("== ADR moves ==")
    for m in moves:
        print(f"{m.src_repo.as_posix()}\t->\t{m.dst_repo.as_posix()}\t(bucket={m.bucket}, status={m.status})")

    replacements: list[tuple[str, str]] = []
    for m in moves:
        old = m.src_repo.as_posix()
        new = m.dst_repo.as_posix()
        replacements.append((old, new))
        replacements.append((f"./{old}", f"./{new}"))

    for m in moves:
        strict_needles.append(m.src_repo.as_posix())
        strict_needles.append(f"./{m.src_repo.as_posix()}")

    if args.dry_run:
        return 0

    # Apply moves.
    for m in moves:
        dst_abs = repo_root / m.dst_repo
        dst_abs.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(
            ["git", "mv", m.src_repo.as_posix(), m.dst_repo.as_posix()],
            cwd=repo_root,
            check=True,
        )

    # Rewrite references across repo.
    max_bytes = 50 * 1024 * 1024
    modified: list[Path] = []
    exclude_repo_paths = default_exclude_repo_paths(repo_root)
    for path in iter_repo_text_files(repo_root, exclude_repo_paths=exclude_repo_paths):
        changed, new_text = rewrite_paths_in_file(path, replacements, max_bytes=max_bytes)
        if not changed:
            continue
        modified.append(path.relative_to(repo_root))
        path.write_text(new_text, encoding="utf-8", newline="\n")

    print("")
    print(f"UPDATED_FILES={len(modified)}")
    for p in sorted(modified):
        print(f"UPDATED={p.as_posix()}")

    remaining = find_remaining_needles(repo_root, strict_needles, max_bytes=max_bytes)
    if remaining:
        print(f"REMAINING_REFERENCES={len(remaining)}", file=sys.stderr)
        for path, needle in remaining:
            print(f"REMAINING={path.as_posix()} needle={needle}", file=sys.stderr)
        print("ERROR: legacy ADR references remain; fix manually and rerun", file=sys.stderr)
        return 2

    # Ensure no legacy ADR files remain under next.
    legacy_left = sorted((repo_root / "docs" / "project_management" / "next").rglob("ADR-*.md"))
    if legacy_left:
        print("ERROR: legacy ADR files remain under docs/project_management/next/**:", file=sys.stderr)
        for p in legacy_left:
            print(f"  {p.relative_to(repo_root)}", file=sys.stderr)
        return 2

    print("REMAINING_REFERENCES=0")
    print("OK: migrated legacy ADRs into docs/project_management/adrs/<bucket>/ and rewrote references")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
