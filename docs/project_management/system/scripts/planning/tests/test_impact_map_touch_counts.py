import unittest

from docs.project_management.system.scripts.planning.impact_map_touch_counts import compute_impact_map_touch_counts


class TestImpactMapTouchCounts(unittest.TestCase):
    def test_explicit_only(self) -> None:
        calls: list[str] = []

        def expand_prefix(prefix: str) -> list[str]:
            calls.append(prefix)
            return []

        out = compute_impact_map_touch_counts(
            {"create": ["a.txt", "b.txt"], "edit": [], "deprecate": [], "delete": []},
            expand_prefix=expand_prefix,
        )
        self.assertEqual(calls, [])
        self.assertEqual(out["dir_prefixes"], [])
        self.assertEqual(out["prefix_present"], False)
        self.assertEqual(out["per_section"]["create"]["raw_count"], 2)
        self.assertEqual(out["per_section"]["create"]["effective_count"], 2.0)
        self.assertEqual(out["per_section"]["create"]["prefix_entries"], 0)

    def test_prefix_only_three_matches(self) -> None:
        def expand_prefix(prefix: str) -> list[str]:
            self.assertEqual(prefix, "p/")
            return ["x", "y", "z"]

        out = compute_impact_map_touch_counts({"create": ["p/"]}, expand_prefix=expand_prefix)
        self.assertEqual(out["dir_prefixes"], ["p/"])
        self.assertEqual(out["prefix_present"], True)

        create = out["per_section"]["create"]
        self.assertEqual(create["raw_count"], 1)
        self.assertEqual(create["explicit_files"], 0)
        self.assertEqual(create["prefix_entries"], 1)
        self.assertEqual(create["prefix_expanded_counts"], {"p/": 3})
        self.assertAlmostEqual(create["effective_count"], 0.6, places=7)

    def test_cap_binds(self) -> None:
        def expand_prefix(prefix: str) -> list[str]:
            return ["x"] * 100

        out = compute_impact_map_touch_counts({"create": ["p/"]}, expand_prefix=expand_prefix)
        create = out["per_section"]["create"]
        self.assertEqual(create["prefix_expanded_counts"], {"p/": 100})
        self.assertAlmostEqual(create["effective_count"], 2.0, places=7)

    def test_mixed_two_prefixes(self) -> None:
        def expand_prefix(prefix: str) -> list[str]:
            if prefix == "p1/":
                return ["x"]
            if prefix == "p2/":
                return ["x"] * 7
            raise AssertionError(f"unexpected prefix: {prefix}")

        out = compute_impact_map_touch_counts(
            {"create": ["a.txt", "p2/", "p1/"]},
            expand_prefix=expand_prefix,
        )
        self.assertEqual(out["dir_prefixes"], ["p1/", "p2/"])

        create = out["per_section"]["create"]
        self.assertEqual(create["raw_count"], 3)
        self.assertEqual(create["explicit_files"], 1)
        self.assertEqual(create["prefix_entries"], 2)
        self.assertEqual(create["prefix_expanded_counts"], {"p1/": 1, "p2/": 7})
        self.assertAlmostEqual(create["effective_count"], 2.6, places=7)

    def test_prefix_enabled_false_does_not_call_provider(self) -> None:
        calls: list[str] = []

        def expand_prefix(prefix: str) -> list[str]:
            calls.append(prefix)
            return ["x"] * 5

        out = compute_impact_map_touch_counts(
            {"create": ["a.txt", "p/"]},
            expand_prefix=expand_prefix,
            prefix_enabled=False,
        )
        self.assertEqual(calls, [])
        self.assertEqual(out["dir_prefixes"], ["p/"])
        create = out["per_section"]["create"]
        self.assertEqual(create["raw_count"], 2)
        self.assertEqual(create["prefix_expanded_counts"], {"p/": 0})
        self.assertAlmostEqual(create["effective_count"], 1.0, places=7)

    def test_empty_expansion(self) -> None:
        def expand_prefix(prefix: str) -> list[str]:
            return []

        out = compute_impact_map_touch_counts({"create": ["p/"]}, expand_prefix=expand_prefix)
        create = out["per_section"]["create"]
        self.assertEqual(create["prefix_expanded_counts"], {"p/": 0})
        self.assertAlmostEqual(create["effective_count"], 0.0, places=7)

    def test_per_section_always_includes_all_sections(self) -> None:
        def expand_prefix(prefix: str) -> list[str]:
            return []

        out = compute_impact_map_touch_counts({"create": ["a.txt"]}, expand_prefix=expand_prefix)
        self.assertEqual(set(out["per_section"].keys()), {"create", "edit", "deprecate", "delete"})
        self.assertEqual(out["per_section"]["edit"]["raw_count"], 0)
        self.assertEqual(out["per_section"]["deprecate"]["raw_count"], 0)
        self.assertEqual(out["per_section"]["delete"]["raw_count"], 0)


if __name__ == "__main__":
    unittest.main()

