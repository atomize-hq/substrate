#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import math
import re
import subprocess
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any


PM_LIFT_BEGIN = "<!-- PM_LIFT_VECTOR:BEGIN -->"
PM_LIFT_END = "<!-- PM_LIFT_VECTOR:END -->"

# Directory-prefix expansion (lift-only; does not rewrite impact_map.md).
EXPAND_DISCOUNT = 0.20
EXPAND_CAP = 10


@dataclass(frozen=True)
class LiftResult:
    model_version: int
    lift_score: int
    estimated_slices: int
    confidence: str
    triggers: list[str]
    missing_inputs: list[str]
    vector: dict[str, Any]
    derived: dict[str, Any]


def _eprint(msg: str) -> None:
    print(msg, file=sys.stderr)


def _die(msg: str, code: int = 1) -> None:
    _eprint(f"ERROR: {msg}")
    raise SystemExit(code)


def _repo_root() -> Path:
    try:
        out = subprocess.check_output(
            ["git", "rev-parse", "--show-toplevel"],
            text=True,
            stderr=subprocess.STDOUT,
        ).strip()
    except Exception as e:
        _die(f"git rev-parse failed (not in repo?): {e}")
    if not out:
        _die("git rev-parse returned empty repo root")
    return Path(out).resolve()


def _is_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool)


def _is_number(value: Any) -> bool:
    return isinstance(value, (int, float)) and not isinstance(value, bool)


def _get_obj(cfg: dict[str, Any], key: str, path: str) -> dict[str, Any]:
    v = cfg.get(key)
    if not isinstance(v, dict):
        _die(f"invalid model config at {path}.{key}: expected object")
    return v


def _get_list(cfg: dict[str, Any], key: str, path: str) -> list[Any]:
    v = cfg.get(key)
    if not isinstance(v, list):
        _die(f"invalid model config at {path}.{key}: expected array")
    return v


def _get_str(cfg: dict[str, Any], key: str, path: str) -> str:
    v = cfg.get(key)
    if not isinstance(v, str):
        _die(f"invalid model config at {path}.{key}: expected string")
    return v


def _get_int(cfg: dict[str, Any], key: str, path: str) -> int:
    v = cfg.get(key)
    if not _is_int(v):
        _die(f"invalid model config at {path}.{key}: expected integer")
    return int(v)


def _get_number(cfg: dict[str, Any], key: str, path: str) -> float:
    v = cfg.get(key)
    if not _is_number(v):
        _die(f"invalid model config at {path}.{key}: expected number")
    return float(v)


def _validate_model_config(cfg: dict[str, Any], *, config_path: Path) -> None:
    if not isinstance(cfg, dict):
        _die(f"invalid model config JSON: {config_path}: expected object at root")

    selection = _get_obj(cfg, "selection", "selection")
    supported = selection.get("supported_model_versions")
    if not isinstance(supported, list) or not supported or not all(_is_int(x) for x in supported):
        _die(f"invalid model config at selection.supported_model_versions: expected non-empty array<int>: {config_path}")
    if 1 not in supported:
        _die(f"invalid model config at selection.supported_model_versions: must include 1: {config_path}")

    default_version = selection.get("default_model_version")
    if not _is_int(default_version):
        _die(f"invalid model config at selection.default_model_version: expected int: {config_path}")
    if int(default_version) != 1:
        _die(f"invalid model config at selection.default_model_version: expected 1 for v1: {config_path}")

    policy = selection.get("vector_model_version_policy")
    if policy != "require_equal":
        _die(
            f"invalid model config at selection.vector_model_version_policy: expected 'require_equal': {config_path}"
        )

    weights = _get_obj(cfg, "weights", "weights")
    for section_name in ("touch", "contract", "qa", "docs", "ops"):
        sec = _get_obj(weights, section_name, f"weights.{section_name}")
        if not all(isinstance(k, str) for k in sec.keys()):
            _die(f"invalid model config at weights.{section_name}: keys must be strings: {config_path}")
        if not all(_is_number(v) for v in sec.values()):
            _die(f"invalid model config at weights.{section_name}: values must be numbers: {config_path}")
    contract_weights = _get_obj(weights, "contract", "weights.contract")
    if "behavior_deltas_blowup" not in contract_weights:
        _die(f"invalid model config at weights.contract.behavior_deltas_blowup: missing: {config_path}")

    risk_multipliers = _get_obj(cfg, "risk_multipliers", "risk_multipliers")
    for k in ("cross_platform", "security_sensitive", "concurrency_or_ordering", "migration_or_backfill"):
        if k not in risk_multipliers or not _is_number(risk_multipliers.get(k)):
            _die(f"invalid model config at risk_multipliers.{k}: expected number: {config_path}")

    unknowns_add = _get_obj(cfg, "unknowns_add", "unknowns_add")
    _get_number(unknowns_add, "unknowns_high_multiplier", "unknowns_add")

    estimated_slices = _get_obj(cfg, "estimated_slices", "estimated_slices")
    _get_number(estimated_slices, "divisor", "estimated_slices")
    _get_int(estimated_slices, "min", "estimated_slices")
    if _get_str(estimated_slices, "rounding", "estimated_slices") != "ceil":
        _die(f"invalid model config at estimated_slices.rounding: expected 'ceil': {config_path}")

    rounding = _get_obj(cfg, "rounding", "rounding")
    if _get_str(rounding, "score", "rounding") != "ceil":
        _die(f"invalid model config at rounding.score: expected 'ceil': {config_path}")

    confidence = _get_obj(cfg, "confidence", "confidence")
    rules = _get_list(confidence, "rules", "confidence")
    whens = []
    for idx, r in enumerate(rules):
        if not isinstance(r, dict):
            _die(f"invalid model config at confidence.rules[{idx}]: expected object: {config_path}")
        when = r.get("when")
        conf = r.get("confidence")
        if when not in ("missing_inputs_nonempty", "touch_set_contains_prefix_entries", "otherwise"):
            _die(f"invalid model config at confidence.rules[{idx}].when: unexpected value: {config_path}")
        if conf not in ("high", "low"):
            _die(f"invalid model config at confidence.rules[{idx}].confidence: expected 'high'|'low': {config_path}")
        whens.append(when)
    for required_when in ("missing_inputs_nonempty", "touch_set_contains_prefix_entries"):
        if required_when not in whens:
            _die(f"invalid model config at confidence.rules: missing {required_when!r} rule: {config_path}")
    if "otherwise" not in whens:
        _die(f"invalid model config at confidence.rules: missing 'otherwise' rule: {config_path}")

    split_triggers = _get_obj(cfg, "split_triggers", "split_triggers")
    adr_candidate = split_triggers.get("adr_candidate")
    if not isinstance(adr_candidate, list) or not adr_candidate:
        _die(f"invalid model config at split_triggers.adr_candidate: expected non-empty array: {config_path}")
    for idx, r in enumerate(adr_candidate):
        if not isinstance(r, dict):
            _die(f"invalid model config at split_triggers.adr_candidate[{idx}]: expected object: {config_path}")
        if not isinstance(r.get("id"), str) or not r["id"]:
            _die(f"invalid model config at split_triggers.adr_candidate[{idx}].id: expected non-empty string: {config_path}")
        if not isinstance(r.get("when"), str) or not r["when"]:
            _die(
                f"invalid model config at split_triggers.adr_candidate[{idx}].when: expected non-empty string: {config_path}"
            )

    prefix_expansion = _get_obj(cfg, "prefix_expansion", "prefix_expansion")
    enabled = prefix_expansion.get("enabled_by_default")
    if not isinstance(enabled, bool):
        _die(f"invalid model config at prefix_expansion.enabled_by_default: expected boolean: {config_path}")
    _get_number(prefix_expansion, "expand_discount", "prefix_expansion")
    _get_int(prefix_expansion, "expand_cap", "prefix_expansion")


