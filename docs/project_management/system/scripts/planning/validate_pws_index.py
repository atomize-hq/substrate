#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable


BEGIN_MARKER = "<!-- PM_PWS_INDEX:BEGIN -->"
END_MARKER = "<!-- PM_PWS_INDEX:END -->"
DEFAULT_TRIAGE_REL = "pre-planning/workstream_triage.md"
LEGACY_TRIAGE_REL = "workstream_triage.md"

HEADING_RE = re.compile(r"^###\s+(?P<id>\S+)\s+—\s+")
FENCE_START_RE = re.compile(r"^```")
JSON_FENCE_RE = re.compile(r"```json\s*\r?\n(?P<body>[\s\S]*?)\r?\n```")
SLUG_RE = re.compile(r"^[a-z0-9_]+$")
SLICE_ID_RE = re.compile(r"^[A-Za-z][A-Za-z0-9]*\d+$")


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


def _extract_pm_pws_index_json(text: str) -> dict[str, Any]:
    begin_n = text.count(BEGIN_MARKER)
    end_n = text.count(END_MARKER)
    if begin_n != 1 or end_n != 1:
        raise ValueError(
            f"expected exactly one {BEGIN_MARKER!r} and one {END_MARKER!r} (found begin={begin_n}, end={end_n})"
        )

    begin_idx = text.find(BEGIN_MARKER)
    end_idx = text.find(END_MARKER)
    if begin_idx < 0 or end_idx < 0 or begin_idx >= end_idx:
        raise ValueError("PM_PWS_INDEX markers are malformed (BEGIN/END order)")

    block = text[begin_idx + len(BEGIN_MARKER) : end_idx]
    matches = list(JSON_FENCE_RE.finditer(block))
    if len(matches) != 1:
        raise ValueError(f"expected exactly one ```json fenced block inside markers (found {len(matches)})")

    body = matches[0].group("body").strip()
    try:
        data = json.loads(body)
    except json.JSONDecodeError as e:
        raise ValueError(f"PM_PWS_INDEX JSON is not valid JSON: {e}") from e
    if not isinstance(data, dict):
        raise ValueError("PM_PWS_INDEX JSON must be a JSON object")
    return data


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


def _extract_heading_ids(markdown_text: str) -> set[str]:
    lines = markdown_text.splitlines()
    ids: set[str] = set()
    in_marker = False
    for line in _iter_non_fenced_lines(lines):
        if BEGIN_MARKER in line and not in_marker:
            in_marker = True
            continue
        if END_MARKER in line and in_marker:
            in_marker = False
            continue
        if in_marker:
            continue

        m = HEADING_RE.match(line)
        if not m:
            continue
        ids.add(m.group("id"))
    return ids


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


def _validate_slice_order_field(
    *,
    triage_path: Path,
    slice_prefix: str,
    field_name: str,
    value: Any,
    required: bool,
) -> tuple[list[str] | None, list[ValidationError]]:
    errors: list[ValidationError] = []
    if value is None:
        if required:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} must be an array of non-empty slice ids"))
        return (None, errors)

    raw = _as_str_list(value)
    if raw is None:
        errors.append(ValidationError(message=f"{triage_path}: {field_name} must be an array of non-empty slice ids"))
        return (None, errors)

    normalized: list[str] = []
    seen: set[str] = set()
    for item in raw:
        slice_id = item.strip()
        if not slice_id:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} contains an empty slice id"))
            continue
        if slice_id in seen:
            errors.append(ValidationError(message=f"{triage_path}: {field_name} contains duplicate slice id: {slice_id!r}"))
            continue
        seen.add(slice_id)
        normalized.append(slice_id)
        if not SLICE_ID_RE.match(slice_id):
            errors.append(ValidationError(message=f"{triage_path}: {field_name} contains invalid slice id: {slice_id!r}"))
            continue
        if slice_prefix and not slice_id.startswith(slice_prefix):
            errors.append(
                ValidationError(
                    message=(
                        f"{triage_path}: {field_name} slice id {slice_id!r} must start with "
                        f"slice_prefix {slice_prefix!r}"
                    )
                )
            )

    if required and not normalized:
        errors.append(ValidationError(message=f"{triage_path}: {field_name} must contain at least one slice id"))

    return (normalized, errors)


