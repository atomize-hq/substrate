#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


FSE_BEGIN_MARKER = "<!-- PM_FSE_WORKSTREAM_INDEX:BEGIN -->"
FSE_END_MARKER = "<!-- PM_FSE_WORKSTREAM_INDEX:END -->"
LEGACY_BEGIN_MARKER = "<!-- PM_PWS_INDEX:BEGIN -->"
LEGACY_END_MARKER = "<!-- PM_PWS_INDEX:END -->"
DEFAULT_TRIAGE_REL = "pre-planning/workstream_triage.md"
LEGACY_TRIAGE_REL = "workstream_triage.md"

HEADING_RE = re.compile(r"^###\s+(?P<id>\S+)\s+—\s+")
FENCE_START_RE = re.compile(r"^```")
JSON_FENCE_RE = re.compile(r"```json\s*\r?\n(?P<body>[\s\S]*?)\r?\n```")
SLUG_RE = re.compile(r"^[a-z0-9_]+$")
LEGACY_SLICE_ID_RE = re.compile(r"^[A-Za-z][A-Za-z0-9]*\d+$")
HEADING_WRAPPERS = ("***", "___", "**", "__", "`", "*", "_")
ACCEPTED_ORDER_LINE_RE = re.compile(
    r"^\s*-\s+(?:Accepted slice order|Recommended candidate order)(?::|\b)",
    re.IGNORECASE,
)
SLICE_BULLET_RE = re.compile(r"^\s*-\s+`?(?P<slice_id>[A-Za-z][A-Za-z0-9]*\d+)`?\s*$")


@dataclass(frozen=True)
class ValidationError:
    message: str


@dataclass(frozen=True)
class SliceAuthority:
    triage_path: Path
    version: int
    slice_prefix: str
    accepted_slice_order: list[str]


def _emit(prefix: str, msg: str) -> None:
    print(f"{prefix}: {msg}", file=sys.stderr)


def _fail(msg: str) -> None:
    _emit("FAIL", msg)
    raise SystemExit(1)


def _warn(msg: str) -> None:
    _emit("WARN", msg)


def _as_str_list(value: Any) -> list[str] | None:
    if not isinstance(value, list) or not all(isinstance(x, str) for x in value):
        return None
    return list(value)


def _normalize_slice_order(raw: list[str]) -> list[str]:
    out: list[str] = []
    seen: set[str] = set()
    for item in raw:
        value = item.strip()
        if not value or value in seen:
            continue
        seen.add(value)
        out.append(value)
    return out


def _normalize_owns_path(raw: str) -> str:
    s = raw.strip().replace("\\", "/")
    while s.startswith("./"):
        s = s[2:]
    # Collapse multiple slashes (defensive; also handles accidental leading //).
    s = re.sub(r"/{2,}", "/", s)
    return s


def _is_absolute_like(path: str) -> bool:
    # POSIX absolute or UNC-ish, after normalization.
    if path.startswith("/"):
        return True
    if path.startswith("//"):
        return True
    # Windows drive letter.
    if re.match(r"^[A-Za-z]:/", path):
        return True
    return False


def _iter_non_fenced_lines(lines: list[str]) -> Iterable[str]:
    in_fence = False
    for line in lines:
        if FENCE_START_RE.match(line.strip()):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        yield line


def _resolve_triage_path(feature_dir: Path, raw: str, *, advisory: bool) -> Path | None:
    raw = (raw or "").strip()
    if not raw:
        if advisory:
            _warn("empty --workstream-triage path; skipping")
            return None
        _fail("--workstream-triage cannot be empty")

    candidate = Path(raw)
    if not candidate.is_absolute():
        candidate = feature_dir / candidate

    if candidate.exists():
        return candidate

    # Legacy fallback only if the default path was used.
    if raw == DEFAULT_TRIAGE_REL:
        legacy = feature_dir / LEGACY_TRIAGE_REL
        if legacy.exists():
            return legacy

    if advisory:
        _warn(f"workstream triage artifact not found: {candidate}")
        return None

    _fail(f"workstream triage artifact not found: {candidate}")
    return None


