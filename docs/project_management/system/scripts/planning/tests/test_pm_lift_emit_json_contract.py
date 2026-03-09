import json
import subprocess
import sys
import unittest
from pathlib import Path


def _write_text(path: Path, text: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TestPmLiftEmitJsonContract(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.repo_root = Path(__file__).resolve()
        while cls.repo_root != cls.repo_root.parent and not (cls.repo_root / ".git").exists():
            cls.repo_root = cls.repo_root.parent
        if not (cls.repo_root / ".git").exists():
            raise unittest.SkipTest("Not running inside a git repo; pm_lift.py requires repo root discovery via git.")

        cls.tmp_root = cls.repo_root / "target" / "pm_script_tests" / "pm_lift" / "emit_json_contract"
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

    def test_emit_json_success_shape_and_determinism(self) -> None:
        fixture = self.tmp_root / "intake_valid.md"
        text = (
            "# Intake Fixture\n\n"
            "<!-- PM_LIFT_VECTOR:BEGIN -->\n"
            "```json\n"
            "{\n"
            '  "model_version": 1,\n'
            '  "touch": {\n'
            '    "create_files": 0,\n'
            '    "edit_files": 0,\n'
            '    "delete_files": 0,\n'
            '    "deprecate_files": 0,\n'
            '    "crates_touched": null,\n'
            '    "boundary_crossings": 0\n'
            "  },\n"
            '  "contract": {\n'
            '    "cli_flags": 0,\n'
            '    "config_keys": 0,\n'
            '    "exit_codes": 0,\n'
            '    "file_formats": 0,\n'
            '    "behavior_deltas": 1\n'
            "  },\n"
            '  "qa": { "new_test_files": 0, "new_test_cases": 0 },\n'
            '  "docs": { "new_docs_files": 0 },\n'
            '  "ops": { "new_smoke_steps": 0, "ci_changes": 0 },\n'
            '  "risk": {\n'
            '    "cross_platform": false,\n'
            '    "security_sensitive": false,\n'
            '    "concurrency_or_ordering": false,\n'
            '    "migration_or_backfill": false,\n'
            '    "unknowns_high": 0\n'
            "  },\n"
            '  "notes": ""\n'
            "}\n"
            "```\n"
            "<!-- PM_LIFT_VECTOR:END -->\n"
        )
        _write_text(fixture, text)

        res = self._run(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 0, msg=res.stderr)
        self.assertEqual(res.stderr.strip(), "", msg="Expected no stderr on success.")

        out = json.loads(res.stdout)
        self.assertIsInstance(out, dict)

        required = [
            "model_version",
            "lift_score",
            "estimated_slices",
            "confidence",
            "triggers",
            "missing_inputs",
            "vector",
            "derived",
        ]
        for k in required:
            self.assertIn(k, out)

        self.assertIsInstance(out["model_version"], int)
        self.assertIsInstance(out["lift_score"], int)
        self.assertIsInstance(out["estimated_slices"], int)
        self.assertIn(out["confidence"], {"high", "low"})

        self.assertIsInstance(out["triggers"], list)
        self.assertTrue(all(isinstance(x, str) for x in out["triggers"]))
        self.assertIsInstance(out["missing_inputs"], list)
        self.assertTrue(all(isinstance(x, str) for x in out["missing_inputs"]))

        self.assertEqual(out["triggers"], sorted(set(out["triggers"])))
        self.assertEqual(out["missing_inputs"], sorted(set(out["missing_inputs"])))

        self.assertEqual(out["confidence"], "low")
        self.assertEqual(out["missing_inputs"], ["touch.crates_touched"])
        self.assertIn("missing_inputs:touch.crates_touched", out["triggers"])

        self.assertIsInstance(out["vector"], dict)
        self.assertIsInstance(out["derived"], dict)
        self.assertIn("base_points", out["derived"])
        self.assertIn("risk_multiplier", out["derived"])
        self.assertIn("model_selection", out["derived"])
        self.assertIsInstance(out["derived"]["model_selection"], dict)
        self.assertIn("selected_model_version", out["derived"]["model_selection"])
        self.assertIsInstance(out["derived"]["base_points"], (int, float))
        self.assertIsInstance(out["derived"]["risk_multiplier"], (int, float))

    def test_emit_json_runtime_error_stdout_empty_exit_1(self) -> None:
        fixture = self.tmp_root / "intake_missing_markers.md"
        _write_text(fixture, "# Missing markers\n")

        res = self._run(["from-intake", "--intake", str(fixture), "--emit-json"])
        self.assertEqual(res.returncode, 1)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("missing lift markers", res.stderr)

    def test_emit_json_usage_error_exit_2(self) -> None:
        res = self._run(["from-intake", "--emit-json"])
        self.assertEqual(res.returncode, 2)
        self.assertEqual(res.stdout.strip(), "")
        self.assertIn("usage:", res.stderr.lower())


if __name__ == "__main__":
    unittest.main()
