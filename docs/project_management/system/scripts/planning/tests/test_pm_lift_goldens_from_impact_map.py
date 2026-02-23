import json
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


def _write_json(path: Path, obj: object) -> None:
    _write_text(path, json.dumps(obj, indent=2, sort_keys=True) + "\n")


def _impact_map_strict(create_tokens: list[str]) -> str:
    create_lines = "\n".join(f"- `{t}`" for t in create_tokens) if create_tokens else "- None"
    return (
        "# Impact Map Fixture\n\n"
        "## Touch set (explicit)\n\n"
        "### Create\n"
        f"{create_lines}\n\n"
        "### Edit\n"
        "- None\n\n"
        "### Deprecate\n"
        "- None\n\n"
        "### Delete\n"
        "- None\n"
    )


class TestPmLiftGoldensFromImpactMap(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; pm_lift.py requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift" / "from_impact_map"
        cls.tmp_root.mkdir(parents=True, exist_ok=True)

        cls.pm_lift = cls.repo_root / "docs" / "project_management" / "system" / "scripts" / "planning" / "pm_lift.py"
        if not cls.pm_lift.is_file():
            raise unittest.SkipTest("pm_lift.py not found at expected canonical path.")

    def _run(self, args: list[str]) -> subprocess.CompletedProcess[str]:
        cmd = [sys.executable, str(self.pm_lift), *args]
        return subprocess.run(
            cmd,
            text=True,
            capture_output=True,
            check=False,
            cwd=str(self.repo_root),
        )

    def _make_strict_feature_dir(self, name: str, *, create_tokens: list[str] | None, include_impact_map: bool = True) -> Path:
        feature_dir = self.tmp_root / name
        feature_dir.mkdir(parents=True, exist_ok=True)

        _write_json(feature_dir / "tasks.json", {"meta": {"slice_spec_version": 2}})

        if include_impact_map:
            tokens = create_tokens or []
            _write_text(feature_dir / "impact_map.md", _impact_map_strict(tokens))

        return feature_dir

    def test_derived_a_explicit_only_no_prefix_trigger(self) -> None:
        feature_dir = self._make_strict_feature_dir(
            "derived_a",
            create_tokens=["__pm_lift_test__/a.txt", "__pm_lift_test__/b.txt"],
        )
        res = self._run(["from-impact-map", "--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success.")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)
        self.assertEqual(out["lift_score"], 6)
        self.assertEqual(out["estimated_slices"], 1)
        self.assertEqual(out["confidence"], "low")
        self.assertNotIn("touch_set_contains_prefix_entries", set(out["triggers"]))

        derived = out["derived"]
        self.assertIn("impact_map_touch_counts", derived)
        self.assertIn("touch_effective_for_scoring", derived)
        self.assertEqual(derived["impact_map_touch_counts"]["create"]["raw_count"], 2)
        self.assertEqual(derived["touch_effective_for_scoring"]["create_files"], 2.0)

    def test_derived_b_prefix_present_trigger_and_discounted_empty_expansion(self) -> None:
        prefix = "__pm_lift_test__/no_such_prefix/"
        feature_dir = self._make_strict_feature_dir(
            "derived_b",
            create_tokens=["__pm_lift_test__/c.txt", prefix],
        )
        res = self._run(["from-impact-map", "--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success.")

        out = json.loads(res.stdout)
        self.assertEqual(out["lift_score"], 3)
        self.assertEqual(out["estimated_slices"], 1)
        self.assertIn("touch_set_contains_prefix_entries", set(out["triggers"]))

        derived = out["derived"]
        self.assertEqual(derived["impact_map_touch_counts"]["create"]["raw_count"], 2)
        self.assertEqual(derived["impact_map_touch_counts"]["create"]["effective_count"], 1.0)
        self.assertEqual(derived["impact_map_touch_counts"]["create"]["prefix_expanded_counts"][prefix], 0)

    def test_negative_missing_impact_map_bubbles_validator_failure(self) -> None:
        feature_dir = self._make_strict_feature_dir("missing_impact_map", create_tokens=None, include_impact_map=False)
        res = self._run(["from-impact-map", "--feature-dir", str(feature_dir), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("validate_impact_map.py failed", res.stderr)
        self.assertIn("missing required path", res.stderr)


if __name__ == "__main__":
    unittest.main()