def _find_index_markers(text: str) -> tuple[str, str, str]:
    marker_pairs = (
        (FSE_BEGIN_MARKER, FSE_END_MARKER, "PM_FSE_WORKSTREAM_INDEX"),
        (LEGACY_BEGIN_MARKER, LEGACY_END_MARKER, "PM_PWS_INDEX"),
    )

    matches: list[tuple[str, str, str]] = []
    for begin_marker, end_marker, label in marker_pairs:
        begin_n = text.count(begin_marker)
        end_n = text.count(end_marker)
        if begin_n == 0 and end_n == 0:
            continue
        if begin_n != 1 or end_n != 1:
            raise ValueError(
                f"expected exactly one {begin_marker!r} and one {end_marker!r} "
                f"(found begin={begin_n}, end={end_n})"
            )
        matches.append((begin_marker, end_marker, label))

    if not matches:
        raise ValueError("missing workstream index markers")
    if len(matches) > 1:
        raise ValueError("workstream triage contains both legacy and FSE workstream index markers")
    return matches[0]


def _extract_index_json(text: str) -> dict[str, Any]:
    begin_marker, end_marker, label = _find_index_markers(text)

    begin_idx = text.find(begin_marker)
    end_idx = text.find(end_marker)
    if begin_idx < 0 or end_idx < 0 or begin_idx >= end_idx:
        raise ValueError(f"{label} markers are malformed (BEGIN/END order)")

    block = text[begin_idx + len(begin_marker) : end_idx]
    matches = list(JSON_FENCE_RE.finditer(block))
    if len(matches) != 1:
        raise ValueError(f"expected exactly one ```json fenced block inside markers (found {len(matches)})")

    body = matches[0].group("body").strip()
    try:
        data = json.loads(body)
    except json.JSONDecodeError as e:
        raise ValueError(f"{label} JSON is not valid JSON: {e}") from e
    if not isinstance(data, dict):
        raise ValueError(f"{label} JSON must be a JSON object")
    return data


def _is_fse_index(idx: dict[str, Any]) -> bool:
    return any(
        key in idx
        for key in (
            "index_version",
            "candidate_prefix",
            "recommended_workstream_order",
            "recommended_candidate_order",
            "workstreams",
        )
    )


def _extract_markdown_accepted_slice_order(text: str) -> list[str]:
    lines = text.splitlines()
    start: int | None = None
    parent_indent = 0
    for idx, line in enumerate(lines):
        if ACCEPTED_ORDER_LINE_RE.match(line):
            start = idx + 1
            parent_indent = len(line) - len(line.lstrip())
            break

    if start is None:
        return []

    ordered: list[str] = []
    seen: set[str] = set()
    for line in lines[start:]:
        if not line.strip():
            if ordered:
                break
            continue
        indent = len(line) - len(line.lstrip())
        if indent <= parent_indent:
            break
        match = SLICE_BULLET_RE.match(line)
        if match is None:
            if ordered:
                break
            continue
        slice_id = match.group("slice_id")
        if slice_id in seen:
            continue
        seen.add(slice_id)
        ordered.append(slice_id)
    return ordered


def _derive_slice_prefix(slice_ids: list[str]) -> str:
    prefixes: set[str] = set()
    for slice_id in slice_ids:
        match = re.match(r"^(?P<prefix>[A-Za-z][A-Za-z0-9]*?)(?P<num>\d+)$", slice_id)
        if match is None:
            continue
        prefixes.add(match.group("prefix"))
    if len(prefixes) == 1:
        return next(iter(prefixes))
    return ""


def _topo_check_acyclic(edges: dict[str, set[str]]) -> None:
    # edges: node -> deps
    indeg: dict[str, int] = {n: 0 for n in edges}
    rev: dict[str, set[str]] = {n: set() for n in edges}
    for node, deps in edges.items():
        for dep in deps:
            indeg[node] += 1
            rev[dep].add(node)

    q = [n for n, d in indeg.items() if d == 0]
    visited = 0
    while q:
        n = q.pop()
        visited += 1
        for out in rev.get(n, set()):
            indeg[out] -= 1
            if indeg[out] == 0:
                q.append(out)

    if visited != len(edges):
        raise ValueError("depends_on graph contains a cycle")