def _load_model_config(repo_root: Path) -> dict[str, Any] | None:
    """
    Model config (CONTRACT-2):
      docs/project_management/system/schemas/work_lift_model.v1.json

    If missing, the script uses baked-in defaults (D7–D9).
    """
    p = repo_root / "docs/project_management/system/schemas/work_lift_model.v1.json"
    if not p.exists():
        return None
    try:
        cfg = json.loads(p.read_text(encoding="utf-8"))
    except Exception as e:
        _die(f"failed to parse model config JSON: {p}: {e}")
    _validate_model_config(cfg, config_path=p)
    return cfg


def _baked_model_config_v1() -> dict[str, Any]:
    return {
        "selection": {
            "supported_model_versions": [1],
            "default_model_version": 1,
            "vector_model_version_policy": "require_equal",
        },
        "weights": {
            "touch": {
                "create_files": 3.0,
                "edit_files": 2.0,
                "delete_files": 1.0,
                "deprecate_files": 1.0,
                "crates_touched": 4.0,
                "boundary_crossings": 3.0,
            },
            "contract": {
                "cli_flags": 3.0,
                "config_keys": 3.0,
                "exit_codes": 4.0,
                "file_formats": 5.0,
                "behavior_deltas_blowup": 10.0,
            },
            "qa": {"new_test_files": 2.0, "new_test_cases": 1.0},
            "docs": {"new_docs_files": 2.0},
            "ops": {"new_smoke_steps": 3.0, "ci_changes": 3.0},
        },
        "risk_multipliers": {
            "cross_platform": 1.15,
            "security_sensitive": 1.20,
            "concurrency_or_ordering": 1.15,
            "migration_or_backfill": 1.25,
        },
        "unknowns_add": {"unknowns_high_multiplier": 2.0},
        "rounding": {"score": "ceil"},
        "estimated_slices": {"divisor": 12.0, "min": 1, "rounding": "ceil"},
        "confidence": {
            "rules": [
                {"when": "missing_inputs_nonempty", "confidence": "low"},
                {"when": "touch_set_contains_prefix_entries", "confidence": "low"},
                {"when": "otherwise", "confidence": "high"},
            ]
        },
        "split_triggers": {
            "adr_candidate": [
                {"id": "split_required:behavior_deltas>1", "when": "contract.behavior_deltas > 1"},
                {"id": "likely_split:crates_touched>2", "when": "touch.crates_touched > 2"},
                {"id": "likely_split:touch_files_sum>12", "when": "touch.create_files + touch.edit_files + touch.delete_files > 12"},
                {"id": "likely_split:contract_surface_sum>4", "when": "contract.cli_flags + contract.config_keys + contract.exit_codes + contract.file_formats > 4"},
                {"id": "likely_split:lift_score>24", "when": "lift_score > 24"},
                {"id": "split_required:estimated_slices>3", "when": "estimated_slices > 3"},
            ]
        },
        "prefix_expansion": {"enabled_by_default": True, "expand_discount": float(EXPAND_DISCOUNT), "expand_cap": int(EXPAND_CAP)},
    }