def _accepted_slice_order_from_index(idx: dict[str, Any]) -> list[str]:
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
            f"{triage_path}: full planning requires PM_PWS_INDEX v2 with explicit accepted_slice_order; "
            "rerun pre-planning/workstream triage before retrying"
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

    try:
        idx = _extract_pm_pws_index_json(triage_path.read_text(encoding="utf-8"))
    except Exception as e:
        return (triage_path, None, [ValidationError(message=f"{triage_path}: {e}")])

    version = idx.get("pws_index_version")
    if require_v2 and version != 2:
        return (triage_path, None, [_explicit_v2_authority_error(triage_path)])

    slice_prefix = idx.get("slice_prefix")
    if not isinstance(slice_prefix, str):
        slice_prefix = ""
    slice_prefix = slice_prefix.strip()

    try:
        accepted_slice_order = _accepted_slice_order_from_index(idx)
    except Exception as e:
        return (triage_path, None, [ValidationError(message=f"{triage_path}: {e}")])

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

    try:
        idx = _extract_pm_pws_index_json(text)
    except Exception as e:
        return [ValidationError(message=f"{triage_path}: {e}")]

    v = idx.get("pws_index_version")
    if v not in (1, 2):
        errors.append(ValidationError(message=f"{triage_path}: pws_index_version must be 1 or 2 (found {v!r})"))

    slice_prefix = idx.get("slice_prefix")
    if not isinstance(slice_prefix, str) or not slice_prefix.strip():
        errors.append(ValidationError(message=f"{triage_path}: slice_prefix must be a non-empty string"))
        slice_prefix = ""
    else:
        slice_prefix = slice_prefix.strip()

    accepted_slice_order: list[str] | None = None
    if v == 2:
        accepted_slice_order, field_errors = _validate_slice_order_field(
            triage_path=triage_path,
            slice_prefix=slice_prefix,
            field_name="accepted_slice_order",
            value=idx.get("accepted_slice_order"),
            required=True,
        )
        errors.extend(field_errors)
        _, field_errors = _validate_slice_order_field(
            triage_path=triage_path,
            slice_prefix=slice_prefix,
            field_name="draft_slice_order",
            value=idx.get("draft_slice_order"),
            required=False,
        )
        errors.extend(field_errors)

    pws_raw = idx.get("pws")
    if not isinstance(pws_raw, list) or not pws_raw:
        errors.append(ValidationError(message=f"{triage_path}: pws must be a non-empty array"))
        return errors

    required_keys = {"id", "role", "depends_on", "assumes", "owns"}
    pws_ids: list[str] = []
    pws_by_id: dict[str, dict[str, Any]] = {}
    for i, raw in enumerate(pws_raw):
        if not isinstance(raw, dict):
            errors.append(ValidationError(message=f"{triage_path}: pws[{i}] must be an object"))
            continue

        missing = sorted(k for k in required_keys if k not in raw)
        if missing:
            errors.append(ValidationError(message=f"{triage_path}: pws[{i}] missing keys: {', '.join(missing)}"))
            continue

        pid = raw.get("id")
        if not isinstance(pid, str) or not pid.strip():
            errors.append(ValidationError(message=f"{triage_path}: pws[{i}].id must be a non-empty string"))
            continue
        pid = pid.strip()
        if pid in pws_by_id:
            errors.append(ValidationError(message=f"{triage_path}: duplicate pws id: {pid!r}"))
            continue

        pws_ids.append(pid)
        pws_by_id[pid] = raw

    if not pws_by_id:
        return errors

    id_set = set(pws_by_id.keys())

    # ID format validation.
    if slice_prefix:
        expected_prefix = f"{slice_prefix}-PWS-"
        for pid in sorted(id_set):
            if not pid.startswith(expected_prefix):
                errors.append(
                    ValidationError(
                        message=f"{triage_path}: PWS id {pid!r} must start with {expected_prefix!r} (slice_prefix mismatch?)"
                    )
                )
                continue
            slug = pid[len(expected_prefix) :]
            if not SLUG_RE.match(slug):
                errors.append(
                    ValidationError(
                        message=f"{triage_path}: PWS id {pid!r} has invalid slug {slug!r} (expected [a-z0-9_]+)"
                    )
                )

        required_contract = f"{slice_prefix}-PWS-contract"
        required_tasks = f"{slice_prefix}-PWS-tasks_checkpoints"
        for req in (required_contract, required_tasks):
            if req not in id_set:
                errors.append(ValidationError(message=f"{triage_path}: missing required PWS id: {req!r}"))

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
                errors.append(ValidationError(message=f"{triage_path}: {pid}.depends_on references unknown PWS id: {dep!r}"))
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
                                f"{triage_path}: {pid}.assumes must not contain PWS id strings; "
                                f"found reference to {other_id!r} in {a!r} (promote to depends_on or rephrase)"
                            )
                        )
                    )
                    break

    # owns: pack-relative + disjoint + tasks.json single-writer.
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

    # Step 3.5 triad alignment: tasks_checkpoints must own triad-critical surfaces.
    if slice_prefix:
        tasks_pws_id = f"{slice_prefix}-PWS-tasks_checkpoints"
        if tasks_pws_id in id_set:
            tasks_owns = set(owns_norm_by_id.get(tasks_pws_id, []))

            if "session_log.md" not in tasks_owns:
                errors.append(
                    ValidationError(
                        message=f"{triage_path}: {tasks_pws_id}.owns must include 'session_log.md' (triad execution surface)"
                    )
                )
            if "kickoff_prompts/" not in tasks_owns:
                errors.append(
                    ValidationError(
                        message=f"{triage_path}: {tasks_pws_id}.owns must include 'kickoff_prompts/' (prefix; triad kickoff prompts)"
                    )
                )

            if v == 2 and accepted_slice_order is not None:
                slice_ids = accepted_slice_order
            else:
                slice_ids = _derive_v1_slice_order(idx)

            for slice_id in slice_ids:
                required = f"slices/{slice_id}/kickoff_prompts/"
                if required not in tasks_owns:
                    errors.append(
                        ValidationError(
                            message=(
                                f"{triage_path}: {tasks_pws_id}.owns must include {required!r} "
                                f"(prefix; slice kickoff prompts)"
                            )
                        )
                    )

    if slice_prefix:
        tasks_pws_id = f"{slice_prefix}-PWS-tasks_checkpoints"
        tasks_owner = owns_owner.get("tasks.json")
        if tasks_owner != tasks_pws_id:
            errors.append(
                ValidationError(
                    message=(
                        f"{triage_path}: tasks.json must be owned by {tasks_pws_id!r} only "
                        f"(found owner={tasks_owner!r})"
                    )
                )
            )

    # Prose alignment: headings must match JSON pws ids.
    heading_ids = _extract_heading_ids(text)
    missing_in_json = sorted(h for h in heading_ids if h not in id_set)
    for hid in missing_in_json:
        errors.append(ValidationError(message=f"{triage_path}: heading PWS id missing from PM_PWS_INDEX JSON: {hid!r}"))

    missing_in_headings = sorted(pid for pid in id_set if pid not in heading_ids)
    for pid in missing_in_headings:
        errors.append(ValidationError(message=f"{triage_path}: PM_PWS_INDEX JSON id missing corresponding '### {pid} —' heading"))

    return errors


def main() -> int:
    ap = argparse.ArgumentParser(description="Validate PM_PWS_INDEX embedded JSON in pre-planning workstream triage.")
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
