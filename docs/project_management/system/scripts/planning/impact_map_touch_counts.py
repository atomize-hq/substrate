from __future__ import annotations

from typing import Any, Callable


SECTIONS = ("create", "edit", "deprecate", "delete")


def compute_impact_map_touch_counts(
    impact_map_emit_json: dict[str, Any],
    *,
    expand_prefix: Callable[[str], list[str]],
    prefix_enabled: bool = True,
    expand_discount: float = 0.20,
    expand_cap: int = 10,
) -> dict[str, Any]:
    """
    Compute raw and effective touch counts from CONTRACT-4 impact_map emit JSON.

    This function is pure: it performs no filesystem access or subprocess calls.
    All prefix expansion is delegated to the injected `expand_prefix(prefix)`.
    """
    if not isinstance(impact_map_emit_json, dict):
        raise TypeError("impact_map_emit_json must be a dict")
    if not isinstance(prefix_enabled, bool):
        raise TypeError("prefix_enabled must be a bool")
    if not isinstance(expand_discount, (int, float)) or isinstance(expand_discount, bool):
        raise TypeError("expand_discount must be a number")
    if not isinstance(expand_cap, int) or isinstance(expand_cap, bool):
        raise TypeError("expand_cap must be an int")
    if expand_cap < 0:
        raise ValueError("expand_cap must be >= 0")

    per_section: dict[str, dict[str, Any]] = {}
    dir_prefixes: set[str] = set()

    for sec in SECTIONS:
        items = impact_map_emit_json.get(sec, [])
        if items is None:
            items = []
        if not isinstance(items, list) or not all(isinstance(x, str) for x in items):
            raise TypeError(f"impact_map_emit_json[{sec!r}] must be a list[str]")

        explicit = [t for t in items if not t.endswith("/")]
        prefixes = sorted(t for t in items if t.endswith("/"))

        dir_prefixes.update(prefixes)

        prefix_expanded_counts: dict[str, int] = {}
        effective = float(len(explicit))

        for pref in prefixes:
            if not prefix_enabled:
                prefix_expanded_counts[pref] = 0
                continue
            expanded = expand_prefix(pref)
            if not isinstance(expanded, list) or not all(isinstance(x, str) for x in expanded):
                raise TypeError("expand_prefix(prefix) must return list[str]")
            expanded_count = len(expanded)
            prefix_expanded_counts[pref] = expanded_count
            effective += min(expanded_count, expand_cap) * float(expand_discount)

        per_section[sec] = {
            "explicit_files": int(len(explicit)),
            "prefix_entries": int(len(prefixes)),
            "prefix_expanded_counts": dict(prefix_expanded_counts),
            "raw_count": int(len(explicit) + len(prefixes)),
            "effective_count": float(effective),
        }

    dir_prefixes_sorted = sorted(dir_prefixes)
    return {
        "per_section": per_section,
        "dir_prefixes": dir_prefixes_sorted,
        "prefix_present": bool(dir_prefixes_sorted),
    }