def _select_model_version(vector: dict[str, Any], cfg: dict[str, Any] | None) -> tuple[int, dict[str, Any]]:
    """
    Returns (selected_model_version, derived_model_selection_obj).
    """
    v_raw = vector.get("model_version", None)

    if cfg is None:
        if v_raw is None:
            return (1, {"selected_model_version": 1, "source": "baked_in_default", "policy": None})
        if not _is_int(v_raw):
            _die("lift vector model_version must be an integer when present")
        selected = int(v_raw)
        if selected != 1:
            _die(f"unsupported model_version (config missing; only v1 supported): {selected}")
        return (selected, {"selected_model_version": selected, "source": "vector_field", "policy": None})

    selection = cfg.get("selection") or {}
    default_version = selection.get("default_model_version", 1)
    supported_versions = selection.get("supported_model_versions", [1])
    policy = selection.get("vector_model_version_policy", "require_equal")

    if v_raw is None:
        if not _is_int(default_version):
            _die("invalid model config selection.default_model_version (expected int)")
        selected = int(default_version)
        source = "config_default"
    else:
        if not _is_int(v_raw):
            _die("lift vector model_version must be an integer when present")
        selected = int(v_raw)
        source = "vector_field"

    if policy == "require_equal" and v_raw is not None:
        if not _is_int(default_version):
            _die("invalid model config selection.default_model_version (expected int)")
        if selected != int(default_version):
            _die(
                f"lift vector model_version={selected} does not match model config default_model_version={int(default_version)}"
            )
    if not isinstance(supported_versions, list) or not all(_is_int(x) for x in supported_versions):
        _die("invalid model config selection.supported_model_versions (expected array<int>)")
    if selected not in [int(x) for x in supported_versions]:
        _die(f"unsupported model_version selected: {selected}")

    return (
        selected,
        {"selected_model_version": selected, "source": source, "policy": str(policy)},
    )


def _extract_lift_json_block(text: str, path: Path) -> dict[str, Any]:
    if PM_LIFT_BEGIN not in text or PM_LIFT_END not in text:
        _die(f"missing lift markers in {path} (expected {PM_LIFT_BEGIN} ... {PM_LIFT_END})")
    start = text.index(PM_LIFT_BEGIN) + len(PM_LIFT_BEGIN)
    end = text.index(PM_LIFT_END, start)
    region = text[start:end]

    # Extract first ```json fenced block within the region.
    m = re.search(r"```json\s*(\{.*?\})\s*```", region, flags=re.DOTALL)
    if not m:
        _die(f"lift markers present but no ```json {{...}}``` block found in {path}")
    raw = m.group(1)
    try:
        data = json.loads(raw)
    except json.JSONDecodeError as e:
        _die(f"invalid lift JSON block in {path}: {e}")
    if not isinstance(data, dict):
        _die(f"lift JSON block must be an object: {path}")
    return data


def _num(value: Any, field: str, missing: list[str]) -> float:
    if value is None:
        missing.append(field)
        return 0.0
    if isinstance(value, bool):
        _die(f"{field} must be a number, got bool")
    if isinstance(value, (int, float)):
        return float(value)
    _die(f"{field} must be a number or null, got {type(value).__name__}")
    return 0.0


def _bool(value: Any, field: str) -> bool:
    if value is None:
        return False
    if isinstance(value, bool):
        return value
    _die(f"{field} must be a boolean, got {type(value).__name__}")
    return False


def _tokenize_when_expr(expr: str) -> list[tuple[str, str]]:
    tokens: list[tuple[str, str]] = []
    i = 0
    while i < len(expr):
        c = expr[i]
        if c.isspace():
            i += 1
            continue
        if c.isdigit():
            j = i + 1
            while j < len(expr) and expr[j].isdigit():
                j += 1
            tokens.append(("NUMBER", expr[i:j]))
            i = j
            continue
        if c.isalpha() or c == "_":
            j = i + 1
            while j < len(expr) and (expr[j].isalnum() or expr[j] in "._"):
                j += 1
            tokens.append(("IDENT", expr[i:j]))
            i = j
            continue
        if expr.startswith(">=", i) or expr.startswith("<=", i) or expr.startswith("==", i) or expr.startswith("!=", i):
            tokens.append(("OP", expr[i : i + 2]))
            i += 2
            continue
        if c in {"+", ">", "<"}:
            tokens.append(("OP", c))
            i += 1
            continue
        raise ValueError(f"unsupported character {c!r}")
    return tokens


def _parse_sum(tokens: list[tuple[str, str]], idx: int, env: dict[str, float]) -> tuple[float, int]:
    def _term(tok: tuple[str, str]) -> float:
        t, v = tok
        if t == "NUMBER":
            return float(int(v))
        if t == "IDENT":
            if v not in env:
                raise ValueError(f"unknown identifier {v!r}")
            return float(env[v])
        raise ValueError(f"unexpected token {tok!r}")

    if idx >= len(tokens):
        raise ValueError("unexpected end of expression")
    left = _term(tokens[idx])
    idx += 1
    while idx < len(tokens) and tokens[idx] == ("OP", "+"):
        idx += 1
        if idx >= len(tokens):
            raise ValueError("expected term after '+'")
        left += _term(tokens[idx])
        idx += 1
    return (left, idx)


def _eval_trigger_when(expr: str, env: dict[str, float]) -> bool:
    tokens = _tokenize_when_expr(expr)
    left, idx = _parse_sum(tokens, 0, env)
    if idx == len(tokens):
        raise ValueError("missing comparison operator")
    if idx >= len(tokens) or tokens[idx][0] != "OP" or tokens[idx][1] not in {">", ">=", "<", "<=", "==", "!="}:
        raise ValueError("unsupported comparison operator")
    op = tokens[idx][1]
    idx += 1
    right, idx = _parse_sum(tokens, idx, env)
    if idx != len(tokens):
        raise ValueError("unexpected trailing tokens")

    if op == ">":
        return left > right
    if op == ">=":
        return left >= right
    if op == "<":
        return left < right
    if op == "<=":
        return left <= right
    if op == "==":
        return left == right
    if op == "!=":
        return left != right
    raise ValueError(f"unsupported comparison operator: {op!r}")