def _extract_heading_ids(markdown_text: str) -> dict[str, set[str]]:
    lines = markdown_text.splitlines()
    ids: dict[str, set[str]] = {}
    in_marker = False
    begin_markers = (FSE_BEGIN_MARKER, LEGACY_BEGIN_MARKER)
    end_markers = (FSE_END_MARKER, LEGACY_END_MARKER)
    for line in _iter_non_fenced_lines(lines):
        if any(marker in line for marker in begin_markers) and not in_marker:
            in_marker = True
            continue
        if any(marker in line for marker in end_markers) and in_marker:
            in_marker = False
            continue
        if in_marker:
            continue

        m = HEADING_RE.match(line)
        if not m:
            continue
        raw = m.group("id")
        normalized = _normalize_heading_id(raw)
        ids.setdefault(normalized, set()).add(raw)
    return ids


def _normalize_heading_id(raw: str) -> str:
    token = raw.strip()
    for wrapper in HEADING_WRAPPERS:
        if not token.startswith(wrapper) or not token.endswith(wrapper):
            continue
        if len(token) <= (2 * len(wrapper)):
            continue
        inner = token[len(wrapper) : -len(wrapper)].strip()
        if inner:
            return inner
    return token


def _describe_heading_id(raw_tokens: set[str], normalized: str) -> str:
    raws = sorted(raw_tokens)
    if len(raws) == 1:
        raw = raws[0]
        if raw == normalized:
            return repr(raw)
        return f"raw {raw!r} (normalized to {normalized!r})"

    rendered = ", ".join(repr(raw) for raw in raws)
    return f"raws [{rendered}] (normalized to {normalized!r})"


def _slice_order_from_owns_entries(owns_entries: Iterable[str]) -> list[str]:
    ordered: list[str] = []
    seen: set[str] = set()
    for own in owns_entries:
        match = re.match(r"^slices/(?P<slice_id>[A-Za-z][A-Za-z0-9]*\d+)/(?P=slice_id)-spec\.md$", own)
        if not match:
            continue
        slice_id = match.group("slice_id")
        if slice_id in seen:
            continue
        seen.add(slice_id)
        ordered.append(slice_id)
    return ordered


def _derive_v1_slice_order(idx: dict[str, Any]) -> list[str]:
    ordered: list[str] = []
    for raw in idx.get("pws", []):
        if not isinstance(raw, dict):
            continue
        owns = _as_str_list(raw.get("owns"))
        if owns is None:
            continue
        ordered.extend(_slice_order_from_owns_entries(_normalize_owns_path(x) for x in owns))
    return _normalize_slice_order(ordered)


def _derive_fse_slice_order(idx: dict[str, Any]) -> list[str]:
    ordered = _as_str_list(idx.get("draft_candidate_order"))
    if ordered is not None:
        return _normalize_slice_order(ordered)

    derived: list[str] = []
    for raw in idx.get("workstreams", []):
        if not isinstance(raw, dict):
            continue
        owns = _as_str_list(raw.get("owns"))
        if owns is None:
            continue
        derived.extend(_slice_order_from_owns_entries(_normalize_owns_path(x) for x in owns))
    return _normalize_slice_order(derived)


def _validate_candidate_order_field(
    *,
    triage_path: Path,
    candidate_prefix: str,
    field_name: str,
    value: Any,
    required: bool,
) -> tuple[list[str] | None, list[ValidationError]]:
    errors: list[ValidationError] = []
    if value is None:
        if required:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} must be an array of non-empty candidate ids"))
        return (None, errors)

    raw = _as_str_list(value)
    if raw is None:
        errors.append(ValidationError(message=f"{triage_path}: {field_name} must be an array of non-empty candidate ids"))
        return (None, errors)

    normalized: list[str] = []
    seen: set[str] = set()
    for item in raw:
        candidate_id = item.strip()
        if not candidate_id:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} contains an empty candidate id"))
            continue
        if candidate_id in seen:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} contains duplicate candidate id: {candidate_id!r}"))
            continue
        seen.add(candidate_id)
        normalized.append(candidate_id)
        if "-FWS-" in candidate_id:
            errors.append(
                ValidationError(
                    message=(
                        f"{triage_path}: {field_name} must contain candidate ids, not workstream ids "
                        f"(got {candidate_id!r})"
                    )
                )
            )
            continue
        if candidate_prefix and not candidate_id.startswith(f"{candidate_prefix}-"):
            errors.append(
                ValidationError(
                    message=(
                        f"{triage_path}: {field_name} candidate id {candidate_id!r} must start with "
                        f"candidate_prefix {candidate_prefix!r}"
                    )
                )
            )
            continue
        if candidate_prefix and LEGACY_SLICE_ID_RE.match(candidate_id):
            continue

    if required and not normalized:
        errors.append(ValidationError(message=f"{triage_path}: {field_name} must contain at least one candidate id"))

    return (normalized, errors)


