#!/usr/bin/env python3
from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT_LEVEL_FILENAMES = {
    "Makefile",
    "Cargo.toml",
    "Cargo.lock",
    "package.json",
    "package-lock.json",
    "pnpm-lock.yaml",
    "yarn.lock",
    "tasks.json",
    "plan.md",
    "contract.md",
    "decision_register.md",
    "manual_testing_playbook.md",
    "session_log.md",
    "quality_gate_report.md",
    "execution_preflight_report.md",
}

REPO_ROOT_PREFIXES = (
    "docs/",
    "scripts/",
    "tests/",
    "src/",
    "crates/",
    ".github/",
    "bin/",
)


def _fail(msg: str) -> None:
    print(f"FAIL: {msg}", file=sys.stderr)
    raise SystemExit(1)


def _looks_like_required_doc_path(token: str) -> bool:
    if not token or any(ch.isspace() for ch in token):
        return False
    if token.startswith("/"):
        return False
    if any(ch in token for ch in ("*", "?", "[")):
        return False
    if token in ROOT_LEVEL_FILENAMES:
        return True
    if "/" not in token:
        return False
    if token.endswith("/"):
        return True
    return "." in Path(token).name


def _extract_required_doc_paths(spec_manifest_text: str) -> list[str]:
    """
    Extract backticked paths from the 'Required spec documents (authoritative)' section.

    This is intentionally strict:
    - Only considers backticked strings in that section.
    - Fails if placeholder tokens appear where paths should be.
    """
    section_header = "## Required spec documents (authoritative)"
    start = spec_manifest_text.find(section_header)
    if start < 0:
        _fail(f"spec_manifest.md missing required section header: {section_header!r}")

    after_header = spec_manifest_text.find("\n", start)
    if after_header < 0:
        _fail("spec_manifest.md is malformed (no newline after section header)")

    remainder = spec_manifest_text[after_header + 1 :]
    # Stop at the next H2 (## ...) header.
    next_h2 = re.search(r"(?m)^##\s+", remainder)
    section_body = remainder[: next_h2.start()] if next_h2 else remainder

    tokens: list[str] = []
    for line in section_body.splitlines():
        match = re.match(r"^- `([^`]+)`(?:\s|$)", line)
        if match:
            tokens.append(match.group(1))
    path_tokens = [t for t in tokens if _looks_like_required_doc_path(t)]
    if not path_tokens:
        _fail("spec_manifest.md required-docs section contains no backticked paths")

    for t in path_tokens:
        if any(x in t for x in ("{{", "}}", "<", ">")):
            _fail(f"spec_manifest.md required-docs section contains placeholder token: `{t}`")

    return path_tokens


def main() -> int:
    ap = argparse.ArgumentParser(description="Validate spec_manifest.md required docs exist.")
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature>")
    ap.add_argument(
        "--spec-manifest",
        default="pre-planning/spec_manifest.md",
        help="Path to spec_manifest.md (absolute or feature-dir-relative). Default: pre-planning/spec_manifest.md",
    )
    args = ap.parse_args()

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists():
        _fail(f"feature dir does not exist: {feature_dir}")

    spec_manifest_path = Path(args.spec_manifest)
    if not spec_manifest_path.is_absolute():
        spec_manifest_path = feature_dir / spec_manifest_path

    if not spec_manifest_path.exists():
        # Legacy fallback for older packs that store artifacts at the feature-dir root.
        if args.spec_manifest == "pre-planning/spec_manifest.md":
            legacy = feature_dir / "spec_manifest.md"
            if legacy.exists():
                spec_manifest_path = legacy
            else:
                _fail(f"missing spec manifest: {spec_manifest_path} (also missing legacy: {legacy})")
        else:
            _fail(f"missing spec manifest: {spec_manifest_path}")

    text = spec_manifest_path.read_text(encoding="utf-8")
    raw_paths = _extract_required_doc_paths(text)

    missing: list[str] = []
    feature_dir_prefix = feature_dir.as_posix().rstrip("/") + "/"
    for raw in raw_paths:
        p = Path(raw)
        if not p.is_absolute():
            raw_norm = raw.replace("\\", "/").lstrip("./")
            # If the manifest lists repo-root-relative paths (common in this repo),
            # treat them as relative to CWD (repo root when invoked via make).
            if raw_norm.startswith(REPO_ROOT_PREFIXES) or raw_norm.startswith(feature_dir_prefix) or raw_norm == feature_dir.as_posix():
                p = Path(raw_norm)
            else:
                p = feature_dir / raw_norm

        if not p.exists():
            missing.append(str(p.resolve() if not p.is_absolute() else p))

    if missing:
        for p in missing:
            print(f"Missing required spec-manifest path: {p}", file=sys.stderr)
        return 1

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