def _compute_lift(
    vector: dict[str, Any],
    *,
    model_config: dict[str, Any] | None = None,
    touch_scoring_overrides: dict[str, float] | None = None,
    touch_set_contains_prefix_entries: bool = False,
) -> LiftResult:
    touch = vector.get("touch") or {}
    contract = vector.get("contract") or {}
    qa = vector.get("qa") or {}
    docs = vector.get("docs") or {}
    ops = vector.get("ops") or {}
    risk = vector.get("risk") or {}

    if not all(isinstance(x, dict) for x in (touch, contract, qa, docs, ops, risk)):
        _die("lift vector sections touch/contract/qa/docs/ops/risk must be objects")

    cfg = model_config or _baked_model_config_v1()
    model_version, model_selection = _select_model_version(vector, model_config)

    missing: list[str] = []

    def _num_override(section: str, key: str) -> float:
        if section == "touch" and touch_scoring_overrides is not None and key in touch_scoring_overrides:
            return float(touch_scoring_overrides[key])
        return _num(touch.get(key), f"touch.{key}", missing) if section == "touch" else 0.0

    create_files = _num_override("touch", "create_files")
    edit_files = _num_override("touch", "edit_files")
    delete_files = _num_override("touch", "delete_files")
    deprecate_files = _num_override("touch", "deprecate_files")
    crates_touched = _num_override("touch", "crates_touched")
    boundary_crossings = _num_override("touch", "boundary_crossings")

    cli_flags = _num(contract.get("cli_flags"), "contract.cli_flags", missing)
    config_keys = _num(contract.get("config_keys"), "contract.config_keys", missing)
    exit_codes = _num(contract.get("exit_codes"), "contract.exit_codes", missing)
    file_formats = _num(contract.get("file_formats"), "contract.file_formats", missing)
    behavior_deltas = _num(contract.get("behavior_deltas"), "contract.behavior_deltas", missing)

    new_test_files = _num(qa.get("new_test_files"), "qa.new_test_files", missing)
    new_test_cases = _num(qa.get("new_test_cases"), "qa.new_test_cases", missing)
    new_docs_files = _num(docs.get("new_docs_files"), "docs.new_docs_files", missing)
    new_smoke_steps = _num(ops.get("new_smoke_steps"), "ops.new_smoke_steps", missing)
    ci_changes = _num(ops.get("ci_changes"), "ops.ci_changes", missing)

    cross_platform = _bool(risk.get("cross_platform"), "risk.cross_platform")
    security_sensitive = _bool(risk.get("security_sensitive"), "risk.security_sensitive")
    concurrency_or_ordering = _bool(risk.get("concurrency_or_ordering"), "risk.concurrency_or_ordering")
    migration_or_backfill = _bool(risk.get("migration_or_backfill"), "risk.migration_or_backfill")
    unknowns_high = _num(risk.get("unknowns_high"), "risk.unknowns_high", missing)

    weights = cfg.get("weights") or {}
    w_touch = (weights.get("touch") or {}) if isinstance(weights.get("touch"), dict) else {}
    w_contract = (weights.get("contract") or {}) if isinstance(weights.get("contract"), dict) else {}
    w_qa = (weights.get("qa") or {}) if isinstance(weights.get("qa"), dict) else {}
    w_docs = (weights.get("docs") or {}) if isinstance(weights.get("docs"), dict) else {}
    w_ops = (weights.get("ops") or {}) if isinstance(weights.get("ops"), dict) else {}

    base_breakdown: dict[str, dict[str, float]] = {
        "touch": {},
        "contract": {},
        "qa": {},
        "docs": {},
        "ops": {},
    }

    def _w(sec: dict[str, Any], k: str) -> float:
        v = sec.get(k)
        if not _is_number(v):
            _die(f"invalid model config weight: weights.*.{k} must be a number")
        return float(v)

    base = 0.0
    for k, val in (
        ("create_files", create_files),
        ("edit_files", edit_files),
        ("delete_files", delete_files),
        ("deprecate_files", deprecate_files),
        ("crates_touched", crates_touched),
        ("boundary_crossings", boundary_crossings),
    ):
        if k in w_touch:
            contrib = _w(w_touch, k) * float(val)
            base_breakdown["touch"][k] = contrib
            base += contrib

    for k, val in (
        ("cli_flags", cli_flags),
        ("config_keys", config_keys),
        ("exit_codes", exit_codes),
        ("file_formats", file_formats),
    ):
        if k in w_contract:
            contrib = _w(w_contract, k) * float(val)
            base_breakdown["contract"][k] = contrib
            base += contrib

    blowup_w = _w(w_contract, "behavior_deltas_blowup")
    blowup_contrib = blowup_w * max(0.0, float(behavior_deltas) - 1.0)
    base_breakdown["contract"]["behavior_deltas_blowup"] = blowup_contrib
    base += blowup_contrib

    for k, val in (("new_test_files", new_test_files), ("new_test_cases", new_test_cases)):
        if k in w_qa:
            contrib = _w(w_qa, k) * float(val)
            base_breakdown["qa"][k] = contrib
            base += contrib

    if "new_docs_files" in w_docs:
        contrib = _w(w_docs, "new_docs_files") * float(new_docs_files)
        base_breakdown["docs"]["new_docs_files"] = contrib
        base += contrib

    for k, val in (("new_smoke_steps", new_smoke_steps), ("ci_changes", ci_changes)):
        if k in w_ops:
            contrib = _w(w_ops, k) * float(val)
            base_breakdown["ops"][k] = contrib
            base += contrib

    m = 1.0
    risk_multipliers = cfg.get("risk_multipliers") or {}
    if not isinstance(risk_multipliers, dict):
        _die("invalid model config risk_multipliers: expected object")

    def _mult(k: str) -> float:
        v = risk_multipliers.get(k)
        if not _is_number(v):
            _die(f"invalid model config risk_multipliers.{k}: expected number")
        return float(v)

    if cross_platform:
        m *= _mult("cross_platform")
    if security_sensitive:
        m *= _mult("security_sensitive")
    if concurrency_or_ordering:
        m *= _mult("concurrency_or_ordering")
    if migration_or_backfill:
        m *= _mult("migration_or_backfill")

    unknowns_add = cfg.get("unknowns_add") or {}
    if not isinstance(unknowns_add, dict):
        _die("invalid model config unknowns_add: expected object")
    unknowns_mult = unknowns_add.get("unknowns_high_multiplier")
    if not _is_number(unknowns_mult):
        _die("invalid model config unknowns_add.unknowns_high_multiplier: expected number")
    unknowns_contrib = float(unknowns_mult) * float(unknowns_high)

    score_unrounded = (base * m) + unknowns_contrib
    rounding = (cfg.get("rounding") or {}) if isinstance(cfg.get("rounding"), dict) else {}
    if rounding.get("score") != "ceil":
        _die("invalid model config rounding.score: expected 'ceil'")
    score = math.ceil(score_unrounded)

    est = cfg.get("estimated_slices") or {}
    if not isinstance(est, dict):
        _die("invalid model config estimated_slices: expected object")
    divisor = est.get("divisor")
    est_min = est.get("min")
    if not _is_number(divisor) or float(divisor) <= 0.0:
        _die("invalid model config estimated_slices.divisor: expected number > 0")
    if not _is_int(est_min) or int(est_min) < 1:
        _die("invalid model config estimated_slices.min: expected int >= 1")
    if est.get("rounding") != "ceil":
        _die("invalid model config estimated_slices.rounding: expected 'ceil'")
    estimated_slices = max(int(est_min), math.ceil(float(score) / float(divisor)))

    env: dict[str, float] = {
        "touch.create_files": float(create_files),
        "touch.edit_files": float(edit_files),
        "touch.delete_files": float(delete_files),
        "touch.deprecate_files": float(deprecate_files),
        "touch.crates_touched": float(crates_touched),
        "touch.boundary_crossings": float(boundary_crossings),
        "contract.cli_flags": float(cli_flags),
        "contract.config_keys": float(config_keys),
        "contract.exit_codes": float(exit_codes),
        "contract.file_formats": float(file_formats),
        "contract.behavior_deltas": float(behavior_deltas),
        "lift_score": float(score),
        "estimated_slices": float(estimated_slices),
    }

    triggers: list[str] = []
    split_triggers = cfg.get("split_triggers") or {}
    if not isinstance(split_triggers, dict):
        _die("invalid model config split_triggers: expected object")
    rules = split_triggers.get("adr_candidate") or []
    if not isinstance(rules, list):
        _die("invalid model config split_triggers.adr_candidate: expected array")
    for r in rules:
        if not isinstance(r, dict):
            _die("invalid model config split_triggers.adr_candidate entry: expected object")
        rid = r.get("id")
        expr = r.get("when")
        if not isinstance(rid, str) or not isinstance(expr, str):
            _die("invalid model config split_triggers.adr_candidate entry: expected {id, when} strings")
        try:
            ok = _eval_trigger_when(expr, env)
        except ValueError as e:
            _die(f"failed to evaluate trigger rule {rid!r}: {expr!r}: {e}")
        if ok:
            triggers.append(rid)

    confidence_rules = (cfg.get("confidence") or {}) if isinstance(cfg.get("confidence"), dict) else {}
    rules_list = confidence_rules.get("rules") or []
    if not isinstance(rules_list, list):
        _die("invalid model config confidence.rules: expected array")

    missing_sorted = sorted(set(missing))
    for f in missing_sorted:
        triggers.append(f"missing_inputs:{f}")

    confidence = "high"
    for r in rules_list:
        if not isinstance(r, dict):
            _die("invalid model config confidence.rules entry: expected object")
        when = r.get("when")
        conf = r.get("confidence")
        if when == "missing_inputs_nonempty":
            if missing_sorted:
                confidence = str(conf)
                break
        elif when == "touch_set_contains_prefix_entries":
            if touch_set_contains_prefix_entries:
                confidence = str(conf)
                break
        elif when == "otherwise":
            confidence = str(conf)
            break
        else:
            _die(f"invalid model config confidence.rules.when: {when!r}")
    if confidence not in ("high", "low"):
        _die("invalid model config confidence.rules: confidence must be 'high'|'low'")

    derived = {
        "base_points": base,
        "risk_multiplier": m,
        "model_selection": model_selection,
        "base_points_breakdown": base_breakdown,
        "unknowns_add": {
            "unknowns_high_multiplier": float(unknowns_mult),
            "unknowns_high": float(unknowns_high),
            "contribution": float(unknowns_contrib),
        },
        "score_unrounded": float(score_unrounded),
        "estimated_slices_mapping": {"divisor": float(divisor), "min": int(est_min), "rounding": "ceil"},
    }

    return LiftResult(
        model_version=int(model_version),
        lift_score=int(score),
        estimated_slices=int(estimated_slices),
        confidence=confidence,
        triggers=sorted(set(triggers)),
        missing_inputs=missing_sorted,
        vector=vector,
        derived=derived,
    )