def _validate_workstream_order_field(
    *,
    triage_path: Path,
    field_name: str,
    value: Any,
    known_workstream_ids: set[str],
    required: bool,
) -> tuple[list[str] | None, list[ValidationError]]:
    errors: list[ValidationError] = []
    if value is None:
        if required:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} must be an array of workstream ids"))
        return (None, errors)

    raw = _as_str_list(value)
    if raw is None:
        errors.append(ValidationError(message=f"{triage_path}: {field_name} must be an array of workstream ids"))
        return (None, errors)

    normalized: list[str] = []
    seen: set[str] = set()
    for item in raw:
        workstream_id = item.strip()
        if not workstream_id:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} contains an empty workstream id"))
            continue
        if workstream_id in seen:
            errors.append(
                ValidationError(message=f"{triage_path}: {field_name} contains duplicate workstream id: {workstream_id!r}")
            )
            continue
        seen.add(workstream_id)
        normalized.append(workstream_id)
        if workstream_id not in known_workstream_ids:
            errors.append(
                ValidationError(
                    message=(
                        f"{triage_path}: {field_name} references unknown workstream id: {workstream_id!r}"
                    )
                )
            )

    if required and not normalized:
        errors.append(ValidationError(message=f"{triage_path}: {field_name} must contain at least one workstream id"))

    return (normalized, errors)


def _accepted_slice_order_from_index(idx: dict[str, Any]) -> list[str]:
    if _is_fse_index(idx):
        accepted = _as_str_list(idx.get("draft_candidate_order"))
        if accepted is None:
            return []
        accepted = _normalize_slice_order(accepted)
        return accepted

    version = idx.get("pws_index_version")
    if version == 2:
        accepted = _as_str_list(idx.get("accepted_slice_order"))
        if accepted is None:
            raise ValueError("accepted_slice_order must be an array of strings for pws_index_version=2")
        accepted = _normalize_slice_order(accepted)
        if not accepted:
            raise ValueError("accepted_slice_order must contain at least one slice id for pws_index_version=2")
        return accepted
    return _derive_v1_slice_order(idx)


def _explicit_v2_authority_error(triage_path: Path) -> ValidationError:
    return ValidationError(
        message=(
            f"{triage_path}: downstream compatibility tooling requires an explicit ordered candidate list "
            "in the workstream triage artifact; rerun workstream triage before retrying"
        )
    )


def _load_slice_authority(
    feature_dir: Path,
    workstream_triage: str = DEFAULT_TRIAGE_REL,
    *,
    advisory: bool,
    require_v2: bool = False,
) -> tuple[Path | None, SliceAuthority | None, list[ValidationError]]:
    triage_path = _resolve_triage_path(feature_dir, workstream_triage, advisory=advisory)
    if triage_path is None:
        return (None, None, [])

    errors = _validate_doc(feature_dir, triage_path, advisory=advisory)
    if errors:
        return (triage_path, None, errors)

    text = triage_path.read_text(encoding="utf-8")
    if FSE_BEGIN_MARKER not in text and LEGACY_BEGIN_MARKER not in text:
        accepted_slice_order = _extract_markdown_accepted_slice_order(text)
        if not accepted_slice_order:
            return (
                triage_path,
                None,
                [ValidationError(message=f"{triage_path}: missing ordered candidate list in workstream triage prose")],
            )
        slice_prefix = _derive_slice_prefix(accepted_slice_order)
        return (
            triage_path,
            SliceAuthority(
                triage_path=triage_path,
                version=0,
                slice_prefix=slice_prefix,
                accepted_slice_order=accepted_slice_order,
            ),
            [],
        )

    try:
        idx = _extract_index_json(text)
    except Exception as e:
        return (triage_path, None, [ValidationError(message=f"{triage_path}: {e}")])

    version = idx.get("index_version") if _is_fse_index(idx) else idx.get("pws_index_version")
    if require_v2 and version != 2:
        return (triage_path, None, [_explicit_v2_authority_error(triage_path)])

    try:
        accepted_slice_order = _accepted_slice_order_from_index(idx)
    except Exception as e:
        return (triage_path, None, [ValidationError(message=f"{triage_path}: {e}")])

    slice_prefix = idx.get("candidate_prefix") if _is_fse_index(idx) else idx.get("slice_prefix")
    if not isinstance(slice_prefix, str):
        slice_prefix = _derive_slice_prefix(accepted_slice_order)
    slice_prefix = slice_prefix.strip()

    return (
        triage_path,
        SliceAuthority(
            triage_path=triage_path,
            version=version if isinstance(version, int) else 0,
            slice_prefix=slice_prefix,
            accepted_slice_order=accepted_slice_order,
        ),
        [],
    )


