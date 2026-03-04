#!/usr/bin/env python3
from __future__ import annotations

import argparse
import heapq
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any

import validate_pws_index as vpi


@dataclass(frozen=True)
class PwsNode:
    pws_id: str
    role: str
    depends_on: list[str]
    owns: list[str]
    normalized_owns: list[str]


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _usage_error(msg: str) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(2)


def _resolve_feature_dir(raw: str) -> Path:
    feature_dir = Path(raw).resolve()
    if not feature_dir.exists():
        _usage_error(f"feature dir does not exist: {feature_dir}")
    if not feature_dir.is_dir():
        _usage_error(f"feature dir is not a directory: {feature_dir}")
    return feature_dir


def _resolve_triage_path(feature_dir: Path, raw: str, *, advisory: bool) -> Path:
    triage_path = vpi._resolve_triage_path(feature_dir, raw, advisory=advisory)
    if triage_path is None:
        # advisory-only mode; not used in pm_pws_plan.py, but keep defensive.
        raise SystemExit(0)
    return triage_path


def _load_index(triage_path: Path) -> dict[str, Any]:
    try:
        text = triage_path.read_text(encoding="utf-8")
    except Exception as e:
        vpi._emit("FAIL", f"unable to read triage artifact: {triage_path} ({e})")
        raise SystemExit(1)

    try:
        return vpi._extract_pm_pws_index_json(text)
    except Exception as e:
        vpi._emit("FAIL", f"{triage_path}: {e}")
        raise SystemExit(1)


def _extract_nodes(idx: dict[str, Any], *, triage_path: Path) -> tuple[str, dict[str, PwsNode]]:
    slice_prefix = idx.get("slice_prefix")
    if not isinstance(slice_prefix, str) or not slice_prefix.strip():
        vpi._emit("FAIL", f"{triage_path}: slice_prefix must be a non-empty string")
        raise SystemExit(1)
    slice_prefix = slice_prefix.strip()

    pws_raw = idx.get("pws")
    if not isinstance(pws_raw, list) or not pws_raw:
        vpi._emit("FAIL", f"{triage_path}: pws must be a non-empty array")
        raise SystemExit(1)

    nodes: dict[str, PwsNode] = {}
    for i, raw in enumerate(pws_raw):
        if not isinstance(raw, dict):
            vpi._emit("FAIL", f"{triage_path}: pws[{i}] must be an object")
            raise SystemExit(1)

        pid = raw.get("id")
        role = raw.get("role")
        depends_on = raw.get("depends_on")
        owns = raw.get("owns")

        if not isinstance(pid, str) or not pid.strip():
            vpi._emit("FAIL", f"{triage_path}: pws[{i}].id must be a non-empty string")
            raise SystemExit(1)
        pid = pid.strip()

        if not isinstance(role, str) or not role.strip():
            vpi._emit("FAIL", f"{triage_path}: {pid}.role must be a non-empty string")
            raise SystemExit(1)
        role = role.strip()

        if not isinstance(depends_on, list) or not all(isinstance(x, str) for x in depends_on):
            vpi._emit("FAIL", f"{triage_path}: {pid}.depends_on must be an array of strings")
            raise SystemExit(1)

        if not isinstance(owns, list) or not all(isinstance(x, str) for x in owns):
            vpi._emit("FAIL", f"{triage_path}: {pid}.owns must be an array of strings")
            raise SystemExit(1)

        normalized_owns = [vpi._normalize_owns_path(x) for x in owns]

        nodes[pid] = PwsNode(
            pws_id=pid,
            role=role,
            depends_on=[x.strip() for x in depends_on],
            owns=list(owns),
            normalized_owns=normalized_owns,
        )

    return slice_prefix, nodes


def _build_graph(nodes: dict[str, PwsNode]) -> tuple[dict[str, set[str]], dict[str, set[str]], dict[str, int]]:
    deps: dict[str, set[str]] = {pid: set(node.depends_on) for pid, node in nodes.items()}
    rev: dict[str, set[str]] = {pid: set() for pid in nodes}
    indeg: dict[str, int] = {pid: len(d) for pid, d in deps.items()}
    for pid, dset in deps.items():
        for dep in dset:
            # Validator guarantees dep exists, but be defensive.
            if dep in rev:
                rev[dep].add(pid)
    return deps, rev, indeg


def _stable_topo_order(nodes: dict[str, PwsNode]) -> list[str]:
    _, rev, indeg = _build_graph(nodes)
    ready = [pid for pid, d in indeg.items() if d == 0]
    heapq.heapify(ready)

    order: list[str] = []
    while ready:
        pid = heapq.heappop(ready)
        order.append(pid)
        for out in sorted(rev.get(pid, set())):
            indeg[out] -= 1
            if indeg[out] == 0:
                heapq.heappush(ready, out)

    if len(order) != len(nodes):
        # Should never happen: validate_pws_index guarantees acyclic.
        raise ValueError("depends_on graph contains a cycle (unexpected after validation)")
    return order