def _schema_type_name(value: Any) -> str:
    if value is None:
        return "null"
    if isinstance(value, bool):
        return "boolean"
    if _is_int(value):
        return "integer"
    if isinstance(value, float):
        return "number"
    if isinstance(value, str):
        return "string"
    if isinstance(value, dict):
        return "object"
    if isinstance(value, list):
        return "array"
    return type(value).__name__


def _schema_allows_type(value: Any, typ: str) -> bool:
    if typ == "null":
        return value is None
    if typ == "object":
        return isinstance(value, dict)
    if typ == "array":
        return isinstance(value, list)
    if typ == "string":
        return isinstance(value, str)
    if typ == "boolean":
        return isinstance(value, bool)
    if typ == "integer":
        return _is_int(value)
    if typ == "number":
        return _is_number(value)
    return False


def _validate_against_schema(value: Any, schema: dict[str, Any], pointer: str, errors: list[str]) -> None:
    if not isinstance(schema, dict):
        errors.append(f"{pointer or '/'}: invalid schema node (expected object)")
        return

    if "const" in schema:
        if value != schema["const"]:
            errors.append(
                f"{pointer or '/'}: const violation (expected {schema['const']!r}, got {value!r})"
            )
            return

    typ = schema.get("type")
    allowed_types: list[str] = []
    if isinstance(typ, str):
        allowed_types = [typ]
    elif isinstance(typ, list) and all(isinstance(x, str) for x in typ):
        allowed_types = list(typ)
    elif typ is None:
        allowed_types = []
    else:
        errors.append(f"{pointer or '/'}: schema type must be string or array of strings")
        return

    if allowed_types:
        if not any(_schema_allows_type(value, t) for t in allowed_types):
            errors.append(
                f"{pointer or '/'}: type violation (expected {allowed_types!r}, got {_schema_type_name(value)})"
            )
            return
        if value is None:
            return

    if "minimum" in schema and _is_number(value):
        minv = schema["minimum"]
        if _is_number(minv) and float(value) < float(minv):
            errors.append(f"{pointer or '/'}: minimum violation (expected >= {minv}, got {value})")

    if (schema.get("type") == "object") or ("object" in allowed_types and isinstance(value, dict)):
        if not isinstance(value, dict):
            errors.append(f"{pointer or '/'}: type violation (expected object)")
            return
        props = schema.get("properties") or {}
        if not isinstance(props, dict):
            errors.append(f"{pointer or '/'}: schema properties must be object")
            return
        req = schema.get("required") or []
        if req is not None and not (isinstance(req, list) and all(isinstance(x, str) for x in req)):
            errors.append(f"{pointer or '/'}: schema required must be array<string>")
            return
        for k in req:
            if k not in value:
                errors.append(f"{pointer or '/'}: missing required key {k!r}")
        addl = schema.get("additionalProperties", None)
        for k, v in value.items():
            if k in props:
                _validate_against_schema(v, props[k], f"{pointer}/{k}", errors)
            else:
                if addl is False:
                    errors.append(f"{pointer or '/'}: unknown key {k!r} (additionalProperties=false)")


