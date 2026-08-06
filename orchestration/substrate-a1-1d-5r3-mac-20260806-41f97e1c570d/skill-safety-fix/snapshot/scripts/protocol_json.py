"""Strict JSON and repository-path helpers for orchestration scripts."""

from __future__ import annotations

import json
import re
import unicodedata
from pathlib import Path, PurePosixPath
from typing import Any


def _reject_nonfinite(value: str) -> None:
    raise ValueError(f"non-finite JSON number: {value}")


def _reject_duplicate_keys(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise ValueError(f"duplicate JSON key: {key}")
        result[key] = value
    return result


def load_json(path: Path) -> Any:
    return json.loads(
        path.read_text(encoding="utf-8"),
        object_pairs_hook=_reject_duplicate_keys,
        parse_constant=_reject_nonfinite,
    )


def has_forbidden_text_control(value: str) -> bool:
    return any(
        unicodedata.category(character).startswith("C")
        or unicodedata.category(character) in {"Zl", "Zp"}
        for character in value
    )


def is_canonical_repo_relative_path(value: str) -> bool:
    if (
        not value
        or value.startswith("/")
        or "\\" in value
        or has_forbidden_text_control(value)
    ):
        return False
    path = PurePosixPath(value)
    if any(part in {"", ".", ".."} for part in path.parts):
        return False
    return path.as_posix() == value


def is_absolute_platform_path(value: str) -> bool:
    if (
        not value
        or has_forbidden_text_control(value)
    ):
        return False
    return (
        value.startswith("/")
        or bool(re.match(r"^[A-Za-z]:[\\/]", value))
        or value.startswith("\\\\")
    )


def is_canonical_branch_ref(value: str) -> bool:
    """Accept a conservative subset of `git check-ref-format` branch refs."""
    prefix = "refs/heads/"
    if not value.startswith(prefix):
        return False
    suffix = value[len(prefix) :]
    if (
        not suffix
        or value.endswith("/")
        or value.endswith(".")
        or ".." in value
        or "//" in value
        or "@{" in value
        or any(character.isspace() for character in value)
        or has_forbidden_text_control(value)
        or any(character in value for character in "~^:?*[\\")
    ):
        return False
    parts = suffix.split("/")
    return all(
        part
        and not part.startswith(".")
        and not part.endswith(".")
        and not part.endswith(".lock")
        for part in parts
    )