def _validate_doc(feature_dir: Path, triage_path: Path, advisory: bool) -> list[ValidationError]:
    errors: list[ValidationError] = []

    try:
        text = triage_path.read_text(encoding="utf-8")
    except Exception as e:
        return [ValidationError(message=f"unable to read triage artifact: {triage_path} ({e})")]

    if FSE_BEGIN_MARKER not in text and LEGACY_BEGIN_MARKER not in text:
        accepted_slice_order = _extract_markdown_accepted_slice_order(text)
        if accepted_slice_order:
            return []
        return [ValidationError(message=f"{triage_path}: missing workstream index block and ordered candidate prose")]

    try:
        idx = _extract_index_json(text)
    except Exception as e:
        return [ValidationError(message=f"{triage_path}: {e}")]

    is_fse = _is_fse_index(idx)
    v = idx.get("index_version") if is_fse else idx.get("pws_index_version")
    if is_fse:
        if v not in (1, None):
            errors.append(ValidationError(message=f"{triage_path}: index_version must be 1 (found {v!r})"))
    elif v not in (1, 2, None):
        errors.append(ValidationError(message=f"{triage_path}: pws_index_version must be 1 or 2 (found {v!r})"))

    slice_prefix = idx.get("candidate_prefix") if is_fse else idx.get("slice_prefix")
    if isinstance(slice_prefix, str) and slice_prefix.strip():
        slice_prefix = slice_prefix.strip()
    else:
        slice_prefix = ""

    accepted_slice_order: list[str] | None = None
    if is_fse:
        draft_slice_order, field_errors = _validate_candidate_order_field(
            triage_path=triage_path,
            candidate_prefix=slice_prefix,
            field_name="draft_candidate_order",
            value=idx.get("draft_candidate_order"),
            required=False,
        )
        errors.extend(field_errors)
        accepted_slice_order = draft_slice_order
    elif v == 2:
        accepted_slice_order, field_errors = _validate_candidate_order_field(
            triage_path=triage_path,
            candidate_prefix=slice_prefix,
            field_name="accepted_slice_order",
            value=idx.get("accepted_slice_order"),
            required=True,
        )
        errors.extend(field_errors)
        draft_slice_order, field_errors = _validate_candidate_order_field(
            triage_path=triage_path,
            candidate_prefix=slice_prefix,
            field_name="draft_slice_order",
            value=idx.get("draft_slice_order"),
            required=False,
        )
        errors.extend(field_errors)
        if accepted_slice_order and draft_slice_order is not None:
            extra = sorted(set(draft_slice_order) - set(accepted_slice_order))
            if extra:
                errors.append(
                    ValidationError(
                        message=(
                            f"{triage_path}: draft_slice_order must stay within accepted_slice_order; "
                            f"extra {extra}"
                        )
                    )
                )

    if accepted_slice_order is None:
        try:
            accepted_slice_order = _accepted_slice_order_from_index(idx)
        except Exception as e:
            errors.append(ValidationError(message=f"{triage_path}: {e}"))

    if accepted_slice_order and not slice_prefix:
        slice_prefix = _derive_slice_prefix(accepted_slice_order)

    entries_key = "workstreams" if is_fse else "pws"
    entry_label = "workstream" if is_fse else "pws"
    plural_label = "workstreams" if is_fse else "pws"
    pws_raw = idx.get(entries_key)
    if pws_raw is None:
        return errors
    if not isinstance(pws_raw, list) or not pws_raw:
        errors.append(ValidationError(message=f"{triage_path}: {entries_key} must be an array when provided"))
        return errors

    required_keys = {"id", "role", "depends_on", "assumes", "owns"}
    if is_fse:
        required_keys.add("outcomes")
    pws_ids: list[str] = []
    pws_by_id: dict[str, dict[str, Any]] = {}
    for i, raw in enumerate(pws_raw):
        if not isinstance(raw, dict):
            errors.append(ValidationError(message=f"{triage_path}: {entries_key}[{i}] must be an object"))
            continue

        missing = sorted(k for k in required_keys if k not in raw)
        if missing:
            errors.append(ValidationError(message=f"{triage_path}: {entries_key}[{i}] missing keys: {', '.join(missing)}"))
            continue

        pid = raw.get("id")
        if not isinstance(pid, str) or not pid.strip():
            errors.append(ValidationError(message=f"{triage_path}: {entries_key}[{i}].id must be a non-empty string"))
            continue
        pid = pid.strip()
        if pid in pws_by_id:
            errors.append(ValidationError(message=f"{triage_path}: duplicate {entry_label} id: {pid!r}"))
            continue

        pws_ids.append(pid)
        pws_by_id[pid] = raw

    if not pws_by_id:
        return errors

    id_set = set(pws_by_id.keys())

    if is_fse:
        workstream_order_value = idx.get("recommended_workstream_order")
        workstream_order_field_name = "recommended_workstream_order"
        if workstream_order_value is None and "recommended_candidate_order" in idx:
            workstream_order_value = idx.get("recommended_candidate_order")
            workstream_order_field_name = "recommended_candidate_order"
        recommended_workstream_order, field_errors = _validate_workstream_order_field(
            triage_path=triage_path,
            field_name=workstream_order_field_name,
            value=workstream_order_value,
            known_workstream_ids=id_set,
            required=True,
        )
        errors.extend(field_errors)
        if recommended_workstream_order is not None:
            missing = sorted(id_set - set(recommended_workstream_order))
            extra = sorted(set(recommended_workstream_order) - id_set)
            if missing:
                errors.append(
                    ValidationError(
                        message=(
                            f"{triage_path}: {workstream_order_field_name} must enumerate every workstream id exactly once; "
                            f"missing {missing}"
                        )
                    )
                )
            if extra:
                errors.append(
                    ValidationError(
                        message=(
                            f"{triage_path}: {workstream_order_field_name} must stay within defined workstream ids; "
                            f"extra {extra}"
                        )
                    )
                )

    # ID format validation.
    if slice_prefix:
        expected_prefix = f"{slice_prefix}-FWS-" if is_fse else f"{slice_prefix}-PWS-"
        for pid in sorted(id_set):
            if not pid.startswith(expected_prefix):
                errors.append(
                    ValidationError(
                        message=f"{triage_path}: {entry_label} id {pid!r} must start with {expected_prefix!r} (prefix mismatch?)"
                    )
                )
                continue
            slug = pid[len(expected_prefix) :]
            if not SLUG_RE.match(slug):
                errors.append(
                    ValidationError(
                        message=f"{triage_path}: {entry_label} id {pid!r} has invalid slug {slug!r} (expected [a-z0-9_]+)"
                    )
                )

    # depends_on integrity + acyclic.
    edges: dict[str, set[str]] = {pid: set() for pid in id_set}
    for pid, obj in pws_by_id.items():
        deps = _as_str_list(obj.get("depends_on"))
        if deps is None:
            errors.append(ValidationError(message=f"{triage_path}: {pid}.depends_on must be an array of strings"))
            continue
        dep_set: set[str] = set()
        for dep in deps:
            dep = dep.strip()
            if not dep:
                errors.append(ValidationError(message=f"{triage_path}: {pid}.depends_on contains an empty string"))
                continue
            if dep == pid:
                errors.append(ValidationError(message=f"{triage_path}: {pid}.depends_on contains self-dependency"))
                continue
            if dep not in id_set:
                errors.append(ValidationError(message=f"{triage_path}: {pid}.depends_on references unknown {entry_label} id: {dep!r}"))
                continue
            if dep in dep_set:
                errors.append(ValidationError(message=f"{triage_path}: {pid}.depends_on contains duplicate id: {dep!r}"))
                continue
            dep_set.add(dep)
        edges[pid] = dep_set

    try:
        _topo_check_acyclic(edges)
    except Exception as e:
        errors.append(ValidationError(message=f"{triage_path}: {e}"))

    # assumes rule: must not contain any PWS id strings.
    for pid, obj in pws_by_id.items():
        assumes = _as_str_list(obj.get("assumes"))
        if assumes is None:
            errors.append(ValidationError(message=f"{triage_path}: {pid}.assumes must be an array of strings"))
            continue
        for a in assumes:
            if not isinstance(a, str):
                continue
            for other_id in id_set:
                if other_id in a:
                    errors.append(
                        ValidationError(
                            message=(
                                f"{triage_path}: {pid}.assumes must not contain {entry_label} id strings; "
                                f"found reference to {other_id!r} in {a!r} (promote to depends_on or rephrase)"
                            )
                        )
                    )
                    break

    # owns: pack-relative + disjoint.
    owns_owner: dict[str, str] = {}
    owns_norm_by_id: dict[str, list[str]] = {}
    for pid, obj in pws_by_id.items():
        owns = _as_str_list(obj.get("owns"))
        if owns is None:
            errors.append(ValidationError(message=f"{triage_path}: {pid}.owns must be an array of strings"))
            continue

        normalized: list[str] = []
        for raw in owns:
            p = _normalize_owns_path(raw)
            if not p:
                errors.append(ValidationError(message=f"{triage_path}: {pid}.owns contains an empty path"))
                continue
            if _is_absolute_like(p):
                errors.append(ValidationError(message=f"{triage_path}: {pid}.owns must be pack-relative (got absolute path {p!r})"))
                continue
            if p.startswith("docs/"):
                errors.append(ValidationError(message=f"{triage_path}: {pid}.owns must be pack-relative (got repo-root path {p!r})"))
                continue
            parts = [seg for seg in p.split("/") if seg]
            if any(seg == ".." for seg in parts):
                errors.append(ValidationError(message=f"{triage_path}: {pid}.owns must not contain '..' segments (got {p!r})"))
                continue
            normalized.append(p)

        owns_norm_by_id[pid] = list(normalized)

        # duplicates within the same PWS
        dupes = sorted({x for x in normalized if normalized.count(x) > 1})
        for d in dupes:
            errors.append(ValidationError(message=f"{triage_path}: {pid}.owns contains duplicate path: {d!r}"))

        for p in normalized:
            prev = owns_owner.get(p)
            if prev is None:
                owns_owner[p] = pid
                continue
            if prev != pid:
                errors.append(
                    ValidationError(
                        message=(
                            f"{triage_path}: owns path overlap is not allowed: {p!r} is claimed by both {prev!r} and {pid!r}"
                        )
                    )
            )

    # Prose alignment: headings must match JSON workstream ids.
    heading_ids = _extract_heading_ids(text)
    normalized_heading_ids = set(heading_ids.keys())
    missing_in_json = sorted(h for h in normalized_heading_ids if h not in id_set)
    for hid in missing_in_json:
        errors.append(
            ValidationError(
                message=(
                    f"{triage_path}: heading workstream id missing from workstream index JSON: "
                    f"{_describe_heading_id(heading_ids[hid], hid)}"
                )
            )
        )

    missing_in_headings = sorted(pid for pid in id_set if pid not in normalized_heading_ids)
    for pid in missing_in_headings:
        errors.append(ValidationError(message=f"{triage_path}: workstream index id missing corresponding '### {pid} —' heading"))

    return errors


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Validate workstream triage slice authority (FSE workstream index JSON when present, otherwise ordered-candidate prose)."
    )
    ap.add_argument("--feature-dir", required=True, help="docs/project_management/packs/<bucket>/<feature> (absolute or relative)")
    ap.add_argument(
        "--workstream-triage",
        default=DEFAULT_TRIAGE_REL,
        help=(
            "Path to workstream_triage.md (absolute or feature-dir-relative). "
            f"Default: {DEFAULT_TRIAGE_REL} (legacy fallback: {LEGACY_TRIAGE_REL})"
        ),
    )
    ap.add_argument("--advisory", action="store_true", help="Warn-only mode (always exits 0).")
    args = ap.parse_args()

    feature_dir = Path(args.feature_dir)
    if not feature_dir.exists():
        _emit("ERROR", f"feature dir does not exist: {feature_dir}")
        return 2

    triage_path = _resolve_triage_path(feature_dir, args.workstream_triage, advisory=args.advisory)
    if triage_path is None:
        return 0 if args.advisory else 1

    errors = _validate_doc(feature_dir, triage_path, advisory=args.advisory)
    if not errors:
        return 0

    if args.advisory:
        for e in errors:
            _warn(e.message)
        return 0

    for e in errors:
        _emit("FAIL", e.message)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