def _validate_lift_vector_against_schema(repo_root: Path, vector: dict[str, Any], *, source_path: Path) -> None:
    schema_path = repo_root / "docs/project_management/system/schemas/work_lift_vector.schema.json"
    if schema_path.exists():
        try:
            schema = json.loads(schema_path.read_text(encoding="utf-8"))
        except Exception as e:
            _die(f"failed to parse lift vector schema JSON: {schema_path}: {e}")
        errors: list[str] = []
        _validate_against_schema(vector, schema, "", errors)
        if errors:
            msg = "\n".join(f"- {e}" for e in errors)
            _die(f"lift vector schema validation failed for {source_path} (schema={schema_path}):\n{msg}")
        return

    # Schema missing: conservative structural validation with actionable guidance.
    if not isinstance(vector, dict):
        _die(f"lift JSON block must be an object: {source_path} (schema missing: expected {schema_path})")

    allowed_root = {"model_version", "touch", "contract", "qa", "docs", "ops", "risk", "notes"}
    unknown_root = sorted(set(vector.keys()) - allowed_root)
    if unknown_root:
        _die(
            f"lift vector contains unknown top-level keys {unknown_root!r} (schema missing: expected {schema_path})"
        )

    for k in ("touch", "contract", "qa", "docs", "ops", "risk"):
        v = vector.get(k, None)
        if not isinstance(v, dict):
            _die(f"lift vector missing or invalid section {k!r} (expected object) (schema missing: {schema_path})")
    if not isinstance(vector.get("notes", None), str):
        _die(f"lift vector missing/invalid notes (expected string) (schema missing: {schema_path})")

    def _check_int_null(section: str, key: str, *, minv: int | None = None) -> None:
        v = (vector.get(section) or {}).get(key, "__missing__")
        if v == "__missing__":
            return
        if v is None:
            return
        if isinstance(v, bool) or not _is_int(v):
            _die(f"lift vector {section}.{key} must be integer or null (schema missing: {schema_path})")
        if minv is not None and int(v) < minv:
            _die(f"lift vector {section}.{key} must be >= {minv} (schema missing: {schema_path})")

    for key in ("create_files", "edit_files", "delete_files", "deprecate_files", "crates_touched", "boundary_crossings"):
        _check_int_null("touch", key, minv=0)
    for key in ("cli_flags", "config_keys", "exit_codes", "file_formats"):
        _check_int_null("contract", key, minv=0)
    _check_int_null("contract", "behavior_deltas", minv=1)
    for key in ("new_test_files", "new_test_cases"):
        _check_int_null("qa", key, minv=0)
    _check_int_null("docs", "new_docs_files", minv=0)
    for key in ("new_smoke_steps", "ci_changes"):
        _check_int_null("ops", key, minv=0)
    _check_int_null("risk", "unknowns_high", minv=0)

    for key in ("cross_platform", "security_sensitive", "concurrency_or_ordering", "migration_or_backfill"):
        v = (vector.get("risk") or {}).get(key, "__missing__")
        if v == "__missing__":
            continue
        if not isinstance(v, bool):
            _die(f"lift vector risk.{key} must be boolean (schema missing: {schema_path})")


def _run_validate_impact_map_emit_json(feature_dir: Path) -> dict[str, Any]:
    script_dir = Path(__file__).resolve().parent
    validator = script_dir / "validate_impact_map.py"
    cmd = [sys.executable, str(validator), "--feature-dir", str(feature_dir), "--emit-json"]
    try:
        res = subprocess.run(cmd, text=True, capture_output=True, check=False)
    except Exception as e:
        _die(f"failed to run validate_impact_map.py: {e}")
    if res.returncode != 0:
        _die(f"validate_impact_map.py failed:\n{res.stderr.strip() or res.stdout.strip()}")
    try:
        data = json.loads(res.stdout)
    except json.JSONDecodeError as e:
        # validate_impact_map.py prints warnings to stderr; stdout should be JSON-only.
        _die(f"validate_impact_map.py did not return JSON on stdout: {e}")
    if not isinstance(data, dict):
        _die("validate_impact_map.py JSON must be an object")
    return data