def _first_owns_conflict(
    *,
    candidate_id: str,
    candidate_owns: list[str],
    exact_claims: dict[str, str],
    prefix_claims: dict[str, str],
) -> tuple[str, str] | None:
    del candidate_id  # present for future debug; deterministic checks use only inputs.

    sorted_prefixes = sorted(prefix_claims.keys())
    sorted_exact = sorted(exact_claims.keys())

    for own in sorted(candidate_owns):
        if not own:
            continue

        prev = exact_claims.get(own)
        if prev is not None:
            return prev, own

        prev = prefix_claims.get(own)
        if prev is not None:
            return prev, own

        for prefix in sorted_prefixes:
            if own.startswith(prefix):
                return prefix_claims[prefix], prefix

        if own.endswith("/"):
            for claimed in sorted_exact:
                if claimed.startswith(own):
                    return exact_claims[claimed], own
            for claimed_prefix in sorted_prefixes:
                if claimed_prefix.startswith(own):
                    return prefix_claims[claimed_prefix], own

    return None


def _parallel_layers(nodes: dict[str, PwsNode]) -> tuple[list[list[str]], list[tuple[int, str, str, str]]]:
    _, rev, indeg = _build_graph(nodes)
    scheduled: set[str] = set()
    layers: list[list[str]] = []
    warnings: list[tuple[int, str, str, str]] = []

    layer_idx = 0
    while len(scheduled) < len(nodes):
        runnable = sorted([pid for pid, d in indeg.items() if d == 0 and pid not in scheduled])
        if not runnable:
            raise ValueError("no runnable nodes (cycle?) (unexpected after validation)")

        layer: list[str] = []
        exact_claims: dict[str, str] = {}
        prefix_claims: dict[str, str] = {}

        for pid in runnable:
            node = nodes[pid]
            conflict = _first_owns_conflict(
                candidate_id=pid,
                candidate_owns=node.normalized_owns,
                exact_claims=exact_claims,
                prefix_claims=prefix_claims,
            )
            if conflict is not None:
                kept_id, conflict_path = conflict
                warnings.append((layer_idx, pid, kept_id, conflict_path))
                continue

            layer.append(pid)
            for own in node.normalized_owns:
                if not own:
                    continue
                if own.endswith("/"):
                    prefix_claims[own] = pid
                else:
                    exact_claims[own] = pid

        layers.append(layer)
        for pid in layer:
            scheduled.add(pid)

        for pid in layer:
            for out in rev.get(pid, set()):
                indeg[out] -= 1

        layer_idx += 1

    return layers, warnings


def _fmt_meta(node: PwsNode) -> str:
    return f"(role={node.role}, depends_on={len(node.depends_on)}, owns={len(node.owns)})"


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Print a read-only schedule plan from PM_PWS_INDEX (dry-run only).")
    ap.add_argument(
        "--feature-dir",
        required=True,
        help="Planning pack directory (absolute or relative).",
    )
    ap.add_argument(
        "--workstream-triage",
        default=vpi.DEFAULT_TRIAGE_REL,
        help=(
            "Path to workstream_triage.md (absolute or feature-dir-relative). "
            f"Default: {vpi.DEFAULT_TRIAGE_REL} (legacy fallback: {vpi.LEGACY_TRIAGE_REL})"
        ),
    )
    args = ap.parse_args(argv)

    feature_dir = _resolve_feature_dir(args.feature_dir)
    triage_path = _resolve_triage_path(feature_dir, args.workstream_triage, advisory=False)

    # Hard validate first (do not plan with a broken index).
    errors = vpi._validate_doc(feature_dir, triage_path, advisory=False)
    if errors:
        for e in errors:
            vpi._emit("FAIL", e.message)
        return 1

    idx = _load_index(triage_path)
    slice_prefix, nodes = _extract_nodes(idx, triage_path=triage_path)

    try:
        topo = _stable_topo_order(nodes)
        layers, warnings = _parallel_layers(nodes)
    except Exception as e:
        vpi._emit("FAIL", f"unable to compute schedule: {e}")
        return 1

    # Header
    print(f"PWS plan (slice_prefix={slice_prefix})")
    print(f"Feature dir: {feature_dir}")
    print(f"Nodes: {len(nodes)}")
    print()

    # Topological order
    print("Topological order:")
    for i, pid in enumerate(topo, start=1):
        print(f"{i}) {pid} {_fmt_meta(nodes[pid])}")
    print()

    # Parallel layers
    print("Parallel layers:")
    for k, layer in enumerate(layers):
        print(f"Layer {k}:")
        for pid in sorted(layer):
            print(f"- {pid} {_fmt_meta(nodes[pid])}")
    print()

    # Notes
    print("Notes:")
    for layer_idx, delayed_id, kept_id, path in warnings:
        print(
            f"WARN: owns conflict prevented full parallelism in layer {layer_idx}: "
            f"{delayed_id} conflicts with {kept_id} (path={path})"
        )
    print("Note: depends_on is the only scheduling signal; assumes is ignored.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))

