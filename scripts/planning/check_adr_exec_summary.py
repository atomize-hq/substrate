#!/usr/bin/env python3

import argparse
import hashlib
import re
from dataclasses import dataclass
from pathlib import Path
from typing import Optional, Tuple


EXEC_HEADING_RE = re.compile(r"^##\s+Executive Summary\s+\(Operator\)\s*$", re.MULTILINE)
HASH_LINE_RE = re.compile(r"^ADR_BODY_SHA256:\s*([0-9a-f]{64})\s*$", re.MULTILINE)

EXISTING_RE = re.compile(r"(?mi)^\s*-\s*Existing:\s+\S")
NEW_RE = re.compile(r"(?mi)^\s*-\s*New:\s+\S")
WHY_RE = re.compile(r"(?mi)^\s*-\s*Why:\s+\S")


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


def check_adr(path: Path, fix: bool) -> int:
    text = path.read_text(encoding="utf-8")
    section = _find_exec_section(text)
    if section is None:
        print(f"{path}: missing `## Executive Summary (Operator)` section", flush=True)
        return 1

    exec_text = text[section.start : section.end]
    if _validate_exec_summary_structure(path=path, exec_text=exec_text) != 0:
        return 1

    body_hash = _adr_body_hash(text, section)
    if section.hash_value == body_hash:
        print(f"OK: {path} executive summary hash matches", flush=True)
        return 0

    if not fix:
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