def _expand_prefix(repo_root: Path, prefix: str) -> list[str]:
    prefix = prefix.strip()
    if not prefix.endswith("/"):
        return []
    try:
        out = subprocess.check_output(
            ["git", "-C", str(repo_root), "ls-files", prefix],
            text=True,
            stderr=subprocess.STDOUT,
        )
    except Exception:
        return []
    files = [line.strip() for line in out.splitlines() if line.strip()]
    return [f for f in files if f.startswith(prefix)]


def _infer_crates_touched(paths: list[str]) -> int:
    crates: set[str] = set()
    for p in paths:
        parts = p.split("/")
        if len(parts) >= 2 and parts[0] == "crates":
            crates.add(parts[1])
    return len(crates)


def cmd_from_intake(intake_path: Path, emit_json: bool) -> int:
    try:
        text = intake_path.read_text(encoding="utf-8")
    except Exception as e:
        _die(f"failed to read intake file {intake_path}: {e}")
    vector = _extract_lift_json_block(text, intake_path)
    repo_root = _repo_root()
    _validate_lift_vector_against_schema(repo_root, vector, source_path=intake_path)
    model_cfg = _load_model_config(repo_root)
    res = _compute_lift(vector, model_config=model_cfg)
    return _print_result(res, emit_json=emit_json)


def cmd_from_impact_map(feature_dir: Path, emit_json: bool) -> int:
    repo_root = _repo_root()
    allow = _run_validate_impact_map_emit_json(feature_dir)

    for k in ("create", "edit", "deprecate", "delete", "dir_prefixes"):
        if k not in allow:
            _die(f"validate_impact_map.py JSON missing required key: {k!r}")
    for k in ("create", "edit", "deprecate", "delete", "dir_prefixes"):
        v = allow.get(k)
        if not isinstance(v, list) or not all(isinstance(x, str) for x in v):
            _die(f"validate_impact_map.py JSON field {k!r} must be an array of strings")
    for p in allow.get("dir_prefixes") or []:
        if not p.endswith("/"):
            _die(f"validate_impact_map.py JSON dir_prefixes entry must end with '/': {p!r}")

    per_section = {}
    expanded_paths_all: list[str] = []
    raw_paths_all: list[str] = []

    model_cfg = _load_model_config(repo_root)
    cfg_for_prefix = model_cfg or _baked_model_config_v1()
    prefix_cfg = cfg_for_prefix.get("prefix_expansion") or {}
    if not isinstance(prefix_cfg, dict):
        _die("invalid model config prefix_expansion: expected object")
    prefix_enabled = bool(prefix_cfg.get("enabled_by_default", True))
    prefix_discount = float(prefix_cfg.get("expand_discount", float(EXPAND_DISCOUNT)))
    prefix_cap = int(prefix_cfg.get("expand_cap", int(EXPAND_CAP)))

    for sec in ("create", "edit", "deprecate", "delete"):
        items = allow.get(sec) or []
        explicit_files = [p for p in items if not p.endswith("/")]
        prefixes = [p for p in items if p.endswith("/")]

        # Raw count policy: prefix counts as 1 file for the vector.
        raw_count = len(explicit_files) + len(prefixes)

        # Effective count policy (lift-only): per-prefix discounted/capped expansion.
        eff = float(len(explicit_files))
        expansions: dict[str, int] = {}
        for pref in prefixes:
            expanded: list[str] = []
            if prefix_enabled:
                expanded = _expand_prefix(repo_root, pref)
                expanded_paths_all.extend(expanded)
                eff += min(len(expanded), prefix_cap) * prefix_discount
            expansions[pref] = len(expanded)

        per_section[sec] = {
            "explicit_files": len(explicit_files),
            "prefix_entries": len(prefixes),
            "prefix_expanded_counts": expansions,
            "raw_count": raw_count,
            "effective_count": eff,
        }

        raw_paths_all.extend(items)

    # Construct a minimal vector for scoring. Contract/QA/etc. are unknown from impact_map alone.
    touch = {
        "create_files": int(per_section["create"]["raw_count"]),
        "edit_files": int(per_section["edit"]["raw_count"]),
        "delete_files": int(per_section["delete"]["raw_count"]),
        "deprecate_files": int(per_section["deprecate"]["raw_count"]),
        "crates_touched": _infer_crates_touched(raw_paths_all + (expanded_paths_all if prefix_enabled else [])),
        "boundary_crossings": None,
    }
    vector = {
        "model_version": 1,
        "touch": touch,
        "contract": {
            "cli_flags": None,
            "config_keys": None,
            "exit_codes": None,
            "file_formats": None,
            "behavior_deltas": 1,
        },
        "qa": {"new_test_files": None, "new_test_cases": None},
        "docs": {"new_docs_files": None},
        "ops": {"new_smoke_steps": None, "ci_changes": None},
        "risk": {
            "cross_platform": False,
            "security_sensitive": False,
            "concurrency_or_ordering": False,
            "migration_or_backfill": False,
            "unknowns_high": None,
        },
        "notes": "",
    }

    touch_scoring_overrides = {
        "create_files": float(per_section["create"]["effective_count"]),
        "edit_files": float(per_section["edit"]["effective_count"]),
        "delete_files": float(per_section["delete"]["effective_count"]),
        "deprecate_files": float(per_section["deprecate"]["effective_count"]),
    }
    prefix_present = bool(allow.get("dir_prefixes"))
    res = _compute_lift(
        vector,
        model_config=model_cfg,
        touch_scoring_overrides=touch_scoring_overrides,
        touch_set_contains_prefix_entries=prefix_present,
    )
    derived = dict(res.derived)
    derived["impact_map_touch_counts"] = per_section
    derived["touch_effective_for_scoring"] = {
        "create_files": per_section["create"]["effective_count"],
        "edit_files": per_section["edit"]["effective_count"],
        "deprecate_files": per_section["deprecate"]["effective_count"],
        "delete_files": per_section["delete"]["effective_count"],
    }
    derived["prefix_expansion"] = {
        "enabled": bool(prefix_enabled),
        "discount": float(prefix_discount),
        "cap": int(prefix_cap),
    }
    derived["touch_scoring_inputs"] = {
        "raw": {
            "create_files": int(per_section["create"]["raw_count"]),
            "edit_files": int(per_section["edit"]["raw_count"]),
            "deprecate_files": int(per_section["deprecate"]["raw_count"]),
            "delete_files": int(per_section["delete"]["raw_count"]),
        },
        "effective": {
            "create_files": float(per_section["create"]["effective_count"]),
            "edit_files": float(per_section["edit"]["effective_count"]),
            "deprecate_files": float(per_section["deprecate"]["effective_count"]),
            "delete_files": float(per_section["delete"]["effective_count"]),
        },
    }
    res = LiftResult(
        model_version=res.model_version,
        lift_score=res.lift_score,
        estimated_slices=res.estimated_slices,
        confidence=res.confidence,
        triggers=sorted(
            set(res.triggers + (["touch_set_contains_prefix_entries"] if prefix_present else []))
        ),
        missing_inputs=res.missing_inputs,
        vector=res.vector,
        derived=derived,
    )
    return _print_result(res, emit_json=emit_json)


