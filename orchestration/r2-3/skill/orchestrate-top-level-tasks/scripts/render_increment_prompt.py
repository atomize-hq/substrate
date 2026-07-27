#!/usr/bin/env python3
"""Render a top-level increment prompt from a template, variables, and contract."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

from protocol_json import (
    has_forbidden_text_control,
    is_absolute_platform_path,
    load_json,
)


PLACEHOLDER_RE = re.compile(r"\{\{([A-Z][A-Z0-9_]*)\}\}")
ANY_PLACEHOLDER_RE = re.compile(r"\{\{(.*?)\}\}", re.DOTALL)
MULTILINE_VARIABLES = {"PROTECTED_CHECKOUTS"}
CONTRACT_SENTINEL = "\x00INCREMENT_CONTRACT\x00"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--template", required=True, type=Path)
    parser.add_argument("--variables", required=True, type=Path)
    parser.add_argument("--contract", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    template = args.template.read_text(encoding="utf-8")
    raw_placeholders = ANY_PLACEHOLDER_RE.findall(template)
    invalid_placeholders = sorted(
        {
            placeholder
            for placeholder in raw_placeholders
            if not re.fullmatch(r"[A-Z][A-Z0-9_]*", placeholder)
        }
    )
    if invalid_placeholders:
        raise SystemExit("invalid placeholders: " + ", ".join(invalid_placeholders))
    if template.count("{{INCREMENT_CONTRACT}}") != 1:
        raise SystemExit("template must contain INCREMENT_CONTRACT exactly once")
    template = template.replace("{{INCREMENT_CONTRACT}}", CONTRACT_SENTINEL)
    try:
        variables = load_json(args.variables)
    except ValueError as exc:
        raise SystemExit(f"invalid variables JSON: {exc}") from exc
    if not isinstance(variables, dict):
        raise SystemExit("variables must be a JSON object")

    expected = set(PLACEHOLDER_RE.findall(template))
    actual = set(variables)
    missing = sorted(expected - actual)
    extra = sorted(actual - expected)
    if missing:
        raise SystemExit("missing variables: " + ", ".join(missing))
    if extra:
        raise SystemExit("unexpected variables: " + ", ".join(extra))

    replacements: dict[str, str] = {}
    for key, value in variables.items():
        if not isinstance(key, str) or not PLACEHOLDER_RE.fullmatch("{{" + key + "}}"):
            raise SystemExit(f"invalid variable name: {key!r}")
        if isinstance(value, list):
            if key not in MULTILINE_VARIABLES:
                raise SystemExit(f"{key} may not be a list")
            if not value or not all(isinstance(item, str) and item for item in value):
                raise SystemExit(f"{key} list entries must be non-empty strings")
            if not all(is_absolute_platform_path(item) for item in value):
                raise SystemExit(f"{key} entries must be absolute platform paths")
            replacements[key] = "\n".join(f"- {item}" for item in value)
        elif isinstance(value, str) and value:
            if key in MULTILINE_VARIABLES:
                raise SystemExit(f"{key} must be an array of absolute platform paths")
            if has_forbidden_text_control(value):
                raise SystemExit(f"{key} must be a single line")
            replacements[key] = value
        else:
            raise SystemExit(f"{key} must be a non-empty string")
    contract = args.contract.read_text(encoding="utf-8").rstrip()
    if not contract:
        raise SystemExit("increment contract must not be empty")
    rendered = PLACEHOLDER_RE.sub(
        lambda match: replacements.get(match.group(1), match.group(0)),
        template,
    )
    unresolved = sorted(set(ANY_PLACEHOLDER_RE.findall(rendered)))
    if unresolved:
        raise SystemExit(
            "unresolved or invalid placeholders: "
            + ", ".join(repr(item) for item in unresolved)
        )
    if "{{" in rendered or "}}" in rendered:
        raise SystemExit("unresolved or malformed placeholder delimiter")
    rendered = rendered.replace(CONTRACT_SENTINEL, contract)

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(rendered.rstrip() + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