def cmd_from_git_diff(git_range: str, emit_json: bool) -> int:
    repo_root = _repo_root()
    try:
        out = subprocess.check_output(
            ["git", "-C", str(repo_root), "diff", "--name-status", "-M", git_range],
            text=True,
            stderr=subprocess.STDOUT,
        )
    except Exception as e:
        _die(f"git diff failed for range {git_range!r}: {e}")

    create = 0
    edit = 0
    delete = 0
    for line in out.splitlines():
        line = line.strip()
        if not line:
            continue
        parts = line.split("\t")
        status = parts[0]
        if status.startswith("A"):
            create += 1
        elif status.startswith("D"):
            delete += 1
        elif status.startswith("M") or status.startswith("R") or status.startswith("C"):
            edit += 1
        else:
            edit += 1

    vector = {
        "model_version": 1,
        "touch": {
            "create_files": create,
            "edit_files": edit,
            "delete_files": delete,
            "deprecate_files": 0,
            "crates_touched": None,
            "boundary_crossings": None,
        },
        "contract": {
            "cli_flags": None,
            "config_keys": None,
            "exit_codes": None,
            "file_formats": None,
            "behavior_deltas": 1,
        },
        "qa": {"new_test_files": None, "new_test_cases": None},
        "docs": {"new_docs_files": None},
        "ops": {"new_smoke_steps": None, "ci_changes": None},
        "risk": {
            "cross_platform": False,
            "security_sensitive": False,
            "concurrency_or_ordering": False,
            "migration_or_backfill": False,
            "unknowns_high": None,
        },
        "notes": "",
    }

    model_cfg = _load_model_config(repo_root)
    res = _compute_lift(vector, model_config=model_cfg)
    return _print_result(res, emit_json=emit_json)


def _print_result(res: LiftResult, *, emit_json: bool) -> int:
    if emit_json:
        out = {
            "model_version": res.model_version,
            "lift_score": res.lift_score,
            "estimated_slices": res.estimated_slices,
            "confidence": res.confidence,
            "triggers": res.triggers,
            "missing_inputs": res.missing_inputs,
            "vector": res.vector,
            "derived": res.derived,
        }
        print(json.dumps(out, indent=2, sort_keys=True))
        return 0

    print(f"Lift Score (v{res.model_version}): {res.lift_score}")
    print(f"Estimated slices: {res.estimated_slices}")
    print(f"Confidence: {res.confidence}")
    if res.triggers:
        print("Triggers:")
        for t in res.triggers:
            print(f"- {t}")
    if res.missing_inputs:
        print("Missing inputs:")
        for m in res.missing_inputs:
            print(f"- {m}")
    return 0


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="Compute time-free Work Lift score + split triggers.")
    sub = ap.add_subparsers(dest="cmd", required=True)

    ap_intake = sub.add_parser("from-intake", help="Compute lift from a markdown intake/ADR containing a PM_LIFT_VECTOR JSON block.")
    ap_intake.add_argument("--intake", required=True, help="Path to intake/ADR markdown file.")
    ap_intake.add_argument("--emit-json", action="store_true", help="Emit machine JSON summary to stdout.")

    ap_impact = sub.add_parser("from-impact-map", help="Compute lift from a Planning Pack impact_map.md touch set.")
    ap_impact.add_argument("--feature-dir", required=True, help="Planning Pack dir (docs/project_management/packs/<bucket>/<feature>).")
    ap_impact.add_argument("--emit-json", action="store_true", help="Emit machine JSON summary to stdout.")

    ap_diff = sub.add_parser("from-git-diff", help="Compute lift from a git diff range (for calibration).")
    ap_diff.add_argument("--git-range", required=True, help="Git range (e.g. base..head).")
    ap_diff.add_argument("--emit-json", action="store_true", help="Emit machine JSON summary to stdout.")

    args = ap.parse_args(argv)

    if args.cmd == "from-intake":
        return cmd_from_intake(Path(args.intake), emit_json=bool(args.emit_json))
    if args.cmd == "from-impact-map":
        return cmd_from_impact_map(Path(args.feature_dir), emit_json=bool(args.emit_json))
    if args.cmd == "from-git-diff":
        return cmd_from_git_diff(str(args.git_range), emit_json=bool(args.emit_json))

    _die(f"unknown command: {args.cmd}", code=2)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
